//! 자식의 종료 RowCut과 뒤 원본 빈 줄이 닫는 같은 물리 프레임을 확인한다.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::{
    control::Control,
    paragraph::LineSeg,
    shape::{TextWrap, VertAlign, VertRelTo},
    table::{Cell, Table},
};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    /// 개체 host의 placeholder 줄 높이 대신 실제 자식 종료 상자로 뒤 줄 소유를 증명한다.
    /// 같은 자식 RowCut을 예약과 배치가 소비하므로 내용을 다시 추측하거나 합산하지 않는다.
    pub(super) fn closed_nested_frame_owns_following_empty_line(
        &self,
        cell: &Cell,
        table: &Table,
        unit: &CellUnit,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let check = || -> Option<()> {
            if cell.dirty_flag || self.profile.get().session_edited() {
                return None;
            }
            let previous_index = unit.para_idx.checked_sub(1)?;
            let previous = cell.paragraphs.get(previous_index)?;
            let paragraph = cell.paragraphs.get(unit.para_idx)?;
            if unit.vis_start != 0
                || unit.vis_end != 1
                || !paragraph.text.is_empty()
                || !paragraph.controls.is_empty()
                || cell.paragraphs.get(unit.para_idx + 1).is_some()
                || !previous.text.is_empty()
                || previous.line_segs.is_empty()
                || previous.line_segs.iter().any(|line| {
                    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        || line.vertical_pos < 0
                        || line.line_height <= 0
                })
                || [previous, paragraph].iter().any(|para| {
                    para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty
                })
            {
                return None;
            }
            let [line] = paragraph.line_segs.as_slice() else {
                return None;
            };
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.vertical_pos <= 0
                || line.line_height <= 0
                || line.text_height != line.line_height
                || line.baseline_distance <= 0
            {
                return None;
            }
            let mut child = None;
            for control in &previous.controls {
                match control {
                    Control::Table(nested) if child.is_none() => child = Some(nested.as_ref()),
                    Control::ColumnDef(columns) if columns.column_count == 1 => {}
                    _ => return None,
                }
            }
            let child = child?;
            if child.common.treat_as_char
                || child.common.text_wrap != TextWrap::TopAndBottom
                || child.common.vert_rel_to != VertRelTo::Para
                || child.common.vert_align != VertAlign::Top
                || child.common.vertical_offset != 0
                || child.caption.is_some()
            {
                return None;
            }
            let units = self.cell_units(cell, table, styles);
            let end = units.iter().position(|owner| std::ptr::eq(owner, unit))?;
            let start = units[..end].iter().rposition(|owner| {
                owner.para_idx == previous_index && owner.stored_frame_break_before
            })?;
            if !units[start..end].iter().all(|owner| {
                owner.para_idx == previous_index
                    && owner.mixed_nested_fragment
                    && owner.mixed_nested_recursive
                    && !owner.mixed_nested_trailing
            }) {
                return None;
            }
            let cut = self
                .mixed_nested_split_from_cut(cell, table, styles, start, end, previous_index)?
                .recursive_cut?;
            if !cut.end_cut.is_empty() {
                return None;
            }
            let physical = self.nested_cut_physical_height(child, &cut, styles)
                + hwpunit_to_px(
                    i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom),
                    self.dpi,
                );
            ((physical - hwpunit_to_px(line.vertical_pos, self.dpi)).abs()
                <= hwpunit_to_px(2, self.dpi))
            .then_some(())
        };
        check().is_some()
    }
}
