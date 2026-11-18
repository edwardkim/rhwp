//! 검사 유형: 관계형 — 위첨자와 뒤 본문의 같은 줄 소속·원문 보존을 검사한다.
//! 주석 갱신: 2026-11-18 09:45 KST.
//! 시행령 13쪽 보정 전 Native 99.16669%, fresh WASM 99.12883%(c89572f72a).
//! 정상 글꼴 환경·PDFium 96 DPI·Print·2px raw. px 좌표를 고정하지 않는다.
//! 한컴 2020 Hancom PDF Print의 3글꼴×3크기 독립 대조와 원본 13쪽에서
//! 본문/첨자 크기 92/59 장치 단위, 상향 0.44em을 확인했다.
//! 18쪽의 일반 공백/글꼴 공백 구분은 한컴 2020·2024 Print 36조건으로 대조한다.
//! 원본 속성과 독립 Print가 요구하는 `신고`의 줄 소속은 최소 글꼴 환경에서도 동일하다.

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
        script_metrics::ScriptMetricsContract,
    },
    DocumentCore,
};

fn line_text(node: &RenderNode) -> String {
    match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => node.children.iter().map(line_text).collect(),
    }
}

fn collect_lines(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(line_text(node));
    } else {
        for child in &node.children {
            collect_lines(child, out);
        }
    }
}

#[test]
fn desktop_print_superscript_keeps_the_following_word_on_its_source_line() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(
        FontEnvironment::from_json(include_str!(
            "../../mydocs/pr/assets/issues7733_7738_20261117/decree-print-font-environment/font-environment.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    assert_eq!(core.page_count(), 74, "독립 Print의 전체 쪽수");
    let tree = core.build_page_render_tree(12).unwrap();
    let mut lines = Vec::new();
    collect_lines(&tree.root, &mut lines);
    let start = lines
        .iter()
        .position(|line| line.starts_with("위원회가 고시로 정하는 기준"))
        .expect("독립 Print 13쪽의 이어지는 본문 줄");
    assert_eq!(
        lines[start].trim_end(),
        "위원회가 고시로 정하는 기준* 이하인 경우에는 신고",
        "첨자 크기를 본문보다 크게 측정하여 마지막 글자를 다음 줄로 밀지 않는다"
    );
    assert_eq!(lines[start + 1].trim(), "의무 면제");
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("위원회가 고시로 정하는 기준"))
            .count(),
        1,
        "해당 본문은 같은 쪽에 한 번만 소속한다"
    );
}

fn first_contract(node: &RenderNode) -> Option<ScriptMetricsContract> {
    match &node.node_type {
        RenderNodeType::TextRun(run) => Some(run.style.script_metrics),
        _ => node.children.iter().find_map(first_contract),
    }
}

#[test]
fn hwp3_layout_retains_its_distinct_legacy_script_contract() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample16.hwp")).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    assert_eq!(
        first_contract(&tree.root),
        Some(ScriptMetricsContract::Legacy)
    );
}

#[test]
fn desktop_script_spaces_keep_body_half_space_and_explicit_glyph_space_separate() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::TextStyle;
    let measure = |style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions(" ", style)[1];
    for family in ["HYGothic-Medium", "Malgun Gothic", "Haansoft Batang"] {
        for size in [12.0, 44.0 / 3.0, 24.0] {
            for superscript in [true, false] {
                for layout_half_space in [false, true] {
                    let body = TextStyle {
                        font_family: family.into(),
                        font_size: size,
                        script_metrics: ScriptMetricsContract::Hangul2024,
                        layout_half_space,
                        ..Default::default()
                    };
                    let script = TextStyle {
                        superscript,
                        subscript: !superscript,
                        ..body.clone()
                    };
                    assert_eq!(
                        measure(&script),
                        measure(&body),
                        "일반 공백은 첨자에 속해도 본문 em을 유지한다: {family} {size}, reflow={layout_half_space}"
                    );
                    let inherited_space = TextStyle {
                        font_space_em: Some(0.5),
                        inherited_half_space_em: true,
                        ..script.clone()
                    };
                    let inherited_body = TextStyle {
                        superscript: false,
                        subscript: false,
                        ..inherited_space.clone()
                    };
                    assert_eq!(
                        measure(&inherited_space),
                        measure(&inherited_body),
                        "상속한 언어의 반각 em은 명시적 글꼴 공백으로 오인하지 않는다"
                    );
                    let glyph_space = TextStyle {
                        font_space_em: Some(1.0 / 3.0),
                        ..script.clone()
                    };
                    let stored_glyph_space = TextStyle {
                        layout_half_space: false,
                        ..glyph_space.clone()
                    };
                    assert_eq!(
                        measure(&glyph_space),
                        measure(&stored_glyph_space),
                        "글꼴 공백 사용은 저장/재조판 모두 첨자 크기로 잰다"
                    );
                    let body_glyph_space = TextStyle {
                        superscript: false,
                        subscript: false,
                        ..glyph_space
                    };
                    assert!(measure(&stored_glyph_space) < measure(&body_glyph_space));
                }
            }
        }
    }
}
