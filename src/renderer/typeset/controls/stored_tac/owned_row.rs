//! 원본의 독립된 TAC 개체 줄에서 실제 펜과 점유 상자를 조회한다.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    table::Table,
};
use crate::renderer::{
    composer::ComposedParagraph, hwpunit_to_px, style_resolver::ResolvedStyleSet,
};

pub(in crate::renderer::typeset) struct OwnedRow {
    pub horizontal_pen: f64,
}

/// 같은 문단의 앞 개체가 실제로 수용한 원점에서 다음 저장 줄의 원점을 잇는다.
/// 가로 펜만 있는 계획·쪽 재시작·불연속 사다리는 세로 축을 인증하지 않는다.
pub(in crate::renderer::typeset) fn preceding_owned_row_top(
    para: &Paragraph,
    para_index: usize,
    control: usize,
    placements: &std::collections::HashMap<
        (usize, usize),
        crate::renderer::float_placement::InlineBoxPlacement,
    >,
    dpi: f64,
) -> Option<f64> {
    let owner = crate::renderer::composer::control_slot_line_index(para, control)?;
    let line = para.line_segs.get(owner)?;
    placements
        .iter()
        .filter(|((paragraph, previous), placement)| {
            *paragraph == para_index
                && *previous < control
                && placement.stored_horizontal_pen.is_some()
                && placement.advance_end.is_some()
        })
        .filter_map(|((_, previous), placement)| {
            let first = crate::renderer::composer::control_slot_line_index(para, *previous)?;
            let prefix = para.line_segs.get(first..=owner)?;
            if first >= owner
                || prefix.iter().any(|line| {
                    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        || line.line_height <= 0
                        || line.line_spacing < 0
                })
                || prefix.windows(2).any(|pair| {
                    i64::from(pair[0].vertical_pos)
                        + i64::from(pair[0].line_height)
                        + i64::from(pair[0].line_spacing)
                        != i64::from(pair[1].vertical_pos)
                })
            {
                return None;
            }
            let top = placement.y
                + hwpunit_to_px(line.vertical_pos.checked_sub(prefix[0].vertical_pos)?, dpi);
            Some((*previous, top))
        })
        .max_by_key(|(previous, _)| *previous)
        .map(|(_, top)| top)
}

