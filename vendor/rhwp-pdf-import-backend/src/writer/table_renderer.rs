//! Table rendering for PDF generation.
//!
//! This module provides support for creating tables in PDF documents,
//! including layout calculation, borders, backgrounds, and text wrapping.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{Table, TableCell, TableStyle};
//!
//! let table = Table::new(vec![
//!     vec![TableCell::text("Name"), TableCell::text("Age")],
//!     vec![TableCell::text("Alice"), TableCell::text("30")],
//!     vec![TableCell::text("Bob"), TableCell::text("25")],
//! ])
//! .with_header_row()
//! .with_borders(BorderStyle::all(0.5));
//! ```

use super::content_stream::ContentStreamBuilder;
use crate::error::Result;

/// Horizontal alignment for cell content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CellAlign {
    /// Align to the left
    #[default]
    Left,
    /// Center horizontally
    Center,
    /// Align to the right
    Right,
}

/// Vertical alignment for cell content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CellVAlign {
    /// Align to the top
    #[default]
    Top,
    /// Center vertically
    Middle,
    /// Align to the bottom
    Bottom,
}

/// Column width specification.
#[derive(Debug, Clone, Copy, Default)]
pub enum ColumnWidth {
    /// Automatic width based on content
    #[default]
    Auto,
    /// Fixed width in points
    Fixed(f32),
    /// Percentage of table width
    Percent(f32),
    /// Proportional weight (flex)
    Weight(f32),
}

/// Border style for tables.
#[derive(Debug, Clone, Copy)]
pub struct TableBorderStyle {
    /// Border width in points
    pub width: f32,
    /// Border color (RGB, 0.0-1.0)
    pub color: (f32, f32, f32),
}

impl Default for TableBorderStyle {
    fn default() -> Self {
        Self {
            width: 0.5,
            color: (0.0, 0.0, 0.0), // Black
        }
    }
}

impl TableBorderStyle {
    /// Create a new border style.
    pub fn new(width: f32) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    /// Create a border with specific color.
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = (r, g, b);
        self
    }

    /// Create a thin border (0.25pt).
    pub fn thin() -> Self {
        Self::new(0.25)
    }

    /// Create a medium border (0.5pt).
    pub fn medium() -> Self {
        Self::new(0.5)
    }

    /// Create a thick border (1.0pt).
    pub fn thick() -> Self {
        Self::new(1.0)
    }

    /// No border.
    pub fn none() -> Self {
        Self::new(0.0)
    }
}

/// Border configuration for a cell or table.
#[derive(Debug, Clone, Copy, Default)]
pub struct Borders {
    /// Top border
    pub top: Option<TableBorderStyle>,
    /// Right border
    pub right: Option<TableBorderStyle>,
    /// Bottom border
    pub bottom: Option<TableBorderStyle>,
    /// Left border
    pub left: Option<TableBorderStyle>,
}

impl Borders {
    /// No borders.
    pub fn none() -> Self {
        Self::default()
    }

    /// All borders with the same style.
    pub fn all(style: TableBorderStyle) -> Self {
        Self {
            top: Some(style),
            right: Some(style),
            bottom: Some(style),
            left: Some(style),
        }
    }

    /// Horizontal borders only (top and bottom).
    pub fn horizontal(style: TableBorderStyle) -> Self {
        Self {
            top: Some(style),
            bottom: Some(style),
            ..Default::default()
        }
    }

    /// Vertical borders only (left and right).
    pub fn vertical(style: TableBorderStyle) -> Self {
        Self {
            left: Some(style),
            right: Some(style),
            ..Default::default()
        }
    }

    /// Set top border.
    pub fn with_top(mut self, style: TableBorderStyle) -> Self {
        self.top = Some(style);
        self
    }

    /// Set bottom border.
    pub fn with_bottom(mut self, style: TableBorderStyle) -> Self {
        self.bottom = Some(style);
        self
    }

    /// Set left border.
    pub fn with_left(mut self, style: TableBorderStyle) -> Self {
        self.left = Some(style);
        self
    }

    /// Set right border.
    pub fn with_right(mut self, style: TableBorderStyle) -> Self {
        self.right = Some(style);
        self
    }
}

/// Cell padding configuration.
#[derive(Debug, Clone, Copy)]
pub struct CellPadding {
    /// Top padding in points
    pub top: f32,
    /// Right padding in points
    pub right: f32,
    /// Bottom padding in points
    pub bottom: f32,
    /// Left padding in points
    pub left: f32,
}

impl Default for CellPadding {
    fn default() -> Self {
        Self {
            top: 4.0,
            right: 4.0,
            bottom: 4.0,
            left: 4.0,
        }
    }
}

impl CellPadding {
    /// Create uniform padding.
    pub fn uniform(padding: f32) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    /// Create padding with horizontal and vertical values.
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// No padding.
    pub fn none() -> Self {
        Self::uniform(0.0)
    }

    /// Total horizontal padding.
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical padding.
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// A single table cell.
#[derive(Debug, Clone)]
pub struct TableCell {
    /// Cell content (text)
    pub content: String,
    /// Number of columns this cell spans
    pub colspan: usize,
    /// Number of rows this cell spans
    pub rowspan: usize,
    /// Horizontal alignment
    pub align: CellAlign,
    /// Vertical alignment
    pub valign: CellVAlign,
    /// Cell-specific padding (overrides table default)
    pub padding: Option<CellPadding>,
    /// Cell-specific borders (overrides table default)
    pub borders: Option<Borders>,
    /// Background color (RGB, 0.0-1.0)
    pub background: Option<(f32, f32, f32)>,
    /// Font name override
    pub font_name: Option<String>,
    /// Font size override
    pub font_size: Option<f32>,
    /// Bold text
    pub bold: bool,
    /// Italic text
    pub italic: bool,
}

impl TableCell {
    /// Create a new text cell.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            colspan: 1,
            rowspan: 1,
            align: CellAlign::default(),
            valign: CellVAlign::default(),
            padding: None,
            borders: None,
            background: None,
            font_name: None,
            font_size: None,
            bold: false,
            italic: false,
        }
    }

    /// Create an empty cell.
    pub fn empty() -> Self {
        Self::text("")
    }

    /// Set column span.
    pub fn colspan(mut self, span: usize) -> Self {
        self.colspan = span.max(1);
        self
    }

    /// Set row span.
    pub fn rowspan(mut self, span: usize) -> Self {
        self.rowspan = span.max(1);
        self
    }

    /// Set horizontal alignment.
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }

    /// Set vertical alignment.
    pub fn valign(mut self, valign: CellVAlign) -> Self {
        self.valign = valign;
        self
    }

    /// Set cell padding.
    pub fn padding(mut self, padding: CellPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set cell borders.
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = Some(borders);
        self
    }

    /// Set background color.
    pub fn background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background = Some((r, g, b));
        self
    }

    /// Set font.
    pub fn font(mut self, name: impl Into<String>, size: f32) -> Self {
        self.font_name = Some(name.into());
        self.font_size = Some(size);
        self
    }

    /// Set bold style.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set italic style.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Create a header cell (centered, bold).
    pub fn header(content: impl Into<String>) -> Self {
        Self::text(content).align(CellAlign::Center).bold()
    }

    /// Create a numeric cell (right-aligned).
    pub fn number(content: impl Into<String>) -> Self {
        Self::text(content).align(CellAlign::Right)
    }
}

