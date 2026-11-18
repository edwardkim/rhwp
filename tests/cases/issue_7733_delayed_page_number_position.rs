//! 검사 유형: 고정형 — 독립 Print의 쪽번호 위치를 96 DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-17 23:01 KST.
//! 화학물질 축소본 80쪽 Native/fresh WASM 각 100.00000%에서 추가했다.
//! 근거: mydocs/pr/assets/issues7733_7738_20261117/page-number-inheritance/cause.md.
//! PDF는 같은 원본의 chemical-labeling-standards-2020.pdf이며 변경하지 않았다.
//! Section4의 첫 자체 쪽번호 지정 전까지 앞 구역의 아라비아 숫자 설정을 상속한다.
//! 좌표는 PDF 80쪽 `- 58 -`의 중심 x=396.544, 논리 상단 y=1046.720px다.
//! 개인 글꼴 설치와 무관하게 제품의 실제 Footer 배치 결과를 검사한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn page_number<'a>(node: &'a RenderNode, matches: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text().replace(' ', "") == "-58-" {
            matches.push(node);
        }
    }
    for child in &node.children {
        page_number(child, matches);
    }
}

#[test]
fn first_pages_inherit_number_position_before_the_sections_own_declaration() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(79).unwrap();
    let mut matches = Vec::new();
    page_number(&tree.root, &mut matches);
    assert_eq!(matches.len(), 1, "원본 80쪽의 쪽번호 누락·중복");
    let bounds = matches[0].bbox;
    assert!((bounds.x + bounds.width / 2.0 - 396.544).abs() <= 2.0);
    assert!((bounds.y - 1046.720).abs() <= 2.0);
}
