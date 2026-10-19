//! Portable Native Skia and bounded horizontal browser instance selection.
//! P46's Text IR v2 export and CanvasKit glyph-run consumer both support synthetic
//! bold/italic for horizontal LTR, level-zero instances without variations.
//! Typeface construction and actual effect replay are tested by their consumers;
//! this contract keeps exact font-resource proof and unsupported subset rejection.

use rhwp::paint::{
    lower_font_native_glyph_sidecars, EmbeddedFontFace, GlyphRunOrientation, LayerGlyphRunPaint,
    LayerNode, LayerNodeKind, PageLayerTree, PaintOp, TextDirection, TextVariantKind,
    VariationAxisValue, WritingMode,
};
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, TextVariantSelectionReport,
    VariantRejectReason, VariantSelectionBackend,
};
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::TextStyle;

const COLLECTION: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactFaceSmoke.ttc");
const VARIABLE: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");

fn tree(bytes: &[u8], face_index: u32, text: &str) -> PageLayerTree {
    let bbox = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
    let run = TextRunNode {
        text: text.to_owned(),
        style: TextStyle {
            font_family: "exact fixture".into(),
            font_size: 16.0,
            ..Default::default()
        },
        char_shape_id: Some(536),
        para_shape_id: None,
        section_index: Some(0),
        para_index: Some(0),
        char_start: Some(0),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 16.0,
        field_marker: Default::default(),
        layout_positions: Some(vec![0.0, 16.0]),
        display_text: None,
    };
    let mut tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]),
    );
    let report = lower_font_native_glyph_sidecars(
        &mut tree.root,
        &mut tree.resources,
        &[EmbeddedFontFace {
            char_shape_id: 536,
            language_index: 0,
            family: "exact fixture",
            alternate_family: None,
            bytes,
            face_index,
        }],
    );
    assert_eq!(report.emitted_glyph_runs, 1, "{report:?}");
    // Only the GlyphRun contract is under test, not bitmap/SVG alternatives.
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("expected a leaf");
    };
    ops.retain(|op| !matches!(op, PaintOp::GlyphOutline { .. }));
    tree
}

fn glyph(tree: &mut PageLayerTree) -> &mut LayerGlyphRunPaint {
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("expected a leaf");
    };
    ops.iter_mut()
        .find_map(|op| match op {
            PaintOp::GlyphRun { run, .. } => Some(run.as_mut()),
            _ => None,
        })
        .expect("nominal producer GlyphRun")
}

fn selection(tree: &PageLayerTree, backend: VariantSelectionBackend) -> TextVariantSelectionReport {
    analyze_text_variant_selection(
        tree,
        TextVariantSelectionOptions {
            backend,
            ..TextVariantSelectionOptions::canvaskit()
        },
    )
    .into_iter()
    .next()
    .expect("one source-bound variant group")
}

fn assert_glyph_selected(tree: &PageLayerTree, backend: VariantSelectionBackend) {
    let report = selection(tree, backend);
    assert_eq!(
        report.selected_variant_kind,
        Some(TextVariantKind::GlyphRun),
        "{report:?}"
    );
    assert!(!report.fallback_required, "{report:?}");
    assert!(report.rejected_variants.is_empty(), "{report:?}");
}

fn assert_glyph_rejected(
    tree: &PageLayerTree,
    backend: VariantSelectionBackend,
    reason: VariantRejectReason,
) {
    let report = selection(tree, backend);
    assert!(report.fallback_required, "{report:?}");
    assert_eq!(
        report.selected_variant_kind,
        Some(TextVariantKind::TextRun),
        "{report:?}"
    );
    assert_eq!(report.rejected_variants.len(), 1, "{report:?}");
    assert_eq!(
        report.rejected_variants[0].variant_kind,
        TextVariantKind::GlyphRun,
        "{report:?}"
    );
    assert!(
        report.rejected_variants[0].reasons.contains(&reason),
        "{report:?}"
    );
}

#[test]
fn native_and_browser_collection_contract_keeps_exact_faces_and_rejects_invalid_indexes() {
    let mut tree = tree(COLLECTION, 1, "\u{e104}");
    for backend in [
        VariantSelectionBackend::NativeSkia,
        VariantSelectionBackend::CanvasKitBrowser,
    ] {
        assert_glyph_selected(&tree, backend);
    }
    tree.resources.font_resources_mut().faces[0].face_index = 999;
    for backend in [
        VariantSelectionBackend::NativeSkia,
        VariantSelectionBackend::CanvasKitBrowser,
    ] {
        assert_glyph_rejected(&tree, backend, VariantRejectReason::FaceIndexUnsupported);
    }
}

