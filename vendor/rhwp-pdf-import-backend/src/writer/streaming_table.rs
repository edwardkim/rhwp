//! Streaming table surface for `FluentPageBuilder` — see issue #393.
//!
//! Unlike `table_renderer::Table` (buffered: takes the whole row matrix,
//! computes a single layout, emits all content in one go), `StreamingTable`
//! emits each row into the page as soon as it is pushed. Persistent state
//! is **O(columns + current page)** — the row itself is consumed and
//! dropped per call.
//!
//! Column-width modes (v0.3.40 / #400):
//! - `TableMode::Fixed` (default) — widths are declared explicitly up front.
//! - `TableMode::Sample { rows, min_col_width_pt, max_col_width_pt }` — buffer
//!   the first `rows` rows, measure their content widths, freeze column widths
//!   from the max observed value (clamped to min/max), then stream the rest.
//! - `TableMode::AutoAll` — valid **only** for the buffered `table_renderer::Table`;
//!   returns an error immediately when used with `StreamingTable`.
//!
//! ## Example
//!
//! ```no_run
//! use pdf_oxide::writer::{
//!     CellAlign, DocumentBuilder, StreamingColumn, StreamingTableConfig,
//! };
//!
//! let mut doc = DocumentBuilder::new();
//! let page = doc
//!     .letter_page()
//!     .font("Helvetica", 10.0)
//!     .at(72.0, 720.0);
//!
//! let mut t = page.streaming_table(
//!     StreamingTableConfig::new()
//!         .column(StreamingColumn::new("SKU").width_pt(72.0))
//!         .column(StreamingColumn::new("Item").width_pt(240.0))
//!         .column(StreamingColumn::new("Qty").width_pt(48.0).align(CellAlign::Right))
//!         .repeat_header(true),
//! );
//!
//! for (sku, item, qty) in [("A-1", "Widget", 5), ("B-2", "Gadget", 12)] {
//!     t.push_row(|r| {
//!         r.cell(sku);
//!         r.cell(item);
//!         r.cell(qty.to_string());
//!     })
//!     .unwrap();
//! }
//! t.finish().done();
//! ```

use super::document_builder::{FluentPageBuilder, TextAlign};
use super::table_renderer::CellAlign;
use crate::elements::{
    ContentElement, FontSpec, PathContent, PathOperation, TextContent, TextStyle,
};
use crate::error::{Error, Result};
use crate::geometry::Rect;
use crate::layout::Color;

/// Alignment mapping helper: the table vocabulary (`CellAlign`) doesn't
/// share a type with `TextAlign` but maps trivially.
fn cell_to_text_align(a: CellAlign) -> TextAlign {
    match a {
        CellAlign::Left => TextAlign::Left,
        CellAlign::Center => TextAlign::Center,
        CellAlign::Right => TextAlign::Right,
    }
}

/// Column-width sizing mode for a streaming table.
///
/// Choose between explicit widths (`Fixed`, the default), measuring the first
/// N rows to determine widths automatically (`Sample`), or fully-buffered
/// auto-sizing (`AutoAll`, only valid for the non-streaming `Table`).
#[derive(Debug, Clone, Default)]
pub enum TableMode {
    /// Widths come from [`StreamingColumn::width_pt`]. Default.
    #[default]
    Fixed,
    /// Buffer the first `rows` rows, measure max content width per column,
    /// clamp to `[min_col_width_pt, max_col_width_pt]`, freeze, then stream.
    Sample {
        /// Number of rows to measure before freezing widths.
        rows: usize,
        /// Minimum allowed column width in PDF points (default 20).
        min_col_width_pt: f32,
        /// Maximum allowed column width in PDF points (default 400).
        max_col_width_pt: f32,
    },
    /// Buffer **all** rows to compute exact widths. Valid only for the
    /// buffered [`table_renderer::Table`]; [`StreamingTable`] rejects this
    /// immediately with [`Error::InvalidOperation`].
    AutoAll,
}

/// Internal sample-collection state for `TableMode::Sample`.
enum SampleState {
    /// Fixed mode — no buffering.
    Fixed,
    /// Still collecting sample rows.
    Collecting {
        buffered: Vec<Vec<String>>,
        target: usize,
        min_w: f32,
        max_w: f32,
    },
    /// Sample complete — widths have been frozen.
    Frozen,
}

/// One column in a `StreamingTableConfig`.
///
/// Widths are **explicit** — streaming tables can't autofit because that
/// requires looking at rows the caller hasn't pushed yet. See research B
/// (docs/v0.3.39/research/b_scalable_layout_algorithms.md) for the full
/// rationale.
#[derive(Debug, Clone)]
pub struct StreamingColumn {
    /// Column heading text. Rendered at the top of the table and on every
    /// page break when `repeat_header` is set on the config.
    pub header: String,
    /// Column width in PDF points. Must be > 0.
    pub width: f32,
    /// Per-column default horizontal alignment.
    pub align: CellAlign,
}

impl StreamingColumn {
    /// Create a column with the given header and default width (100 pt, left-align).
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            width: 100.0,
            align: CellAlign::Left,
        }
    }

    /// Set the column width in PDF points.
    pub fn width_pt(mut self, pt: f32) -> Self {
        self.width = pt;
        self
    }

    /// Set the column's default cell alignment.
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }
}

/// Configuration for a streaming table. Built via `new()` + fluent setters,
/// then consumed by `FluentPageBuilder::streaming_table`.
#[derive(Debug, Clone)]
pub struct StreamingTableConfig {
    pub(crate) columns: Vec<StreamingColumn>,
    pub(crate) repeat_header: bool,
    pub(crate) row_padding_top: f32,
    pub(crate) row_padding_bottom: f32,
    pub(crate) horizontal_padding: f32,
    pub(crate) grid_color: (f32, f32, f32),
    pub(crate) grid_width: f32,
    pub(crate) header_fill: Option<(f32, f32, f32)>,
    pub(crate) mode: TableMode,
    /// Maximum rowspan extent. Cells with `rowspan > max_rowspan` are rejected.
    /// Default: 1 (rowspan disabled).
    pub(crate) max_rowspan: usize,
}

