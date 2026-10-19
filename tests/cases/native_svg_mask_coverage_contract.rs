//! SVG 마스크의 면적 피복률·원래 프레임·premultiplied 색 계약이다.
//! 합성 도형의 독립적인 50% 면적 법칙이며 실물 문서의 시각 완료 증거는 아니다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use base64::Engine;
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode, RawSvgNode};
use rhwp::renderer::skia::SkiaLayerRenderer;

fn patterned_mask() -> String {
    let checker = image::RgbaImage::from_fn(8, 8, |x, y| {
        let shade = if (x + y) % 2 == 0 { 0 } else { 255 };
        image::Rgba([shade, shade, shade, 255])
    });
    let mut encoded = std::io::Cursor::new(Vec::new());
    checker
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let data = base64::engine::general_purpose::STANDARD.encode(encoded.into_inner());
    format!(
        r##"<defs>
<pattern id="p" width="8" height="8" patternUnits="userSpaceOnUse">
<image width="8" height="8" href="data:image/png;base64,{data}"/></pattern>
<filter id="inverse" color-interpolation-filters="sRGB"><feComponentTransfer>
<feFuncR type="table" tableValues="1 0"/><feFuncG type="table" tableValues="1 0"/>
<feFuncB type="table" tableValues="1 0"/></feComponentTransfer></filter>
<mask id="m" x="0" y="0" width="320" height="240" maskUnits="userSpaceOnUse" mask-type="luminance">
<rect width="320" height="240" fill="url(#p)" filter="url(#inverse)"/></mask></defs>
<rect width="320" height="240" fill="#804020" mask="url(#m)"/>"##
    )
}

fn render(content: &str, fragment: bool) -> (image::RgbaImage, BoundingBox) {
    let bounds = BoundingBox::new(8.0, 6.0, 32.0, 24.0);
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="240" viewBox="0 0 320 240">{content}</svg>"#
    );
    let op = if fragment {
        // RawSvg는 이미 페이지 좌표인 조각을 소비한다.
        PaintOp::raw_svg(
            bounds,
            RawSvgNode::new(format!(
                r#"<svg x="{}" y="{}" width="{}" height="{}" viewBox="0 0 320 240">{content}</svg>"#,
                bounds.x, bounds.y, bounds.width, bounds.height,
            )),
        )
    } else {
        PaintOp::image(bounds, ImageNode::new(1, Some(svg.into_bytes())), None)
    };
    let tree = PageLayerTree::new(64.0, 48.0, LayerNode::leaf(bounds, Some(1), vec![op]));
    let png = SkiaLayerRenderer::new().render_png(&tree).unwrap();
    (image::load_from_memory(&png).unwrap().to_rgba8(), bounds)
}

fn coverage(image: &image::RgbaImage, bounds: BoundingBox) -> f64 {
    let mut total = 0.0;
    let mut count = 0;
    for (x, y, pixel) in image.enumerate_pixels() {
        let x = f64::from(x) + 0.5;
        let y = f64::from(y) + 0.5;
        if x < bounds.x
            || x >= bounds.x + bounds.width
            || y < bounds.y
            || y >= bounds.y + bounds.height
        {
            assert_eq!(pixel[3], 0, "마스크 적분이 원래 프레임 밖을 칠하면 안 된다");
        }
        // 테두리의 재표본 오차를 제외한 내용에서 면적과 색을 함께 검사한다.
        if x > bounds.x + bounds.width * 0.25
            && x < bounds.x + bounds.width * 0.75
            && y > bounds.y + bounds.height * 0.25
            && y < bounds.y + bounds.height * 0.75
        {
            total += f64::from(pixel[3]) / 255.0;
            count += 1;
            if pixel[3] > 64 {
                for (actual, expected) in pixel.0[..3].iter().zip([128i16, 64, 32]) {
                    assert!(
                        (i16::from(*actual) - expected).abs() <= 3,
                        "알파 적분이 원래 색을 바꿨다: {pixel:?}"
                    );
                }
            }
        }
    }
    assert!(count > 0);
    total / f64::from(count)
}

#[test]
fn native_svg_bitmap_mask_keeps_area_coverage_and_its_original_frame() {
    for fragment in [false, true] {
        let (image, bounds) = render(&patterned_mask(), fragment);
        let measured = coverage(&image, bounds);
        assert!(
            (measured - 0.5).abs() < 0.15,
            "동일 면적 흑백 마스크의 피복률이 사라졌다: fragment={fragment}, coverage={measured}"
        );
    }
}

#[test]
fn native_svg_without_mask_keeps_opaque_color_and_original_frame() {
    for fragment in [false, true] {
        let (image, bounds) = render(
            r##"<rect width="320" height="240" fill="#804020"/>"##,
            fragment,
        );
        assert!((coverage(&image, bounds) - 1.0).abs() < 0.01);
    }
}
