//! Reserve the source prefix that the fragment budget can actually own.

use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::LayoutEngine;

impl LayoutEngine {
    fn stored_continuous_span_prefixes(
        &self,
        table: &Table,
        cell: &Cell,
    ) -> Option<Vec<(f64, f64)>> {
        let profile = self.profile.get();
        crate::renderer::float_placement::stored_continuous_span_prefixes_hu(
            table,
            cell,
            profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout(),
            profile.session_edited(),
            self.render_normalization
                .borrow()
                .table_text_reflowed(table),
        )
        .map(|prefixes| {
            prefixes
                .into_iter()
                .map(|(ink, padded)| {
                    (
                        hwpunit_to_px(ink, self.dpi),
                        hwpunit_to_px(padded, self.dpi),
                    )
                })
                .collect()
        })
    }

    pub(crate) fn has_stored_continuous_spans(&self, table: &Table) -> bool {
        table
            .cells
            .iter()
            .any(|cell| self.stored_continuous_span_prefixes(table, cell).is_some())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stored_span_fragment_minimum(
        &self,
        table: &Table,
        start_row: usize,
        end_row: usize,
        row_heights: &[f64],
        fresh_row_budget: f64,
        available_row_budget: Option<f64>,
        minimum_content_height: f64,
    ) -> Option<f64> {
        if start_row >= end_row || end_row > row_heights.len() {
            return None;
        }
        let spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        table
            .cells
            .iter()
            .filter(|cell| {
                let first = usize::from(cell.row);
                first >= start_row
                    && first < end_row
                    && first + usize::from(cell.row_span) > end_row
            })
            .filter_map(|cell| {
                let prefixes = self.stored_continuous_span_prefixes(table, cell)?;
                // Reuse the established orphan threshold in source line space.
                // A short complete owner need not invent another line to reach it.
                let opening = prefixes
                    .iter()
                    .find(|(ink, _)| *ink >= minimum_content_height)
                    .or_else(|| prefixes.last())?;
                if opening.1 > fresh_row_budget {
                    return None;
                }
                let first = usize::from(cell.row);
                let origin = row_heights[start_row..first].iter().sum::<f64>()
                    + spacing * (first - start_row) as f64;
                let height = if let Some(available) = available_row_budget {
                    prefixes
                        .iter()
                        .rev()
                        .find(|(_, padded)| origin + *padded <= available)?
                        .1
                } else {
                    opening.1
                };
                Some(origin + height)
            })
            .reduce(f64::max)
    }
}
