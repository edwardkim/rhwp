//! 구형 파일의 표시 세대가 선택한 HFT 문자와 자연 전진폭.
//!
//! 독립 문자 84개의 한컴 2020 Print(a888172f-962c-4602-9606-7f22815797c9)에서
//! Type3 75자를 확인했다. 별도 literal 108조건 Print에서 세 문자의 같은 d1을 확인했다.
//! 정체 전진은 원본 d1과 같지만 기본 이탤릭의 논리 전진은 Susik Italic이다.
//! 독립 정상 저장의 ASCII 260개·비정수 크기 56개 전진으로 두 소유를 대조했다.
//! 표시 프로그램·shear와 별개로 96DPI 논리 글자 크기와 전진을 각각 반올림한다.
//! 글꼴 프로그램은 호스트가 공급한다. 사설 바이트·윤곽을 이 모듈에 포함하지 않는다.

use super::hancom_glyphs::{GlyphRun, Role};

pub(super) const FONT_FAMILY: &str = "RHWP Hancom Legacy Equation";

const UPPER: [u16; 26] = [
    875, 750, 750, 750, 750, 750, 750, 875, 375, 500, 875, 750, 1000, 875, 750, 750, 750, 750, 625,
    813, 875, 875, 1000, 813, 813, 625,
];
const LOWER: [u16; 26] = [
    500, 563, 500, 563, 500, 375, 625, 563, 313, 313, 625, 313, 875, 563, 500, 563, 563, 375, 500,
    375, 563, 625, 875, 625, 563, 500,
];

// UPM 1,000. 독립 저장의 단일/반복 문자 차이와 같은 논리 face의 메트릭이다.
const ITALIC_UPPER: [u16; 26] = [
    875, 750, 750, 750, 750, 750, 750, 813, 375, 375, 813, 750, 875, 750, 750, 688, 750, 750, 625,
    813, 813, 813, 813, 875, 813, 625,
];
const ITALIC_LOWER: [u16; 26] = [
    563, 500, 500, 563, 500, 375, 563, 563, 313, 313, 500, 313, 875, 625, 500, 500, 500, 500, 375,
    313, 625, 500, 750, 500, 500, 500,
];

fn advance(character: char, role: Role) -> Option<u16> {
    match role {
        Role::Variable { italic: true } if character.is_ascii_uppercase() => {
            Some(ITALIC_UPPER[character as usize - 'A' as usize])
        }
        Role::Variable { italic: true } if character.is_ascii_lowercase() => {
            Some(ITALIC_LOWER[character as usize - 'a' as usize])
        }
        Role::Variable { .. } if character.is_ascii_uppercase() => {
            Some(UPPER[character as usize - 'A' as usize])
        }
        Role::Variable { .. } if character.is_ascii_lowercase() => {
            Some(LOWER[character as usize - 'a' as usize])
        }
        // 독립 원본 Print의 소수점도 숫자와 같은 500/1000em 슬롯이다.
        // 일부 문자 결측 때문에 0.5 같은 숫자 run 전체를 다른 face로 보내지 않는다.
        Role::Number if character.is_ascii_digit() || character == '.' => Some(500),
        Role::LegacyLiteral if matches!(character, '%' | '△' | '․') => Some(1000),
        Role::Operator => match character {
            '+' | '-' | '=' | '⋅' => Some(1000),
            ',' | '.' | '/' | '<' | '>' => Some(500),
            ':' | '(' | ')' => Some(375),
            '[' | ']' => Some(387),
            _ => None,
        },
        _ => None,
    }
}

/// 논리 전진의 누적 원점과 자연 표시 크기를 함께 보존한다. 저장 너비로 글리프를 늘리지 않는다.
pub(super) fn measure(text: &str, role: Role, fs: f64, dpi: f64, pad: f64) -> Option<GlyphRun> {
    if text.is_empty()
        || !fs.is_finite()
        || fs <= 0.0
        || !dpi.is_finite()
        || dpi <= 0.0
        || !pad.is_finite()
        || pad < 0.0
    {
        return None;
    }
    let mut positions = Vec::with_capacity(text.len());
    let mut shears = Vec::with_capacity(text.len());
    let mut width = pad;
    for character in text.chars() {
        let units = advance(character, role)?;
        positions.push(width);
        shears.push(if matches!(role, Role::Variable { italic: true }) {
            // 원본의 대·소문자는 같은 정체 윤곽에 이 행렬을 적용한다.
            0.333
        } else {
            0.0
        });
        let logical_font_pixels = (fs * 96.0 / dpi).round();
        width += (f64::from(units) * logical_font_pixels / 1000.0).round() * dpi / 96.0;
    }
    Some(GlyphRun {
        font_family: FONT_FAMILY,
        glyphs: text.to_owned(),
        positions,
        shears,
        width: width + pad,
        baseline_shift: 0.0,
    })
}

/// 구형 원본 face의 단일 글줄만 인증한다. 분수·중첩 첨자·대체 글리프를
/// 본문 한 줄로 접지 않으며, 행과 자식이 같은 기준선을 소유해야 한다.
pub(super) fn is_line(value: &super::layout::LayoutBox, fs: f64) -> bool {
    if (value.font_size - fs).abs() > 1e-8 {
        return false;
    }
    match &value.kind {
        super::layout::LayoutKind::HancomGlyphs(run) => {
            run.font_family == FONT_FAMILY && !run.glyphs.is_empty()
        }
        super::layout::LayoutKind::Row(items) => {
            !items.is_empty()
                && items.iter().all(|item| {
                    (item.y + item.baseline - value.baseline).abs() < 1e-8 && is_line(item, fs)
                })
        }
        super::layout::LayoutKind::FontStyle { body, .. }
        | super::layout::LayoutKind::Color { body, .. } => {
            (body.y + body.baseline - value.baseline).abs() < 1e-8 && is_line(body, fs)
        }
        _ => false,
    }
}
