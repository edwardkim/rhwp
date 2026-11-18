//! 검사 유형: 고정형 — 양쪽 98% 이상인 화학 56쪽의 뒤 본문 원점을 ±2px로 보호한다.
//! 주석 갱신: 2026-11-18 11:36 KST.
//! 보정 전 Native 99.66074%, fresh WASM 99.70125%(96 DPI·PDFium·2px raw).
//! 독립 한컴 2020 Hancom PDF Print의 '방법 마련' 시작은 x=281.76px다.
//! 실제 글줄의 내용 소속·쪽수도 함께 검사하며 개인 글꼴 파일을 열지 않는다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, out);
    }
}

#[test]
fn body_after_missing_gothic_extra_point_keeps_the_print_origin() {
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"),
    )
    .unwrap();
    let mut core = DocumentCore::from_bytes(&input).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 103);
    let tree = core.build_page_render_tree(55).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let matches: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text == "방법 마련"))
        .collect();
    assert_eq!(matches.len(), 1, "제목의 뒤 본문을 한 번만 보존한다");
    assert!(
        (matches[0].bbox.x - 281.76).abs() <= 2.0,
        "독립 Print의 뒤 본문 원점: {}",
        matches[0].bbox.x
    );
}
