//! [#7515] 셀 합치기는 합쳐지는 셀의 문단을 통째로 옮긴다.
//!
//! 종전 `Table::merge_cells` 는 글자가 있는 문단만 골라 `..Default::default()` 로 다시
//! 만들었다. 누름틀 같은 컨트롤과 필드 범위가 빠지고, 빈 문단과 컨트롤만 있는 문단은
//! 통째로 사라졌다. 빈 셀(빈 문단 하나)만 건너뛴다.

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;

fn table_of(doc: &HwpDocument, (para, ctrl): (usize, usize)) -> &Table {
    match &doc.document().sections[0].paragraphs[para].controls[ctrl] {
        Control::Table(table) => table,
        _ => panic!("표 컨트롤"),
    }
}

fn blank_document() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    doc
}

/// 1행 표를 만들고 (문단, 컨트롤) 인덱스를 돌려준다.
fn create_table(doc: &mut HwpDocument, cols: u16) -> (usize, usize) {
    let created = doc
        .create_table_native(0, 0, 0, 1, cols)
        .expect("표 만들기");
    let parsed: serde_json::Value = serde_json::from_str(&created).expect("createTable JSON");
    let index = |key: &str| parsed[key].as_u64().expect(key) as usize;
    (index("paraIdx"), index("controlIdx"))
}

fn merged_texts(doc: &HwpDocument, at: (usize, usize)) -> Vec<String> {
    let table = table_of(doc, at);
    let cell = table.cell_at(0, 0).expect("합친 셀");
    cell.paragraphs.iter().map(|p| p.text.clone()).collect()
}

fn field_names(doc: &HwpDocument) -> Vec<String> {
    let list: serde_json::Value =
        serde_json::from_str(&doc.get_field_list_json()).expect("필드 목록 JSON");
    list.as_array()
        .expect("필드 배열")
        .iter()
        .filter(|f| f["cellField"] == false)
        .map(|f| f["name"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn merge_keeps_click_here_field_from_secondary_cell() {
    let mut doc = blank_document();
    let at = create_table(&mut doc, 2);
    doc.insert_text_in_cell_native(0, at.0, at.1, 0, 0, 0, "A")
        .expect("셀 0 글자");
    doc.insert_click_here_field_at_in_cell(0, at.0, at.1, 1, 0, 0, false, "이름", "", "name", true)
        .expect("셀 1 누름틀");
    assert_eq!(field_names(&doc), vec!["name"]);

    doc.merge_table_cells_native(0, at.0, at.1, 0, 0, 0, 1)
        .expect("셀 합치기");

    assert_eq!(field_names(&doc), vec!["name"], "합친 뒤 누름틀");
    let table = table_of(&doc, at);
    let cell = table.cell_at(0, 0).expect("합친 셀");
    assert_eq!(cell.paragraphs.len(), 2);
    let moved = &cell.paragraphs[1];
    assert!(matches!(moved.controls.as_slice(), [Control::Field(_)]));
    assert_eq!(moved.field_ranges.len(), 1);
    assert_eq!(moved.ctrl_data_records.len(), moved.controls.len());

    let hwp = doc.export_hwp_native().expect("HWP 저장");
    let reopened = HwpDocument::from_bytes(&hwp).expect("HWP 다시 열기");
    assert_eq!(field_names(&reopened), vec!["name"], "HWP 다시 열기");

    let hwpx = doc.export_hwpx_native().expect("HWPX 저장");
    let reopened = HwpDocument::from_bytes(&hwpx).expect("HWPX 다시 열기");
    assert_eq!(field_names(&reopened), vec!["name"], "HWPX 다시 열기");
}

#[test]
fn merge_keeps_empty_paragraphs_and_skips_empty_cell() {
    let mut doc = blank_document();
    let at = create_table(&mut doc, 3);
    doc.insert_text_in_cell_native(0, at.0, at.1, 0, 0, 0, "A")
        .expect("셀 0 글자");
    doc.insert_text_in_cell_native(0, at.0, at.1, 1, 0, 0, "B")
        .expect("셀 1 글자");
    doc.split_paragraph_in_cell_native(0, at.0, at.1, 1, 0, 1, None)
        .expect("셀 1 문단 나누기");

    doc.merge_table_cells_native(0, at.0, at.1, 0, 0, 0, 2)
        .expect("셀 합치기");

    // 셀 1의 빈 둘째 문단은 남고, 빈 셀 2는 문단을 보태지 않는다.
    assert_eq!(merged_texts(&doc, at), vec!["A", "B", ""]);
}
