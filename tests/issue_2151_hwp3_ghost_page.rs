//! Issue #2151 — HWP3 그림 pgy=0 페이지 시작 후 거짓 쪽 경계(유령 페이지) 핀.
//!
//! HWP3 저장 line_info.pgy 는 한글97 계산 줄 Y 좌표로, 감소 시 새 페이지 신호다.
//! 그림 호스트 문단은 pgy=0 으로 저장되는데, 이 문단이 새 페이지를 시작한 경우
//! `prev_last_pgy`(직전 유효 pgy 기준)를 리셋하지 않으면 다음 문단의 정상
//! 새-페이지 pgy(예: sample14 pi17 pgy=3521 — HWP5 변환본 vpos 14084/4 와 일치)가
//! 이전 페이지 기준(15441)보다 작아 거짓 쪽 경계로 재승격 → 그림만 있는
//! 유령 페이지가 생긴다.
//!
//! 권위: 한글 2022 COM per-pi 오라클 (samples/ 전수 스윕 #2154, hwp3-sample14
//! 한글 11쪽 / hwp3-sample11 한글 151쪽 — 동일 문서 HWP5/HWPX 변환본도 각각
//! 11쪽·151쪽 MATCH).

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
fn hwp3_sample14_no_ghost_image_page() {
    let pages = page_count_of("samples/hwp3-sample14.hwp");
    assert_eq!(
        pages, 11,
        "hwp3-sample14 기대 11쪽 (한글 2022 오라클 + HWP5 변환본 정합). 실측 {}p — \
         12p면 그림 pgy=0 페이지 시작 후 prev_last_pgy 미리셋으로 거짓 쪽 경계(#2151) 회귀.",
        pages
    );
}

#[test]
fn hwp3_sample11_page_count_matches_hangul() {
    let pages = page_count_of("samples/hwp3-sample11.hwp");
    assert_eq!(
        pages, 151,
        "hwp3-sample11 기대 151쪽 (한글 2022 오라클 + HWP5/HWPX 변환본 정합). 실측 {}p (#2151).",
        pages
    );
}
