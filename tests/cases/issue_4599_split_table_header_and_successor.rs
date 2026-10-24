//! Issue #4599 후속: `1790387-202300151 정책연구용역 중간진도보고서.hwpx` 의 90% 미만 쪽 두 결함.
//!
//! 표본은 원본에서 한 구역의 문단 0(구역 속성)과 대상 문단 범위만 남긴 것이다. 기준 PDF 는 같은
//! 표본을 한컴 2020 엔진으로 변환한 `pdf/1790387_*-2020.pdf` 다(쪽수 4 로 rhwp 와 같다).
//!
//! 1. `1790387_rowspan_header_repeat.hwpx` (원본 0구역 pi83..90, 원본 6쪽)
//!    34×6 표의 제목 셀은 행 0 의 "지표 구분" 하나뿐이지만 같은 행의 "사업지역(대조지역)"·"권역" 이
//!    2행 걸친다. 한/글은 이어받기 쪽에서 행 0·1 을 함께 반복한다(PDF 3쪽에 "대·소·기관명·성명").
//!    수정 전 rhwp 는 행 0 만 반복해 걸친 셀을 잘랐다(Native 48.9% → 98.8%).
//!
//! 2. `1790387_tac_successor_after_split.hwpx` (원본 2구역 pi227..241, 원본 57쪽)
//!    빈 호스트 오프셋 표(pi11)의 끝 조각 뒤 글자처럼 표 문단(pi12, 위 간격 0)은 저장 줄
//!    vpos 15555 — 조각 아래 + 바깥 아래 여백 — 에서 시작한다. 수정 전 rhwp 는 위 간격이 0 인
//!    뒤 문단의 저장 줄을 인증하지 않아 호스트 아래 간격 13.3px 을 다시 더했다(59.7% → 99.2%).
//!
//! 검사는 관계로 한다: 쪽수, 반복 제목행의 행 구성과 본문 행보다 위, 본문 행의 누락·중복,
//! 같은 쪽 글자처럼 표 문단들의 간격이 저장 사다리의 간격과 같음.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const HU_PER_PX: f64 = 75.0;

#[test]
fn leading_header_rows_close_over_a_non_header_rowspan() {
    use rhwp::model::table::{Cell, Table};
    // 제목 셀은 행 0 만 덮지만 같은 행의 일반 셀이 2행 걸침 → 반복 블록은 [0,1]
    let mut cells = Vec::new();
    for row in 0..4u16 {
        for col in 0..2u16 {
            cells.push(Cell::new_empty(col, row, 3600, 1000, 1));
        }
    }
    let mut table = Table {
        row_count: 4,
        col_count: 2,
        cells,
        ..Default::default()
    };
    table.rebuild_grid();
    let header = table.cell_index_at(0, 0).unwrap();
    table.cells[header].is_header = true;
    let spanning = table.cell_index_at(0, 1).unwrap();
    table.cells[spanning].row_span = 2;
    assert_eq!(table.leading_header_rows(), vec![0, 1]);
}

fn open(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {sample}: {e}"))
}

fn page_tree(document: &rhwp::wasm_api::HwpDocument, page: u32) -> serde_json::Value {
    let json = document
        .get_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree page {page}: {e:?}"));
    serde_json::from_str(&json).expect("parse render tree json")
}

fn is(node: &serde_json::Value, kind: &str) -> bool {
    node.get("type").and_then(|t| t.as_str()) == Some(kind)
}

fn children(node: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    node.get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
}

fn num(node: &serde_json::Value, key: &str) -> Option<u64> {
    node.get(key).and_then(|v| v.as_u64())
}

fn bbox_y(node: &serde_json::Value) -> (f64, f64) {
    let b = node.get("bbox").expect("bbox");
    let y = b["y"].as_f64().expect("y");
    (y, y + b["h"].as_f64().expect("h"))
}

fn tables_of<'a>(node: &'a serde_json::Value, pi: u64, out: &mut Vec<&'a serde_json::Value>) {
    if is(node, "Table") && num(node, "pi") == Some(pi) {
        out.push(node);
        return;
    }
    for child in children(node) {
        tables_of(child, pi, out);
    }
}

fn one_table(tree: &serde_json::Value, pi: u64, page: u32) -> &serde_json::Value {
    let mut found = Vec::new();
    tables_of(tree, pi, &mut found);
    assert_eq!(
        found.len(),
        1,
        "{}쪽에 pi={pi} 표가 하나 있어야 한다",
        page + 1
    );
    found[0]
}

fn body_top(node: &serde_json::Value) -> Option<f64> {
    if is(node, "Body") {
        return Some(bbox_y(node).0);
    }
    children(node).find_map(body_top)
}

/// 표 조각의 직속 셀: (행, 셀 위, 셀 아래)
fn cells(table: &serde_json::Value) -> Vec<(u64, f64, f64)> {
    children(table)
        .filter(|c| is(c, "Cell"))
        .map(|c| {
            let (top, bottom) = bbox_y(c);
            (num(c, "row").expect("row"), top, bottom)
        })
        .collect()
}

