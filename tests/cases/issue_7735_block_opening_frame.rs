//! 검사 유형: 관계형 — 원본 첫 표 상자와 실제 병합 칸의 닫는 경계를 대조한다.
//! 주석 갱신: 2026-11-17 23:43 KST.
//! 공식 형식 문서 31·32쪽 Native raw 100%, fresh WASM 99.99919%에서 추가했다.
//! 기준은 기존 공식 PDF이며 다시 출력하거나 수정하지 않았다.
//! 내용 컷 뒤 인증한 Common 상자를 paint 행 목록에서도 소비하고, 다음 쪽의
//! 동일 셀 내용을 누락·중복하지 않는지 검사한다. 고정 쪽 좌표를 기대값으로 쓰지 않는다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn source_table(root: &RenderNode) -> &RenderNode {
    if matches!(&root.node_type, RenderNodeType::Table(meta)
        if meta.section_index == Some(3) && meta.para_index == Some(193)
            && meta.control_index == Some(0))
    {
        return root;
    }
    root.children
        .iter()
        .find_map(find_table)
        .expect("원본 표 조각이 있어야 한다")
}

fn find_table(root: &RenderNode) -> Option<&RenderNode> {
    if matches!(&root.node_type, RenderNodeType::Table(meta)
        if meta.section_index == Some(3) && meta.para_index == Some(193)
            && meta.control_index == Some(0))
    {
        Some(root)
    } else {
        root.children.iter().find_map(find_table)
    }
}

fn cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(meta)
            if meta.row == row && meta.col == col)
        })
        .expect("원본 셀의 실제 조각")
}

fn text(root: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        // 표시용 옛한글 정규화와 원문의 PUA를 혼동하지 않는다.
        out.push_str(&run.text);
    }
    for child in &root.children {
        text(child, out);
    }
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_control())
        .collect()
}

#[test]
fn accepted_opening_box_reaches_the_painted_cells_without_losing_continuation_text() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/한글문서파일형식_5.0_revision1.3.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 71, "공식 PDF 전체 쪽수 보존");
    let Control::Table(original) = &core.document().sections[3].paragraphs[193].controls[0] else {
        panic!("번호 형식 원본 표");
    };
    let opening = f64::from(original.common.height) * 96.0 / 7200.0;
    let original_text: String = original
        .cells
        .iter()
        .find(|cell| cell.row == 7 && cell.col == 2)
        .unwrap()
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect();
    let first_tree = core.build_page_render_tree(30).unwrap();
    let first = source_table(&first_tree.root);
    for col in 0..3 {
        let closing = cell(first, 7, col);
        let actual = closing.bbox.y + closing.bbox.height - first.bbox.y;
        assert!(
            (actual - opening).abs() < 1e-6,
            "{col}열: 원본 상자 {opening}와 그린 첫 조각 {actual}의 물리 끝이 다르다"
        );
    }
    let second_tree = core.build_page_render_tree(31).unwrap();
    let mut painted = String::new();
    text(cell(first, 7, 2), &mut painted);
    text(cell(source_table(&second_tree.root), 7, 2), &mut painted);
    assert_eq!(
        compact(&painted),
        compact(&original_text),
        "분할 셀의 원문 단일 소유"
    );
}
