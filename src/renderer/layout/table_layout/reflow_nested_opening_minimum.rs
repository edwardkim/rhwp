//! 첫 중첩 셀 컷의 내용과 선언 최소의 빈 물리 공간을 함께 예약한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{control::Control, shape::VertAlign, table::Table, table::TablePageBreak};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    /// 내용 컷은 그대로 두고 현재 쪽이 수용한 최소 상자만 발행한다.
    /// 뒤 조각은 같은 자식 커서를 이어받으므로 빈 공간을 글줄로 소비하지 않는다.
    pub(crate) fn reflow_nested_opening_minimum_band(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<super::reflow_nested_band::ReflowNestedBand> {
        if table.common.treat_as_char
            || table.layout_page_break() != TablePageBreak::RowBreak
            || !self.row_uses_reflow_physical_frame(table, row)
            || !available.is_finite()
            || available <= 0.0
        {
            return None;
        }
        let selected_height = self.row_cut_content_height(table, row, start, end, styles);
        let mut owner = None;
        for (slot, index) in self
            .indexed_row_cut_cell_order(table, row)
            .into_iter()
            .enumerate()
        {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            let first = start.get(slot).copied().unwrap_or(0).min(units.len());
            let last = end
                .get(slot)
                .copied()
                .unwrap_or(units.len())
                .min(units.len());
            if first >= last
                || !units[first..last]
                    .iter()
                    .any(|unit| unit.mixed_nested_fragment)
            {
                continue;
            }
            if first != 0 || last >= units.len() || cell.row_span != 1 || owner.is_some() {
                return None;
            }
            let pi = units[first].para_idx;
            if units[first..last]
                .iter()
                .any(|unit| unit.para_idx != pi || !unit.mixed_nested_fragment)
            {
                return None;
            }
            let host = cell.paragraphs.get(pi)?;
            let mut children = host.controls.iter().filter_map(|control| match control {
                Control::Table(child) => Some(child.as_ref()),
                _ => None,
            });
            let child = children.next()?;
            let [child_cell] = child.cells.as_slice() else {
                return None;
            };
            if children.next().is_some()
                || !host.text.trim().is_empty()
                || !crate::renderer::float_placement::is_para_topbottom_float(&child.common)
                || !child.common.flow_with_text
                || child.common.allow_overlap
                || child.common.vert_align != VertAlign::Top
                || child.caption.is_some()
                || child.row_count != 1
                || child.col_count != 1
                || child_cell.row_span != 1
                || child_cell.height == 0
                || child_cell.height > i32::MAX as u32
            {
                return None;
            }
            let split = self.mixed_nested_split_from_cut(cell, table, styles, first, last, pi)?;
            let cut = split.recursive_cut.as_ref()?;
            if split.terminal
                || cut.start_row != 0
                || cut.end_row != 1
                || !cut.start_cut.is_empty()
                || cut.end_cut.is_empty()
                || cut.is_block_split
                || cut.start_cut_is_block
            {
                return None;
            }
            let content = self.nested_cut_physical_height(child, cut, styles);
            let minimum = hwpunit_to_px(child_cell.height as i32, self.dpi);
            if minimum <= content {
                return None;
            }
            let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
            let outer = hwpunit_to_px(
                i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom),
                self.dpi,
            );
            let lead = hwpunit_to_px(
                crate::renderer::float_placement::topbottom_flow_vertical_offset_hu(&child.common),
                self.dpi,
            ) + styles
                .para_styles
                .get(host.para_shape_id as usize)?
                .spacing_before;
            let parent_height = (minimum + lead + top + bottom + outer).min(available);
            if parent_height <= selected_height {
                return None;
            }
            let last_row_height = parent_height - lead - top - bottom - outer;
            if last_row_height < content {
                return None;
            }
            owner = Some(super::reflow_nested_band::ReflowNestedBand {
                parent_height,
                last_row_height,
            });
        }
        owner
    }
}
