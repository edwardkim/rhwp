//! 관계형: 원본 HFT en dash 전진의 em 비율과 표시 별칭 적용 전후의 보존을 검사한다.
//! 주석 갱신: 2026-11-15 02:55:45 KST. 신규 파일이며 직전 파일 변경 커밋은 없다.
//! 2026-11-15, 96 DPI·compat2022·PDFium·색상 경계 보정 전 2px 실루엣.
//! 원본 issue2007_nested_cell_pagination_42065.hwp 전17쪽: 보정 전 Native 최저98.30%,
//! fresh WASM 최저98.36%; 대상16쪽도 각각98.30%·98.36%다. 절대 px 좌표는 고정하지 않는다.
//! 독립 Print T6/HFT376의1000/1000em 및 실제 원인: mydocs/pr/assets/issue_9060/hft-en-dash-analysis.md.
//! 기존 기호 슬롯 대조군은 issue_8315_symbol_slot_advance에 있으며 궁서 TrueType 폭을 유지한다.
//! 설치 글꼴·외부 글꼴 파일을 요구하지 않는 원본 전진 계약이며 실물 검증은 별도다.

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    TextStyle,
};

fn source_style(size: f64, ratio: f64) -> TextStyle {
    TextStyle {
        font_family: "HY Sinmyeongjo".into(),
        metric_font_family: Some("한양신명조 (HFT)".into()),
        source_font_is_hft: true,
        font_size: size,
        ratio,
        ..Default::default()
    }
}

fn advance(style: &TextStyle, text: &str) -> f64 {
    EmbeddedTextMeasurer.compute_char_positions(text, style)[1]
}

#[test]
fn original_hft_en_dash_uses_its_independent_full_em_advance() {
    for size in [13.333333333333334, 20.0, 40.0] {
        for ratio in [0.8, 1.0, 1.2] {
            let style = source_style(size, ratio);
            let expected = size * ratio;
            // 독립 원본의 1em과 원래 4HU 격자의 반올림 반폭을 구분한다.
            assert!(
                (advance(&style, "–") - expected).abs() <= 2.0 / 75.0 + f64::EPSILON,
                "원본 HFT en dash의 전진은 크기·장평에 비례한다"
            );
        }
    }
}

#[test]
fn display_alias_does_not_replace_the_declared_hft_en_dash_advance() {
    let original = source_style(20.0, 1.0);
    let alias = TextStyle {
        font_family: "표시용 대체 글꼴".into(),
        ..original.clone()
    };
    assert_eq!(advance(&original, "–"), advance(&alias, "–"));
    assert!(
        advance(&original, "–") > advance(&original, "‧"),
        "같은 원본 T6의 전각 en dash와 반각 구분점은 서로 다른 계약이다"
    );
}
