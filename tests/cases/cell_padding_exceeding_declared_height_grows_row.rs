//! 칸 안 여백이 칸 선언 높이보다 커도 행은 내용 + 안 여백만큼 자란다.
//!
//! 한/글에서 칸 선언 높이(cellSz)는 최소 높이다. 건축허가 신청서 서식의 빈 간격 행
//! (선언 282HU, 안 여백 위 283·아래 142HU, 줄 200HU)을 한/글 PDF 는 625HU 로 그린다.
//! 종전에는 안 여백 합이 선언 높이를 넘으면 비정상 저장값으로 보고 행을 선언 높이에
//! 묶은 뒤 안 여백을 절반으로 줄였다. 그래서 그런 행마다 표가 위로 당겨졌다.
//!
//! 불변식: 행 높이는 위 안 여백 + 줄 높이 + 아래 안 여백 이상이고, 글줄은 위 안
//! 여백 아래에서 시작한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

const PAD_TOP: i16 = 1000;
const PAD_BOTTOM: i16 = 500;
const DECLARED: u32 = 1200;

/// 2×1 표의 첫 칸을 "안 여백 합 > 선언 높이 ≥ 줄 높이" 로 만든다. 줄 높이를 돌려준다.
fn document() -> (DocumentCore, i32) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    {
        let mut section = rhwp::model::document::Section::default();
        section.section_def.page_def = rhwp::model::page::PageDef::a4_default();
        section
            .paragraphs
            .push(rhwp::model::paragraph::Paragraph::new_empty());
        let mut document = core.document().clone();
        document.sections.clear();
        document.sections.push(section);
        core.set_document(document);
    }
    core.create_table_native(0, 0, 0, 2, 1).expect("2×1 표");
    core.insert_text_in_cell_native(0, 0, 0, 0, 0, 0, "가")
        .expect("칸 글 삽입");
    let mut document = core.document().clone();
    let Some(Control::Table(table)) = document.sections[0].paragraphs[0].controls.first_mut()
    else {
        panic!("표를 찾지 못했다");
    };
    let cell = table
        .cells
        .iter_mut()
        .find(|cell| cell.row == 0)
        .expect("첫 행 칸");
    cell.apply_inner_margin = true;
    cell.padding.top = PAD_TOP;
    cell.padding.bottom = PAD_BOTTOM;
    cell.height = DECLARED;
    let line_height = cell.paragraphs[0].line_segs[0].line_height;
    assert!(
        line_height <= DECLARED as i32 && i32::from(PAD_TOP + PAD_BOTTOM) > DECLARED as i32,
        "전제: 줄은 선언 높이 안에 들고 안 여백 합만 선언 높이를 넘는다 (줄 {line_height})"
    );
    core.set_document(document);
    (core, line_height)
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

#[test]
fn cell_padding_exceeding_declared_height_grows_row() {
    let (core, line_height) = document();
    let tree = core.build_page_render_tree(0).expect("렌더 트리");
    let cell = find(
        &tree.root,
        &|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0),
    )
    .expect("첫 행 칸");
    let line = find(cell, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
    .expect("칸 글줄");

    let px = |hu: i32| hwpunit_to_px(hu, DEFAULT_DPI);
    let minimum = px(i32::from(PAD_TOP) + line_height + i32::from(PAD_BOTTOM));
    assert!(
        cell.bbox.height + 0.5 >= minimum,
        "행은 안 여백 + 줄 높이({minimum:.2}px) 이상이어야 한다, got {:.2}px",
        cell.bbox.height
    );
    assert!(
        line.bbox.y + 0.5 >= cell.bbox.y + px(i32::from(PAD_TOP)),
        "글줄은 위 안 여백 아래에서 시작해야 한다: 칸 y={:.2}, 글줄 y={:.2}",
        cell.bbox.y,
        line.bbox.y
    );
}
