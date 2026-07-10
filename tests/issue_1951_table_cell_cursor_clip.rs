//! Issue #1951: 고정 높이 표 셀의 장문 입력 뒤 캐럿이 셀 가시 범위를 벗어나는 회귀.
//!
//! `복학원서.hwp` 첫 표의 대학명 입력 칸은 고정 높이 셀이다. 장문을 입력해도
//! 편집 캐럿과 IME 조합 기준 좌표는 해당 셀 bbox 안에 남아야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

const SAMPLE: &str = "samples/복학원서.hwp";

fn sample_bytes() -> Vec<u8> {
    std::fs::read(SAMPLE).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

fn find_college_value_cell(core: &DocumentCore) -> (usize, usize, usize) {
    for (parent_para_idx, para) in core.document().sections[0].paragraphs.iter().enumerate() {
        for (control_idx, control) in para.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            let Some(label) = table.cells.iter().find(|cell| {
                cell.paragraphs
                    .iter()
                    .any(|paragraph| paragraph.text.contains("Name of College"))
            }) else {
                continue;
            };
            let value_col = label.col + label.col_span;
            let value_idx = table
                .cells
                .iter()
                .position(|cell| cell.row == label.row && cell.col == value_col)
                .expect("대학명 값 입력 셀");
            return (parent_para_idx, control_idx, value_idx);
        }
    }
    panic!("Name of College 표 셀을 찾지 못함");
}

fn parse_json(label: &str, value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|e| panic!("parse {label}: {e}; json={value}"))
}

fn assert_rect_inside_cell(rect: &Value, cell: &Value, label: &str) {
    assert_eq!(
        rect["pageIndex"], cell["pageIndex"],
        "{label}: page mismatch"
    );
    let x = rect["x"].as_f64().expect("cursor x");
    let y = rect["y"].as_f64().expect("cursor y");
    let h = rect["height"].as_f64().expect("cursor height");
    let left = cell["x"].as_f64().expect("cell x");
    let top = cell["y"].as_f64().expect("cell y");
    let right = left + cell["w"].as_f64().expect("cell w");
    let bottom = top + cell["h"].as_f64().expect("cell h");
    const EPSILON: f64 = 0.11;

    assert!(
        x >= left - EPSILON && x <= right + EPSILON,
        "{label}: caret x must stay inside cell: rect={rect}, cell={cell}"
    );
    assert!(
        y >= top - EPSILON && y + h <= bottom + EPSILON,
        "{label}: caret y/height must stay inside cell: rect={rect}, cell={cell}"
    );
}

#[test]
fn long_text_cursor_stays_in_fixed_cell_for_direct_and_path_queries() {
    let bytes = sample_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("parse sample for target lookup");
    let (parent_para, control, cell) = find_college_value_cell(&core);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse sample for edit");
    let text = "가".repeat(160);

    doc.insert_text_in_cell_deferred_pagination(
        0,
        parent_para as u32,
        control as u32,
        cell as u32,
        0,
        0,
        &text,
    )
    .expect("insert long text in college cell with deferred pagination");

    let bboxes = parse_json(
        "table cell bboxes",
        &doc.get_table_cell_bboxes(0, parent_para as u32, control as u32, Some(0))
            .expect("get table cell bboxes"),
    );
    let target_bbox = bboxes
        .as_array()
        .expect("cell bboxes array")
        .iter()
        .find(|bbox| bbox["cellIdx"].as_u64() == Some(cell as u64))
        .expect("target cell bbox");

    let direct = parse_json(
        "direct cell cursor",
        &doc.get_cursor_rect_in_cell(
            0,
            parent_para as u32,
            control as u32,
            cell as u32,
            0,
            text.chars().count() as u32,
        )
        .expect("get direct cell cursor"),
    );
    assert_rect_inside_cell(&direct, target_bbox, "direct cell cursor");
    assert_eq!(
        direct["cellOverflowed"].as_bool(),
        Some(true),
        "지연 페이지네이션 중에는 overflow 상태를 Studio에 알려야 함: {direct}"
    );
    assert_eq!(
        direct["cellBounds"]["x"], target_bbox["x"],
        "direct cell cursor는 활성 셀 bbox를 함께 반환해야 함"
    );

    let path = json!([{
        "controlIndex": control,
        "cellIndex": cell,
        "cellParaIndex": 0,
    }])
    .to_string();
    let by_path = parse_json(
        "path cell cursor",
        &doc.get_cursor_rect_by_path(0, parent_para as u32, &path, text.chars().count() as u32)
            .expect("get path cell cursor"),
    );
    assert_rect_inside_cell(&by_path, target_bbox, "path cell cursor");
    assert_eq!(
        by_path["cellOverflowed"].as_bool(),
        Some(true),
        "path cursor도 지연 레이아웃 overflow를 알려야 함: {by_path}"
    );
}

#[test]
fn one_depth_path_insert_reflows_the_same_as_direct_cell_insert() {
    let bytes = sample_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("parse sample for target lookup");
    let (parent_para, control, cell) = find_college_value_cell(&core);
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse sample for edit");
    let text = "가".repeat(160);
    let path = json!([{
        "controlIndex": control,
        "cellIndex": cell,
        "cellParaIndex": 0,
    }])
    .to_string();

    doc.insert_text_in_cell_by_path(0, parent_para, &[(control, cell, 0)], 0, &text)
        .expect("insert long text through one-depth path");

    let bboxes = parse_json(
        "table cell bboxes",
        &doc.get_table_cell_bboxes(0, parent_para as u32, control as u32, Some(0))
            .expect("get table cell bboxes"),
    );
    let target_bbox = bboxes
        .as_array()
        .expect("cell bboxes array")
        .iter()
        .find(|bbox| bbox["cellIdx"].as_u64() == Some(cell as u64))
        .expect("target cell bbox");
    let rect = parse_json(
        "path cursor after path insert",
        &doc.get_cursor_rect_by_path(0, parent_para as u32, &path, text.chars().count() as u32)
            .expect("get path cursor"),
    );

    assert_rect_inside_cell(&rect, target_bbox, "path cursor after path insert");
    assert_eq!(
        rect["cellOverflowed"].as_bool(),
        Some(false),
        "즉시 재페이지네이션을 거친 one-depth path 삽입은 overflow가 남지 않아야 함: {rect}"
    );
}
