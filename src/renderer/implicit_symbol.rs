//! 표시 Unicode와 구별한 암묵적 기호 프로그램의 전진·잉크 소유.
//!
//! 독립 한컴 Print `pdf/basic/sungeo-hwp-2020.pdf` 5쪽의 Wingdings-Regular
//! (xref 36, glyph 3): UPEM 2048, advance 937, 윤곽 (173,444)..(765,1036).
//! U+F09F의 원은 U+2022를 대체 face로 그린 원과 서로 다른 글리프다.
//! 지정 공식 파일 형식 PDF 70쪽의 Wingdings-Regular glyph 3은 같은 전진·상자의
//! 네모다. U+F0A7을 표시용 U+25AA 글꼴의 폭·잉크로 바꾸지 않는다.
//!
//! U+F0A1(Wingdings 0xA1)은 굵은 고리다. 독립 한컴 2022 Print
//! `pdf/issue614/issue614-bullet-end-char-shape-2022.pdf` 2쪽 F3(xref 22, glyph 3)과
//! 한컴 2020 Print `pdf/issue8680/wingdings_pua_symbols-hwpx-2020.pdf` 1쪽 F2(xref 14,
//! glyph 7)의 윤곽 좌표가 같다: UPEM 2048, advance 1825, 바깥 (173,0)..(1653,1480),
//! 안쪽 (321,148)..(1505,1332). 표시용 U+25CB(○)는 전각에 가까운 가는 원이라 지름이
//! 더 크고 선이 가늘다. 웹한글기안기 Haan Wings 출력(#8680 이력)과 구별한다.
//! 같은 2020 Print 계열 `pdf/1480000-201900042-chemical-labeling-standards-2020.pdf`
//! 16쪽 Wingdings(xref 44)도 0xA1을 같은 윤곽(glyph 3)으로, U+F06F(0x6F, glyph 4)를
//! 굵은 네모 틀로 싣는다: advance 1825, 바깥 (173,0)..(1653,1480), 안쪽
//! (370,197)..(1456,1283). 표시용 U+25A1(□)의 가는 틀로 바꾸지 않는다.

use super::TextStyle;

#[derive(Clone, Copy)]
pub(crate) enum InkShape {
    Ellipse,
    Rectangle,
    /// 바깥 타원에서 같은 중심의 안쪽 타원(반지름 × `inner`)을 뺀 고리.
    Ring {
        inner: f64,
    },
    /// 바깥 사각형에서 같은 중심의 안쪽 사각형(반폭 × `inner`)을 뺀 틀.
    Frame {
        inner: f64,
    },
}

pub(crate) fn advance_em(source: char) -> Option<f64> {
    match source {
        '\u{f09f}' | '\u{f0a7}' => Some(937.0 / 2048.0),
        '\u{f0a1}' | '\u{f06f}' => Some(1825.0 / 2048.0),
        _ => None,
    }
}

fn has_original_hft_bullet(style: &TextStyle) -> bool {
    style.source_font_is_hft
        && !style.italic
        && style
            .original_glyph_metric_family
            .as_deref()
            .is_some_and(|face| face == "한양신명조")
}

/// 일대일 표시 투영만 원 주소를 연결한다. 옛한글 확장 등은 추측하지 않는다.
pub(crate) fn sources(source: &str, display: &str, style: &TextStyle) -> Option<Vec<char>> {
    if !source
        .chars()
        .any(|ch| advance_em(ch).is_some() || (ch == '•' && has_original_hft_bullet(style)))
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
    let source = sources?.get(index).copied()?;
    let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    if source == '•' && has_original_hft_bullet(style) {
        // 한컴 2020 Print의 화학제품 표시 기준 91쪽 Type3 T11/T18은
        // 같은 1em 원형 잉크를 쓰며, 굵게만 0.02em 외곽선을 더한다.
        // 원점·장평은 문서에서 가져오고 전진폭은 기존 조판 결정에 맡긴다.
        let size = style.script_font_size(style.effective_font_size());
        let bold_radius = if style.bold { 10.0 } else { 0.0 };
        return Some((
            InkShape::Ellipse,
            size * ratio * (501.375 / 1000.0),
            -size * (348.0 / 1000.0),
            size * ratio * ((185.625 + bold_radius) / 1000.0),
            size * ((186.0 + bold_radius) / 1000.0),
        ));
    }
    // (도형, 중심 x, 기준선 위 중심 높이, 반지름) — 원 프로그램의 2048 단위.
    let (shape, center_x, center_y, radius) = match source {
        '\u{f09f}' => (InkShape::Ellipse, 469.0, 740.0, 296.0),
        '\u{f0a7}' => (InkShape::Rectangle, 469.0, 740.0, 296.0),
        // 안쪽 반지름 592 / 바깥 740.
        '\u{f0a1}' => (
            InkShape::Ring {
                inner: 592.0 / 740.0,
            },
            913.0,
            740.0,
            740.0,
        ),
        // 안쪽 반폭 543 / 바깥 740.
        '\u{f06f}' => (
            InkShape::Frame {
                inner: 543.0 / 740.0,
            },
            913.0,
            740.0,
            740.0,
        ),
        _ => return None,
    };
    let size = style.script_font_size(style.font_size);
    Some((
        shape,
        size * ratio * (center_x / 2048.0),
        -size * (center_y / 2048.0),
        size * ratio * (radius / 2048.0),
        size * (radius / 2048.0),
    ))
}
