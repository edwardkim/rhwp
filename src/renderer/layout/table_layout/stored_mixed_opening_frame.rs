//! 선언된 셀의 첫 프레임과 그 안에서 끝난 자식 source 프레임의 공통 점유.

use super::{row_block_cut::CellCutFrame, LayoutEngine, ResolvedStyleSet};
use crate::model::{
    control::Control,
    paragraph::LineSeg,
    table::{Cell, Table},
};
use crate::renderer::{float_placement::ParagraphFloatPlacement, hwpunit_to_px};

#[derive(Clone, Copy)]
pub(in crate::renderer::layout) struct StoredMixedOpeningFrame {
    pub physical_height: f64,
    pub content_height: f64,
}

impl LayoutEngine {
    /// 셀 선언 예산에 정확히 수용되는 source 컷이 자식의 첫 저장 프레임을
    /// 닫을 때, 예약 높이와 정렬 점유를 함께 반환한다. 전체 내용이 훨씬 큰
    /// page wrapper의 선언 높이를 현재 조각의 정렬 상자로 가져오지 않는다.
    pub(in crate::renderer::layout) fn stored_mixed_opening_frame(
        &self,
        parent: &Table,
        cell: &Cell,
        styles: &ResolvedStyleSet,
        start: usize,
        end: usize,
    ) -> Option<StoredMixedOpeningFrame> {
        let profile = self.profile.get();
        if start != 0
            || end == 0
            || cell.height == 0
            || cell.height > i32::MAX as u32
            || cell.row_span != 1
            || cell.text_direction != 0
            || cell.dirty_flag
            || profile.session_edited()
            || !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(parent)
        {
            return None;
        }
        let units = self.cell_units(cell, parent, styles);
        let last = units.get(end.checked_sub(1)?)?;
        let next = units.get(end)?;
        if !last.mixed_nested_fragment
            || !next.mixed_nested_fragment
            || last.para_idx != next.para_idx
            || !next.stored_frame_break_before
        {
            return None;
        }
        let host_index = last.para_idx;
        let first = cell.paragraphs.first()?.line_segs.first()?;
        if first.vertical_pos != 0 {
            return None;
        }
        let mut previous = None;
        for (index, paragraph) in cell.paragraphs.iter().enumerate().take(host_index + 1) {
            if paragraph.stored_text_partition_is_dirty()
                || paragraph.cell_format_vpos_dirty
                || paragraph.line_segs.is_empty()
                || (index < host_index
                    && !crate::renderer::cell_paragraph_has_only_single_column_metadata(paragraph))
            {
                return None;
            }
            for line in &paragraph.line_segs {
                if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || line.vertical_pos < 0
                    || line.line_height <= 0
                    || previous.is_some_and(|origin| line.vertical_pos < origin)
                {
                    return None;
                }
                previous = Some(line.vertical_pos);
            }
        }
        let host = cell.paragraphs.get(host_index)?;
        let [Control::Table(child)] = host.controls.as_slice() else {
            return None;
        };
        let [child_cell] = child.cells.as_slice() else {
            return None;
        };
        if child.common.height == 0
            || child.common.height > i32::MAX as u32
            || child_cell.dirty_flag
            || child_cell.text_direction != 0
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(child)
        {
            return None;
        }
        let split = self.mixed_nested_split_from_cut(cell, parent, styles, 0, end, host_index)?;
        let cut = split.recursive_cut.as_ref()?;
        if cut.start_row != 0 || cut.start_cut.iter().any(|n| *n != 0) || cut.end_cut.len() != 1 {
            return None;
        }
        let child_end = cut.end_cut[0];
        let child_units = self.cell_units(child_cell, child, styles);
        let child_last = child_units.get(child_end.checked_sub(1)?)?;
        let child_next = child_units.get(child_end)?;
        if !child_next.stored_frame_break_before && !child_next.page_frame_reset_before {
            return None;
        }
        let mut child_previous_origin = None;
        if child_units[..child_end].iter().any(|unit| {
            unit.mixed_nested_fragment
                || unit.nested_row.is_some()
                || unit.nested_table_fragment.is_some()
                || child_cell
                    .paragraphs
                    .get(unit.para_idx)
                    .is_none_or(|paragraph| {
                        paragraph.stored_text_partition_is_dirty()
                            || paragraph.cell_format_vpos_dirty
                            || !crate::renderer::cell_paragraph_has_only_single_column_metadata(
                                paragraph,
                            )
                            || paragraph
                                .line_segs
                                .get(unit.vis_start..unit.vis_end)
                                .is_none_or(|lines| {
                                    lines.is_empty()
                                        || lines.iter().any(|line| {
                                            let invalid = line.tag
                                                & LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                                != 0
                                                || line.vertical_pos < 0
                                                || line.line_height <= 0
                                                || child_previous_origin.is_some_and(|origin| {
                                                    line.vertical_pos < origin
                                                });
                                            child_previous_origin = Some(line.vertical_pos);
                                            invalid
                                        })
                                })
                    })
        }) {
            return None;
        }
        if child_units
            .iter()
            .take(child_end)
            .skip(1)
            .any(|unit| unit.stored_frame_break_before)
            || child_cell
                .paragraphs
                .first()?
                .line_segs
                .first()?
                .vertical_pos
                != 0
        {
            return None;
        }
        let last_line = child_cell
            .paragraphs
            .get(child_last.para_idx)?
            .line_segs
            .get(child_last.vis_end.checked_sub(1)?)?;
        let padding = child_cell.effective_padding(&child.padding);
        let closed_height_hu = i64::from(last_line.vertical_pos)
            + i64::from(last_line.line_height)
            + i64::from(padding.top)
            + i64::from(padding.bottom);
        if closed_height_hu != i64::from(child.common.height)
            || last_line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        {
            return None;
        }
        // 선언 예산의 maximal prefix가 같은 컷이어야 한다. 임의 쪽 컷이나
        // 전체 셀 높이를 조각 상자로 오인한 wrapper는 다음 유닛도 fit하므로 기각한다.
        let frame = CellCutFrame::new(self, cell, parent, styles, &units, 0);
        let advance = units[..end].iter().map(|unit| unit.height).sum::<f64>();
        let physical_height = hwpunit_to_px(cell.height as i32, self.dpi);
        if frame.height_without_mixed_opening(end, advance) > physical_height
            || frame.height_without_mixed_opening(end + 1, advance + next.height) <= physical_height
        {
            return None;
        }
        let source_origin = hwpunit_to_px(host.line_segs.first()?.vertical_pos, self.dpi);
        let child_height = hwpunit_to_px(child.common.height as i32, self.dpi);
        let placement = ParagraphFloatPlacement::from_stored_host(
            host,
            child,
            0,
            source_origin,
            child_height,
            self.dpi,
        )?;
        let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, parent, styles);
        let content_height = placement.occupied_bottom;
        (content_height.is_finite()
            && content_height > 0.0
            && content_height + top + bottom <= physical_height)
            .then_some(StoredMixedOpeningFrame {
                physical_height,
                content_height,
            })
    }
}
