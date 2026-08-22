//! [#5910] 병합 셀 선언 높이가 걸친 행합보다 **작을** 때 마지막 걸침 행이 차이를 흡수한다.
//!
//! HWP 표는 행 높이를 셀마다 따로 저장하므로 `row_span>1` 셀 선언과 그 셀이 걸친 행들의
//! `row_span==1` 선언 합이 어긋난 문서가 있다. 걸침 선언이 **더 클** 때 잔여를 마지막
//! 걸침 행에 더하는 규칙은 이미 있었지만(#2291/#2237), 반대 방향에는 규칙이 없어 걸침
//! 묶음이 실제보다 부풀었다. 걸침 묶음은 행 단위로 쪼갤 수 없으므로 묶음 전체가 다음
//! 쪽으로 밀려 문서가 한글보다 길어진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

use rhwp::model::shape::CommonObjAttr;
use rhwp::model::table::{Cell, Table};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/kps-ai.hwp")
        .to_string_lossy()
        .into_owned()
}

fn page_texts() -> Vec<String> {
    let out = Command::new(rhwp_bin())
        .args(["export-text", &sample(), "--json"])
        .output()
        .expect("export-text 실행");
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON 파싱");
    v["pages"]
        .as_array()
        .expect("pages 배열")
        .iter()
        .map(|p| p["text"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// 한글 2022 정본(`pdf/kps-ai-2022.pdf`)은 77쪽이다.
#[test]
fn kps_ai_page_count_matches_hangul_master() {
    let pages = page_texts();
    assert_eq!(
        pages.len(),
        77,
        "한글 정본 77쪽과 같아야 한다 (43쪽 평가기준 표의 rs=3 묶음이 통째로 밀리면 78쪽)"
    );
}

/// 정본 43쪽(0기준 45쪽)은 「프로젝트 지원(15점)」 rs=3 묶음까지 한 쪽에 담는다.
#[test]
fn kps_ai_keeps_rowspan_block_on_declared_page() {
    let pages = page_texts();
    let p45 = &pages[45];
    for needle in ["교육훈련", "사후관리", "기술이전"] {
        assert!(
            p45.contains(needle),
            "정본 43쪽에 있어야 할 `{needle}` 이 rhwp 45쪽에 없다"
        );
    }
    assert!(
        !pages[46].contains("교육훈련"),
        "rs=3 묶음이 다음 쪽으로 밀리면 안 된다"
    );
}

fn cell(row: u16, col: u16, row_span: u16, height: u32) -> Cell {
    let mut c = Cell::new_empty(col, row, 6000, height, 0);
    c.row_span = row_span;
    c
}

/// 저장 표 높이(`common.height`)가 축소 결과를 확인해 줄 때만 적용한다.
#[test]
fn shrink_requires_declared_table_height_corroboration() {
    // 3행: 선언 1000/1000/1000, r1 rs=2 선언 1800 → 마지막 행이 200 을 흡수해야
    // 행합 2800 = common.height 가 된다.
    let mut table = Table {
        row_count: 3,
        col_count: 2,
        common: CommonObjAttr {
            height: 2800,
            ..Default::default()
        },
        ..Default::default()
    };
    table.cells.push(cell(0, 0, 1, 1000));
    table.cells.push(cell(0, 1, 1, 1000));
    table.cells.push(cell(1, 0, 2, 1800));
    table.cells.push(cell(1, 1, 1, 1000));
    table.cells.push(cell(2, 1, 1, 1000));

    assert_eq!(
        table.rowspan_declared_overflow_shrink(),
        vec![0, 0, 200],
        "마지막 걸침 행이 200 을 흡수해야 한다"
    );

    // 저장 표 높이가 확인해 주지 않으면 종전 동작(축소 없음)을 유지한다.
    table.common.height = 3000;
    assert_eq!(
        table.rowspan_declared_overflow_shrink(),
        vec![0, 0, 0],
        "common.height 가 닫히지 않으면 축소하지 않는다"
    );
}

/// 손상 선언(걸침 선언 0)은 따라가지 않는다 — 확인 조건이 걸러 낸다.
#[test]
fn damaged_span_declaration_is_ignored() {
    let mut table = Table {
        row_count: 2,
        col_count: 2,
        common: CommonObjAttr {
            height: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    table.cells.push(cell(0, 0, 2, 0));
    table.cells.push(cell(0, 1, 1, 750));
    table.cells.push(cell(1, 1, 1, 750));

    assert_eq!(
        table.rowspan_declared_overflow_shrink(),
        vec![0, 0],
        "걸침 선언 0 인 손상 표는 축소 대상이 아니다"
    );
}
