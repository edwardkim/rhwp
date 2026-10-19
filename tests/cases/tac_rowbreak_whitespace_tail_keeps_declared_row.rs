//! 글자처럼 취급한 RowBreak 표에서 칸의 마지막 공백 문단 줄간격이 행을 키운다.
//!
//! 강원 입사지원서 주소 칸은 `" (우편번호 : - )"` 다음에 공백 한 칸 문단을 둔다.
//! 저장 줄 끝(1600 + 1000HU)에 안 여백 282HU 를 더하면 선언 2915HU 안에 든다.
//! 측정은 그 공백 문단의 줄간격(600HU)을 더해 행을 3482HU 로 키웠다. 줄간격만으로
//! 선언을 넘는 행을 선언으로 되돌리는 규칙(#1763)이 RowBreak 표 전체를 빼고 있었다.
//!
//! 불변식: 마지막 줄간격을 뺀 내용 + 안 여백이 선언 안에 들면 행은 선언 높이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::TablePageBreak;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

const LINE: i32 = 1000;
const SPACING: i32 = 600;
const SLACK: i32 = 33;

/// 1×1 TAC RowBreak 표, 칸 문단 `["가", " "]`. 선언 높이(HU)를 돌려준다.
fn document() -> (DocumentCore, u32) {
    let mut core = DocumentCore::new_empty();
    {
        let mut section = rhwp::model::document::Section::default();
        section.section_def.page_def = rhwp::model::page::PageDef::a4_default();
        section
            .paragraphs
            .push(rhwp::model::paragraph::Paragraph::new_empty());
        let mut document = rhwp::model::document::Document::default();
        document.sections.push(section);
        core.set_document(document);
    }
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    core.insert_text_in_cell_native(0, 0, 0, 0, 0, 0, "가")
        .expect("칸 글 삽입");
    core.split_paragraph_in_cell_native(0, 0, 0, 0, 0, 1, None)
        .expect("칸 문단 나누기");
    core.insert_text_in_cell_native(0, 0, 0, 0, 1, 0, " ")
        .expect("공백 문단");
    let mut document = core.document().clone();
    let Some(Control::Table(table)) = document.sections[0].paragraphs[0].controls.first_mut()
    else {
        panic!("표를 찾지 못했다");
    };
    table.common.treat_as_char = true;
    // 저장할 때 CommonObjAttr 를 IR 에서 다시 만든다.
    table.raw_ctrl_data.clear();
    table.page_break = TablePageBreak::RowBreak;
    let padding = table.cells[0].effective_vertical_padding_hu(&table.padding);
    let cell = &mut table.cells[0];
    assert_eq!(cell.paragraphs.len(), 2, "칸 문단 두 개");
    for (index, para) in cell.paragraphs.iter_mut().enumerate() {
        assert_eq!(para.line_segs.len(), 1, "문단마다 한 줄");
        let seg = &mut para.line_segs[0];
        seg.vertical_pos = index as i32 * (LINE + SPACING);
        seg.line_height = LINE;
        seg.text_height = LINE;
        seg.baseline_distance = LINE * 85 / 100;
        seg.line_spacing = SPACING;
    }
    // 선언은 마지막 줄 끝 + 안 여백보다 조금 크다 (원본 2915 = 2600 + 282 + 33).
    // 마지막 줄간격까지 더하면 선언을 넘는다.
    let declared = (LINE + SPACING + LINE + padding + SLACK) as u32;
    assert!(declared < (2 * (LINE + SPACING) + padding) as u32);
    cell.height = declared;
    // 저장 조판 정보를 쓰는 원본 경로로 읽도록 HWP 로 저장했다가 다시 연다.
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(&document).expect("HWP 저장");
    let core = DocumentCore::from_bytes(&bytes).expect("HWP 다시 열기");
    (core, declared)
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

#[test]
fn tac_rowbreak_whitespace_tail_keeps_declared_row() {
    let (core, declared) = document();
    let tree = core.build_page_render_tree(0).expect("렌더 트리");
    let cell = find(
        &tree.root,
        &|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0),
    )
    .expect("칸");
    let declared_px = hwpunit_to_px(declared as i32, DEFAULT_DPI);
    assert!(
        (cell.bbox.height - declared_px).abs() <= 0.5,
        "행은 선언 높이({declared_px:.2}px)여야 한다 — 마지막 공백 문단 줄간격({:.2}px)이 \
         행을 키우면 안 된다, got {:.2}px",
        hwpunit_to_px(SPACING, DEFAULT_DPI),
        cell.bbox.height
    );
}