/// A table row.
#[derive(Debug, Clone)]
pub struct TableRow {
    /// Cells in this row
    pub cells: Vec<TableCell>,
    /// Minimum row height
    pub min_height: Option<f32>,
    /// Row background color (applied to all cells without explicit background)
    pub background: Option<(f32, f32, f32)>,
    /// Whether this is a header row
    pub is_header: bool,
}

impl TableRow {
    /// Create a new row from cells.
    pub fn new(cells: Vec<TableCell>) -> Self {
        Self {
            cells,
            min_height: None,
            background: None,
            is_header: false,
        }
    }

    /// Create a header row.
    pub fn header(cells: Vec<TableCell>) -> Self {
        Self {
            cells,
            min_height: None,
            background: None,
            is_header: true,
        }
    }

    /// Set minimum height.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Set row background.
    pub fn background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background = Some((r, g, b));
        self
    }

    /// Mark as header row.
    pub fn as_header(mut self) -> Self {
        self.is_header = true;
        self
    }
}

/// Table style configuration.
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// Default cell padding
    pub cell_padding: CellPadding,
    /// Default cell borders
    pub cell_borders: Borders,
    /// Table outer border
    pub outer_border: Option<TableBorderStyle>,
    /// Default font name
    pub font_name: String,
    /// Default font size
    pub font_size: f32,
    /// Header row background color
    pub header_background: Option<(f32, f32, f32)>,
    /// Alternating row colors (even rows)
    pub stripe_color: Option<(f32, f32, f32)>,
    /// Space between rows
    pub row_spacing: f32,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            cell_padding: CellPadding::default(),
            cell_borders: Borders::all(TableBorderStyle::thin()),
            outer_border: None,
            font_name: "Helvetica".to_string(),
            font_size: 10.0,
            header_background: Some((0.9, 0.9, 0.9)), // Light gray
            stripe_color: None,
            row_spacing: 0.0,
        }
    }
}

impl TableStyle {
    /// Create a new default style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set cell padding.
    pub fn cell_padding(mut self, padding: CellPadding) -> Self {
        self.cell_padding = padding;
        self
    }

    /// Set cell borders.
    pub fn cell_borders(mut self, borders: Borders) -> Self {
        self.cell_borders = borders;
        self
    }

    /// Set outer border.
    pub fn outer_border(mut self, border: TableBorderStyle) -> Self {
        self.outer_border = Some(border);
        self
    }

    /// Set default font.
    pub fn font(mut self, name: impl Into<String>, size: f32) -> Self {
        self.font_name = name.into();
        self.font_size = size;
        self
    }

    /// Set header background color.
    pub fn header_background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.header_background = Some((r, g, b));
        self
    }

    /// Enable striped rows.
    pub fn striped(mut self, r: f32, g: f32, b: f32) -> Self {
        self.stripe_color = Some((r, g, b));
        self
    }

    /// Create a minimal style (no borders).
    pub fn minimal() -> Self {
        Self {
            cell_borders: Borders::none(),
            outer_border: None,
            header_background: None,
            ..Default::default()
        }
    }

    /// Create a bordered style.
    pub fn bordered() -> Self {
        Self {
            cell_borders: Borders::all(TableBorderStyle::medium()),
            outer_border: Some(TableBorderStyle::thick()),
            ..Default::default()
        }
    }
}

/// A complete table.
#[derive(Debug, Clone)]
pub struct Table {
    /// Table rows
    pub rows: Vec<TableRow>,
    /// Column widths
    pub column_widths: Vec<ColumnWidth>,
    /// Table style
    pub style: TableStyle,
    /// Total table width (None = auto)
    pub width: Option<f32>,
    /// Column alignments (default for cells in column)
    pub column_aligns: Vec<CellAlign>,
}

impl Table {
    /// Create a new table from rows of cells.
    pub fn new(rows: Vec<Vec<TableCell>>) -> Self {
        let rows: Vec<TableRow> = rows.into_iter().map(TableRow::new).collect();
        Self::from_rows(rows)
    }

    /// Create a table from TableRow objects.
    pub fn from_rows(rows: Vec<TableRow>) -> Self {
        let num_cols = rows
            .iter()
            .map(|r| r.cells.iter().map(|c| c.colspan).sum::<usize>())
            .max()
            .unwrap_or(0);

        Self {
            rows,
            column_widths: vec![ColumnWidth::Auto; num_cols],
            style: TableStyle::default(),
            width: None,
            column_aligns: vec![CellAlign::Left; num_cols],
        }
    }

    /// Create an empty table.
    pub fn empty() -> Self {
        Self {
            rows: Vec::new(),
            column_widths: Vec::new(),
            style: TableStyle::default(),
            width: None,
            column_aligns: Vec::new(),
        }
    }

    /// Add a row to the table.
    pub fn add_row(&mut self, row: TableRow) {
        self.rows.push(row);
    }

    /// Set the first row as header.
    pub fn with_header_row(mut self) -> Self {
        if let Some(row) = self.rows.first_mut() {
            row.is_header = true;
        }
        self
    }

