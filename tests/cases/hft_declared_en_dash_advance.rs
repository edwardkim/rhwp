//! 관계형: 원본 HFT en dash의 일반 전진과 굵게 대체 전진·표시 별칭의 경계를 검사한다.
//! 주석 갱신: 2026-11-15 11:44:02 KST. 직전 변경: 2026-11-15T08:41:33+09:00,
//! 8acc24e3099d506cb5d422d0908cdfd5c042b204. 측정 시각과 Git 변경 시각을 구분한다.
//! 2026-11-15, 96 DPI·compat2022·PDFium·색상 경계 보정 전 2px 실루엣.
//! 원본 issue2007_nested_cell_pagination_42065.hwp 전17쪽: 보정 전 Native 최저98.30%,
//! fresh WASM 최저98.36%; 대상16쪽도 각각98.30%·98.36%다. 절대 px 좌표는 고정하지 않는다.
//! 독립 Print T6/HFT376의1000/1000em 및 실제 원인: mydocs/pr/assets/issue_9060/hft-en-dash-analysis.md.
//! 기존 기호 슬롯 대조군은 issue_8315_symbol_slot_advance에 있으며 궁서 TrueType 폭을 유지한다.
//! 설치 글꼴·외부 글꼴 파일을 요구하지 않는 원본 전진 계약이며 실물 검증은 별도다.
//! 2026-11-15 source833 재측정: nested 전17쪽 Native/WASM 최저98.59%/98.59%,
//! 16쪽99.99%/99.99%. 굵게 반례 CBTA47쪽98.61%/98.54%, 각각 전204쪽 측정 완료.
//! 독립 CBTA Print F4/CID83은500/1000em이며 일반1em을 굵게에 상속하지 않는다.

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

#[test]
fn unmeasured_bold_hft_en_dash_keeps_its_narrower_supplier_advance() {
    for size in [13.333333333333334, 20.0, 40.0] {
        for ratio in [0.8, 1.0, 1.2] {
            let regular = source_style(size, ratio);
            let bold = TextStyle {
                bold: true,
                ..regular.clone()
            };
            let regular_end = advance(&regular, "–");
            let bold_end = advance(&bold, "–");
            // 독립 Print는 일반 HFT의 1em과 굵게 HCI Poppy의 0.5em을 구분한다.
            // 굵기 자간이 포함된 최종 전진도 일반 전각 칸보다 좁아야 한다.
            assert!(
                bold_end > 0.0 && bold_end < regular_end,
                "일반 HFT의 부분 메트릭을 미확인 굵게에 상속하면 뒤 글자가 밀린다: {bold_end} / {regular_end}"
            );
        }
    }
}