#[test]
fn repeated_header_closes_over_rowspans_started_in_the_header_row() {
    const SAMPLE: &str = "samples/issue4599/1790387_rowspan_header_repeat.hwpx";
    const TABLE_PI: u64 = 7;
    let document = open(SAMPLE);
    assert_eq!(document.page_count(), 4, "한/글 2020 PDF 는 4쪽");

    let first = page_tree(&document, 1);
    let second = page_tree(&document, 2);
    let head_cells = cells(one_table(&first, TABLE_PI, 1));
    let tail_cells = cells(one_table(&second, TABLE_PI, 2));

    // 이어받기 조각의 반복 제목: 행 0 과, 행 0 셀이 걸친 행 1.
    let tail_rows: BTreeSet<u64> = tail_cells.iter().map(|c| c.0).collect();
    assert!(
        tail_rows.contains(&0) && tail_rows.contains(&1),
        "이어받기 조각은 제목행 0·1 을 함께 반복해야 한다 — 실제 행 {tail_rows:?}"
    );
    let header_bottom = tail_cells
        .iter()
        .filter(|c| c.0 <= 1)
        .map(|c| c.2)
        .fold(f64::MIN, f64::max);
    let body_top_y = tail_cells
        .iter()
        .filter(|c| c.0 >= 2)
        .map(|c| c.1)
        .fold(f64::MAX, f64::min);
    assert!(
        header_bottom <= body_top_y + 0.5,
        "반복 제목(아래 {header_bottom:.1})이 본문 행(위 {body_top_y:.1})과 겹친다"
    );

    // 본문 행(2..33)은 두 조각에 걸쳐 누락·중복 없이 나뉜다. 쪽 경계를 걸친 행 걸침 셀(예:
    // "음주")만 두 조각에 함께 나온다 — 앞 조각에서 그 셀은 조각 바닥까지 닿는다.
    let head_bottom = bbox_y(one_table(&first, TABLE_PI, 1)).1;
    let head_body: BTreeSet<u64> = head_cells.iter().map(|c| c.0).filter(|r| *r >= 2).collect();
    let tail_body: BTreeSet<u64> = tail_cells.iter().map(|c| c.0).filter(|r| *r >= 2).collect();
    for row in head_body.intersection(&tail_body) {
        assert!(
            head_cells
                .iter()
                .any(|c| c.0 == *row && (c.2 - head_bottom).abs() <= 1.0),
            "본문 행 {row} 이 두 조각에 중복된다(쪽 경계를 걸친 셀이 아니다)"
        );
    }
    let all: BTreeSet<u64> = head_body.union(&tail_body).copied().collect();
    assert_eq!(all, (2..34).collect::<BTreeSet<u64>>(), "본문 행 누락");
}

#[test]
fn tac_successor_after_split_float_keeps_its_stored_line() {
    const SAMPLE: &str = "samples/issue4599/1790387_tac_successor_after_split.hwpx";
    const FLOAT_PI: u64 = 11;
    // 같은 쪽 글자처럼 표 문단과 저장 줄 vpos (원본 pi238·239·240).
    const TAC: [(u64, f64); 3] = [(12, 15555.0), (13, 33769.0), (14, 52832.0)];
    let document = open(SAMPLE);
    assert_eq!(document.page_count(), 4, "한/글 2020 PDF 는 4쪽");

    let page = page_tree(&document, 2);
    let _ = body_top(&page).expect("3쪽 Body");
    let float_bottom = bbox_y(one_table(&page, FLOAT_PI, 2)).1;
    let tops: Vec<f64> = TAC
        .iter()
        .map(|(pi, _)| bbox_y(one_table(&page, *pi, 2)).0)
        .collect();
    assert!(
        tops[0] > float_bottom,
        "pi12 표(위 {:.1})가 앞 표 끝 조각(아래 {float_bottom:.1})과 겹친다",
        tops[0]
    );
    // 같은 쪽 글자처럼 표 문단의 표 위끼리 간격은 저장 사다리 간격이다. 끝 조각 뒤 첫 문단에
    // 호스트 아래 간격이 다시 더해지면 첫 간격만 그만큼 줄어든다(수정 전 241.0 / 저장 242.85).
    // pi13→pi14 의 1.9px 차(252.3 / 254.2)는 수정 전후 같은 별개 잔여라 여기서 검사하지 않는다.
    let stored = (TAC[1].1 - TAC[0].1) / HU_PER_PX;
    let actual = tops[1] - tops[0];
    assert!(
        (actual - stored).abs() <= 1.0,
        "pi12→pi13 표 간격 {actual:.2} ≠ 저장 사다리 {stored:.2}"
    );
    // 4쪽: 다음 표(pi15).
    let next = page_tree(&document, 3);
    let _ = one_table(&next, 15, 3);
}
