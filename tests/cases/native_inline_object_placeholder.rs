//! U+FFFC is an inline control slot, not a printable fallback glyph.
//! Synthetic paint equivalence: no serialized document layout or pixel golden.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;

fn paint(text: &str, positions: Vec<f64>, profile: RenderProfile) -> Vec<u8> {
    paint_runs(vec![(text, positions, 8.0)], profile)
}

fn paint_runs(parts: Vec<(&str, Vec<f64>, f64)>, profile: RenderProfile) -> Vec<u8> {
    let ops = parts
        .into_iter()
        .map(|(text, positions, x)| {
            let bounds = BoundingBox::new(x, 4.0, 120.0, 48.0);
            let run = TextRunNode {
                text: text.into(),
                style: TextStyle {
                    font_family: "Rhwp Inline Placeholder Contract".into(),
                    font_size: 28.0,
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
                baseline: 34.0,
                field_marker: Default::default(),
                layout_positions: Some(positions),
                display_text: None,
            };
            PaintOp::text_run(bounds, run)
        })
        .collect();
    let tree = PageLayerTree::with_profile(
        144.0,
        64.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 144.0, 64.0), None, ops),
        profile,
    );
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[include_bytes!(
            "../fixtures/native_inline_placeholder/placeholder-contract.ttf"
        )
        .to_vec()])
        .unwrap();
    image::load_from_memory(&renderer.render_png(&tree).unwrap())
        .unwrap()
        .to_rgba8()
        .into_raw()
}

#[test]
fn object_slot_does_not_paint_or_reindex_the_following_text() {
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let blank = paint("", vec![0.0], profile);
        assert_eq!(
            paint("\u{fffc}", vec![0.0, 0.0], profile),
            blank,
            "a zero-advance control slot has no text ink even when the font supplies a glyph"
        );
        assert_ne!(
            paint("\u{fffd}", vec![0.0, 24.0], profile),
            blank,
            "the visible replacement character is not an object control slot"
        );
        let reference = paint("A\u{fffd}A", vec![0.0, 40.0, 80.0, 104.0], profile);
        assert_eq!(
            paint(
                "A\u{fffc}\u{fffd}A",
                vec![0.0, 40.0, 40.0, 80.0, 104.0],
                profile
            ),
            reference,
            "skipping the object ink preserves the original character-indexed origins"
        );
        assert_eq!(
            paint("क\u{fffc}क", vec![0.0, 30.0, 90.0, 114.0], profile),
            paint_runs(
                vec![("क", vec![0.0, 30.0], 8.0), ("क", vec![0.0, 24.0], 98.0)],
                profile
            ),
            "complex spans must not shape across an inline object slot"
        );
    }
}
