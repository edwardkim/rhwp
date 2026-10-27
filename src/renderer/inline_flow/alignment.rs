//! 공통 물리 줄의 종료 이유와 정렬 간격을 확정한다.

use super::{InlineFlowBox, InlineFlowContent};
use crate::model::style::Alignment;

#[derive(Clone, Copy)]
pub(super) enum RowEnding<'a> {
    Soft(&'a [char]),
    Forced(&'a [char]),
    ParagraphEnd(&'a [char]),
}

impl<'a> RowEnding<'a> {
    pub(super) fn trimmed_chars(self) -> Option<&'a [char]> {
        match self {
            Self::Soft(chars) | Self::Forced(chars) => Some(chars),
            Self::ParagraphEnd(_) => None,
        }
    }

    fn chars(self) -> &'a [char] {
        match self {
            Self::Soft(chars) | Self::Forced(chars) | Self::ParagraphEnd(chars) => chars,
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
    let mut gaps = vec![0.0; row.len()];
    let enabled = match alignment {
        Alignment::Justify => matches!(ending, RowEnding::Soft(_)),
        Alignment::Split => !matches!(ending, RowEnding::Forced(_)),
        Alignment::Distribute => true,
        _ => false,
    };
    if !enabled || !spare.is_finite() || spare <= 0.0 {
        return gaps;
    }
    let chars = ending.chars();
    let visible = |item: &InlineFlowBox| match &item.content {
        InlineFlowContent::Text { range, .. } => {
            chars[range.clone()].iter().any(|ch| !ch.is_whitespace())
        }
        InlineFlowContent::Table { .. } | InlineFlowContent::Picture { .. } => true,
        InlineFlowContent::FloatingTable { .. } => false,
    };
    let Some(first) = row.iter().position(visible) else {
        return gaps;
    };
    let last = row.iter().rposition(visible).unwrap();
    // 작성 들여쓰기와 말미 공백은 본문 분배 영역에 넣지 않는다.
    let word_slots: Vec<_> = (first..last)
        .filter(|&index| {
            matches!(&row[index].content, InlineFlowContent::Text { range, .. }
                if !range.is_empty() && chars[range.clone()].iter().all(|ch| *ch == ' '))
        })
        .collect();
    let slots = if alignment != Alignment::Distribute && !word_slots.is_empty() {
        word_slots
    } else {
        (first..last)
            .filter(|&index| visible(&row[index]))
            .collect()
    };
    if !slots.is_empty() {
        let extra = spare / slots.len() as f64;
        for index in slots {
            gaps[index] = extra;
        }
    }
    gaps
}
