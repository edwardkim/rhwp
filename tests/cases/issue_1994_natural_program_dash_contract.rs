//! 관계형: 명시적 자연 전진 보존과 기존 leader 추정·결측 경로의 분리를 검사한다.
//! 실물 페이지의 px 핀이 아니며 시스템 글꼴 설치 없이 공통 측정 API를 실행한다.
//! 주석 갱신: 2026-11-07 23:49:45 KST. 관계형이며 설치된 사설 글꼴에 의존하지 않는다.
//! 독립 PDF의 가사 좌표 고정형은 issue_1994_print_lyric_dash_spacing.rs에 분리한다.
//! 주석 갱신: 2026-11-08T19:43:46.094614+09:00. 직전 파일 변경: 2026-11-08T06:19:56+09:00, 7f76cee60e38fceb63feacaff706cf984b19c427.
//! 자연 전진 선택과 윤곽 보존을 분리한 합성 계약이며 설치 글꼴·실물 좌표에 의존하지 않는다.
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
            natural_program_metrics: true,
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
        natural_program_metrics: true,
        ..style.clone()
    };
    assert_eq!(positions("---", &selected), positions("---", &style));
}

#[test]
fn automatic_truetype_keeps_the_leader_until_natural_program_is_selected() {
    let style = TextStyle {
        font_family: "NanumGothic".into(),
        font_size: 20.0,
        source_font_is_ttf: true,
        font_metric_trusted: true,
        ratio: 0.9,
        letter_spacing: 2.0,
        ..TextStyle::default()
    };
    let single = positions("-", &style)[1];
    assert!(
        positions("---", &style)[3] < 3.0 * single,
        "유형 선언과 폭 표 hit는 명시적인 자연 전진 선택을 대신하지 않는다"
    );
    let selected = TextStyle {
        preserve_glyph_ink: true,
        natural_program_metrics: true,
        ..style
    };
    for (index, x) in positions("---", &selected).into_iter().enumerate() {
        assert!((x - single * index as f64).abs() < 1e-9);
    }
}

#[test]
fn automatic_leader_survives_a_serialized_font_type_hint() {
    for source_hft in [false, true] {
        let hft = TextStyle {
            font_family: "HY신명조".into(),
            font_size: 20.0,
            source_font_is_hft: source_hft,
            font_metric_trusted: true,
            ..TextStyle::default()
        };
        let typed = TextStyle {
            source_font_is_ttf: true,
            ..hft.clone()
        };
        assert_eq!(
            positions("-----------", &hft),
            positions("-----------", &typed),
            "직렬화의 유형 표기만으로 원본 HFT leader 전진을 바꾸지 않는다"
        );
        let explicit = TextStyle {
            preserve_glyph_ink: true,
            natural_program_metrics: true,
            ..typed
        };
        assert!(
            positions("-----------", &explicit)[11] > positions("-----------", &hft)[11],
            "명시적으로 선택한 자연 전진은 별개의 계약으로 유지한다"
        );
    }
}
