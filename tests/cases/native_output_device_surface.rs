//! 장치가 확정한 PNG와 원본 bitmap의 재생 소유를 구분한다.
//! 문서 배치의 px/golden이 아닌 합성 paint 프로토콜 검사다.
//! 독립 Print 대조와 실제 문서 전쪽 증거: #7445 stage8 C417.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::paint::{
    LayerNode, PageLayerTree, PaintOp, RenderProfile, ResolvedImageKind, ResolvedImagePayload,
};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode};
use rhwp::renderer::skia::SkiaLayerRenderer;

fn device_pattern() -> Vec<u8> {
    // 원본 샘플의 흑/백과 주기를 명시한다. 문서의 쪽·좌표·글꼴에 의존하지 않는다.
    let source = image::RgbaImage::from_fn(64, 32, |x, _| {
        let gray = if x % 3 == 2 { 255 } else { 0 };
        image::Rgba([gray, gray, gray, 255])
    });
    let mut encoded = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    encoded.into_inner()
}

fn replay(renderer: &SkiaLayerRenderer, bytes: &[u8], kind: ResolvedImageKind) -> image::RgbaImage {
    let bounds = BoundingBox::new(0.0, 0.0, 16.0, 8.0);
    let op = PaintOp::image(
        bounds.clone(),
        ImageNode::new(1, Some(bytes.to_vec())),
        Some(ResolvedImagePayload {
            data: bytes.to_vec(),
            mime: "image/png",
            kind,
            suppress_effects: false,
        }),
    );
    let tree = PageLayerTree::with_profile(
        bounds.width,
        bounds.height,
        LayerNode::leaf(bounds, None, vec![op]),
        RenderProfile::Print,
    );
    let output = renderer
        .render_raster_with_options(
            &tree,
            RasterRenderOptions {
                transparent: false,
                ..Default::default()
            },
        )
        .expect("실제 Native Print 소비자");
    image::load_from_memory(&output.bytes).unwrap().to_rgba8()
}

fn interior_minimum(image: &image::RgbaImage) -> u8 {
    image
        .enumerate_pixels()
        .filter(|(x, y, _)| *x > 0 && *x + 1 < image.width() && *y > 0 && *y + 1 < image.height())
        .map(|(_, _, pixel)| pixel[0])
        .min()
        .unwrap()
}

#[test]
fn native_print_preserves_device_samples_without_changing_source_bitmap_reduction() {
    let bytes = device_pattern();
    let renderer = SkiaLayerRenderer::new();
    let original = replay(&renderer, &bytes, ResolvedImageKind::FormatConverted);
    let device = replay(&renderer, &bytes, ResolvedImageKind::OutputDevice);
    assert!(
        interior_minimum(&device) < interior_minimum(&original),
        "장치의 흑색 표본을 일반 원본 bitmap의 구간 평균으로 다시 지우지 않는다"
    );
    assert!(
        interior_minimum(&device) < 8,
        "최종 장치 패턴의 흑색을 회색 배경으로 바꾸지 않는다"
    );
    // 같은 바이트를 두 경로로 재생해도 cache가 원본 bitmap의 필터를 덮지 않는다.
    assert_eq!(
        replay(&renderer, &bytes, ResolvedImageKind::FormatConverted),
        original,
        "장치 재생 뒤에도 원본 bitmap의 기존 축소를 유지한다"
    );
    assert_eq!(
        replay(&renderer, &bytes, ResolvedImageKind::OutputDevice),
        device,
        "같은 장치 표본의 반복 재생"
    );
}
