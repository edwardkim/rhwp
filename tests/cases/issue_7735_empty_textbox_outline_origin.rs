//! 검사 유형: 고정형 — 공식 PDF의 제목·부제목 가로 시작을 96 DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-17 23:01 KST.
//! 공식 형식문서 8쪽 Native 99.68651% / fresh WASM 99.66582%에서 추가했다.
//! 근거: mydocs/pr/assets/issues7733_7738_20261117/empty-textbox-outline/cause.md.
//! 공식 한컴 배포 PDF는 재산출하거나 수정하지 않았다.
//! 저장 줄 없는 글상자의 빈 개요 형식도 본문 거리를 점유한다. 제목의 원점만 옮기고
//! 개요 머리가 없는 부제목이나 글상자 자체를 함께 옮기는 오보정도 검출한다.
//! 개인 글꼴의 실제 glyph raster 대신 제품의 본문 원점과 원문 소속을 검사한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn find_run_x(node: &RenderNode, text: &str, values: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text().starts_with(text) {
            values.push(node.bbox.x);
        }
    }
    for child in &node.children {
        find_run_x(child, text, values);
    }
}

#[test]
fn empty_outline_head_reserves_its_print_body_distance_inside_the_textbox() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/한글문서파일형식_5.0_revision1.3.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(7).unwrap();
    for (text, expected) in [("I. ", 226.44), ("(Hwp Document", 264.02)] {
        let mut values = Vec::new();
        find_run_x(&tree.root, text, &mut values);
        assert_eq!(values.len(), 1, "{text}: 원문 제목·부제목 소속 보존");
        assert!(
            (values[0] - expected).abs() <= 2.0,
            "{text}: 실제 {}px, 독립 PDF {expected}px",
            values[0]
        );
    }
}
