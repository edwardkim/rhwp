//! Table content element for PDF read/write operations.
//!
//! This module provides structures for representing table content that can be
//! extracted from existing PDFs or generated into new PDFs.

use crate::geometry::Rect;

/// Source of table data - how the table was identified/created.
///
/// This tracks the origin of table content to help downstream consumers
/// understand the reliability and nature of the table data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableSource {
    /// Table extracted from PDF structure tree (Tagged PDF).
    ///
    /// High confidence - explicitly marked in PDF per ISO 32000-1:2008 Section 14.8.4.3.4.
    /// Includes THead, TBody, TFoot, TR, TH, TD elements.
    StructureTree,
    /// Table detected via spatial/heuristic analysis.
    ///
    /// Medium confidence - detected through layout analysis, grid detection,
    /// or other spatial heuristics. May have false positives.
    #[default]
    SpatialDetection,
    /// Table created programmatically by the user/API.
    ///
    /// High confidence - explicitly constructed for PDF generation.
    UserGenerated,
}

/// Detection metadata for tables.
///
/// Provides additional context about how a table was detected,
/// including confidence scores for heuristic detections.
#[derive(Debug, Clone, Default)]
pub struct TableDetectionInfo {
    /// Source of this table data.
    pub source: TableSource,
    /// Confidence score for heuristic detections (0.0 - 1.0).
    ///
    /// - StructureTree: always 1.0 (explicit PDF markup)
    /// - SpatialDetection: varies based on detection quality
    /// - UserGenerated: always 1.0 (explicit creation)
    pub confidence: f32,
    /// Optional detection method name for debugging.
    pub detection_method: Option<String>,
}

impl TableDetectionInfo {
    /// Create info for structure tree source (high confidence).
    pub fn from_structure_tree() -> Self {
        Self {
            source: TableSource::StructureTree,
            confidence: 1.0,
            detection_method: Some("structure_tree".to_string()),
        }
    }

    /// Create info for spatial detection with confidence.
    pub fn from_spatial_detection(confidence: f32, method: impl Into<String>) -> Self {
        Self {
            source: TableSource::SpatialDetection,
            confidence: confidence.clamp(0.0, 1.0),
            detection_method: Some(method.into()),
        }
    }

    /// Create info for user-generated table.
    pub fn user_generated() -> Self {
        Self {
            source: TableSource::UserGenerated,
            confidence: 1.0,
            detection_method: None,
        }
    }
}

/// Table content element.
///
/// Represents a table with rows, columns, and cells that can be
/// extracted from or written to a PDF.
#[derive(Debug, Clone)]
pub struct TableContent {
    /// Bounding box of the entire table
    pub bbox: Rect,
    /// Table rows
    pub rows: Vec<TableRowContent>,
    /// Column widths (if known)
    pub column_widths: Vec<f32>,
    /// Reading order index
    pub reading_order: Option<usize>,
    /// Optional table caption
    pub caption: Option<String>,
    /// Table style information
    pub style: TableContentStyle,
    /// Detection/source information
    pub detection_info: TableDetectionInfo,
}

impl Default for TableContent {
    fn default() -> Self {
        Self {
            bbox: Rect::new(0.0, 0.0, 0.0, 0.0),
            rows: Vec::new(),
            column_widths: Vec::new(),
            reading_order: None,
            caption: None,
            style: TableContentStyle::default(),
            detection_info: TableDetectionInfo::default(),
        }
    }
}

impl TableContent {
    /// Create a new empty table with a bounding box.
    pub fn new(bbox: Rect) -> Self {
        Self {
            bbox,
            ..Default::default()
        }
    }

    /// Add a row to the table.
    pub fn add_row(&mut self, row: TableRowContent) {
        self.rows.push(row);
    }

    /// Get the number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns (based on first row).
    pub fn column_count(&self) -> usize {
        self.rows.first().map(|r| r.cells.len()).unwrap_or(0)
    }

    /// Get a cell at the specified row and column.
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&TableCellContent> {
        self.rows.get(row).and_then(|r| r.cells.get(col))
    }

