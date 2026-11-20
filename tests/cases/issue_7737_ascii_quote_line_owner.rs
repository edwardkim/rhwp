//! 함수별 검사 유형: 줄 소속·금칙은 관계형, 독립 Print 원점은 고정형으로 구분한다.
//! 주석 갱신: 2026-11-18 11:02:20 KST.
//! 시행령 61쪽 보정 전 Native 99.55732% / fresh WASM 99.55724%.
//! 96 DPI·compat2022·PDFium·Hancom PDF Print·색상 경계 보정 전 2px raw다.
//! 독립 한컴 2020 Print는 첫 ASCII 작은따옴표를 둘째 줄 끝에 남긴다.
//! 원점 검사는 양쪽 98% 이상인 같은 61쪽만 ±2px로 보호한다.
//! 설치 글꼴·비공개 프로그램에 의존하지 않는다.
//! 추가 관계형 검사: 한컴 2024 Print 7·16쪽의 마지막 음절과 다음 줄 소속.
//! Native 96 DPI·2px raw는 각각 100.00%, 99.99%이며 좌표를 고정하지 않는다.

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

/// 검사 유형: 관계형. 표의 폭 중립 표식과 본문에서 독립 Print의 문단 끝 소속을 비교한다.
#[test]
fn compressed_terminal_syllable_owns_an_automatic_continuation_line() {
    fn collect_lines(node: &RenderNode, lines: &mut Vec<String>) {
        if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
            lines.push(
                node.children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect(),
            );
        }
        for child in &node.children {
            collect_lines(child, lines);
        }
    }
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    core.set_hangul2024_compat(true);
    core.set_font_environment(Some(
        FontEnvironment::from_json(
            r#"{
                "id": "decree-2024-terminal-ownership",
                "substitutions": {"HCI Poppy": "Gulim", "한양중고딕": "HYGothic-Medium"},
                "trueTypeFaces": ["Gulim", "HYGothic-Medium"],
                "preserveSourceMetrics": true
            }"#,
        )
        .unwrap(),
    ))
    .unwrap();
    assert_eq!(core.page_count(), 74);
    for (page, prefix, ending, continuation) in [
        (7, "'정보제공형 신고'", "'에 해", "당"),
        (16, "내용 등과 관련한 산업계", "모니터", "링"),
    ] {
        let tree = core.build_page_render_tree(page - 1).unwrap();
        let mut lines = Vec::new();
        collect_lines(&tree.root, &mut lines);
        let mut source = lines.iter().skip_while(|line| !line.starts_with(prefix));
        assert!(
            source
                .next()
                .is_some_and(|line| line.trim_end().ends_with(ending)),
            "{page}쪽 문단 마지막 음절 앞의 줄"
        );
        assert_eq!(
            source.next().map(|line| line.trim()),
            Some(continuation),
            "{page}쪽 마지막 음절의 줄 소속"
        );
        assert!(
            !source.any(|line| line.trim() == continuation),
            "{page}쪽 마지막 음절 중복"
        );
    }
}

