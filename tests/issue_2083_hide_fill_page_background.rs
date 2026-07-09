//! Issue #2083: `hide_fill`(쪽 배경 감추기) 페이지가 export-png 에서 전체 검정으로 렌더.
//!
//! 원인: `hide_fill=true` 페이지는 pageBackground 노드를 통째로 스킵했고, native-skia raster
//! 기본 clear 가 투명이라 pageBackground 미방출 시 surface 가 투명으로 남아 RGB flatten 시
//! 페이지 전체가 검게 나왔다. (SVG/render-tree/한컴은 정상 — raster 전용 증상.)
//!
//! 수정: hide_fill 시에도 흰 종이 바탕 pageBackground 를 방출한다. 이 테스트는 해당 페이지가
//! 투명/검정이 아니라 불투명 흰 종이로 렌더되는지 검증한다.
//!
//! 표본 `samples/issue2083_hide_fill_page.hwpx` 4쪽(idx 3)이 `hide_fill=true` 페이지.
#![cfg(feature = "native-skia")]

use rhwp::document_core::DocumentCore;
use std::fs;

#[test]
fn hide_fill_page_renders_opaque_white_not_transparent_black() {
    let bytes = fs::read("samples/issue2083_hide_fill_page.hwpx").expect("load issue2083 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");

    // 4쪽(0-기반 3): hide_fill=true 페이지.
    let png = core.render_page_png_native(3).expect("render page png");
    let img = image::load_from_memory(&png)
        .expect("decode png")
        .to_rgba8();
    let (w, h) = img.dimensions();
    assert!(w > 0 && h > 0);

    let pixels: Vec<&image::Rgba<u8>> = img.pixels().collect();
    let total = pixels.len() as f64;

    // 투명(알파 0) 픽셀 비율 — 버그 시 ~99%. 수정 후 거의 0 이어야 한다.
    let transparent = pixels.iter().filter(|p| p.0[3] == 0).count() as f64;
    let transparent_ratio = transparent / total;
    assert!(
        transparent_ratio < 0.02,
        "hide_fill 페이지가 투명하게 렌더됨(알파0 {:.1}%) — pageBackground 흰 바탕 누락 회귀(#2083)",
        transparent_ratio * 100.0
    );

    // 흰(밝은) 배경 픽셀 비율 — 흰 종이 바탕이 대부분을 차지해야 한다.
    let bright = pixels
        .iter()
        .filter(|p| p.0[3] == 255 && p.0[0] > 240 && p.0[1] > 240 && p.0[2] > 240)
        .count() as f64;
    assert!(
        bright / total > 0.5,
        "hide_fill 페이지의 흰 종이 바탕 비율이 낮음({:.1}%) — #2083 회귀",
        bright / total * 100.0
    );
}
