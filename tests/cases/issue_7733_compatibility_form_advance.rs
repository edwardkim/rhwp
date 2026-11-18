//! 검사 유형: 관계형 — 메트릭 결측 세로 호환기호의 전각과 반각 형식을 구분한다.
//! 주석 갱신: 2026-11-18 00:25 KST.
//! 화학 14쪽 Native 97.82415%, 이전 10270c15 소스 fresh WASM 96.357%에서 추가했다.
//! 실물 PDF의 절대 좌표를 고정하지 않는다. 독립 Print의 Haansoft Batang U+FE40은
//! 1em이고 설치 원본의 같은 글리프 hmtx도 1024/1024다. HY중고딕 원본에는 이 글자가 없다.
//! 원 글꼴의 실제 폭이 없는 경우의 Unicode 전각 계약을 검사한다. 개인 글꼴 설치는 필요 없다.

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::TextStyle;
use unicode_width::UnicodeWidthChar;

fn width(ch: char) -> f64 {
    EmbeddedTextMeasurer.estimate_text_width(
        &ch.to_string(),
        &TextStyle {
            font_family: "RHWP 미등록 호환기호 검사".to_owned(),
            font_size: 20.0,
            ..Default::default()
        },
    )
}

#[test]
fn missing_wide_compatibility_forms_keep_a_full_character_cell() {
    let mut examined = 0;
    for ch in '\u{FE30}'..='\u{FE4F}' {
        if ch.width() == Some(2) {
            examined += 1;
            let actual = width(ch);
            assert!(
                (actual - 20.0).abs() < 1e-6,
                "전각 호환기호 U+{:04X}의 결측이 반각으로 바뀌지 않는다: {actual}",
                ch as u32
            );
        }
    }
    assert!(examined > 10, "실제 전각 호환기호를 검사한다");
}

#[test]
fn genuinely_halfwidth_forms_keep_half_a_character_cell() {
    for ch in ['｡', '｢', '｣', '･'] {
        assert_eq!(ch.width(), Some(1));
        assert!((width(ch) - 10.0).abs() < 1e-6);
    }
}
