//! 본문 참조와 주석 접두가 같은 저장 번호 서식을 사용한다.

use super::{format_number, NumberFormat};
use crate::model::{
    control::{AutoNumber, AutoNumberType, Control},
    paragraph::Paragraph,
};

/// 표 134의 사용자 지정 문자(0x81)는 별도 WCHAR를 번호만큼 반복한다.
/// 일반 번호는 기존 공통 포맷을 유지하며 기호가 없는 경우 숫자로 대체하지 않는다.
pub(crate) fn number_text(number: u16, format: u8, user_symbol: char) -> String {
    if format == 0x81 {
        if user_symbol == '\0' {
            String::new()
        } else {
            user_symbol.to_string().repeat(usize::from(number))
        }
    } else {
        format_number(number, NumberFormat::from_hwp_format(format))
    }
}

pub(crate) fn supported_format(format: u8) -> bool {
    format <= 16 || matches!(format, 0x80 | 0x81)
}

pub(crate) fn leading(
    paragraphs: &[Paragraph],
    kind: AutoNumberType,
) -> Option<(usize, &AutoNumber)> {
    paragraphs.first().and_then(|paragraph| {
        paragraph
            .controls
            .iter()
            .enumerate()
            .zip(paragraph.control_text_positions())
            .find_map(|((index, control), position)| match control {
                Control::AutoNumber(auto)
                    if position == 0
                        && auto.number_type == kind
                        && supported_format(auto.format) =>
                {
                    Some((index, auto))
                }
                _ => None,
            })
    })
}

pub(crate) fn text(
    paragraphs: &[Paragraph],
    kind: AutoNumberType,
    number: u16,
    number_shape: u32,
    before: u16,
    after: u16,
) -> String {
    let (number_text, prefix, suffix) = if let Some((_, auto)) = leading(paragraphs, kind) {
        (
            number_text(number, auto.format, auto.user_symbol),
            auto.prefix_char,
            auto.suffix_char,
        )
    } else {
        (
            number_text(number, number_shape as u8, '\0'),
            char::from_u32(u32::from(before)).unwrap_or('\0'),
            char::from_u32(u32::from(after))
                .filter(|&c| c != '\0')
                .unwrap_or(')'),
        )
    };
    let mut text = String::new();
    if prefix != '\0' {
        text.push(prefix);
    }
    text.push_str(&number_text);
    if suffix != '\0' {
        text.push(suffix);
    }
    text
}
