//! 줄 선두 공백의 소유와 배분 전 자연 전진을 보존한다.

use super::{compute_char_positions, effective_text_for_metrics, ComposedLine, TextStyle};
use crate::renderer::composer::ComposedTextRun;

/// 줄 선두에서 본문 배분을 받지 않는 들여쓰기 공백이다.
pub(super) fn is_leading_indentation_space(ch: char) -> bool {
    matches!(ch, ' ' | '\u{00a0}')
}

/// 표시 확장이 원문의 선두 들여쓰기 공백을 보존했는지 확인한다.
/// PUA 표시 확장과 모델 한 칸을 다른 문자열로 바꾸는 필드를 구분한다.
pub(super) fn source_leading_spaces(run: &ComposedTextRun) -> Option<usize> {
    if run.char_overlap.is_some() || run.footnote_marker.is_some() || run.inserted_control_text {
        return None;
    }
    let source = run
        .text
        .chars()
        .take_while(|&ch| is_leading_indentation_space(ch));
    let displayed = effective_text_for_metrics(run)
        .chars()
        .take_while(|&ch| is_leading_indentation_space(ch));
    source.clone().eq(displayed).then(|| source.count())
}

pub(super) struct LeadingSpaces {
    by_run: Vec<usize>,
}

impl LeadingSpaces {
    pub(super) fn new(line: &ComposedLine, first_object: Option<usize>) -> Self {
        let mut remaining = first_object.map_or(usize::MAX, |position| {
            position.saturating_sub(line.char_start)
        });
        let mut by_run = vec![0; line.runs.len()];
        for (index, run) in line.runs.iter().enumerate() {
            // 겹쳐 쓰기 그룹은 여러 문자를 포갠 가시 개체다. 그 안의 첫 공백을
            // 줄 들여쓰기로 해석하거나 다음 그룹까지 선두 범위를 늘리지 않는다.
            let Some(count) = source_leading_spaces(run) else {
                break;
            };
            let text = effective_text_for_metrics(run);
            let owned = count.min(remaining);
            by_run[index] = owned;
            remaining -= owned;
            if owned < text.chars().count() || remaining == 0 {
                break;
            }
        }
        Self { by_run }
    }

    pub(super) fn in_fragment(&self, run: usize, start: usize, end: usize) -> usize {
        self.by_run[run]
            .saturating_sub(start)
            .min(end.saturating_sub(start))
    }
}

/// 선두 공백은 자연 폭으로, 뒤 텍스트는 배분된 스타일로 같은 전진을 계산한다.
/// 탭 뒤 위치도 새 pen에서 다시 구하므로 전체 뒤쪽 좌표를 일괄 이동하지 않는다.
pub(super) fn natural_leading_positions(
    text: &str,
    style: &TextStyle,
    count: usize,
) -> Option<(Vec<f64>, f64)> {
    if count == 0 || style.extra_word_spacing == 0.0 {
        return None;
    }
    let split = text
        .char_indices()
        .nth(count)
        .map_or(text.len(), |(index, _)| index);
    let (prefix, suffix) = text.split_at(split);
    if prefix.chars().count() != count || !prefix.chars().all(is_leading_indentation_space) {
        return None;
    }
    let original = compute_char_positions(text, style);
    let mut natural = style.clone();
    natural.extra_word_spacing = 0.0;
    let mut positions = compute_char_positions(prefix, &natural);
    let prefix_end = positions.last().copied()?;
    let mut remaining = style.clone();
    remaining.line_x_offset += prefix_end;
    positions.extend(
        compute_char_positions(suffix, &remaining)
            .into_iter()
            .skip(1)
            .map(|position| prefix_end + position),
    );
    let recovered = original.last()? - positions.last()?;
    Some((positions, recovered))
}
