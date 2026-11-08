//! Plain text output converter.
//!
//! Converts ordered text spans to plain text format.

use crate::error::Result;
use crate::pipeline::{OrderedTextSpan, TextPipelineConfig};
use crate::structure::table_extractor::Table;
use crate::text::HyphenationHandler;

use super::OutputConverter;

/// Plain text output converter.
///
/// Converts ordered text spans to plain text, preserving paragraph structure
/// but removing all formatting.
pub struct PlainTextConverter {
    /// Line spacing threshold ratio for paragraph detection.
    paragraph_gap_ratio: f32,
}

impl PlainTextConverter {
    /// Create a new plain text converter with default settings.
    pub fn new() -> Self {
        Self {
            paragraph_gap_ratio: 1.5,
        }
    }

    /// Detect paragraph breaks between spans based on vertical spacing.
    fn is_paragraph_break(&self, current: &OrderedTextSpan, previous: &OrderedTextSpan) -> bool {
        let line_height = current.span.font_size.max(previous.span.font_size);
        let gap = (previous.span.bbox.y - current.span.bbox.y).abs();
        gap > line_height * self.paragraph_gap_ratio
    }
}

impl Default for PlainTextConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputConverter for PlainTextConverter {
    fn convert(&self, spans: &[OrderedTextSpan], config: &TextPipelineConfig) -> Result<String> {
        self.render_spans(spans, &[], config)
    }

    fn convert_with_tables(
        &self,
        spans: &[OrderedTextSpan],
        tables: &[Table],
        config: &TextPipelineConfig,
    ) -> Result<String> {
        self.render_spans(spans, tables, config)
    }

    fn name(&self) -> &'static str {
        "PlainTextConverter"
    }

    fn mime_type(&self) -> &'static str {
        "text/plain"
    }
}

impl PlainTextConverter {
    /// Render a Table as space-padded plain text.
    ///
    /// Delegates to the table's own `render_text()` method to ensure consistent
    /// formatting across all plain text output paths.
    fn render_table_text(table: &Table) -> String {
        table.render_text()
    }

    /// Whether a run of spans occupies distinct vertical rows — a real table
    /// column has one cell per row. A group whose spans share a Y row (e.g. a
    /// name split into adjacent same-line fragments) is a broken-up line, not a
    /// column; treating it as one makes the row-interleave hoist same-row spans
    /// out of left-to-right order.
    fn run_has_distinct_rows(spans: &[&OrderedTextSpan]) -> bool {
        let mut ys: Vec<f32> = spans.iter().map(|s| s.span.bbox.y).collect();
        ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        let tol = spans
            .iter()
            .map(|s| s.span.font_size)
            .fold(0.0_f32, f32::max)
            .max(1.0)
            * 0.5;
        ys.windows(2).all(|w| (w[1] - w[0]).abs() >= tol)
    }

