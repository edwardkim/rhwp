//! Content ownership of a complete prefix encoded by a block cut vector.

use crate::model::{paragraph::Paragraph, table::Table};
use crate::renderer::style_resolver::ResolvedStyleSet;

use super::super::LayoutEngine;

impl LayoutEngine {
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
