//! Markdown input parser.
//!
//! Parses Markdown text into ContentElements for PDF generation.

use crate::elements::{ContentElement, FontSpec, TextContent, TextStyle};
use crate::error::Result;
use crate::geometry::Rect;

use super::{InputParser, InputParserConfig};

/// Parser for Markdown format.
///
/// Supports common Markdown elements:
/// - Headings (# to ######)
/// - Paragraphs
/// - Bold (**text** or __text__)
/// - Italic (*text* or _text_)
/// - Code blocks (``` or indented)
/// - Inline code (`code`)
/// - Lists (- or * or numbered)
/// - Horizontal rules (---, ***, ___)
#[derive(Debug, Clone, Default)]
pub struct MarkdownParser {
    /// Custom heading sizes (H1 to H6)
    heading_sizes: Option<[f32; 6]>,
}

impl MarkdownParser {
    /// Create a new Markdown parser with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom heading sizes.
    pub fn with_heading_sizes(mut self, sizes: [f32; 6]) -> Self {
        self.heading_sizes = Some(sizes);
        self
    }

    /// Get heading size for a given level (1-6).
    fn heading_size(&self, level: usize, base_size: f32) -> f32 {
        if let Some(sizes) = &self.heading_sizes {
            sizes
                .get(level.saturating_sub(1))
                .copied()
                .unwrap_or(base_size)
        } else {
            // Default heading sizes relative to base
            match level {
                1 => base_size * 2.0,
                2 => base_size * 1.5,
                3 => base_size * 1.25,
                4 => base_size * 1.1,
                5 => base_size * 1.0,
                6 => base_size * 0.9,
                _ => base_size,
            }
        }
    }

    /// Parse markdown into content elements.
    fn parse_markdown(
        &self,
        input: &str,
        config: &InputParserConfig,
    ) -> Result<Vec<ContentElement>> {
        let mut elements = Vec::new();
        let mut y_position = config.page_height - config.margin_top;
        let mut reading_order = 0;

        let lines: Vec<&str> = input.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim_end();

            // Skip empty lines
            if line.is_empty() {
                y_position -= config.paragraph_spacing;
                i += 1;
                continue;
            }

            // Check for headings
            if let Some((level, text)) = self.parse_heading(line) {
                let font_size = self.heading_size(level, config.default_font_size);
                let line_height = font_size * config.line_height;

                y_position -= line_height;

                let element = self.create_text_element(
                    text,
                    config.margin_left,
                    y_position,
                    config.content_width(),
                    font_size,
                    &config.default_font,
                    TextStyle::bold(),
                    reading_order,
                );
                elements.push(element);
                reading_order += 1;

                // Extra spacing after headings
                y_position -= config.paragraph_spacing;
                i += 1;
                continue;
            }

            // Check for horizontal rule
            if self.is_horizontal_rule(line) {
                y_position -= config.paragraph_spacing;
                i += 1;
                continue;
            }

            // Check for code block
            if line.starts_with("```") {
                let (code_block, consumed) = self.parse_code_block(&lines[i..]);
                if !code_block.is_empty() {
                    let font_size = config.default_font_size * 0.9;
                    let line_height = font_size * config.line_height;

                    for code_line in code_block.lines() {
                        y_position -= line_height;

                        let element = self.create_text_element(
                            code_line,
                            config.margin_left + 20.0, // Indent code
                            y_position,
                            config.content_width() - 20.0,
                            font_size,
                            "Courier",
                            TextStyle::default(),
                            reading_order,
                        );
                        elements.push(element);
                        reading_order += 1;
                    }

                    y_position -= config.paragraph_spacing;
                }
                i += consumed;
                continue;
            }

            // Check for list item
            if let Some(text) = self.parse_list_item(line) {
                let line_height = config.default_font_size * config.line_height;
                y_position -= line_height;

                // Add bullet
                let bullet_element = self.create_text_element(
                    "\u{2022}", // Bullet character
                    config.margin_left,
                    y_position,
                    20.0,
                    config.default_font_size,
                    &config.default_font,
                    TextStyle::default(),
                    reading_order,
                );
                elements.push(bullet_element);
                reading_order += 1;

                // Add list item text
                let text_element = self.create_text_element(
                    text,
                    config.margin_left + 20.0,
                    y_position,
                    config.content_width() - 20.0,
                    config.default_font_size,
                    &config.default_font,
                    TextStyle::default(),
                    reading_order,
                );
                elements.push(text_element);
                reading_order += 1;

                i += 1;
                continue;
            }

            // Regular paragraph
            let paragraph = self.collect_paragraph(&lines[i..]);
            if !paragraph.is_empty() {
                let parsed_spans = self.parse_inline_formatting(&paragraph);
                let line_height = config.default_font_size * config.line_height;
                y_position -= line_height;

                for (text, style) in parsed_spans {
                    let element = self.create_text_element(
                        &text,
                        config.margin_left,
                        y_position,
                        config.content_width(),
                        config.default_font_size,
                        &config.default_font,
                        style,
                        reading_order,
                    );
                    elements.push(element);
                    reading_order += 1;
                }

                y_position -= config.paragraph_spacing;
            }

            // Skip consumed lines
            while i < lines.len() && !lines[i].is_empty() {
                i += 1;
            }
            i += 1;
        }

