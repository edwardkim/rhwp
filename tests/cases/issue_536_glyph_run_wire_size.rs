//! Font preflight and GlyphRun replay must agree after fixed-decimal JSON export.

use rhwp::paint::{
    lower_font_native_glyph_sidecars, EmbeddedFontFace, FontGlyphLoweringReport, LayerNode,
    LayerNodeKind, PageLayerTree, PaintOp, TextVariantKind,
};
use rhwp::renderer::canvaskit_policy::{analyze_canvaskit_replay_plan, CanvasKitReplayMode};
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantRejectReason,
};
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::TextStyle;

fn glyph_run_tree(font_size: f64) -> (PageLayerTree, FontGlyphLoweringReport) {
    let bounds = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
    let text = TextRunNode {
        text: "\u{e100}".into(),
        style: TextStyle {
            font_family: "wire fallback family".into(),
            font_size,
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
        layout_positions: None,
        display_text: None,
    };
    let mut tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(bounds, None, vec![PaintOp::text_run(bounds, text)]),
    );
    let report = lower_font_native_glyph_sidecars(
        &mut tree.root,
        &mut tree.resources,
        &[EmbeddedFontFace {
            char_shape_id: 536,
            language_index: 0,
            family: "fixture family",
            alternate_family: None,
            bytes: include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf"),
            face_index: 0,
        }],
    );
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        unreachable!("leaf fixture");
    };
    // Isolate GlyphRun selection from the font's bitmap/SVG alternatives.
    ops.retain(|op| !matches!(op, PaintOp::GlyphOutline { .. }));
    (tree, report)
}

#[test]
fn glyph_run_selection_keeps_fallback_fonts_when_font_size_collapses_on_wire() {
    let (control, report) = glyph_run_tree(16.0);
    assert_eq!(report.emitted_glyph_runs, 1);
    assert_eq!(
        analyze_text_variant_selection(&control, TextVariantSelectionOptions::canvaskit())[0]
            .selected_variant_kind,
        Some(TextVariantKind::GlyphRun)
    );
    for size in [f64::MIN_POSITIVE, 0.000_000_49] {
        let mut tree = control.clone();
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!("leaf fixture");
        };
        let run = ops
            .iter_mut()
            .find_map(|op| match op {
                PaintOp::GlyphRun { run, .. } => Some(run),
                _ => None,
            })
            .unwrap();
        run.shape_key.font_instance.size_px = size;
        let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
        let serialized = wire["root"]["ops"]
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["type"] == "glyphRun")
            .unwrap();
        assert_eq!(serialized["shapeKey"]["fontInstance"]["sizePx"], 0.0);
        let selection =
            analyze_text_variant_selection(&tree, TextVariantSelectionOptions::canvaskit());
        assert_eq!(
            selection[0].selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(selection[0].fallback_required);
        assert!(selection[0].rejected_variants.iter().any(|variant| variant
            .reasons
            .contains(&VariantRejectReason::FontInstanceInvalid)));
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(plan.required_font_families, ["wire fallback family"]);
        assert!(plan.required_font_families_complete);
    }
}

#[test]
fn nominal_glyph_run_producer_requires_a_positive_wire_font_size() {
    for size in [f64::MIN_POSITIVE, 0.000_000_49] {
        let (tree, report) = glyph_run_tree(size);
        assert_eq!(report.emitted_glyph_runs, 0, "size={size}");
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(plan.required_font_families, ["wire fallback family"]);
        assert!(plan.required_font_families_complete);
    }
    let (tree, report) = glyph_run_tree(0.000_001);
    assert_eq!(report.emitted_glyph_runs, 1);
    let selection = analyze_text_variant_selection(&tree, TextVariantSelectionOptions::canvaskit());
    assert_eq!(
        selection[0].selected_variant_kind,
        Some(TextVariantKind::GlyphRun)
    );
    let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
    let serialized = wire["root"]["ops"]
        .as_array()
        .unwrap()
        .iter()
        .find(|op| op["type"] == "glyphRun")
        .unwrap();
    assert_eq!(serialized["shapeKey"]["fontInstance"]["sizePx"], 0.000_001);
}
