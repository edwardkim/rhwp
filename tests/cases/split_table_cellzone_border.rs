//! 쪽을 넘는 표 조각도 `hp:cellzone` 테두리를 그려야 한다.
//!
//! 비분할 표 경로만 cellzone 덮어쓰기(#6619)·표 외곽 보충(#6311)을 적용하고, 표 조각
//! 경로(`table_partial`)는 둘 다 건너뛰었다. 그래서 쪽을 넘는 표는 머리행을 감싼
//! zone 의 굵은 테두리가 통째로 빠졌다(문화체육관광부 세계 책의 날 행사표, 한컴 PDF).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{BorderFill, BorderLine, BorderLineType, CharShape, ParaShape};
use rhwp::model::table::{Cell, Table, TablePageBreak, TableZone};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::hwpx::serialize_hwpx;

const COL: u32 = 8000;
const ROW: u32 = 3000;
const ROWS: u16 = 40;
const ZONE_COLOR: u32 = 0x0000_00FF;

fn fill(line_type: BorderLineType, color: u32) -> BorderFill {
    let line = BorderLine {
        line_type,
        width: 3,
        color,
    };
    BorderFill {
        borders: [line, line, line, line],
        ..Default::default()
    }
}

fn cell(col: u16, row: u16) -> Cell {
    Cell {
        col,
        row,
        col_span: 1,
        row_span: 1,
        width: COL,
        height: ROW,
        border_fill_id: 1,
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    }
}

/// 2열 × 40행 표(한 쪽에 안 들어간다). 칸·표는 테두리 없음(1), 첫 행만 zone(2)이
/// 네 변 SOLID 로 감싼다.
fn split_zoned_document() -> Document {
    let mut table = Table {
        row_count: ROWS,
        col_count: 2,
        border_fill_id: 1,
        page_break: TablePageBreak::RowBreak,
        cells: (0..ROWS)
            .flat_map(|row| [cell(0, row), cell(1, row)])
            .collect(),
        zones: vec![TableZone {
            start_col: 0,
            start_row: 0,
            end_col: 1,
            end_row: 0,
            border_fill_id: 2,
        }],
        ..Default::default()
    };
    table.common.width = COL * 2;
    table.common.height = ROW * ROWS as u32;
    table.rebuild_grid();

    let mut owner = Paragraph::default();
    owner.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59528,
        height: 84188,
        ..Default::default()
    };
    section.paragraphs.push(owner);

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.doc_info.char_shapes = vec![CharShape::default()];
    doc.doc_info.border_fills = vec![
        fill(BorderLineType::None, 0),
        fill(BorderLineType::Solid, ZONE_COLOR),
    ];
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    doc
}

fn collect_lines(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64, u32)>) {
    if let RenderNodeType::Line(l) = &node.node_type {
        out.push((l.x1, l.y1, l.x2, l.y2, l.style.color));
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

fn find_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node);
    }
    node.children.iter().find_map(find_table)
}

#[test]
fn split_table_fragment_draws_cellzone_border() {
    let bytes = serialize_hwpx(&split_zoned_document()).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("reload");
    assert!(core.page_count() >= 2, "표가 쪽을 넘어야 한다");

    let page = core.build_page_render_tree(0).expect("render tree");
    let table = find_table(&page.root).expect("first fragment");
    let mut lines = Vec::new();
    collect_lines(table, &mut lines);
    let zone: Vec<_> = lines.iter().filter(|l| l.4 == ZONE_COLOR).collect();

    let (left, right) = (table.bbox.x, table.bbox.x + table.bbox.width);
    let horizontal_full = |y_pred: &dyn Fn(f64) -> bool| {
        zone.iter().any(|&&(x1, y1, x2, y2, _)| {
            (y1 - y2).abs() <= 0.5
                && y_pred(y1)
                && x1.min(x2) <= left + 2.0
                && x1.max(x2) >= right - 2.0
        })
    };
    let top = table.bbox.y;
    let row_h = ROW as f64 * 96.0 / 7200.0;
    assert!(
        horizontal_full(&|y| (y - top).abs() <= 2.0),
        "zone 윗변이 표 조각에 있어야 한다 (zone lines={zone:?})"
    );
    assert!(
        horizontal_full(&|y| (y - (top + row_h)).abs() <= 4.0),
        "zone 아랫변(첫 행 아래)이 표 조각에 있어야 한다 (zone lines={zone:?})"
    );
}
