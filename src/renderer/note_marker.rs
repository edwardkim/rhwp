//! 본문 참조와 주석 접두가 같은 저장 번호 서식을 사용한다.

use super::{format_number, NumberFormat};
use crate::model::{
    control::{AutoNumber, AutoNumberType, Control},
    paragraph::Paragraph,
};

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
                    if position == 0 && auto.number_type == kind && auto.format <= 8 =>
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
    let (format, prefix, suffix) = if let Some((_, auto)) = leading(paragraphs, kind) {
        (
            NumberFormat::from_hwp_format(auto.format),
            auto.prefix_char,
            auto.suffix_char,
        )
    } else {
        let format = match number_shape as u8 {
            1 => NumberFormat::CircledDigit,
            2 => NumberFormat::RomanUpper,
            3 => NumberFormat::RomanLower,
            4 => NumberFormat::LatinUpper,
            5 => NumberFormat::LatinLower,
            8 => NumberFormat::HangulGaNaDa,
            12 => NumberFormat::HangulNumber,
            13 => NumberFormat::HanjaNumber,
            _ => NumberFormat::Digit,
        };
        (
            format,
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
    text.push_str(&format_number(number, format));
    if suffix != '\0' {
        text.push(suffix);
    }
    text
}
