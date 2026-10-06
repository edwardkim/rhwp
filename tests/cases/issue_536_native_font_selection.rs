//! Portable Native Skia instance selection; actual Typeface construction is tested
//! separately by the native renderer. Browser authority must not widen with it.

use rhwp::paint::{
    lower_font_native_glyph_sidecars, EmbeddedFontFace, LayerGlyphRunPaint, LayerNode,
    LayerNodeKind, PageLayerTree, PaintOp, TextVariantKind, VariationAxisValue,
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

#[test]
fn native_collection_contract_keeps_the_exact_face_and_rejects_invalid_indexes() {
    let mut tree = tree(COLLECTION, 1, "\u{e104}");
    let report = selection(&tree, VariantSelectionBackend::NativeSkia);
    assert_eq!(
        report.selected_variant_kind,
        Some(TextVariantKind::GlyphRun)
    );
    assert!(!report.fallback_required);
    tree.resources.font_resources_mut().faces[0].face_index = 999;
    let report = selection(&tree, VariantSelectionBackend::NativeSkia);
    assert!(report.fallback_required);
    assert!(report.rejected_variants[0]
        .reasons
        .contains(&VariantRejectReason::FaceIndexUnsupported));
}

#[test]
fn native_synthetic_instances_do_not_grant_browser_authority_or_bypass_digest_proof() {
    let mut tree = tree(COLLECTION, 1, "\u{e104}");
    glyph(&mut tree).shape_key.font_instance.synthetic_bold = true;
    glyph(&mut tree).shape_key.font_instance.synthetic_italic = true;
    assert!(!selection(&tree, VariantSelectionBackend::NativeSkia).fallback_required);
    let browser = selection(&tree, VariantSelectionBackend::CanvasKitBrowser);
    assert!(browser.fallback_required);
    assert!(browser.rejected_variants[0]
        .reasons
        .contains(&VariantRejectReason::SyntheticStyleAuthorityPending));
    tree.resources.font_resources_mut().blobs[0]
        .digest
        .as_mut()
        .unwrap()
        .value = "not-the-embedded-font".into();
    let report = selection(&tree, VariantSelectionBackend::NativeSkia);
    assert!(report.fallback_required);
    assert!(report.rejected_variants[0]
        .reasons
        .contains(&VariantRejectReason::FontBlobDigestMismatch));
}

#[test]
fn native_variation_contract_checks_exact_axes_ranges_duplicates_and_budget() {
    let mut tree = tree(VARIABLE, 0, "한");
    glyph(&mut tree).shape_key.font_instance.variations = vec![VariationAxisValue {
        tag: "wght".into(),
        value: 900.0,
    }];
    assert!(!selection(&tree, VariantSelectionBackend::NativeSkia).fallback_required);
    assert!(selection(&tree, VariantSelectionBackend::CanvasKitBrowser).fallback_required);
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