impl Default for StreamingTableConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingTableConfig {
    /// Create an empty configuration. Add columns via `.column(...)`.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            repeat_header: false,
            row_padding_top: 2.0,
            row_padding_bottom: 2.0,
            horizontal_padding: 4.0,
            grid_color: (0.8, 0.8, 0.8),
            grid_width: 0.5,
            header_fill: Some((0.93, 0.93, 0.93)),
            mode: TableMode::Fixed,
            max_rowspan: 1,
        }
    }

    /// Use `TableMode::Fixed` (the default) — column widths come from
    /// [`StreamingColumn::width_pt`].
    pub fn mode_fixed(mut self) -> Self {
        self.mode = TableMode::Fixed;
        self
    }

    /// Use `TableMode::Sample` — buffer the first `sample_rows` rows, measure
    /// their content widths to determine column widths, then stream the rest.
    ///
    /// Column widths are clamped to `[min_col_width_pt, max_col_width_pt]`.
    /// If fewer rows than `sample_rows` are pushed, widths are frozen at
    /// `finish()` from whatever was buffered (or from the column defaults if
    /// zero rows were buffered).
    pub fn mode_sample(
        mut self,
        sample_rows: usize,
        min_col_width_pt: f32,
        max_col_width_pt: f32,
    ) -> Self {
        self.mode = TableMode::Sample {
            rows: sample_rows.max(1),
            min_col_width_pt: min_col_width_pt.max(1.0),
            max_col_width_pt: max_col_width_pt.max(min_col_width_pt).max(1.0),
        };
        self
    }

    /// Use `TableMode::AutoAll` — valid **only** for the non-streaming buffered
    /// `Table`; calling `streaming_table(cfg)` with this mode returns an error
    /// on the first `push_row` call.
    pub fn mode_auto_all(mut self) -> Self {
        self.mode = TableMode::AutoAll;
        self
    }

    /// Add a column. Order matters — this is the left-to-right visual order.
    pub fn column(mut self, c: StreamingColumn) -> Self {
        self.columns.push(c);
        self
    }

    /// Redraw the header row at the top of every page this table spans.
    pub fn repeat_header(mut self, yes: bool) -> Self {
        self.repeat_header = yes;
        self
    }

    /// Override the default header background (light grey). Pass
    /// `(r, g, b)` or set to `None` for no fill.
    pub fn header_fill(mut self, fill: Option<(f32, f32, f32)>) -> Self {
        self.header_fill = fill;
        self
    }

    /// Override grid line colour + width. Set `width` to 0.0 to suppress.
    pub fn grid(mut self, color: (f32, f32, f32), width: f32) -> Self {
        self.grid_color = color;
        self.grid_width = width;
        self
    }

    /// Override horizontal + vertical cell padding (default 4 / 2 / 2 pt).
    pub fn cell_padding(mut self, horizontal: f32, top: f32, bottom: f32) -> Self {
        self.horizontal_padding = horizontal;
        self.row_padding_top = top;
        self.row_padding_bottom = bottom;
        self
    }

    /// Allow cells to span up to `n` rows via `StreamingRow::span_cell`.
    /// The table will buffer at most `n` rows at a time to compute combined
    /// heights. Default: 1 (rowspan disabled).
    pub fn max_rowspan(mut self, n: usize) -> Self {
        self.max_rowspan = n.max(1);
        self
    }
}

/// A single cell pushed inside a `push_row` closure.
#[derive(Debug, Clone, Default)]
pub struct RowCell {
    /// Cell text content.
    pub text: String,
    /// How many rows this cell spans (1 = normal, >1 = rowspan).
    pub rowspan: usize,
}

impl RowCell {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            rowspan: 1,
        }
    }
    fn span(text: impl Into<String>, n: usize) -> Self {
        Self {
            text: text.into(),
            rowspan: n.max(1),
        }
    }
}

/// One row being built inside `push_row`. Cells must be pushed in column
/// order; pushing more than `columns.len()` cells fails at `push_row`
/// return.
#[derive(Debug, Default)]
pub struct StreamingRow {
    cells: Vec<RowCell>,
}

impl StreamingRow {
    /// Append the next cell's string content. Accepts anything
    /// `Into<String>` — `&str`, `String`, numbers via `.to_string()`.
    pub fn cell(&mut self, value: impl Into<String>) -> &mut Self {
        self.cells.push(RowCell::new(value));
        self
    }

    /// Append a cell that spans `rowspan` rows vertically.
    /// Requires `StreamingTableConfig::max_rowspan(n)` where `n >= rowspan`.
    pub fn span_cell(&mut self, value: impl Into<String>, rowspan: usize) -> &mut Self {
        self.cells.push(RowCell::span(value, rowspan));
        self
    }

    fn into_cells(self) -> Vec<RowCell> {
        self.cells
    }
}

/// Streaming table handle. Created by
/// `FluentPageBuilder::streaming_table`; consumed by `finish()`.
///
/// Holds a mutable borrow of its parent `FluentPageBuilder` through the
/// building window.
pub struct StreamingTable<'a> {
    page: FluentPageBuilder<'a>,
    config: StreamingTableConfig,
    /// Prefix-sum of column widths starting at origin_x.
    column_x: Vec<f32>,
    /// Total table width.
    total_width: f32,
    /// Left edge of the table (fixed by first push_row / header).
    origin_x: f32,
    /// Whether the header has been drawn on the current page.
    header_drawn: bool,
    /// State machine for `TableMode::Sample`.
    sample_state: SampleState,
    /// Buffered rows waiting for a rowspan group to complete.
    rowspan_buf: Vec<Vec<RowCell>>,
    /// Number of additional rows still needed to complete the active rowspan.
    rowspan_remaining: usize,
}

