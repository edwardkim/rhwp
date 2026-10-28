//! [#4275] 다른 문서로 중첩 표를 HTML 붙여넣으면 셀존 배경·문단 정렬이 유지된다.
//!
//! 같은 문서 native 클립보드는 스타일을 보존하지만, 새 문서는 HTML 경로로 넘어가
//! `table.zones` 유효 BorderFill 과 CSS 로 만든 `para_shape_id` 가 빠졌다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{
    Alignment, BorderFill, BorderLine, BorderLineType, Fill, FillType, ParaShape, SolidFill,
};
use rhwp::model::table::{Cell, Table, TableZone};

const GREY: u32 = 0x00C0C0C0;

fn grey_fill() -> BorderFill {
    let line = BorderLine {
        line_type: BorderLineType::Solid,
        width: 2,
        color: 0,
    };
    BorderFill {
        borders: [line, line, line, line],
        fill: Fill {
            fill_type: FillType::Solid,
            solid: Some(SolidFill {
                background_color: GREY,
                pattern_color: 0,
                pattern_type: 0,
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn cell(text: &str, para_shape_id: u16) -> Cell {
    let n = text.chars().count() as u32;
    Cell {
        col: 0,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: 4000,
        height: 1000,
        border_fill_id: 1,
        paragraphs: vec![Paragraph {
            text: text.to_string(),
            char_count: n,
            char_offsets: (0..=n).collect(),
            para_shape_id,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn inner_table() -> Table {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell("유형", 1)],
        zones: vec![TableZone {
            start_col: 0,
            start_row: 0,
            end_col: 0,
            end_row: 0,
            border_fill_id: 3,
        }],
        ..Default::default()
    };
    table.common.width = 4000;
    table.common.height = 1000;
    table.rebuild_grid();
    table
}

fn source_document() -> Document {
    let mut center = ParaShape::default();
    center.alignment = Alignment::Center;
    center.line_spacing = 160;
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default(), center];
    doc.doc_info.border_fills = vec![BorderFill::default(), BorderFill::default(), grey_fill()];

    let mut host = Paragraph::default();
    host.controls.push(Control::Table(Box::new(inner_table())));

    let mut section = Section::default();
    section.paragraphs.push(host);
    doc.sections.push(section);
    doc
}

fn pasted_table(core: &DocumentCore) -> &Table {
    for para in &core.document().sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::Table(t) = ctrl {
                return t;
            }
        }
    }
    panic!("붙여넣은 표가 없다");
}

fn cell_fill_color(core: &DocumentCore, table: &Table) -> Option<u32> {
    let cell = &table.cells[0];
    let id = if cell.border_fill_id > 0 {
        cell.border_fill_id
    } else {
        return None;
    };
    let bf = core
        .document()
        .doc_info
        .border_fills
        .get((id as usize).saturating_sub(1))?;
    bf.fill.solid.as_ref().map(|s| s.background_color)
}

#[test]
fn issue_4275_cellzone_fill_exported_in_html() {
    let mut src = DocumentCore::new_empty();
    src.set_document(source_document());
    let html = src
        .export_control_html_native(0, 0, &[], 0)
        .expect("HTML 내보내기");
    assert!(
        html.to_lowercase().contains("background-color:#c0c0c0"),
        "cellzone 회색 배경이 HTML 에 있어야 한다:\n{html}"
    );
    assert!(
        html.contains("text-align:center"),
        "셀 문단 가운데 정렬이 HTML 에 있어야 한다:\n{html}"
    );
}

#[test]
fn issue_4275_cross_document_html_paste_keeps_cell_style() {
    let mut src = DocumentCore::new_empty();
    src.set_document(source_document());
    let html = src
        .export_control_html_native(0, 0, &[], 0)
        .expect("HTML 내보내기");

    let mut dst = DocumentCore::new_empty();
    dst.create_blank_document_native().expect("빈 문서");
    dst.paste_html_native(0, 0, 0, &html)
        .expect("교차 문서 HTML 붙여넣기");

    let table = pasted_table(&dst);
    let fill = cell_fill_color(&dst, table);
    assert_eq!(
        fill,
        Some(GREY),
        "붙여넣은 헤더 셀 배경이 회색이어야 한다: {fill:?}"
    );

    let ps_id = table.cells[0].paragraphs[0].para_shape_id;
    let align = dst
        .document()
        .doc_info
        .para_shapes
        .get(ps_id as usize)
        .map(|ps| ps.alignment);
    assert_eq!(
        align,
        Some(Alignment::Center),
        "붙여넣은 셀 문단 정렬이 가운데여야 한다 (para_shape_id={ps_id})"
    );
}
