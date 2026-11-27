//! 공통 물리 줄의 종료 이유와 정렬 간격을 확정한다.

use super::{InlineFlowBox, InlineFlowContent};
use crate::model::style::Alignment;

#[derive(Clone, Copy)]
pub(super) enum RowEnding<'a> {
    Soft(&'a [char]),
    Forced(&'a [char]),
    ParagraphEnd {
        chars: &'a [char],
        trailing_spaces: usize,
        mark_font_size: Option<f64>,
    },
}

impl<'a> RowEnding<'a> {
    pub(super) fn trimmed_chars(self) -> Option<&'a [char]> {
        match self {
            Self::Soft(chars) | Self::Forced(chars) => Some(chars),
            Self::ParagraphEnd { .. } => None,
        }
    }

    /// 정렬 폭에서 뺄 줄 끝 공백 상자 수. 자동·작성 줄바꿈은 [`Self::trimmed_chars`]
    /// 가 공백 상자를 모두 걷고, 문단 끝은 호출자가 정한 수만 걷는다.
    pub(super) fn paragraph_end_trimmed_boxes(self) -> usize {
        match self {
            Self::ParagraphEnd {
                trailing_spaces, ..
            } => trailing_spaces,
            Self::Soft(_) | Self::Forced(_) => 0,
        }
    }

    fn chars(self) -> &'a [char] {
        match self {
            Self::Soft(chars) | Self::Forced(chars) | Self::ParagraphEnd { chars, .. } => chars,
        }
    }
}

/// 분배 폭은 글리프 크기를 바꾸지 않고 뒤 상자의 원점을 전진시킨다.
/// 문단 끝·작성 개행과 폭 부족으로 생긴 자동 줄바꿈을 구분한다.
pub(super) fn gaps(
    row: &[InlineFlowBox],
    ending: RowEnding<'_>,
    alignment: Alignment,
    spare: f64,
) -> Vec<f64> {
    match gaps_with_storage(row, ending, alignment, spare, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(super) fn gaps_with_storage<S: crate::storage::Storage>(
    row: &[InlineFlowBox],
    ending: RowEnding<'_>,
    alignment: Alignment,
    spare: f64,
    storage: &S,
) -> Result<Vec<f64>, S::Error> {
    let mut gaps = storage.filled(row.len(), 0.0)?;
    let enabled = match alignment {
        Alignment::Justify => matches!(ending, RowEnding::Soft(_)),
        Alignment::Split => !matches!(ending, RowEnding::Forced(_)),
        Alignment::Distribute => true,
        _ => false,
    };
    if !enabled || !spare.is_finite() || spare <= 0.0 {
        return Ok(gaps);
    }
    let chars = ending.chars();
    let visible = |item: &InlineFlowBox| match &item.content {
        InlineFlowContent::Text { range, .. } => {
            chars[range.clone()].iter().any(|ch| !ch.is_whitespace())
        }
        InlineFlowContent::Table { .. } | InlineFlowContent::Picture { .. } => true,
        InlineFlowContent::FloatingTable { .. } | InlineFlowContent::FloatingShape { .. } => false,
    };
    let Some(first) = row.iter().position(visible) else {
        return Ok(gaps);
    };
    let last = row.iter().rposition(visible).unwrap();
    // 작성 들여쓰기와 말미 공백은 본문 분배 영역에 넣지 않는다.
    let word_slots: Vec<_> = storage.collect((first..last).filter(|&index| {
        matches!(&row[index].content, InlineFlowContent::Text { range, .. }
                if !range.is_empty() && chars[range.clone()].iter().all(|ch| *ch == ' '))
    }))?;
    let slots = if alignment != Alignment::Distribute && !word_slots.is_empty() {
        word_slots
    } else {
        storage.collect((first..last).filter(|&index| visible(&row[index])))?
    };
    if !slots.is_empty() {
        let extra = spare / slots.len() as f64;
        for index in slots {
            gaps[index] = extra;
        }
    }
    Ok(gaps)
}

/// [planet6897/rhwp-issue-docs#497] 가운데 정렬 문단 끝 줄의 말미 공백 상자 수.
///
/// 한/글은 가운데 정렬 줄의 폭에서 마지막 글자·개체 **뒤** 공백을 뺀다. 본문 글줄
/// 경로의 `center_alignment_excludes_trailing_space` ·
/// `trailing_space_width_after_last_inline_object`(#5820·#6173·#7081)와 같은 규칙이다:
/// 줄에 가시 글자나 개체가 있어야 하고(공백만인 줄은 그대로), 밑줄 친 말미 공백은
/// 보이는 내용이라 남긴다. 개체 앞·사이 공백은 말미가 아니므로 뒤에서부터 연속한
/// 공백 상자만 센다. 왼쪽·양쪽·배분 정렬은 문단 끝 공백이 원점을 바꾸지 않거나
/// 별도 배분 계약을 가지므로 대상이 아니다.
///
/// 실측 `18561241_[별표 8]` pi=4(`[TAC 표][공백][공백]`, 가운데 정렬, 저장 줄 없음):
/// 한/글 2022 Print 의 표 왼쪽 괘선 x=100.7~101.1px 는 표+바깥여백만 가운데 둔 값이다.
/// 첫 공백을 폭에 넣으면 97.05px 로 4px 왼쪽에 놓인다.
pub(super) fn paragraph_end_aligned_trailing_spaces(
    row: &[InlineFlowBox],
    chars: &[char],
    alignment: Alignment,
    underlined: impl Fn(u32) -> bool,
) -> usize {
    if alignment != Alignment::Center || !super::separator_spaces::has_flow_content(row, chars) {
        return 0;
    }
    row.iter()
        .rev()
        .take_while(|item| match &item.content {
            InlineFlowContent::Text { range, style, .. } => {
                chars[range.clone()].iter().all(|ch| *ch == ' ') && !underlined(*style)
            }
            _ => false,
        })
        .count()
}
