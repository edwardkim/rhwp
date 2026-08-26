//! Issue #1035: HWP3 vs HWP5 변환본 페이지 alignment 회귀 가드.
//!
//! Task #1035 가 PR #1009 (Task #1007, closed) 의 vpos reset 휴리스틱을 narrow 가드
//! 적용하여 적용 (high_threshold 0.85→0.95, aux_trigger 제거). sample16-hwp5 페이지 수
//! 64 유지 (over-split 회피) + alignment 24/64 → 60/64.

use rhwp::wasm_api::HwpDocument;

fn assert_sample16_hwp5_page_count_64(path: &str, label: &str) {
    let bytes = std::fs::read(path).expect("read");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let pages = doc.page_count();
    assert_eq!(
        pages, 64,
        "{label} 페이지 수 64 유지 (PR #1009 over-split 회귀 재발 방지)"
    );
}

/// sample16-hwp5 페이지 수 = 64 단언 — PR #1009 의 over-split (65) 회귀 재발 방지.
#[test]
fn hwp3_sample16_hwp5_page_count_64() {
    assert_sample16_hwp5_page_count_64("samples/hwp3-sample16-hwp5.hwp", "sample16-hwp5");
}

#[test]
fn hwp3_sample16_hwp5_2018_page_count_64() {
    assert_sample16_hwp5_page_count_64("samples/hwp3-sample16-hwp5-2018.hwp", "sample16-hwp5-2018");
}

#[test]
fn hwp3_sample16_hwp5_2022_page_count_64() {
    assert_sample16_hwp5_page_count_64("samples/hwp3-sample16-hwp5-2022.hwp", "sample16-hwp5-2022");
}

#[test]
fn hwp3_sample16_hwp5_2024_page_count_64() {
    assert_sample16_hwp5_page_count_64("samples/hwp3-sample16-hwp5-2024.hwp", "sample16-hwp5-2024");
}
