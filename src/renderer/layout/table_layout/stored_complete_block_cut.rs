//! Content ownership of a complete prefix encoded by a block cut vector.

use crate::model::{paragraph::Paragraph, table::Table};
use crate::renderer::style_resolver::ResolvedStyleSet;

use super::super::LayoutEngine;

impl LayoutEngine {
    /// 완전히 선택한 원본 행 prefix의 끝. 물리 높이가 다음 행의 시작을
    /// 넘더라도 다음 행의 유닛을 소비하지 않았으면 그 행을 새로 열지 않는다.
    pub(crate) fn row_block_cut_complete_prefix_end(
        &self,
        table: &Table,
        block: (usize, usize),
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        let mut owners = Self::row_block_cells(table, block.0, block.1);
        owners.sort_by_key(|cell| (cell.row, cell.col));
        if owners.len() != cut.len() {
            return None;
        }
        let row_end = owners
            .iter()
            .zip(cut)
            .filter(|(_, selected)| **selected > 0)
            .map(|(cell, _)| usize::from(cell.row) + 1)
            .max()?;
        self.row_block_cut_is_complete_row_boundary(table, block, row_end, cut, styles)
            .then_some(row_end)
    }

    pub(crate) fn stored_complete_block_opening_frame_height(
        &self,
        table: &Table,
        host: &Paragraph,
        row_end: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if !(self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().hwpx_stored_layout())
            || self.profile.get().session_edited()
            || host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || !self.stored_row_boxes_exceed_first_frame(table)
            || table.common.height == 0
            || table.common.height > i32::MAX as u32
        {
            return None;
        }
        let block =
            super::super::table_partial::rowspan_block_range(table, row_end.checked_sub(1)?);
        self.row_block_cut_owns_complete_prefix(table, block, row_end, cut, styles)
            .then(|| crate::renderer::hwpunit_to_px(table.common.height as i32, self.dpi))
    }

    pub(crate) fn row_block_cut_owns_complete_prefix(
        &self,
        table: &Table,
        block: (usize, usize),
        row_end: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let (first, end) = block;
        if row_end <= first || row_end >= end {
            return false;
        }
        let mut owners = Self::row_block_cells(table, first, end);
        owners.sort_by_key(|cell| (cell.row, cell.col));
        owners.len() == cut.len()
            && owners.iter().zip(cut).all(|(cell, &selected)| {
                if usize::from(cell.row) < row_end {
                    selected == self.cell_units(cell, table, styles).len()
                } else {
                    selected == 0
                }
            })
    }
}
