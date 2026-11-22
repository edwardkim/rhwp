//! 관계형. 중첩 표의 소유 경로와 행 경계를 옮긴 뒤의 열 격자를 비교한다.
//! 주석 갱신 2026-11-23 04:40 KST (+0900) · 직전 파일 변경 2026-11-22T01:42:43+09:00 (eeff0619f5fa5a8b53c9b7779423ccba7fa74754).
//! 2026-11-23 한컴 Print 대조. 96 DPI, 보정 전 2px 실루엣 raw의 전체 쪽 최저값(Native / fresh WASM).
//! nested-table-hwp 전체 1쪽 100.00% / 100.00%.
//! nested-table-hwpx 전체 1쪽 100.00% / 100.00%.
//! single-row-resize-hwp 전체 1쪽 100.00% / 100.00%.
//! single-row-resize-hwpx 전체 1쪽 100.00% / 100.00%.
//! 같은 동작을 실제 파일로 저장한 인쇄 대조군이다. 합성 모델의 값은 테스트가 구성한 입력과 독립 기대값으로 검사한다.
//! 전쪽 결과와 원본은 mydocs/pr/assets/semanticist21-20261123-table-notes-shadows/focus-pages.tsv 및 hancom-print.tsv에 기록했다.
//!
//! - 칸 안 표는 쪽 개체 목록(`getPageControlLayout`)에 바깥 표 칸까지의 `cellPath`로 나온다.
//!   경로가 없으면 칸 문단 번호(0)·컨트롤 번호로만 나와 본문 문단 0의 표처럼 읽힌다.
//! - 한 줄의 칸 경계만 옮기면 한컴 12.30처럼 그 자리에 새 열 경계를 두고 다른 줄 셀을
//!   두 열에 걸친다. 열 수를 그대로 두면 열마다 가장 넓은 셀로 그리는 한컴이 세 줄을 모두
//!   넓혀 그린다. 모든 줄의 같은 경계를 옮기면 열 격자는 그대로다.

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn para_of(json: &str) -> usize {
    let parsed: Value = serde_json::from_str(json).expect("JSON 파싱");
    parsed["paraIdx"].as_u64().expect("paraIdx") as usize
}

fn table_of(doc: &HwpDocument, para_idx: usize) -> &Table {
    doc.document().sections[0].paragraphs[para_idx]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤")
}

fn grid(table: &Table) -> Vec<(u16, u16, u16, rhwp::model::HwpUnit)> {
    table
        .cells
        .iter()
        .map(|c| (c.row, c.col, c.col_span, c.width))
        .collect()
}

#[test]
fn page_control_layout_reports_nested_table_cell_path() {
    let mut doc = HwpDocument::create_empty();
    doc.insert_text_native(0, 0, 0, "A").expect("글 넣기");
    doc.insert_paragraph_native(0, 1).expect("문단 넣기");
    doc.create_table_native(0, 1, 0, 3, 3).expect("바깥 표");
    doc.create_table_in_cell_by_path_native(0, 1, &[(0, 4, 0)], 0, 2, 2, None, None, false)
        .expect("칸 안 표");

    let layout: Value =
        serde_json::from_str(&doc.get_page_control_layout_native(0).expect("개체 목록"))
            .expect("개체 목록 JSON");
    let tables: Vec<&Value> = layout["controls"]
        .as_array()
        .expect("controls")
        .iter()
        .filter(|control| control["type"] == "table")
        .collect();
    assert_eq!(tables.len(), 2, "바깥 표와 칸 안 표: {layout}");
    assert!(tables[0].get("cellPath").is_none(), "본문 표는 경로가 없다");
    assert_eq!(tables[0]["paraIdx"], 1);
    assert_eq!(tables[1]["parentParaIdx"], 1, "{layout}");
    assert_eq!(
        tables[1]["cellPath"],
        serde_json::json!([{ "controlIndex": 0, "cellIndex": 4, "cellParaIndex": 0 }]),
        "칸 안 표는 그 칸까지의 경로를 준다: {layout}"
    );
    assert_eq!(tables[1]["controlIdx"], 0);
    assert_eq!(tables[1]["rowCount"], 2);
}

#[test]
fn resize_one_row_column_border_splits_column_grid_like_hancom() {
    let mut doc = HwpDocument::create_empty();
    let para = para_of(&doc.create_table_native(0, 0, 0, 3, 3).expect("표 만들기"));
    let (width, w) = {
        let table = table_of(&doc, para);
        (table.common.width, table.cells[3].width)
    };
    // 가운데 줄(셀 3·4)의 첫 경계만 3000HU 오른쪽으로 옮긴다.
    doc.resize_table_cells(
        0,
        para as u32,
        0,
        r#"[{"cellIdx":3,"widthDelta":3000},{"cellIdx":4,"widthDelta":-3000}]"#,
    )
    .expect("가운데 줄 경계 옮기기");

    let expected = vec![
        (0, 0, 1, w),
        (0, 1, 2, w),
        (0, 3, 1, w),
        (1, 0, 2, w + 3000),
        (1, 2, 1, w - 3000),
        (1, 3, 1, w),
        (2, 0, 1, w),
        (2, 1, 2, w),
        (2, 3, 1, w),
    ];
    let table = table_of(&doc, para);
    assert_eq!(table.col_count, 4);
    assert_eq!(grid(table), expected);
    assert_eq!(table.common.width, width, "표 너비는 그대로다");

    let reopened =
        HwpDocument::from_bytes(&doc.export_hwp_native().expect("HWP 저장")).expect("다시 열기");
    let table = table_of(&reopened, para);
    assert_eq!(table.col_count, 4);
    assert_eq!(grid(table), expected);
}

#[test]
fn resize_whole_column_border_keeps_column_grid() {
    let mut doc = HwpDocument::create_empty();
    let para = para_of(&doc.create_table_native(0, 0, 0, 3, 3).expect("표 만들기"));
    doc.resize_table_cells(
        0,
        para as u32,
        0,
        r#"[{"cellIdx":0,"widthDelta":3000},{"cellIdx":1,"widthDelta":-3000},{"cellIdx":3,"widthDelta":3000},{"cellIdx":4,"widthDelta":-3000},{"cellIdx":6,"widthDelta":3000},{"cellIdx":7,"widthDelta":-3000}]"#,
    )
    .expect("칸 경계 옮기기");
    let table = table_of(&doc, para);
    assert_eq!(table.col_count, 3);
    assert!(table.cells.iter().all(|c| c.col_span == 1));
}
