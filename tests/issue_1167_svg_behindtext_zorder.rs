//! Issue #1167: SVG 렌더러 BehindText 워터마크 z-order 결함 (#1017 / PR #1163 후속).
//!
//! `samples/복학원서.hwp` 의 중앙 baked watermark 는 `wrap=BehindText` 이므로
//! 본문 텍스트보다 **뒤(아래)** 에 합성되어야 한다. PR #1163 이 PaintOp replay plane
//! 으로 native Skia(PNG)/CanvasKit(웹캔버스) z-order 를 정정했으나, SVG 렌더러는
//! RenderNode 트리를 단순 DFS 순회하여 워터마크가 본문 `<text>` 뒤(SVG 후순위=위)
//! 에 그려져 본문을 덮었다.
//!
//! SVG 는 문서 순서상 뒤에 오는 요소가 위에 그려지므로, BehindText `<image>` 는
//! 본문 첫 `<text>` 보다 **앞**(작은 줄 번호)에 출현해야 한다.

use std::fs;
use std::path::Path;

fn render_page_svg(rel: &str, page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    doc.render_page_svg(page).expect("render svg")
}

/// SVG 문자열에서 첫 번째 `<text ` 태그의 바이트 오프셋.
fn first_text_offset(svg: &str) -> usize {
    svg.find("<text ").expect("본문 <text> 없음")
}

/// SVG 문자열에서 본문 영역(페이지 배경 이후) BehindText 워터마크 `<image>` 의
/// 바이트 오프셋들. 복학원서는 페이지 배경 1개 + 중앙 워터마크 1개 = `<image>` 2개.
/// 페이지 배경은 항상 첫 `<text>` 앞이므로, 본문 텍스트와 z-order 가 문제되는 것은
/// 중앙 워터마크(2번째 image)다.
fn image_offsets(svg: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut start = 0;
    while let Some(pos) = svg[start..].find("<image") {
        offsets.push(start + pos);
        start += pos + 6;
    }
    offsets
}

#[test]
fn behindtext_watermark_renders_before_body_text_in_svg() {
    let svg = render_page_svg("samples/복학원서.hwp", 0);

    let first_text = first_text_offset(&svg);
    let images = image_offsets(&svg);
    assert!(
        images.len() >= 2,
        "복학원서 SVG 에 페이지 배경 + 중앙 워터마크 image 가 있어야 함 (found {})",
        images.len()
    );

    // 모든 BehindText 워터마크 `<image>` 가 본문 첫 `<text>` 앞(아래)에 그려져야 한다.
    // 중앙 워터마크(본문 텍스트 좌표대에 위치)가 본문 뒤(위)에 오면 본문을 덮는다.
    let watermark_after_text: Vec<usize> = images
        .iter()
        .copied()
        .filter(|&img| img > first_text)
        .collect();

    assert!(
        watermark_after_text.is_empty(),
        "BehindText 워터마크 image(offset {:?})가 본문 첫 text(offset {})보다 뒤 — \
         SVG 후순위로 본문을 덮음 (z-order 결함)",
        watermark_after_text,
        first_text
    );
}
