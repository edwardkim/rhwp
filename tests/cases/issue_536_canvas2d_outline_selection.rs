//! Canvas2D outlines require an explicit capability and the shared strict proof.
//! These are producer-contract checks; browser tests cover atomic preparation
//! and actual Canvas2D pixels before any TextRun is suppressed.

use rhwp::paint::*;
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, TextVariantSelectionReport,
    VariantRejectReason, VariantSelectedReason, VariantSelectionBackend,
};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::{PathCommand, TextStyle};

include!("../support/issue_536_portable_glyph_geometry.rs");

const PNG: &[u8] = include_bytes!("../../assets/logo/logo-32.png");
const COLLECTION: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactFaceSmoke.ttc");

fn ops_mut(tree: &mut PageLayerTree) -> &mut Vec<PaintOp> {
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("source-bound leaf fixture");
    };
    ops
}

fn outline_mut(tree: &mut PageLayerTree) -> &mut LayerGlyphOutlinePaint {
    let PaintOp::GlyphOutline { outline, .. } = &mut ops_mut(tree)[1] else {
        panic!("source-bound outline fixture");
    };
    outline
}

fn report(
    tree: &PageLayerTree,
    options: TextVariantSelectionOptions,
) -> TextVariantSelectionReport {
    let mut reports = analyze_text_variant_selection(tree, options);
    assert_eq!(reports.len(), 1, "one source-bound equivalence group");
    reports.remove(0)
}

fn assert_selected(tree: &PageLayerTree) {
    let selected = report(tree, TextVariantSelectionOptions::canvas2d_strict_outline());
    assert_eq!(selected.backend, VariantSelectionBackend::Canvas2D);
    assert_eq!(
        selected.selected_variant_kind,
        Some(TextVariantKind::GlyphOutline)
    );
    assert_eq!(
        selected.selected_reason,
        VariantSelectedReason::GlyphOutlineStrictProfile
    );
    assert!(!selected.fallback_required, "{selected:?}");
    assert!(selected.rejected_variants.is_empty(), "{selected:?}");
}

fn assert_fallback(tree: &PageLayerTree, reason: VariantRejectReason) {
    let selected = report(tree, TextVariantSelectionOptions::canvas2d_strict_outline());
    assert_eq!(
        selected.selected_variant_kind,
        Some(TextVariantKind::TextRun)
    );
    assert!(selected.fallback_required, "{selected:?}");
    assert_eq!(selected.rejected_variants.len(), 1, "{selected:?}");
    assert!(
        selected.rejected_variants[0].reasons.contains(&reason),
        "{selected:?}"
    );
}

fn color_tree(payload: ColorLayersPayload) -> PageLayerTree {
    assert!(
        payload.has_colrv0_resolved_layer_contract()
            || payload.has_colrv1_supported_graph_contract()
    );
    let mut tree = monochrome_selector_tree();
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
    outline.paths.clear();
    outline.color_layers = Some(payload);
    tree
}

fn bitmap_tree() -> PageLayerTree {
    let mut tree = monochrome_selector_tree();
    let image_ref = tree.resources.intern_image_bytes(PNG);
    let payload = BitmapGlyphPayload {
        image_ref,
        ..bitmap()
    };
    assert!(payload.has_strict_visual_contract());
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
    outline.paths.clear();
    outline.bitmap_glyph = Some(payload);
    tree
}

fn svg_tree() -> PageLayerTree {
    let mut tree = monochrome_selector_tree();
    let svg_ref = tree
        .resources
        .intern_svg_fragment("<path d=\"M0 0H8V8Z\" fill=\"#204080\"/>");
    let payload = SvgGlyphPayload { svg_ref, ..svg() };
    assert!(payload.has_static_sanitized_contract());
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
    outline.paths.clear();
    outline.svg_glyph = Some(payload);
    tree
}

