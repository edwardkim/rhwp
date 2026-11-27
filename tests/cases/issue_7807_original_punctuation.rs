//! 관계형·고정형 — #7807 HWP3·HWPX 원본 구두점의 소유·순서 및 독립 Print의 전진 비율.
//! 주석 갱신 2026-11-28T03:36:12+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 f3dae4a6483f62e5854417ef73cac111ec5dde10.
//! so-sueop-hwp3 · 기준 pdf/SO-SUEOP-hwp-2020.pdf · 전체 46쪽.
//! 전쪽 최저 Native 98.88% / fresh WASM 98.49%.
//! 13쪽 Native 99.97% / fresh WASM 99.97%.
//! 26쪽 Native 99.39% / fresh WASM 99.52%.
//! 27쪽 Native 99.75% / fresh WASM 99.98%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/so-sueop-hwp3/에 있다.
//! so-sueop-hwpx · 기준 pdf/SO-SUEOP-hwpx-2024.pdf · 전체 46쪽.
//! 전쪽 최저 Native 99.53% / fresh WASM 98.62%.
//! 13쪽 Native 99.96% / fresh WASM 99.97%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/so-sueop-hwpx/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use rhwp::DocumentCore;

fn append_owned_runs<'tree>(
    node: &'tree RenderNode,
    paragraph: usize,
    runs: &mut Vec<(&'tree RenderNode, &'tree TextRunNode)>,
) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(0) && run.para_index == Some(paragraph) {
            runs.push((node, run));
        }
    }
    for child in &node.children {
        append_owned_runs(child, paragraph, runs);
    }
}

fn assert_original_hyphen_advances(input: &str, hangul2024: bool) {
    let bytes = std::fs::read(input).expect("회귀 원본");
    let mut core = DocumentCore::from_bytes(&bytes).expect("원본 문서 열기");
    core.set_hangul2024_compat(hangul2024);
    assert_eq!(core.page_count(), 46, "독립 한컴 Print와 같은 쪽수");
    let (paragraph, source) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, paragraph)| {
            paragraph
                .text
                .starts_with("문서방이 소작료를 내지 못함 --- 인가가")
        })
        .expect("원본 하이픈과 뒤 본문을 소유한 문단");
    let rendered = core.build_page_render_tree(12).expect("정본의 13쪽");
    let mut runs = Vec::new();
    append_owned_runs(&rendered.root, paragraph, &mut runs);
    assert_eq!(
        runs.iter()
            .map(|(_, run)| run.text.as_str())
            .collect::<String>(),
        source.text,
        "하이픈과 앞뒤 본문이 원본 순서로 각각 한 번 배치된다"
    );

    let dash_index = runs
        .iter()
        .position(|(_, run)| run.text == "---")
        .expect("세 하이픈은 한글 본문과 다른 영문 전진 런을 소유한다");
    let (dash_body, dashes) = runs[dash_index];
    let (follow_body, follow) = runs[dash_index + 1];
    assert!(follow.text.trim_start().starts_with("인가가 문서방의"));
    let advances = dashes.layout_positions.clone().unwrap_or_else(|| {
        EmbeddedTextMeasurer.compute_char_positions(&dashes.text, &dashes.style)
    });
    let em = dashes.style.font_size;
    for pair in advances.windows(2).take(2) {
        let advance_em = (pair[1] - pair[0]) / em;
        assert!(
            (0.295..0.330).contains(&advance_em),
            "독립 Print T5 하이픈 전진은 본문 크기의 약 0.31em: {advance_em:.4}",
        );
    }
    assert_eq!(dash_body.bbox.y, follow_body.bbox.y, "같은 물리 글줄");
    assert!(
        follow_body.bbox.x >= dash_body.bbox.x + dash_body.bbox.width - 0.01,
        "뒤 본문은 하이픈 전진 끝에서 이어 그린다"
    );
}

#[test]
fn hwpx_hft_latin_hyphens_keep_their_original_advance_between_hangul() {
    assert_original_hyphen_advances("samples/SO-SUEOP.hwpx", true);
}

#[test]
fn hwp3_hft_latin_hyphens_keep_their_original_advance_between_hangul() {
    assert_original_hyphen_advances("samples/SO-SUEOP.hwp", false);
}

