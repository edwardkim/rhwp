//! 독립 한컴 저장 대조군의 RGB 바이트 계약. 합성 canvas는 실제 문서의 위치 핀이 아니다.
#![cfg(feature = "native-skia")]
use rhwp::model::image::ImageEffect;
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode};
use rhwp::renderer::skia::SkiaLayerRenderer;

fn rendered(effect: ImageEffect, brightness: i8, contrast: i8, source: [u8; 3]) -> [u8; 4] {
    let mut bytes = Vec::new();
    image::RgbaImage::from_pixel(4, 4, image::Rgba([source[0], source[1], source[2], 255]))
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    let mut image = ImageNode::new(0, Some(bytes));
    image.effect = effect;
    image.brightness = brightness;
    image.contrast = contrast;
    let bbox = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
    let tree = PageLayerTree::with_profile(
        32.0,
        32.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::image(bbox, image, None)]),
        RenderProfile::Print,
    );
    let output = SkiaLayerRenderer::new()
        .render_raster_with_options(
            &tree,
            RasterRenderOptions {
                transparent: false,
                ..Default::default()
            },
        )
        .expect("실제 Native PNG");
    image::load_from_memory(&output.bytes)
        .unwrap()
        .to_rgba8()
        .get_pixel(16, 16)
        .0
}

#[test]
fn issue_7751_normal_blackwhite_tones_are_discrete_after_adjustment() {
    // 독립 PDF의 원본11색 중 밝기와 이산화 순서가 갈리는 색·임계점.
    // B=0/10/50 PDF에서 직접 추출한 값이며 구현식으로 재계산하지 않는다.
    for (brightness, source, expected) in [
        (0, [128, 128, 128], 0),
        (0, [170, 170, 170], 255),
        (10, [100, 100, 100], 0),
        (10, [128, 128, 128], 255),
        (50, [0, 0, 0], 0),
        (50, [50, 50, 50], 255),
        (50, [80, 100, 120], 255),
        (-100, [255, 255, 255], 0),
        (-50, [255, 255, 255], 0),
        (-20, [170, 170, 170], 0),
        (100, [0, 0, 0], 255),
    ] {
        assert_eq!(
            rendered(ImageEffect::BlackWhite, brightness, 0, source),
            [expected, expected, expected, 255],
            "독립 Print 흑백: B={brightness}, 원본={source:?}"
        );
    }
    // 별도 axis 대조군의 대비 음수·끝값. 원본의 효과·B/C 보존도 실제 WASM에서 검사한다.
    for (contrast, source, expected) in [
        (-100, [255, 255, 255], 0),
        (-50, [170, 170, 170], 255),
        (100, [128, 128, 128], 0),
        (100, [170, 170, 170], 255),
    ] {
        assert_eq!(
            rendered(ImageEffect::BlackWhite, 0, contrast, source),
            [expected, expected, expected, 255],
            "독립 Print 흑백: C={contrast}, 원본={source:?}"
        );
    }
}

#[test]
fn issue_7751_single_axis_grayscale_tones_preserve_opaque_print_pixels() {
    for (source, expected) in [([80, 100, 120], 64), ([255, 0, 0], 24), ([0, 0, 0], 0)] {
        assert_eq!(
            rendered(ImageEffect::GrayScale, 0, 50, source),
            [expected, expected, expected, 255]
        );
    }
}
