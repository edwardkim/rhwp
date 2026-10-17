//! 병합 격자 표의 열 경계는 저장 칸 폭의 행별 누적으로 정한다.
//!
//! 실물 증상: `samples/80168_regulatory_analysis.hwp` 「규제 개요」 24×14 표(17·33·60·73쪽 등)에서
//! 6번 열이 1011HU → 1434HU 로 넓어져 표가 선언 폭 48490HU 보다 423HU(5.64px) 넓게 그려졌다.
//! 한컴 PDF(`pdf/80168_regulatory_analysis-hwp-2024.pdf`) 17쪽 세로 괘선은 모든 행에서
//! 칸 폭 누적 경계와 일치한다(오른쪽 괘선 724.25px = 표 선언 폭).
//! 종전 `resolve_column_widths` 는 단일 칸 최댓값으로 열을 정한 뒤, 미지 열이 둘 이상인 병합
//! 제약을 span 순서대로 균등 분배하고 부족분을 마지막 열에 보탰다. 뒤 행의 병합 칸이 정하는
//! 경계를 앞서 균등 분배로 덮어써 모순이 생기고, 부족분 보정이 표를 선언 폭 밖으로 밀었다.
//!
//! 합성 표본 `samples/table_merged_grid_column_boundaries.hwpx`
//! (`tools/fixture_generators/build_merged_grid_column_fixture.py`, 비-TAC 표 5개):
//! - T1 6열: 경계가 병합 칸으로만 정해지고 1행 앞쪽은 세로 병합에 덮인다(결함 형상).
//!   종전 해석은 열 폭 10500/10500/6000/4500/3000/3000HU(합 37500)를 선언 36000 으로 비례 축소해
//!   0행 칸 경계가 x=344.4px 로 그려졌다(한컴 PDF 355.5px, 저장 칸 폭 21000HU → 355.6px).
//! - T2 4열: 두 행이 경계 1 에서 모순(9000 vs 12000) — 새 해석이 적용되지 않는 반례.
//! - T3 3열: 행은 일관되나 칸 폭 합 35800 이 선언 36000 보다 작음 — 새 해석이 적용되지 않는 반례.
//! - T4 3열: 단일 행·병합 없음.
//! - T5 14열: 80168 「규제 개요」 24×14 표의 서로 다른 행 구획(저장 칸 폭 그대로)만 남긴
//!   7행 축소판. 0번 열만 4800 → 4498HU 로 줄여 표 폭 = 본문 폭 48188HU. 종전 해석은
//!   실물과 같이 표 오른쪽을 선언 폭보다 5.6px 밖으로 민다.
//!
//! 한컴 기준 `pdf/table_merged_grid_column_boundaries-2020.pdf`(engine 2020 Print, 같은 표본):
//! T1·T4·T5 의 세로 괘선은 모든 행에서 저장 칸 폭 누적 경계·선언 폭과 0.5px 안에서 일치한다.
//! 한컴이 이 표본을 HWP 로 다시 저장해도 모든 칸의 행·열·병합·칸 폭은 입력과 같고,
//! 표 선언 폭만 T2 36000 → 39000, T3 36000 → 35800 으로 고친다. 즉 한컴은 T2 를 열마다
//! 최댓값 격자(0번 열 12000, 표 39000)로, T3 을 칸 폭 합 35800 으로 그린다. 두 반례에 대한
//! rhwp 해석(행별 선언 구획 #5590, 마지막 열 보정)은 이 수정 전후로 같고 한컴과 다르다 —
//! 이 파일은 그 두 표의 칸 폭을 고정하지 않고, 한컴과 rhwp 가 함께 지키는 관계(같은 행 끝
//! 정렬, 첫 칸 폭)만 잰다.
//!
//! 기대값은 저장 칸 폭(cellSz)과 선언 폭(sz)에서 직접 나온다. 렌더 트리의 칸 상자 폭,
//! 같은 경계를 공유하는 칸의 x, 표 상자 오른쪽 끝 관계만 잰다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;
use std::path::Path;

const SAMPLE: &str = "samples/table_merged_grid_column_boundaries.hwpx";
const TOL: f64 = 0.05;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

#[derive(Debug, Clone, Copy)]
struct CellBox {
    row: u64,
    col: u64,
    x: f64,
    w: f64,
}

#[derive(Debug)]
struct TableBox {
    x: f64,
    w: f64,
    cells: Vec<CellBox>,
}