    /// Set table style.
    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Set total table width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set column widths.
    pub fn with_column_widths(mut self, widths: Vec<ColumnWidth>) -> Self {
        self.column_widths = widths;
        self
    }

    /// Set column alignments.
    pub fn with_column_aligns(mut self, aligns: Vec<CellAlign>) -> Self {
        self.column_aligns = aligns;
        self
    }

    /// Get the number of columns.
    pub fn num_columns(&self) -> usize {
        self.rows
            .iter()
            .map(|r| r.cells.iter().map(|c| c.colspan).sum::<usize>())
            .max()
            .unwrap_or(0)
    }

    /// Get the number of rows.
    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Calculated layout for a table.
#[derive(Debug, Clone)]
pub struct TableLayout {
    /// Calculated column widths in points
    pub column_widths: Vec<f32>,
    /// Calculated row heights in points
    pub row_heights: Vec<f32>,
    /// Total table width
    pub total_width: f32,
    /// Total table height
    pub total_height: f32,
    /// Cell positions (row, col) -> (x, y, width, height)
    pub cell_positions: Vec<Vec<CellPosition>>,
    /// Wrapped lines and per-line widths for each cell, parallel to
    /// `cell_positions`. Consumed by `render` to emit one text op per line.
    pub cell_layouts: Vec<Vec<CellLayout>>,
}

/// Position and size of a cell.
#[derive(Debug, Clone, Copy)]
pub struct CellPosition {
    /// X position (left edge)
    pub x: f32,
    /// Y position (top edge, relative to table top)
    pub y: f32,
    /// Cell width
    pub width: f32,
    /// Cell height
    pub height: f32,
}

/// Pre-computed text layout for a single cell: the wrapped lines and each
/// line's measured width. Produced once by `calculate_layout` so `render`
/// can emit per-line text with correct alignment without needing font
/// metrics at render time.
#[derive(Debug, Clone, Default)]
pub struct CellLayout {
    /// Content wrapped to the cell's content width.
    pub lines: Vec<String>,
    /// Measured width of each corresponding line, in points.
    pub line_widths: Vec<f32>,
}

impl Table {
    /// Calculate the layout for this table.
    pub fn calculate_layout(
        &self,
        available_width: f32,
        font_metrics: &dyn FontMetrics,
    ) -> TableLayout {
        let num_cols = self.num_columns();
        if num_cols == 0 || self.rows.is_empty() {
            return TableLayout {
                column_widths: vec![],
                row_heights: vec![],
                total_width: 0.0,
                total_height: 0.0,
                cell_positions: vec![],
                cell_layouts: vec![],
            };
        }

        let table_width = self.width.unwrap_or(available_width);

        // Calculate column widths
        let column_widths = self.calculate_column_widths(table_width, num_cols, font_metrics);

        // Wrap every cell once, measuring each wrapped line's width. Consumed
        // by row-height aggregation below and by `render` later.
        let cell_layouts = self.wrap_all_cells(&column_widths, font_metrics);

        // Calculate row heights from the pre-wrapped layouts
        let row_heights = self.calculate_row_heights(&cell_layouts);

        // Calculate cell positions
        let cell_positions = self.calculate_cell_positions(&column_widths, &row_heights);

        let total_width: f32 = column_widths.iter().sum();
        let total_height: f32 = row_heights.iter().sum();

        TableLayout {
            column_widths,
            row_heights,
            total_width,
            total_height,
            cell_positions,
            cell_layouts,
        }
    }

    fn calculate_column_widths(
        &self,
        table_width: f32,
        num_cols: usize,
        font_metrics: &dyn FontMetrics,
    ) -> Vec<f32> {
        let padding = &self.style.cell_padding;
        let mut widths = vec![0.0f32; num_cols];
        let mut _fixed_width = 0.0f32;
        let mut weight_total = 0.0f32;
        let mut _percent_total = 0.0f32;

        // First pass: calculate minimum widths and collect constraints
        for (col, spec) in self.column_widths.iter().take(num_cols).enumerate() {
            match spec {
                ColumnWidth::Fixed(w) => {
                    widths[col] = *w;
                    _fixed_width += *w;
                },
                ColumnWidth::Percent(p) => {
                    let w = table_width * (*p / 100.0);
                    widths[col] = w;
                    _percent_total += *p;
                },
                ColumnWidth::Weight(w) => {
                    weight_total += *w;
                },
                ColumnWidth::Auto => {
                    // Calculate based on content
                    let mut max_width = 0.0f32;
                    for row in &self.rows {
                        let mut col_idx = 0;
                        for cell in &row.cells {
                            if col_idx == col && cell.colspan == 1 {
                                let font_size = cell.font_size.unwrap_or(self.style.font_size);
                                let text_width = font_metrics.text_width(&cell.content, font_size);
                                let cell_padding = cell.padding.as_ref().unwrap_or(padding);
                                max_width = max_width.max(text_width + cell_padding.horizontal());
                            }
                            col_idx += cell.colspan;
                        }
                    }
                    widths[col] = max_width.max(20.0); // Minimum 20pt
                },
            }
        }

        // Handle remaining auto columns with default width
        for col in self.column_widths.len()..num_cols {
            let mut max_width = 0.0f32;
            for row in &self.rows {
                let mut col_idx = 0;
                for cell in &row.cells {
                    if col_idx == col && cell.colspan == 1 {
                        let font_size = cell.font_size.unwrap_or(self.style.font_size);
                        let text_width = font_metrics.text_width(&cell.content, font_size);
                        let cell_padding = cell.padding.as_ref().unwrap_or(padding);
                        max_width = max_width.max(text_width + cell_padding.horizontal());
                    }
                    col_idx += cell.colspan;
                }
            }
            widths[col] = max_width.max(20.0);
        }

        // Distribute remaining space to weighted columns
        let used_width: f32 = widths.iter().sum();
        let remaining = (table_width - used_width).max(0.0);

        if weight_total > 0.0 && remaining > 0.0 {
            for (col, spec) in self.column_widths.iter().take(num_cols).enumerate() {
                if let ColumnWidth::Weight(w) = spec {
                    widths[col] = remaining * (*w / weight_total);
                }
            }
        }

        // Scale if total exceeds table width
        let total: f32 = widths.iter().sum();
        if total > table_width && total > 0.0 {
            let scale = table_width / total;
            for w in &mut widths {
                *w *= scale;
            }
        }

        widths
    }

