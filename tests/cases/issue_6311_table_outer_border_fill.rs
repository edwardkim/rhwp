//! [#6311] 일러두기 틀은 3×3 표의 `borderFillIDRef` 가 바깥 네 변 SOLID 인데
//! 칸은 그 변을 NONE 으로 둔다. 표 테두리를 occupancy 로 막으면 왼쪽·아래·
//! 제목왼쪽이 레이아웃에서 사라진다.

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

fn solid_fill() -> BorderFill {
    let solid = BorderLine {
        line_type: BorderLineType::Solid,
        width: 1,
        color: 0,
    };
    BorderFill {
        borders: [solid, solid, solid, solid],
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

/// 일러두기 축소판: 제목 칸만 네 변 SOLID, 나머지 칸은 테두리 없음.
/// 표 자신은 네 변 SOLID (`borderFillIDRef=1`).
fn callout_document() -> Document {
    const COL0: u32 = 4000;
    const COL1: u32 = 4000;
    const ROW0: u32 = 1500;
    const ROW1: u32 = 8000;

    let mut table = Table {
        row_count: 2,
        col_count: 2,
        border_fill_id: 1,
        cells: vec![
            cell(0, 0, 1, 1, COL0, ROW0, 0, ""),
            cell(1, 0, 1, 1, COL1, ROW0, 1, "일러두기"),
            cell(0, 1, 2, 1, COL0 + COL1, ROW1, 0, "본문"),
        ],
        ..Default::default()
    };
    table.common.width = COL0 + COL1;
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
    doc.doc_info.border_fills = vec![solid_fill()];
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
fn issue_6311_table_border_fill_emits_left_bottom_and_title_left() {
    let bytes = serialize_hwpx(&callout_document()).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("reload");
    let page = core.build_page_render_tree(0).expect("render tree");
    let table = find_table(&page.root).expect("callout table");
    let mut lines = Vec::new();
    collect_lines(table, &mut lines);

    let left = table.bbox.x;
    let top = table.bbox.y;
    let right = table.bbox.x + table.bbox.width;
    let bottom = table.bbox.y + table.bbox.height;
    let mid_x = (left + right) / 2.0;

    let has_left = lines.iter().any(|&(x1, y1, x2, y2)| {
        (x1 - left).abs() <= 2.0
            && (x2 - left).abs() <= 2.0
            && (y1.min(y2) - top).abs() <= 4.0
            && y1.max(y2) >= bottom - 4.0
    });
    let has_bottom = lines.iter().any(|&(x1, y1, x2, y2)| {
        (y1 - bottom).abs() <= 2.0
            && (y2 - bottom).abs() <= 2.0
            && (x1.min(x2) - left).abs() <= 4.0
            && x1.max(x2) >= right - 4.0
    });
    let has_title_left = lines.iter().any(|&(x1, y1, x2, y2)| {
        (y1 - top).abs() <= 2.0
            && (y2 - top).abs() <= 2.0
            && (x1.min(x2) - left).abs() <= 4.0
            && x1.max(x2) <= mid_x + 2.0
            && (x1 - x2).abs() >= 8.0
    });

    assert!(
        has_left,
        "표 왼쪽 세로선이 있어야 한다 (got {lines:?}, bbox={:?})",
        table.bbox
    );
    assert!(
        has_bottom,
        "표 아래 가로선이 있어야 한다 (got {lines:?}, bbox={:?})",
        table.bbox
    );
    assert!(
        has_title_left,
        "제목 왼쪽 가로선이 있어야 한다 (got {lines:?}, bbox={:?})",
        table.bbox
    );
}
