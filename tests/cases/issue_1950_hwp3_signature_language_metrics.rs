//! #1950 원본의 `집행관 (인)` 서명에서 중립 문자의 영문 속성과 내용 순서를 보호한다.
//!
//! 독립 기준은 같은 원본의 한컴 2020 Print PDF다. 원본은 한글 자간 -5%,
//! 영문 자간 0%를 선언한다. 같은 face라는 이유로 공백·괄호에 한글 자간을
//! 상속하면 `(인)`이 왼쪽으로 이동한다. 화면 좌표 대신 원본 속성과 소속·순서를 검사한다.
//! Native/fresh WASM 전쪽 TSV와 확대 판독은 task_m100_7642_stage3905.md에 연결한다.

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn neutral(ch: char) -> bool {
    matches!(ch, ' ' | '(' | ')')
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn signature_runs<'a>(node: &'a RenderNode, paragraph: usize, runs: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(paragraph) && run.cell_context.is_none() {
            runs.push(node);
        }
    }
    for child in &node.children {
        signature_runs(child, paragraph, runs);
    }
}

#[test]
fn signature_neutral_characters_keep_latin_metrics_and_seal_order() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1950_hwp3_tab_charoffset.hwp");
    let bytes = std::fs::read(path).expect("공개 HWP3 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("HWP3 원본 파싱");
    let source = core.document();
    let (paragraph, signature) = source.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, para)| compact(&para.text) == "집행관(인)")
        .expect("원본의 서명 문단");
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 전1쪽");
    let tree = core.build_page_render_tree(0).expect("서명이 있는 첫 쪽");
    let mut runs = Vec::new();
    signature_runs(&tree.root, paragraph, &mut runs);
    assert!(!runs.is_empty(), "서명 문단의 출력 run이 필요하다");

    let mut joined = String::new();
    let mut checked_neutral = 0;
    let mut expected_space_advance = 0.0;
    let mut positions = Vec::new();
    for node in runs {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        joined.push_str(&run.text);
        let shape =
            &source.doc_info.char_shapes[run.char_shape_id.expect("원본 글자모양 소속") as usize];
        let calculated = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        let offsets = run.layout_positions.as_ref().unwrap_or(&calculated);
        assert_eq!(offsets.len(), run.text.chars().count() + 1);
        for (index, ch) in run.text.chars().enumerate() {
            if neutral(ch) {
                let latin_size =
                    f64::from(shape.base_size) / 75.0 * f64::from(shape.relative_sizes[1]) / 100.0;
                let latin_tracking = latin_size * f64::from(shape.spacings[1]) / 100.0;
                assert!(
                    (run.style.letter_spacing - latin_tracking).abs() < 1e-9,
                    "서명 중립 문자 {ch:?}의 자간은 원본 영문 슬롯이어야 한다: 실제 {}, 원본 {latin_tracking}",
                    run.style.letter_spacing,
                );
                assert!(
                    (run.style.ratio - f64::from(shape.ratios[1]) / 100.0).abs() < 1e-9,
                    "서명 중립 문자의 장평도 원본 영문 슬롯을 보존한다",
                );
                if ch == ' ' {
                    assert!(!shape.use_font_space, "이 원본은 기본 반각 공백을 선언한다");
                    assert_eq!(shape.spacings[1], 0, "원본 영문 자간 0%를 확인한다");
                    expected_space_advance += latin_size * 0.5 * f64::from(shape.ratios[1]) / 100.0;
                }
                checked_neutral += 1;
            } else if ('가'..='힣').contains(&ch) {
                let hangul_size =
                    f64::from(shape.base_size) / 75.0 * f64::from(shape.relative_sizes[0]) / 100.0;
                let hangul_tracking = hangul_size * f64::from(shape.spacings[0]) / 100.0;
                assert!(
                    (run.style.letter_spacing - hangul_tracking).abs() < 1e-9,
                    "중립 문자를 분리해도 뒤 한글의 원본 자간은 유지한다",
                );
            }
            positions.push((ch, node.bbox.x + offsets[index], node.bbox.y + run.baseline));
        }
    }
    assert_eq!(
        compact(&joined),
        compact(&signature.text),
        "서명 내용·순서 보존"
    );
    let expected_neutral = signature.text.chars().filter(|&ch| neutral(ch)).count();
    assert!(expected_neutral > 0, "공백과 괄호가 있는 원본이어야 한다");
    assert_eq!(
        checked_neutral, expected_neutral,
        "공백·괄호의 누락·중복 방지"
    );

    let visible: String = positions.iter().map(|&(ch, _, _)| ch).collect();
    assert_eq!(compact(&visible), "집행관(인)");
    for pair in positions.windows(2) {
        assert!(
            pair[0].1 < pair[1].1,
            "서명 글자와 공백은 앞뒤 순서를 유지한다"
        );
    }
    let opening = positions.iter().position(|&(ch, _, _)| ch == '(').unwrap();
    let label_end = positions.iter().position(|&(ch, _, _)| ch == '관').unwrap();
    let blanks = &positions[label_end + 1..opening];
    assert!(!blanks.is_empty() && blanks.iter().all(|&(ch, _, _)| ch == ' '));
    let actual_space_advance = positions[opening].1 - blanks[0].1;
    // 독립 Print에서도 연속 공백은 원본 영문 em의 반각이다. 장치 격자 반올림은
    // 상대 오차 1%로 허용하며, 특정 화면 x좌표나 고정 px 간격을 기대값으로 삼지 않는다.
    let advance_ratio = actual_space_advance / expected_space_advance;
    assert!(
        (0.99..=1.01).contains(&advance_ratio),
        "날인까지의 실제 공백 전진이 원본 영문 반각 합계와 달라졌다: 비율 {advance_ratio}",
    );
    let seal: Vec<_> = positions
        .iter()
        .filter(|&&(ch, _, _)| matches!(ch, '(' | '인' | ')'))
        .collect();
    assert_eq!(seal.len(), 3, "서명 괄호와 인의 누락·중복 방지");
    for mark in &seal {
        assert!(
            mark.1 > positions[label_end].1,
            "날인 위치는 집행관 다음이다"
        );
        assert!(
            mark.1 < tree.root.bbox.width,
            "날인 문구는 쪽 안에 있어야 한다"
        );
        assert!((mark.2 - seal[0].2).abs() < 1e-9, "날인 문구의 기준선 보존");
    }
}
