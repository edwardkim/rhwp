//! [#5678] 언어별 자간 배열이 있으면 그 값이, 범위 밖이면 스칼라 기본이 이긴다.
//!
//! 종전 src 쪽 `per_language_spacing_wins_when_present` 를 공개 타입
//! (`ResolvedCharStyle::letter_spacing_for_lang`) 직접 시험으로 옮긴 판이다.
//! (짝이던 두-경로 동등성 시험은 `resolved_to_text_style` 이 같은 접근자를
//! 호출하도록 단일 출처화해 구조적으로 대체했다.)

use rhwp::renderer::style_resolver::ResolvedCharStyle;

#[test]
fn per_language_spacing_wins_when_present() {
    let cs = ResolvedCharStyle {
        letter_spacing: 0.5,
        letter_spacings: vec![1.25, -0.75],
        ..Default::default()
    };
    assert_eq!(cs.letter_spacing_for_lang(0), 1.25);
    assert_eq!(cs.letter_spacing_for_lang(1), -0.75);
    assert_eq!(cs.letter_spacing_for_lang(2), 0.5, "범위 밖이면 스칼라 기본값");
}