    /// Detect consecutive groups that form a columnar layout and reorder
    /// their spans row-by-row instead of column-by-column.
    ///
    /// Columnar groups are detected when:
    /// 1. Two or more consecutive groups (by reading order) each have the same
    ///    number of spans.
    /// 2. The groups are side-by-side horizontally (distinct X ranges).
    /// 3. Each group's spans align vertically (similar Y positions across groups).
    ///
    /// When detected, the spans are re-interleaved so that row 1 of group A is
    /// followed by row 1 of group B, then row 2 of group A followed by row 2 of
    /// group B, etc.  This produces the expected table-like reading order.
    /// Returns the set of indices in `sorted` that belong to reordered columnar
    /// groups.  The rendering loop uses this to suppress group-change paragraph
    /// breaks for those spans.
    fn reorder_columnar_groups<'a>(
        sorted: &mut Vec<&'a OrderedTextSpan>,
    ) -> std::collections::HashSet<usize> {
        let mut columnar_indices = std::collections::HashSet::new();
        // Collect runs of spans sharing the same group_id, in reading order.
        if sorted.is_empty() {
            return columnar_indices;
        }

        // Build list of (group_id, start_index, count) for contiguous runs.
        let mut runs: Vec<(usize, usize, usize)> = Vec::new();
        let mut run_start = 0;
        let mut current_group: Option<usize> = sorted[0].group_id;
        for (i, span) in sorted.iter().enumerate().skip(1) {
            let g = span.group_id;
            if g != current_group {
                if let Some(gid) = current_group {
                    runs.push((gid, run_start, i - run_start));
                }
                current_group = g;
                run_start = i;
            }
        }
        if let Some(gid) = current_group {
            runs.push((gid, run_start, sorted.len() - run_start));
        }

        if runs.len() < 2 {
            return columnar_indices;
        }

        // Find sequences of consecutive runs that form columns.
        // Criteria: same span count, horizontally separated, Y-aligned rows.
        let mut i = 0;
        // Collect rewrite instructions: (start_index_in_sorted, total_span_count, num_columns, rows_per_col)
        let mut rewrites: Vec<(usize, usize, usize, usize)> = Vec::new();

        while i < runs.len() {
            let base_count = runs[i].2;
            // Need at least 2 rows per column (header + data) and at least 2
            // columns. A column must also occupy distinct Y rows (one cell per
            // row); a group whose spans share a row is a split line, not a column.
            if base_count < 2
                || !Self::run_has_distinct_rows(&sorted[runs[i].1..runs[i].1 + runs[i].2])
            {
                i += 1;
                continue;
            }

            // Try to extend the columnar sequence as far as possible.
            // Allow groups with similar span counts (within ±50% and at least 2)
            // to handle cases where one column has extra header/footer spans.
            let mut j = i + 1;
            let min_match = (base_count / 2).max(2);
            let max_match = base_count * 2;
            while j < runs.len() && runs[j].2 >= min_match && runs[j].2 <= max_match {
                // Check that groups are horizontally separated.
                let prev_spans = &sorted[runs[j - 1].1..runs[j - 1].1 + runs[j - 1].2];
                let curr_spans = &sorted[runs[j].1..runs[j].1 + runs[j].2];

                // A candidate column must occupy distinct Y rows (one cell per
                // row). A group with two spans on the same row is a split line,
                // not a column — interleaving it reorders same-row spans.
                if !Self::run_has_distinct_rows(curr_spans) {
                    break;
                }

                // Compute X range for each group.
                let prev_min_x = prev_spans
                    .iter()
                    .map(|s| s.span.bbox.x)
                    .fold(f32::INFINITY, f32::min);
                let prev_max_x = prev_spans
                    .iter()
                    .map(|s| s.span.bbox.x + s.span.bbox.width)
                    .fold(f32::NEG_INFINITY, f32::max);
                let curr_min_x = curr_spans
                    .iter()
                    .map(|s| s.span.bbox.x)
                    .fold(f32::INFINITY, f32::min);
                let curr_max_x = curr_spans
                    .iter()
                    .map(|s| s.span.bbox.x + s.span.bbox.width)
                    .fold(f32::NEG_INFINITY, f32::max);

                // Groups must not substantially overlap in X.
                let overlap_x = prev_max_x.min(curr_max_x) - prev_min_x.max(curr_min_x);
                let min_width = (prev_max_x - prev_min_x)
                    .min(curr_max_x - curr_min_x)
                    .max(1.0);
                if overlap_x > min_width * 0.3 {
                    break;
                }

                // Check Y-alignment: sort each group's spans by Y (descending,
                // PDF coords) and verify rows match within tolerance.
                let mut prev_ys: Vec<f32> = prev_spans.iter().map(|s| s.span.bbox.y).collect();
                let mut curr_ys: Vec<f32> = curr_spans.iter().map(|s| s.span.bbox.y).collect();
                prev_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a));
                curr_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a));

                let font_size = prev_spans.first().map(|s| s.span.font_size).unwrap_or(12.0);
                let y_tolerance = font_size * 1.5;
                let aligned = prev_ys
                    .iter()
                    .zip(curr_ys.iter())
                    .all(|(py, cy)| (py - cy).abs() < y_tolerance);

                if !aligned {
                    break;
                }

                j += 1;
            }

            let num_columns = j - i;
            if num_columns >= 2 {
                let start_idx = runs[i].1;
                // Use the minimum span count across all groups for row count
                let min_rows = (i..j).map(|k| runs[k].2).min().unwrap_or(base_count);
                let total_count: usize = (i..j).map(|k| runs[k].2).sum();
                rewrites.push((start_idx, total_count, num_columns, min_rows));
                i = j;
            } else {
                i += 1;
            }
        }

        // Apply rewrites in reverse order to preserve indices.
        // We record the per-column span counts during detection so that
        // columns with different run lengths are sliced correctly (previously
        // the code assumed all columns had exactly `rows_per_col` spans, which
        // dropped trailing spans from longer columns).
        for &(start_idx, total_count, num_columns, rows_per_col) in rewrites.iter().rev() {
            // Rebuild per-column slices using the original run lengths recorded
            // in `runs`.  We need to locate the runs that participate in this
            // rewrite by matching their start index.
            let run_start = runs
                .iter()
                .position(|(_, s, _)| *s == start_idx)
                .expect("rewrite start_idx must correspond to a recorded run");
            let run_end = run_start + num_columns;

            // Split into columns using the exact run slices, each sorted by
            // Y descending (top-to-bottom in PDF coordinates).
            let mut columns: Vec<Vec<&'a OrderedTextSpan>> = Vec::with_capacity(num_columns);
            let mut cursor = start_idx;
            for run_idx in run_start..run_end {
                let run_len = runs[run_idx].2;
                let mut col: Vec<&'a OrderedTextSpan> = sorted[cursor..cursor + run_len].to_vec();
                col.sort_by(|a, b| crate::utils::safe_float_cmp(b.span.bbox.y, a.span.bbox.y));
                columns.push(col);
                cursor += run_len;
            }

            // Interleave: for each row in 0..rows_per_col pull one span from
            // every column, then append any remaining overflow spans from
            // longer columns in stable (top-to-bottom) order. This guarantees
            // we reinsert exactly `total_count` spans — no data loss.
            let mut interleaved: Vec<&'a OrderedTextSpan> = Vec::with_capacity(total_count);
            for row in 0..rows_per_col {
                for col in &columns {
                    interleaved.push(col[row]);
                }
            }
            for col in &columns {
                for span in col.iter().skip(rows_per_col) {
                    interleaved.push(span);
                }
            }
            debug_assert_eq!(
                interleaved.len(),
                total_count,
                "interleaved span count must match input"
            );

            // Replace the columnar section in sorted with the interleaved order.
            sorted.splice(start_idx..start_idx + total_count, interleaved);

            // Mark all indices in this rewritten range.
            for idx in start_idx..start_idx + total_count {
                columnar_indices.insert(idx);
            }
        }

        columnar_indices
    }

    /// Re-sort spans so that spans sharing the same Y-position (within a
    /// font-size-based tolerance) are adjacent and ordered by X, regardless
    /// of which reading-order group they came from.
    ///
    /// This preserves the overall top-to-bottom reading order while ensuring
    /// that a label and its value at the same vertical position (but in
    /// different groups) end up on the same output line.
    ///
    /// Spans already identified as part of reordered columnar layouts are
    /// excluded via `columnar_indices`. No additional pairwise group-level
    /// column-detection check is performed here; remaining spans are merged
    /// by Y-cluster and then ordered by X within each cluster.
    fn merge_same_y_across_groups(
        sorted: &mut Vec<&OrderedTextSpan>,
        columnar_indices: &std::collections::HashSet<usize>,
    ) {
        if sorted.len() < 2 {
            return;
        }

        // Spans already handled by reorder_columnar_groups() are excluded via
        // the `columnar_indices` set — no additional group-pair check is
        // performed (an earlier version over-excluded label-value pairs).

        // Build Y-clusters: group spans whose Y-positions are within tolerance.
        // We process spans in their current order (reading order) to assign
        // each span to the first matching cluster (or create a new one).
        // Clusters are kept in top-to-bottom order (descending Y in PDF coords).
        let mut clusters: Vec<(f32, Vec<usize>)> = Vec::new(); // (representative_y, span_indices)

        for idx in 0..sorted.len() {
            if columnar_indices.contains(&idx) {
                continue; // columnar spans keep their original order
            }
            let y = sorted[idx].span.bbox.y;
            let tol = sorted[idx].span.font_size.max(1.0) * 0.5;

            let mut matched = false;
            for cluster in clusters.iter_mut() {
                if (cluster.0 - y).abs() < tol {
                    cluster.1.push(idx);
                    matched = true;
                    break;
                }
            }
            if !matched {
                clusters.push((y, vec![idx]));
            }
        }

        // Sort clusters top-to-bottom (highest Y first in PDF coords).
        clusters.sort_by(|a, b| crate::utils::safe_float_cmp(b.0, a.0));

        // Within each cluster, sort spans by X-position (left to right).
        for cluster in &mut clusters {
            cluster.1.sort_by(|&a, &b| {
                crate::utils::safe_float_cmp(sorted[a].span.bbox.x, sorted[b].span.bbox.x)
            });
        }

        // Rebuild the sorted vec: non-columnar spans in Y-cluster order,
        // columnar spans in their original positions.
        let non_columnar: Vec<&OrderedTextSpan> = clusters
            .iter()
            .flat_map(|c| c.1.iter().map(|&idx| sorted[idx]))
            .collect();

        // If no spans were Y-reordered, nothing to do
        if non_columnar.is_empty() {
            return;
        }

        // Merge: columnar spans stay at their original indices,
        // non-columnar spans fill the remaining slots in Y-cluster order.
        let mut result: Vec<&OrderedTextSpan> = Vec::with_capacity(sorted.len());
        let mut nc_iter = non_columnar.into_iter();
        for idx in 0..sorted.len() {
            if columnar_indices.contains(&idx) {
                result.push(sorted[idx]); // keep original position
            } else if let Some(span) = nc_iter.next() {
                result.push(span); // fill with Y-reordered span
            }
        }
        // Append any remaining non-columnar spans
        result.extend(nc_iter);

        *sorted = result;
    }

    /// Core rendering logic.
    fn render_spans(
        &self,
        spans: &[OrderedTextSpan],
        tables: &[Table],
        config: &TextPipelineConfig,
    ) -> Result<String> {
        if spans.is_empty() && tables.is_empty() {
            return Ok(String::new());
        }

        let mut sorted: Vec<_> = spans.iter().collect();
        sorted.sort_by_key(|s| s.reading_order);

        // Detect and fix columnar groups that should read row-by-row.
        let columnar_indices = Self::reorder_columnar_groups(&mut sorted);

        // Merge spans from different groups that share the same Y-position.
        // This handles label-value pairs that share a Y-coordinate but live in
        // different reading-order groups (they would otherwise be separated).
        // Only applies when columnar reordering didn't already handle the spans.
        Self::merge_same_y_across_groups(&mut sorted, &columnar_indices);

        let mut tables_rendered = vec![false; tables.len()];
        // Pre-render table text so we can check for orphaned spans.
        let table_texts: Vec<String> = tables.iter().map(Self::render_table_text).collect();
        // Collect spans skipped because they are in a table region, keyed by table index.
        let mut table_skipped_spans: Vec<Vec<&OrderedTextSpan>> = vec![Vec::new(); tables.len()];
        let mut result = String::new();
        let mut prev_span: Option<&OrderedTextSpan> = None;
        // Track (byte_offset, y_position) at the start of each span appended
        // to `result`.  Used later to insert orphaned spans at the correct
        // vertical position instead of appending them at the end.
        let mut line_markers: Vec<(usize, f32)> = Vec::new();

        for (span_idx, span) in sorted.iter().enumerate() {
            // Check if span is in a table region
            if !tables.is_empty() {
                if let Some(table_idx) = super::span_in_table(span, tables) {
                    if !tables_rendered[table_idx] {
                        // Add blank line before table
                        if !result.is_empty() && !result.ends_with("\n\n") {
                            if !result.ends_with('\n') {
                                result.push('\n');
                            }
                            result.push('\n');
                        }
                        result.push_str(&table_texts[table_idx]);
                        tables_rendered[table_idx] = true;
                        prev_span = None;
                    }
                    table_skipped_spans[table_idx].push(span);
                    continue;
                }
            }

            if let Some(prev) = prev_span {
                // Group boundary: when group_id changes, insert a paragraph break
                // to keep spatially partitioned regions (e.g. columns) contiguous.
                // However, spans that were reordered from columnar groups should
                // NOT get paragraph breaks at group boundaries — they use normal
                // line-break logic instead.
                let in_columnar = columnar_indices.contains(&span_idx);
                let y_diff = (span.span.bbox.y - prev.span.bbox.y).abs();
                let same_y = y_diff < span.span.font_size.max(1.0) * 0.5;
                let group_changed = if in_columnar || same_y {
                    false // suppress group-change breaks for reordered columnar spans
                          // and for spans at the same Y-position (merged across groups)
                } else {
                    match (span.group_id, prev.group_id) {
                        (Some(a), Some(b)) => a != b,
                        _ => false,
                    }
                };

                if group_changed || self.is_paragraph_break(span, prev) {
                    result.push_str("\n\n");
                } else {
                    let same_line = same_y;
                    if !same_line {
                        // For columnar spans, a row transition should produce a
                        // newline rather than a space.
                        if in_columnar {
                            result.push('\n');
                        } else {
                            result.push(' ');
                        }
                    } else {
                        // Same visual line: insert a space when there is a
                        // meaningful horizontal gap between the previous span's
                        // right edge and this span's left edge.  This mirrors the
                        // gap-based space insertion in the Markdown converter and
                        // prevents labels from being concatenated with their values
                        // (e.g. "Label$100.00" -> "Label $100.00").
                        //
                        // For columnar spans, always insert a space between
                        // different-group same-row spans even when the gap is
                        // large (the "column boundary" heuristic doesn't apply
                        // after deliberate row-interleaving).
                        let columnar_same_row_space = in_columnar
                            && !result.ends_with(' ')
                            && !span.span.text.starts_with(' ');
                        let needs_gap_space = !result.ends_with(' ')
                            && !span.span.text.starts_with(' ')
                            && super::has_horizontal_gap(&prev.span, &span.span);
                        // "Box label above value" pattern: spans have a small but
                        // non-zero Y offset (2-5pt) and overlap horizontally — the
                        // label sits directly above the value.  Without an explicit
                        // space the two get concatenated (e.g. "49$0.00").
                        let stacked_needs_space = !needs_gap_space
                            && !result.ends_with(' ')
                            && !span.span.text.starts_with(' ')
                            && y_diff > 2.0
                            && !super::has_horizontal_gap(&prev.span, &span.span);
                        if columnar_same_row_space || needs_gap_space || stacked_needs_space {
                            result.push(' ');
                        }
                    }
                }
            }

            line_markers.push((result.len(), span.span.bbox.y));
            result.push_str(&span.span.text);
            prev_span = Some(span);
        }

        // Recover orphaned spans: spans inside a table region whose text does
        // not appear in the rendered table output.  Instead of appending them
        // at the very end (which scatters values away from their context),
        // insert each orphan at the position in the text flow that matches its
        // Y-coordinate.  This keeps form box values next to their labels.
        let mut all_orphans: Vec<&OrderedTextSpan> = Vec::new();
        for (table_idx, skipped) in table_skipped_spans.iter().enumerate() {
            if !tables_rendered[table_idx] || skipped.is_empty() {
                continue;
            }
            let rendered = &table_texts[table_idx];
            for s in skipped {
                let trimmed = s.span.text.trim();
                if !trimmed.is_empty() && !rendered.contains(trimmed) {
                    all_orphans.push(s);
                }
            }
        }

        if !all_orphans.is_empty() {
            // Sort orphans top-to-bottom (descending Y in PDF coordinates),
            // then left-to-right for spans on the same line.
            all_orphans.sort_by(|a, b| {
                let ya = a.span.bbox.y;
                let yb = b.span.bbox.y;
                // Higher Y = higher on page = comes first in reading order
                crate::utils::safe_float_cmp(yb, ya)
                    .then_with(|| crate::utils::safe_float_cmp(a.span.bbox.x, b.span.bbox.x))
            });

            // Group orphans that share the same Y-line (within tolerance).
            let mut orphan_lines: Vec<(f32, Vec<&OrderedTextSpan>)> = Vec::new();
            for orphan in &all_orphans {
                let y = orphan.span.bbox.y;
                let font_size = orphan.span.font_size.max(1.0);
                let tolerance = font_size * 0.5;
                if let Some(last) = orphan_lines.last_mut() {
                    if (last.0 - y).abs() < tolerance {
                        last.1.push(orphan);
                        continue;
                    }
                }
                orphan_lines.push((y, vec![orphan]));
            }

            // Build the text for each orphan line group.
            // Tuple: (byte_position, y_position, text)
            let mut insertions: Vec<(usize, f32, String)> = Vec::new();
            for (orphan_y, group) in &orphan_lines {
                let mut orphan_text = String::new();
                let mut prev: Option<&OrderedTextSpan> = None;
                for span in group {
                    // Space between same-line orphan fragments only when there
                    // is a real horizontal gap. Overlapping fragments are one
                    // continuous run the producer's kerning split across spans;
                    // a space there would break a word.
                    if let Some(p) = prev {
                        if super::has_horizontal_gap(&p.span, &span.span) {
                            orphan_text.push(' ');
                        }
                    }
                    orphan_text.push_str(&span.span.text);
                    prev = Some(span);
                }

                // Find the best insertion point: the last line_marker whose
                // Y-position is >= the orphan's Y (i.e. the last span that
                // appears at or above the orphan on the page).  Insert right
                // after that span's text.
                //
                // PDF coordinates: higher Y = higher on page.
                // line_markers are in reading order (top-to-bottom = decreasing Y).
                let insert_byte = if let Some(pos) = line_markers
                    .iter()
                    .rposition(|&(_, marker_y)| marker_y >= *orphan_y - 1.0)
                {
                    // Find the end of the text that was appended at this marker.
                    // The next marker's offset (or end of result) tells us where
                    // this span's text ends.  But we also need to account for
                    // separators (spaces, newlines) that were added before the
                    // *next* span.  The simplest correct point: the start of
                    // the next marker (which is right before the next span's
                    // separator).  If there is no next marker, use result.len().
                    if pos + 1 < line_markers.len() {
                        line_markers[pos + 1].0
                    } else {
                        result.len()
                    }
                } else {
                    // All markers have Y < orphan_y, meaning the orphan is
                    // above everything in the output.  Insert at the beginning.
                    0
                };

                insertions.push((insert_byte, *orphan_y, orphan_text));
            }

            // Apply insertions from back to front so byte offsets stay valid.
            // For the same byte position, insert lower-Y items first (they end
            // up after higher-Y items, preserving top-to-bottom reading order).
            insertions.sort_by(|a, b| {
                b.0.cmp(&a.0)
                    .then_with(|| crate::utils::safe_float_cmp(a.1, b.1))
            });
            for (byte_pos, _y, text) in insertions {
                // Determine the separator to use before the orphan text.
                let before = if byte_pos > 0 {
                    result.as_bytes().get(byte_pos - 1).copied()
                } else {
                    None
                };
                let mut to_insert = String::new();
                if byte_pos > 0 && before != Some(b' ') && before != Some(b'\n') {
                    to_insert.push(' ');
                }
                to_insert.push_str(&text);
                result.insert_str(byte_pos, &to_insert);
            }
        }

        // Render any unmatched tables
        for (i, table) in tables.iter().enumerate() {
            if !tables_rendered[i] && !table.is_empty() {
                if !result.is_empty() && !result.ends_with("\n\n") {
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push('\n');
                }
                result.push_str(&table_texts[i]);
            }
        }

        if !result.ends_with('\n') {
            result.push('\n');
        }

        // Merge key-value pairs that were split across lines due to column-based
        // reading order (e.g. "Grand Total\n$750.00" → "Grand Total $750.00").
        result = super::merge_key_value_pairs(&result);

        if config.enable_hyphenation_reconstruction {
            let handler = HyphenationHandler::new();
            result = handler.process_text(&result);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::{Color, FontWeight, TextSpan};
    use crate::pipeline::converters::span_in_table;

    fn make_span(text: &str, x: f32, y: f32) -> OrderedTextSpan {
        OrderedTextSpan::new(
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: text.to_string(),
                bbox: Rect::new(x, y, 50.0, 12.0),
                font_name: "Test".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                offset_semantic: false,
                split_boundary_before: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            0,
        )
    }

    #[test]
    fn test_empty_spans() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();
        let result = converter.convert(&[], &config).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_single_line() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span("Hello world", 0.0, 100.0)];
        let result = converter.convert(&spans, &config).unwrap();
        assert_eq!(result, "Hello world\n");
    }

    #[test]
    fn test_paragraph_break() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();
        let mut spans = vec![
            make_span("First paragraph", 0.0, 100.0),
            make_span("Second paragraph", 0.0, 50.0), // Large gap indicates new paragraph
        ];
        spans[1].reading_order = 1;

        let result = converter.convert(&spans, &config).unwrap();
        assert!(result.contains("\n\n"));
    }

    // ============================================================================
    // Table rendering tests
    // ============================================================================

    use crate::structure::table_extractor::{TableCell, TableRow};

    #[test]
    fn test_render_table_text_basic() {
        let mut table = Table::new();
        let mut row1 = TableRow::new(false);
        row1.add_cell(TableCell::new("A".to_string(), false));
        row1.add_cell(TableCell::new("B".to_string(), false));
        table.add_row(row1);

        let mut row2 = TableRow::new(false);
        row2.add_cell(TableCell::new("C".to_string(), false));
        row2.add_cell(TableCell::new("D".to_string(), false));
        table.add_row(row2);

        let result = PlainTextConverter::render_table_text(&table);
        assert_eq!(result, "A   B\nC   D\n");
    }

    #[test]
    fn test_render_table_text_empty() {
        let table = Table::new();
        let result = PlainTextConverter::render_table_text(&table);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_table_text_trims_whitespace() {
        let mut table = Table::new();
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("  padded  ".to_string(), false));
        table.add_row(row);

        let result = PlainTextConverter::render_table_text(&table);
        assert_eq!(result, "padded\n");
    }

    #[test]
    fn test_convert_with_tables_renders_tab_delimited() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("X".to_string(), false));
        row.add_cell(TableCell::new("Y".to_string(), false));
        table.add_row(row);

        let result = converter
            .convert_with_tables(&[], &[table], &config)
            .unwrap();

        assert!(
            result.contains("X") && result.contains("Y") && !result.contains('\t'),
            "Should contain space-padded cells (no tabs): {:?}",
            result,
        );
    }

    #[test]
    fn test_convert_with_tables_mixed_content() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        let mut span_before = make_span("Before", 10.0, 200.0);
        span_before.reading_order = 0;

        // Span inside table region whose text matches table cell content
        // (not an orphan — should be absorbed by the table rendering).
        let mut span_in_table = make_span("Cell", 50.0, 70.0);
        span_in_table.reading_order = 1;

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("Cell".to_string(), false));
        table.add_row(row);

        let result = converter
            .convert_with_tables(&[span_before, span_in_table], &[table], &config)
            .unwrap();

        assert!(result.contains("Before"), "Should contain text before table");
        assert!(result.contains("Cell"), "Should contain table cell");
    }

    #[test]
    fn test_convert_with_tables_no_tables_same_as_convert() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span("Hello", 0.0, 100.0)];

        let result_convert = converter.convert(&spans, &config).unwrap();
        let result_with_tables = converter.convert_with_tables(&spans, &[], &config).unwrap();

        assert_eq!(result_convert, result_with_tables);
    }

    #[test]
    fn test_span_in_table_plain_text() {
        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));

        let inside = make_span("inside", 50.0, 70.0);
        let outside = make_span("outside", 500.0, 500.0);

        assert_eq!(span_in_table(&inside, &[table.clone()]), Some(0));
        assert_eq!(span_in_table(&outside, &[table]), None);
    }

    #[test]
    fn test_span_in_table_no_bbox() {
        let table = Table::new(); // No bbox
        let span = make_span("text", 50.0, 70.0);

        assert_eq!(span_in_table(&span, &[table]), None);
    }

    /// Helper: create a span with explicit width (for gap detection tests).
    fn make_span_with_width(text: &str, x: f32, y: f32, width: f32) -> OrderedTextSpan {
        OrderedTextSpan::new(
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: text.to_string(),
                bbox: Rect::new(x, y, width, 12.0),
                font_name: "Test".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                offset_semantic: false,
                split_boundary_before: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            0,
        )
    }

    #[test]
    fn test_no_concatenation_label_and_currency() {
        // "Subtotal" span followed by "$1,250.00" span with a small gap.
        // Pipeline output should have "Subtotal $1,250.00" with space.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // "Subtotal" at x=100, width=80 -> ends at x=180
        // "$1,250.00" at x=185          -> gap of 5pt (> 12*0.15 = 1.8pt)
        let mut span1 = make_span_with_width("Subtotal", 100.0, 200.0, 80.0);
        span1.reading_order = 0;
        let mut span2 = make_span_with_width("$1,250.00", 185.0, 200.0, 55.0);
        span2.reading_order = 1;

        let result = converter.convert(&[span1, span2], &config).unwrap();
        assert!(
            result.contains("Subtotal $1,250.00"),
            "Should insert space between label and currency: {:?}",
            result,
        );
        assert!(
            !result.contains("Subtotal$1,250.00"),
            "Should NOT concatenate label and currency without space: {:?}",
            result,
        );
    }

    #[test]
    fn test_box_label_above_value_gets_space() {
        // Form box: label "49" at y=930.8, value "$0.00" at y=927.9.
        // Y-diff is 2.9pt — within same-line threshold (font_size*0.5 = 4.0)
        // but they overlap horizontally (label sits ABOVE value).
        // Should produce "49 $0.00", not "49$0.00".
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // "49" at x=100, y=930.8, width=12 (small box label)
        let mut span1 = make_span_with_width("49", 100.0, 930.8, 12.0);
        span1.span.font_size = 8.0;
        span1.reading_order = 0;

        // "$0.00" at x=100, y=927.9, width=30 (value directly below label)
        let mut span2 = make_span_with_width("$0.00", 100.0, 927.9, 30.0);
        span2.span.font_size = 8.0;
        span2.reading_order = 1;

        let result = converter.convert(&[span1, span2], &config).unwrap();
        assert!(
            result.contains("49 $0.00"),
            "Should insert space between stacked box label and value: {:?}",
            result,
        );
    }

    #[test]
    fn test_orphaned_spans_in_table_region_preserved() {
        // Form: table detected by vector lines covers a region with dollar
        // values, but the table's cell text doesn't include all the values.
        // Orphaned spans (in table bbox but not in table cell text) should
        // still appear in the output.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Table covers region x=50..250, y=100..300
        let mut table = Table::new();
        table.bbox = Some(Rect::new(50.0, 100.0, 200.0, 200.0));
        table.col_count = 2;
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("Gross revenue".to_string(), false));
        row.add_cell(TableCell::new("".to_string(), false)); // empty cell!
        table.add_row(row);

        // Span BEFORE table
        let mut span_before = make_span("John Doe", 10.0, 400.0);
        span_before.reading_order = 0;

        // Span INSIDE table region - text IS in table cells
        let mut span_in_table = make_span("Gross revenue", 60.0, 250.0);
        span_in_table.reading_order = 1;

        // Span INSIDE table region - text NOT in any cell (orphaned)
        let mut orphan_span = make_span_with_width("85432.50", 150.0, 200.0, 50.0);
        orphan_span.reading_order = 2;

        // Another orphan
        let mut orphan_span2 = make_span_with_width("42716.25", 150.0, 170.0, 50.0);
        orphan_span2.reading_order = 3;

        let result = converter
            .convert_with_tables(
                &[span_before, span_in_table, orphan_span, orphan_span2],
                &[table],
                &config,
            )
            .unwrap();

        assert!(
            result.contains("85432.50"),
            "Orphaned span '85432.50' should appear in output: {:?}",
            result,
        );
        assert!(
            result.contains("42716.25"),
            "Orphaned span '42716.25' should appear in output: {:?}",
            result,
        );
        assert!(
            result.contains("Gross revenue"),
            "Table cell content should appear: {:?}",
            result,
        );
    }

    #[test]
    fn test_orphaned_spans_inserted_at_correct_y_position() {
        // Orphaned spans should be inserted at their correct Y-position in the
        // text flow, not appended at the very end.
        //
        // Layout:
        //   "Header" at y=500 (above table)
        //   Table region y=200..400 containing orphans:
        //     "Value1" at y=350, "Value2" at y=300
        //   "Footer" at y=100 (below table)
        //
        // Expected order: Header, [table], Value1, Value2, Footer
        // NOT:            Header, [table], Footer, Value1, Value2
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Table covers region x=50..250, y=200..400
        let mut table = Table::new();
        table.bbox = Some(Rect::new(50.0, 200.0, 200.0, 200.0));
        table.col_count = 1;
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("Label".to_string(), false));
        table.add_row(row);

        // Header span above the table
        let mut span_header = make_span("Header", 10.0, 500.0);
        span_header.reading_order = 0;

        // Span inside table that matches table cell (not orphaned)
        let mut span_label = make_span("Label", 60.0, 380.0);
        span_label.reading_order = 1;

        // Orphaned span at y=350 (inside table region, not in rendered table)
        let mut orphan1 = make_span_with_width("Value1", 150.0, 350.0, 50.0);
        orphan1.reading_order = 2;

        // Orphaned span at y=300 (inside table region, not in rendered table)
        let mut orphan2 = make_span_with_width("Value2", 150.0, 300.0, 50.0);
        orphan2.reading_order = 3;

        // Footer span below the table
        let mut span_footer = make_span("Footer", 10.0, 100.0);
        span_footer.reading_order = 4;

        let result = converter
            .convert_with_tables(
                &[span_header, span_label, orphan1, orphan2, span_footer],
                &[table],
                &config,
            )
            .unwrap();

        // All content should be present
        assert!(result.contains("Header"), "Missing Header: {:?}", result);
        assert!(result.contains("Value1"), "Missing Value1: {:?}", result);
        assert!(result.contains("Value2"), "Missing Value2: {:?}", result);
        assert!(result.contains("Footer"), "Missing Footer: {:?}", result);

        // Orphans should appear BEFORE Footer, not after it.
        let value1_pos = result.find("Value1").unwrap();
        let value2_pos = result.find("Value2").unwrap();
        let footer_pos = result.find("Footer").unwrap();

        assert!(
            value1_pos < footer_pos,
            "Value1 (pos {}) should appear before Footer (pos {}): {:?}",
            value1_pos,
            footer_pos,
            result,
        );
        assert!(
            value2_pos < footer_pos,
            "Value2 (pos {}) should appear before Footer (pos {}): {:?}",
            value2_pos,
            footer_pos,
            result,
        );
        // Value1 is at higher Y (350) than Value2 (300), so should come first.
        assert!(
            value1_pos < value2_pos,
            "Value1 (y=350, pos {}) should appear before Value2 (y=300, pos {}): {:?}",
            value1_pos,
            value2_pos,
            result,
        );
    }

    #[test]
    fn test_orphan_overlapping_fragments_no_spurious_space() {
        // Same overlapping-fragment defect as the main converter path, but in
        // the in-table orphan-recovery path: a value drawn as a Td-chain of
        // runs lands inside a detected table region without matching any cell,
        // so the spans become orphans. Adjacent orphan fragments whose boxes
        // overlap are one continuous run the producer split; joining them with
        // a space would break the word ("DELT" + "A" -> "DELT A").
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Table region x=50..250, y=100..300, with a cell that does NOT contain
        // the fragment text, forcing the fragments to be treated as orphans.
        let mut table = Table::new();
        table.bbox = Some(Rect::new(50.0, 100.0, 200.0, 200.0));
        table.col_count = 1;
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("Label".to_string(), false));
        table.add_row(row);

        // "DELT" ends at x = 100 + 30 = 130; "A" starts at 129.5 (overlap),
        // same Y (y=200), both inside the table region.
        let mut frag1 = make_span_with_width("DELT", 100.0, 200.0, 30.0);
        frag1.reading_order = 0;
        let mut frag2 = make_span_with_width("A", 129.5, 200.0, 8.0);
        frag2.reading_order = 1;

        let result = converter
            .convert_with_tables(&[frag1, frag2], &[table], &config)
            .unwrap();

        assert!(result.contains("DELTA"), "overlapping orphan fragments must join: {:?}", result,);
        assert!(
            !result.contains("DELT A"),
            "no spurious space between overlapping orphan fragments: {:?}",
            result,
        );
    }

    #[test]
    fn test_orphan_gapped_fragments_keep_space() {
        // Counterpart to the overlap case: orphan fragments on the same line
        // with a real horizontal gap are distinct tokens and must keep their
        // space ("Field" + wide gap + "Value" -> "Field Value").
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        let mut table = Table::new();
        table.bbox = Some(Rect::new(50.0, 100.0, 400.0, 200.0));
        table.col_count = 1;
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("Label".to_string(), false));
        table.add_row(row);

        // "Field" ends at x = 100 + 40 = 140; "Value" starts at 200 (60pt gap).
        let mut frag1 = make_span_with_width("Field", 100.0, 200.0, 40.0);
        frag1.reading_order = 0;
        let mut frag2 = make_span_with_width("Value", 200.0, 200.0, 40.0);
        frag2.reading_order = 1;

        let result = converter
            .convert_with_tables(&[frag1, frag2], &[table], &config)
            .unwrap();

        assert!(
            result.contains("Field Value"),
            "gapped orphan fragments must keep their space: {:?}",
            result,
        );
    }

    #[test]
    fn test_no_extra_space_when_overlapping_spans() {
        // Spans with overlapping/touching bboxes should NOT get an extra space.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // "Hello" at x=100, width=30 -> ends at x=130
        // "World" at x=130           -> gap of 0 (touching)
        let mut span1 = make_span_with_width("Hello ", 100.0, 200.0, 30.0);
        span1.reading_order = 0;
        let mut span2 = make_span_with_width("World", 130.0, 200.0, 30.0);
        span2.reading_order = 1;

        let result = converter.convert(&[span1, span2], &config).unwrap();
        assert_eq!(result, "Hello World\n");
    }

    #[test]
    fn test_split_line_not_misdetected_as_column() {
        // Regression: one line of text is emitted as several spans in distinct
        // reading-order groups, with adjacent spans overlapping by a fraction
        // of a point. Here "ALPHA BETA GAMMA
        // DELT" (group 0) is continued by "A EP" + "SILON" (group 1) on the SAME
        // row. The columnar detector used to treat group 1 (two spans sharing a
        // Y) as a vertical column and interleave it, hoisting "A EP" to the front
        // ("A EP ALPHA ... DELT SILON"). A real column has one cell per row, so a
        // group whose spans share a row must not be a column candidate.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        let mut header = make_span_with_width("Header", 35.0, 634.0, 64.0);
        header.reading_order = 0;
        header.group_id = Some(0);
        // wide span ending at x≈138
        let mut name_a = make_span_with_width("ALPHA BETA GAMMA DELT", 35.0, 622.0, 103.0);
        name_a.reading_order = 1;
        name_a.group_id = Some(0);
        let mut row_c = make_span_with_width("LabelC", 35.0, 606.0, 30.0);
        row_c.reading_order = 2;
        row_c.group_id = Some(0);
        let mut row_d = make_span_with_width("LabelD", 35.0, 578.0, 51.0);
        row_d.reading_order = 3;
        row_d.group_id = Some(0);
        // group 1: two fragments on the SAME row (y=622), overlapping group 0
        let mut frag_a = make_span_with_width("A EP", 137.7, 622.0, 18.4);
        frag_a.reading_order = 4;
        frag_a.group_id = Some(1);
        let mut frag_b = make_span_with_width("SILON", 155.5, 622.0, 15.1);
        frag_b.reading_order = 5;
        frag_b.group_id = Some(1);
        let mut right_c = make_span_with_width("RightC", 167.0, 606.0, 66.0);
        right_c.reading_order = 6;
        right_c.group_id = Some(1);

        let spans = vec![header, name_a, row_c, row_d, frag_a, frag_b, right_c];
        let result = converter.convert(&spans, &config).unwrap();

        // Reconstructed left-to-right, the fragments form one word: DELT + A EP +
        // SILON = "DELTA EPSILON". Order preserved, no spurious mid-word space.
        assert!(
            result.contains("ALPHA BETA GAMMA DELTA EPSILON"),
            "name should read in order with no spurious split: {:?}",
            result,
        );
        assert!(
            !result.contains("A EP ALPHA"),
            "fragment must not be hoisted ahead of the line: {:?}",
            result,
        );
    }

    #[test]
    fn test_overlapping_cross_group_spans_no_spurious_space() {
        // Two same-row spans from different reading-order groups that OVERLAP
        // horizontally are one continuous run the grouping split — they must not
        // get the cross-group "label/value" space.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // "DELT" ends at x = 100 + 30 = 130; "A" starts at 129.5 (overlap).
        let mut s1 = make_span_with_width("DELT", 100.0, 200.0, 30.0);
        s1.reading_order = 0;
        s1.group_id = Some(0);
        let mut s2 = make_span_with_width("A", 129.5, 200.0, 8.0);
        s2.reading_order = 1;
        s2.group_id = Some(1);

        let result = converter.convert(&[s1, s2], &config).unwrap();
        assert!(result.contains("DELTA"), "overlapping fragments must join: {:?}", result);
        assert!(!result.contains("DELT A"), "no spurious cross-group space: {:?}", result);
    }

    // ============================================================================
    // Columnar group reordering tests
    // ============================================================================

    #[test]
    fn test_columnar_groups_merged_into_rows() {
        // Simulates a columnar appointments table:
        //   Group 0 (x=50):  "Name",   "Alice", "Bob",  "Carol"
        //   Group 1 (x=200): "Service", "Checkup", "Consultation", "Follow-up"
        //   Group 2 (x=400): "Date",    "Sep 11, 2025", "Oct 23, 2025", "Nov 17, 2025"
        //
        // Without fix: columns render sequentially (all names, then all services, then all dates).
        // With fix: rows render together (Name Service Date, Alice Checkup Sep 11...).
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Column 1: Names (group_id=0)
        let mut s0 = make_span_with_width("Name", 50.0, 400.0, 60.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        let mut s1 = make_span_with_width("Alice", 50.0, 385.0, 40.0);
        s1.reading_order = 1;
        s1.group_id = Some(0);

        let mut s2 = make_span_with_width("Bob", 50.0, 370.0, 40.0);
        s2.reading_order = 2;
        s2.group_id = Some(0);

        let mut s3 = make_span_with_width("Carol", 50.0, 355.0, 40.0);
        s3.reading_order = 3;
        s3.group_id = Some(0);

        // Column 2: Services (group_id=1)
        let mut s4 = make_span_with_width("Service", 200.0, 400.0, 80.0);
        s4.reading_order = 4;
        s4.group_id = Some(1);

        let mut s5 = make_span_with_width("Checkup", 200.0, 385.0, 80.0);
        s5.reading_order = 5;
        s5.group_id = Some(1);

        let mut s6 = make_span_with_width("Consultation", 200.0, 370.0, 100.0);
        s6.reading_order = 6;
        s6.group_id = Some(1);

        let mut s7 = make_span_with_width("Follow-up", 200.0, 355.0, 100.0);
        s7.reading_order = 7;
        s7.group_id = Some(1);

        // Column 3: Dates (group_id=2)
        let mut s8 = make_span_with_width("Date", 400.0, 400.0, 80.0);
        s8.reading_order = 8;
        s8.group_id = Some(2);

        let mut s9 = make_span_with_width("Sep 11, 2025", 400.0, 385.0, 80.0);
        s9.reading_order = 9;
        s9.group_id = Some(2);

        let mut s10 = make_span_with_width("Oct 23, 2025", 400.0, 370.0, 80.0);
        s10.reading_order = 10;
        s10.group_id = Some(2);

        let mut s11 = make_span_with_width("Nov 17, 2025", 400.0, 355.0, 80.0);
        s11.reading_order = 11;
        s11.group_id = Some(2);

        let spans = vec![s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11];
        let result = converter.convert(&spans, &config).unwrap();

        // Verify row-by-row ordering: each header/data row should appear together.
        assert!(
            result.contains("Name") && result.contains("Service") && result.contains("Date"),
            "Should contain all headers: {:?}",
            result,
        );

        // Verify the output is row-by-row by checking each line.
        let lines: Vec<&str> = result.trim().lines().collect();
        assert!(
            lines.len() >= 4,
            "Should have at least 4 lines (header + 3 data rows): {:?}",
            result,
        );

        // Header row: Name, Service, Date all on one line.
        assert!(
            lines[0].contains("Name") && lines[0].contains("Service") && lines[0].contains("Date"),
            "Header row should contain Name, Service, and Date: {:?}",
            lines[0],
        );

        // Data row 1: Alice, Checkup, Sep 11, 2025 all on one line.
        assert!(
            lines[1].contains("Alice")
                && lines[1].contains("Checkup")
                && lines[1].contains("Sep 11, 2025"),
            "Row 1 should contain Alice, Checkup, Sep 11, 2025: {:?}",
            lines[1],
        );

        // Data row 2: Bob, Consultation, Oct 23, 2025 all on one line.
        assert!(
            lines[2].contains("Bob")
                && lines[2].contains("Consultation")
                && lines[2].contains("Oct 23, 2025"),
            "Row 2 should contain Bob, Consultation, Oct 23, 2025: {:?}",
            lines[2],
        );

        // Data row 3: Carol, Follow-up, Nov 17, 2025 all on one line.
        assert!(
            lines[3].contains("Carol")
                && lines[3].contains("Follow-up")
                && lines[3].contains("Nov 17, 2025"),
            "Row 3 should contain Carol, Follow-up, Nov 17, 2025: {:?}",
            lines[3],
        );

        // Verify the output does NOT have the old column-by-column pattern
        // where all names appear before any services.
        let name_pos = result.find("Name").unwrap();
        let alice_pos = result.find("Alice").unwrap();
        let service_pos = result.find("Service").unwrap();
        assert!(
            service_pos < alice_pos,
            "Service (header) should appear before Alice (data): {:?}",
            result,
        );
        assert!(
            name_pos < service_pos,
            "Name should appear before Service on header line: {:?}",
            result,
        );
    }

    #[test]
    fn test_columnar_groups_two_columns() {
        // Minimal case: two columns, two rows each.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        let mut s0 = make_span_with_width("A", 50.0, 200.0, 30.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        let mut s1 = make_span_with_width("B", 50.0, 185.0, 30.0);
        s1.reading_order = 1;
        s1.group_id = Some(0);

        let mut s2 = make_span_with_width("C", 200.0, 200.0, 30.0);
        s2.reading_order = 2;
        s2.group_id = Some(1);

        let mut s3 = make_span_with_width("D", 200.0, 185.0, 30.0);
        s3.reading_order = 3;
        s3.group_id = Some(1);

        let spans = vec![s0, s1, s2, s3];
        let result = converter.convert(&spans, &config).unwrap();

        // Should produce: "A C\nB D\n" (row-by-row), not "A B\n\nC D\n" (col-by-col).
        let lines: Vec<&str> = result.trim().lines().collect();
        assert!(lines.len() >= 2, "Should have at least 2 lines: {:?}", result,);
        assert!(
            lines[0].contains('A') && lines[0].contains('C'),
            "First row should have A and C: {:?}",
            result,
        );
        assert!(
            lines[1].contains('B') && lines[1].contains('D'),
            "Second row should have B and D: {:?}",
            result,
        );
    }

    #[test]
    fn test_key_value_pair_merging_in_plain_text() {
        // Simulate the scenario where label and value are in different groups
        // (different columns) but at the same Y-position. After column reordering
        // fails (they don't meet columnar criteria), they appear on separate lines.
        // The post-processing merge should combine them.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Group 0 spans (labels column)
        let mut s0 = make_span("Grand Total", 50.0, 200.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        let mut s1 = make_span("Net Amount", 50.0, 185.0);
        s1.reading_order = 1;
        s1.group_id = Some(0);

        // Group 1 spans (values column) - same Y positions
        let mut s2 = make_span("$750.00", 300.0, 200.0);
        s2.reading_order = 2;
        s2.group_id = Some(1);

        let mut s3 = make_span("$250.00", 300.0, 185.0);
        s3.reading_order = 3;
        s3.group_id = Some(1);

        let spans = vec![s0, s1, s2, s3];
        let result = converter.convert(&spans, &config).unwrap();

        // The key-value merge should combine label+value pairs
        assert!(
            result.contains("Grand Total $750.00"),
            "Should merge label with value on same line: {:?}",
            result,
        );
        assert!(
            result.contains("Net Amount $250.00"),
            "Should merge label with value on same line: {:?}",
            result,
        );
    }

    #[test]
    fn test_non_columnar_groups_not_merged() {
        // Groups that are vertically stacked (not side-by-side columns) should
        // NOT be reordered.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Group 0: two spans at x=50, y=400..385
        let mut s0 = make_span_with_width("Title", 50.0, 400.0, 60.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        let mut s1 = make_span_with_width("Subtitle", 50.0, 385.0, 60.0);
        s1.reading_order = 1;
        s1.group_id = Some(0);

        // Group 1: two spans at x=50 (same X!), y=300..285
        // This is NOT a column layout — it is stacked vertically.
        let mut s2 = make_span_with_width("Body line 1", 50.0, 300.0, 80.0);
        s2.reading_order = 2;
        s2.group_id = Some(1);

        let mut s3 = make_span_with_width("Body line 2", 50.0, 285.0, 80.0);
        s3.reading_order = 3;
        s3.group_id = Some(1);

        let spans = vec![s0, s1, s2, s3];
        let result = converter.convert(&spans, &config).unwrap();

        // Should keep paragraph break between groups (vertical stacking).
        assert!(
            result.contains("Subtitle\n\nBody line 1"),
            "Vertically stacked groups should have paragraph break: {:?}",
            result,
        );
    }

    #[test]
    fn test_same_y_spans_from_different_groups_on_same_line() {
        // Invoice scenario: "Amount Due" at (x=478, y=134) in group 5
        // and "$1,500.00" at (x=551, y=134) in group 7.
        // They share the same Y-position but are in different reading-order groups.
        // The output should place them on the same line.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Simulate several groups with spans at different Y-positions, plus the
        // cross-group same-Y pair.
        let mut s0 = make_span_with_width("Patient Name", 50.0, 300.0, 100.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        let mut s1 = make_span_with_width("John Smith", 50.0, 285.0, 100.0);
        s1.reading_order = 1;
        s1.group_id = Some(0);

        // Group 5: label at y=134
        let mut s2 = make_span_with_width("Amount Due", 478.0, 134.0, 70.0);
        s2.reading_order = 10;
        s2.group_id = Some(5);

        // Group 7: value at y=134 (same Y!)
        let mut s3 = make_span_with_width("$1,500.00", 551.0, 134.0, 55.0);
        s3.reading_order = 20;
        s3.group_id = Some(7);

        let spans = vec![s0, s1, s2, s3];
        let result = converter.convert(&spans, &config).unwrap();

        assert!(
            result.contains("Amount Due $1,500.00"),
            "Same-Y spans from different groups should appear on the same line: {:?}",
            result,
        );
    }

    #[test]
    fn test_same_y_multiple_groups_sorted_by_x() {
        // Three spans at the same Y from three different groups should be
        // sorted by X-position on a single line.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        let mut s0 = make_span_with_width("Left", 50.0, 200.0, 40.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        let mut s1 = make_span_with_width("Middle", 200.0, 200.0, 50.0);
        s1.reading_order = 5;
        s1.group_id = Some(2);

        let mut s2 = make_span_with_width("Right", 400.0, 200.0, 50.0);
        s2.reading_order = 10;
        s2.group_id = Some(4);

        let spans = vec![s0, s1, s2];
        let result = converter.convert(&spans, &config).unwrap();

        // All three should be on a single line in L-to-R order.
        let line = result.trim();
        assert!(
            line.contains("Left") && line.contains("Middle") && line.contains("Right"),
            "All three should be on the same line: {:?}",
            result,
        );
        let left_pos = line.find("Left").unwrap();
        let mid_pos = line.find("Middle").unwrap();
        let right_pos = line.find("Right").unwrap();
        assert!(
            left_pos < mid_pos && mid_pos < right_pos,
            "Should be sorted by X: Left < Middle < Right: {:?}",
            result,
        );
    }

    #[test]
    fn test_same_y_merge_preserves_distinct_y_ordering() {
        // Spans at different Y-positions should still maintain top-to-bottom order
        // even when same-Y merging is active.
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Line 1 at y=300
        let mut s0 = make_span_with_width("Header", 50.0, 300.0, 60.0);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        // Line 2 at y=200 - label in group 1, value in group 2
        let mut s1 = make_span_with_width("Amount", 50.0, 200.0, 60.0);
        s1.reading_order = 1;
        s1.group_id = Some(1);

        let mut s2 = make_span_with_width("$100.00", 300.0, 200.0, 50.0);
        s2.reading_order = 5;
        s2.group_id = Some(2);

        // Line 3 at y=100
        let mut s3 = make_span_with_width("Footer", 50.0, 100.0, 60.0);
        s3.reading_order = 10;
        s3.group_id = Some(3);

        let spans = vec![s0, s1, s2, s3];
        let result = converter.convert(&spans, &config).unwrap();

        // "Amount $100.00" should be on the same line
        assert!(
            result.contains("Amount $100.00"),
            "Same-Y pair should be on one line: {:?}",
            result,
        );

        // Order: Header before Amount, Amount before Footer
        let header_pos = result.find("Header").unwrap();
        let amount_pos = result.find("Amount").unwrap();
        let footer_pos = result.find("Footer").unwrap();
        assert!(
            header_pos < amount_pos && amount_pos < footer_pos,
            "Top-to-bottom order should be preserved: {:?}",
            result,
        );
    }

    /// Bidi-isolation markers (UAX #9 §2.4) are a markdown-only
    /// concern (#537 follow-up). Plain-text consumers do not honour
    /// UAX #9 — the markers would appear as literal garbage glyphs
    /// in terminals, grep output, RAG ingestion. The plain-text
    /// converter MUST NOT emit U+2066 / U+2067 / U+2068 / U+2069
    /// even for RTL content. This is the contract that the v0.3.55
    /// plan's acceptance criterion #2 enshrines.
    #[test]
    fn plain_text_omits_all_isolation_markers() {
        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();

        // Mix Hebrew with English so both block-direction branches of
        // the (markdown-only) wrapper would have fired.
        let mut s0 = make_span("The article שלום עולם is greetings.", 0.0, 200.0);
        s0.reading_order = 0;
        let mut s1 = make_span("הספר Microsoft חדש", 0.0, 180.0);
        s1.reading_order = 1;

        let result = converter.convert(&[s0, s1], &config).unwrap();
        for marker in ['\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'] {
            assert!(
                !result.contains(marker),
                "plain-text output must not contain U+{:04X}, got:\n{:?}",
                marker as u32,
                result
            );
        }
    }
}