    /// Check if the table has a header row.
    pub fn has_header(&self) -> bool {
        self.rows.first().is_some_and(|r| r.is_header)
    }

    /// Set the detection info for this table.
    pub fn with_detection_info(mut self, info: TableDetectionInfo) -> Self {
        self.detection_info = info;
        self
    }

    /// Create a user-generated table.
    pub fn user_generated(bbox: Rect) -> Self {
        Self {
            bbox,
            detection_info: TableDetectionInfo::user_generated(),
            ..Default::default()
        }
    }

    /// Check if table came from structure tree.
    pub fn is_from_structure_tree(&self) -> bool {
        self.detection_info.source == TableSource::StructureTree
    }

    /// Get detection confidence.
    pub fn detection_confidence(&self) -> f32 {
        self.detection_info.confidence
    }
}

/// A row in a table.
#[derive(Debug, Clone, Default)]
pub struct TableRowContent {
    /// Cells in this row
    pub cells: Vec<TableCellContent>,
    /// Whether this is a header row
    pub is_header: bool,
    /// Row height (if known)
    pub height: Option<f32>,
    /// Background color (R, G, B, each 0.0-1.0)
    pub background: Option<(f32, f32, f32)>,
}

impl TableRowContent {
    /// Create a new row with cells.
    pub fn new(cells: Vec<TableCellContent>) -> Self {
        Self {
            cells,
            ..Default::default()
        }
    }

    /// Create a header row with cells.
    pub fn header(cells: Vec<TableCellContent>) -> Self {
        Self {
            cells,
            is_header: true,
            ..Default::default()
        }
    }

    /// Add a cell to the row.
    pub fn add_cell(&mut self, cell: TableCellContent) {
        self.cells.push(cell);
    }
}

/// A cell in a table.
#[derive(Debug, Clone)]
pub struct TableCellContent {
    /// Cell text content
    pub text: String,
    /// Bounding box of the cell
    pub bbox: Rect,
    /// Number of columns this cell spans
    pub colspan: usize,
    /// Number of rows this cell spans
    pub rowspan: usize,
    /// Horizontal alignment
    pub align: TableCellAlign,
    /// Vertical alignment
    pub valign: TableCellVAlign,
    /// Whether this is a header cell
    pub is_header: bool,
    /// Background color (R, G, B, each 0.0-1.0)
    pub background: Option<(f32, f32, f32)>,
    /// Font size
    pub font_size: Option<f32>,
    /// Font name
    pub font_name: Option<String>,
    /// Whether text is bold
    pub bold: bool,
    /// Whether text is italic
    pub italic: bool,
}

impl Default for TableCellContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            bbox: Rect::new(0.0, 0.0, 0.0, 0.0),
            colspan: 1,
            rowspan: 1,
            align: TableCellAlign::Left,
            valign: TableCellVAlign::Top,
            is_header: false,
            background: None,
            font_size: None,
            font_name: None,
            bold: false,
            italic: false,
        }
    }
}

impl TableCellContent {
    /// Create a new cell with text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Create a header cell with text.
    pub fn header(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_header: true,
            bold: true,
            ..Default::default()
        }
    }

    /// Set the cell's bounding box.
    pub fn with_bbox(mut self, bbox: Rect) -> Self {
        self.bbox = bbox;
        self
    }

    /// Set column span.
    pub fn with_colspan(mut self, colspan: usize) -> Self {
        self.colspan = colspan;
        self
    }

    /// Set row span.
    pub fn with_rowspan(mut self, rowspan: usize) -> Self {
        self.rowspan = rowspan;
        self
    }

    /// Set horizontal alignment.
    pub fn with_align(mut self, align: TableCellAlign) -> Self {
        self.align = align;
        self
    }

    /// Set vertical alignment.
    pub fn with_valign(mut self, valign: TableCellVAlign) -> Self {
        self.valign = valign;
        self
    }

    /// Set background color.
    pub fn with_background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background = Some((r, g, b));
        self
    }

    /// Set font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set bold.
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// Set italic.
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }
}

/// Horizontal alignment for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableCellAlign {
    /// Left-aligned (default)
    #[default]
    Left,
    /// Center-aligned
    Center,
    /// Right-aligned
    Right,
}

