//! 검사 유형: 관계형 — 같은 문단 모양의 일반 목록과 인라인 표 목록을 대조한다.
//! 주석 갱신: 2026-11-18 02:06 KST.
//! 화학 95쪽 Native 94.95486%, 이전 10270c15 소스 fresh WASM 94.94197%다.
//! PDF 절대 좌표를 고정하지 않는다. 원본 Section4 문단 259·260은 같은 ParaShape 149다.
//! 표가 든 문단도 같은 글머리표 앞 영역과 후속 줄 내어쓰기를 소비해야 한다.
//! 표의 원본 줄이 바깥 상자를 정확히 담으면 그 줄의 상단·위 여백에서 표가 시작한다.
//! 원본의 첫 본문 괄호와 후속 줄의 원점을 비교하며 개인 글꼴 설치는 요구하지 않는다.
//! 11221e7710 뒤 Native 집중 출력에서 텍스트 줄의 정렬은 복구됐지만, TAC 줄까지
//! 배분한 중간 후보는 97.36%로 후퇴했다. 실제 컨트롤의 저장 줄 소속을 별도로 보호한다.

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

fn text_only_line_fills_its_own_stored_region(line_index: usize, prefix: &str) {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let source = &core.document().sections[4].paragraphs[260];
    let stored = &source.line_segs[line_index];
    assert!(
        source.line_segs.len() > 3,
        "검사 줄은 문단의 마지막 줄이 아니다"
    );
    let right_offset = f64::from(stored.column_start + stored.segment_width) * 96.0 / 7200.0;
    let tree = core.build_page_render_tree(94).unwrap();
    let mut nodes = Vec::new();
    runs(&tree.root, &mut nodes);
    let column = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .unwrap();
    let first = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(260) && run.text.starts_with(prefix))
        })
        .expect("표를 담지 않은 저장 본문 줄");
    let last = nodes.iter().rev().find(|node| matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(260) && (node.bbox.y - first.bbox.y).abs() < 1e-6 && !run.text.trim_end_matches(' ').is_empty()))
        .unwrap();
    let RenderNodeType::TextRun(run) = &last.node_type else {
        unreachable!()
    };
    let visible = run.text.trim_end_matches(' ').chars().count();
    let positions = run
        .layout_positions
        .as_ref()
        .expect("인라인 본문 문자별 전진");
    // 저장 우단은 가시 점유 끝이다. 마지막 글자 뒤의 signed 자간은 다음
    // 글자가 없는 이 경계에 포함하지 않는다. advance 자체와 구분한다.
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    let final_character = run.text.chars().nth(visible - 1).unwrap().to_string();
    let tracked = *EmbeddedTextMeasurer
        .compute_char_positions(&final_character, &run.style)
        .last()
        .unwrap();
    let mut natural_style = run.style.clone();
    natural_style.letter_spacing = 0.0;
    if let Some(latin) = natural_style.ascii_punct_latin.as_mut() {
        latin.letter_spacing = 0.0;
    }
    let natural = *EmbeddedTextMeasurer
        .compute_char_positions(&final_character, &natural_style)
        .last()
        .unwrap();
    let actual = last.bbox.x + positions[visible] - (tracked - natural);
    let expected = column.bbox.x + right_offset;
    assert!(
        (actual - expected).abs() <= 96.0 / 7200.0,
        "표가 없는 중간 줄은 자신의 저장 우단까지 공백을 배분한다: {actual} vs {expected}; first={:?}, last={:?}, text={:?}, size={}, ratio={}, tracking={}, word={}, char={}, tracked={}, natural={}",
        first.bbox, last.bbox, run.text, run.style.font_size, run.style.ratio,
        run.style.letter_spacing, run.style.extra_word_spacing, run.style.extra_char_spacing, tracked, natural
    );
}

#[test]
fn text_only_continuation_after_inline_table_fills_its_own_stored_line_region() {
    text_only_line_fills_its_own_stored_region(2, "제품이");
}

#[test]
fn text_only_opening_before_inline_table_fills_its_own_stored_line_region() {
    text_only_line_fills_its_own_stored_region(0, "(");
}

#[test]
fn table_owned_line_keeps_its_natural_prefix_and_does_not_distribute_the_object_width() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let source = &core.document().sections[4].paragraphs[260];
    let Control::Table(table) = &source.controls[0] else {
        panic!("원본 인라인 표")
    };
    let left_margin = f64::from(table.outer_margin_left) * 96.0 / 7200.0;
    let tree = core.build_page_render_tree(94).unwrap();
    let mut nodes = Vec::new();
    runs(&tree.root, &mut nodes);
    let arrow = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(260) && run.text == "→ ")
        })
        .expect("표 바로 앞 원본 화살표");
    let RenderNodeType::TextRun(run) = &arrow.node_type else {
        unreachable!()
    };
    let natural = EmbeddedTextMeasurer.estimate_text_width(&run.text, &run.style);
    let table = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(260) && meta.control_index == Some(0))
        })
        .unwrap();
    assert!(
        (table.bbox.x - arrow.bbox.x - natural - left_margin).abs() <= 96.0 / 7200.0,
        "TAC 줄의 실제 표 폭을 텍스트 여유로 중복 배분하지 않는다"
    );
}
