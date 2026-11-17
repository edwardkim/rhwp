//! 검사 유형: 관계형 — 원본 저장 줄의 단 원점·cs·내어쓰기와 실제 본문 원점을 대조한다.
//! 주석 갱신: 2026-11-18 00:03 KST.
//! 화학 원본 13쪽 Native 99.98064%, fresh WASM 99.99100%에서 추가했다.
//! 같은 문단에 Square 그림이 있어도 전폭 줄의 문단 여백을 중복 소비하지 않는다.
//! 그림에 실제로 잘린 줄은 기존 square_stored_line_start 검사가 별도로 보호한다.
//! PDF 절대 px 좌표나 특정 글꼴의 raster를 기대값으로 사용하지 않는다.

use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect(child, nodes);
    }
}

#[test]
fn full_width_source_lines_do_not_add_the_paragraph_margin_twice() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let source = &core.document().sections[0].paragraphs[131];
    assert!(source.controls.iter().any(|control| {
        matches!(control, Control::Picture(picture)
            if !picture.common.treat_as_char && picture.common.text_wrap == TextWrap::Square)
    }));
    let shape = &core.document().doc_info.para_shapes[source.para_shape_id as usize];
    let hanging = f64::from((-shape.indent).max(0)) * 96.0 / 14400.0;
    assert!(hanging > 0.0, "원본의 후속 줄 내어쓰기 보존");
    let tree = core.build_page_render_tree(12).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let column = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .unwrap();
    let mut lines: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(131))
        })
        .collect();
    lines.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
    assert_eq!(
        lines.len(),
        source.line_segs.len(),
        "원본 줄 소속·개수 보존"
    );
    for (index, (line, original)) in lines.iter().zip(&source.line_segs).enumerate() {
        let relative = line.bbox.x - column.bbox.x;
        let stored = f64::from(original.column_start) * 96.0 / 7200.0
            + if index == 0 { 0.0 } else { hanging };
        assert!(
            (relative - stored).abs() < 1e-6,
            "본문 원점 {relative}는 원본 cs {stored}를 한 번만 소비해야 한다: y={} / 원본 vpos={}",
            line.bbox.y,
            original.vertical_pos
        );
    }
}
