//! 영어 시험지의 중립 공백 전진·내용 소유와 독립 Print 좌표를 보호한다.
//!
//! 전8쪽 Native/fresh WASM과 5쪽의 99%대 직접 판독 증거:
//! mydocs/pr/assets/issue7642/english_neutral_space_20261019/README.md
//! 첫 검사는 원본 em과 공백의 비율·순서를 검사한다. 두 번째의 ±2px만
//! 양쪽 raw가 98% 이상인 완료된 5쪽의 독립 PDF 원점에 적용한다.

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn compact(value: &str) -> String {
    value.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn owned_runs<'a>(node: &'a RenderNode, owner: usize, runs: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(owner) && run.cell_context.is_none() {
            runs.push(node);
        }
    }
    for child in &node.children {
        owned_runs(child, owner, runs);
    }
}

#[test]
fn english_neutral_spaces_preserve_half_em_advance_and_source_order() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_eng.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let document = core.document();
    let (owner, paragraph) = document.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| compact(&p.text) == "①학교종소리교체계획을알리려고")
        .expect("독립 Print 첫 문항의 첫 선택지");
    assert_eq!(core.page_count(), 8, "전쪽 Print와 같은 여덟 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut runs = Vec::new();
    owned_runs(&tree.root, owner, &mut runs);
    let mut text = String::new();
    let mut spaces = 0;
    let mut previous = None;
    for node in runs {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        let shape = &document.doc_info.char_shapes[run.char_shape_id.unwrap() as usize];
        let measured = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        let positions = run.layout_positions.as_ref().unwrap_or(&measured);
        assert_eq!(positions.len(), run.text.chars().count() + 1);
        text.push_str(&run.text);
        for (i, ch) in run.text.chars().enumerate() {
            let x = node.bbox.x + positions[i];
            if let Some(before) = previous {
                assert!(x > before, "원문 글자·공백의 전진 순서");
            }
            previous = Some(x);
            if !matches!(ch, ' ' | '\u{00a0}') {
                continue;
            }
            assert!(!shape.use_font_space, "기본 반각을 선언한 원본");
            assert_eq!(shape.spacings[1], 0, "독립 Print의 영문 자간 0%");
            assert_eq!(shape.relative_sizes[0], shape.relative_sizes[5]);
            assert_eq!(
                shape.ratios[0], shape.ratios[5],
                "원문 한글·선택지 기호의 같은 반각 기준"
            );
            assert_ne!(shape.relative_sizes[0], shape.relative_sizes[1]);
            assert_ne!(shape.ratios[0], shape.ratios[1]);
            let inherited_em =
                f64::from(shape.base_size) / 75.0 * f64::from(shape.relative_sizes[0]) / 100.0
                    * f64::from(shape.ratios[0])
                    / 100.0;
            // PDF의 단어 원점 간 차이에서 같은 HFT 글리프의 전진을 제외하면
            // 공백은 상속 em의 반각이다. 장치 반올림을 상대 오차로 허용한다.
            let ratio = (positions[i + 1] - positions[i]) / inherited_em;
            assert!(
                (0.49..=0.51).contains(&ratio),
                "공백 {ch:?} 전진은 원래 em의 반각이어야 한다: 실제 비율 {ratio}"
            );
            spaces += 1;
        }
    }
    assert_eq!(
        compact(&text),
        compact(&paragraph.text),
        "내용·순서·소유 보존"
    );
    assert_eq!(
        spaces,
        paragraph
            .text
            .chars()
            .filter(|ch| matches!(ch, ' ' | '\u{00a0}'))
            .count(),
        "공백의 누락·중복 방지"
    );
    assert!(spaces > 1, "선택지 앞 공백과 단어 사이 공백을 함께 검사");
}

#[test]
fn english_completed_page_five_keeps_print_word_origins_and_owned_content() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_eng.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let source = core.document();
    let (owner, paragraph) = source.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| compact(&p.text) == "29.다음글의밑줄친부분중,어법상틀린것은?")
        .expect("독립 Print 5쪽의 문29 지시문");
    assert_eq!(core.page_count(), 8, "독립 Print의 전체 쪽수");
    let mut expected_page_runs = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut runs = Vec::new();
        owned_runs(&tree.root, owner, &mut runs);
        if page == 4 {
            expected_page_runs = runs.into_iter().cloned().collect();
        } else {
            assert!(
                runs.is_empty(),
                "지시문의 다른 쪽 중복·잘못된 소속: {}",
                page + 1
            );
        }
    }
    let mut joined = String::new();
    let mut anchors = Vec::new();
    let mut previous = None;
    for node in &expected_page_runs {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        joined.push_str(&run.text);
        let measured = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        let offsets = run.layout_positions.as_ref().unwrap_or(&measured);
        assert_eq!(offsets.len(), run.text.chars().count() + 1);
        let (_, y) = run
            .style
            .script_draw_metrics(run.style.font_size, node.bbox.y + run.baseline);
        for (i, ch) in run.text.chars().enumerate() {
            let x = node.bbox.x + offsets[i];
            if let Some(before) = previous {
                assert!(x >= before, "원문 순서·공백 전진 유지");
            }
            previous = Some(x);
            if matches!(ch, '다' | '어' | '것') {
                anchors.push((ch, x, y));
            }
        }
    }
    assert_eq!(
        compact(&joined),
        compact(&paragraph.text),
        "문29 내용·순서·누락·중복 보존"
    );
    // 한컴 2020 Print PDF의 다/어/것 glyph origin을 pt × 96/72로 환산했다.
    // 위치로 run을 선택하지 않고 원문 문단·문자로 선택한 뒤 독립 원점을 대조한다.
    let expected = [('다', 151.199992), ('어', 336.480001), ('것', 419.679972)];
    assert_eq!(anchors.len(), expected.len());
    for ((ch, x, y), (printed_ch, printed_x)) in anchors.into_iter().zip(expected) {
        assert_eq!(ch, printed_ch);
        assert!(
            (x - printed_x).abs() <= 2.0,
            "완료된 5쪽의 {ch} 원점 x={x}, 독립 Print x={printed_x}"
        );
        assert!(
            (y - 227.039998).abs() <= 2.0,
            "완료된 5쪽의 {ch} 기준선 y={y}, 독립 Print y=227.039998"
        );
    }
}
