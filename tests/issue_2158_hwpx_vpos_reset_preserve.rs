//! Issue #2158 — HWPX lineseg 재계산이 저장 쪽-상대 vpos 리셋(쪽나눔 인코딩)을
//! 파괴하던 회귀를 막는 페이지 수 핀.
//!
//! HWPX 원본 XML은 HWP5와 동일하게 쪽-상대 vertpos를 저장하며, 직전 문단 대비
//! 급감(예: sample16 pi87 70412 → pi88 568=sb)이 쪽나눔을 인코딩한다. HWPX 로딩은
//! lineSegArray 부재 문단이 있으면 구역 전체 vpos를 재계산하는데, 이때 원본
//! lineseg 문단의 리셋 신호가 누적 좌표로 덮여(568→208008) typeset의 vpos-reset
//! 쪽나눔(#321/#1921)이 무력화 — 동일 문서가 HWP5로는 64쪽(한글 정합), HWPX로는
//! 63쪽이 되던 결함.
//!
//! 권위: 한글 2022 COM per-pi 오라클 (#2154 스윕 + 단건 재검).

use std::fs;
use std::path::Path;

fn page_count_of(rel: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.page_count()
}

#[test]
fn hwp3_sample16_hwpx_matches_hwp5_and_hangul() {
    let hwpx = page_count_of("samples/hwp3-sample16-hwp5.hwpx");
    let hwp5 = page_count_of("samples/hwp3-sample16-hwp5.hwp");
    assert_eq!(
        (hwpx, hwp5),
        (64, 64),
        "동일 문서 HWPX/HWP5 쪽수는 한글 2022(64쪽)와 삼자 일치해야 함. \
         hwpx={hwpx} hwp5={hwp5} — hwpx가 63이면 저장 vpos 리셋 신호 파괴(#2158) 회귀."
    );
}

#[test]
fn onsaemiro_hwpx_page_count_matches_hangul() {
    let pages = page_count_of("samples/[2027] 온새미로 1 본교재.hwpx");
    assert_eq!(
        pages, 47,
        "온새미로 hwpx 기대 47쪽 (한글 2022 오라클). 실측 {pages}p — \
         46p면 vpos 리셋 미보존(#2158) 회귀."
    );
}
