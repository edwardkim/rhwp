//! 논리 프레임이 확인된 HYhwpEQ 확장 구분자의 예약 전진폭.
//!
//! 한컴 2024 저장·Print의 CJK/숫자 분수와 중첩 괄호를 800/1200/1600HU로
//! 대조했다. 둥근 괄호는 정상 저장에서 확인한 크기별 전진폭을 사용하고
//! 대괄호는 1/2 em을 논리 격자로 양자화한다. 표시 여백을 중복 가산하지 않는다.

/// 바닥·천장 괄호 한 개의 논리 전진폭(1em).
/// #8927 첨부 한컴 2020 실측: `LFLOOR a RFLOOR`·`LCEIL a RCEIL`의 폭에서 `a`를 뺀
/// 괄호 한 개가 10pt 1050HU·14pt 1350HU로, 두 수식 face의 논리 글자 크기 격자의 1em이다.
/// 한컴 2022 Print에서 괄호 글리프는 이 칸의 가운데에 놓인다.
pub(super) fn floor_ceil(text: &str, fs: f64, dpi: f64) -> Option<f64> {
    matches!(text, "⌊" | "⌋" | "⌈" | "⌉")
        .then(|| super::super::hancom_glyphs::logical_em_advance(fs, dpi))
}

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
