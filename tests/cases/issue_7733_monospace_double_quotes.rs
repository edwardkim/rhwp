//! 검사 유형: 관계형 — 고정폭 글꼴의 곡선 큰따옴표와 전각 글자의 전진을 비교한다.
//! 주석 갱신: 2026-11-17 23:45 KST.
//! 원본 화학 문서 44쪽 Native 95.83356%, 앞선 fresh WASM도 98% 미만인 상태다.
//! 따라서 실물의 고정 px 좌표를 등록하지 않고 원본 글꼴 메트릭의 문자 폭 관계를 검사한다.
//! 기존 독립 2020 Print에서 BatangChe “/” 전진은 같은 줄 한글과 동일한 1em이다.
//! 생성 폭 표도 해당 글자를 1024/1024로 기록한다. 글꼴 설치 없이 공통 측정기를 검사하며,
//! 비례 글꼴의 좁은 따옴표까지 전각으로 바꾸는 보정은 반례로 거부한다.

use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::TextStyle;

fn advance(family: &str, text: &str) -> f64 {
    EmbeddedTextMeasurer.estimate_text_width(
        text,
        &TextStyle {
            font_family: family.to_owned(),
            font_size: 16.0,
            ..Default::default()
        },
    )
}

#[test]
fn untyped_monospace_double_quotes_keep_their_full_character_cell() {
    for family in [
        "바탕체",
        "BatangChe",
        "돋움체",
        "DotumChe",
        "굴림체",
        "GulimChe",
    ] {
        let full = advance(family, "가");
        assert!(full > 0.0, "{family}: 유효한 전각 기준");
        for quote in ["‘", "’", "“", "”"] {
            let actual = advance(family, quote);
            assert!(
                (actual - full).abs() < 1e-6,
                "{family} {quote}: 전각 {full}와 다른 전진 {actual}"
            );
        }
    }
}

#[test]
fn proportional_quotes_are_not_promoted_to_full_character_cells() {
    for quote in ["“", "”"] {
        assert!(advance("Times New Roman", quote) < advance("Times New Roman", "가"));
    }
}
