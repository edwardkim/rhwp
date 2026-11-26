//! 검사 유형: 관계형 — 확인된 TrueType 돋움의 작은 네모와 전각 글자 폭을 대조한다.
//! 주석 갱신: 2026-11-18 02:06 KST.
//! 화학 원본 28쪽 보정 전 Native 97.56592%이며, 최신 fresh WASM 전쪽은 미검증이다.
//! PDF 절대 좌표를 고정하지 않는다. 설치 원본 두 gulim TTC의 Dotum face에서
//! U+25AA H18543의 hmtx는 384/1024다. 같은 원본의 독립 Print도 이 비율을 사용한다.
//! 해소 슬롯의 미확인 폭 표는 유지하며, 원본 altType=1과 family 체인의 부분 glyph를 확인한다.
//! 원본과 같은 프로그램의 별칭 체인에서도 확인한 폭을 소비하며 명시적 HFT는 유지한다.
//! 검사는 내장 메트릭을 사용하며 개인 글꼴 설치를 요구하지 않는다.

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::TextStyle;

fn width(family: &str, text: &str, size: f64, verified: bool) -> f64 {
    EmbeddedTextMeasurer.estimate_text_width(
        text,
        &TextStyle {
            font_family: family.to_owned(),
            font_size: size,
            font_metric_trusted: verified,
            source_font_is_ttf: verified,
            ..Default::default()
        },
    )
}

#[test]
fn verified_dotum_small_square_preserves_its_original_three_eighths_cell() {
    for family in ["돋움", "Dotum"] {
        for size in [16.0, 24.0, 32.0] {
            let full = width(family, "가", size, true);
            let square = width(family, "▪", size, true);
            assert!(
                (square - full * 3.0 / 8.0).abs() < 1e-6,
                "확인된 {family} 작은 네모의 원폭: 네모={square}, 전각={full}"
            );
        }
    }
}

#[test]
fn explicit_legacy_program_keeps_its_existing_compatibility_cell() {
    for family in ["돋움", "Dotum"] {
        let style = TextStyle {
            font_family: family.to_owned(),
            font_size: 24.0,
            source_font_is_hft: true,
            ..Default::default()
        };
        let full = EmbeddedTextMeasurer.estimate_text_width("가", &style);
        let square = EmbeddedTextMeasurer.estimate_text_width("▪", &style);
        assert!(
            (square - full).abs() < 1e-6,
            "확인하지 않은 프로그램을 새 TrueType 원폭으로 추정하지 않는다"
        );
    }
}

#[test]
fn actual_source_dotum_slot_uses_the_verified_glyph_without_trusting_the_whole_table() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn find(node: &RenderNode) -> Option<&rhwp::renderer::render_tree::TextRunNode> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.starts_with('▪') {
                return Some(run);
            }
        }
        node.children.iter().find_map(find)
    }
    let mut core = rhwp::DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(27).unwrap();
    let run = find(&tree.root).expect("원본 작은 네모");
    println!(
        "family={} ttf={} hft={} trusted={}",
        run.style.font_family,
        run.style.source_font_is_ttf,
        run.style.source_font_is_hft,
        run.style.font_metric_trusted
    );
    assert!(
        !run.style.source_font_is_hft,
        "유형 미상 슬롯을 구형 글꼴로 추정하지 않는다"
    );
    let mut style = run.style.clone();
    style.font_size = 32.0;
    style.ratio = 1.0;
    style.letter_spacing = 0.0;
    let actual = EmbeddedTextMeasurer.estimate_text_width("▪", &style);
    let em = EmbeddedTextMeasurer.estimate_text_width("가", &style);
    assert!(
        (actual - em * 3.0 / 8.0).abs() < 1e-6,
        "전체 폭 표를 신뢰하지 않아도 확인된 원 글리프의 폭은 보존한다: {actual} vs {em}"
    );
}

#[test]
fn document_font_alias_keeps_the_verified_small_square_source_advance() {
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = rhwp::DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"dotum-source-alias","preserveSourceMetrics":true,"substitutions":{"돋움":"Dotum"},"trueTypeFaces":["Dotum"]}"#,
    ).unwrap())).unwrap();
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(27, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let square = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["source"]["character"] == "▪")
        .expect("실제 원본 작은 네모의 폭 결정");
    let shape = &core.document().doc_info.char_shapes
        [square["source"]["charShapeId"].as_u64().unwrap() as usize];
    let expected = f64::from(shape.base_size) * 384.0 / 1024.0;
    let actual = square["layoutMetric"]["baseAdvanceHwpunit"]
        .as_f64()
        .unwrap();
    assert!(
        (actual - expected).abs() <= 1.0,
        "원본과 같은 프로그램의 family 별칭에서도 384/1024em: {actual} vs {expected}; {square}"
    );
}