/// 검사 유형: 관계형. 문자·문단·줄의 소속과 수량을 보존한다.
#[test]
fn ascii_apostrophe_keeps_the_independent_print_line_and_following_text_owner() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    core.set_font_environment(Some(
        FontEnvironment::from_json(include_str!(
            "../../mydocs/pr/assets/issues7733_7738_20261117/decree-original-program-selection/font-environment-native.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    assert_eq!(core.page_count(), 74);
    let tree = core.build_page_render_tree(60).unwrap();
    let mut nodes = Vec::new();
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.cell_context.as_ref().is_some_and(|context| {
                context.parent_para_index == 622
                    && context
                        .path
                        .first()
                        .is_some_and(|entry| entry.cell_index == 29 && entry.cell_para_index == 0)
            }) {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    collect(&tree.root, &mut nodes);
    nodes.sort_by(|a, b| {
        a.bbox
            .y
            .total_cmp(&b.bbox.y)
            .then(a.bbox.x.total_cmp(&b.bbox.x))
    });
    let mut rows: Vec<(f64, String)> = Vec::new();
    for node in nodes {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        if let Some((y, text)) = rows
            .last_mut()
            .filter(|(y, _)| (*y - node.bbox.y).abs() < 1e-6)
        {
            let _ = y;
            text.push_str(&run.text);
        } else {
            rows.push((node.bbox.y, run.text.clone()));
        }
    }
    assert_eq!(rows.len(), 6, "원본 비용 문단의 여섯 줄 소속");
    assert!(
        rows[1].1.trim_end().ends_with('\''),
        "독립 Print의 ASCII 작은따옴표를 다음 줄로 밀면 양쪽 정렬 간격도 달라진다: {}",
        rows[1].1
    );
    assert!(rows[2].1.trim_start().starts_with("메타버스"));
    let text: String = rows.into_iter().map(|(_, text)| text).collect();
    assert_eq!(text.matches('\'').count(), 2, "원본 따옴표의 수량 보존");
    assert_eq!(text.matches("메타버스 지원센터").count(), 1);
}

fn independent_control_rows(spaced: bool) -> std::collections::BTreeMap<usize, Vec<String>> {
    let bytes: &[u8] = if spaced {
        include_bytes!("../../mydocs/pr/assets/issues7733_7738_20261117/decree-ascii-quote-line-owner/with-leading-space/quote-space-boundary-2020-controls.hwpx")
    } else {
        include_bytes!("../../mydocs/pr/assets/issues7733_7738_20261117/decree-ascii-quote-line-owner/quote-boundary-2020-controls.hwpx")
    };
    let core = DocumentCore::from_bytes(bytes).unwrap();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut paragraphs: std::collections::BTreeMap<usize, Vec<(f64, f64, String)>> =
        std::collections::BTreeMap::new();
    fn collect(
        node: &RenderNode,
        out: &mut std::collections::BTreeMap<usize, Vec<(f64, f64, String)>>,
    ) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if let Some(index) = run.para_index {
                out.entry(index)
                    .or_default()
                    .push((node.bbox.y, node.bbox.x, run.text.clone()));
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    collect(&tree.root, &mut paragraphs);
    paragraphs
        .into_iter()
        .map(|(index, mut runs)| {
            runs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
            let mut rows: Vec<(f64, String)> = Vec::new();
            for (y, _, text) in runs {
                if let Some((_, row)) = rows
                    .last_mut()
                    .filter(|(previous, _)| (previous - y).abs() < 1e-6)
                {
                    row.push_str(&text);
                } else {
                    rows.push((y, text));
                }
            }
            (index, rows.into_iter().map(|(_, text)| text).collect())
        })
        .collect()
}

/// 검사 유형: 관계형. 독립 대조군의 ASCII 따옴표와 다음 한글이 서로 다른 줄에 속한다.
#[test]
fn independent_ascii_quotes_can_finish_the_line_before_hangul() {
    let paragraphs = independent_control_rows(true);
    for family in 0..2 {
        for (condition, quote) in [(0, '\''), (1, '"')] {
            let rows = &paragraphs[&(2 + 2 * (family * 6 + condition))];
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0], format!("가가가가가가 {quote}"));
            assert_eq!(rows[1], "나나다라");
        }
    }
}

/// 검사 유형: 관계형. 방향성 여는 기호의 금칙과 닫는 prime의 기존 소속을 보존한다.
#[test]
fn directional_openers_and_existing_closing_prime_keep_their_owners() {
    let paragraphs = independent_control_rows(true);
    for family in 0..2 {
        for (condition, quote) in [(2, '‘'), (3, '“'), (4, '(')] {
            let rows = &paragraphs[&(2 + 2 * (family * 6 + condition))];
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].trim_end(), "가가가가가가");
            assert_eq!(rows[1], format!("{quote}나나다라"));
        }
        let rows = &paragraphs[&(2 + 2 * (family * 6 + 5))];
        assert_eq!(rows.len(), 2);
        assert!(rows[0].starts_with("AAAAAA'"));
        assert_eq!(rows.concat(), "AAAAAA'가나다라");
    }
}

/// 검사 유형: 고정형. 독립 Print 61쪽의 ASCII 원점과 다음 줄 시작을 ±2px로 검사한다.
#[test]
fn original_ascii_quote_and_next_line_keep_the_independent_print_origins() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    core.set_font_environment(Some(
        FontEnvironment::from_json(include_str!(
            "../../mydocs/pr/assets/issues7733_7738_20261117/decree-original-program-selection/font-environment-native.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    let tree = core.build_page_render_tree(60).unwrap();
    let mut nodes = Vec::new();
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.cell_context.as_ref().is_some_and(|context| {
                context.parent_para_index == 622
                    && context
                        .path
                        .first()
                        .is_some_and(|entry| entry.cell_index == 29 && entry.cell_para_index == 0)
            }) {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    collect(&tree.root, &mut nodes);
    let first_quote = nodes
        .iter()
        .copied()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains('\'')))
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .expect("원본 ASCII 작은따옴표");
    let body = nodes
        .iter()
        .copied()
        .find(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.starts_with("메타버스")))
        .expect("독립 Print의 다음 줄 본문");
    let baseline = |node: &RenderNode| match &node.node_type {
        RenderNodeType::TextRun(run) => node.bbox.y + run.baseline,
        _ => unreachable!(),
    };
    let RenderNodeType::TextRun(run) = &first_quote.node_type else {
        unreachable!()
    };
    let quote_index = run.text.chars().position(|ch| ch == '\'').unwrap();
    let positions = run
        .layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style));
    let quote_x = first_quote.bbox.x + positions[quote_index];
    assert!(
        (quote_x - 717.2799).abs() <= 2.0,
        "줄 끝 ASCII 원점: {quote_x}"
    );
    assert!((baseline(first_quote) - 116.48).abs() <= 2.0);
    assert!((body.bbox.x - 268.8).abs() <= 2.0);
    assert!((baseline(body) - 140.0).abs() <= 2.0);
}