#[test]
fn hwp3_pilgi_quotes_preserve_the_original_half_em_advance() {
    let bytes = std::fs::read("samples/SO-SUEOP.hwp").expect("원본 HWP3");
    let mut core = DocumentCore::from_bytes(&bytes).expect("원본 문서 열기");
    core.set_hangul2024_compat(false);
    let paragraph = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|para| para.text.starts_with("‘닭싸움’을 매개로"))
        .expect("필기 인용을 소유한 원본 문단");
    let rendered = core.build_page_render_tree(26).expect("정본의 27쪽");
    let mut runs = Vec::new();
    append_owned_runs(&rendered.root, paragraph, &mut runs);
    let mut positions = Vec::new();
    for (node, run) in &runs {
        if let Some(start) = run.char_start.filter(|offset| *offset <= 5) {
            let advances = run.layout_positions.clone().unwrap_or_else(|| {
                EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style)
            });
            positions.extend(run.text.chars().enumerate().zip(advances).map(
                |((offset, glyph), advance)| {
                    (
                        start as usize + offset,
                        glyph,
                        node.bbox.x + advance,
                        node.bbox.y,
                    )
                },
            ));
        }
    }
    positions.sort_by_key(|value| value.0);
    let [quote, first, second, third, close, next] = &positions[..6] else {
        panic!("인용문과 뒤 본문의 글자 원점");
    };
    assert_eq!(
        [quote.1, first.1, second.1, third.1, close.1, next.1],
        ['‘', '닭', '싸', '움', '’', '을'],
        "같은 줄에 인용 전후 원문이 순서대로 한 번 표시된다"
    );
    assert!(positions[..6].iter().all(|item| item.3 == quote.3));
    let adjacent_syllable_advance = second.2 - first.2;
    for quote_advance in [first.2 - quote.2, next.2 - close.2] {
        let ratio = quote_advance / adjacent_syllable_advance;
        assert!(
            (0.47..=0.53).contains(&ratio),
            "독립 Print의 필기 따옴표는 인접 한글 전진의 약 절반: {ratio:.4}",
        );
    }
}

#[test]
fn hwp3_myeongjo_one_dot_leader_keeps_a_full_syllable_advance() {
    let bytes = std::fs::read("samples/SO-SUEOP.hwp").expect("회귀 원본");
    let mut core = DocumentCore::from_bytes(&bytes).expect("원본 문서 열기");
    core.set_hangul2024_compat(false);
    let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(
        r#"{
            "id":"original-myeongjo-dot",
            "substitutions":{"명조":"Myeongjo"},
            "selectedPrograms":{"명조":{
                "family":"Myeongjo","query":"명조","programId":"test-myeongjo-cff",
                "trueType":false,"openTypeCff":true
            }},
            "preserveSourceMetrics":true
        }"#,
    )
    .expect("표시용 별칭과 원본 폭 보존 설정");
    core.set_font_environment(Some(environment))
        .expect("실제 Print 비교의 표시용 별칭 사용");
    let rendered = core.build_page_render_tree(25).expect("독립 Print의 26쪽");
    for label in ["구성면에서", "계층적인 인물 설정이나"] {
        let paragraph = core.document().sections[0]
            .paragraphs
            .iter()
            .position(|paragraph| {
                paragraph
                    .text
                    .trim_start()
                    .starts_with(&format!("․{label}"))
            })
            .expect("한점 리더를 소유한 원본 문단");
        let mut runs = Vec::new();
        append_owned_runs(&rendered.root, paragraph, &mut runs);
        let dot = runs
            .iter()
            .position(|(_, run)| run.text == "․")
            .expect("한점 리더");
        let (dot_node, _) = runs[dot];
        let (body_node, body) = runs[dot + 1];
        assert!(body.text.starts_with(label));
        assert_eq!(dot_node.bbox.y, body_node.bbox.y, "같은 줄의 기호와 본문");
        let positions = body.layout_positions.clone().unwrap_or_else(|| {
            EmbeddedTextMeasurer.compute_char_positions(&body.text, &body.style)
        });
        let width = body_node.bbox.x - dot_node.bbox.x;
        let ratio = width / (positions[1] - positions[0]);
        assert!(
            (0.97..=1.04).contains(&ratio),
            "한컴 Print의 한점 리더는 인접 한글 한 글자 폭으로 전진한다: {ratio:.4}"
        );
    }
}
