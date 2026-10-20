//! 옛 호스트 원점과 새 쪽의 뒤 저장 줄이 공유하는 TAC 줄 프레임.

use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};

use super::InlineBoxPlacement;
use crate::renderer::{pagination::PageItem, style_resolver::ResolvedStyleSet};
use std::collections::HashMap;

/// 표의 전체 점유 줄과 후행 간격이 새 로컬 사다리의 첫 줄을 닫는다.
/// 물리 쪽/단의 수용 여부와 원점은 호출자가 별도로 확인한다.
pub(crate) fn restarted_tac_line_advance(
    para: &Paragraph,
    next: &Paragraph,
    spacing_after: f64,
    next_spacing_before: f64,
    dpi: f64,
) -> Option<f64> {
    if !dpi.is_finite()
        || dpi <= 0.0
        || !spacing_after.is_finite()
        || !next_spacing_before.is_finite()
        || para.line_segs.len() != 1
        || para.cell_format_vpos_dirty
        || next.cell_format_vpos_dirty
        || next.stored_text_partition_is_dirty()
        || !next.controls.is_empty()
        || next.raw_break_type != 0
        || next.column_type != crate::model::paragraph::ColumnBreakType::None
    {
        return None;
    }
    let [Control::Table(_)] = para.controls.as_slice() else {
        return None;
    };
    let line = crate::renderer::composer::stored_first_tac_line(para)?;
    let after = next.line_segs.first()?;
    if line.line_spacing < 0
        || after.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || after.vertical_pos < 0
        || after.vertical_pos >= line.vertical_pos
    {
        return None;
    }
    let end = line
        .line_height
        .checked_add(line.line_spacing)?
        .checked_add(px_to_hwpunit(spacing_after, dpi))?;
    (end.checked_add(px_to_hwpunit(next_spacing_before, dpi))? == after.vertical_pos)
        .then(|| hwpunit_to_px(end, dpi))
}

/// 수용된 표의 실제 원점·전진 끝도 같은 프레임일 때만 쪽의 0축을 보존한다.
pub(crate) fn accepted_opening_origin(
    para: &Paragraph,
    next: &Paragraph,
    spacing_after: f64,
    next_spacing_before: f64,
    placement: &InlineBoxPlacement,
    dpi: f64,
) -> Option<i32> {
    let end = restarted_tac_line_advance(para, next, spacing_after, next_spacing_before, dpi)?;
    let tolerance = hwpunit_to_px(1, dpi);
    // StoredTacPlan은 바깥여백을 포함한 줄 원점을 발행한다.
    // 테두리 원점의 위 여백은 paint가 소비하므로 여기서 다시 더하지 않는다.
    (placement.y.abs() <= tolerance && (placement.advance_end? - end).abs() <= tolerance)
        .then_some(0)
}

/// 단을 실제로 여는 표 항목과 확정 배치에서 같은 원본 축을 조회한다.
pub(crate) fn accepted_column_origin(
    items: &[PageItem],
    placements: &HashMap<(usize, usize), InlineBoxPlacement>,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(usize, i32)> {
    let PageItem::Table {
        para_index,
        control_index,
    } = items.first()?
    else {
        return None;
    };
    let para = paragraphs.get(*para_index)?;
    let next = paragraphs.get(para_index.checked_add(1)?)?;
    let after = styles
        .para_styles
        .get(para.para_shape_id as usize)?
        .spacing_after;
    let before = styles
        .para_styles
        .get(next.para_shape_id as usize)?
        .spacing_before;
    let origin = accepted_opening_origin(
        para,
        next,
        after,
        before,
        placements.get(&(*para_index, *control_index))?,
        dpi,
    )?;
    Some((*para_index, origin))
}