impl TableBox {
    fn cell(&self, row: u64, col: u64) -> CellBox {
        *self
            .cells
            .iter()
            .find(|c| c.row == row && c.col == col)
            .unwrap_or_else(|| panic!("칸 r{row} c{col} 없음: {:?}", self.cells))
    }
    fn right(&self) -> f64 {
        self.x + self.w
    }
}

fn bbox_xw(node: &Value) -> (f64, f64) {
    let b = node.get("bbox").expect("bbox");
    let f = |k: &str| b.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    (f("x"), f("w"))
}

fn collect_cells(node: &Value, out: &mut Vec<CellBox>) {
    if node.get("type").and_then(Value::as_str) == Some("Table") {
        return;
    }
    if node.get("type").and_then(Value::as_str) == Some("Cell") {
        let (x, w) = bbox_xw(node);
        out.push(CellBox {
            row: node.get("row").and_then(Value::as_u64).expect("row"),
            col: node.get("col").and_then(Value::as_u64).expect("col"),
            x,
            w,
        });
        return;
    }
    for child in node
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        collect_cells(child, out);
    }
}

fn collect_tables(node: &Value, out: &mut Vec<TableBox>) {
    if node.get("type").and_then(Value::as_str) == Some("Table") {
        let (x, w) = bbox_xw(node);
        let mut cells = Vec::new();
        for child in node
            .get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            collect_cells(child, &mut cells);
        }
        out.push(TableBox { x, w, cells });
        return;
    }
    for child in node
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        collect_tables(child, out);
    }
}

fn tables() -> Vec<TableBox> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(
        &std::fs::read(&path).unwrap_or_else(|e| panic!("{SAMPLE} 읽기: {e}")),
    )
    .unwrap_or_else(|e| panic!("{SAMPLE} 열기: {e:?}"));
    let tree = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let tree: Value = serde_json::from_str(&tree.root.to_json()).expect("렌더 트리 JSON");
    let mut out = Vec::new();
    collect_tables(&tree, &mut out);
    assert_eq!(out.len(), 5, "표 5개가 1쪽에 있어야 한다: {out:?}");
    out
}

fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= TOL,
        "{what}: 기대 {expected:.3}, 실제 {actual:.3}"
    );
}

/// (row, col, 저장 칸 폭 HU)
fn assert_stored_widths(table: &TableBox, name: &str, cells: &[(u64, u64, f64)]) {
    assert_eq!(
        table.cells.len(),
        cells.len(),
        "{name} 칸 수: {:?}",
        table.cells
    );
    for &(row, col, width) in cells {
        assert_close(
            table.cell(row, col).w,
            hu(width),
            &format!("{name} r{row} c{col} 칸 폭 = 저장 칸 폭"),
        );
    }
}

#[test]
fn merged_grid_boundaries_follow_stored_cell_widths() {
    let tables = tables();
    let t1 = &tables[0];
    assert_stored_widths(
        t1,
        "T1",
        &[
            (0, 0, 21000.0),
            (0, 2, 15000.0),
            (1, 2, 9000.0),
            (1, 4, 6000.0),
            (2, 0, 27000.0),
            (2, 3, 9000.0),
        ],
    );
    // 같은 경계를 공유하는 칸은 같은 x 에서 만난다.
    assert_close(
        t1.cell(1, 2).x,
        t1.cell(0, 2).x,
        "T1 경계2: 1행 c2 왼쪽 = 0행 c2 왼쪽",
    );
    assert_close(
        t1.cell(1, 4).x + t1.cell(1, 4).w,
        t1.cell(0, 2).x + t1.cell(0, 2).w,
        "T1 경계6: 1행·0행 오른쪽 끝",
    );
    assert_close(
        t1.cell(2, 3).x + t1.cell(2, 3).w,
        t1.right(),
        "T1 2행 오른쪽 끝 = 표 오른쪽",
    );
    // 표 오른쪽 경계 = 선언 폭.
    assert_close(t1.w, hu(36000.0), "T1 표 폭 = 선언 폭");
    assert_close(t1.cell(0, 0).x, t1.x, "T1 왼쪽 끝");
}

