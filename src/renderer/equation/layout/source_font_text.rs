//! HYhwpEQ의 정체·이탤릭 전진폭으로 글리프 원점을 구성한다.
//! 설치 글꼴 SHA-256: a3a3cd992a89ac38e7707b58143ebdb1da9511d3c462f019cda7f92c30467807.
//! 원본 MCP Print의 subset에 포함된 130개 hmtx 값과 설치 글꼴이 일치한다.
//! 사설 글꼴 바이트·윤곽은 포함하지 않는다. 공개 표시 글리프의 모양은 유지한다.

use super::{variable_style, LayoutBox, LayoutKind};

// UPM 1024, 기본 이탤릭 a..z의 cmap U+E0E5..U+E0FE.
const ITALIC_LOWER: [u16; 26] = [
    541, 439, 442, 532, 476, 553, 487, 590, 352, 421, 532, 305, 899, 614, 496, 515, 457, 462, 480,
    369, 585, 496, 732, 585, 502, 476,
];

// 기본 이탤릭 A..Z는 cmap U+E000..U+E019다.
const ITALIC_UPPER: [u16; 26] = [
    768, 725, 739, 782, 696, 667, 803, 768, 369, 526, 795, 640, 938, 768, 795, 696, 795, 753, 568,
    739, 768, 768, 1051, 768, 768, 625,
];
const ROMAN_UPPER: [u16; 26] = [
    768, 682, 682, 725, 682, 640, 725, 768, 384, 469, 768, 640, 938, 768, 725, 640, 725, 682, 640,
    768, 768, 725, 981, 682, 682, 640,
];
const ROMAN_LOWER: [u16; 26] = [
    512, 554, 512, 554, 554, 384, 554, 554, 298, 298, 554, 298, 853, 554, 554, 554, 554, 426, 512,
    384, 554, 554, 810, 597, 597, 469,
];

/// 한컴이 재생성한 6/9/12/18pt 글자·괄호의 논리 전진폭 격자다.
/// HWPUNIT에서 반올림하므로 출력 DPI를 바꿔도 물리 전진폭은 같다.
pub(super) fn logical_advance(units: u16, fs: f64, dpi: f64) -> f64 {
    logical_em_advance(f64::from(units) / 1024.0, fs, dpi)
}

/// 상대 em 예약 폭을 같은 HWPUNIT 논리 격자로 반올림한다.
pub(super) fn logical_em_advance(em: f64, fs: f64, dpi: f64) -> f64 {
    const ADVANCE_QUANTUM_HWP: f64 = 75.0;
    let font_size_hwp = fs * 7200.0 / dpi;
    let advance_hwp = (em * font_size_hwp / ADVANCE_QUANTUM_HWP).round() * ADVANCE_QUANTUM_HWP;
    advance_hwp * dpi / 7200.0
}

pub(super) fn layout(
    text: &str,
    fs: f64,
    dpi: f64,
    italic: bool,
    bold: bool,
    outline_contract: bool,
) -> Option<LayoutBox> {
    if text.is_empty()
        || !text.bytes().all(|ch| ch.is_ascii_alphabetic())
        || (bold && !outline_contract)
    {
        return None;
    }
    // 확인하지 않은 굵은 face 크기는 기존 경로로 남기고 추정 메트릭을 발행하지 않는다.
    let paint = if bold {
        Some(super::source_variable_style::bold_paint(fs, dpi)?)
    } else {
        None
    };
    let mut x = 0.0;
    let mut children = Vec::with_capacity(text.len());
    for ch in text.bytes() {
        let (advances, first) = match (italic, ch.is_ascii_uppercase()) {
            (true, true) => (&ITALIC_UPPER, b'A'),
            (true, false) => (&ITALIC_LOWER, b'a'),
            (false, true) => (&ROMAN_UPPER, b'A'),
            (false, false) => (&ROMAN_LOWER, b'a'),
        };
        let width = if !italic {
            super::source_roman_advance::measured(&char::from(ch).to_string(), fs, dpi)
        } else {
            super::source_italic_advance::measured(ch, fs, dpi).or_else(|| {
                (ch == b'f')
                    .then(|| super::source_font_advance::measured(fs, dpi, 3))
                    .flatten()
            })
        }
        .unwrap_or_else(|| {
            // 미측정 크기도 글자·기호가 공유하는 한컴 논리 em 격자로 전진한다.
            let em = super::source_roman_advance::em_advance(fs, dpi).unwrap_or(fs);
            logical_advance(advances[usize::from(ch - first)], em, dpi)
        }) + if bold {
            super::source_variable_style::bold_extra_advance(dpi)
        } else {
            0.0
        };
        let display = variable_style::display_text(&char::from(ch).to_string(), italic, bold);
        let outline = outline_contract
            .then(|| {
                super::super::variable_outline::commands(char::from(ch), fs, dpi, fs * 0.8, italic)
            })
            .flatten();
        let kind = if let Some(outline) = outline {
            LayoutKind::FontGlyph {
                text: display,
                font_family: "Latin Modern Math, STIX Two Math, serif",
                outline: Some(outline),
                paint,
            }
        } else {
            LayoutKind::Text(display)
        };
        children.push(LayoutBox {
            font_size: fs,
            x,
            y: 0.0,
            width,
            height: fs,
            baseline: fs * 0.8,
            kind,
        });
        x += width;
    }
    Some(LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width: x,
        height: fs,
        baseline: fs * 0.8,
        kind: LayoutKind::Row(children),
    })
}
