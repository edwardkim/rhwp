//! [Issue #6697] 표 칸 안의 자리차지 중첩 표가 `vertOffset` 을 통째로 무시했다.
//!
//! 본문 경로는 문단 기준(`vertRelTo=PARA`) 자리차지 표를 앵커 y + 오프셋에 놓는데
//! (`#6104`), 셀 경로는 그 값을 한 번도 읽지 않아 표가 **호스트 줄과 같은 y** 에
//! 놓였다.
//!
//! `80550` 30쪽 실측 — 문서 지시 `vertOffset=3062HU`(=40.8px):
//!
//! ```text
//!   수정 전  13×7 표 상단  p30 959.1 · p31 33.8   (머리행이 두 쪽에 반씩 잘림)
//!   수정 후                p30 999.9 · p31 74.6   (+40.8px, 지시값 그대로)
//!   한/글                  p31 괘선 상단 84.2
//! ```
//!
//! ⚠ 흐름 계상에는 **호스트가 칸의 마지막 문단일 때만** 그 몫을 싣는다. 뒤에 형제
//! 문단이 있으면 한/글은 그 몫으로 뒷내용을 밀지 않는다 — 밀면 `#1921` 59043 p36 의
//! `□ 편익` 이 p37 로 넘어간다(한/글 2024 오라클은 `②피규제 이외 일반국민` 과 같은 쪽).
//!
//! ⚠ `vertical_offset` 은 `u32` 에 담긴 **부호 있는** 값이다(`4294944683` = −22613HU).
//! `signed_hwpunit` 으로 풀지 않으면 음수가 양수로 통과해 표가 위로 튄다.

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
const OFFSET_HU: u32 = 3062;
const NESTED_H: u32 = 4000;
const CELL_W: u32 = 30000;

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

/// 문단 기준 자리차지 중첩 표.
fn nested_float_table(offset: u32) -> Table {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell("중첩", NESTED_H)],
        ..Default::default()
    };
    table.common.width = CELL_W;
    table.common.height = NESTED_H;
    table.common.treat_as_char = false;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.common.vertical_offset = offset;
    table.common.flow_with_text = true;
    table.rebuild_grid();
    table
}

/// 바깥 1×1 표 — 그 칸의 문단이 글자와 중첩 표를 함께 갖는다.
fn document(offset: u32) -> Document {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    let mut host = Paragraph {
        text: "캡션".to_string(),
        char_count: 2,
        char_offsets: (0..=2).collect(),
        line_segs: vec![LineSeg {
            text_start: 0,
            vertical_pos: 0,
            line_height: LINE_H,
            text_height: LINE_H,
            baseline_distance: LINE_H * 85 / 100,
            segment_width: 30000,
            ..Default::default()
        }],
        ..Default::default()
    };
    host.controls
        .push(Control::Table(Box::new(nested_float_table(offset))));

    let mut outer = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
            width: CELL_W,
            height: NESTED_H + 8000,
            paragraphs: vec![host],
            apply_inner_margin: true,
            ..Default::default()
        }],
        ..Default::default()
    };
    outer.common.width = CELL_W;
    outer.common.height = NESTED_H + 8000;
    outer.common.treat_as_char = true;
    outer.rebuild_grid();

    let mut outer_para = Paragraph::default();
    outer_para
        .controls
        .push(Control::Table(Box::new(outer.clone())));

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
    section.paragraphs.push(outer_para);
    doc.sections.push(section);
    doc
}

fn collect_tables(node: &RenderNode, out: &mut Vec<(bool, f64)>) {
    if let RenderNodeType::Table(tbl) = &node.node_type {
        out.push((tbl.cell_context.is_some(), node.bbox.y));
    }
    for child in &node.children {
        collect_tables(child, out);
    }
}

fn nested_top(offset: u32) -> f64 {
    let mut core = DocumentCore::new_empty();
    core.set_document(document(offset));
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let mut tables = Vec::new();
    collect_tables(&tree.root, &mut tables);
    // 중첩 표만 `cell_context` 를 갖는다(#4334).
    let nested: Vec<f64> = tables
        .iter()
        .filter(|(nested, _)| *nested)
        .map(|(_, y)| *y)
        .collect();
    assert_eq!(
        nested.len(),
        1,
        "칸 안 중첩 표가 정확히 하나여야 한다: {tables:?}"
    );
    nested[0]
}

/// 오프셋을 준 중첩 표는 오프셋이 0 일 때보다 정확히 그만큼 아래에 놓여야 한다.
#[test]
fn cell_nested_float_table_honors_vert_offset() {
    let zero = nested_top(0);
    let offset = nested_top(OFFSET_HU);
    let expected = f64::from(OFFSET_HU) / 75.0; // HWPUNIT → px @96dpi
    let moved = offset - zero;
    assert!(
        (moved - expected).abs() <= 1.0,
        "칸 안 자리차지 중첩 표가 vertOffset 만큼 내려가야 한다 — #6697 회귀 \
         (기대 +{expected:.1}px, 실제 +{moved:.1}px; 오프셋 0 일 때 {zero:.1}, \
          오프셋 {OFFSET_HU}HU 일 때 {offset:.1})"
    );
}

/// ⚠ `u32` 에 담긴 음수 오프셋을 양수로 읽으면 표가 위로 튄다(3184241 −301.5px).
#[test]
fn negative_vert_offset_does_not_lift_the_nested_table() {
    let zero = nested_top(0);
    let negative = nested_top((-22613i32) as u32);
    assert!(
        (negative - zero).abs() <= 1.0,
        "음수 vertOffset 은 적용하지 않는다 — u32 를 부호 없이 읽으면 표가 위로 튄다 \
         (오프셋 0 {zero:.1} vs 음수 {negative:.1})"
    );
}
