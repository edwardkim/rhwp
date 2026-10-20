//! [#7732] 쪽 나눔(RowBreak) 표의 쪽 끝 행 — 각주 경계와 빈 잔여.
//!
//! 한/글 2020·2024 Print(합성 `samples/issue1937/gen_row_trim.py`, 한 줄 내용에 선언 높이만 큰
//! 행)에서 확인한 두 규칙을 잠근다.
//!
//! 1. 표에 자기 각주가 없고 이 쪽의 각주가 이미 정해졌으면 행을 각주 구분선 바로 위까지
//!    놓는다(40px 여유 없음). `trim_n1_*`: 표 하단 993.3px · 구분선 1005.9px.
//! 2. 쪽 끝 행의 내용이 이 쪽에서 끝나면, 남는 선언 높이가 1282HU(17.1px, 10pt + 위아래
//!    기본 안 여백) 이하일 때 다음 쪽에 이어받지 않고 버린다. 그보다 크면 빈 띠로 이어받는다.
//!    경계는 글자 크기·칸 안 여백·내용 줄 수와 무관하다(20pt 에서도 19.9px 는 이어받는다).
//!
//! ```text
//!   입력                          정본(2020)                  1쪽 끝 행 · 2쪽 시작
//!   trim_n0_F02                   잔여 11.9px 버림            R10 · R11
//!   trim_n0_h6630_F02             잔여 16.0px 버림            R09 · R10
//!   trim_n0_h6660_F02             잔여 19.9px 이어받음        R09 · R09(이어받기)
//!   trim_n0_l1_big_h6500_F01      20pt, 잔여 19.9px 이어받음  R09 · R09(이어받기)
//!   trim_n1_F01                   쪽 각주, R10 을 나눠 놓음    R10 · R10(이어받기)
//!   trim_n1_F04                   쪽 각주, 잔여 6.7px 버림    R09 · R10
//! ```
//!
//! 실물: `samples/issue1937_rowbreak_footnote_overpagination.hwp` 32쪽 표(pi=300)는 정본처럼
//! 행 0–4 를 싣고(행 4 는 선언 131.6px 중 125.3px 만) 33쪽은 행 5 부터 시작한다. 수정 전에는
//! 첫 조각이 행 0–3 이라 33–43쪽이 한 행씩 밀렸다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use serde_json::Value;

/// 문서의 첫 쪽 나눔 표 조각들: (쪽, 시작 행, 끝 행, 시작 컷 있음, 끝 컷 있음).
fn fragments(sample: &str, para: Option<u64>) -> Vec<(usize, u64, u64, bool, bool)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse");
    let pages = core.dump_page_items_json(None);
    let mut out = Vec::new();
    for (page, value) in pages.as_array().unwrap().iter().enumerate() {
        for column in value["columns"].as_array().unwrap() {
            for item in column["items"].as_array().unwrap() {
                if item["kind"] != "partialTable" {
                    continue;
                }
                if para.is_some_and(|p| item["paraIndex"].as_u64() != Some(p)) {
                    continue;
                }
                let has = |key: &str| {
                    item[key]
                        .as_array()
                        .is_some_and(|cut: &Vec<Value>| !cut.is_empty())
                };
                out.push((
                    page,
                    item["startRow"].as_u64().unwrap(),
                    item["endRow"].as_u64().unwrap(),
                    has("startCut"),
                    has("endCut"),
                ));
            }
        }
    }
    out
}

/// 1쪽 끝 행과 2쪽 시작이 정본과 같은지. `continued` 면 같은 행을 2쪽에서 이어받는다.
fn check(sample: &str, last_row: u64, continued: bool) {
    let frags = fragments(sample, None);
    assert!(
        frags.len() >= 2,
        "{sample}: 표가 두 쪽에 나뉘어야 한다 — {frags:?}"
    );
    let (p1, _, end1, _, cut1) = frags[0];
    let (p2, start2, _, start_cut2, _) = frags[1];
    assert_eq!((p1, p2), (0, 1), "{sample}: 조각은 1·2쪽 — {frags:?}");
    assert_eq!(end1 - 1, last_row, "{sample}: 1쪽 끝 행 — {frags:?}");
    assert!(cut1, "{sample}: 1쪽 끝 행은 쪽 끝에서 잘린다 — {frags:?}");
    if continued {
        assert_eq!(
            (start2, start_cut2),
            (last_row, true),
            "{sample}: 남는 선언 높이를 2쪽에서 이어받는다 — {frags:?}"
        );
    } else {
        assert_eq!(
            (start2, start_cut2),
            (last_row + 1, false),
            "{sample}: 남는 빈 선언 높이는 버리고 2쪽은 다음 행부터 — {frags:?}"
        );
    }
}

#[test]
fn page_end_blank_remainder_up_to_default_empty_cell_is_dropped() {
    check("samples/issue1937/trim_n0_F02.hwpx", 10, false);
    check("samples/issue1937/trim_n0_h6630_F02.hwpx", 9, false);
    check("samples/issue1937/trim_n1_F04.hwpx", 9, false);
}

#[test]
fn page_end_blank_remainder_above_default_empty_cell_is_continued() {
    check("samples/issue1937/trim_n0_h6660_F02.hwpx", 9, true);
    // 20pt 내용: 줄 높이(26.7px) 기준이었다면 19.9px 잔여를 버렸다.
    check("samples/issue1937/trim_n0_l1_big_h6500_F01.hwpx", 9, true);
}

#[test]
fn rowbreak_table_without_own_notes_fills_to_known_footnote_boundary() {
    // 40px 여유를 두면 R10 이 통째로 2쪽으로 가고 1쪽 끝은 R09 가 된다.
    check("samples/issue1937/trim_n1_F01.hwpx", 10, true);
}

#[test]
fn issue1937_page32_table_keeps_trimmed_row_four_and_page33_starts_at_row_five() {
    let frags = fragments(
        "samples/issue1937_rowbreak_footnote_overpagination.hwp",
        Some(300),
    );
    assert!(frags.len() >= 2, "표 pi=300 조각 — {frags:?}");
    let (_, start1, end1, _, cut1) = frags[0];
    let (_, start2, _, start_cut2, _) = frags[1];
    assert_eq!(
        (start1, end1, cut1),
        (0, 5, true),
        "32쪽 조각은 행 0–4(행 4 는 각주 경계에서 잘림) — {frags:?}"
    );
    assert_eq!(
        (start2, start_cut2),
        (5, false),
        "33쪽은 행 5 부터(행 4 의 빈 잔여 9.1px 는 이어받지 않음) — {frags:?}"
    );
}
