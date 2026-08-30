//! [#6104] 문단 오프셋을 가진 자리차지 표 위로 뒤 문단의 TAC 제목 상자가 올라타지 않는다.
//!
//! 자리차지(TopAndBottom, vert=Para) 표는 앵커 y+offset 에 그려지고 exclusion 밴드를
//! 남긴다. 뒤 문단의 글자처럼 취급(TAC) 표는 PageItem::Table 이라 문단 경로의
//! 밴드 소비를 건너뛰어, 표 데이터 행을 덮었다(36483048 4쪽).
//!
//! 이미 그린 선행 밴드만 피하므로 앵커 예약 이중 계상(#4090)은 없다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const LINE_H: i32 = 800;
const OFFSET: u32 = 1500;
const TB_H: u32 = 2000;
const TAC_H: u32 = 1200;
const CELL_W: u32 = 12000;

fn cell(text: &str, height: u32) -> Cell {
    let n = text.chars().count() as u32;
    Cell {
        col: 0,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: CELL_W,
        height,
        paragraphs: vec![Paragraph {
            text: text.to_string(),
            char_count: n,
            char_offsets: (0..=n).collect(),
            ..Default::default()
        }],
        apply_inner_margin: true,
        ..Default::default()
    }
}

fn table_block(text: &str, height: u32, treat_as_char: bool, offset: u32) -> Table {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell(text, height)],
        dirty: true,
        ..Default::default()
    };
    table.common.width = CELL_W;
    table.common.height = height;
    table.common.treat_as_char = treat_as_char;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.common.vertical_offset = offset;
    table.rebuild_grid();
    table
}

fn document() -> Document {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    let mut host = Paragraph {
        text: "추진기간".to_string(),
        char_count: 4,
        char_offsets: (0..=4).collect(),
        line_segs: vec![LineSeg {
            text_start: 0,
            vertical_pos: 0,
            line_height: LINE_H,
            text_height: LINE_H,
            baseline_distance: LINE_H * 85 / 100,
            segment_width: 40000,
            ..Default::default()
        }],
        ..Default::default()
    };
    host.controls.push(Control::Table(Box::new(table_block(
        "표1", TB_H, false, OFFSET,
    ))));

    let mut title = Paragraph::default();
    title.controls.push(Control::Table(Box::new(table_block(
        "제목", TAC_H, true, 0,
    ))));

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
    section.paragraphs.push(host);
    section.paragraphs.push(title);
    doc.sections.push(section);
    doc
}

fn collect_tables(node: &RenderNode, out: &mut Vec<(usize, f64, f64)>) {
    if let RenderNodeType::Table(tbl) = &node.node_type {
        if let Some(pi) = tbl.para_index {
            out.push((pi, node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        collect_tables(child, out);
    }
}

#[test]
fn issue_6104_tac_title_clears_offset_topbottom_table() {
    let mut core = DocumentCore::new_empty();
    core.set_document(document());
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let mut tables = Vec::new();
    collect_tables(&tree.root, &mut tables);
    let tb = tables
        .iter()
        .find(|(pi, _, _)| *pi == 0)
        .expect("자리차지 표가 있어야 한다");
    let tac = tables
        .iter()
        .find(|(pi, _, _)| *pi == 1)
        .expect("TAC 제목 상자가 있어야 한다");
    let tb_bottom = tb.1 + tb.2;
    assert!(
        tac.1 + 0.5 >= tb_bottom,
        "TAC 제목 상자 상단({:.1})이 자리차지 표 하단({tb_bottom:.1}) 아래여야 한다 \
         (결함 시 표 데이터 행을 덮음)",
        tac.1
    );
}