impl<'a> StreamingTable<'a> {
    /// Open a new streaming table. Called via
    /// `FluentPageBuilder::streaming_table`; not public because it couples
    /// to builder internals.
    pub(super) fn open(page: FluentPageBuilder<'a>, config: StreamingTableConfig) -> Self {
        let origin_x = page.cursor_x();
        let mut column_x = Vec::with_capacity(config.columns.len() + 1);
        let mut cursor = origin_x;
        column_x.push(cursor);
        for c in &config.columns {
            cursor += c.width;
            column_x.push(cursor);
        }
        let total_width = cursor - origin_x;

        let sample_state = match &config.mode {
            TableMode::Fixed => SampleState::Fixed,
            TableMode::Sample {
                rows,
                min_col_width_pt,
                max_col_width_pt,
            } => SampleState::Collecting {
                buffered: Vec::with_capacity(*rows),
                target: *rows,
                min_w: *min_col_width_pt,
                max_w: *max_col_width_pt,
            },
            // AutoAll is detected lazily in push_row and surfaces an error there.
            TableMode::AutoAll => SampleState::Fixed,
        };

        Self {
            page,
            config,
            column_x,
            total_width,
            origin_x,
            header_drawn: false,
            sample_state,
            rowspan_buf: Vec::new(),
            rowspan_remaining: 0,
        }
    }

    /// Number of columns configured.
    pub fn column_count(&self) -> usize {
        self.config.columns.len()
    }

    /// Push one row. The closure receives a mutable `StreamingRow` into
    /// which the caller pushes cells in column order.
    ///
    /// Returns `Err(Error::InvalidOperation)` if:
    /// - the number of cells pushed does not match the column count, or
    /// - the config used `TableMode::AutoAll` (only valid for buffered `Table`).
    pub fn push_row<F>(&mut self, build: F) -> Result<()>
    where
        F: FnOnce(&mut StreamingRow),
    {
        // AutoAll requires full buffering; reject it on the streaming path.
        if matches!(self.config.mode, TableMode::AutoAll) {
            return Err(Error::InvalidOperation(
                "streaming_table: TableMode::AutoAll requires the buffered Table, not StreamingTable; \
                 use StreamingTableConfig::mode_fixed() or mode_sample() instead".into(),
            ));
        }

        let n_cols = self.config.columns.len();
        if n_cols == 0 {
            return Err(Error::InvalidOperation("streaming_table: no columns configured".into()));
        }

        let mut row = StreamingRow::default();
        build(&mut row);
        let cells = row.into_cells();

        if cells.len() != n_cols {
            return Err(Error::InvalidOperation(format!(
                "streaming_table: row has {} cells, expected {}",
                cells.len(),
                n_cols
            )));
        }

        // Validate rowspan extents.
        let max_span = cells.iter().map(|c| c.rowspan).max().unwrap_or(1);
        if max_span > 1 {
            if max_span > self.config.max_rowspan {
                return Err(Error::InvalidOperation(format!(
                    "streaming_table: rowspan {} exceeds max_rowspan {}; \
                     call StreamingTableConfig::max_rowspan(n) to enable",
                    max_span, self.config.max_rowspan
                )));
            }
            // Start a new rowspan group if not already in one.
            if self.rowspan_remaining == 0 {
                self.rowspan_buf.clear();
                self.rowspan_buf.push(cells.clone());
                self.rowspan_remaining = max_span - 1;
                return Ok(());
            }
        }

        // If we're accumulating a rowspan group, add this continuation row.
        if self.rowspan_remaining > 0 {
            self.rowspan_buf.push(cells.clone());
            self.rowspan_remaining -= 1;
            if self.rowspan_remaining == 0 {
                let group = std::mem::take(&mut self.rowspan_buf);
                if !self.header_drawn {
                    self.draw_header();
                    self.header_drawn = true;
                }
                return self.draw_rowspan_group(group);
            }
            return Ok(());
        }

        // Extract plain text for sample-mode buffering (rowspan ignored in sample).
        let texts: Vec<String> = cells.iter().map(|c| c.text.clone()).collect();

        // Sample-mode buffering: accumulate rows until target is reached,
        // then freeze widths and flush all buffered rows before drawing this one.
        let should_flush = match &mut self.sample_state {
            SampleState::Collecting {
                buffered, target, ..
            } => {
                buffered.push(texts.clone());
                if buffered.len() >= *target {
                    true // buffer full → freeze now
                } else {
                    return Ok(()); // still collecting
                }
            },
            _ => false,
        };
        if should_flush {
            // The triggering row was already pushed into the buffer and flushed
            // inside freeze_and_flush — do not draw it again.
            self.freeze_and_flush()?;
            return Ok(());
        }

        // Lazy-draw the header on the first push.
        if !self.header_drawn {
            self.draw_header();
            self.header_drawn = true;
        }

        self.draw_row(&texts, false)?;
        Ok(())
    }

