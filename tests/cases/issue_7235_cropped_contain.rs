//! #7235: contain은 crop 뒤의 가시 영역 비율을 사용한다.
//! 정사각형 원본에서 아래쪽 절반(2:1)을 잘라 정사각형 칸에 넣으면
//! 가시 영역은 가로 전체·세로 절반을 차지해야 한다. 원본의 빨강은 보이면 안 된다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::style::ImageFillMode;
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::LayerRenderer;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode};
use rhwp::renderer::svg_layer::SvgLayerRenderer;

fn cropped_tree(mode: ImageFillMode) -> PageLayerTree {
    let mut pixels = image::RgbaImage::new(40, 40);
    for (_, y, pixel) in pixels.enumerate_pixels_mut() {
        *pixel = if y < 20 {
            image::Rgba([255, 0, 0, 255])
        } else {
            image::Rgba([0, 0, 255, 255])
        };
    }
    let mut bytes = std::io::Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let mut image = ImageNode::new(1, Some(bytes.into_inner()));
    image.fill_mode = Some(mode);
    image.crop = Some((0, 20, 40, 40));
    image.original_size_hu = Some((40, 40));
    let bbox = BoundingBox::new(0.0, 0.0, 80.0, 80.0);
    PageLayerTree::new(
        80.0,
        80.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::image(bbox, image, None)]),
    )
}

#[test]
fn svg_contains_cropped_pixels_instead_of_the_uncropped_picture() {
    for mode in [ImageFillMode::None, ImageFillMode::Zoom] {
        let mut renderer = SvgLayerRenderer::new();
        renderer.render_page(&cropped_tree(mode)).unwrap();
        let svg = renderer.output();
        assert!(
            svg.contains("x=\"0\" y=\"20\" width=\"80\" height=\"40\" viewBox=\"0 20 40 20\""),
            "{mode:?}: {svg}"
        );
        assert!(
            svg.contains("overflow=\"hidden\""),
            "crop 외부가 letterbox에 보이지 않아야 한다"
        );
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn skia_contains_the_crop_without_stretching_or_leaking_other_pixels() {
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::skia::SkiaLayerRenderer;
    for mode in [
        ImageFillMode::None,
        ImageFillMode::Zoom,
        ImageFillMode::FitToSize,
        ImageFillMode::Total,
    ] {
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&cropped_tree(mode), RasterRenderOptions::default())
            .unwrap();
        let pixels = image::load_from_memory(&output.bytes).unwrap().to_rgba8();
        assert_eq!(
            *pixels.get_pixel(40, 40),
            image::Rgba([0, 0, 255, 255]),
            "{mode:?}: 가시 영역 색"
        );
        let expected_alpha = if matches!(mode, ImageFillMode::None | ImageFillMode::Zoom) {
            0
        } else {
            255
        };
        assert_eq!(
            pixels.get_pixel(40, 10)[3],
            expected_alpha,
            "{mode:?}: 위 letterbox"
        );
        assert_eq!(
            pixels.get_pixel(40, 70)[3],
            expected_alpha,
            "{mode:?}: 아래 letterbox"
        );
        assert!(
            !pixels.pixels().any(|p| p[0] > 128 && p[3] > 128),
            "{mode:?}: 잘린 빨강이 다시 나타남"
        );
    }
}

/// 독립적인 쪽 배경 정본 없이 셀 채우기의 의미를 쪽 전체로 확대하지 않는다.
#[test]
fn page_background_none_keeps_its_existing_stretch_contract() {
    use rhwp::renderer::render_tree::{PageBackgroundImage, PageBackgroundNode};
    let pixels = image::RgbaImage::from_pixel(40, 20, image::Rgba([0, 0, 255, 255]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let bbox = BoundingBox::new(0.0, 0.0, 80.0, 80.0);
    let tree = PageLayerTree::new(
        80.0,
        80.0,
        LayerNode::leaf(
            bbox,
            None,
            vec![PaintOp::page_background(
                bbox,
                PageBackgroundNode {
                    background_color: None,
                    border_color: None,
                    border_width: 0.0,
                    gradient: None,
                    image: Some(PageBackgroundImage {
                        data: bytes.into_inner(),
                        fill_mode: ImageFillMode::None,
                        brightness: 0,
                        contrast: 0,
                        effect: rhwp::model::image::ImageEffect::RealPic,
                    }),
                },
            )],
        ),
    );
    let mut svg = SvgLayerRenderer::new();
    svg.render_page(&tree).unwrap();
    assert!(svg.output().contains("preserveAspectRatio=\"none\""));
    assert!(!svg.output().contains("xMidYMid meet"));
    #[cfg(feature = "native-skia")]
    {
        let output = rhwp::renderer::skia::SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, Default::default())
            .unwrap();
        let pixels = image::load_from_memory(&output.bytes).unwrap().to_rgba8();
        assert_eq!(*pixels.get_pixel(40, 10), image::Rgba([0, 0, 255, 255]));
        assert_eq!(*pixels.get_pixel(40, 70), image::Rgba([0, 0, 255, 255]));
    }
}
