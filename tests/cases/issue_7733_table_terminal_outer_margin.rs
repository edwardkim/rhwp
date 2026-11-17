//! 검사 유형: 관계형 — 이어받은 표의 마지막 조각과 뒤 저장 문단 원점을 대조한다.
//! 주석 갱신: 2026-11-18 00:56 KST.
//! 화학 원본 40쪽 Native 96.46128%, 이전 10270c15 소스 fresh WASM 96.3035%다.
//! PDF 절대 px 좌표를 고정하지 않는다. Section1 문단 200·201의 원본 vpos와
//! 본문 단 원점, 표의 원본 아래 바깥여백을 같은 좌표계에서 대조한다.
//! 조판이 이미 소비한 바깥여백을 paint 뒤 흐름에서도 보존해야 한다. 글꼴 설치는 필요 없다.

use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

#[test]
fn terminal_fragment_preserves_the_stored_successor_origin_and_outer_bottom_gap() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let paragraphs = &core.document().sections[1].paragraphs;
    let Control::Table(source) = &paragraphs[199].controls[0] else {
        panic!("원본 이어받는 표");
    };
    assert!(!source.common.treat_as_char);
    let bottom_margin = f64::from(source.outer_margin_bottom) * 96.0 / 7200.0;
    assert!(bottom_margin > 0.0);
    let stored: Vec<_> = [200, 201]
        .iter()
        .map(|&pi| {
            let [line] = paragraphs[pi].line_segs.as_slice() else {
                panic!("원본 단일 저장 줄");
            };
            assert_eq!(line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY, 0);
            (pi, f64::from(line.vertical_pos) * 96.0 / 7200.0)
        })
        .collect();
    let tree = core.build_page_render_tree(39).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let column = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .unwrap();
    for (pi, expected) in stored {
        let line = nodes
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::TextLine(meta)
            if meta.para_index == Some(pi))
            })
            .unwrap();
        let actual = line.bbox.y - column.bbox.y;
        assert!((actual - expected).abs() < 1e-6,
            "문단 {pi} 원본 저장 원점 보존: actual={actual}, source={expected}, 아래 여백={bottom_margin}");
    }
}

#[test]
fn existing_terminal_host_flow_consumes_a_changed_outer_margin_only_once() {
    fn successor_y(core: &DocumentCore) -> f64 {
        let tree = core.build_page_render_tree(56).unwrap();
        let mut nodes = Vec::new();
        collect(&tree.root, &mut nodes);
        nodes
            .iter()
            .find(|node| {
                matches!(&node.node_type,
            RenderNodeType::TextLine(meta) if meta.para_index == Some(371))
            })
            .expect("공식 문서의 표 뒤 제목")
            .bbox
            .y
    }
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/한글문서파일형식_5.0_revision1.3.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let before = successor_y(&core);
    let mut source = core.document().clone();
    let Control::Table(table) = &mut source.sections[3].paragraphs[370].controls[0] else {
        panic!("원본 57쪽 마지막 조각");
    };
    table.outer_margin_bottom += 75;
    core.set_document(source);
    let after = successor_y(&core);
    assert!(
        (after - before - 1.0).abs() < 1e-6,
        "같은 종료 여백을 두 번 더하지 않는다: 원본 75HU 변경, 실제 전진 {}",
        after - before
    );
}
