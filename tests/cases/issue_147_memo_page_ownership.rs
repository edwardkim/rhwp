//! #147 원본 aift.hwp의4쪽 목차와 주석의 소유 계약.
//!
//! 독립 이슈의 결함은「기업 소개 및 본 과제 관련 기술력 소개」주석이
//! 바탕쪽으로 오분류되어 목차에 보이는 것이다. 탭 리더의 절대 끝 좌표나
//! SVG 전체 hash는 그 계약의 기대값이 아니다. 같은 원본의 한컴2022
//! Print PDF4쪽과 Native/fresh WASM 선행 시각 검증을 review에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

#[test]
fn issue_147_aift_page3() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/aift.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(file).expect("original aift fixture"))
        .expect("parse original aift fixture");
    let tree = core.build_page_render_tree(3).expect("original TOC page4");
    fn visible_text(node: &RenderNode, output: &mut String) {
        if !node.visible || node.editor_only {
            return;
        }
        if let RenderNodeType::TextRun(run) = &node.node_type {
            output.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
        }
        for child in &node.children {
            visible_text(child, output);
        }
    }
    let mut text = String::new();
    visible_text(&tree.root, &mut text);
    assert!(
        !text.contains("기업소개및본과제관련기술력소개"),
        "the annotation must not become visible master-page content on the TOC"
    );
    let first = text.find("1.과제의필요성").expect("first TOC heading");
    let last = text
        .find("8.사업비사용에관한계획")
        .expect("last TOC heading");
    assert_eq!(text.matches("목차").count(), 1, "one TOC title");
    assert_eq!(text.matches("1.과제의필요성").count(), 1);
    assert_eq!(text.matches("8.사업비사용에관한계획").count(), 1);
    assert!(first < last, "the original TOC heading order must survive");
}
