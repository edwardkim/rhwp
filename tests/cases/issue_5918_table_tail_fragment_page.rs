//! Issue #5918: 쪽 경계에서 잘린 표의 뒤 조각이 한 쪽을 독차지한다.
//!
//! 재현 형상(samples/issue4514/sample1-repro.hwp — 한글 2020 정본 46쪽):
//! - RowBreak 표(pi=578)의 앞 조각이 쪽을 채우면 꼬리 조각(rows 3..4)이
//!   새 쪽 상단에 배출되는데, 그 새 쪽이 곧 저장 사다리의 리셋 지점
//!   (pi=608 vpos=0)과 같은 물리 경계였다. 리셋 트리거가 한 번 더
//!   advance해 꼬리 조각+빈 줄만 담근 근빈 쪽(used≈78px)이 생기고,
//!   뒤따르는 PMR-002 표(701.4px)가 남은 ≈874px 공간으로 흐르지 못해
//!   문서 전체가 48쪽으로 늘었다(정본 대비 delta +2).
//! - 수정(#5918): 저장 vpos 리셋 시 현재 단이 continuation 꼬리 조각과
//!   빈 필러 문단만 담고 있으면
//!   (`page_holds_only_fresh_table_continuation`) advance를 건너뛴다.
//!   실 내용이 함께 놓인 쪽의 저장 경계는 종전대로 존중한다.
//!
//! 기대: 46쪽(정본 일치), 29쪽(0-based 28)에 PMR-001 꼬리 조각과
//! PMR-002 표(pi=612)가 함께 배치된다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue4514/sample1-repro.hwp";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_5918_tail_fragment_shares_page_with_following_table() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        46,
        "#5918 꼬리 조각의 이중 쪽 경계 제거로 정본과 같은 46쪽이어야 한다"
    );

    // 29쪽(0-based 28): PMR-001 꼬리 조각(pi=578 rows 3..4, continuation)과
    // PMR-002 표(pi=612)가 같은 쪽에 배치된다.
    let page29 = doc.dump_page_items(Some(28));
    assert!(
        page29.contains("pi=578"),
        "29쪽에 PMR-001 꼬리 조각(pi=578)이 있어야 한다\n---\n{}",
        page29
    );
    assert!(
        page29.contains("pi=612"),
        "29쪽에 뒤따르는 PMR-002 표(pi=612)가 흘러 들어와야 한다 — \
         꼬리 조각만 남은 근빈 쪽은 #5918 회귀다\n---\n{}",
        page29
    );
}
