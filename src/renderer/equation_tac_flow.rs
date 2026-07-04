use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::ComposedParagraph;

#[derive(Debug, Clone)]
pub(crate) struct EquationTacLineFlow {
    tac_rows: Vec<(usize, usize)>,
    pub extra_rows: usize,
    visual_row_base: usize,
}

impl EquationTacLineFlow {
    pub(crate) fn row_for_tac(&self, tac_index: usize) -> Option<usize> {
        self.tac_rows
            .iter()
            .find_map(|(idx, row)| (*idx == tac_index).then_some(*row))
    }

    pub(crate) fn visual_line_idx_for_row(&self, row: usize) -> usize {
        self.visual_row_base + row
    }
}

pub(crate) fn compute_equation_only_tac_line_flow(
    para: Option<&Paragraph>,
    composed: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
    available_width: f64,
    continuation_available_width: f64,
) -> Option<EquationTacLineFlow> {
    let para = para?;
    if tac_offsets_px.is_empty()
        || composed.lines.is_empty()
        || line_idx >= composed.lines.len()
        || available_width.is_nan()
        || available_width <= 0.0
    {
        return None;
    }
    if !composed.lines.iter().all(|line| line.runs.is_empty()) {
        return None;
    }
    if !tac_offsets_px.iter().all(|(_, _, control_index)| {
        matches!(
            para.controls.get(*control_index),
            Some(Control::Equation(_))
        )
    }) {
        return None;
    }

    let wrap_width = if available_width.is_finite() {
        available_width
    } else {
        f64::INFINITY
    };
    let continuation_wrap_width =
        if continuation_available_width.is_finite() && continuation_available_width > 0.0 {
            continuation_available_width
        } else {
            wrap_width
        };

    let assignments = equation_only_tac_line_assignment(para, composed, tac_offsets_px);
    let mut assigned_lines: Vec<usize> = assignments
        .iter()
        .copied()
        .filter(|assigned_line| *assigned_line <= line_idx)
        .collect();
    assigned_lines.sort_unstable();
    assigned_lines.dedup();

    let mut visual_row_base = 0usize;
    for assigned_line in assigned_lines
        .iter()
        .copied()
        .filter(|assigned_line| *assigned_line < line_idx)
    {
        let tacs = tacs_for_assigned_line(&assignments, tac_offsets_px, assigned_line);
        let first_width = if visual_row_base == 0 {
            wrap_width
        } else {
            continuation_wrap_width
        };
        let (_, visual_rows) = pack_equation_tac_rows(tacs, first_width, continuation_wrap_width);
        visual_row_base += visual_rows;
    }

    let current_line_tacs = tacs_for_assigned_line(&assignments, tac_offsets_px, line_idx);
    let first_width = if visual_row_base == 0 {
        wrap_width
    } else {
        continuation_wrap_width
    };
    let (tac_rows, visual_rows) =
        pack_equation_tac_rows(current_line_tacs, first_width, continuation_wrap_width);

    Some(EquationTacLineFlow {
        tac_rows,
        extra_rows: visual_rows.saturating_sub(1),
        visual_row_base,
    })
}