        Ok(elements)
    }

    /// Parse a heading line, returning level and text.
    fn parse_heading<'a>(&self, line: &'a str) -> Option<(usize, &'a str)> {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            return None;
        }

        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level > 6 || level == 0 {
            return None;
        }

        let text = trimmed[level..].trim();
        if text.is_empty() {
            return None;
        }

        Some((level, text))
    }

    /// Check if a line is a horizontal rule.
    fn is_horizontal_rule(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.len() < 3 {
            return false;
        }

        let chars: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.is_empty() {
            return false;
        }

        let first = chars[0];
        (first == '-' || first == '*' || first == '_') && chars.iter().all(|&c| c == first)
    }

    /// Parse a code block, returning content and lines consumed.
    fn parse_code_block(&self, lines: &[&str]) -> (String, usize) {
        if lines.is_empty() || !lines[0].trim_start().starts_with("```") {
            return (String::new(), 0);
        }

        let mut content = String::new();
        let mut consumed = 1;

        for line in &lines[1..] {
            consumed += 1;
            if line.trim_start().starts_with("```") {
                break;
            }
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(line);
        }

        (content, consumed)
    }

    /// Parse a list item, returning the text without the marker.
    fn parse_list_item<'a>(&self, line: &'a str) -> Option<&'a str> {
        let trimmed = line.trim_start();

        // Unordered list
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            return Some(trimmed[2..].trim_start());
        }

        // Ordered list (1. 2. etc.)
        let mut chars = trimmed.chars().peekable();
        let mut has_digit = false;

        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                has_digit = true;
                chars.next();
            } else {
                break;
            }
        }

        if has_digit {
            if let Some('.') = chars.next() {
                if let Some(' ') = chars.next() {
                    let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
                    return Some(trimmed[digit_count + 2..].trim_start());
                }
            }
        }

        None
    }

    /// Collect a paragraph from consecutive non-empty lines.
    fn collect_paragraph(&self, lines: &[&str]) -> String {
        let mut paragraph = String::new();

        for line in lines {
            if line.is_empty() {
                break;
            }
            // Skip special lines
            if self.parse_heading(line).is_some()
                || self.is_horizontal_rule(line)
                || line.trim_start().starts_with("```")
                || self.parse_list_item(line).is_some()
            {
                break;
            }

            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(line.trim());
        }

        paragraph
    }

    /// Parse inline formatting (bold, italic, code).
    fn parse_inline_formatting(&self, text: &str) -> Vec<(String, TextStyle)> {
        // Simple implementation - returns single span with detected style
        // A full implementation would handle nested formatting
        let mut result = Vec::new();

        // Check for bold
        if (text.starts_with("**") && text.ends_with("**") && text.len() > 4)
            || (text.starts_with("__") && text.ends_with("__") && text.len() > 4)
        {
            result.push((text[2..text.len() - 2].to_string(), TextStyle::bold()));
            return result;
        }

        // Check for italic
        if (text.starts_with('*') && text.ends_with('*') && text.len() > 2)
            || (text.starts_with('_') && text.ends_with('_') && text.len() > 2)
        {
            result.push((text[1..text.len() - 1].to_string(), TextStyle::italic()));
            return result;
        }

        // Default: plain text
        result.push((text.to_string(), TextStyle::default()));
        result
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
        style: TextStyle,
        reading_order: usize,
    ) -> ContentElement {
        // Estimate height based on font size
        let height = font_size;

        ContentElement::Text(TextContent {
            artifact_type: None,
 
            text: text.to_string(),
            bbox: Rect::new(x, y, width, height),
            font: FontSpec::new(font_name, font_size),
            style,
            reading_order: Some(reading_order),
        })
    }
}