    /// Wrap every cell's content to its resolved column width and measure
    /// each resulting line. Returns a per-row, per-cell matrix of
    /// `CellLayout` indexed identically to `self.rows[row].cells[cell]`.
    fn wrap_all_cells(
        &self,
        column_widths: &[f32],
        font_metrics: &dyn FontMetrics,
    ) -> Vec<Vec<CellLayout>> {
        let padding = &self.style.cell_padding;
        let mut out = Vec::with_capacity(self.rows.len());

        for row in &self.rows {
            let mut row_out = Vec::with_capacity(row.cells.len());
            let mut col_idx = 0;

            for cell in &row.cells {
                let cell_width: f32 = if cell.colspan == 1 {
                    column_widths.get(col_idx).copied().unwrap_or(100.0)
                } else {
                    column_widths[col_idx..col_idx + cell.colspan].iter().sum()
                };

                let cell_padding = cell.padding.as_ref().unwrap_or(padding);
                let content_width = cell_width - cell_padding.horizontal();
                let font_size = cell.font_size.unwrap_or(self.style.font_size);

                let lines = wrap_text(&cell.content, content_width, font_size, font_metrics);
                let line_widths = lines
                    .iter()
                    .map(|l| font_metrics.text_width(l, font_size))
                    .collect();

                row_out.push(CellLayout { lines, line_widths });
                col_idx += cell.colspan;
            }

            out.push(row_out);
        }

        out
    }

    fn calculate_row_heights(&self, cell_layouts: &[Vec<CellLayout>]) -> Vec<f32> {
        let padding = &self.style.cell_padding;
        let mut heights = Vec::with_capacity(self.rows.len());

        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut max_height = 0.0f32;

            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if cell.rowspan == 1 {
                    let cell_padding = cell.padding.as_ref().unwrap_or(padding);
                    let font_size = cell.font_size.unwrap_or(self.style.font_size);
                    let line_height = font_size * 1.2;

                    let n_lines = cell_layouts[row_idx][cell_idx].lines.len() as f32;
                    let text_height = n_lines * line_height;

                    let cell_height = text_height + cell_padding.vertical();
                    max_height = max_height.max(cell_height);
                }
            }

            // Apply minimum height if specified
            if let Some(min_h) = row.min_height {
                max_height = max_height.max(min_h);
            }

            heights.push(max_height.max(self.style.font_size * 1.5));
        }