#[test]
fn canvas2d_outline_capability_is_off_by_default_even_with_strict_preference() {
    assert!(!TextVariantSelectionOptions::default().allow_canvas2d_glyph_outline);
    assert!(!TextVariantSelectionOptions::canvaskit_strict_outline().allow_canvas2d_glyph_outline);
    let tree = monochrome_selector_tree();
    assert_selected(&tree);
    for prefer_strict_outline in [false, true] {
        let selected = report(
            &tree,
            TextVariantSelectionOptions {
                backend: VariantSelectionBackend::Canvas2D,
                prefer_strict_outline,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(
            selected.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(selected.fallback_required);
        assert!(selected.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::BackendDoesNotSupportVariant));
    }
}

#[test]
fn explicit_canvas2d_capability_accepts_the_supported_outline_families() {
    assert_selected(&monochrome_selector_tree());
    for paint_order in [
        GlyphOutlinePaintOrder::FillThenStroke,
        GlyphOutlinePaintOrder::StrokeThenFill,
    ] {
        let mut tree = monochrome_selector_tree();
        let stroke = GlyphOutlineStrokeStyle {
            color: 0x204080,
            width: 1.0,
            join: GlyphOutlineStrokeJoin::Miter,
            cap: GlyphOutlineStrokeCap::Butt,
            miter_limit: 2.0,
            paint_order,
        };
        assert!(stroke.is_strict_subset());
        let outline = outline_mut(&mut tree);
        outline.payload_kind = GlyphOutlinePayloadKind::MonochromeFillStroke;
        outline.stroke = Some(stroke);
        assert_selected(&tree);
    }
    assert_selected(&color_tree(colrv0()));
    for kind in [
        ColorPaintGraphNodeKind::SolidPath,
        ColorPaintGraphNodeKind::LinearGradientPath,
        ColorPaintGraphNodeKind::RadialGradientPath,
        ColorPaintGraphNodeKind::SweepGradientPath,
    ] {
        assert_selected(&color_tree(colrv1(kind)));
    }
    assert_selected(&bitmap_tree());
    assert_selected(&svg_tree());
}

#[test]
fn canvas2d_opt_in_preserves_per_family_capability_switches() {
    for (tree, options, reason) in [
        (
            color_tree(colrv0()),
            TextVariantSelectionOptions {
                allow_colrv0_color_layers: false,
                ..TextVariantSelectionOptions::canvas2d_strict_outline()
            },
            VariantRejectReason::UnsupportedColorGlyph,
        ),
        (
            color_tree(colrv1(ColorPaintGraphNodeKind::SolidPath)),
            TextVariantSelectionOptions {
                allow_colrv1_stage1_color_graph: false,
                ..TextVariantSelectionOptions::canvas2d_strict_outline()
            },
            VariantRejectReason::UnsupportedColorGlyph,
        ),
        (
            bitmap_tree(),
            TextVariantSelectionOptions {
                allow_bitmap_glyph: false,
                ..TextVariantSelectionOptions::canvas2d_strict_outline()
            },
            VariantRejectReason::UnsupportedBitmapGlyph,
        ),
        (
            svg_tree(),
            TextVariantSelectionOptions {
                allow_svg_glyph: false,
                ..TextVariantSelectionOptions::canvas2d_strict_outline()
            },
            VariantRejectReason::UnsupportedSvgGlyph,
        ),
    ] {
        assert_selected(&tree);
        let selected = report(&tree, options);
        assert_eq!(
            selected.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(selected.fallback_required);
        assert!(
            selected.rejected_variants[0].reasons.contains(&reason),
            "{selected:?}"
        );
    }
}

#[test]
fn canvas2d_opt_in_keeps_whole_variant_part_validation() {
    let mut complete = monochrome_selector_tree();
    outline_mut(&mut complete).variant.part_count = 2;
    let mut second = ops_mut(&mut complete)[1].clone();
    let PaintOp::GlyphOutline { outline, .. } = &mut second else {
        unreachable!()
    };
    outline.variant.part_index = 1;
    ops_mut(&mut complete).push(second);
    assert_selected(&complete);

    let mut missing = complete.clone();
    ops_mut(&mut missing).pop();
    assert_fallback(&missing, VariantRejectReason::VariantPartsIncomplete);
    let mut duplicate = complete.clone();
    let duplicate_part = ops_mut(&mut duplicate)[1].clone();
    ops_mut(&mut duplicate).push(duplicate_part);
    assert_fallback(&duplicate, VariantRejectReason::VariantDuplicatePart);
    let mut malformed = complete;
    let PaintOp::GlyphOutline { outline, .. } = &mut ops_mut(&mut malformed)[2] else {
        unreachable!()
    };
    outline.paths[0].commands[0] = PathCommand::MoveTo(f64::NAN, 0.0);
    assert_fallback(&malformed, VariantRejectReason::UnsupportedOutlinePayload);
}

#[test]
fn canvas2d_opt_in_preserves_geometry_style_and_diagnostic_rejection() {
    let mutations: [(fn(&mut LayerGlyphOutlinePaint), VariantRejectReason); 8] = [
        (
            |outline| outline.paths.clear(),
            VariantRejectReason::EmptyGlyphOutlinePayload,
        ),
        (
            |outline| outline.paths[0].commands[1] = PathCommand::LineTo(1e100, 0.0),
            VariantRejectReason::UnsupportedOutlinePayload,
        ),
        (
            |outline| outline.placement.run_to_page.e = 1e100,
            VariantRejectReason::UnsupportedOutlinePayload,
        ),
        (
            |outline| outline.diagnostics.strict_visual_eligible = false,
            VariantRejectReason::StrictVisualIneligible,
        ),
        (
            |outline| outline.diagnostics.missing_glyph_count = 1,
            VariantRejectReason::MissingGlyph,
        ),
        (
            |outline| outline.diagnostics.cluster_mismatch_count = 1,
            VariantRejectReason::ClusterMismatch,
        ),
        (
            |outline| outline.diagnostics.used_fallback_font_count = 1,
            VariantRejectReason::FontNotPortable,
        ),
        (
            |outline| {
                outline.diagnostics.quality = TextVariantQuality::PositionAdjusted;
                outline.diagnostics.max_residual_after_adjustment_px = 0.5;
            },
            VariantRejectReason::PositionAdjustedResidualTooLarge,
        ),
    ];
    for (mutate, reason) in mutations {
        let mut tree = monochrome_selector_tree();
        assert_selected(&tree);
        mutate(outline_mut(&mut tree));
        assert_fallback(&tree, reason);
    }
    let mut mixed = monochrome_selector_tree();
    outline_mut(&mut mixed).color_layers = Some(colrv0());
    assert_fallback(&mixed, VariantRejectReason::MixedGlyphOutlinePayload);
    let mut unsupported_stroke = monochrome_selector_tree();
    let outline = outline_mut(&mut unsupported_stroke);
    outline.payload_kind = GlyphOutlinePayloadKind::MonochromeFillStroke;
    outline.stroke = Some(GlyphOutlineStrokeStyle {
        color: 0,
        width: 1.0,
        join: GlyphOutlineStrokeJoin::Round,
        cap: GlyphOutlineStrokeCap::Butt,
        miter_limit: 2.0,
        paint_order: GlyphOutlinePaintOrder::FillThenStroke,
    });
    assert_fallback(
        &unsupported_stroke,
        VariantRejectReason::GlyphOutlineStrokeStyleUnsupported,
    );
}

#[test]
fn canvas2d_opt_in_rejects_missing_or_malformed_payload_resources() {
    for mut tree in [bitmap_tree(), svg_tree()] {
        assert_selected(&tree);
        tree.resources = ResourceArena::default();
        assert_fallback(&tree, VariantRejectReason::MissingGlyphPayloadResource);
    }
    let mut tree = bitmap_tree();
    let invalid_image = tree.resources.intern_image_bytes(b"not a decoded bitmap");
    outline_mut(&mut tree)
        .bitmap_glyph
        .as_mut()
        .unwrap()
        .image_ref = invalid_image;
    assert_fallback(&tree, VariantRejectReason::UnsupportedBitmapGlyph);
    let mut tree = svg_tree();
    let invalid_svg = tree
        .resources
        .intern_svg_fragment("<path d=\"M0 0LNaN 5\"/>");
    outline_mut(&mut tree).svg_glyph.as_mut().unwrap().svg_ref = invalid_svg;
    assert_fallback(&tree, VariantRejectReason::UnsupportedSvgGlyph);
    let mut tree = color_tree(colrv1(ColorPaintGraphNodeKind::SolidPath));
    outline_mut(&mut tree)
        .color_layers
        .as_mut()
        .unwrap()
        .source_font_ref = None;
    assert_fallback(&tree, VariantRejectReason::UnsupportedColorGlyph);
}

#[test]
fn canvas2d_opt_in_does_not_claim_q3_variable_outline_authority() {
    let mut tree = monochrome_selector_tree();
    assert_selected(&tree);
    outline_mut(&mut tree).diagnostics.reason = Some("q3VariableOutlineProjectionV1".into());
    assert_fallback(&tree, VariantRejectReason::BackendDoesNotSupportVariant);
}

#[test]
fn canvas2d_outline_opt_in_never_enables_glyph_runs_or_skips_font_byte_proof() {
    let mut tree = monochrome_selector_tree();
    let ops = ops_mut(&mut tree);
    ops.truncate(1);
    let PaintOp::TextRun { run, source, .. } = &mut ops[0] else {
        unreachable!()
    };
    run.text = "\u{e104}".into();
    run.char_shape_id = Some(536);
    run.layout_positions = Some(vec![0.0, 12.0]);
    *source = Some(TextSourceSpan::for_text_run(0, run));
    tree.text_sources = TextSourceTable::from_layer_node(&tree.root);
    let emitted = lower_font_native_glyph_sidecars(
        &mut tree.root,
        &mut tree.resources,
        &[EmbeddedFontFace {
            char_shape_id: 536,
            language_index: 0,
            family: "Test",
            alternate_family: None,
            bytes: COLLECTION,
            face_index: 1,
        }],
    );
    assert_eq!(emitted.emitted_glyph_runs, 1, "{emitted:?}");
    ops_mut(&mut tree).retain(|op| !matches!(op, PaintOp::GlyphOutline { .. }));
    let browser_options = TextVariantSelectionOptions {
        backend: VariantSelectionBackend::CanvasKitBrowser,
        ..TextVariantSelectionOptions::canvaskit()
    };
    let supported = report(&tree, browser_options);
    assert_eq!(
        supported.selected_variant_kind,
        Some(TextVariantKind::GlyphRun),
        "{supported:?}"
    );
    assert_fallback(&tree, VariantRejectReason::BackendDoesNotSupportVariant);
    let mut resources_without_bytes = ResourceArena::default();
    *resources_without_bytes.font_resources_mut() = tree.resources.font_resources().clone();
    tree.resources = resources_without_bytes;
    assert_fallback(&tree, VariantRejectReason::BackendDoesNotSupportVariant);
    // Canvas2D never opens the GlyphRun lane. The supporting browser backend
    // must still reject the same real face when its verified bytes disappear.
    let missing_bytes = report(&tree, browser_options);
    assert!(missing_bytes.fallback_required, "{missing_bytes:?}");
    assert_eq!(
        missing_bytes.selected_variant_kind,
        Some(TextVariantKind::TextRun)
    );
    assert!(
        missing_bytes.rejected_variants.iter().any(|rejected| {
            rejected
                .reasons
                .contains(&VariantRejectReason::FontBlobBytesMissing)
        }),
        "{missing_bytes:?}"
    );
}
