//! 실제 수식 상자로 바뀐 줄의 점유 메트릭을 공통 구성 결과에 반영한다.

use super::{ComposedLine, ComposedParagraph, ResolvedStyleSet};
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit, DEFAULT_DPI};

pub(super) fn refresh_equation_rows(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) {
    refresh_rows(composed, para, styles, false);
}

fn refresh_rows(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    regenerated: bool,
) {
    append_missing_single_tac_row(composed, para, styles);
    let positions = para.control_text_positions();
    let dpi = styles.resolved_dpi.unwrap_or(DEFAULT_DPI);
    let tac_offsets_px: Vec<_> = composed
        .tac_controls
        .iter()
        .map(|(position, width, control)| (*position, hwpunit_to_px(*width, dpi), *control))
        .collect();
    // 글자 없는 수식 줄은 컨트롤 순번이 text 길이를 넘을 수 있다.
    // 표시 경로의 실제 줄 소속을 소비하며 text 범위로 다시 추측하지 않는다.
    let equation_owners =
        crate::renderer::equation_tac_flow::uses_equation_only_flow(para, composed).then(|| {
            crate::renderer::equation_tac_flow::equation_only_tac_line_assignment(
                para,
                composed,
                &tac_offsets_px,
            )
            .into_iter()
            .zip(&composed.tac_controls)
            .map(|(line, (_, _, control))| (*control, line))
            .collect::<Vec<_>>()
        });
    let line_count = composed.lines.len();
    for line_index in 0..composed.lines.len() {
        let following_box_height =
            following_reserved_box_height(para, &positions, composed, line_index);
        let stored_pair = para
            .line_segs
            .get(line_index)
            .zip(para.line_segs.get(line_index + 1));
        let end = composed
            .lines
            .get(line_index + 1)
            .map_or(para.text.chars().count(), |line| line.char_start);
        let line = &mut composed.lines[line_index];
        let mut old_ascent = 0_i32;
        let mut old_logical_height = 0_i32;
        let mut new_logical_height = 0_i32;
        let mut new_ascent = 0_i32;
        let mut new_descent = 0_i32;
        let mut changed = false;
        let mut has_equation = false;
        let mut supported = true;
        for (control_index, (control, position)) in para.controls.iter().zip(&positions).enumerate()
        {
            let belongs_to_line = if let Some(owners) = &equation_owners {
                owners.contains(&(control_index, line_index))
            } else {
                *position >= line.char_start
                    && (*position < end || (*position == end && line_index + 1 == line_count))
            };
            if !belongs_to_line {
                continue;
            }
            match control {
                Control::Equation(equation) if equation.common.treat_as_char => {
                    has_equation = true;
                    let old_height = equation.common.height as i32;
                    let old_baseline =
                        old_height.saturating_mul(i32::from(equation.baseline)) / 100;
                    let (_, height, baseline) =
                        crate::renderer::equation::flow_metrics_hwp(equation);
                    let height = height as i32;
                    old_ascent = old_ascent.max(old_baseline);
                    old_logical_height = old_logical_height.max(old_height);
                    new_logical_height = new_logical_height.max(height);
                    new_ascent = new_ascent.max(baseline);
                    new_descent = new_descent.max(height - baseline);
                    // 한글 대체 글꼴 경로도 같은 flow 메트릭을 발행한다.
                    // 특정 정규화 helper의 실행 여부 대신 실제 변경을 소비한다.
                    changed |= height != old_height || baseline != old_baseline;
                }
                Control::Picture(_) | Control::Table(_) | Control::Shape(_) => supported = false,
                _ => {}
            }
        }
        // 수식이 없는 앞 텍스트 줄도 뒤 개체의 저장 예약 상자를 공유할 수 있다.
        // 글자 메트릭을 합치기 전에 반환하면 아래의 실제 소유 검사가 실행되지 않는다.
        // 현재 줄에 수식이 있되 그 메트릭이 유효하지 않은 경우는 계속 보류한다.
        if !supported {
            continue;
        }
        if has_equation
            && new_logical_height <= 0
            && !line.runs.iter().any(|run| !run.text.trim().is_empty())
        {
            // 표시 개체의 0 점유가 유효한 호스트 빈 글줄을 지우지 않는다.
            // 저장 줄의 별도 계약은 보존하고 새로 만든 줄만 원본 글자/문단
            // 속성의 공통 frame 결과로 복원한다. 후속 문단도 이 결과를 소비한다.
            let generated = para.line_segs.get(line_index).is_some_and(|source| {
                source.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            });
            if generated || regenerated || para.line_segs.is_empty() {
                let fs = line
                    .runs
                    .iter()
                    .filter_map(|run| styles.char_styles.get(run.char_style_id as usize))
                    .map(|style| style.font_size)
                    .reduce(f64::max)
                    .or_else(|| {
                        para.char_shapes
                            .first()
                            .and_then(|shape| styles.char_styles.get(shape.char_shape_id as usize))
                            .map(|style| style.font_size)
                    });
                if let Some(fs) = fs.filter(|value| *value > 0.0) {
                    let ps = styles.para_styles.get(para.para_shape_id as usize);
                    let frame = super::frame_metrics_for_line(
                        fs,
                        fs,
                        ps.map_or(crate::model::style::LineSpacingType::Percent, |style| {
                            style.line_spacing_type
                        }),
                        ps.map_or(160.0, |style| style.line_spacing),
                        dpi,
                    );
                    line.line_height = frame.line_height;
                    line.baseline_distance = frame.baseline_distance;
                    line.line_spacing = frame.line_spacing;
                }
            }
            continue;
        }
        // 실제 텍스트가 있는 줄은 텍스트의 어센트·하강부도 같은 합집합에 포함한다.
        // 빈 run의 존재만으로 보이지 않는 텍스트 strut를 새로 추가하지 않는다.
        let text_height = line
            .runs
            .iter()
            .filter(|run| !run.text.trim().is_empty())
            .filter_map(|run| styles.char_styles.get(run.char_style_id as usize))
            .map(|style| px_to_hwpunit(style.font_size, dpi))
            .max()
            .unwrap_or(0);
        let text_ascent = (f64::from(text_height) * 0.85).round() as i32;
        let text_descent = text_height - text_ascent;
        if !has_equation && text_height == 0 {
            continue;
        }
        old_ascent = old_ascent.max(text_ascent);
        old_logical_height = old_logical_height.max(text_height);
        new_logical_height = new_logical_height.max(text_height);
        new_ascent = new_ascent.max(text_ascent);
        new_descent = new_descent.max(text_descent);
        // 같은 저장 예약 구간의 뒤 인라인 개체가 큰 높이를 실제 소유해야 한다.
        // 일반 텍스트 줄이나 저장 컷 앞 줄도 뒤 개체의 점유를 중복하지 않는다.
        let following_inline_box_was_reserved = stored_pair.is_some_and(|(source, next)| {
            use crate::model::paragraph::LineSeg;
            !para.stored_text_partition_is_dirty()
                && line_count == para.line_segs.len()
                && source.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && next.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && source.text_height == old_logical_height
                && source.line_height > old_logical_height
                && following_box_height == Some(source.line_height)
                && (source.baseline_distance - old_ascent).abs() <= 1
                && source.vertical_pos >= 0
                && (source
                    .vertical_pos
                    .checked_add(old_logical_height)
                    .and_then(|end| end.checked_add(source.line_spacing))
                    == Some(next.vertical_pos)
                    || next.vertical_pos < source.vertical_pos)
        });
        let generated_row = para.line_segs.get(line_index).is_some_and(|line| {
            line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        });
        // 저장 줄이 없거나 재생성된 줄은 본문과 수식의 실제 합집합을
        // 소비한다. 본문 유무로 수식의 점유·기준선을 제외하지 않는다.
        let owned_equation_row =
            has_equation && (generated_row || regenerated || para.line_segs.is_empty());
        // 저장 줄이 이 개체 합집합에서 나온 경우만 메트릭을 다시 발행한다.
        // 별도 줄 높이·수직 정렬 계약을 실제 상자의 대용으로 덮어쓰지 않는다.
        if (changed
            && line.line_height == old_logical_height
            && (line.baseline_distance - old_ascent).abs() <= 1)
            || following_inline_box_was_reserved
            || owned_equation_row
        {
            let occupied_height = new_ascent + new_descent;
            // 저장 간격은 논리 상자 높이 뒤의 leading이다. 실제 기준선 합집합이
            // 더 크면 그 차이는 leading에서 소비해 상자/간격을 이중 예약하지 않는다.
            line.line_spacing = line
                .line_spacing
                .saturating_add(new_logical_height.saturating_sub(occupied_height));
            line.line_height = occupied_height;
            line.baseline_distance = new_ascent;
            debug_assert!(hwpunit_to_px(line.line_height, DEFAULT_DPI) > 0.0);
        }
    }
}

