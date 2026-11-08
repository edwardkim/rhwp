//! DOCX to PDF conversion.
//!
//! Parses Microsoft Word documents (.docx) and converts them to PDF.
//!
//! DOCX files are ZIP archives containing XML files in Open XML format.
//! The main content is in `word/document.xml`.

use super::styles::{
    half_points_to_points, parse_color, ParagraphAlignment, ParagraphStyle, TextStyle,
};
use super::OfficeConfig;
use crate::error::{Error, Result};
use crate::writer::{DocumentBuilder, DocumentMetadata};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// DOCX to PDF converter.
pub struct DocxConverter {
    config: OfficeConfig,
}

impl DocxConverter {
    /// Create a new DOCX converter.
    pub fn new(config: OfficeConfig) -> Self {
        Self { config }
    }

    /// Convert DOCX bytes to PDF bytes.
    pub fn convert(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| Error::InvalidPdf(format!("Failed to open DOCX archive: {}", e)))?;

        // Parse document.xml for main content
        let content = self.parse_document(&mut archive)?;

        // Parse metadata from core.xml if available
        let title = self.parse_metadata(&mut archive).unwrap_or_default();

        // Build PDF
        self.build_pdf(&title, &content)
    }

    /// Parse the main document.xml.
    fn parse_document<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<Vec<DocumentParagraph>> {
        let mut paragraphs = Vec::new();

        // Read word/document.xml
        let xml_content = match archive.by_name("word/document.xml") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).map_err(|e| {
                    Error::InvalidPdf(format!("Failed to read document.xml: {}", e))
                })?;
                content
            },
            Err(_) => return Ok(paragraphs),
        };

        // Parse XML
        let mut reader = Reader::from_str(&xml_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_paragraph = DocumentParagraph::default();
        let mut current_run = TextRun::default();
        let mut in_paragraph = false;
        let mut in_run = false;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                    b"p" => {
                        in_paragraph = true;
                        current_paragraph = DocumentParagraph::default();
                        self.parse_paragraph_properties(e, &mut current_paragraph);
                    },
                    b"r" => {
                        in_run = true;
                        current_run = TextRun::default();
                    },
                    b"t" => {
                        in_text = true;
                    },
                    b"pPr" => {
                        // Paragraph properties - handled in element parsing
                    },
                    b"rPr" => {
                        // Run properties - handled in element parsing
                    },
                    b"b" | b"bCs" => {
                        if in_run {
                            current_run.style.bold = true;
                        }
                    },
                    b"i" | b"iCs" => {
                        if in_run {
                            current_run.style.italic = true;
                        }
                    },
                    b"u" => {
                        if in_run {
                            current_run.style.underline = true;
                        }
                    },
                    b"strike" => {
                        if in_run {
                            current_run.style.strikethrough = true;
                        }
                    },
                    b"sz" => {
                        if in_run {
                            if let Some(size) = get_attribute(e, "val") {
                                if let Ok(half_pts) = size.parse::<i32>() {
                                    current_run.style.font_size =
                                        Some(half_points_to_points(half_pts));
                                }
                            }
                        }
                    },
                    b"color" => {
                        if in_run {
                            if let Some(val) = get_attribute(e, "val") {
                                if val != "auto" {
                                    current_run.style.color = parse_color(&val);
                                }
                            }
                        }
                    },
                    b"jc" => {
                        if in_paragraph {
                            if let Some(val) = get_attribute(e, "val") {
                                current_paragraph.style.alignment = match val.as_str() {
                                    "center" => ParagraphAlignment::Center,
                                    "right" => ParagraphAlignment::Right,
                                    "both" => ParagraphAlignment::Justify,
                                    _ => ParagraphAlignment::Left,
                                };
                            }
                        }
                    },
                    b"pStyle" => {
                        if in_paragraph {
                            if let Some(val) = get_attribute(e, "val") {
                                // Detect heading styles
                                if val.starts_with("Heading") || val.starts_with("heading") {
                                    let level = val
                                        .chars()
                                        .filter(|c| c.is_ascii_digit())
                                        .collect::<String>()
                                        .parse::<u8>()
                                        .unwrap_or(1);
                                    current_paragraph.style.heading_level = Some(level);
                                }
                            }
                        }
                    },
                    b"numPr" => {
                        // Numbering properties - marks this as a list item
                        if in_paragraph {
                            current_paragraph.is_list_item = true;
                        }
                    },
                    b"ilvl" => {
                        if in_paragraph && current_paragraph.is_list_item {
                            if let Some(val) = get_attribute(e, "val") {
                                current_paragraph.style.list_level = val.parse().ok();
                            }
                        }
                    },
                    _ => {},
                },
                Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                    b"p" => {
                        in_paragraph = false;
                        if !current_paragraph.runs.is_empty() || current_paragraph.is_empty_line {
                            paragraphs.push(std::mem::take(&mut current_paragraph));
                        }
                    },
                    b"r" => {
                        in_run = false;
                        if !current_run.text.is_empty() {
                            current_paragraph
                                .runs
                                .push(std::mem::take(&mut current_run));
                        }
                    },
                    b"t" => {
                        in_text = false;
                    },
                    _ => {},
                },
                Ok(Event::Empty(ref e)) => match e.local_name().as_ref() {
                    b"b" | b"bCs" => {
                        if in_run {
                            current_run.style.bold = true;
                        }
                    },
                    b"i" | b"iCs" => {
                        if in_run {
                            current_run.style.italic = true;
                        }
                    },
                    b"u" => {
                        if in_run {
                            current_run.style.underline = true;
                        }
                    },
                    b"strike" => {
                        if in_run {
                            current_run.style.strikethrough = true;
                        }
                    },
                    b"sz" => {
                        if in_run {
                            if let Some(size) = get_attribute(e, "val") {
                                if let Ok(half_pts) = size.parse::<i32>() {
                                    current_run.style.font_size =
                                        Some(half_points_to_points(half_pts));
                                }
                            }
                        }
                    },
                    b"br" => {
                        // Line break within run
                        if in_run {
                            current_run.text.push('\n');
                        }
                    },
                    b"tab" => {
                        // Tab character
                        if in_run {
                            current_run.text.push('\t');
                        }
                    },
                    _ => {},
                },
                Ok(Event::Text(e)) => {
                    if in_text && in_run {
                        current_run
                            .text
                            .push_str(&e.xml11_content().unwrap_or_default());
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(Error::InvalidPdf(format!("XML parse error: {}", e)));
                },
                _ => {},
            }
            buf.clear();
        }

        Ok(paragraphs)
    }

    /// Parse paragraph properties from the <w:p> element.
    fn parse_paragraph_properties(&self, _e: &BytesStart, _para: &mut DocumentParagraph) {
        // Properties are typically in child elements, handled in the main loop
    }

    /// Parse document metadata from docProps/core.xml.
    fn parse_metadata<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Option<String> {
        let xml_content = match archive.by_name("docProps/core.xml") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).ok()?;
                content
            },
            Err(_) => return None,
        };

        // Simple title extraction
        let mut reader = Reader::from_str(&xml_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_title = false;
        let mut title = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"title" {
                        in_title = true;
                    }
                },
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"title" {
                        in_title = false;
                    }
                },
                Ok(Event::Text(e)) => {
                    if in_title {
                        title = Some(e.xml11_content().unwrap_or_default().to_string());
                    }
                },
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {},
            }
            buf.clear();
        }

        title
    }

    /// Build PDF from parsed content.
    fn build_pdf(&self, title: &str, paragraphs: &[DocumentParagraph]) -> Result<Vec<u8>> {
        let metadata = DocumentMetadata::new()
            .title(if title.is_empty() { "Document" } else { title })
            .creator("pdf_oxide");

        let mut builder = DocumentBuilder::new().metadata(metadata);

        let (_page_width, page_height) = self.config.page_size.dimensions();
        let base_line_height = self.config.default_font_size * self.config.line_height;

        // Pre-process paragraphs into render instructions
        #[derive(Clone)]
        enum RenderOp {
            NewPage,
            Heading {
                level: u8,
                text: String,
                y: f32,
            },
            Text {
                x: f32,
                y: f32,
                text: String,
                font: String,
                size: f32,
            },
        }

        let mut ops: Vec<RenderOp> = Vec::new();
        let mut current_y = page_height - self.config.margins.top;

        for para in paragraphs {
            let max_font_size = para
                .runs
                .iter()
                .filter_map(|r| r.style.font_size)
                .max_by(|a, b| crate::utils::safe_float_cmp(*a, *b))
                .unwrap_or(self.config.default_font_size);

            let line_height = if para.style.heading_level.is_some() {
                max_font_size * 1.5
            } else {
                max_font_size * self.config.line_height
            };

            // Add space before
            if para.style.space_before > 0.0 {
                current_y -= para.style.space_before;
            }

            // Check if we need a new page
            if current_y < self.config.margins.bottom + line_height {
                ops.push(RenderOp::NewPage);
                current_y = page_height - self.config.margins.top;
            }

            // Handle headings
            if let Some(level) = para.style.heading_level {
                ops.push(RenderOp::Heading {
                    level,
                    text: para.get_text(),
                    y: current_y,
                });
                current_y -= line_height;
            } else if para.runs.is_empty() {
                // Empty paragraph - just add spacing
                current_y -= base_line_height;
            } else {
                let mut x = self.config.margins.left;

                if para.is_list_item {
                    let indent = (para.style.list_level.unwrap_or(0) as f32) * 18.0;
                    x += indent;
                    ops.push(RenderOp::Text {
                        x,
                        y: current_y,
                        text: "• ".to_string(),
                        font: self.config.default_font.clone(),
                        size: self.config.default_font_size,
                    });
                    x += 12.0;
                }

                for run in &para.runs {
                    let font_name = run.style.pdf_font_name().to_string();
                    let font_size = run.style.font_size.unwrap_or(self.config.default_font_size);

                    ops.push(RenderOp::Text {
                        x,
                        y: current_y,
                        text: run.text.clone(),
                        font: font_name,
                        size: font_size,
                    });

                    x += run.text.len() as f32 * font_size * 0.5;
                }

                current_y -= line_height;
            }

            // Add space after
            let space_after = if para.style.space_after > 0.0 {
                para.style.space_after
            } else if para.style.heading_level.is_some() {
                base_line_height * 0.5
            } else {
                0.0
            };
            if space_after > 0.0 {
                current_y -= space_after;
            }
        }

        // Render all operations
        let mut page_builder = builder.page(self.config.page_size);
        page_builder =
            page_builder.at(self.config.margins.left, page_height - self.config.margins.top);

        for op in &ops {
            match op {
                RenderOp::NewPage => {
                    page_builder.done();
                    page_builder = builder.page(self.config.page_size);
                },
                RenderOp::Heading { level, text, y } => {
                    page_builder = page_builder
                        .at(self.config.margins.left, *y)
                        .heading(*level, text);
                },
                RenderOp::Text {
                    x,
                    y,
                    text,
                    font,
                    size,
                } => {
                    page_builder = page_builder.at(*x, *y).font(font, *size).text(text);
                },
            }
        }

        page_builder.done();
        builder.build()
    }
}