        heights
    }

    fn calculate_cell_positions(
        &self,
        column_widths: &[f32],
        row_heights: &[f32],
    ) -> Vec<Vec<CellPosition>> {
        let mut positions = Vec::with_capacity(self.rows.len());
        let mut y = 0.0;

        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut row_positions = Vec::with_capacity(row.cells.len());
            let mut x = 0.0;
            let mut col_idx = 0;

            for cell in &row.cells {
                let width: f32 = column_widths[col_idx..col_idx + cell.colspan].iter().sum();
                let height: f32 = row_heights[row_idx..row_idx + cell.rowspan].iter().sum();

                row_positions.push(CellPosition {
                    x,
                    y,
                    width,
                    height,
                });

                x += width;
                col_idx += cell.colspan;
            }

            positions.push(row_positions);
            y += row_heights[row_idx];
        }

        positions
    }

    /// Render the table to a list of high-level `ContentElement`s.
    ///
    /// This is the blessed path for the fluent `FluentPageBuilder::table`
    /// surface (#393). Emitting `ContentElement::Text` + `ContentElement::
    /// Path` rather than writing raw content-stream ops lets the writer's
    /// subsetter (v0.3.38 #385) re-key glyph IDs for dynamically-added
    /// CJK cell text — raw `ContentStreamBuilder.text()` bypasses that
    /// dispatch and would corrupt subset embedding.
    ///
    /// The low-level `render()` method is retained as an escape hatch for
    /// callers that already have a `ContentStreamBuilder` in hand, and
    /// for the renderer's own unit tests which inspect emitted PDF ops.
    pub fn to_content_elements(
        &self,
        x: f32,
        y: f32,
        layout: &TableLayout,
    ) -> Vec<crate::elements::ContentElement> {
        use crate::elements::{
            ContentElement, FontSpec, PathContent, PathOperation, TextContent, TextStyle,
        };
        use crate::geometry::Rect;
        use crate::layout::Color;

        // y is top of table (paragraph/text convention — y decreases
        // going down a page).
        let table_top = y;
        let mut elements: Vec<ContentElement> = Vec::new();

        // ── Pass 1 ── backgrounds and cell borders ──────────────────
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                let pos = &layout.cell_positions[row_idx][cell_idx];
                let cell_x = x + pos.x;
                let cell_y = table_top - pos.y - pos.height;

                // Background colour precedence: cell > header-row > stripe > row.
                let bg = cell.background.or({
                    if row.is_header {
                        self.style.header_background
                    } else if let Some(stripe) = self.style.stripe_color {
                        if row_idx % 2 == 1 {
                            Some(stripe)
                        } else {
                            row.background
                        }
                    } else {
                        row.background
                    }
                });

                if let Some((r, g, b)) = bg {
                    let mut path =
                        PathContent::new(Rect::new(cell_x, cell_y, pos.width, pos.height));
                    path.operations
                        .push(PathOperation::Rectangle(cell_x, cell_y, pos.width, pos.height));
                    path.fill_color = Some(Color { r, g, b });
                    path.stroke_color = None;
                    path.reading_order = Some(elements.len());
                    elements.push(ContentElement::Path(path));
                }

                let borders = cell.borders.as_ref().unwrap_or(&self.style.cell_borders);
                Self::push_cell_border_elements(
                    &mut elements,
                    cell_x,
                    cell_y,
                    pos.width,
                    pos.height,
                    borders,
                );
            }
        }

        // ── Pass 2 ── text per wrapped line, aligned by measured width
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if cell.content.is_empty() {
                    continue;
                }

                let pos = &layout.cell_positions[row_idx][cell_idx];
                let padding = cell.padding.as_ref().unwrap_or(&self.style.cell_padding);
                let cell_x = x + pos.x + padding.left;
                let cell_y = table_top - pos.y - padding.top;
                let content_width = pos.width - padding.horizontal();

                let align = if cell.align != CellAlign::Left {
                    cell.align
                } else {
                    self.column_aligns
                        .get(cell_idx)
                        .copied()
                        .unwrap_or(CellAlign::Left)
                };

                let font_name = cell.font_name.as_deref().unwrap_or(&self.style.font_name);
                let font_size = cell.font_size.unwrap_or(self.style.font_size);

                let actual_font = if cell.bold && cell.italic {
                    format!("{}-BoldOblique", font_name)
                } else if cell.bold || row.is_header {
                    format!("{}-Bold", font_name)
                } else if cell.italic {
                    format!("{}-Oblique", font_name)
                } else {
                    font_name.to_string()
                };

                let line_height = font_size * 1.2;
                let cell_layout = &layout.cell_layouts[row_idx][cell_idx];

                for (line_idx, (line, line_width)) in cell_layout
                    .lines
                    .iter()
                    .zip(cell_layout.line_widths.iter())
                    .enumerate()
                {
                    if line.is_empty() {
                        continue;
                    }

                    let text_x = match align {
                        CellAlign::Left => cell_x,
                        CellAlign::Center => cell_x + (content_width - line_width) / 2.0,
                        CellAlign::Right => cell_x + content_width - line_width,
                    };

                    // TextContent.bbox uses the top-edge y convention; the
                    // first line sits at the cell's top-after-padding.
                    let line_top = cell_y - (line_idx as f32) * line_height;

                    elements.push(ContentElement::Text(TextContent {
                        text: line.clone(),
                        bbox: Rect::new(text_x, line_top, *line_width, font_size),
                        font: FontSpec {
                            name: actual_font.clone(),
                            size: font_size,
                        },
                        style: TextStyle::default(),
                        reading_order: Some(elements.len()),
                        artifact_type: None,
                        origin: None,
                        rotation_degrees: None,
                        matrix: None,
                    }));
                }
            }
        }

        // ── Pass 3 ── outer table border ────────────────────────────
        if let Some(outer) = &self.style.outer_border {
            if outer.width > 0.0 {
                let outer_y = table_top - layout.total_height;
                let mut path = PathContent::new(Rect::new(
                    x,
                    outer_y,
                    layout.total_width,
                    layout.total_height,
                ));
                path.operations.push(PathOperation::Rectangle(
                    x,
                    outer_y,
                    layout.total_width,
                    layout.total_height,
                ));
                path.stroke_color = Some(Color {
                    r: outer.color.0,
                    g: outer.color.1,
                    b: outer.color.2,
                });
                path.stroke_width = outer.width;
                path.fill_color = None;
                path.reading_order = Some(elements.len());
                elements.push(ContentElement::Path(path));
            }
        }

        elements
    }

    /// Push one `ContentElement::Path` per enabled border side. Used by
    /// `to_content_elements` to mirror `draw_cell_borders` semantics.
    fn push_cell_border_elements(
        elements: &mut Vec<crate::elements::ContentElement>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        borders: &Borders,
    ) {
        use crate::elements::{ContentElement, PathContent, PathOperation};
        use crate::geometry::Rect;
        use crate::layout::Color;

        let mut push_line = |x1: f32, y1: f32, x2: f32, y2: f32, style: &TableBorderStyle| {
            if style.width <= 0.0 {
                return;
            }
            let min_x = x1.min(x2);
            let min_y = y1.min(y2);
            let w = (x2 - x1).abs().max(1.0);
            let h = (y2 - y1).abs().max(1.0);
            let mut path = PathContent::new(Rect::new(min_x, min_y, w, h));
            path.operations.push(PathOperation::MoveTo(x1, y1));
            path.operations.push(PathOperation::LineTo(x2, y2));
            path.stroke_color = Some(Color {
                r: style.color.0,
                g: style.color.1,
                b: style.color.2,
            });
            path.stroke_width = style.width;
            path.fill_color = None;
            path.reading_order = Some(elements.len());
            elements.push(ContentElement::Path(path));
        };

        if let Some(border) = &borders.top {
            push_line(x, y + height, x + width, y + height, border);
        }
        if let Some(border) = &borders.bottom {
            push_line(x, y, x + width, y, border);
        }
        if let Some(border) = &borders.left {
            push_line(x, y, x, y + height, border);
        }
        if let Some(border) = &borders.right {
            push_line(x + width, y, x + width, y + height, border);
        }
    }

    /// Render the table to a content stream.
    pub fn render(
        &self,
        builder: &mut ContentStreamBuilder,
        x: f32,
        y: f32,
        layout: &TableLayout,
    ) -> Result<()> {
        // Y is top of table, PDF coordinates are bottom-up
        let table_top = y;

        // Draw backgrounds and borders
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                let pos = &layout.cell_positions[row_idx][cell_idx];
                let cell_x = x + pos.x;
                let cell_y = table_top - pos.y - pos.height;

                // Determine background color
                let bg = cell.background.or({
                    if row.is_header {
                        self.style.header_background
                    } else if let Some(stripe) = self.style.stripe_color {
                        if row_idx % 2 == 1 {
                            Some(stripe)
                        } else {
                            row.background
                        }
                    } else {
                        row.background
                    }
                });

                // Draw background
                if let Some((r, g, b)) = bg {
                    builder.set_fill_color(r, g, b);
                    builder.rect(cell_x, cell_y, pos.width, pos.height);
                    builder.fill();
                }

                // Draw borders
                let borders = cell.borders.as_ref().unwrap_or(&self.style.cell_borders);
                self.draw_cell_borders(builder, cell_x, cell_y, pos.width, pos.height, borders);
            }
        }

        // Draw text
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if cell.content.is_empty() {
                    continue;
                }

                let pos = &layout.cell_positions[row_idx][cell_idx];
                let padding = cell.padding.as_ref().unwrap_or(&self.style.cell_padding);

                let cell_x = x + pos.x + padding.left;
                let cell_y = table_top - pos.y - padding.top;
                let content_width = pos.width - padding.horizontal();

                // Get alignment (cell -> column -> default)
                let align = if cell.align != CellAlign::Left {
                    cell.align
                } else {
                    self.column_aligns
                        .get(cell_idx)
                        .copied()
                        .unwrap_or(CellAlign::Left)
                };

                let font_name = cell.font_name.as_deref().unwrap_or(&self.style.font_name);
                let font_size = cell.font_size.unwrap_or(self.style.font_size);

                // Adjust font name for bold/italic
                let actual_font = if cell.bold && cell.italic {
                    format!("{}-BoldOblique", font_name)
                } else if cell.bold || row.is_header {
                    format!("{}-Bold", font_name)
                } else if cell.italic {
                    format!("{}-Oblique", font_name)
                } else {
                    font_name.to_string()
                };

                builder.begin_text().set_font(&actual_font, font_size);

                let line_height = font_size * 1.2;
                let cell_layout = &layout.cell_layouts[row_idx][cell_idx];

                for (line_idx, (line, line_width)) in cell_layout
                    .lines
                    .iter()
                    .zip(cell_layout.line_widths.iter())
                    .enumerate()
                {
                    if line.is_empty() {
                        continue;
                    }

                    // Per-line x placement: each line aligns independently.
                    // `text` places its first glyph at the given x, so Center
                    // and Right must offset by this line's measured width.
                    let text_x = match align {
                        CellAlign::Left => cell_x,
                        CellAlign::Center => cell_x + (content_width - line_width) / 2.0,
                        CellAlign::Right => cell_x + content_width - line_width,
                    };

                    // Baseline: first line sits `font_size` below the cell
                    // top; each subsequent line drops by `line_height`.
                    let text_y = cell_y - font_size - (line_idx as f32) * line_height;

                    builder.text(line, text_x, text_y);
                }

                builder.end_text();
            }
        }

        // Draw outer border
        if let Some(outer) = &self.style.outer_border {
            if outer.width > 0.0 {
                builder.set_stroke_color(outer.color.0, outer.color.1, outer.color.2);
                builder.set_line_width(outer.width);
                builder.rect(
                    x,
                    table_top - layout.total_height,
                    layout.total_width,
                    layout.total_height,
                );
                builder.stroke();
            }
        }

        Ok(())
    }

    fn draw_cell_borders(
        &self,
        builder: &mut ContentStreamBuilder,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        borders: &Borders,
    ) {
        // Top border
        if let Some(border) = &borders.top {
            if border.width > 0.0 {
                builder.set_stroke_color(border.color.0, border.color.1, border.color.2);
                builder.set_line_width(border.width);
                builder.move_to(x, y + height);
                builder.line_to(x + width, y + height);
                builder.stroke();
            }
        }

        // Bottom border
        if let Some(border) = &borders.bottom {
            if border.width > 0.0 {
                builder.set_stroke_color(border.color.0, border.color.1, border.color.2);
                builder.set_line_width(border.width);
                builder.move_to(x, y);
                builder.line_to(x + width, y);
                builder.stroke();
            }
        }

        // Left border
        if let Some(border) = &borders.left {
            if border.width > 0.0 {
                builder.set_stroke_color(border.color.0, border.color.1, border.color.2);
                builder.set_line_width(border.width);
                builder.move_to(x, y);
                builder.line_to(x, y + height);
                builder.stroke();
            }
        }

        // Right border
        if let Some(border) = &borders.right {
            if border.width > 0.0 {
                builder.set_stroke_color(border.color.0, border.color.1, border.color.2);
                builder.set_line_width(border.width);
                builder.move_to(x + width, y);
                builder.line_to(x + width, y + height);
                builder.stroke();
            }
        }
    }
}