/// 같은 높이를 공유하는 저장 예약 구간의 끝을 실제 후속 개체의 소유로 확인한다.
/// 중간 줄의 전진이 내용 높이로 닫히거나 저장 원점으로 돌아갈 때만 구간을 따른다.
fn following_reserved_box_height(
    para: &Paragraph,
    positions: &[usize],
    composed: &ComposedParagraph,
    index: usize,
) -> Option<i32> {
    let source = para.line_segs.get(index)?;
    if source.line_height <= source.text_height || composed.lines.len() != para.line_segs.len() {
        return None;
    }
    for next_index in index + 1..para.line_segs.len() {
        let previous = &para.line_segs[next_index - 1];
        let next = &para.line_segs[next_index];
        if next.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || next.vertical_pos < 0
            || !(previous
                .vertical_pos
                .checked_add(previous.text_height)
                .and_then(|end| end.checked_add(previous.line_spacing))
                == Some(next.vertical_pos)
                || next.vertical_pos < previous.vertical_pos)
        {
            return None;
        }
        if next.text_height == source.line_height {
            let line = composed.lines.get(next_index)?;
            return source_inline_box_height(
                para,
                positions,
                line,
                composed
                    .lines
                    .get(next_index + 1)
                    .map(|line| line.char_start),
            )
            .filter(|height| *height == source.line_height);
        }
        if next.line_height != source.line_height || next.text_height != source.text_height {
            return None;
        }
    }
    None
}

