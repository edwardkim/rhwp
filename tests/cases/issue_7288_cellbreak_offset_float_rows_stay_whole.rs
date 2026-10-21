//! [#7288] 값 1 «셀 단위로 나눔» 표는 세로 오프셋이 있는 자리차지 표여도 행을 자르지 않는다.
//!
//! 표 첫 단위를 다음 쪽으로 이월하는 게이트는 양수 세로 오프셋의 자리차지 float 을 원자 규칙에서
//! 뺀다(표만 옮기면 저장 사다리를 따르는 뒤 본문과 좌표계가 어긋난다, #7198 축). 그 제외가 이미
//! 시작한 표의 조각 끝에도 걸려 값 1 표가 쪽 끝 행을 행 안에서 잘랐다.
//!
//! 한/글 2020·2024 Print(합성 `samples/issue7288/gen_cellbreak_rows.py`: 2열 30행 자리차지 표,
//! 오른쪽 칸 7줄, 문단 기준 세로 오프셋 502HU)는 값 1 표의 행을 쪽 경계에서 자르지 않고 쪽 바닥을
//! 비운 채 다음 쪽에 통째로 놓는다 — 쪽별 행 R00–R04 · R05–R10 · R11–R16 · R17–R22 · R23–R28 · R29.
//! 같은 입력의 값 2 «나눔» 은 행 안을 자른다(대조군).
//!
//! 실물: 10k `유해화학물질 비상대응지침` 구역 6 의 2232×7 값 1 표(세로 오프셋 502HU)는 정본처럼
//! 행을 통째로 넘긴다(종전 rhwp 는 행 안을 잘라 쪽마다 행이 앞당겨졌다).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use serde_json::Value;

/// 첫 표 조각들: (쪽, 시작 행, 끝 행, 시작 컷 있음, 끝 컷 있음).
fn fragments(sample: &str) -> Vec<(usize, u64, u64, bool, bool)> {
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

#[test]
fn cell_unit_table_with_vertical_offset_moves_page_end_rows_whole() {
    let frags = fragments("samples/issue7288/cellbreak_TABLE_v502_L7.hwpx");
    assert!(
        frags
            .iter()
            .all(|&(_, _, _, start_cut, end_cut)| !start_cut && !end_cut),
        "값 1 표는 행 안을 자르지 않는다 — {frags:?}"
    );
    // 조각이 행을 빠짐없이·겹침 없이 잇는다(이어받기 행 없음).
    for pair in frags.windows(2) {
        assert_eq!(
            pair[0].2, pair[1].1,
            "다음 조각은 앞 조각 끝 행에서 — {frags:?}"
        );
    }
    let rows: Vec<(u64, u64)> = frags.iter().map(|&(_, s, e, _, _)| (s, e)).collect();
    assert_eq!(
        rows,
        vec![(0, 5), (5, 11), (11, 17), (17, 23), (23, 29), (29, 30)],
        "정본 쪽별 행"
    );
}

#[test]
fn row_break_table_with_vertical_offset_still_cuts_inside_rows() {
    let frags = fragments("samples/issue7288/cellbreak_CELL_v502_L7.hwpx");
    assert!(
        frags
            .iter()
            .skip(1)
            .all(|&(_, _, _, start_cut, _)| start_cut),
        "값 2 «나눔» 표는 쪽 끝 행 안을 잘라 이어받는다 — {frags:?}"
    );
}
