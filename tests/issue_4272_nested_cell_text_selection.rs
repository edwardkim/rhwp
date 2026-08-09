//! Issue #4272: 중첩 표 안쪽 셀의 논리 텍스트 선택은 생성되지만 평면 셀 rect API가
//! 안쪽 TextRun을 찾지 못해 Studio 하이라이트가 표시되지 않았다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::layout::CellContext;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;
use std::fs;
use std::path::Path;

const FIXTURE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";
const TARGET_TEXT: &str = "23,504";
const TARGET_PATH: &str = r#"[{"controlIndex":1,"cellIndex":0,"cellParaIndex":0},{"controlIndex":2,"cellIndex":0,"cellParaIndex":12},{"controlIndex":0,"cellIndex":50,"cellParaIndex":0}]"#;
const PAGE11_PATH: &str = r#"[{"controlIndex":1,"cellIndex":0,"cellParaIndex":0},{"controlIndex":2,"cellIndex":0,"cellParaIndex":84},{"controlIndex":0,"cellIndex":0,"cellParaIndex":22}]"#;
const PAGE11_SELECTED_TEXT: &str = " 다른 목적 등을 위하여 조사권을 남용하여";
const PAGE5_SELECTED_TABLE_OWNER_PATH: &[(usize, usize, usize)] = &[(1, 0, 0), (2, 0, 12)];

fn fixture_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn find_text_context(node: &RenderNode, target: &str) -> Option<CellContext> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == target {
            return run.cell_context.clone();
        }
    }
    node.children
        .iter()
        .find_map(|child| find_text_context(child, target))
}

#[test]
fn issue_4272_path_api_returns_rects_for_page5_innermost_cell_text() {
    let bytes = fixture_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("parse #4272 fixture");
    assert_eq!(core.page_count(), 17, "#4069 17쪽 pagination 계약");
    let page5 = core
        .build_page_render_tree(4)
        .expect("render physical page 5");
    let context = find_text_context(&page5.root, TARGET_TEXT)
        .expect("physical page 5 `23,504` TextRun cell context");
    let path = context
        .path
        .iter()
        .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
        .collect::<Vec<_>>();
    assert_eq!(context.parent_para_index, 7);
    assert_eq!(path, vec![(1, 0, 0), (2, 0, 12), (0, 50, 0)]);

    let mut document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse HwpDocument");

    let flat = document
        .get_selection_rects_in_cell(0, 7, 1, 0, 0, 0, 0, 6)
        .expect("legacy flat selection rect query");
    assert_eq!(flat, "[]", "평면 바깥 셀 좌표는 깊이 3 run을 매칭하지 않음");

    let rects_json = document
        .get_selection_rects_in_cell_by_path(0, 7, TARGET_PATH, 0, 0, 0, 6)
        .expect("full cellPath selection rect query");
    let rects: Vec<Value> = serde_json::from_str(&rects_json).expect("selection rect JSON");
    assert!(
        !rects.is_empty(),
        "`23,504` 선택 사각형 1개 이상: {rects_json}"
    );
    assert!(
        rects.iter().all(|rect| rect["pageIndex"] == 4),
        "물리 5쪽 선택 사각형: {rects_json}"
    );
    assert!(
        rects
            .iter()
            .all(|rect| rect["width"].as_f64().unwrap_or(0.0) > 0.0),
        "양수 폭 선택 사각형: {rects_json}"
    );

    let copied = document
        .copy_selection_in_cell_by_path(0, 7, TARGET_PATH, 0, 0, 0, 6)
        .expect("full cellPath copy query");
    assert_eq!(
        serde_json::from_str::<Value>(&copied).unwrap()["text"],
        TARGET_TEXT,
        "중첩 셀 선택 plain text"
    );
    assert_eq!(document.get_clipboard_text(), TARGET_TEXT);

    let html = document
        .export_selection_in_cell_html_by_path(0, 7, TARGET_PATH, 0, 0, 0, 6)
        .expect("full cellPath HTML export");
    assert!(html.contains(TARGET_TEXT), "중첩 셀 선택 HTML: {html}");
    assert!(
        html.contains("<!--StartFragment-->"),
        "HTML fragment marker"
    );

    let page11_copied = document
        .copy_selection_in_cell_by_path(0, 7, PAGE11_PATH, 22, 66, 22, 89)
        .expect("physical page 11 full cellPath copy query");
    assert_eq!(
        serde_json::from_str::<Value>(&page11_copied).unwrap()["text"],
        PAGE11_SELECTED_TEXT,
        "물리 11쪽 자식 표 문단 22 선택 plain text"
    );
    assert_eq!(document.get_clipboard_text(), PAGE11_SELECTED_TEXT);

    let page11_html = document
        .export_selection_in_cell_html_by_path(0, 7, PAGE11_PATH, 22, 66, 22, 89)
        .expect("physical page 11 full cellPath HTML export");
    assert!(
        page11_html.contains(PAGE11_SELECTED_TEXT),
        "물리 11쪽 중첩 셀 선택 HTML: {page11_html}"
    );
}

#[test]
fn issue_4272_page5_nested_table_control_copies_from_owner_path() {
    let bytes = fixture_bytes();
    let mut document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse HwpDocument");

    let copied = document
        .copy_control_native(0, 7, PAGE5_SELECTED_TABLE_OWNER_PATH, 0)
        .expect("copy physical page 5 nested table control");

    assert!(copied.contains("[표]"), "중첩 표 복사 결과: {copied}");
    assert!(document.has_internal_clipboard_native());
    assert!(document.clipboard_has_control_native());
}
