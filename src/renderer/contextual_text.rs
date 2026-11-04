//! 문맥에 따라 이어지는 글자를 출력 경로의 한 shaping 입력으로 전달한다.

use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

fn script(text: &str) -> Option<u32> {
    let ch = text.chars().next()?;
    if !matches!(
        ch.general_category(),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    ) {
        return None;
    }
    let first = ch as u32;
    match first {
        0x0590..=0x05ff | 0xfb1d..=0xfb4f => Some(1),
        0x0600..=0x08ff | 0xfb50..=0xfdff | 0xfe70..=0xfeff => Some(2),
        // 인도계 문자 블록은 각각 독립된 shaping 구간이다.
        0x0900..=0x0dff => Some(first / 128),
        0x0e00..=0x0e7f => Some(3),
        0x1780..=0x17ff => Some(4),
        _ => None,
    }
}

pub(crate) fn coalesce(clusters: Vec<(usize, String)>) -> Vec<(usize, String)> {
    let mut result: Vec<(usize, String)> = Vec::with_capacity(clusters.len());
    let mut previous = None;
    for (index, text) in clusters {
        let current = script(&text);
        // 이모지 ZWJ 시퀀스와 서로 다른 문자 체계는 합치지 않는다.
        // 공백·기호·비문맥 글자는 기존의 저장 원점을 그대로 유지한다.
        let joiner = matches!(text.as_str(), "\u{200c}" | "\u{200d}");
        if previous.is_some() && (current == previous || joiner) {
            result.last_mut().unwrap().1.push_str(&text);
        } else {
            result.push((index, text));
        }
        if !joiner {
            previous = current;
        }
    }
    result
}

pub(crate) fn is_contextual(text: &str) -> bool {
    script(text).is_some()
}
