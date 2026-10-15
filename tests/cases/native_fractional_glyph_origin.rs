#![cfg(feature = "native-skia")]

//! 조판과 무관한 raster 원점 계약: 같은 글리프의 입력 이동량이 잉크에도 보존돼야 한다.

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::RasterPixels;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;

fn glyph_at(renderer: &SkiaLayerRenderer, y: f64) -> RasterPixels {
    let run = TextRunNode {
        text: "H".into(),
        style: TextStyle {
            font_family: "Noto Sans KR".into(),
            font_size: 32.0,
            color: 0,
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
        baseline: 32.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    };
    let tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 64.0, 64.0),
            None,
            vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
        ),
    );
    renderer
        .render_pixels_with_options(&tree, RasterRenderOptions::default())
        .expect("글리프를 투명 RGBA에 재생한다")
}

fn ink_center(image: &RasterPixels) -> (f64, f64) {
    let mut mass = 0.0;
    let mut x_sum = 0.0;
    let mut y_sum = 0.0;
    for (y, row) in image.bytes.chunks_exact(image.stride).enumerate() {
        for (x, pixel) in row[..image.width as usize * 4].chunks_exact(4).enumerate() {
            let weight = f64::from(pixel[3]);
            mass += weight;
            x_sum += x as f64 * weight;
            y_sum += y as f64 * weight;
        }
    }
    assert!(mass > 0.0, "공백/누락을 같은 위치의 잉크로 판정하지 않는다");
    (x_sum / mass, y_sum / mass)
}

#[test]
fn positioned_glyph_preserves_quarter_pixel_baseline_translation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[root.join("ttfs/opensource")]);
    let original = ink_center(&glyph_at(&renderer, 4.1));
    let quarter = ink_center(&glyph_at(&renderer, 4.35));
    let whole = ink_center(&glyph_at(&renderer, 5.1));
    // 입력은 정확히1/4px 이동이다. alpha 양자화만 허용하며, 같은 정수 기준선으로
    // 반올림돼 이동이0이 되는 기존 raster 동작을 검출한다.
    let quarter_shift = quarter.1 - original.1;
    assert!(
        (quarter_shift - 0.25).abs() <= 0.08,
        "소수 기준선 이동이 잉크에 보존돼야 한다: {quarter_shift}"
    );
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}
