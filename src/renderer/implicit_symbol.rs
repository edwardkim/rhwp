//! 표시 Unicode와 구별한 암묵적 기호 프로그램의 전진·잉크 소유.
//!
//! 독립 한컴 Print `pdf/basic/sungeo-hwp-2020.pdf` 5쪽의 Wingdings-Regular
//! (xref 36, glyph 3): UPEM 2048, advance 937, 윤곽 (173,444)..(765,1036).
//! U+F09F의 원은 U+2022를 대체 face로 그린 원과 서로 다른 글리프다.
//! 지정 공식 파일 형식 PDF 70쪽의 Wingdings-Regular glyph 3은 같은 전진·상자의
//! 네모다. U+F0A7을 표시용 U+25AA 글꼴의 폭·잉크로 바꾸지 않는다.

use super::TextStyle;

#[derive(Clone, Copy)]
pub(crate) enum InkShape {
    Ellipse,
    Rectangle,
}

pub(crate) fn advance_em(source: char) -> Option<f64> {
    matches!(source, '\u{f09f}' | '\u{f0a7}').then_some(937.0 / 2048.0)
}

/// 일대일 표시 투영만 원 주소를 연결한다. 옛한글 확장 등은 추측하지 않는다.
pub(crate) fn sources(source: &str, display: &str) -> Option<Vec<char>> {
    if !source.chars().any(|ch| advance_em(ch).is_some())
        || source.chars().count() != display.chars().count()
    {
        return None;
    }
    Some(source.chars().collect())
}

/// 도형과 기준선 원점에 더할 중심·반폭·반높이. 문서·backend·설치 글꼴과 무관하다.
pub(crate) fn ink(
    sources: Option<&[char]>,
    index: usize,
    style: &TextStyle,
) -> Option<(InkShape, f64, f64, f64, f64)> {
    let shape = match sources?.get(index).copied()? {
        '\u{f09f}' => InkShape::Ellipse,
        '\u{f0a7}' => InkShape::Rectangle,
        _ => return None,
    };
    let size = style.script_font_size(style.font_size);
    let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    Some((
        shape,
        size * ratio * (469.0 / 2048.0),
        -size * (740.0 / 2048.0),
        size * ratio * (296.0 / 2048.0),
        size * (296.0 / 2048.0),
    ))
}