/// 줄을 앞 글자의 논리 위치로 압축하지 않고 제어 슬롯의 UTF-16 소속을 사용한다.
/// 같은 줄의 여러 개체나 수정된 줄은 기존 일반 구성 경로가 처리한다.
pub(in crate::renderer::typeset) fn prepare(
    para: &Paragraph,
    table: &Table,
    control: usize,
    composed: Option<&ComposedParagraph>,
    measured_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<OwnedRow> {
    if !table.common.treat_as_char
        || table.caption.is_some()
        || !para.text.chars().all(char::is_whitespace)
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || table.common.height > i32::MAX as u32
    {
        return None;
    }
    let owner = crate::renderer::composer::control_slot_line_index(para, control)?;
    let line = para.line_segs.get(owner)?;
    let height = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || line.line_spacing < 0
        || i64::from(line.line_height) != height
        || i64::from(line.text_height) != height
        || line.column_start < 0
        || line.segment_width <= 0
        || (measured_height - hwpunit_to_px(table.common.height as i32, dpi)).abs() > dpi / 7200.0
        || para.controls.iter().enumerate().any(|(ci, c)| {
            ci != control
                && matches!(
                    c,
                    Control::Table(_)
                        | Control::Picture(_)
                        | Control::Shape(_)
                        | Control::Equation(_)
                )
                && crate::renderer::composer::control_slot_line_index(para, ci) == Some(owner)
        })
    {
        return None;
    }
    horizontal_pen(para, table, control, composed, styles, dpi)
}

/// 캡션을 포함한 현재 점유 높이와 구성 결과가 일치하는 독립 개체 줄이다.
/// 편집 직후와 저장 후에 같은 줄을 소유하면 같은 펜/끝을 paint에도 전달한다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn prepare_occupied(
    para: &Paragraph,
    table: &Table,
    control: usize,
    composed: Option<&ComposedParagraph>,
    line_heights: &[f64],
    measured_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<OwnedRow> {
    if !table.common.treat_as_char
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || !para.text.chars().all(char::is_whitespace)
    {
        return None;
    }
    let owner = crate::renderer::composer::control_slot_line_index(para, control)?;
    let row = para.line_segs.get(owner)?;
    let occupied = measured_height
        + hwpunit_to_px(
            i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
            dpi,
        );
    if row.line_spacing < 0
        || row.column_start < 0
        || row.segment_width <= 0
        || (hwpunit_to_px(row.line_height, dpi) - occupied).abs() > dpi / 7200.0
        || (line_heights.get(owner)? - occupied).abs() > dpi / 7200.0
        || para.controls.iter().enumerate().any(|(ci, c)| {
            ci != control
                && matches!(
                    c,
                    Control::Table(_)
                        | Control::Picture(_)
                        | Control::Shape(_)
                        | Control::Equation(_)
                )
                && crate::renderer::composer::control_slot_line_index(para, ci) == Some(owner)
        })
    {
        return None;
    }
    horizontal_pen(para, table, control, composed, styles, dpi)
}

fn horizontal_pen(
    para: &Paragraph,
    table: &Table,
    control: usize,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<OwnedRow> {
    let owner = crate::renderer::composer::control_slot_line_index(para, control)?;
    // 첫 개체 앞의 일반 문단 줄은 자기 점유·전진을 소유한다.
    // 표의 가로 pen만 확인한 결과로 그 prefix까지 소비했다고 선언하면
    // before_text가 0이 되어 측정·배치 모두 표를 앞 줄 위에 놓는다.
    // 이미 앞 TAC가 소비한 개체 줄들은 각자의 기존 소유 경로를 유지한다.
    if crate::renderer::composer::ordinary_tac_prefix_rows(para, owner)
        && !para.controls.iter().enumerate().any(|(ci, item)| {
            matches!(item, Control::Table(candidate) if candidate.common.treat_as_char)
                && crate::renderer::composer::control_slot_line_index(para, ci)
                    .is_some_and(|line| line < owner)
        })
    {
        return None;
    }
    let line = para.line_segs.get(owner)?;
    let composed = composed?;
    if composed.lines.len() != para.line_segs.len() {
        return None;
    }
    let position = crate::renderer::composer::find_render_inline_control_positions(para)
        .get(control)
        .copied();
    let leading = crate::renderer::layout::compute_tac_leading_width(
        composed,
        control,
        styles,
        position,
        Some(owner),
    );
    let width = hwpunit_to_px(table.common.width.min(i32::MAX as u32) as i32, dpi)
        + hwpunit_to_px(
            i32::from(table.outer_margin_left) + i32::from(table.outer_margin_right),
            dpi,
        );
    let left = hwpunit_to_px(line.column_start, dpi);
    let available = hwpunit_to_px(line.segment_width, dpi);
    let para_style = styles.para_styles.get(para.para_shape_id as usize)?;
    // 저장 개체 줄도 본문과 같은 줄별 들여쓰기 계약을 소비한다.
    // 첫 줄의 공백 펜을 뒤 줄에 재사용하지 않고 그 줄의 내어쓰기를 복원한다.
    let indent = crate::renderer::equation_tac_flow::paragraph_line_indent_for_source(
        para_style.indent,
        owner,
        Some(para),
        composed.lines.len(),
        true,
        available,
        dpi,
    );
    let left = left + indent;
    let available = available - indent;
    let alignment = para_style.alignment;
    let horizontal_pen = match alignment {
        crate::model::style::Alignment::Center => {
            left + (available - leading - width) / 2.0 + leading
        }
        crate::model::style::Alignment::Right => left + available - width,
        _ => left + leading,
    };
    Some(OwnedRow { horizontal_pen })
}