/// A text run within a paragraph.
#[derive(Debug, Default)]
struct TextRun {
    text: String,
    style: TextStyle,
}

/// A parsed paragraph from the document.
#[derive(Debug, Default)]
struct DocumentParagraph {
    runs: Vec<TextRun>,
    style: ParagraphStyle,
    is_list_item: bool,
    is_empty_line: bool,
}

impl DocumentParagraph {
    /// Get the full text content of the paragraph.
    fn get_text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

/// Helper to get an attribute value from an XML element.
fn get_attribute(e: &BytesStart, name: &str) -> Option<String> {
    // Check both with and without namespace prefix
    for attr in e.attributes().flatten() {
        let key = attr.key.local_name();
        if key.as_ref() == name.as_bytes() {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_text() {
        let mut para = DocumentParagraph::default();
        para.runs.push(TextRun {
            text: "Hello ".to_string(),
            style: TextStyle::default(),
        });
        para.runs.push(TextRun {
            text: "World".to_string(),
            style: TextStyle {
                bold: true,
                ..Default::default()
            },
        });

        assert_eq!(para.get_text(), "Hello World");
    }

    #[test]
    fn test_text_run_default() {
        let run = TextRun::default();
        assert!(run.text.is_empty());
        assert!(!run.style.bold);
    }
}
