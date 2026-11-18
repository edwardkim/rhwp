//! 검사 유형: 고정형 — 목록 본문 시작을 독립 공식 PDF의 96 DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-17 23:53 KST.
//! 형식 문서 67쪽 Native 99.99760%, fresh WASM 99.99646%에서 추가했다.
//! 원본 공식 PDF는 변경하지 않았다. 별도 ·/-/○ × 실제 폭 속성 0/1 대조군도
//! 독립 한컴 2024 Print와 Native/fresh WASM 각각 raw 100%로 확인했다.
//! 글머리표의 명목 문자 칸을 실제 글리프 폭 또는 무조건 전각으로 바꾸는 회귀를 검출한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn body_origins(node: &RenderNode, found: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.starts_with("HISTORY_INFO_FLAG_VERSION") {
            found.push(node.bbox.x);
        }
    }
    for child in &node.children {
        body_origins(child, found);
    }
}

#[test]
fn nominal_bullet_keeps_the_independent_print_body_origin() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/한글문서파일형식_5.0_revision1.3.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(66).unwrap();
    let mut positions = Vec::new();
    body_origins(&tree.root, &mut positions);
    assert_eq!(positions.len(), 1, "목록 본문은 원본 67쪽에 한 번 표시한다");
    assert!(
        (positions[0] - 201.97054).abs() <= 2.0,
        "글머리표 뒤 본문 시작: 실제 {}px, 독립 PDF 201.97054px",
        positions[0]
    );
}
