//! 관계형: 명시적 자연 전진 보존과 기존 leader 추정·결측 경로의 분리를 검사한다.
//! 실물 페이지의 px 핀이 아니며 시스템 글꼴 설치 없이 공통 측정 API를 실행한다.
//! 주석 갱신: 2026-11-07 16:22:56 KST.
//! 독립 PDF의 가사 좌표 고정형은 issue_1994_print_lyric_dash_spacing.rs에 분리한다.
use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    TextStyle,
};
fn positions(text: &str, style: &TextStyle) -> Vec<f64> {
    EmbeddedTextMeasurer.compute_char_positions(text, style)
}
#[test]
fn selected_known_program_keeps_single_and_repeated_dash_advances_equal() {
    for family in ["Haansoft Batang", "HY신명조"] {
        let style = TextStyle {
            font_family: family.into(),
            font_size: 20.0,
            preserve_glyph_ink: true,
            font_metric_trusted: true,
            source_font_is_ttf: true,
            ratio: 0.9,
            letter_spacing: 2.0,
            ..TextStyle::default()
        };
        let one = positions("-", &style)[1];
        let repeated = positions("---", &style);
        for (index, x) in repeated.into_iter().enumerate() {
            assert!(
                (x - one * index as f64).abs() < 1e-9,
                "{family}: repeated={x}, source={}",
                one * index as f64
            );
        }
    }
}
#[test]
fn legacy_program_still_uses_the_established_narrow_leader_contract() {
    let style = TextStyle {
        font_family: "HY신명조".into(),
        font_size: 20.0,
        ..TextStyle::default()
    };
    assert!(positions("---", &style)[3] < 3.0 * positions("-", &style)[1]);
}
#[test]
fn unknown_program_does_not_gain_unverified_natural_advances() {
    let style = TextStyle {
        font_family: "missing-private-font-1994".into(),
        font_size: 20.0,
        ..TextStyle::default()
    };
    let selected = TextStyle {
        preserve_glyph_ink: true,
        ..style.clone()
    };
    assert_eq!(positions("---", &selected), positions("---", &style));
}
