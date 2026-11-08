//! Input parsers for the PDF writing pipeline.
//!
//! This module provides the InputParser trait and implementations for
//! parsing various input formats into ContentElements that can be written to PDF.
//!
//! # Available Parsers
//!
//! - [`MarkdownParser`]: Parse Markdown format
//! - [`PlainTextParser`]: Parse plain text
//! - [`HtmlParser`]: Parse HTML format
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::pipeline::input_parsers::{InputParser, MarkdownParser};
//!
//! let parser = MarkdownParser::new();
//! let elements = parser.parse("# Hello World\n\nThis is a paragraph.")?;
//! ```

mod html;
mod markdown;
mod plaintext;

pub use html::HtmlParser;
pub use markdown::MarkdownParser;
pub use plaintext::PlainTextParser;

use crate::elements::ContentElement;
use crate::error::Result;

/// Configuration for input parsing.
#[derive(Debug, Clone)]
pub struct InputParserConfig {
    /// Default font name for text
    pub default_font: String,
    /// Default font size in points
    pub default_font_size: f32,
    /// Default page width in points (72 points = 1 inch)
    pub page_width: f32,
    /// Default page height in points
    pub page_height: f32,
    /// Left margin in points
    pub margin_left: f32,
    /// Right margin in points
    pub margin_right: f32,
    /// Top margin in points
    pub margin_top: f32,
    /// Bottom margin in points
    pub margin_bottom: f32,
    /// Line height multiplier (relative to font size)
    pub line_height: f32,
    /// Paragraph spacing in points
    pub paragraph_spacing: f32,
}

impl Default for InputParserConfig {
    fn default() -> Self {
        Self {
            default_font: "Helvetica".to_string(),
            default_font_size: 12.0,
            // US Letter size
            page_width: 612.0,  // 8.5 inches
            page_height: 792.0, // 11 inches
            margin_left: 72.0,  // 1 inch
            margin_right: 72.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
            line_height: 1.2,
            paragraph_spacing: 12.0,
        }
    }
}

impl InputParserConfig {
    /// Create config for A4 paper size.
    pub fn a4() -> Self {
        Self {
            page_width: 595.0,  // 210mm
            page_height: 842.0, // 297mm
            ..Default::default()
        }
    }

    /// Set the default font.
    pub fn with_font(mut self, font: impl Into<String>) -> Self {
        self.default_font = font.into();
        self
    }

    /// Set the default font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.default_font_size = size;
        self
    }

    /// Set margins uniformly.
    pub fn with_margins(mut self, margin: f32) -> Self {
        self.margin_left = margin;
        self.margin_right = margin;
        self.margin_top = margin;
        self.margin_bottom = margin;
        self
    }

    /// Get the usable content width.
    pub fn content_width(&self) -> f32 {
        self.page_width - self.margin_left - self.margin_right
    }

    /// Get the usable content height.
    pub fn content_height(&self) -> f32 {
        self.page_height - self.margin_top - self.margin_bottom
    }

    /// Get the Y coordinate for the start of content (top of content area).
    /// PDF coordinates have origin at bottom-left, so this is page_height - margin_top.
    pub fn content_start_y(&self) -> f32 {
        self.page_height - self.margin_top
    }
}

/// Trait for parsing input formats into ContentElements.
///
/// Implementations transform text input (Markdown, HTML, plain text, etc.)
/// into a sequence of ContentElements suitable for PDF generation.
///
/// This is the symmetric counterpart to OutputConverter, enabling
/// round-trip operations: PDF → String → PDF
pub trait InputParser: Send + Sync {
    /// Parse input text into content elements.
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to parse
    /// * `config` - Configuration for parsing and layout
    ///
    /// # Returns
    ///
    /// A vector of ContentElements representing the parsed content.
    fn parse(&self, input: &str, config: &InputParserConfig) -> Result<Vec<ContentElement>>;

    /// Return the name of this parser for debugging.
    fn name(&self) -> &'static str;

    /// Return the MIME type this parser handles.
    fn mime_type(&self) -> &'static str;

    /// Return common file extensions for this format.
    fn extensions(&self) -> &[&'static str];
}

/// Create a parser based on the input format name.
pub fn create_parser(format: &str) -> Option<Box<dyn InputParser>> {
    match format.to_lowercase().as_str() {
        "markdown" | "md" => Some(Box::new(MarkdownParser::new())),
        "text" | "plain" | "txt" => Some(Box::new(PlainTextParser::new())),
        "html" | "htm" => Some(Box::new(HtmlParser::new())),
        _ => None,
    }
}

/// Detect the parser based on file extension.
pub fn parser_for_extension(extension: &str) -> Option<Box<dyn InputParser>> {
    match extension.to_lowercase().as_str() {
        "md" | "markdown" => Some(Box::new(MarkdownParser::new())),
        "txt" | "text" => Some(Box::new(PlainTextParser::new())),
        "html" | "htm" => Some(Box::new(HtmlParser::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InputParserConfig::default();
        assert_eq!(config.page_width, 612.0);
        assert_eq!(config.page_height, 792.0);
        assert_eq!(config.default_font_size, 12.0);
    }

    #[test]
    fn test_a4_config() {
        let config = InputParserConfig::a4();
        assert_eq!(config.page_width, 595.0);
        assert_eq!(config.page_height, 842.0);
    }

    #[test]
    fn test_content_dimensions() {
        let config = InputParserConfig::default();
        // 612 - 72 - 72 = 468
        assert_eq!(config.content_width(), 468.0);
        // 792 - 72 - 72 = 648
        assert_eq!(config.content_height(), 648.0);
    }

    #[test]
    fn test_create_parser_markdown() {
        let parser = create_parser("markdown").unwrap();
        assert_eq!(parser.name(), "MarkdownParser");
        assert_eq!(parser.mime_type(), "text/markdown");
    }

    #[test]
    fn test_create_parser_text() {
        let parser = create_parser("text").unwrap();
        assert_eq!(parser.name(), "PlainTextParser");
        assert_eq!(parser.mime_type(), "text/plain");
    }

    #[test]
    fn test_create_parser_unknown() {
        assert!(create_parser("unknown").is_none());
    }

    #[test]
    fn test_parser_for_extension() {
        assert!(parser_for_extension("md").is_some());
        assert!(parser_for_extension("txt").is_some());
        assert!(parser_for_extension("html").is_some());
        assert!(parser_for_extension("htm").is_some());
        assert!(parser_for_extension("pdf").is_none());
    }

    #[test]
    fn test_create_parser_html() {
        let parser = create_parser("html").unwrap();
        assert_eq!(parser.name(), "html");
        assert_eq!(parser.mime_type(), "text/html");
    }

    #[test]
    fn test_content_start_y() {
        let config = InputParserConfig::default();
        // page_height (792) - margin_top (72) = 720
        assert_eq!(config.content_start_y(), 720.0);
    }
}