fn tacs_for_assigned_line(
    assignments: &[usize],
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> Vec<(usize, f64)> {
    assignments
        .iter()
        .enumerate()
        .filter_map(|(tac_index, assigned_line)| {
            (*assigned_line == line_idx)
                .then_some((tac_index, tac_offsets_px[tac_index].1.max(0.0)))
        })
        .collect()
}

fn pack_equation_tac_rows(
    tacs: Vec<(usize, f64)>,
    first_wrap_width: f64,
    continuation_wrap_width: f64,
) -> (Vec<(usize, usize)>, usize) {
    if tacs.is_empty() {
        return (Vec::new(), 0);
    }

    let mut row = 0usize;
    let mut row_width = 0.0f64;
    let mut current_wrap_width = first_wrap_width;
    let mut tac_rows = Vec::with_capacity(tacs.len());
    for (tac_index, tac_width) in tacs {
        if row_width > 0.0 && row_width + tac_width > current_wrap_width + 0.5 {
            row += 1;
            row_width = 0.0;
            current_wrap_width = continuation_wrap_width;
        }
        tac_rows.push((tac_index, row));
        row_width += tac_width;
    }

    (tac_rows, row + 1)
}

pub(crate) fn paragraph_line_indent(indent: f64, visual_line_idx: usize) -> f64 {
    paragraph_line_indent_with_scale(indent, visual_line_idx, 1.0)
}

pub(crate) fn paragraph_line_indent_with_scale(
    indent: f64,
    visual_line_idx: usize,
    indent_scale: f64,
) -> f64 {
    let scaled_indent = indent * indent_scale;
    if indent > 0.0 {
        if visual_line_idx == 0 {
            scaled_indent
        } else {
            0.0
        }
    } else if indent < 0.0 {
        if visual_line_idx == 0 {
            0.0
        } else {
            scaled_indent.abs()
        }
    } else {
        0.0
    }
}

pub(crate) fn paragraph_effective_margin_left(
    margin_left: f64,
    indent: f64,
    visual_line_idx: usize,
) -> f64 {
    margin_left + paragraph_line_indent(indent, visual_line_idx)
}

pub(crate) fn paragraph_effective_margin_left_with_indent_scale(
    margin_left: f64,
    indent: f64,
    visual_line_idx: usize,
    indent_scale: f64,
) -> f64 {
    margin_left + paragraph_line_indent_with_scale(indent, visual_line_idx, indent_scale)
}

fn equation_only_tac_line_assignment(
    para: &Paragraph,
    composed: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
) -> Vec<usize> {
    let n_lines = composed.lines.len();
    if n_lines == 0 {
        return Vec::new();
    }

    let degenerate = n_lines > 1
        && composed
            .lines
            .windows(2)
            .any(|window| window[1].char_start <= window[0].char_start);

    if !degenerate {
        return tac_offsets_px
            .iter()
            .map(|(pos, _, _)| {
                (0..n_lines)
                    .find(|&line_idx| {
                        let line_start = composed.lines[line_idx].char_start;
                        let line_end = composed_line_char_end(composed, line_idx);
                        char_pos_in_line(*pos, line_start, line_end)
                    })
                    .unwrap_or(n_lines - 1)
            })
            .collect();
    }

    let mut assignments = vec![0usize; tac_offsets_px.len()];
    let mut tac_idx = 0usize;
    let mut line_start = 0usize;
    while tac_idx < tac_offsets_px.len() {
        let pos = tac_offsets_px[tac_idx].0;
        let group_start = tac_idx;
        while tac_idx < tac_offsets_px.len() && tac_offsets_px[tac_idx].0 == pos {
            tac_idx += 1;
        }
        let group_end = tac_idx;

        let group_len = group_end - group_start;
        let all_line_targets: Vec<usize> = (line_start..n_lines)
            .filter(|&li| composed.lines[li].char_start == pos)
            .collect();
        let filtered_line_targets: Vec<usize> = all_line_targets
            .iter()
            .copied()
            .filter(|&li| {
                !line_is_leading_empty_equation_tac_guide(para, composed, tac_offsets_px, li)
            })
            .collect();
        let mut line_targets = if group_len > 1 && all_line_targets.len() >= group_len {
            // 같은 char_start에 여러 TAC 수식이 있고 저장 LINE_SEG도 같은 수만큼 있으면
            // 선행 빈 guide 줄도 한컴의 물리 수식 줄로 보존한다.
            all_line_targets
        } else {
            filtered_line_targets
        };

        if line_targets.is_empty() {
            let fallback = (line_start..n_lines)
                .find(|&li| {
                    let line_start_char = composed.lines[li].char_start;
                    let line_end_char = composed_line_char_end(composed, li);
                    char_pos_in_line(pos, line_start_char, line_end_char)
                })
                .unwrap_or_else(|| line_start.min(n_lines - 1));
            line_targets.push(fallback);
        }

        for (group_offset, idx) in (group_start..group_end).enumerate() {
            let target = line_targets
                .get(group_offset)
                .copied()
                .unwrap_or_else(|| *line_targets.last().unwrap());
            assignments[idx] = target;
        }

        line_start = line_targets.last().copied().unwrap_or(line_start) + 1;
    }

    assignments
}

fn composed_line_char_end(composed: &ComposedParagraph, line_idx: usize) -> usize {
    composed
        .lines
        .get(line_idx + 1)
        .map(|line| line.char_start)
        .unwrap_or(usize::MAX)
}

fn char_pos_in_line(pos: usize, line_start: usize, line_end: usize) -> bool {
    if line_end == usize::MAX {
        pos >= line_start
    } else if line_end <= line_start {
        pos == line_start
    } else {
        pos >= line_start && pos < line_end
    }
}

fn line_is_leading_empty_equation_tac_guide(
    para: &Paragraph,
    composed: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> bool {
    if line_idx + 1 >= composed.lines.len() {
        return false;
    }
    let line = &composed.lines[line_idx];
    let next = &composed.lines[line_idx + 1];
    if line.char_start != next.char_start {
        return false;
    }
    !line_has_strict_tac_control(composed, tac_offsets_px, line_idx)
        && line_has_strict_equation_tac_control(para, composed, tac_offsets_px, line_idx + 1)
}

fn line_has_strict_tac_control(
    composed: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> bool {
    let line_start = composed.lines[line_idx].char_start;
    let line_end = composed_line_char_end(composed, line_idx);
    tac_offsets_px
        .iter()
        .any(|(pos, _, _)| pos >= &line_start && *pos < line_end)
}

fn line_has_strict_equation_tac_control(
    para: &Paragraph,
    composed: &ComposedParagraph,
    tac_offsets_px: &[(usize, f64, usize)],
    line_idx: usize,
) -> bool {
    let line_start = composed.lines[line_idx].char_start;
    let line_end = composed_line_char_end(composed, line_idx);
    tac_offsets_px.iter().any(|(pos, _, control_index)| {
        pos >= &line_start
            && *pos < line_end
            && matches!(
                para.controls.get(*control_index),
                Some(Control::Equation(_))
            )
    })
}
