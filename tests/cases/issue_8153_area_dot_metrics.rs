//! 관계형: 독립 Print와 원 프로그램의 U+318D 전진 계약.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 313쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 313쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 편람의 327개씩은 KoPub glyph 전진 900/1000em이다.
//! 실제 페이지의 절대 좌표와 구분하며 글꼴 파일을 배포하지 않는다.
//! 근거: mydocs/plans/issue_8153_area_dot_20261025.md.

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    TextStyle,
};
#[test]
fn measured_area_dot_uses_original_kopub_program_instead_of_fullwidth_fallback() {
    for family in [
        "KoPub바탕체 Light",
        "KoPub바탕체 Medium",
        "KoPub바탕체 Bold",
        "KoPub돋움체 Light",
        "KoPub돋움체 Medium",
        "KoPub돋움체 Bold",
    ] {
        let style = TextStyle {
            font_family: family.into(),
            font_size: 80.0,
            ..Default::default()
        };
        let positions = EmbeddedTextMeasurer.compute_char_positions("ㆍㆍ", &style);
        let expected = 80.0 * 900.0 / 1000.0;
        assert_eq!(positions.len(), 3);
        for span in positions.windows(2) {
            assert!(
                (span[1] - span[0] - expected).abs() <= 2.0 / 75.0,
                "{family}: {:?}, independent hmtx900/1000",
                positions
            );
        }
    }
}
#[test]
fn unknown_program_keeps_the_existing_fullwidth_contract() {
    let style = TextStyle {
        font_family: "명조".into(),
        font_size: 80.0,
        ..Default::default()
    };
    let positions = EmbeddedTextMeasurer.compute_char_positions("ㆍㆍ", &style);
    assert_eq!(positions.len(), 3);
    for span in positions.windows(2) {
        assert!((span[1] - span[0] - 80.0).abs() <= 2.0 / 75.0);
    }
}