#[test]
fn native_and_browser_support_horizontal_synthetic_instances_with_digest_proof() {
    for (bold, italic) in [(true, false), (false, true), (true, true)] {
        let mut tree = tree(COLLECTION, 1, "\u{e104}");
        glyph(&mut tree).shape_key.font_instance.synthetic_bold = bold;
        glyph(&mut tree).shape_key.font_instance.synthetic_italic = italic;
        // P46: text_v2::glyph_run_is_strict permits this subset, and
        // canvaskit/glyph-run-fonts.ts applies setEmbolden/setSkewX after proof.
        for backend in [
            VariantSelectionBackend::NativeSkia,
            VariantSelectionBackend::CanvasKitBrowser,
        ] {
            assert_glyph_selected(&tree, backend);
        }
        tree.resources.font_resources_mut().blobs[0]
            .digest
            .as_mut()
            .unwrap()
            .value = "not-the-embedded-font".into();
        for backend in [
            VariantSelectionBackend::NativeSkia,
            VariantSelectionBackend::CanvasKitBrowser,
        ] {
            assert_glyph_rejected(&tree, backend, VariantRejectReason::FontBlobDigestMismatch);
        }
    }
}

#[test]
fn synthetic_instances_do_not_grant_bidi_or_generic_vertical_authority() {
    let mut baseline = tree(COLLECTION, 1, "\u{e104}");
    glyph(&mut baseline).shape_key.font_instance.synthetic_bold = true;
    glyph(&mut baseline)
        .shape_key
        .font_instance
        .synthetic_italic = true;
    type Mutation = fn(&mut LayerGlyphRunPaint);
    let cases: [(Mutation, VariantRejectReason); 4] = [
        (
            |run| {
                run.direction = TextDirection::Rtl;
                run.shape_key.direction = TextDirection::Rtl;
            },
            VariantRejectReason::BidiDirectionAuthorityPending,
        ),
        (
            |run| run.bidi_level = Some(1),
            VariantRejectReason::BidiLevelAuthorityPending,
        ),
        (
            |run| {
                run.writing_mode = WritingMode::VerticalRl;
                run.shape_key.writing_mode = WritingMode::VerticalRl;
            },
            VariantRejectReason::WritingModeAuthorityPending,
        ),
        (
            |run| run.orientation = GlyphRunOrientation::VerticalUpright,
            VariantRejectReason::VerticalGlyphOrientationAuthorityPending,
        ),
    ];
    for (mutate, reason) in cases {
        let mut invalid = baseline.clone();
        mutate(glyph(&mut invalid));
        for backend in [
            VariantSelectionBackend::NativeSkia,
            VariantSelectionBackend::CanvasKitBrowser,
        ] {
            assert_glyph_rejected(&invalid, backend, reason);
        }
    }
}

#[test]
fn native_variation_contract_checks_exact_axes_ranges_duplicates_and_budget() {
    let mut tree = tree(VARIABLE, 0, "한");
    glyph(&mut tree).shape_key.font_instance.variations = vec![VariationAxisValue {
        tag: "wght".into(),
        value: 900.0,
    }];
    assert_glyph_selected(&tree, VariantSelectionBackend::NativeSkia);
    assert_glyph_rejected(
        &tree,
        VariantSelectionBackend::CanvasKitBrowser,
        VariantRejectReason::VariationUnsupported,
    );
    for axes in [
        vec![VariationAxisValue {
            tag: "nope".into(),
            value: 900.0,
        }],
        vec![VariationAxisValue {
            tag: "wght".into(),
            value: f32::NAN,
        }],
        vec![VariationAxisValue {
            tag: "wght".into(),
            value: 1_000_000.0,
        }],
        vec![
            VariationAxisValue {
                tag: "wght".into(),
                value: 900.0
            };
            2
        ],
        vec![
            VariationAxisValue {
                tag: "wght".into(),
                value: 900.0
            };
            17
        ],
        vec![VariationAxisValue {
            tag: "wgt".into(),
            value: 900.0,
        }],
    ] {
        glyph(&mut tree).shape_key.font_instance.variations = axes;
        let report = selection(&tree, VariantSelectionBackend::NativeSkia);
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::VariationUnsupported));
    }
}
