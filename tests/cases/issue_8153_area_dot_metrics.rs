//! #8153: 독립 Print와 원 프로그램의 U+318D 전진 계약.
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
