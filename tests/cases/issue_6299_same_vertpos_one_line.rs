//! [#6299] 같은 `vertpos` 의 LINE_SEG 조각(어울림 개체 좌·우)을 각각 별개 줄로
//! 세면 표 칸 높이가 2배가 되고, `vertAlign=CENTER` 칸 내용이 내려와 겹친다.
//!
//! 저장 사다리는 한 글줄을 그림 좌·우로 쪼개도 **같은 vertpos** 를 적는다.
//! 높이 회계는 그 짝을 한 줄로 센다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table, VerticalAlign};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 시각 줄 높이(HWPUNIT) — 96dpi 에서 16.0px.
const LINE_H: i32 = 1200;
/// 선언 칸 높이 — 내용보다 작게 두어 측정이 행을 키우게 한다.
const DECLARED_H: u32 = 800;
const CELL_W: u32 = 20000;

fn wrap_frag(text_start: u32, vertical_pos: i32, left: bool) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos,
        line_height: LINE_H,
        text_height: LINE_H,
        baseline_distance: LINE_H * 85 / 100,
        line_spacing: 0,
        column_start: if left { 0 } else { 9000 },
        segment_width: 8000,
        tag: if left {
            LineSeg::TAG_FIRST_SEGMENT
        } else {
            LineSeg::TAG_LAST_SEGMENT
        },
        ..Default::default()
    }
}

fn paragraph_with_segs(text: &str, segs: Vec<LineSeg>) -> Paragraph {
    let n = text.chars().count() as u32;
    Paragraph {
        text: text.to_string(),
        char_count: n,
        char_offsets: (0..=n).collect(),
        line_segs: segs,
        ..Default::default()
    }
}

fn table_with_cell_para(para: Paragraph) -> Table {
    let mut cell = Cell {
        col: 0,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: CELL_W,
        height: DECLARED_H,
        paragraphs: vec![para],
        apply_inner_margin: true,
        vertical_align: VerticalAlign::Center,
        ..Default::default()
    };
    cell.padding = Default::default();

    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell],
        dirty: true,
        ..Default::default()
    };
    table.common.width = CELL_W;
    table.common.height = DECLARED_H;
    table.padding = Default::default();
    table.rebuild_grid();
    table
}

fn document_with_table(table: Table) -> Document {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    let mut anchor = Paragraph::default();
    anchor.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59529,
        height: 84189,
        margin_left: 8504,
        margin_right: 8504,
        margin_top: 5668,
        margin_bottom: 4252,
        margin_header: 4252,
        margin_footer: 4252,
        ..Default::default()
    };
    section.paragraphs.push(anchor);
    doc.sections.push(section);
    doc
}

fn cell_height(doc: Document) -> f64 {
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let mut heights = Vec::new();
    collect_table_heights(&tree.root, &mut heights);
    *heights.first().expect("표가 있어야 한다")
}

fn collect_table_heights(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node.bbox.height);
        return;
    }
    for child in &node.children {
        collect_table_heights(child, out);
    }
}

/// 두 시각 줄이 좌·우 조각 4개로 쪼개져도 칸 높이는 두 줄분이다.
#[test]
fn issue_6299_same_vertpos_fragments_count_as_one_line() {
    let para = paragraph_with_segs(
        "가나다라",
        vec![
            wrap_frag(0, 0, true),
            wrap_frag(1, 0, false),
            wrap_frag(2, LINE_H, true),
            wrap_frag(3, LINE_H, false),
        ],
    );
    let h = cell_height(document_with_table(table_with_cell_para(para)));

    // 2줄 × 16px = 32px. 수정 전엔 4조각을 합산해 ~64px.
    assert!(
        h < 48.0,
        "같은 vertpos 조각을 별개 줄로 세면 칸이 2배가 된다: {h:.1}px"
    );
    assert!(h > 20.0, "시각 두 줄 높이는 유지돼야 한다: {h:.1}px");
}

/// 서로 다른 vertpos 의 두 줄은 종전대로 둘 다 센다.
#[test]
fn issue_6299_distinct_vertpos_lines_still_sum() {
    let para = paragraph_with_segs(
        "가나",
        vec![wrap_frag(0, 0, true), wrap_frag(1, LINE_H, true)],
    );
    let h = cell_height(document_with_table(table_with_cell_para(para)));
    assert!(
        h > 20.0,
        "서로 다른 vertpos 두 줄은 합산돼야 한다: {h:.1}px"
    );
}
