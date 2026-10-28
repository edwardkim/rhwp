//! 구형 파일의 표시 세대가 선택한 HFT 문자와 자연 전진폭.
//!
//! 독립 문자 84개의 한컴 2020 Print(a888172f-962c-4602-9606-7f22815797c9)에서
//! Type3 75자를 확인했다. 별도 literal 108조건 Print에서 세 문자의 같은 d1을 확인했다.
//! 아래 값은 화면 좌표가 아닌 원본 d1 전진(UPM 1,000)이다.
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

fn advance(character: char, role: Role) -> Option<u16> {
    match role {
        Role::Variable { .. } if character.is_ascii_uppercase() => {
            Some(UPPER[character as usize - 'A' as usize])
        }
        Role::Variable { .. } if character.is_ascii_lowercase() => {
            Some(LOWER[character as usize - 'a' as usize])
        }
        Role::Number if character.is_ascii_digit() => Some(500),
        Role::LegacyLiteral if matches!(character, '%' | '△' | '․') => Some(1000),
        Role::Operator => match character {
            '+' | '-' | '=' => Some(1000),
            ',' | '.' | '/' | '<' | '>' => Some(500),
            ':' | '(' | ')' => Some(375),
            '[' | ']' => Some(387),
            _ => None,
        },
        _ => None,
    }
}

/// 표시 원점은 같은 원본 전진의 누적값이다. 저장 개체 너비로 글리프를 늘리지 않는다.
pub(super) fn measure(text: &str, role: Role, fs: f64, pad: f64) -> Option<GlyphRun> {
    if text.is_empty() || !fs.is_finite() || fs <= 0.0 || !pad.is_finite() || pad < 0.0 {
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
        width += f64::from(units) * fs / 1000.0;
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
