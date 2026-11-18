//! 관계형: 누락 메트릭의 한자 숫자 영을 전각 CJK와 같은 폭으로 측정한다.
//! 주석 갱신: 2026-11-07T03:04:48+09:00 (새 파일).
//! 근거: mydocs/pr/assets/planet6897_20261106/pr8563/ideographic-zero-plan.md.
//! 2026-11-07 측정, 96 DPI·2px raw·Print profile·기존 독립 한컴 2020 Print PDF.
//! 원본 samples/issue8560/21138757-agenda-cover-form.hwp는 전체 1쪽이고 PDF와 같다.
//! R15 Native 전쪽 최저 99.74%, fresh WASM 전쪽 최저 99.75%; 대상 1쪽도 같은 수치다.
//! 정식 TSV·review·overlay는 위 분석 문서에 연결한다. 실제 좌표·고정 영역을 검사하지 않는다.
//! 공개 측정기의 미등록 가족을 사용하므로 설치 OTF/TTF나 특정 프로그램에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    TextStyle,
};

fn style(font_size: f64) -> TextStyle {
    TextStyle {
        font_family: "RHWP missing wide number fixture".into(),
        font_size,
        ratio: 0.9,
        letter_spacing: font_size * 0.08,
        ..Default::default()
    }
}

#[test]
fn ideographic_zero_fallback_shares_the_cjk_fullwidth_contract() {
    let measure = EmbeddedTextMeasurer;
    for font_size in [8.0, 13.5, 31.0] {
        let style = style(font_size);
        let zero = measure.estimate_text_width("〇", &style);
        assert_eq!(zero, measure.estimate_text_width("漢", &style));
        assert_eq!(zero, measure.estimate_text_width("가", &style));
        assert!(zero > measure.estimate_text_width("0", &style));
    }
}

#[test]
fn repeated_ideographic_zero_preserves_the_following_text_boundary() {
    let measure = EmbeddedTextMeasurer;
    for font_size in [8.0, 13.5, 31.0] {
        let style = style(font_size);
        let zero = measure.compute_char_positions("〇〇표기", &style);
        let cjk = measure.compute_char_positions("漢漢표기", &style);
        assert_eq!(zero.len(), 5, "네 문자와 마지막 경계를 보존한다");
        assert_eq!(zero, cjk, "동일 전각 전진 뒤의 본문 원점을 보존한다");
        assert!(zero.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
