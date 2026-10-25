#![cfg(feature = "native-skia")]

//! #8153: Print의 소수 원점과 Screen의 기존 격자를 실제 Native raster로 대조한다.
//! 합성 이동 계약이며 한컴 실물 문서 시각 정확도의 증거로 대신하지 않는다.

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::RasterPixels;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;

fn glyph_at(
    renderer: &SkiaLayerRenderer,
    y: f64,
    fixed_glyph_advances: bool,
    profile: RenderProfile,
) -> RasterPixels {
    let run = TextRunNode {
        text: "H".into(),
        style: TextStyle {
            font_family: "Noto Sans KR".into(),
            font_size: 32.0,
            color: 0,
            fixed_glyph_advances,
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
    let mut tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 64.0, 64.0),
            None,
            vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
        ),
    );
    tree.profile = profile;
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

fn check(renderer: &SkiaLayerRenderer, profile: RenderProfile) {
    let original = ink_center(&glyph_at(renderer, 4.1, false, profile));
    let quarter = ink_center(&glyph_at(renderer, 4.35, false, profile));
    let whole = ink_center(&glyph_at(renderer, 5.1, false, profile));
    let shift = quarter.1 - original.1;
    if profile == RenderProfile::Print {
        assert!(
            (shift - 0.25).abs() <= 0.08,
            "Print는 원본 1/4px 이동을 보존해야 한다: {shift}"
        );
    } else {
        assert!(shift.abs() <= 0.02, "Screen의 기존 격자 계약: {shift}");
    }
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}
fn renderer() -> SkiaLayerRenderer {
    SkiaLayerRenderer::new().with_font_paths(&[
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ttfs/opensource")
    ])
}
#[test]
fn print_ordinary_ttf_preserves_fractional_baseline() {
    check(&renderer(), RenderProfile::Print);
}
#[test]
fn screen_ordinary_ttf_keeps_existing_baseline_grid() {
    check(&renderer(), RenderProfile::Screen);
}
#[test]
fn print_screen_print_roundtrip_keeps_each_raster_contract() {
    let renderer = renderer();
    for profile in [
        RenderProfile::Print,
        RenderProfile::Screen,
        RenderProfile::Print,
    ] {
        check(&renderer, profile);
    }
}