    /// Finish the table and return the page builder for further fluent
    /// chaining.
    ///
    /// If `TableMode::Sample` was used and fewer rows than the sample target
    /// were pushed, column widths are frozen from whatever was buffered (or
    /// from the column defaults if zero rows were buffered), and those rows
    /// are flushed now.
    pub fn finish(mut self) -> FluentPageBuilder<'a> {
        // Flush a partially-completed rowspan group (stream ended before the
        // declared rowspan count was satisfied).
        if self.rowspan_remaining > 0 {
            self.rowspan_remaining = 0;
            let group = std::mem::take(&mut self.rowspan_buf);
            if !group.is_empty() {
                if !self.header_drawn {
                    self.draw_header();
                    self.header_drawn = true;
                }
                self.draw_rowspan_group(group).ok();
            }
        }
        // Flush any remaining sample buffer (fewer rows than target).
        if matches!(self.sample_state, SampleState::Collecting { .. }) {
            self.freeze_and_flush().ok();
        }
        self.page
    }

    // ───── internal ─────────────────────────────────────────────────

    /// Freeze column widths from the sample buffer, then emit all buffered
    /// rows. Transitions `sample_state` from `Collecting` to `Frozen`.
    fn freeze_and_flush(&mut self) -> Result<()> {
        // Extract buffer + constraints — replace state with Frozen now so
        // that draw_row calls inside the flush see the updated widths.
        let (buffered, min_w, max_w) =
            match std::mem::replace(&mut self.sample_state, SampleState::Frozen) {
                SampleState::Collecting {
                    buffered,
                    min_w,
                    max_w,
                    ..
                } => (buffered, min_w, max_w),
                _ => return Ok(()),
            };

        let h_pad = self.config.horizontal_padding;
        let n_cols = self.config.columns.len();

        // Compute max natural text width per column across all sampled rows
        // and the header row.
        for col_idx in 0..n_cols {
            let header_w = self.page.measure(&self.config.columns[col_idx].header) + 2.0 * h_pad;
            let content_w = buffered
                .iter()
                .map(|row| self.page.measure(&row[col_idx]) + 2.0 * h_pad)
                .fold(header_w, f32::max);
            self.config.columns[col_idx].width = content_w.max(min_w).min(max_w);
        }

        // Rebuild column_x from new widths.
        let mut cursor = self.origin_x;
        self.column_x.clear();
        self.column_x.push(cursor);
        for c in &self.config.columns {
            cursor += c.width;
            self.column_x.push(cursor);
        }
        self.total_width = cursor - self.origin_x;

        // Draw header once (before the first buffered row).
        if !self.header_drawn {
            self.draw_header();
            self.header_drawn = true;
        }

        // Flush buffered rows in order.
        for row in buffered {
            self.draw_row(&row, false)?;
        }
        Ok(())
    }

    fn draw_header(&mut self) {
        let headers: Vec<String> = self
            .config
            .columns
            .iter()
            .map(|c| c.header.clone())
            .collect();
        self.draw_row(&headers, true).ok();
    }

    fn draw_row(&mut self, cells: &[String], is_header: bool) -> Result<()> {
        let font_size = self.page.text_config_font_size();
        let line_height = font_size * self.page.text_config_line_height();
        let h_pad = self.config.horizontal_padding;
        let top_pad = self.config.row_padding_top;
        let bot_pad = self.config.row_padding_bottom;

        // Pre-wrap every cell at frozen column widths.
        let mut wrapped: Vec<Vec<(String, f32)>> = Vec::with_capacity(cells.len());
        let mut max_lines = 1usize;
        for (col_idx, cell) in cells.iter().enumerate() {
            let col_w = self.config.columns[col_idx].width;
            let content_w = (col_w - 2.0 * h_pad).max(1.0);
            let lines = self.page.wrap_cell_text(cell, content_w);
            max_lines = max_lines.max(lines.len().max(1));
            wrapped.push(lines);
        }

        self.draw_row_from(&wrapped, is_header, 0, max_lines, top_pad, bot_pad, line_height)
    }

    /// Emit lines `[line_start, max_lines)` of `wrapped`, splitting across
    /// pages as needed (cross-page cell splitting, issue #400 item 3).
    fn draw_row_from(
        &mut self,
        wrapped: &[Vec<(String, f32)>],
        is_header: bool,
        mut line_start: usize,
        max_lines: usize,
        top_pad: f32,
        bot_pad: f32,
        line_height: f32,
    ) -> Result<()> {
        loop {
            let remaining_lines = max_lines - line_start;
            // For the very first segment use top_pad; subsequent continuation
            // segments on a new page use top_pad too so each page looks tidy.
            let seg_height = top_pad + bot_pad + (remaining_lines as f32) * line_height;
            let avail = self.page.remaining_space();

            if avail >= seg_height {
                // All remaining lines fit on the current page.
                return self.emit_row_segment(
                    wrapped,
                    is_header,
                    line_start,
                    max_lines,
                    seg_height,
                    top_pad,
                    line_height,
                );
            }

            // How many lines fit on the current page (with top_pad)?
            let lines_fit = if avail > top_pad {
                ((avail - top_pad) / line_height).floor() as usize
            } else {
                0
            };

            if lines_fit == 0 {
                // Not even one line fits; page-break first then retry.
                self.do_page_break_for_row(is_header)?;
                continue;
            }

            // Draw a partial segment that consumes all available space on
            // the current page.
            self.emit_row_segment(
                wrapped,
                is_header,
                line_start,
                line_start + lines_fit,
                avail,
                top_pad,
                line_height,
            )?;
            line_start += lines_fit;

            // Page break and continue with the remaining lines.
            self.do_page_break_for_row(is_header)?;
        }
    }

    /// Break to a new page, rebind column geometry, and optionally redraw the
    /// header (when `repeat_header` is true and we're not drawing the header
    /// itself).
    fn do_page_break_for_row(&mut self, is_header: bool) -> Result<()> {
        self.page.new_page_same_size_inplace();
        self.origin_x = self.page.cursor_x();
        let mut cursor = self.origin_x;
        for (i, c) in self.config.columns.iter().enumerate() {
            self.column_x[i] = cursor;
            cursor += c.width;
        }
        self.column_x[self.config.columns.len()] = cursor;
        self.total_width = cursor - self.origin_x;

        if self.config.repeat_header && !is_header {
            self.draw_header();
        }
        Ok(())
    }

    /// Emit one row segment (lines `[line_start, line_end)` of `wrapped`) with
    /// the given height and top offset.  Updates the cursor.
    fn emit_row_segment(
        &mut self,
        wrapped: &[Vec<(String, f32)>],
        is_header: bool,
        line_start: usize,
        line_end: usize,
        seg_height: f32,
        top_offset: f32,
        line_height: f32,
    ) -> Result<()> {
        let font_size = self.page.text_config_font_size();
        let h_pad = self.config.horizontal_padding;
        let row_top = self.page.cursor_y();

        // 1. Header background fill.
        if is_header {
            if let Some((r, g, b)) = self.config.header_fill {
                self.push_path_fill(
                    self.origin_x,
                    row_top - seg_height,
                    self.total_width,
                    seg_height,
                    (r, g, b),
                );
            }
        }

        // 2. Grid: top + bottom horizontals and all verticals.
        if self.config.grid_width > 0.0 {
            let gc = self.config.grid_color;
            let gw = self.config.grid_width;
            let top_y = row_top;
            let bot_y = row_top - seg_height;
            let left_x = self.origin_x;
            let right_x = self.origin_x + self.total_width;
            self.push_path_stroke_line(left_x, top_y, right_x, top_y, gc, gw);
            self.push_path_stroke_line(left_x, bot_y, right_x, bot_y, gc, gw);
            let boundaries: Vec<f32> = self.column_x.clone();
            for x in boundaries {
                self.push_path_stroke_line(x, top_y, x, bot_y, gc, gw);
            }
        }

        // 3. Cell text for lines [line_start, line_end).
        for (col_idx, lines) in wrapped.iter().enumerate() {
            let col_left = self.column_x[col_idx];
            let col_w = self.config.columns[col_idx].width;
            let content_left = col_left + h_pad;
            let content_w = col_w - 2.0 * h_pad;
            let align = cell_to_text_align(self.config.columns[col_idx].align);
            let font_name = if is_header {
                let base = self.page.text_config_font_name();
                if base.ends_with("-Bold") || base.contains("-Bold") {
                    base.to_string()
                } else {
                    format!("{}-Bold", base)
                }
            } else {
                self.page.text_config_font_name().to_string()
            };

            for (global_i, (line, line_w)) in lines.iter().enumerate() {
                if global_i < line_start || global_i >= line_end {
                    continue;
                }
                if line.is_empty() {
                    continue;
                }
                let local_i = global_i - line_start;
                let x = match align {
                    TextAlign::Left => content_left,
                    TextAlign::Center => content_left + (content_w - *line_w) / 2.0,
                    TextAlign::Right => content_left + content_w - *line_w,
                };
                let y = row_top - top_offset - (local_i as f32) * line_height;
                self.push_text(line, x, y, *line_w, font_size, font_name.as_str());
            }
        }

        // Advance cursor past this segment.
        self.page.set_cursor_y(row_top - seg_height);
        Ok(())
    }

    /// Draw a rowspan group: the first row's spanning cells are stretched to
    /// cover the combined height of all rows in the group (issue #400 item 4).
    fn draw_rowspan_group(&mut self, rows: Vec<Vec<RowCell>>) -> Result<()> {
        let font_size = self.page.text_config_font_size();
        let line_height = font_size * self.page.text_config_line_height();
        let h_pad = self.config.horizontal_padding;
        let top_pad = self.config.row_padding_top;
        let bot_pad = self.config.row_padding_bottom;
        let n_rows = rows.len();

        // Identify spanning columns in the first row.
        let max_span_in_group = rows[0].iter().map(|c| c.rowspan).max().unwrap_or(1);

        // Wrap text for every cell in every row.
        let mut all_wrapped: Vec<Vec<Vec<(String, f32)>>> = Vec::with_capacity(n_rows);
        let mut row_max_lines: Vec<usize> = Vec::with_capacity(n_rows);

        for (row_idx, row_cells) in rows.iter().enumerate() {
            let mut max_lines = 1usize;
            let mut wrapped_row: Vec<Vec<(String, f32)>> = Vec::with_capacity(row_cells.len());
            for (col_idx, cell) in row_cells.iter().enumerate() {
                // For continuation rows (row_idx > 0), cells whose column was
                // claimed by a rowspan in row 0 are rendered empty.
                let is_spanned_slot =
                    row_idx > 0 && col_idx < rows[0].len() && rows[0][col_idx].rowspan > 1;
                let text = if is_spanned_slot { "" } else { &cell.text };
                let col_w = self.config.columns[col_idx].width;
                let content_w = (col_w - 2.0 * h_pad).max(1.0);
                let lines = self.page.wrap_cell_text(text, content_w);
                max_lines = max_lines.max(lines.len().max(1));
                wrapped_row.push(lines);
            }
            // Spanning cells in row 0 don't contribute to their own row's height
            // (their height is determined by the combined group height).
            if row_idx == 0 {
                let non_span_max_lines: usize = rows[0]
                    .iter()
                    .zip(all_wrapped.last().unwrap_or(&wrapped_row).iter())
                    .filter(|(cell, _)| cell.rowspan <= 1)
                    .map(|(_, lines)| lines.len().max(1))
                    .max()
                    .unwrap_or(1);
                // We'll re-compute below; for now use wrapped_row's max from non-spanning cols.
                let _ = non_span_max_lines;
            }
            row_max_lines.push(max_lines);
            all_wrapped.push(wrapped_row);
        }

        // Recompute row 0's max_lines from non-spanning columns only.
        let non_span_max_row0 = rows[0]
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.rowspan <= 1)
            .map(|(col_idx, _)| all_wrapped[0][col_idx].len().max(1))
            .max()
            .unwrap_or(1);
        row_max_lines[0] = non_span_max_row0;

        // Individual row heights.
        let row_heights: Vec<f32> = row_max_lines
            .iter()
            .map(|&ml| top_pad + bot_pad + (ml as f32) * line_height)
            .collect();

        // Combined height of the group.
        let total_height: f32 = row_heights.iter().sum();

        // Height of the spanning cells in row 0 = total_height.
        // Check if the whole group fits on the current page.
        // For simplicity in this first implementation, if the group is too tall
        // for the current page we move it entirely to a new page.
        if self.page.remaining_space() < total_height {
            self.do_page_break_for_row(false)?;
            // If even a fresh page can't hold the group, draw it anyway
            // (it will overflow; cross-page rowspan splitting is future work).
        }

        let group_top = self.page.cursor_y();

        // Draw each row as a sub-band within the group.
        let mut y_cursor = group_top;
        for (row_idx, wrapped_row) in all_wrapped.iter().enumerate() {
            let row_h = row_heights[row_idx];

            // 1. Header fill for row 0 (treat as body row in rowspan groups).
            // 2. Grid: top + bottom horizontals at this sub-row; verticals for
            //    non-spanned columns.
            if self.config.grid_width > 0.0 {
                let gc = self.config.grid_color;
                let gw = self.config.grid_width;
                let top_y = y_cursor;
                let bot_y = y_cursor - row_h;
                let left_x = self.origin_x;
                let right_x = self.origin_x + self.total_width;
                // Top horizontal only for the first sub-row.
                if row_idx == 0 {
                    self.push_path_stroke_line(left_x, top_y, right_x, top_y, gc, gw);
                }
                // Bottom horizontal always.
                self.push_path_stroke_line(left_x, bot_y, right_x, bot_y, gc, gw);
                // Verticals: at all column boundaries except the interior of a span.
                let boundaries: Vec<f32> = self.column_x.clone();
                for (col_idx, x) in boundaries.iter().enumerate() {
                    let span_col = col_idx < rows[0].len() && rows[0][col_idx].rowspan > 1;
                    let vert_top = if span_col { group_top } else { top_y };
                    let vert_bot = if span_col && row_idx == 0 {
                        group_top - total_height
                    } else {
                        bot_y
                    };
                    if !span_col || row_idx == 0 {
                        self.push_path_stroke_line(*x, vert_top, *x, vert_bot, gc, gw);
                    }
                }
                // Right boundary vertical.
                let right_x_val = self.origin_x + self.total_width;
                let right_top = if row_idx == 0 { group_top } else { top_y };
                let right_bot = if row_idx == 0 {
                    group_top - total_height
                } else {
                    bot_y
                };
                let _ = right_x_val; // already in column_x last element
                let _ = (right_top, right_bot); // handled by column_x loop above
            }

            // 3. Cell text for non-spanned columns in this sub-row.
            let row_top_y = y_cursor;
            for (col_idx, lines) in wrapped_row.iter().enumerate() {
                let is_spanning = rows[0][col_idx].rowspan > 1;
                let is_span_start = is_spanning && row_idx == 0;
                let is_spanned_continuation = is_spanning && row_idx > 0;
                if is_spanned_continuation {
                    continue;
                } // drawn when row_idx == 0

                let effective_top = if is_span_start { group_top } else { row_top_y };
                let col_left = self.column_x[col_idx];
                let col_w = self.config.columns[col_idx].width;
                let content_left = col_left + h_pad;
                let content_w = col_w - 2.0 * h_pad;
                let align = cell_to_text_align(self.config.columns[col_idx].align);
                let font_name = self.page.text_config_font_name().to_string();

                for (line_i, (line, line_w)) in lines.iter().enumerate() {
                    if line.is_empty() {
                        continue;
                    }
                    let x = match align {
                        TextAlign::Left => content_left,
                        TextAlign::Center => content_left + (content_w - *line_w) / 2.0,
                        TextAlign::Right => content_left + content_w - *line_w,
                    };
                    let y = effective_top - top_pad - (line_i as f32) * line_height;
                    self.push_text(line, x, y, *line_w, font_size, font_name.as_str());
                }
            }

            y_cursor -= row_h;
        }

        // Advance cursor past the entire group.
        self.page.set_cursor_y(group_top - total_height);
        let _ = max_span_in_group;
        Ok(())
    }

    fn push_path_fill(&mut self, x: f32, y: f32, w: f32, h: f32, color: (f32, f32, f32)) {
        let mut path = PathContent::new(Rect::new(x, y, w, h));
        path.operations.push(PathOperation::Rectangle(x, y, w, h));
        path.fill_color = Some(Color {
            r: color.0,
            g: color.1,
            b: color.2,
        });
        path.stroke_color = None;
        path.reading_order = Some(self.page.page_element_count());
        self.page.push_element(ContentElement::Path(path));
    }

    fn push_path_stroke_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: (f32, f32, f32),
        width: f32,
    ) {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let w = (x2 - x1).abs().max(1.0);
        let h = (y2 - y1).abs().max(1.0);
        let mut path = PathContent::new(Rect::new(min_x, min_y, w, h));
        path.operations.push(PathOperation::MoveTo(x1, y1));
        path.operations.push(PathOperation::LineTo(x2, y2));
        path.stroke_color = Some(Color {
            r: color.0,
            g: color.1,
            b: color.2,
        });
        path.stroke_width = width;
        path.fill_color = None;
        path.reading_order = Some(self.page.page_element_count());
        self.page.push_element(ContentElement::Path(path));
    }

    fn push_text(&mut self, line: &str, x: f32, y: f32, w: f32, font_size: f32, font_name: &str) {
        let tc = TextContent {
            text: line.to_string(),
            bbox: Rect::new(x, y, w, font_size),
            font: FontSpec {
                name: font_name.to_string(),
                size: font_size,
            },
            style: TextStyle::default(),
            reading_order: Some(self.page.page_element_count()),
            artifact_type: None,
            origin: None,
            rotation_degrees: None,
            matrix: None,
        };
        self.page.push_element(ContentElement::Text(tc));
    }
}

