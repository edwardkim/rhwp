//! 다음 쪽의 저장 원점으로 닫힌 첫 표 조각과 전체 행 본체를 구분한다.

use crate::model::{
    paragraph::{LineSeg, Paragraph},
    provenance::LayoutCompatibilityProfile,
    shape::VertAlign,
    table::{Table, TablePageBreak},
};
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::{float_placement::is_para_topbottom_float, hwpunit_to_px};

/// 전체 행 합 = 첫 조각 높이 + 다음 쪽 종료 원점인 원본은 행을 비례 축소하지 않는다.
/// HWPX의 문단 지역 vpos나 편집·합성 줄로 물리 쪽 경계를 새로 만들지 않는다.
pub(super) fn stored_following_tail_owns_rows<S: crate::storage::Storage>(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    table: &Table,
    measured: &MeasuredTable,
    profile: LayoutCompatibilityProfile,
    dpi: f64,
    storage: &S,
) -> Result<bool, S::Error> {
    if !profile.hwp5_stored_pagination_layout()
        || profile.session_edited()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !is_para_topbottom_float(&table.common)
        || table.common.size_protect
        || !matches!(table.common.vert_align, VertAlign::Top)
        || table.layout_page_break() != TablePageBreak::RowBreak
        || table.caption.is_some()
        || table.row_count < 2
        || measured.row_heights.len() != usize::from(table.row_count)
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || !super::para_has_non_whitespace_text(para)
        || para.controls.len() != 1
        || para.line_segs.len() != 1
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
    {
        return Ok(false);
    }
    let host = &para.line_segs[0];
    let Some(next) = next_para.filter(|next| {
        next.controls.is_empty()
            && !next.stored_text_partition_is_dirty()
            && !next.cell_format_vpos_dirty
    }) else {
        return Ok(false);
    };
    let Some(tail) = next.line_segs.first() else {
        return Ok(false);
    };
    let source_line = |seg: &LineSeg| {
        seg.vertical_pos >= 0
            && seg.line_height > 0
            && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
    };
    if !source_line(host)
        || !source_line(tail)
        || tail.vertical_pos <= 0
        || tail.vertical_pos >= host.vertical_pos
        || table.cells.is_empty()
        || table.cells.iter().any(|cell| {
            cell.dirty_flag
                || cell.paragraphs.is_empty()
                || cell.paragraphs.iter().any(|p| {
                    p.stored_text_partition_is_dirty()
                        || p.cell_format_vpos_dirty
                        || p.line_segs.is_empty()
                        || p.line_segs.iter().any(|seg| !source_line(seg))
                })
        })
    {
        return Ok(false);
    }
    let rows = table.get_raw_row_heights_with_storage(storage)?;
    if rows.iter().any(|&h| h == 0 || h > i32::MAX as u32) {
        return Ok(false);
    }
    let total = rows.iter().map(|&h| i64::from(h)).sum::<i64>()
        + i64::from(table.cell_spacing) * (rows.len() - 1) as i64;
    if total != i64::from(table.common.height) + i64::from(tail.vertical_pos) {
        return Ok(false);
    }
    let unit = dpi / 7200.0;
    Ok(rows
        .iter()
        .zip(&measured.row_heights)
        .all(|(&raw, &height)| {
            let declared = hwpunit_to_px(raw as i32, dpi);
            height.is_finite()
                && (height - declared).abs()
                    <= unit * 0.5 + (height.abs() + declared.abs()) * f64::EPSILON
        }))
}
