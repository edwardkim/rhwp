//! 논리 프레임이 확인된 HYhwpEQ 확장 구분자의 예약 전진폭.
//!
//! 한컴 2024 저장·Print의 CJK/숫자 분수와 중첩 괄호를 800/1200/1600HU로
//! 대조했다. 둥근 괄호는 정상 저장에서 확인한 크기별 전진폭을 사용하고
//! 대괄호는 1/2 em을 논리 격자로 양자화한다. 표시 여백을 중복 가산하지 않는다.

pub(super) fn source_stretch(left: &str, right: &str, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    let width = |text| {
        let advance = match text {
            "" => return Some(0.0),
            "(" | ")" => return Some(super::source_font_advance::paren(fs, dpi)),
            "[" | "]" => 512,
            _ => return None,
        };
        Some(super::source_font_text::logical_advance(advance, fs, dpi))
    };
    Some((width(left)?, width(right)?))
}
