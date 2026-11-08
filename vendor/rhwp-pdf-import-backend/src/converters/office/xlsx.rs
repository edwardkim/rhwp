//! XLSX to PDF conversion.
//!
//! Parses Microsoft Excel spreadsheets (.xlsx) and converts them to PDF.
//!
//! Uses the calamine crate for reading Excel files.

use super::OfficeConfig;
use crate::error::{Error, Result};
use crate::writer::{DocumentBuilder, DocumentMetadata};
use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use std::io::Cursor;

/// XLSX to PDF converter.
pub struct XlsxConverter {
    config: OfficeConfig,
}

impl XlsxConverter {
    /// Create a new XLSX converter.
    pub fn new(config: OfficeConfig) -> Self {
        Self { config }
    }

    /// Convert XLSX bytes to PDF bytes.
    pub fn convert(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cursor = Cursor::new(bytes);
        let mut workbook = open_workbook_auto_from_rs(cursor)
            .map_err(|e| Error::InvalidPdf(format!("Failed to open XLSX: {}", e)))?;

        let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

        if sheet_names.is_empty() {
            return Err(Error::InvalidPdf("No sheets found in workbook".to_string()));
        }

        // Convert each sheet
        let mut all_sheets: Vec<SheetContent> = Vec::new();

        for name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let content = self.parse_sheet(name, &range);
                all_sheets.push(content);
            }
        }

        self.build_pdf(&all_sheets)
    }

    /// Parse a worksheet into structured content.
    fn parse_sheet(&self, name: &str, range: &Range<Data>) -> SheetContent {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut max_cols = 0;

        for row in range.rows() {
            let cells: Vec<String> = row.iter().map(|cell| self.cell_to_string(cell)).collect();
            max_cols = max_cols.max(cells.len());
            rows.push(cells);
        }

        SheetContent {
            name: name.to_string(),
            rows,
            max_columns: max_cols,
        }
    }

    /// Convert a cell value to a string.
    fn cell_to_string(&self, cell: &Data) -> String {
        match cell {
            Data::Empty => String::new(),
            Data::String(s) => s.clone(),
            Data::Int(i) => i.to_string(),
            Data::Float(f) => {
                // Format floats nicely - remove trailing zeros
                if f.fract() == 0.0 {
                    format!("{:.0}", f)
                } else {
                    format!("{:.2}", f)
                }
            },
            Data::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Data::DateTime(dt) => {
                // Excel datetime - format as string
                format!("{}", dt)
            },
            Data::DateTimeIso(s) => s.clone(),
            Data::DurationIso(s) => s.clone(),
            Data::Error(e) => format!("#ERR:{:?}", e),
        }
    }

    /// Build PDF from parsed sheets.
    fn build_pdf(&self, sheets: &[SheetContent]) -> Result<Vec<u8>> {
        let metadata = DocumentMetadata::new()
            .title("Spreadsheet")
            .creator("pdf_oxide");

        let mut builder = DocumentBuilder::new().metadata(metadata);

        let (page_width, page_height) = self.config.page_size.dimensions();
        let content_width = page_width - self.config.margins.left - self.config.margins.right;

        // Pre-process into render operations
        #[derive(Clone)]
        enum RenderOp {
            NewPage,
            Heading { text: String, y: f32 },
            Text { x: f32, y: f32, text: String },
        }

        let line_height = self.config.default_font_size * self.config.line_height;
        let mut all_ops: Vec<Vec<RenderOp>> = Vec::new();

        for sheet in sheets {
            let mut ops: Vec<RenderOp> = Vec::new();
            let mut current_y = page_height - self.config.margins.top;

            // Sheet title
            ops.push(RenderOp::Heading {
                text: sheet.name.clone(),
                y: current_y,
            });
            current_y -= line_height * 2.0;

            if sheet.rows.is_empty() {
                ops.push(RenderOp::Text {
                    x: self.config.margins.left,
                    y: current_y,
                    text: "(Empty sheet)".to_string(),
                });
                all_ops.push(ops);
                continue;
            }

            let col_widths = self.calculate_column_widths(sheet, content_width);

            for row in &sheet.rows {
                if current_y < self.config.margins.bottom + line_height {
                    ops.push(RenderOp::NewPage);
                    current_y = page_height - self.config.margins.top;
                }

                let mut x = self.config.margins.left;

                for (i, cell) in row.iter().enumerate() {
                    let col_width = col_widths.get(i).copied().unwrap_or(50.0);

                    let max_chars = (col_width / (self.config.default_font_size * 0.5)) as usize;
                    let display_text = if cell.len() > max_chars && max_chars > 3 {
                        format!("{}...", &cell[..max_chars - 3])
                    } else {
                        cell.clone()
                    };

                    ops.push(RenderOp::Text {
                        x,
                        y: current_y,
                        text: display_text,
                    });
                    x += col_width + 10.0;
                }

                current_y -= line_height;
            }

            all_ops.push(ops);
        }

        // Render all operations
        for ops in &all_ops {
            let mut page_builder = builder.page(self.config.page_size);

            for op in ops {
                match op {
                    RenderOp::NewPage => {
                        page_builder.done();
                        page_builder = builder.page(self.config.page_size);
                    },
                    RenderOp::Heading { text, y } => {
                        page_builder = page_builder
                            .at(self.config.margins.left, *y)
                            .heading(2, text);
                    },
                    RenderOp::Text { x, y, text } => {
                        page_builder = page_builder
                            .at(*x, *y)
                            .font(&self.config.default_font, self.config.default_font_size)
                            .text(text);
                    },
                }
            }

            page_builder.done();
        }

        builder.build()
    }

    /// Calculate column widths based on content.
    fn calculate_column_widths(&self, sheet: &SheetContent, max_width: f32) -> Vec<f32> {
        if sheet.max_columns == 0 {
            return vec![];
        }

        let mut max_lengths: Vec<usize> = vec![0; sheet.max_columns];

        // Find max length for each column
        for row in &sheet.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < max_lengths.len() {
                    max_lengths[i] = max_lengths[i].max(cell.len());
                }
            }
        }

        // Convert to widths with min/max constraints
        let char_width = self.config.default_font_size * 0.5;
        let min_col_width = 30.0;
        let max_col_width = 150.0;
        let column_gap = 10.0;

        let mut widths: Vec<f32> = max_lengths
            .iter()
            .map(|&len| (len as f32 * char_width).clamp(min_col_width, max_col_width))
            .collect();

        // Scale down if total exceeds available width
        let total_width: f32 = widths.iter().sum::<f32>() + (widths.len() - 1) as f32 * column_gap;
        if total_width > max_width {
            let scale = max_width / total_width;
            for w in &mut widths {
                *w *= scale;
            }
        }

        widths
    }
}

