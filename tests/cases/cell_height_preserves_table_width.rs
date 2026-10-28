//! 셀 높이 편집은 행마다 다른 칸 너비와 표의 선언 너비를 바꾸지 않는다.
//! 모델·HWP 원본 헤더와 두 저장 형식의 속성 보존 계약만 검사한다.

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;

fn table(doc: &HwpDocument, paragraph: usize) -> &Table {
    doc.document().sections[0].paragraphs[paragraph]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표")
}

fn created_table() -> (HwpDocument, usize, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let created: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 0, 3, 2).unwrap()).unwrap();
    (
        doc,
        created["paraIdx"].as_u64().unwrap() as usize,
        created["controlIdx"].as_u64().unwrap() as usize,
    )
}

fn assert_widths(doc: &HwpDocument, paragraph: usize, width: u32, cells: &[u32]) {
    let table = table(doc, paragraph);
    assert_eq!(table.common.width, width, "높이 편집은 표 너비를 보존한다");
    assert_eq!(
        table
            .cells
            .iter()
            .map(|cell| cell.width)
            .collect::<Vec<_>>(),
        cells,
        "모든 행의 칸 너비를 보존한다"
    );
    // HWP CommonObjAttr에서 너비는 12..16바이트다. HWPX는 원본 헤더가 없다.
    if let Some(raw_width) = table.raw_ctrl_data.get(12..16) {
        assert_eq!(u32::from_le_bytes(raw_width.try_into().unwrap()), width);
    }
}

#[test]
fn height_only_preserves_independent_row_widths_and_both_exports() {
    let (mut original, paragraph, control) = created_table();
    original
        .resize_table_cells(
            0,
            paragraph as u32,
            control as u32,
            r#"[{"cellIdx":2,"widthDelta":900},{"cellIdx":3,"widthDelta":-900}]"#,
        )
        .unwrap();
    let width = table(&original, paragraph).common.width;
    let cells: Vec<u32> = table(&original, paragraph)
        .cells
        .iter()
        .map(|cell| cell.width)
        .collect();
    assert_ne!(cells[0], cells[2], "가운데 행만 경계를 옮긴 표");

    for source in [
        original.export_hwp().unwrap(),
        original.export_hwpx().unwrap(),
    ] {
        let mut doc = HwpDocument::from_bytes(&source).unwrap();
        assert_widths(&doc, paragraph, width, &cells);
        let had_raw = !table(&doc, paragraph).raw_ctrl_data.is_empty();
        doc.set_cell_properties_native(0, paragraph, control, 0, r#"{"height":3000}"#)
            .unwrap();
        assert_widths(&doc, paragraph, width, &cells);
        assert_eq!(table(&doc, paragraph).cells[0].height, 3000);
        assert_eq!(!table(&doc, paragraph).raw_ctrl_data.is_empty(), had_raw);
        for bytes in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
            let reopened = HwpDocument::from_bytes(&bytes).unwrap();
            assert_widths(&reopened, paragraph, width, &cells);
            assert_eq!(table(&reopened, paragraph).cells[0].height, 3000);
        }
    }
}

#[test]
fn combined_width_and_height_edit_still_updates_the_declared_width() {
    let (mut doc, paragraph, control) = created_table();
    let width = table(&doc, paragraph).common.width;
    let cell_width = table(&doc, paragraph).cells[0].width;
    doc.set_cell_properties_native(
        0,
        paragraph,
        control,
        0,
        &format!(r#"{{"width":{},"height":3000}}"#, cell_width + 900),
    )
    .unwrap();
    assert_eq!(table(&doc, paragraph).common.width, width + 900);
    assert_eq!(table(&doc, paragraph).cells[0].width, cell_width + 900);
    assert_eq!(table(&doc, paragraph).cells[0].height, 3000);
}
