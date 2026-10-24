//! 늘여 채우는 SVG 그림은 장치 픽셀 격자에서 한 번만 래스터화한다(#4599 1790387 35쪽).
//! 목적지 크기 래스터를 소수 좌표에 다시 놓으면 한 행을 덮는 가는 선이 이웃 행으로 번진다.
//! 기대값은 SVG 좌표를 목적지로 옮긴 독립 기하이며 실물 문서의 시각 완료 증거는 아니다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode};
use rhwp::renderer::skia::SkiaLayerRenderer;

/// 가로선 하나를 담은 100×20 SVG를 `bounds`에 늘여 그리고 열 가운데의 행별 알파를 돌려준다.
fn row_alpha(bounds: BoundingBox, line_center: f64, stroke_width: f64) -> Vec<u8> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="20" viewBox="0 0 100 20"><path d="M 0 {line_center} L 100 {line_center}" fill="none" stroke="#000000" stroke-width="{stroke_width}"/></svg>"##
    );
    let op = PaintOp::image(bounds, ImageNode::new(1, Some(svg.into_bytes())), None);
    let tree = PageLayerTree::new(128.0, 40.0, LayerNode::leaf(bounds, Some(1), vec![op]));
    let png = SkiaLayerRenderer::new().render_png(&tree).unwrap();
    let image = image::load_from_memory(&png).unwrap().to_rgba8();
    let column = (bounds.x + bounds.width / 2.0) as u32;
    (0..image.height())
        .map(|y| image.get_pixel(column, y)[3])
        .collect()
}

fn assert_single_row(rows: &[u8], row: usize, case: &str) {
    assert!(
        rows[row] >= 240,
        "{case}: 장치 {row}행을 정확히 덮는 선이 흐려졌다: {:?}",
        &rows[row - 2..=row + 2]
    );
    for neighbor in [row - 1, row + 1] {
        assert!(
            rows[neighbor] <= 16,
            "{case}: 한 행을 덮는 선이 이웃 {neighbor}행으로 번졌다: {:?}",
            &rows[row - 2..=row + 2]
        );
    }
}

#[test]
fn stretched_svg_line_on_a_device_row_stays_on_that_row() {
    // 배율 1: 사용자 y 9.5..10.5 + 원점 6.5 = 장치 16..17.
    let rows = row_alpha(BoundingBox::new(4.25, 6.5, 100.0, 20.0), 10.0, 1.0);
    assert_single_row(&rows, 16, "배율 1·반 픽셀 원점");

    // 배율 0.95: 중심 10.5×0.95 + 6.525 = 16.5, 폭 (1/0.95)×0.95 = 1 → 장치 16..17.
    let rows = row_alpha(BoundingBox::new(4.25, 6.525, 95.0, 19.0), 10.5, 1.0 / 0.95);
    assert_single_row(&rows, 16, "배율 0.95·소수 원점");
}

/// SVG의 설명·미사용 정의·단순 그룹은 같은 래스터 그림의 표본 위상을 바꾸지 않는다.
/// 특정 문서 좌표나 현재 출력을 golden으로 삼지 않는 SVG 의미 보존 검사다.
#[test]
fn raster_wrapper_metadata_does_not_change_sampling() {
    use base64::Engine;
    let source = image::RgbaImage::from_fn(64, 32, |x, y| {
        let v = if (x + y) % 3 == 0 { 255 } else { 0 };
        image::Rgba([v, v, v, 255])
    });
    let mut buffer = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut buffer, image::ImageFormat::Png)
        .unwrap();
    let data = base64::engine::general_purpose::STANDARD.encode(buffer.into_inner());
    let picture = format!(r#"<image width="64" height="32" href="data:image/png;base64,{data}"/>"#);
    let render = |content: &str| {
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32" viewBox="0 0 64 32">{content}</svg>"#
        );
        let bounds = BoundingBox::new(4.25, 6.5, 64.0, 32.0);
        let op = PaintOp::image(bounds, ImageNode::new(1, Some(svg.into_bytes())), None);
        let tree = PageLayerTree::new(80.0, 48.0, LayerNode::leaf(bounds, Some(1), vec![op]));
        image::load_from_memory(&SkiaLayerRenderer::new().render_png(&tree).unwrap())
            .unwrap()
            .to_rgba8()
    };
    let plain = render(&picture);
    for wrapped in [
        format!("<title>같은 비트맵</title>{picture}"),
        format!("<defs/><g>{picture}</g>"),
    ] {
        let decorated = render(&wrapped);
        let changed = plain
            .pixels()
            .zip(decorated.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            changed, 0,
            "그림 내용이 같은데 구조 요소로 {changed}픽셀의 표본이 달라졌다"
        );
    }
}
