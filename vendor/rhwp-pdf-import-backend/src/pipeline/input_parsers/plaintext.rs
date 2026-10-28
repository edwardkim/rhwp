//! Plain text input parser.
//!
//! Parses plain text into ContentElements for PDF generation.

use crate::elements::{ContentElement, FontSpec, TextContent, TextStyle};
use crate::error::Result;
use crate::geometry::Rect;

use super::{InputParser, InputParserConfig};

/// Parser for plain text format.
///
/// Handles plain text with:
/// - Paragraph detection (blank lines separate paragraphs)
/// - Line wrapping (optional)
/// - Consistent spacing
#[derive(Debug, Clone, Default)]
pub struct PlainTextParser {
    /// Whether to preserve original line breaks
    preserve_line_breaks: bool,
    /// Whether to treat double newlines as paragraph breaks
    paragraph_on_blank_line: bool,
}

impl PlainTextParser {
    /// Create a new plain text parser with default settings.
    pub fn new() -> Self {
        Self {
            preserve_line_breaks: false,
            paragraph_on_blank_line: true,
        }
    }

    /// Preserve original line breaks instead of reflowing.
    pub fn with_preserved_line_breaks(mut self, preserve: bool) -> Self {
        self.preserve_line_breaks = preserve;
        self
    }

    /// Set whether blank lines create paragraph breaks.
    pub fn with_paragraph_on_blank_line(mut self, enabled: bool) -> Self {
        self.paragraph_on_blank_line = enabled;
        self
    }

    /// Parse plain text into content elements.
    fn parse_text(&self, input: &str, config: &InputParserConfig) -> Result<Vec<ContentElement>> {
        let mut elements = Vec::new();
        let mut y_position = config.page_height - config.margin_top;
        let mut reading_order = 0;

        let line_height = config.default_font_size * config.line_height;

        if self.preserve_line_breaks {
            // Preserve original line structure
            for line in input.lines() {
                if line.is_empty() {
                    y_position -= config.paragraph_spacing;
                    continue;
                }

                y_position -= line_height;

                let element = self.create_text_element(
                    line,
                    config.margin_left,
                    y_position,
                    config.content_width(),
                    config.default_font_size,
                    &config.default_font,
                    reading_order,
                );
                elements.push(element);
                reading_order += 1;
            }
        } else {
            // Reflow text into paragraphs
            let paragraphs = self.split_paragraphs(input);

            for paragraph in paragraphs {
                if paragraph.is_empty() {
                    y_position -= config.paragraph_spacing;
                    continue;
                }

                y_position -= line_height;

                let element = self.create_text_element(
                    &paragraph,
                    config.margin_left,
                    y_position,
                    config.content_width(),
                    config.default_font_size,
                    &config.default_font,
                    reading_order,
                );
                elements.push(element);
                reading_order += 1;

                y_position -= config.paragraph_spacing;
            }
        }

        Ok(elements)
    }

    /// Split text into paragraphs.
    fn split_paragraphs(&self, input: &str) -> Vec<String> {
        if self.paragraph_on_blank_line {
            // Split on blank lines
            input
                .split("\n\n")
                .map(|p| {
                    // Join lines within paragraph
                    p.lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .filter(|p| !p.is_empty())
                .collect()
        } else {
            // Treat entire input as one paragraph
            let text = input
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            if text.is_empty() {
                Vec::new()
            } else {
                vec![text]
            }
        }
    }

    /// Create a text content element.
    fn create_text_element(
        &self,
        text: &str,
        x: f32,
        y: f32,
        width: f32,
        font_size: f32,
        font_name: &str,
        reading_order: usize,
    ) -> ContentElement {
        let height = font_size;

        ContentElement::Text(TextContent {
            artifact_type: None,
 
            text: text.to_string(),
            bbox: Rect::new(x, y, width, height),
            font: FontSpec::new(font_name, font_size),
            style: TextStyle::default(),
            reading_order: Some(reading_order),
        })
    }
}

impl InputParser for PlainTextParser {
    fn parse(&self, input: &str, config: &InputParserConfig) -> Result<Vec<ContentElement>> {
        self.parse_text(input, config)
    }

    fn name(&self) -> &'static str {
        "PlainTextParser"
    }

    fn mime_type(&self) -> &'static str {
        "text/plain"
    }

    fn extensions(&self) -> &[&'static str] {
        &["txt", "text"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let parser = PlainTextParser::new();
        let config = InputParserConfig::default();

        let input = "Hello World";
        let elements = parser.parse(input, &config).unwrap();

        assert_eq!(elements.len(), 1);
        if let ContentElement::Text(text) = &elements[0] {
            assert_eq!(text.text, "Hello World");
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_parse_multiple_paragraphs() {
        let parser = PlainTextParser::new();
        let config = InputParserConfig::default();

        let input = "First paragraph.\n\nSecond paragraph.";
        let elements = parser.parse(input, &config).unwrap();

        assert_eq!(elements.len(), 2);

        if let ContentElement::Text(text) = &elements[0] {
            assert_eq!(text.text, "First paragraph.");
        }
        if let ContentElement::Text(text) = &elements[1] {
            assert_eq!(text.text, "Second paragraph.");
        }
    }

    #[test]
    fn test_preserve_line_breaks() {
        let parser = PlainTextParser::new().with_preserved_line_breaks(true);
        let config = InputParserConfig::default();

        let input = "Line 1\nLine 2\nLine 3";
        let elements = parser.parse(input, &config).unwrap();

        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn test_reflow_text() {
        let parser = PlainTextParser::new().with_preserved_line_breaks(false);
        let config = InputParserConfig::default();

        let input = "Line 1\nLine 2\nLine 3";
        let elements = parser.parse(input, &config).unwrap();

        // Should be reflowed into one paragraph
        assert_eq!(elements.len(), 1);

        if let ContentElement::Text(text) = &elements[0] {
            assert_eq!(text.text, "Line 1 Line 2 Line 3");
        }
    }

    #[test]
    fn test_split_paragraphs() {
        let parser = PlainTextParser::new();

        let input = "Para 1 line 1\nPara 1 line 2\n\nPara 2";
        let paragraphs = parser.split_paragraphs(input);

        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0], "Para 1 line 1 Para 1 line 2");
        assert_eq!(paragraphs[1], "Para 2");
    }

    #[test]
    fn test_reading_order() {
        let parser = PlainTextParser::new();
        let config = InputParserConfig::default();

        let input = "First\n\nSecond\n\nThird";
        let elements = parser.parse(input, &config).unwrap();

        for (i, element) in elements.iter().enumerate() {
            if let ContentElement::Text(text) = element {
                assert_eq!(text.reading_order, Some(i));
            }
        }
    }

    #[test]
    fn test_empty_input() {
        let parser = PlainTextParser::new();
        let config = InputParserConfig::default();

        let elements = parser.parse("", &config).unwrap();
        assert!(elements.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let parser = PlainTextParser::new();
        let config = InputParserConfig::default();

        let elements = parser.parse("   \n\n   ", &config).unwrap();
        assert!(elements.is_empty());
    }
}