#[test]
fn contradictory_rows_end_on_one_edge() {
    let tables = tables();
    let t2 = &tables[1];
    assert_eq!(t2.cells.len(), 4, "T2 칸 수: {:?}", t2.cells);
    // 한컴과 rhwp 가 함께 지키는 관계: 두 행은 같은 x 에서 시작하고 같은 x 에서 끝난다.
    // (칸 폭 자체는 한컴 최댓값 격자와 rhwp 행별 구획이 갈린다 — 위 머리말.)
    assert_close(t2.cell(1, 0).x, t2.cell(0, 0).x, "T2 두 행 왼쪽 끝");
    let r0_right = t2.cell(0, 1).x + t2.cell(0, 1).w;
    let r1_right = t2.cell(1, 1).x + t2.cell(1, 1).w;
    assert_close(r0_right, r1_right, "T2 두 행 오른쪽 끝");
    assert_close(r0_right, t2.right(), "T2 행 끝 = 표 오른쪽");
}

#[test]
fn short_rows_stay_aligned_to_each_other_and_the_table_edge() {
    let tables = tables();
    let t3 = &tables[2];
    // 첫 칸은 저장 칸 폭 그대로(한컴 PDF 도 같다).
    assert_close(t3.cell(0, 0).w, hu(10000.0), "T3 r0 c0");
    assert_close(t3.cell(1, 0).w, hu(22000.0), "T3 r1 c0");
    // 두 행의 오른쪽 끝은 서로 같고 표 상자 오른쪽에 닿는다(한컴은 그 끝이 칸 폭 합 35800,
    // rhwp 는 선언 36000 — 이 수정 범위 밖이라 끝 좌표는 고정하지 않는다).
    let r0_right = t3.cell(0, 1).x + t3.cell(0, 1).w;
    let r1_right = t3.cell(1, 2).x + t3.cell(1, 2).w;
    assert_close(r0_right, r1_right, "T3 두 행 오른쪽 끝");
    assert_close(r0_right, t3.right(), "T3 행 끝 = 표 오른쪽");
}

#[test]
fn single_row_without_merge_keeps_cell_widths() {
    let tables = tables();
    let t4 = &tables[3];
    assert_stored_widths(
        t4,
        "T4",
        &[(0, 0, 8000.0), (0, 1, 12000.0), (0, 2, 16000.0)],
    );
    assert_close(t4.w, hu(36000.0), "T4 표 폭 = 선언 폭");
}

#[test]
fn regulatory_overview_grid_keeps_stored_widths_and_declared_edge() {
    let tables = tables();
    let t5 = &tables[4];
    let cells: &[(u64, u64, f64)] = &[
        (0, 0, 4498.0),
        (0, 1, 9328.0),
        (0, 2, 34362.0),
        (1, 1, 9328.0),
        (1, 2, 12414.0),
        (1, 7, 8535.0),
        (1, 9, 13413.0),
        (2, 1, 9328.0),
        (2, 2, 11403.0),
        (2, 6, 11403.0),
        (2, 10, 11556.0),
        (3, 1, 9328.0),
        (3, 2, 5194.0),
        (3, 3, 29168.0),
        (4, 2, 5194.0),
        (4, 3, 23973.0),
        (4, 12, 5195.0),
        (5, 2, 10039.0),
        (5, 5, 14284.0),
        (5, 11, 10039.0),
        (6, 2, 10039.0),
        (6, 5, 14284.0),
        (6, 11, 10039.0),
    ];
    assert_stored_widths(t5, "T5", cells);
    assert_close(t5.w, hu(48188.0), "T5 표 폭 = 선언 폭");
    assert_close(t5.cell(0, 0).x, t5.x, "T5 왼쪽 끝");
    // 행마다 마지막 칸의 오른쪽 끝 = 표 오른쪽(선언 폭).
    for (row, col) in [(0, 2), (1, 9), (2, 10), (3, 3), (4, 12), (5, 11), (6, 11)] {
        let c = t5.cell(row, col);
        assert_close(
            c.x + c.w,
            t5.right(),
            &format!("T5 r{row} 오른쪽 끝 = 표 오른쪽"),
        );
    }
    // 여러 행이 공유하는 경계.
    assert_close(t5.cell(4, 3).x, t5.cell(3, 3).x, "T5 경계3: r3·r4");
    assert_close(t5.cell(6, 5).x, t5.cell(5, 5).x, "T5 경계5: r5·r6");
    for row in 1..=6 {
        assert_close(
            t5.cell(row, 2).x,
            t5.cell(0, 2).x,
            &format!("T5 경계2: r{row}·r0"),
        );
    }
}
