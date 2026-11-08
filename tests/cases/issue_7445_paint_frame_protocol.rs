//! 독립 Print의 여섯 크기와 잘못된 affine 계약을 public API에서 검증한다.
//! 기존 source-side unit의 모든 기대값과 반례를 integration 경로에 보존한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::style::UnderlineType;
use rhwp::paint::{LayerBuilder, LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::canvaskit_policy::{
    analyze_canvaskit_replay_plan, CanvasKitReplayMode, CanvasKitReplayReason,
    CanvasKitReplayStatus,
};
use rhwp::renderer::{
    render_tree::{
        BoundingBox, FieldMarkerType, PageRenderTree, RenderNode, RenderNodeType, TextRunNode,
    },
    TextStyle,
};
fn bbox() -> BoundingBox {
    BoundingBox::new(0.0, 0.0, 20.0, 20.0)
}

fn text_run(text: &str) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle::default(),
        char_shape_id: None,
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    }
}

fn tree_with_ops(ops: Vec<PaintOp>) -> PageLayerTree {
    PageLayerTree::new(100.0, 100.0, LayerNode::leaf(bbox(), None, ops))
}

#[test]
fn print_underline_uses_independent_paper_frame_without_moving_source_text() {
    // Same six-size HCR/HY probe as C234/C237. Independent Hancom 2020
    // Print PDF SHA256 3571c9e3cff577b73e241246826fb36933a28faf7340c48362008ad665f202ca.
    // Expected paper-space CSS y comes from PDF strokes, not this producer.
    let samples = [
        (132.267, 10.2, 12.0, 144.32267252604166),
        (151.467, 13.6, 16.0, 167.656005859375),
        (177.067, 17.0, 20.0, 197.22403971354166),
        (209.067, 20.4, 24.0, 233.34529622395834),
        (247.467, 27.2, 32.0, 279.8533528645833),
        (298.667, 40.8, 48.0, 347.2999674479167),
    ];
    fn decoration(value: &serde_json::Value) -> Option<&serde_json::Value> {
        if value.get("type").and_then(|v| v.as_str()) == Some("textDecoration") {
            return Some(value);
        }
        match value {
            serde_json::Value::Object(map) => map.values().find_map(decoration),
            serde_json::Value::Array(items) => items.iter().find_map(decoration),
            _ => None,
        }
    }
    for family in ["함초롬바탕", "HYSinMyeongJo-Medium,Haansoft Batang"] {
        for (y, baseline, font_size, independent_y) in samples {
            let mut run = text_run("HxHxHx");
            run.style.font_family = family.to_string();
            run.style.font_size = font_size;
            run.style.underline = UnderlineType::Bottom;
            run.baseline = baseline;
            let bbox = BoundingBox::new(113.387, y, 100.0, font_size);
            let mut tree = PageRenderTree::new(0, 59528.0 / 75.0, 84186.0 / 75.0);
            tree.root.children.push(RenderNode::new(
                1,
                RenderNodeType::TextRun(run.clone()),
                bbox,
            ));
            let layer = LayerBuilder::new(RenderProfile::Print).build(&tree);
            rhwp::paint::validate_text_variant_scope(&layer).unwrap();
            assert_eq!(layer.text_sources.entries[0].text, run.text);
            let json: serde_json::Value = serde_json::from_str(&layer.to_json()).unwrap();
            let op = decoration(&json).expect("same-source underline");
            let actual_y = op["bbox"]["y"].as_f64().unwrap()
                + op["decoration"]["baseline"].as_f64().unwrap()
                + op["decoration"]["fontSize"].as_f64().unwrap() * 0.17;
            // Half a 600-DPI device dot plus 0.001 CSS serialization precision.
            assert!(
                (actual_y - independent_y).abs() <= 0.081,
                "{family} {font_size}: {actual_y} vs independent {independent_y}"
            );
            let LayerNodeKind::Group { children, .. } = &layer.root.kind else {
                panic!()
            };
            let LayerNodeKind::Leaf { ops } = &children[0].kind else {
                panic!()
            };
            for op in ops {
                if let PaintOp::TextRun {
                    bbox: source_bbox,
                    run: source_run,
                    ..
                } = op
                {
                    assert_eq!(source_bbox.x, bbox.x);
                    assert_eq!(source_bbox.y, bbox.y);
                    assert_eq!(source_run.as_ref(), &run);
                }
            }
        }
    }
}

#[test]
fn malformed_glyph_output_frame_is_never_direct_or_hidden_overlay() {
    for mode in [CanvasKitReplayMode::Default, CanvasKitReplayMode::Compat] {
        for (a, d) in [(f64::NAN, 1.0), (1.0, f64::INFINITY), (0.0, 1.0)] {
            let mut op = PaintOp::text_run(bbox(), policy_text_run("A"));
            if let PaintOp::TextRun {
                paint_transform, ..
            } = &mut op
            {
                *paint_transform = Some(rhwp::paint::LayerAffineTransform {
                    a,
                    b: 0.0,
                    c: 0.0,
                    d,
                    e: 0.0,
                    f: 0.0,
                });
            }
            let plan = analyze_canvaskit_replay_plan(&tree_with_ops(vec![op]), mode);
            assert_eq!(plan.items.len(), 1);
            assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Unsupported);
            assert_eq!(
                plan.items[0].reason,
                CanvasKitReplayReason::UnsupportedFeature
            );
            assert_eq!(
                plan.items[0].detail.as_deref(),
                Some("invalidGlyphPaintTransform")
            );
            assert!(!plan.items[0].compat_overlay_allowed);
        }
    }
}

fn policy_text_run(text: &str) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle {
            font_family: "Test".to_string(),
            font_size: 12.0,
            shade_color: 0x00FF_FFFF,
            ..Default::default()
        },
        char_shape_id: None,
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: FieldMarkerType::None,
        layout_positions: None,
        display_text: None,
    }
}