/// 다음 줄의 소유 범위에서 원 저장 인라인 개체 높이를 확인한다.
/// 현재 줄의 캐시 높이를 다음 줄의 실제 개체로 재인용하지 않는다.
fn source_inline_box_height(
    para: &Paragraph,
    positions: &[usize],
    line: &ComposedLine,
    next_start: Option<usize>,
) -> Option<i32> {
    let end = next_start.unwrap_or_else(|| para.text.chars().count());
    para.controls
        .iter()
        .zip(positions)
        .filter(|(_, position)| {
            **position >= line.char_start
                && (**position < end || (next_start.is_none() && **position == end))
        })
        .filter_map(|(control, _)| match control {
            Control::Equation(value) if value.common.treat_as_char => {
                Some(value.common.height as i32)
            }
            Control::Picture(value) if value.common.treat_as_char => {
                Some(value.common.height as i32)
            }
            _ => None,
        })
        .max()
        .filter(|height| *height > 0)
}

/// 줄 캐시가 없는 단일 TAC 수식·양식 개체 호스트의 같은 구성 결과를 세 소비자에 공급한다.
///
/// rhwp가 합성한 줄은 HWPX에 저장하지 않으므로, 양식 개체만 든 문단을 저장했다가 다시 열면
/// 줄이 하나도 없어 개체가 그려지지 않았다. 양식 개체 줄은 재조판처럼 개체 높이를 줄 높이로 쓴다.
fn append_missing_single_tac_row(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) {
    let form_height = match para.controls.as_slice() {
        [Control::Equation(e)] if e.common.treat_as_char => 0,
        [Control::Form(f)] if f.common.treat_as_char => f.height as i32,
        _ => return,
    };
    if !composed.lines.is_empty()
        || !para.line_segs.is_empty()
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && !c.is_control() && c != '\u{fffc}')
    {
        return;
    }
    let fs = para
        .char_shapes
        .first()
        .and_then(|s| styles.char_styles.get(s.char_shape_id as usize))
        .map_or(12.0, |s| s.font_size);
    let ps = styles.para_styles.get(para.para_shape_id as usize);
    let metrics = super::frame_metrics_for_line(
        fs,
        fs,
        ps.map_or(crate::model::style::LineSpacingType::Percent, |s| {
            s.line_spacing_type
        }),
        ps.map_or(160.0, |s| s.line_spacing),
        styles.resolved_dpi.unwrap_or(DEFAULT_DPI),
    );
    composed.lines.push(ComposedLine {
        runs: Vec::new(),
        line_height: metrics.line_height.max(form_height),
        baseline_distance: metrics
            .baseline_distance
            .max(super::line_breaking::baseline_distance_hwp(form_height)),
        segment_width: 0,
        column_start: 0,
        line_spacing: metrics.line_spacing,
        has_line_break: false,
        char_start: 0,
    });
}

/// 실제 재조판 뒤 수식 전용 줄을 composition과 같은 메트릭으로 저장한다.
/// 단일/여러 수식의 실제 줄 소속은 기존 composition이 결정한다.
pub(super) fn publish_regenerated_rows(para: &mut Paragraph, styles: &ResolvedStyleSet) {
    if para.controls.is_empty()
        || !para
            .controls
            .iter()
            .all(|c| matches!(c, Control::Equation(e) if e.common.treat_as_char))
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && !c.is_control() && c != '\u{fffc}')
    {
        return;
    }
    let mut composed = super::compose_paragraph(para);
    refresh_rows(&mut composed, para, styles, true);
    if composed.lines.len() != para.line_segs.len() {
        return;
    }
    let mut rows = para.line_segs.clone();
    let mut top = rows.first().map_or(0, |row| row.vertical_pos);
    for (row, common) in rows.iter_mut().zip(composed.lines) {
        row.vertical_pos = top;
        row.line_height = common.line_height;
        row.text_height = common.line_height;
        row.baseline_distance = common.baseline_distance;
        row.line_spacing = common.line_spacing;
        row.tag |= crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        top = top
            .saturating_add(row.line_height)
            .saturating_add(row.line_spacing);
    }
    para.replace_line_segs(rows);
}
