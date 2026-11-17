//! 검사 유형: 관계형 — 원본 결측 기호의 폭과 독립 Print 공급 프로그램의 전진을 대조한다.
//! 주석 갱신: 2026-11-18 02:41 KST.
//! 시행령 13쪽 보정 전 Native 96.51000%이며 최신 fresh WASM 전쪽은 미검증이다.
//! PDF 절대 좌표는 고정하지 않는다. 맑은 고딕 보통·굵은 프로그램에는 U+2981이 없다.
//! 독립 한컴 2020 Hancom PDF Print는 HCRDotum의 같은 glyph 윤곽과 hmtx 970/1000em을 사용한다.
//! 실제 원본 네 기호는 보통 굵기 슬롯이다. 다른 굵기의 공급 폭은 이 근거로 인증하지 않는다.
//! 원본의 다른 글자·명시적 HFT·호출자가 정한 glyph 공급자는 바꾸지 않는다.
//! 검사는 내장 폭과 공개 결정 추적을 사용하며 개인 글꼴 설치를 요구하지 않는다.

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::TextStyle;
use rhwp::DocumentCore;

#[test]
fn missing_malgun_spot_uses_the_same_advance_as_the_independent_print_supplier() {
    for family in ["Malgun Gothic", "맑은 고딕"] {
        let source = TextStyle {
            font_family: family.into(),
            font_size: 32.0,
            source_font_is_ttf: true,
            ..Default::default()
        };
        let supplied = TextStyle {
            font_family: "HCR Dotum".into(),
            font_size: 32.0,
            source_font_is_ttf: true,
            ..Default::default()
        };
        let actual = EmbeddedTextMeasurer.estimate_text_width("⦁", &source);
        let expected = EmbeddedTextMeasurer.estimate_text_width("⦁", &supplied);
        assert!(
            (actual - expected).abs() <= 96.0 / 7200.0,
            "확인한 원본 결측은 같은 공급자의 전진을 쓴다: family={family}, {actual} vs {expected}"
        );
    }
}

#[test]
fn actual_source_spots_keep_the_declared_slot_and_use_the_verified_missing_glyph_supplier() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(12, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let spots: Vec<_> = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["source"]["character"] == "⦁")
        .collect();
    assert_eq!(spots.len(), 4, "원본 13쪽의 네 기호를 모두 검증한다");
    for spot in spots {
        assert_eq!(
            spot["document"]["face"], "맑은 고딕",
            "원본 글꼴 선언은 보존한다"
        );
        let shape = &core.document().doc_info.char_shapes
            [spot["source"]["charShapeId"].as_u64().unwrap() as usize];
        let expected = f64::from(shape.base_size) * 970.0 / 1000.0;
        let actual = spot["layoutMetric"]["baseAdvanceHwpunit"].as_f64().unwrap();
        assert!(
            (actual - expected).abs() <= 1.0,
            "반각 추정 대신 독립 Print 공급 프로그램의 폭: {actual} vs {expected}; {spot}"
        );
    }
}

#[test]
fn explicit_hft_does_not_borrow_a_truetype_programs_missing_glyph_contract() {
    let style = TextStyle {
        font_family: "Malgun Gothic".into(),
        font_size: 32.0,
        source_font_is_hft: true,
        ..Default::default()
    };
    assert_eq!(
        EmbeddedTextMeasurer.estimate_text_width("⦁", &style),
        16.0,
        "기존 HFT 미확인 폭을 다른 TTF 프로그램의 cmap으로 바꾸지 않는다"
    );
}

#[test]
fn explicit_glyph_supplier_keeps_priority_over_the_generated_fallback_chain() {
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(
        FontEnvironment::from_json(
            r#"{"id":"explicit-spot-supplier","substitutions":{},"glyphFallbacks":{"맑은 고딕":["HCR Batang"]}}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(12, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let spot = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["source"]["character"] == "⦁")
        .unwrap();
    assert_eq!(
        spot["layoutMetric"]["aliasResolvedFace"], "HCR Batang",
        "호출자가 정한 실제 공급자를 자동 사다리로 덮어쓰지 않는다: {spot}"
    );
}