impl InputParser for MarkdownParser {
    fn parse(&self, input: &str, config: &InputParserConfig) -> Result<Vec<ContentElement>> {
        self.parse_markdown(input, config)
    }

    fn name(&self) -> &'static str {
        "MarkdownParser"
    }

    fn mime_type(&self) -> &'static str {
        "text/markdown"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let parser = MarkdownParser::new();

        assert_eq!(parser.parse_heading("# Title"), Some((1, "Title")));
        assert_eq!(parser.parse_heading("## Section"), Some((2, "Section")));
        assert_eq!(parser.parse_heading("### Subsection"), Some((3, "Subsection")));
        assert_eq!(parser.parse_heading("Not a heading"), None);
        assert_eq!(parser.parse_heading("####### Too many"), None);
    }

    #[test]
    fn test_is_horizontal_rule() {
        let parser = MarkdownParser::new();

        assert!(parser.is_horizontal_rule("---"));
        assert!(parser.is_horizontal_rule("***"));
        assert!(parser.is_horizontal_rule("___"));
        assert!(parser.is_horizontal_rule("- - -"));
        assert!(!parser.is_horizontal_rule("--"));
        assert!(!parser.is_horizontal_rule("text"));
    }

    #[test]
    fn test_parse_list_item() {
        let parser = MarkdownParser::new();

        assert_eq!(parser.parse_list_item("- Item"), Some("Item"));
        assert_eq!(parser.parse_list_item("* Item"), Some("Item"));
        assert_eq!(parser.parse_list_item("1. First"), Some("First"));
        assert_eq!(parser.parse_list_item("10. Tenth"), Some("Tenth"));
        assert_eq!(parser.parse_list_item("Not a list"), None);
    }

    #[test]
    fn test_parse_code_block() {
        let parser = MarkdownParser::new();

        let lines = ["```rust", "let x = 1;", "```"];
        let (content, consumed) = parser.parse_code_block(&lines);
        assert_eq!(content, "let x = 1;");
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_parse_simple_markdown() {
        let parser = MarkdownParser::new();
        let config = InputParserConfig::default();

        let input = "# Hello World\n\nThis is a paragraph.";
        let elements = parser.parse(input, &config).unwrap();

        assert!(elements.len() >= 2); // At least heading and paragraph

        // First element should be heading
        if let ContentElement::Text(text) = &elements[0] {
            assert_eq!(text.text, "Hello World");
            assert!(text.style.weight.is_bold());
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_heading_sizes() {
        let parser = MarkdownParser::new();
        let base = 12.0;

        assert_eq!(parser.heading_size(1, base), 24.0); // 2x
        assert_eq!(parser.heading_size(2, base), 18.0); // 1.5x
        assert_eq!(parser.heading_size(3, base), 15.0); // 1.25x
    }

    #[test]
    fn test_custom_heading_sizes() {
        let parser = MarkdownParser::new().with_heading_sizes([36.0, 28.0, 22.0, 18.0, 14.0, 12.0]);

        assert_eq!(parser.heading_size(1, 12.0), 36.0);
        assert_eq!(parser.heading_size(2, 12.0), 28.0);
    }
}
