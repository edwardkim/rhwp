//! 관계형. 병합 셀의 원본 너비와 행·열 삽입 뒤의 각 열 및 표 너비를 비교한다.
//! 주석 갱신 2026-11-23 04:40 KST (+0900) · 직전 파일 변경 2026-11-22T22:30:20+09:00 (7c505b07e44baa8f1cb3b49c06409c84186fbad1).
//! 2026-11-23 한컴 Print 대조. 96 DPI, 보정 전 2px 실루엣 raw의 전체 쪽 최저값(Native / fresh WASM).
//! merged-grid-source-hwp 전체 1쪽 100.00% / 100.00%.
//! merged-grid-source-hwpx 전체 1쪽 100.00% / 100.00%.
//! merged-grid-row-hwp 전체 1쪽 100.00% / 100.00%.
//! merged-grid-row-hwpx 전체 1쪽 100.00% / 100.00%.
//! merged-grid-column-hwp 전체 1쪽 100.00% / 100.00%.
//! merged-grid-column-hwpx 전체 1쪽 100.00% / 100.00%.
//! 같은 동작을 실제 파일로 저장한 인쇄 대조군이다. 합성 모델의 값은 테스트가 구성한 입력과 독립 기대값으로 검사한다.
//! 전쪽 결과와 원본은 mydocs/pr/assets/semanticist21-20261123-table-notes-shadows/focus-pages.tsv 및 hancom-print.tsv에 기록했다.
//!
//! 홀로 선 셀 없이 가로 병합 셀로만 덮인 열의 폭.
//!
//! 공문 서식(용역내역서)처럼 머리 줄은 `A | B+C | D`, 맨 아래 줄은 `A+B | C+D`로
//! 병합돼 B·C 열에 `col_span == 1` 셀이 하나도 없는 표가 있다. 열 폭을 그런
//! 셀에서만 읽으면 B·C가 기본값 1800으로 떨어져, 줄을 넣을 때 새 줄 셀이 좁아지고
//! 표 전체 폭(개체 폭)이 줄었다.
use rhwp::model::table::{Cell, Table};

const A: u32 = 15_495;
const B: u32 = 9_269;
const C: u32 = 6_226;
const D: u32 = 15_495;

fn form_table() -> Table {
    let mut table = Table {
        row_count: 2,
        col_count: 4,
        row_sizes: vec![3, 2],
        border_fill_id: 1,
        ..Default::default()
    };
    table.common.width = A + B + C + D;
    table.common.height = 2_000;
    let mut push = |col: u16, row: u16, span: u16, width: u32| {
        let mut cell = Cell::new_empty(col, row, width, 1_000, 1);
        cell.col_span = span;
        table.cells.push(cell);
    };
    push(0, 0, 1, A);
    push(1, 0, 2, B + C);
    push(3, 0, 1, D);
    push(0, 1, 2, A + B);
    push(2, 1, 2, C + D);
    table.rebuild_grid();
    table
}

#[test]
fn merged_only_columns_are_solved_from_merged_cells() {
    assert_eq!(form_table().get_column_widths(), vec![A, B, C, D]);
}

#[test]
fn insert_row_keeps_table_width_with_merged_only_columns() {
    let mut table = form_table();
    table.insert_row(0, true).unwrap();

    let new_row: Vec<u32> = table
        .cells
        .iter()
        .filter(|cell| cell.row == 1)
        .map(|cell| cell.width)
        .collect();
    assert_eq!(new_row, vec![A, B, C, D]);
    assert_eq!(table.common.width, A + B + C + D);
}

#[test]
fn insert_column_keeps_merged_only_column_width() {
    let mut table = form_table();
    table.insert_column(1, true).unwrap();

    let new_col: Vec<u32> = table
        .cells
        .iter()
        .filter(|cell| cell.col == 2 && cell.col_span == 1)
        .map(|cell| cell.width)
        .collect();
    assert!(new_col.iter().all(|width| *width == B), "{new_col:?}");
}