/// Trait for font metrics needed for layout.
pub trait FontMetrics {
    /// Calculate the width of text in points.
    fn text_width(&self, text: &str, font_size: f32) -> f32;
}

/// Simple font metrics using average character width.
#[derive(Debug, Clone, Copy)]
pub struct SimpleFontMetrics {
    /// Average character width as proportion of font size
    pub char_width_ratio: f32,
}

impl Default for SimpleFontMetrics {
    fn default() -> Self {
        Self {
            char_width_ratio: 0.5, // Typical for proportional fonts
        }
    }
}

impl SimpleFontMetrics {
    /// Create metrics for monospace fonts.
    pub fn monospace() -> Self {
        Self {
            char_width_ratio: 0.6,
        }
    }
}

impl FontMetrics for SimpleFontMetrics {
    fn text_width(&self, text: &str, font_size: f32) -> f32 {
        text.chars().count() as f32 * font_size * self.char_width_ratio
    }
}

/// Wrap text to fit within a given width.
fn wrap_text(text: &str, max_width: f32, font_size: f32, metrics: &dyn FontMetrics) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let width = metrics.text_width(&test_line, font_size);

        if width <= max_width || current_line.is_empty() {
            current_line = test_line;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_cell_creation() {
        let cell = TableCell::text("Hello");
        assert_eq!(cell.content, "Hello");
        assert_eq!(cell.colspan, 1);
        assert_eq!(cell.rowspan, 1);
    }

    #[test]
    fn test_table_cell_header() {
        let cell = TableCell::header("Title");
        assert!(cell.bold);
        assert_eq!(cell.align, CellAlign::Center);
    }

    #[test]
    fn test_table_cell_spanning() {
        let cell = TableCell::text("Wide").colspan(2).rowspan(3);
        assert_eq!(cell.colspan, 2);
        assert_eq!(cell.rowspan, 3);
    }

    #[test]
    fn test_table_creation() {
        let table = Table::new(vec![
            vec![TableCell::text("A"), TableCell::text("B")],
            vec![TableCell::text("C"), TableCell::text("D")],
        ]);

        assert_eq!(table.num_columns(), 2);
        assert_eq!(table.num_rows(), 2);
    }

    #[test]
    fn test_table_with_header() {
        let table = Table::new(vec![
            vec![TableCell::text("Name"), TableCell::text("Age")],
            vec![TableCell::text("Alice"), TableCell::text("30")],
        ])
        .with_header_row();

        assert!(table.rows[0].is_header);
    }

    #[test]
    fn test_cell_padding() {
        let padding = CellPadding::uniform(10.0);
        assert_eq!(padding.horizontal(), 20.0);
        assert_eq!(padding.vertical(), 20.0);

        let asym = CellPadding::symmetric(5.0, 10.0);
        assert_eq!(asym.horizontal(), 10.0);
        assert_eq!(asym.vertical(), 20.0);
    }

    #[test]
    fn test_borders() {
        let borders = Borders::all(TableBorderStyle::medium());
        assert!(borders.top.is_some());
        assert!(borders.right.is_some());
        assert!(borders.bottom.is_some());
        assert!(borders.left.is_some());

        let horiz = Borders::horizontal(TableBorderStyle::thin());
        assert!(horiz.top.is_some());
        assert!(horiz.bottom.is_some());
        assert!(horiz.left.is_none());
        assert!(horiz.right.is_none());
    }

    #[test]
    fn test_column_width_types() {
        let _auto = ColumnWidth::Auto;
        let _fixed = ColumnWidth::Fixed(100.0);
        let _percent = ColumnWidth::Percent(25.0);
        let _weight = ColumnWidth::Weight(1.0);
    }

    #[test]
    fn test_table_style_presets() {
        let minimal = TableStyle::minimal();
        assert!(minimal.cell_borders.top.is_none());
        assert!(minimal.outer_border.is_none());

        let bordered = TableStyle::bordered();
        assert!(bordered.outer_border.is_some());
    }

    #[test]
    fn test_table_layout_calculation() {
        let table = Table::new(vec![
            vec![TableCell::text("Name"), TableCell::text("Value")],
            vec![TableCell::text("Test"), TableCell::text("123")],
        ]);

        let metrics = SimpleFontMetrics::default();
        let layout = table.calculate_layout(400.0, &metrics);

        assert_eq!(layout.column_widths.len(), 2);
        assert_eq!(layout.row_heights.len(), 2);
        assert!(layout.total_width > 0.0);
        assert!(layout.total_height > 0.0);
    }

    #[test]
    fn test_text_wrapping() {
        let metrics = SimpleFontMetrics::default();
        let lines = wrap_text("Hello World", 100.0, 12.0, &metrics);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_empty_table() {
        let table = Table::empty();
        assert!(table.is_empty());
        assert_eq!(table.num_columns(), 0);
        assert_eq!(table.num_rows(), 0);
    }

    #[test]
    fn test_cell_alignments() {
        let left = TableCell::text("Left").align(CellAlign::Left);
        let center = TableCell::text("Center").align(CellAlign::Center);
        let right = TableCell::number("123");

        assert_eq!(left.align, CellAlign::Left);
        assert_eq!(center.align, CellAlign::Center);
        assert_eq!(right.align, CellAlign::Right);
    }

    #[test]
    fn test_row_creation() {
        let row = TableRow::new(vec![TableCell::text("A"), TableCell::text("B")]);
        assert_eq!(row.cells.len(), 2);
        assert!(!row.is_header);

        let header = TableRow::header(vec![TableCell::text("Name"), TableCell::text("Value")]);
        assert!(header.is_header);
    }

    #[test]
    fn test_striped_table() {
        let style = TableStyle::new().striped(0.95, 0.95, 0.95);
        assert!(style.stripe_color.is_some());
    }

    #[test]
    fn test_multi_line_cell_layout_stores_wrapped_lines() {
        // A cell with content that must wrap into multiple lines.
        let long = "The quick brown fox jumps over the lazy dog";
        let table = Table::new(vec![vec![TableCell::text(long), TableCell::text("short")]])
            .with_column_widths(vec![ColumnWidth::Fixed(60.0), ColumnWidth::Fixed(80.0)]);

        let metrics = SimpleFontMetrics::default();
        let layout = table.calculate_layout(140.0, &metrics);

        // cell_layouts must mirror the cells grid.
        assert_eq!(layout.cell_layouts.len(), 1);
        assert_eq!(layout.cell_layouts[0].len(), 2);

        // First cell should wrap to > 1 line; second cell stays on 1 line.
        assert!(
            layout.cell_layouts[0][0].lines.len() > 1,
            "expected multi-line wrap, got {} lines: {:?}",
            layout.cell_layouts[0][0].lines.len(),
            layout.cell_layouts[0][0].lines
        );
        assert_eq!(layout.cell_layouts[0][1].lines.len(), 1);

        // Line widths must match the line count for every cell.
        for row in &layout.cell_layouts {
            for cell in row {
                assert_eq!(
                    cell.lines.len(),
                    cell.line_widths.len(),
                    "lines and line_widths must be parallel"
                );
            }
        }

        // Row height must scale with the wrapped line count.
        // With >=2 lines and font_size 10 (default style), text_height alone
        // exceeds the minimum `font_size * 1.5` floor.
        let expected_min = 2.0 * (layout.cell_layouts[0][0].lines.len() as f32) * 0.5;
        assert!(layout.row_heights[0] > expected_min);
    }

    #[test]
    fn test_multi_line_render_emits_one_tj_per_line() {
        use super::super::content_stream::ContentStreamBuilder;

        // Force wrap by choosing a narrow column.
        let long = "alpha beta gamma delta epsilon zeta eta theta";
        let table = Table::new(vec![vec![TableCell::text(long)]])
            .with_column_widths(vec![ColumnWidth::Fixed(40.0)]);

        let metrics = SimpleFontMetrics::default();
        let layout = table.calculate_layout(40.0, &metrics);
        let expected_lines = layout.cell_layouts[0][0].lines.len();
        assert!(expected_lines >= 3, "fixture must wrap to >=3 lines to be meaningful");

        // Render to a content stream and count `Tj` text-show operations.
        let mut builder = ContentStreamBuilder::new();
        table
            .render(&mut builder, 0.0, 800.0, &layout)
            .expect("render");

        let bytes = builder.build().expect("build content stream");
        let text = String::from_utf8_lossy(&bytes);
        let tj_count = text.matches(" Tj").count();

        assert_eq!(
            tj_count, expected_lines,
            "expected {} Tj operations (one per wrapped line), got {}\n--- stream ---\n{}",
            expected_lines, tj_count, text
        );
    }

    #[test]
    fn test_to_content_elements_emits_text_per_line() {
        // The fluent-builder path: to_content_elements must emit one
        // ContentElement::Text per wrapped line per cell, using the measured
        // line width for alignment.
        use crate::elements::ContentElement;

        let long = "alpha beta gamma delta epsilon zeta eta theta";
        let table = Table::new(vec![vec![TableCell::text(long), TableCell::text("x")]])
            .with_column_widths(vec![ColumnWidth::Fixed(40.0), ColumnWidth::Fixed(40.0)]);

        let metrics = SimpleFontMetrics::default();
        let layout = table.calculate_layout(80.0, &metrics);
        let expected_lines_col0 = layout.cell_layouts[0][0].lines.len();
        assert!(expected_lines_col0 >= 2, "fixture must wrap >=2 lines");

        let elements = table.to_content_elements(0.0, 800.0, &layout);
        let texts: Vec<_> = elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Text(t) => Some(t),
                _ => None,
            })
            .collect();

        // One Text per non-empty line per cell (cell 0 wraps, cell 1 is 1 line).
        assert_eq!(texts.len(), expected_lines_col0 + 1);

        // Every text bbox must stay inside the table horizontally.
        for t in &texts {
            assert!(t.bbox.x >= 0.0 - 0.01);
            assert!(t.bbox.x + t.bbox.width <= 80.0 + 0.01);
        }

        // Lines within a cell stack top-down (y decreasing).
        let col0_lines: Vec<_> = texts.iter().take(expected_lines_col0).collect();
        for pair in col0_lines.windows(2) {
            assert!(
                pair[1].bbox.y < pair[0].bbox.y,
                "multi-line cell lines must move down: {} then {}",
                pair[0].bbox.y,
                pair[1].bbox.y
            );
        }
    }

    #[test]
    fn test_to_content_elements_path_for_backgrounds_and_borders() {
        use crate::elements::ContentElement;

        let mut style = TableStyle::new().striped(0.9, 0.9, 0.9);
        style.outer_border = Some(TableBorderStyle::medium());

        let table = Table::new(vec![
            vec![TableCell::text("a"), TableCell::text("b")],
            vec![TableCell::text("c"), TableCell::text("d")],
        ])
        .with_style(style)
        .with_column_widths(vec![ColumnWidth::Fixed(60.0), ColumnWidth::Fixed(60.0)]);

        let metrics = SimpleFontMetrics::default();
        let layout = table.calculate_layout(120.0, &metrics);
        let elements = table.to_content_elements(0.0, 800.0, &layout);

        let paths: Vec<_> = elements
            .iter()
            .filter_map(|e| match e {
                ContentElement::Path(p) => Some(p),
                _ => None,
            })
            .collect();

        // At least one stripe fill on row 1 + 4 cell borders per cell ×
        // 4 cells + outer border. Check the fill/stroke mix is plausible.
        let fills = paths.iter().filter(|p| p.fill_color.is_some()).count();
        let strokes = paths.iter().filter(|p| p.stroke_color.is_some()).count();
        assert!(fills >= 2, "expected stripe fills on row 1 cells, got {}", fills);
        assert!(strokes >= 1, "expected at least outer border stroke, got {}", strokes);

        // Reading order must be monotone — tables are emitted in drawing
        // order so later overlays paint on top of earlier ones.
        let orders: Vec<_> = elements.iter().filter_map(|e| e.reading_order()).collect();
        for pair in orders.windows(2) {
            assert!(pair[1] > pair[0], "reading_order must increase: {:?}", orders);
        }
    }

    #[test]
    fn test_alignment_offsets_per_line_width() {
        // Single-line cells in a single row, each with its own alignment.
        // Verify the emitted Tm x-offset matches cell_x + alignment maths
        // applied to the MEASURED line width, not the column content width.
        use super::super::content_stream::ContentStreamBuilder;

        let table = Table::new(vec![vec![
            TableCell::text("L").align(CellAlign::Left),
            TableCell::text("C").align(CellAlign::Center),
            TableCell::text("R").align(CellAlign::Right),
        ]])
        .with_column_widths(vec![
            ColumnWidth::Fixed(60.0),
            ColumnWidth::Fixed(60.0),
            ColumnWidth::Fixed(60.0),
        ])
        .with_style(TableStyle::new().cell_padding(CellPadding::uniform(0.0)));

        let metrics = SimpleFontMetrics::default();
        let layout = table.calculate_layout(180.0, &metrics);

        let mut builder = ContentStreamBuilder::new();
        table
            .render(&mut builder, 0.0, 800.0, &layout)
            .expect("render");
        let bytes = builder.build().expect("build");
        let text = String::from_utf8_lossy(&bytes);

        // Extract all `a b c d e f Tm` operations and grab the e component
        // (x-offset). We expect three, corresponding to the three cells in
        // left→center→right order.
        let tm_xs: Vec<f32> = text
            .lines()
            .filter_map(|l| l.strip_suffix(" Tm"))
            .filter_map(|prefix| {
                let parts: Vec<&str> = prefix.split_whitespace().collect();
                if parts.len() == 6 {
                    parts[4].parse::<f32>().ok()
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(tm_xs.len(), 3, "expected 3 Tm ops, got {}: {:?}", tm_xs.len(), tm_xs);

        // Column widths all 60.0, padding zero, single-char content.
        // Left:  x = 0
        // Center: x = 60 + (60 - line_width) / 2  = 60 + (60 - 5) / 2 = 87.5
        // Right:  x = 120 + (60 - line_width)     = 120 + 55 = 175
        // Single char "L"/"C"/"R" at default metrics = 1 * 10 * 0.5 = 5.0
        let char_w = 5.0f32;
        let left_expected = 0.0;
        let center_expected = 60.0 + (60.0 - char_w) / 2.0;
        let right_expected = 120.0 + 60.0 - char_w;

        assert!((tm_xs[0] - left_expected).abs() < 0.01, "left x={}", tm_xs[0]);
        assert!((tm_xs[1] - center_expected).abs() < 0.01, "center x={}", tm_xs[1]);
        assert!((tm_xs[2] - right_expected).abs() < 0.01, "right x={}", tm_xs[2]);
    }
}
