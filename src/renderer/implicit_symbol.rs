//! 표시 Unicode와 구별한 암묵적 기호 프로그램의 전진·잉크 소유.
//!
//! 독립 한컴 Print `pdf/basic/sungeo-hwp-2020.pdf` 5쪽의 Wingdings-Regular
//! (xref 36, glyph 3): UPEM 2048, advance 937, 윤곽 (173,444)..(765,1036).
//! U+F09F의 원은 U+2022를 대체 face로 그린 원과 서로 다른 글리프다.

use super::TextStyle;

pub(crate) fn advance_em(source: char) -> Option<f64> {
    (source == '\u{f09f}').then_some(937.0 / 2048.0)
}

/// 일대일 표시 투영만 원 주소를 연결한다. 옛한글 확장 등은 추측하지 않는다.
pub(crate) fn sources(source: &str, display: &str) -> Option<Vec<char>> {
    if !source.contains('\u{f09f}') || source.chars().count() != display.chars().count() {
        return None;
    }
    Some(source.chars().collect())
}

/// 기준선 원점에 더할 중심과 반지름. 문서·backend·설치 글꼴과 무관한 em 계약이다.
pub(crate) fn ink(
    sources: Option<&[char]>,
    index: usize,
    style: &TextStyle,
) -> Option<(f64, f64, f64, f64)> {
    if sources?.get(index).copied()? != '\u{f09f}' {
        return None;
    }
    let size = style.script_font_size(style.font_size);
    let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    Some((
        size * ratio * (469.0 / 2048.0),
        -size * (740.0 / 2048.0),
        size * ratio * (296.0 / 2048.0),
        size * (296.0 / 2048.0),
    ))
}
