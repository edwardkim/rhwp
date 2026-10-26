//! 같은 개체 ID를 가진 표가 있어도 빈 논리 행 보존 HWPX 저장이 실패하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table};

fn table(rows: u16, id: u32) -> Table {
    let mut cell = Cell {
        row_span: rows,
        col_span: 1,
        width: 10_000,
        height: 1000 * u32::from(rows),
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    };
    cell.col = 0;
    cell.row = 0;
    let mut table = Table {
        row_count: rows,
        col_count: 1,
        cells: vec![cell],
        ..Default::default()
    };
    table.common.instance_id = id;
    table.common.width = 10_000;
    table.common.height = 1000 * u32::from(rows);
    table.common.treat_as_char = true;
    table.rebuild_grid();
    table
}

#[test]
fn duplicate_table_ids_keep_empty_logical_rows_in_hwpx() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    let host = |table: Table| Paragraph {
        char_count: 9,
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };
    // 둘째 행에 셀 시작이 없는 표와, 같은 ID를 가진 일반 표.
    document.sections[0].paragraphs.push(host(table(2, 7)));
    document.sections[0].paragraphs.push(host(table(1, 7)));
    core.set_document(document);
    let bytes = core
        .export_hwpx_native()
        .expect("중복 ID 표도 HWPX로 저장한다");
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let rows: Vec<u16> = reopened.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(table) => Some(table.row_count),
            _ => None,
        })
        .collect();
    assert_eq!(rows, vec![2, 1]);
}