/// Parsed content from a worksheet.
struct SheetContent {
    name: String,
    rows: Vec<Vec<String>>,
    max_columns: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_to_string_empty() {
        let converter = XlsxConverter::new(OfficeConfig::default());
        assert_eq!(converter.cell_to_string(&Data::Empty), "");
    }

    #[test]
    fn test_cell_to_string_int() {
        let converter = XlsxConverter::new(OfficeConfig::default());
        assert_eq!(converter.cell_to_string(&Data::Int(42)), "42");
    }

    #[test]
    fn test_cell_to_string_float() {
        let converter = XlsxConverter::new(OfficeConfig::default());
        assert_eq!(converter.cell_to_string(&Data::Float(1.23)), "1.23");
        assert_eq!(converter.cell_to_string(&Data::Float(10.0)), "10");
    }

    #[test]
    fn test_cell_to_string_bool() {
        let converter = XlsxConverter::new(OfficeConfig::default());
        assert_eq!(converter.cell_to_string(&Data::Bool(true)), "TRUE");
        assert_eq!(converter.cell_to_string(&Data::Bool(false)), "FALSE");
    }

    #[test]
    fn test_cell_to_string_string() {
        let converter = XlsxConverter::new(OfficeConfig::default());
        assert_eq!(converter.cell_to_string(&Data::String("Hello".to_string())), "Hello");
    }
}
