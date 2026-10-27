//! 잉크를 그리지 않는 빈칸 제어와 글꼴이 표시하는 문자를 구분한다.
//! 전진폭·줄 점유·장식은 측정과 배치가 별도로 소유한다.

/// HWP의 빈칸 제어와 줄 구분은 문자 글리프를 그리지 않는다.
/// 그 밖의 Unicode 공백은 이 속성만으로 빈 glyph라고 단정하지 않는다.
/// 실제 cmap의 빈 glyph는 그려도 잉크가 없고, U+3000처럼 윤곽이 있는 문자는 보존된다.
pub(crate) fn is_nonprinting_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            ' ' | '\u{00a0}' | '\u{2007}' | '\u{202f}' | '\u{2028}' | '\u{2029}'
        )
}

pub(crate) fn is_nonprinting_cluster(text: &str) -> bool {
    text.chars().all(is_nonprinting_character)
}