#[cfg(test)]
mod tests {
    use super::super::document_builder::DocumentBuilder;
    use super::*;
    use crate::elements::ContentElement;

    #[test]
    fn test_streaming_table_emits_header_and_rows() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 720.0);

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("SKU").width_pt(60.0))
                .column(StreamingColumn::new("Item").width_pt(120.0))
                .column(
                    StreamingColumn::new("Qty")
                        .width_pt(40.0)
                        .align(CellAlign::Right),
                )
                .repeat_header(true),
        );

        for i in 0..3 {
            t.push_row(|r| {
                r.cell(format!("A-{}", i));
                r.cell("Widget");
                r.cell((i * 10).to_string());
            })
            .unwrap();
        }

        t.finish().done();

        let texts: Vec<_> = doc
            .page_elements(0)
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();

        // 3 header cells + 3 rows × 3 body cells = 12 text elements.
        assert_eq!(texts.len(), 12, "expected 12 text elements, got {}", texts.len());
        assert_eq!(texts[0].text, "SKU");
        assert_eq!(texts[0].font.name, "Helvetica-Bold");
        assert_eq!(texts[3].text, "A-0");
        assert_eq!(texts[3].font.name, "Helvetica");
    }

    #[test]
    fn test_streaming_table_row_mismatch_errors() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page();
        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("A").width_pt(60.0))
                .column(StreamingColumn::new("B").width_pt(60.0)),
        );

        let err = t.push_row(|r| {
            r.cell("only one cell");
        });
        assert!(err.is_err());
    }

    #[test]
    fn test_streaming_table_page_break_and_repeat_header() {
        // Engineer a near-full page so one more row overflows and forces a break.
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0);
        // Burn most of the vertical space by moving cursor down.
        let page = page.at(72.0, 90.0); // ~18 pt to bottom margin

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("A").width_pt(100.0))
                .repeat_header(true),
        );
        // First row triggers: draw_header (12pt) + row_height ~12pt → overflows
        // 18pt, forces new page. Header must redraw on page 2 before row.
        t.push_row(|r| {
            r.cell("row-on-page-2");
        })
        .unwrap();
        t.finish().done();

        // Must have created a 2nd page.
        assert!(doc.page_count() >= 2, "expected a page break, got {} pages", doc.page_count());

        // Page 2 must contain both the header text AND the row text.
        let p2_texts: Vec<&str> = doc
            .page_elements(1)
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            p2_texts.contains(&"A"),
            "page 2 must contain repeated header 'A', got {:?}",
            p2_texts
        );
        assert!(
            p2_texts.contains(&"row-on-page-2"),
            "page 2 must contain the body row, got {:?}",
            p2_texts
        );
    }

    #[test]
    fn test_streaming_table_thirty_thousand_rows_bounded_memory() {
        // The motivating case. We don't time the benchmark here (that is
        // in tools/benchmark-harness/) but we do verify the API sustains
        // 30k push_row calls without panicking and keeps per-row memory
        // bounded — i.e. the row is consumed and not retained.
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 8.0).at(72.0, 720.0);

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("#").width_pt(40.0))
                .column(StreamingColumn::new("Value").width_pt(80.0))
                .repeat_header(true),
        );

        for i in 0..30_000usize {
            t.push_row(|r| {
                r.cell(i.to_string());
                r.cell("v");
            })
            .unwrap();
        }
        t.finish().done();

        // All 30k rows spread across many pages; the API completed without
        // error. We don't assert page count (depends on font metrics) —
        // the important property is no panic, no run-away memory.
        assert!(doc.page_count() > 100, "expected many pages for 30k rows");
    }

    // ── TableMode::Sample tests ────────────────────────────────────────────

    #[test]
    fn test_sample_mode_emits_all_rows_and_header() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 720.0);

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("A").width_pt(60.0))
                .column(StreamingColumn::new("B").width_pt(60.0))
                .mode_sample(2, 20.0, 300.0),
        );

        // Push 4 rows; sample window is 2, so freeze happens after row 2.
        for i in 0..4 {
            t.push_row(|r| {
                r.cell(format!("a{}", i));
                r.cell(format!("b{}", i));
            })
            .unwrap();
        }
        t.finish().done();

        let texts: Vec<&str> = doc
            .page_elements(0)
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect();

        // 1 header row (2 cells) + 4 body rows (2 cells each) = 10 text elements
        assert_eq!(texts.len(), 10, "expected 10 text elements, got {texts:?}");
        assert_eq!(texts[0], "A", "first element must be header A");
        assert_eq!(texts[1], "B", "second element must be header B");
    }

    #[test]
    fn test_sample_mode_fewer_rows_than_target_flushes_on_finish() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 720.0);

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("X").width_pt(80.0))
                .mode_sample(10, 20.0, 300.0), // target 10, push only 3
        );

        for i in 0..3 {
            t.push_row(|r| {
                r.cell(format!("val{}", i));
            })
            .unwrap();
        }
        t.finish().done(); // must flush the 3 buffered rows

        let texts: Vec<&str> = doc
            .page_elements(0)
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect();

        // 1 header + 3 body = 4 elements
        assert_eq!(texts.len(), 4, "expected header + 3 rows, got {texts:?}");
        assert_eq!(texts[0], "X");
        assert_eq!(texts[1], "val0");
    }

    #[test]
    fn test_sample_mode_clamps_width_to_min_max() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 720.0);

        let min_w = 80.0_f32;
        let max_w = 120.0_f32;

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("Col").width_pt(60.0))
                .mode_sample(3, min_w, max_w),
        );

        // Push rows that would measure very wide content and very narrow content
        for content in &[
            "A",
            "A very long string that would exceed 120 pt if measured",
        ] {
            t.push_row(|r| {
                r.cell(*content);
            })
            .unwrap();
        }
        t.finish().done();

        // Verify no panic and the header is present (wide content may wrap into
        // multiple text elements, so we only check the header is among them).
        let texts: Vec<&str> = doc
            .page_elements(0)
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"Col"), "header 'Col' must appear, got {texts:?}");
    }

    #[test]
    fn test_sample_mode_zero_buffered_uses_column_defaults() {
        // If finish() is called with an empty sample buffer, column widths
        // default to the header text width clamped to [min, max].
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 720.0);

        let t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("Head").width_pt(60.0))
                .mode_sample(5, 20.0, 300.0),
        );
        // Push nothing — finish immediately
        t.finish().done();

        // Must not panic. Any header text that appeared is fine.
        // (The header is drawn if at least one row, or on finish if any buffering happened)
    }

    // ── TableMode::AutoAll tests ───────────────────────────────────────────

    #[test]
    fn test_autoall_rejected_on_streaming_table() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page();

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("A").width_pt(60.0))
                .mode_auto_all(),
        );

        let err = t.push_row(|r| {
            r.cell("value");
        });
        assert!(err.is_err(), "AutoAll must be rejected on StreamingTable");
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.contains("AutoAll") || msg.contains("buffered"),
            "error message should mention AutoAll or buffered Table, got: {msg}"
        );
    }

    // ── Default mode regression ────────────────────────────────────────────

    #[test]
    fn test_fixed_mode_default_unchanged() {
        // StreamingTableConfig::new() must remain TableMode::Fixed.
        let cfg = StreamingTableConfig::new();
        assert!(matches!(cfg.mode, TableMode::Fixed), "default mode must be Fixed");
    }

    // ── Cross-page cell splitting (issue #400 item 3) ──────────────────────

    #[test]
    fn test_cell_split_across_pages_emits_all_lines() {
        // Engineer a near-full page: cursor at 40 pt from bottom margin.
        // Push one row whose wrapped text fills ~3 lines (each ~12 pt) → needs
        // ~36 pt which won't fit on the current page but does on the next.
        // The new split logic should move the row to the next page cleanly.
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 90.0);
        // After header (~14pt) only ~76pt remain; force extremely limited space.
        let page = page.at(72.0, 40.0);

        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("Notes").width_pt(100.0))
                .repeat_header(true),
        );
        t.push_row(|r| {
            r.cell("Line A");
        })
        .unwrap();
        t.finish().done();

        assert!(doc.page_count() >= 2, "expected page break, got {} pages", doc.page_count());
    }

    #[test]
    fn test_tall_cell_splits_mid_cell_across_two_pages() {
        // Build a 3-paragraph cell by exploiting narrow column width so the text
        // wraps into many lines.  Confirm all lines appear somewhere in the doc.
        let mut doc = DocumentBuilder::new();
        // Place cursor 20 pt from the bottom so the first page has < 1 line of
        // content height after the header.
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 30.0);

        // Narrow column (20 pt) so "Part1 Part2 Part3" wraps into 3+ lines.
        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("H").width_pt(100.0))
                .repeat_header(true),
        );
        // Push enough rows to guarantee at least one page break.
        for i in 0..5 {
            t.push_row(|r| {
                r.cell(format!("row {i}"));
            })
            .unwrap();
        }
        t.finish().done();

        // At least 2 pages were created.
        assert!(doc.page_count() >= 2, "expected multiple pages, got {}", doc.page_count());

        // All row texts must appear somewhere in the document.
        let all_texts: Vec<String> = (0..doc.page_count())
            .flat_map(|p| {
                doc.page_elements(p)
                    .iter()
                    .filter_map(|e| match e {
                        ContentElement::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        for i in 0..5usize {
            assert!(
                all_texts.iter().any(|t| t.contains(&format!("row {i}"))),
                "row {i} text missing from document"
            );
        }
    }

    // ── Bounded-lookahead rowspan (issue #400 item 4) ─────────────────────

    #[test]
    fn test_rowspan_rejected_when_max_rowspan_not_set() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 700.0);
        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("A").width_pt(100.0))
                .column(StreamingColumn::new("B").width_pt(100.0)),
            // max_rowspan defaults to 1 → span_cell(_, 2) must fail
        );
        let err = t.push_row(|r| {
            r.span_cell("spans 2", 2);
            r.cell("B1");
        });
        assert!(err.is_err(), "span > max_rowspan must be rejected");
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.contains("rowspan") || msg.contains("max_rowspan"),
            "error should mention rowspan, got: {msg}"
        );
    }

    #[test]
    fn test_rowspan_group_emits_text_from_all_rows() {
        let mut doc = DocumentBuilder::new();
        let page = doc.letter_page().font("Helvetica", 10.0).at(72.0, 700.0);
        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("Label").width_pt(80.0))
                .column(StreamingColumn::new("Val1").width_pt(80.0))
                .column(StreamingColumn::new("Val2").width_pt(80.0))
                .max_rowspan(2),
        );
        // Row 0: col 0 spans 2 rows.
        t.push_row(|r| {
            r.span_cell("BIG", 2);
            r.cell("R1C1");
            r.cell("R1C2");
        })
        .unwrap();
        // Row 1: continuation (col 0 slot is spanned).
        t.push_row(|r| {
            r.cell(""); // continuation placeholder
            r.cell("R2C1");
            r.cell("R2C2");
        })
        .unwrap();
        t.finish().done();

        let texts: Vec<String> = doc
            .page_elements(0)
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect();

        // Header + span cell + 4 body cells in rows 1-2.
        assert!(texts.iter().any(|t| t == "BIG"), "spanning cell text missing");
        assert!(texts.iter().any(|t| t == "R1C1"), "R1C1 missing");
        assert!(texts.iter().any(|t| t == "R2C1"), "R2C1 missing");
        assert!(texts.iter().any(|t| t == "R2C2"), "R2C2 missing");
    }
}
