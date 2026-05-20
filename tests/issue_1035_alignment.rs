//! Issue #1035: HWP3 vs HWP5 변환본 페이지 alignment 회귀 가드.
//!
//! Task #1035 가 PR #1009 (Task #1007, closed) 의 vpos reset 휴리스틱을 narrow 가드
//! 적용하여 적용 (high_threshold 0.85→0.95, aux_trigger 제거). sample16-hwp5 페이지 수
//! 64 유지 (over-split 회피) + alignment 24/64 → 60/64.

use rhwp::wasm_api::HwpDocument;

/// sample16-hwp5 페이지 수 = 64 단언 — PR #1009 의 over-split (65) 회귀 재발 방지.
#[test]
fn hwp3_sample16_hwp5_page_count_64() {
    let bytes = std::fs::read("samples/hwp3-sample16-hwp5.hwp").expect("read");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let pages = doc.page_count();
    assert_eq!(
        pages, 64,
        "sample16-hwp5 페이지 수 64 유지 (PR #1009 over-split 회귀 재발 방지)"
    );
}
