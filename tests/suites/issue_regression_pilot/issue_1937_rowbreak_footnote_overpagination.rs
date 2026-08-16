//! Issue #1937: 각주가 많은 큰 RowBreak 표가 연속(continuation) 페이지에서 과분할되는
//! 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플): `samples/issue1937_rowbreak_footnote_overpagination.hwp`
//! (정책연구정보서비스 공개 문서 "소상공인 중간보고서(2)", HWP5). 한글 2022 = 50쪽.
//!
//! 결함 본질: `typeset_block_table` 이 표 가용높이 `available = base − total_footnote`
//! 를 한 번 계산한다. 이 표는 셀 각주 22개(projected ~820px)를 가져 시작 페이지의
//! `table_available` 이 ~75.8px 로 좁아지는데, 행 분할 루프가 이 좁은 값을 **모든 연속
//! 페이지에 재사용**해(각주 없는 신선 full-page 인데) 페이지당 ~1행만 배치 → 122행 표가
//! 188쪽으로, 문서 전체가 231쪽으로 폭주(한글 50).
//!
//! 정정: 연속 페이지 `page_avail` 을 시작 페이지 `table_available` 대신 신선 페이지
//! `base_available`(zone offset·border tolerance 유지)로 계산 — 레퍼런스 Paginator
//! `engine.rs:2502-2503` 과 정합. 수정 후 전체 52쪽(pi=306 표 188→9쪽).

use std::fs;
use std::path::Path;

fn load_page_count(rel: &str) -> u32 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    doc.page_count()
}

#[test]
fn rowbreak_footnote_table_does_not_over_paginate() {
    let pages = load_page_count("samples/issue1937_rowbreak_footnote_overpagination.hwp");
    // 수정 전 231쪽(폭주), 수정 후 52쪽, 한글 2022 = 50쪽.
    // 하한(45)은 과소 페이지 회귀, 상한(80)은 연속 페이지 과분할 재발을 잡는다.
    assert!(
        (45..=80).contains(&pages),
        "issue1937: 페이지 수 {pages} 가 기대 범위(45..=80, 한글 50) 밖 — \
         각주 RowBreak 표 연속 페이지 과분할(수정 전 231) 회귀 의심",
    );
}
