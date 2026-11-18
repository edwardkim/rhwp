//! 검사 유형: 고정형 — 양쪽 98% 이상인 화학 27쪽의 독립 Print 본문 x를 ±2px로 보호한다.
//! 주석 갱신: 2026-11-18 07:01 KST.
//! 보정 전 Native 99.25221%, fresh WASM 99.37167%(96 DPI·PDFium·2px raw).
//! 독립 한컴 2020 Hancom PDF Print의 두 U+2027 뒤 '색깔' 원점은 x=346.24px다.
//! 원본 기호 폭을 잘못 누적한 뒤 본문을 이동시키는 보정으로 대신하지 않는다.
//! 개인 글꼴을 열지 않고 기존 공개 원본의 내용 소속·쪽수를 함께 검사한다.

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
fn body_after_two_missing_batang_points_keeps_the_print_origin() {
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"),
    )
    .unwrap();
    let mut core = DocumentCore::from_bytes(&input).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 103);
    let tree = core.build_page_render_tree(26).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let matches: Vec<_> = nodes.iter().copied().filter(|n| {
        matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text.starts_with("색깔 등 세부적 표시방법 제시"))
    }).collect();
    assert_eq!(matches.len(), 1, "원래 본문을 한 번만 보존한다");
    assert!(
        (matches[0].bbox.x - 346.24).abs() <= 2.0,
        "독립 Print의 뒤 본문 원점: {}",
        matches[0].bbox.x
    );
}
