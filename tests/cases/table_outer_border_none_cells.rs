//! 표 자신의 `borderFillIDRef` 가 네 변 SOLID 여도, 바깥 칸이 그 변을 NONE 으로
//! 선언하면 한컴은 선을 그리지 않는다(사행행위 시설완료 신고서 처리절차 표,
//! 한컴 2020 PDF). 제목 칸만 위 선을 그린 모양을 «부분 프레임»으로 보고 표 테두리로
//! 메우던 휴리스틱이 흐름도 둘레에 없는 틀을 그렸다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{BorderFill, BorderLine, BorderLineType, CharShape, ParaShape};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::hwpx::serialize_hwpx;

fn fill(line_type: BorderLineType) -> BorderFill {
    let line = BorderLine {
        line_type,
        width: 1,
        color: 0,
    };
    BorderFill {
        borders: [line, line, line, line],
        ..Default::default()
    }
}

fn cell(
    col: u16,
    row: u16,
    col_span: u16,
    row_span: u16,
    w: u32,
    h: u32,
    bf: u16,
    text: &str,
) -> Cell {
    Cell {
        col,
        row,
        col_span,
        row_span,
        width: w,
        height: h,
        border_fill_id: bf,
        paragraphs: vec![Paragraph {
            text: text.to_string(),
            char_count: text.chars().count() as u32,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 처리절차 표 축소판: 제목 칸만 네 변 SOLID(bf=1), 나머지 칸은 네 변 NONE 을
/// 선언한다(bf=2). 표 자신은 네 변 SOLID (`borderFillIDRef=1`).
fn none_cells_document() -> Document {
    const COL: u32 = 4000;
    const ROW0: u32 = 1500;
    const ROW1: u32 = 3000;

    let mut table = Table {
        row_count: 2,
        col_count: 2,
        border_fill_id: 1,
        cells: vec![
            cell(0, 0, 1, 1, COL, ROW0, 2, ""),
            cell(1, 0, 1, 1, COL, ROW0, 1, "신고서 작성"),
            cell(0, 1, 2, 1, COL * 2, ROW1, 2, "(신청인)"),
        ],
        ..Default::default()
    };
    table.common.width = COL * 2;
    table.common.height = ROW0 + ROW1;
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
    doc.doc_info.border_fills = vec![fill(BorderLineType::Solid), fill(BorderLineType::None)];
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    doc
}

fn collect_lines(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if let RenderNodeType::Line(line) = &node.node_type {
        out.push((line.x1, line.y1, line.x2, line.y2));
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
fn table_border_does_not_cover_cell_edges_declared_none() {
    let bytes = serialize_hwpx(&none_cells_document()).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("reload");
    let page = core.build_page_render_tree(0).expect("render tree");
    let table = find_table(&page.root).expect("flow table");
    let mut lines = Vec::new();
    collect_lines(table, &mut lines);

    let left = table.bbox.x;
    let top = table.bbox.y;
    let right = table.bbox.x + table.bbox.width;
    let bottom = table.bbox.y + table.bbox.height;
    let mid_x = (left + right) / 2.0;

    // 칸이 NONE 으로 선언한 바깥 변(왼쪽·아래·제목 왼쪽 위)에는 선이 없어야 한다.
    let on_left = lines
        .iter()
        .any(|&(x1, _, x2, _)| (x1 - left).abs() <= 1.0 && (x2 - left).abs() <= 1.0);
    let on_bottom = lines
        .iter()
        .any(|&(_, y1, _, y2)| (y1 - bottom).abs() <= 1.0 && (y2 - bottom).abs() <= 1.0);
    let top_left = lines.iter().any(|&(x1, y1, x2, y2)| {
        (y1 - top).abs() <= 1.0 && (y2 - top).abs() <= 1.0 && x1.min(x2) < mid_x - 2.0
    });
    // 제목 칸 자신의 위 선은 그대로 남는다.
    let title_top = lines.iter().any(|&(x1, y1, x2, y2)| {
        (y1 - top).abs() <= 1.0 && (y2 - top).abs() <= 1.0 && x1.max(x2) >= right - 2.0
    });

    assert!(!on_left, "표 왼쪽 세로선이 없어야 한다 (got {lines:?})");
    assert!(!on_bottom, "표 아래 가로선이 없어야 한다 (got {lines:?})");
    assert!(!top_left, "제목 왼쪽 가로선이 없어야 한다 (got {lines:?})");
    assert!(title_top, "제목 칸 윗선은 있어야 한다 (got {lines:?})");
}
