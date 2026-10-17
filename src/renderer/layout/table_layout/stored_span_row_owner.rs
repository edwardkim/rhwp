//! Original row boxes can identify a shared source restart independently of
//! extra physical space accepted for a merged cell.

use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::{LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    fn span_source_row_restart(
        &self,
        table: &Table,
        cell: &Cell,
        boundary_row: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        let first_row = usize::from(cell.row);
        if !self.stored_plain_text_span_frame(table, cell)
            || boundary_row <= first_row
            || boundary_row >= first_row + usize::from(cell.row_span)
        {
            return None;
        }
        let rows = table.get_raw_row_heights();
        let source_rows = rows.get(first_row..boundary_row)?;
        if source_rows
            .iter()
            .any(|&height| height == 0 || height > i32::MAX as u32)
        {
            return None;
        }
        // These are original row boxes, not the grown physical ledger. Their
        // prefix is only a witness for a source cut; it never replaces paint's
        // accepted row heights or the fragment's occupied extent.
        let prefix_hu = source_rows.iter().map(|&h| i64::from(h)).sum::<i64>()
            + i64::from(table.cell_spacing) * source_rows.len() as i64;
        let padding = cell.effective_padding(&table.padding);
        let prefix = hwpunit_to_px(i32::try_from(prefix_hu).ok()?, self.dpi)
            - hwpunit_to_px(i32::from(padding.top), self.dpi);
        let cut = self.cell_units_fitting_height(cell, table, styles, prefix);
        let units = self.cell_units(cell, table, styles);
        let next = units.get(cut)?;
        if cut == 0
            || !next.hard_break_before
            || units[..cut].iter().any(|unit| unit.hard_break_before)
        {
            return None;
        }
        let mut previous: Option<&LineSeg> = None;
        for unit in &units[..cut] {
            if unit.empty_spacer || unit.vis_end != unit.vis_start + 1 {
                return None;
            }
            let line = cell
                .paragraphs
                .get(unit.para_idx)?
                .line_segs
                .get(unit.vis_start)?;
            if previous.is_none() && line.vertical_pos != 0 {
                return None;
            }
            if previous.is_some_and(|before| {
                i64::from(line.vertical_pos)
                    != i64::from(before.vertical_pos)
                        + i64::from(before.line_height)
                        + i64::from(before.line_spacing)
            }) {
                return None;
            }
            previous = Some(line);
        }
        let before = previous?;
        let restarted = cell
            .paragraphs
            .get(next.para_idx)?
            .line_segs
            .get(next.vis_start)?;
        // Equal local zeros and a paragraph-local origin are not sufficient.
        // Require actual forward source progress, then a return to zero at the
        // cut independently selected by the original row prefix.
        if before.vertical_pos <= 0 || restarted.vertical_pos != 0 {
            return None;
        }
        Some(cut)
    }

    pub(super) fn stored_span_row_boundary_owner(
        &self,
        table: &Table,
        cell: &Cell,
        boundary_row: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        let cut = self.span_source_row_restart(table, cell, boundary_row, styles)?;
        // A second independent source owner must identify this same physical
        // row boundary. Different span lengths can share one page frame.
        table
            .cells
            .iter()
            .any(|peer| {
                peer.row == cell.row
                    && peer.col != cell.col
                    && self
                        .span_source_row_restart(table, peer, boundary_row, styles)
                        .is_some()
            })
            .then_some(cut)
    }
}
