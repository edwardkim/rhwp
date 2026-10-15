//! 분수 원점의 본문 glyph를 Native raster가 정수 좌표로 되돌리지 않는 계약.
//! 실물 문서 좌표/golden이 아니라 공급한 OFL 윤곽의 평행 이동을 검사한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;

fn render(profile: RenderProfile, offset: f64) -> image::RgbaImage {
    let bbox = BoundingBox::new(10.0 + offset, 10.0, 50.0, 30.0);
    let run = TextRunNode {
        text: "A".into(),
        style: TextStyle {
            font_family: "RHWP Supplied Style Contract".into(),
            font_size: 24.0,
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
        baseline: 26.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    };
    let tree = PageLayerTree::with_profile(
        96.0,
        64.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]),
        profile,
    );
    let fonts =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/native_font_styles");
    let png = SkiaLayerRenderer::new()
        .with_font_paths(&[fonts])
        .render_png(&tree)
        .unwrap();
    image::load_from_memory(&png).unwrap().to_rgba8()
}

fn ink_center(image: &image::RgbaImage) -> f64 {
    let mut mass = 0.0;
    let mut moment = 0.0;
    for (x, _, pixel) in image.enumerate_pixels() {
        let darkness = (765.0 - f64::from(pixel[0]) - f64::from(pixel[1]) - f64::from(pixel[2]))
            * f64::from(pixel[3])
            / 255.0;
        mass += darkness;
        moment += f64::from(x) * darkness;
    }
    assert!(mass > 0.0, "공급한 glyph가 실제로 보인다");
    moment / mass
}

#[test]
fn positioned_body_glyph_preserves_fractional_horizontal_origin() {
    eprintln!(
        "Skia default subpixel={}",
        skia_safe::Font::default().is_subpixel()
    );
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let original = render(profile, 0.0);
        let shifted = render(profile, 0.25);
        let integer = render(profile, 1.0);
        let delta = ink_center(&shifted) - ink_center(&original);
        assert!(
            original != shifted,
            "분수 원점을 정수 glyph 원점으로 소실하면 안 된다"
        );
        assert!(
            (delta - 0.25).abs() < 0.10,
            "요청한 평행 이동 0.25, 실제 ink 이동 {delta}"
        );
        let integer_delta = ink_center(&integer) - ink_center(&original);
        assert!(
            (integer_delta - 1.0).abs() < 0.001,
            "정수 평행 이동 대조군: {integer_delta}"
        );
    }
}
