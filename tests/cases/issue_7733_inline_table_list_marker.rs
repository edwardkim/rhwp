//! 검사 유형: 관계형 — 같은 문단 모양의 일반 목록과 인라인 표 목록을 대조한다.
//! 주석 갱신: 2026-11-18 00:25 KST.
//! 화학 95쪽 Native 94.95486%, 이전 10270c15 소스 fresh WASM 94.94197%다.
//! PDF 절대 좌표를 고정하지 않는다. 원본 Section4 문단 259·260은 같은 ParaShape 149다.
//! 표가 든 문단도 같은 글머리표 앞 영역과 후속 줄 내어쓰기를 소비해야 한다.
//! 표의 원본 줄이 바깥 상자를 정확히 담으면 그 줄의 상단·위 여백에서 표가 시작한다.
//! 원본의 첫 본문 괄호와 후속 줄의 원점을 비교하며 개인 글꼴 설치는 요구하지 않는다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        runs(child, out);
    }
}

#[test]
fn inline_table_list_uses_the_same_body_and_hanging_origin_as_its_plain_peer() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let paragraphs = &core.document().sections[4].paragraphs;
    assert_eq!(paragraphs[259].para_shape_id, paragraphs[260].para_shape_id);
    assert!(paragraphs[260]
        .controls
        .iter()
        .any(|control| matches!(control, Control::Table(table) if table.common.treat_as_char)));
    let Control::Table(source) = &paragraphs[260].controls[0] else {
        panic!("원본 인라인 표");
    };
    let line = &paragraphs[260].line_segs[1];
    assert_eq!(
        i64::from(line.line_height),
        i64::from(source.common.height)
            + i64::from(source.outer_margin_top)
            + i64::from(source.outer_margin_bottom),
        "표가 소유한 저장 줄의 바깥 상자"
    );
    let table_top_offset = f64::from(
        line.vertical_pos - paragraphs[260].line_segs[0].vertical_pos
            + i32::from(source.outer_margin_top),
    ) * 96.0
        / 7200.0;
    let tree = core.build_page_render_tree(94).unwrap();
    let mut nodes = Vec::new();
    runs(&tree.root, &mut nodes);
    let origin = |pi, prefix: &str| {
        nodes
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.para_index == Some(pi) && run.text.starts_with(prefix))
            })
            .unwrap_or_else(|| panic!("문단 {pi} 본문 {prefix}"))
            .bbox
            .x
    };
    let plain = origin(259, "(14.");
    let inline = origin(260, "(");
    assert!(
        (plain - inline).abs() < 1e-6,
        "같은 글머리표 영역의 첫 본문 원점: 일반={plain}, 표 문단={inline}"
    );
    let continued = origin(259, "잉크");
    let arrow = origin(260, "→");
    assert!(
        (continued - arrow).abs() < 1e-6,
        "후속 줄은 같은 내어쓰기를 소비한다: 일반={continued}, 표 문단={arrow}"
    );
    let first_y = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(260) && run.text == "(" )
        })
        .unwrap()
        .bbox
        .y;
    let table = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(260) && meta.control_index == Some(0))
        })
        .unwrap();
    assert!(
        (table.bbox.y - first_y - table_top_offset).abs() < 1e-6,
        "표는 자기 저장 줄의 상단과 위 바깥 여백을 소비한다"
    );
    for pi in [259, 260] {
        let markers = nodes
            .iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.para_index == Some(pi) && run.char_start.is_none())
            })
            .count();
        assert_eq!(markers, 1, "문단 {pi} 글머리표의 누락·중복 방지");
    }
}
