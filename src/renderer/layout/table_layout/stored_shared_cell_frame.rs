//! A short cell restart can share the physical frame of an overlapping span.
//! Original line boxes certify ownership; measured rows supply physical origins.

use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

fn source_line<'a>(cell: &'a Cell, unit: &CellUnit) -> Option<&'a LineSeg> {
    (unit.vis_end == unit.vis_start + 1)
        .then(|| {
            cell.paragraphs
                .get(unit.para_idx)?
                .line_segs
                .get(unit.vis_start)
        })
        .flatten()
}

impl LayoutEngine {
    pub(crate) fn remember_stored_frame_rows(&self, table: &Table, rows: &[f64]) {
        if rows.len() == usize::from(table.row_count)
            && rows
                .iter()
                .all(|height| height.is_finite() && *height >= 0.0)
        {
            self.stored_frame_row_plans
                .borrow_mut()
                .insert(table as *const Table as usize, rows.to_vec());
        }
    }

    /// The first original ladder ends at an adjacent, same-paragraph zero slot.
    /// Empty lines participate in the ladder with their original line advance.
    fn source_opening_cell_cut(
        &self,
        table: &Table,
        cell: &Cell,
        styles: &ResolvedStyleSet,
    ) -> Option<(usize, f64)> {
        if cell.row_span != 1 || !self.stored_plain_text_cell_frame(table, cell) {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let padding = cell.effective_padding(&table.padding);
        let padding = hwpunit_to_px(i32::from(padding.top) + i32::from(padding.bottom), self.dpi);
        let epsilon = hwpunit_to_px(2, self.dpi);
        let mut previous: Option<(&LineSeg, &CellUnit)> = None;
        for (index, unit) in units.iter().enumerate() {
            let line = source_line(cell, unit)?;
            if let Some((before, prior_unit)) = previous {
                if line.vertical_pos == 0 && before.vertical_pos > 0 {
                    if prior_unit.para_idx != unit.para_idx
                        || prior_unit.vis_end != unit.vis_start
                        || !unit.hard_break_before
                    {
                        return None;
                    }
                    let ink = hwpunit_to_px(
                        before.vertical_pos.checked_add(before.line_height)?,
                        self.dpi,
                    ) + padding;
                    return Some((index, ink));
                }
                if i64::from(line.vertical_pos)
                    != i64::from(before.vertical_pos)
                        + i64::from(before.line_height)
                        + i64::from(before.line_spacing)
                {
                    return None;
                }
            } else if line.vertical_pos != 0 {
                return None;
            }
            let advance = hwpunit_to_px(line.line_height.checked_add(line.line_spacing)?, self.dpi);
            if (unit.height - advance).abs() > epsilon {
                return None;
            }
            previous = Some((line, unit));
        }
        None
    }

    fn shared_frame_peer_owner(
        &self,
        table: &Table,
        ordinary: &Cell,
        opening_ink: f64,
        peer: &Cell,
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        let row = usize::from(ordinary.row);
        let first_row = usize::from(peer.row);
        if peer.row_span <= 1 || first_row >= row || row >= first_row + usize::from(peer.row_span) {
            return None;
        }
        let (first_owner, first_height) = self.first_source_frame_owner(table, peer, styles)?;
        let prefix = self.stored_frame_row_plans.borrow();
        let rows = prefix.get(&(table as *const Table as usize))?;
        let candidate = rows.get(first_row..row)?.iter().sum::<f64>()
            + hwpunit_to_px(table.cell_spacing as i32, self.dpi) * (row - first_row) as f64
            + opening_ink;
        drop(prefix);
        let units = self.cell_units(peer, table, styles);
        let padding = peer.effective_padding(&table.padding);
        let padding = hwpunit_to_px(i32::from(padding.top) + i32::from(padding.bottom), self.dpi);
        let epsilon = hwpunit_to_px(2, self.dpi);
        let mut lower = first_height;
        let mut upper = first_height;
        let mut previous: Option<&LineSeg> = None;
        for (index, unit) in units.iter().enumerate().skip(first_owner) {
            let line = source_line(peer, unit)?;
            if let Some(before) = previous {
                if line.vertical_pos == 0 && before.vertical_pos > 0 {
                    // Later intervals have their own page-scale stored witness.
                    // A local restart does not extend the certificate.
                    if !unit.stored_frame_break_before || !unit.page_frame_reset_before {
                        return None;
                    }
                    let ink = hwpunit_to_px(
                        before.vertical_pos.checked_add(before.line_height)?,
                        self.dpi,
                    ) + padding;
                    lower += ink;
                    upper += ink + hwpunit_to_px(before.line_spacing, self.dpi);
                    if candidate + epsilon >= lower && candidate <= upper + epsilon {
                        return Some(index);
                    }
                    if candidate < lower {
                        return None;
                    }
                    previous = None;
                } else if i64::from(line.vertical_pos)
                    != i64::from(before.vertical_pos)
                        + i64::from(before.line_height)
                        + i64::from(before.line_spacing)
                {
                    return None;
                }
            }
            if previous.is_none() && line.vertical_pos != 0 {
                return None;
            }
            let advance = hwpunit_to_px(line.line_height.checked_add(line.line_spacing)?, self.dpi);
            if (unit.height - advance).abs() > epsilon {
                return None;
            }
            previous = Some(line);
        }
        None
    }

    pub(super) fn stored_shared_cell_frame(
        &self,
        table: &Table,
        cell: &Cell,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let Some((owner, ink)) = self.source_opening_cell_cut(table, cell, styles) else {
            return false;
        };
        owner == end
            && table.cells.iter().any(|peer| {
                self.shared_frame_peer_owner(table, cell, ink, peer, styles)
                    .is_some()
            })
    }

    pub(super) fn stored_shared_span_cut_owner(
        &self,
        table: &Table,
        span: &Cell,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        let cells = Self::row_cut_cell_order(table, row);
        if cells.len() != cut.len() || cut.is_empty() {
            return None;
        }
        for (&index, &end) in cells.iter().zip(cut) {
            let cell = table.cells.get(index)?;
            if usize::from(span.row) <= row
                && row < usize::from(span.row) + usize::from(span.row_span)
                && self
                    .first_source_frame_owner(table, cell, styles)
                    .is_some_and(|(owner, _)| owner == end)
            {
                if let Some((owner, _)) = self.first_source_frame_owner(table, span, styles) {
                    return Some(owner);
                }
            }
            let Some((owner, ink)) = self.source_opening_cell_cut(table, cell, styles) else {
                continue;
            };
            if end == owner {
                if let Some(owner) = self.shared_frame_peer_owner(table, cell, ink, span, styles) {
                    return Some(owner);
                }
            }
        }
        None
    }

    pub(super) fn row_cut_has_original_shared_frame(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let cells = Self::row_cut_cell_order(table, row);
        cells.len() == cut.len()
            && cells.iter().zip(cut).any(|(&index, &end)| {
                let cell = &table.cells[index];
                self.first_source_frame_owner(table, cell, styles)
                    .is_some_and(|(owner, _)| owner == end)
                    || self.stored_shared_cell_frame(table, cell, end, styles)
            })
    }

    /// A shared source cut may end inside its final line-spacing band. Select
    /// that bounded physical box while scanning, before it is accepted or drawn.
    pub(crate) fn stored_shared_row_end_frame(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = Self::row_cut_cell_order(table, row);
        if cells.len() != cut.len() || cut.is_empty() || !available.is_finite() {
            return None;
        }
        let mut owners = Vec::new();
        let mut upper = f64::INFINITY;
        for (&index, &end) in cells.iter().zip(cut) {
            let cell = table.cells.get(index)?;
            let units = self.cell_units(cell, table, styles);
            if end < units.len() {
                let (owner, ink) = self.source_opening_cell_cut(table, cell, styles)?;
                if owner != end || !self.stored_shared_cell_frame(table, cell, end, styles) {
                    return None;
                }
                let last = source_line(cell, units.get(end.checked_sub(1)?)?)?;
                upper = upper.min(ink + hwpunit_to_px(last.line_spacing, self.dpi));
                owners.push(cell);
            }
        }
        let physical = available.min(upper);
        let selected = self.row_cut_content_height(table, row, &[], cut, styles);
        if owners.is_empty() || physical < selected {
            return None;
        }
        owners
            .iter()
            .all(|cell| {
                table.cells.iter().any(|peer| {
                    self.shared_frame_peer_owner(table, cell, physical, peer, styles)
                        .is_some()
                })
            })
            .then_some(physical)
    }

    /// A later cut consumes one accepted physical box, leaving the same row's
    /// measured remainder for the next fragment's scanner and paint.
    pub(crate) fn stored_shared_frame_row_remainder(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        measured_rows: &[f64],
        accepted_height: Option<f64>,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = Self::row_cut_cell_order(table, row);
        if cells.len() != cut.len() || cut.is_empty() {
            return None;
        }
        let mut witnessed = false;
        for (&index, &end) in cells.iter().zip(cut) {
            let cell = table.cells.get(index)?;
            if end < self.cell_units(cell, table, styles).len() {
                if !self.stored_shared_cell_frame(table, cell, end, styles) {
                    return None;
                }
                witnessed = true;
            }
        }
        if !witnessed {
            return None;
        }
        let accepted = accepted_height
            .unwrap_or_else(|| self.row_cut_content_height(table, row, &[], cut, styles));
        let remaining = *measured_rows.get(row)? - accepted;
        let content = self.row_cut_content_height(table, row, cut, &[], styles);
        (remaining > content + hwpunit_to_px(2, self.dpi)).then_some(remaining)
    }
}