/// Vertical alignment for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableCellVAlign {
    /// Top-aligned (default)
    #[default]
    Top,
    /// Middle-aligned
    Middle,
    /// Bottom-aligned
    Bottom,
}

/// Style information for a table.
#[derive(Debug, Clone)]
pub struct TableContentStyle {
    /// Border width (0.0 for no border)
    pub border_width: f32,
    /// Border color (R, G, B, each 0.0-1.0)
    pub border_color: (f32, f32, f32),
    /// Cell padding
    pub cell_padding: f32,
    /// Whether to show horizontal borders
    pub horizontal_borders: bool,
    /// Whether to show vertical borders
    pub vertical_borders: bool,
    /// Whether to show outer border
    pub outer_border: bool,
    /// Header background color
    pub header_background: Option<(f32, f32, f32)>,
    /// Alternating row background color (for striped tables)
    pub stripe_background: Option<(f32, f32, f32)>,
}

impl Default for TableContentStyle {
    fn default() -> Self {
        Self {
            border_width: 0.5,
            border_color: (0.0, 0.0, 0.0),
            cell_padding: 4.0,
            horizontal_borders: true,
            vertical_borders: true,
            outer_border: true,
            header_background: None,
            stripe_background: None,
        }
    }
}

impl TableContentStyle {
    /// Create a minimal style (horizontal lines only).
    pub fn minimal() -> Self {
        Self {
            vertical_borders: false,
            outer_border: false,
            ..Default::default()
        }
    }

    /// Create a bordered style (all borders).
    pub fn bordered() -> Self {
        Self::default()
    }

    /// Create a striped style with alternating row colors.
    pub fn striped() -> Self {
        Self {
            stripe_background: Some((0.95, 0.95, 0.95)),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_content_creation() {
        let mut table = TableContent::new(Rect::new(0.0, 0.0, 400.0, 200.0));

        let header = TableRowContent::header(vec![
            TableCellContent::header("Name"),
            TableCellContent::header("Value"),
        ]);
        table.add_row(header);

        let row = TableRowContent::new(vec![
            TableCellContent::new("Item 1"),
            TableCellContent::new("100"),
        ]);
        table.add_row(row);

        assert_eq!(table.row_count(), 2);
        assert_eq!(table.column_count(), 2);
        assert!(table.has_header());
    }

    #[test]
    fn test_cell_content_builder() {
        let cell = TableCellContent::new("Test")
            .with_colspan(2)
            .with_rowspan(1)
            .with_align(TableCellAlign::Center)
            .with_valign(TableCellVAlign::Middle)
            .with_background(0.9, 0.9, 0.9)
            .with_font_size(12.0)
            .with_bold(true);

        assert_eq!(cell.text, "Test");
        assert_eq!(cell.colspan, 2);
        assert_eq!(cell.align, TableCellAlign::Center);
        assert_eq!(cell.valign, TableCellVAlign::Middle);
        assert!(cell.bold);
        assert_eq!(cell.font_size, Some(12.0));
    }

    #[test]
    fn test_table_get_cell() {
        let mut table = TableContent::default();
        table.add_row(TableRowContent::new(vec![
            TableCellContent::new("A1"),
            TableCellContent::new("B1"),
        ]));
        table.add_row(TableRowContent::new(vec![
            TableCellContent::new("A2"),
            TableCellContent::new("B2"),
        ]));

        assert_eq!(table.get_cell(0, 0).map(|c| c.text.as_str()), Some("A1"));
        assert_eq!(table.get_cell(1, 1).map(|c| c.text.as_str()), Some("B2"));
        assert!(table.get_cell(2, 0).is_none());
    }

    #[test]
    fn test_table_style_presets() {
        let minimal = TableContentStyle::minimal();
        assert!(!minimal.vertical_borders);
        assert!(!minimal.outer_border);

        let bordered = TableContentStyle::bordered();
        assert!(bordered.vertical_borders);
        assert!(bordered.outer_border);

        let striped = TableContentStyle::striped();
        assert!(striped.stripe_background.is_some());
    }
}
