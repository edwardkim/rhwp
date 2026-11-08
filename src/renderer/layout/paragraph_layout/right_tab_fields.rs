//! 오른쪽 탭의 동적 필드 상자는 표시값 뒤의 반각 슬롯까지 정렬 폭에 포함한다.
//! 같은 조회를 런 폭 측정과 실제 문자 배치에 적용한다.

use super::*;
use crate::model::control::{AutoNumberType, FieldType};
use std::ops::Range;

pub(super) fn tail_space(
    para: Option<&Paragraph>,
    range: Range<usize>,
    styles: &ResolvedStyleSet,
) -> f64 {
    let Some(para) = para else { return 0.0 };
    let mut positions: Vec<usize> = para
        .text
        .chars()
        .enumerate()
        .filter_map(|(position, character)| {
            (range.contains(&position) && matches!(character, '\u{0015}'..='\u{0017}'))
                .then_some(position)
        })
        .collect();
    for kind in [AutoNumberType::Page, AutoNumberType::TotalPage] {
        positions.extend(
            LayoutEngine::resolve_auto_number_placeholder_positions(para, kind)
                .into_iter()
                .filter(|position| range.contains(position)),
        );
    }
    for field in &para.field_ranges {
        if range.contains(&field.start_char_idx)
            && matches!(para.controls.get(field.control_idx),
                Some(Control::Field(control)) if control.field_type == FieldType::Path)
        {
            positions.push(field.start_char_idx);
        }
    }
    positions.sort_unstable();
    positions.dedup();
    let [position] = positions.as_slice() else {
        return 0.0;
    };
    para.char_shape_id_at(*position)
        .and_then(|id| styles.char_styles.get(id as usize))
        .map_or(0.0, |style| style.font_size * 0.5)
}

pub(super) fn apply_block_width(
    style: &mut TextStyle,
    line: &ComposedLine,
    run_index: usize,
    para: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
) {
    let run = &line.runs[run_index];
    let model_chars: Vec<char> = run.text.chars().collect();
    let Some(tab) = model_chars.iter().rposition(|character| *character == '\t') else {
        return;
    };
    if line
        .runs
        .iter()
        .skip(run_index + 1)
        .any(|run| run.text.contains('\t'))
    {
        return;
    }
    let start = line.char_start
        + line
            .runs
            .iter()
            .take(run_index)
            .map(|run| run.text.chars().count())
            .sum::<usize>();
    let end = start
        + line
            .runs
            .iter()
            .skip(run_index)
            .map(|run| run.text.chars().count())
            .sum::<usize>();
    let padding = tail_space(para, start + tab + 1..end, styles);
    if padding <= 0.0 {
        return;
    }
    let displayed = effective_text_for_metrics(run);
    let Some((before, after)) = displayed.rsplit_once('\t') else {
        return;
    };
    let mut measured = style.clone();
    measured.right_tab_block_width_override = None;
    let before_width = estimate_text_width(before, &measured);
    let (_, tab_type, _) = find_next_tab_stop(
        style.line_x_offset + before_width,
        &style.tab_stops,
        if style.default_tab_width > 0.0 {
            style.default_tab_width
        } else {
            48.0
        },
        style.auto_tab_right,
        style.available_width,
        style.text_start_offset,
    );
    if tab_type != 1 {
        return;
    }
    let width = estimate_text_width(after, &measured)
        + line
            .runs
            .iter()
            .skip(run_index + 1)
            .map(|run| {
                estimate_text_width(effective_text_for_metrics(run), &run.text_style(styles))
            })
            .sum::<f64>();
    style.right_tab_block_width_override = Some(width + padding);
}
