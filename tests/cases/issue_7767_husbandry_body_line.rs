//! 관계형·고정형 회귀 — #7767의 둘째 쪽 마지막 문장 소속과 독립 Print 쪽수.
//! 주석 작성 2026-11-23 20:46 KST.
//! 2026-11-23 독립 한컴 2024 1-up Print, 96 DPI, 2px raw 실루엣.
//! Native/fresh WASM 전12쪽 최저 99.98%/99.98%, 2쪽 100.00%/100.00%.
//! 근거는 `mydocs/pr/assets/issue7767/`의 전쪽 TSV와 2쪽 비교 PNG다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::RenderNodeType;

#[test]
fn husbandry_second_page_keeps_last_husbandry_line_in_body() {
    let bytes =
        include_bytes!("../../samples/issue6778/156757920-animal-welfare-husbandry-guidelines.hwp");
    let document = DocumentCore::from_bytes(bytes).expect("사육지침 원본");
    assert_eq!(document.page_count(), 12, "독립 Print 전체 쪽수");
    let page = document.build_page_render_tree(1).expect("2쪽 본문");
    let body = page
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문 영역");
    let column = body
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .expect("본문 단");
    let terminal = column
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .filter(|node| {
            let text: String = node
                .children
                .iter()
                .filter_map(|item| match &item.node_type {
                    RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                    _ => None,
                })
                .collect();
            text.trim() == "높일 수 있는 관리 기준을 담았다."
        })
        .collect::<Vec<_>>();
    assert_eq!(terminal.len(), 1, "마지막 문장 줄을 본문에 한 번 그린다");
    let terminal = terminal[0];
    assert!(terminal.bbox.y >= column.bbox.y);
    assert!(terminal.bbox.y + terminal.bbox.height <= column.bbox.y + column.bbox.height);
}
