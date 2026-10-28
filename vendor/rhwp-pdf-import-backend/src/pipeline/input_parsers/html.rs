//! HTML input parser.
//!
//! Parses HTML into ContentElements for PDF generation.
//! This is a simple parser that handles basic HTML elements.

use super::{InputParser, InputParserConfig};
use crate::elements::{ContentElement, FontSpec, TextContent, TextStyle};
use crate::error::Result;
use crate::geometry::Rect;
use crate::layout::FontWeight;

/// HTML parser for converting HTML to ContentElements.
///
/// Supports basic HTML elements:
/// - Headings (h1-h6)
/// - Paragraphs (p)
/// - Bold (b, strong)
/// - Italic (i, em)
/// - Line breaks (br)
/// - Horizontal rules (hr)
/// - Lists (ul, ol, li)
/// - Code (code, pre)
///
/// Note: This is a simple regex-based parser, not a full HTML parser.
/// For complex HTML, consider using an HTML parsing library.
#[derive(Debug, Default)]
pub struct HtmlParser;

impl HtmlParser {
    /// Create a new HTML parser.
    pub fn new() -> Self {
        Self
    }
}

impl InputParser for HtmlParser {
    fn parse(&self, input: &str, config: &InputParserConfig) -> Result<Vec<ContentElement>> {
        let mut elements = Vec::new();
        let mut y_position = config.content_start_y();
        let x_position = config.margin_left;
        let mut reading_order = 0;

        // Simple HTML processing - strip tags and extract structure
        let cleaned = strip_html_comments(input);
        let mut current_text = String::new();
        let mut in_tag = false;
        let mut tag_name = String::new();
        let mut current_style = TextStyleState::default();

        let chars: Vec<char> = cleaned.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '<' {
                // Flush any accumulated text
                if !current_text.trim().is_empty() {
                    let element = create_text_element(
                        &current_text,
                        x_position,
                        y_position,
                        config,
                        &current_style,
                        reading_order,
                    );
                    y_position -= element.bbox().height * 1.2;
                    elements.push(element);
                    reading_order += 1;
                }
                current_text.clear();

                // Parse tag
                in_tag = true;
                tag_name.clear();
            } else if ch == '>' && in_tag {
                in_tag = false;

                // Process the tag
                let tag = tag_name.trim().to_lowercase();
                let (tag_type, closing) = parse_tag(&tag);

                match tag_type.as_str() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        if !closing {
                            let level: u8 =
                                tag_type.chars().last().unwrap().to_digit(10).unwrap() as u8;
                            current_style.heading_level = Some(level);
                            current_style.bold = level <= 2;
                        } else {
                            current_style.heading_level = None;
                            current_style.bold = false;
                            y_position -= config.default_font_size * 0.5; // Extra space after heading
                        }
                    },
                    "p" => {
                        if closing {
                            y_position -= config.default_font_size * 0.5; // Paragraph spacing
                        }
                    },
                    "br" => {
                        y_position -= config.default_font_size * config.line_height;
                    },
                    "hr" => {
                        // Add horizontal rule
                        let hr_element = create_horizontal_rule(
                            x_position,
                            y_position,
                            config.page_width - config.margin_left - config.margin_right,
                            reading_order,
                        );
                        elements.push(hr_element);
                        reading_order += 1;
                        y_position -= config.default_font_size;
                    },
                    "b" | "strong" => {
                        current_style.bold = !closing;
                    },
                    "i" | "em" => {
                        current_style.italic = !closing;
                    },
                    "code" | "pre" => {
                        current_style.monospace = !closing;
                    },
                    "li" => {
                        if !closing {
                            current_text.push_str("• ");
                        } else {
                            y_position -= config.default_font_size * 0.3; // List item spacing
                        }
                    },
                    "ul" | "ol" => {
                        if closing {
                            y_position -= config.default_font_size * 0.3; // List spacing
                        }
                    },
                    _ => {},
                }

                tag_name.clear();
            } else if in_tag {
                tag_name.push(ch);
            } else {
                // Handle HTML entities
                if ch == '&' {
                    let entity_end = chars[i..].iter().position(|&c| c == ';');
                    if let Some(end) = entity_end {
                        let entity: String = chars[i..i + end + 1].iter().collect();
                        let decoded = decode_html_entity(&entity);
                        current_text.push_str(&decoded);
                        i += end;
                    } else {
                        current_text.push(ch);
                    }
                } else if ch == '\n' || ch == '\r' {
                    // Normalize whitespace
                    if !current_text.ends_with(' ') && !current_text.is_empty() {
                        current_text.push(' ');
                    }
                } else {
                    current_text.push(ch);
                }
            }

            i += 1;
        }

        // Flush remaining text
        if !current_text.trim().is_empty() {
            let element = create_text_element(
                &current_text,
                x_position,
                y_position,
                config,
                &current_style,
                reading_order,
            );
            elements.push(element);
        }

        Ok(elements)
    }

    fn name(&self) -> &'static str {
        "html"
    }

    fn mime_type(&self) -> &'static str {
        "text/html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }
}

/// Internal state for tracking text styling.
#[derive(Debug, Default, Clone)]
struct TextStyleState {
    bold: bool,
    italic: bool,
    monospace: bool,
    heading_level: Option<u8>,
}

/// Parse a tag name, returning (tag_name, is_closing).
fn parse_tag(tag: &str) -> (String, bool) {
    let tag = tag.trim();
    if tag.starts_with('/') {
        (tag[1..].split_whitespace().next().unwrap_or("").to_string(), true)
    } else {
        (tag.split_whitespace().next().unwrap_or("").to_string(), false)
    }
}

/// Strip HTML comments from the input.
fn strip_html_comments(input: &str) -> String {
    let mut result = String::new();
    let mut in_comment = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if !in_comment && i + 3 < chars.len() {
            let slice: String = chars[i..i + 4].iter().collect();
            if slice == "<!--" {
                in_comment = true;
                i += 4;
                continue;
            }
        }

        if in_comment && i + 2 < chars.len() {
            let slice: String = chars[i..i + 3].iter().collect();
            if slice == "-->" {
                in_comment = false;
                i += 3;
                continue;
            }
        }

        if !in_comment {
            result.push(chars[i]);
        }
        i += 1;
    }

    result
}

/// Decode common HTML entities.
fn decode_html_entity(entity: &str) -> String {
    match entity {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&apos;" => "'".to_string(),
        "&nbsp;" => " ".to_string(),
        "&mdash;" | "&emdash;" => "—".to_string(),
        "&ndash;" | "&endash;" => "–".to_string(),
        "&copy;" => "©".to_string(),
        "&reg;" => "®".to_string(),
        "&trade;" => "™".to_string(),
        "&hellip;" => "…".to_string(),
        _ => {
            // Try numeric entities
            if entity.starts_with("&#") && entity.ends_with(';') {
                let num_str = &entity[2..entity.len() - 1];
                if let Some(stripped) = num_str.strip_prefix('x') {
                    // Hex entity
                    if let Ok(code) = u32::from_str_radix(stripped, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            return ch.to_string();
                        }
                    }
                } else {
                    // Decimal entity
                    if let Ok(code) = num_str.parse::<u32>() {
                        if let Some(ch) = char::from_u32(code) {
                            return ch.to_string();
                        }
                    }
                }
            }
            entity.to_string()
        },
    }
}

/// Create a text element with the current styling.
fn create_text_element(
    text: &str,
    x: f32,
    y: f32,
    config: &InputParserConfig,
    style_state: &TextStyleState,
    reading_order: usize,
) -> ContentElement {
    let text = normalize_whitespace(text);

    // Determine font size based on heading level
    let font_size = if let Some(level) = style_state.heading_level {
        match level {
            1 => 24.0,
            2 => 20.0,
            3 => 16.0,
            4 => 14.0,
            _ => 12.0,
        }
    } else {
        config.default_font_size
    };

    // Determine font name
    let font_name = if style_state.monospace {
        if style_state.bold {
            "Courier-Bold"
        } else {
            "Courier"
        }
    } else if style_state.bold && style_state.italic {
        "Helvetica-BoldOblique"
    } else if style_state.bold {
        "Helvetica-Bold"
    } else if style_state.italic {
        "Helvetica-Oblique"
    } else {
        "Helvetica"
    };

    let weight = if style_state.bold {
        FontWeight::Bold
    } else {
        FontWeight::Normal
    };

    ContentElement::Text(TextContent {
        artifact_type: None,
 
        text,
        bbox: Rect::new(
            x, y, 400.0, // Approximate width
            font_size,
        ),
        font: FontSpec {
            name: font_name.to_string(),
            size: font_size,
        },
        style: TextStyle {
            weight,
            italic: style_state.italic,
            ..Default::default()
        },
        reading_order: Some(reading_order),
    })
}

/// Create a horizontal rule element.
fn create_horizontal_rule(x: f32, y: f32, width: f32, reading_order: usize) -> ContentElement {
    use crate::elements::{LineCap, LineJoin, PathContent, PathOperation};
    use crate::layout::Color;

    ContentElement::Path(PathContent {
        operations: vec![
            PathOperation::MoveTo(x, y),
            PathOperation::LineTo(x + width, y),
        ],
        bbox: Rect::new(x, y, width, 1.0),
        stroke_color: Some(Color {
            r: 0.7,
            g: 0.7,
            b: 0.7,
        }),
        fill_color: None,
        stroke_width: 1.0,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,
        reading_order: Some(reading_order),
    })
}

/// Normalize whitespace in text.
fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser_creation() {
        let parser = HtmlParser::new();
        assert_eq!(parser.name(), "html");
        assert_eq!(parser.mime_type(), "text/html");
    }

    #[test]
    fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let config = InputParserConfig::default();
        let html = "<p>Hello, World!</p>";
        let elements = parser.parse(html, &config).unwrap();

        assert!(!elements.is_empty());
        if let ContentElement::Text(text) = &elements[0] {
            assert_eq!(text.text, "Hello, World!");
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_parse_headings() {
        let parser = HtmlParser::new();
        let config = InputParserConfig::default();
        let html = "<h1>Title</h1><p>Content</p>";
        let elements = parser.parse(html, &config).unwrap();

        assert!(elements.len() >= 2);

        // First element should be the heading
        if let ContentElement::Text(text) = &elements[0] {
            assert_eq!(text.text, "Title");
            assert_eq!(text.font.size, 24.0); // h1 size
        } else {
            panic!("Expected text element for heading");
        }
    }

    #[test]
    fn test_parse_bold_italic() {
        let parser = HtmlParser::new();
        let config = InputParserConfig::default();
        let html = "<b>Bold</b> <i>Italic</i>";
        let elements = parser.parse(html, &config).unwrap();

        assert!(!elements.is_empty());
    }

    #[test]
    fn test_html_entities() {
        let parser = HtmlParser::new();
        let config = InputParserConfig::default();
        let html = "<p>&amp; &lt; &gt; &quot;</p>";
        let elements = parser.parse(html, &config).unwrap();

        if let ContentElement::Text(text) = &elements[0] {
            assert!(text.text.contains('&'));
            assert!(text.text.contains('<'));
            assert!(text.text.contains('>'));
        }
    }

    #[test]
    fn test_strip_comments() {
        let input = "Hello <!-- this is a comment --> World";
        let result = strip_html_comments(input);
        assert_eq!(result, "Hello  World");
    }

    #[test]
    fn test_horizontal_rule() {
        let parser = HtmlParser::new();
        let config = InputParserConfig::default();
        let html = "<p>Before</p><hr><p>After</p>";
        let elements = parser.parse(html, &config).unwrap();

        // Should have text, hr, text
        let has_path = elements
            .iter()
            .any(|e| matches!(e, ContentElement::Path(_)));
        assert!(has_path);
    }

    #[test]
    fn test_list_items() {
        let parser = HtmlParser::new();
        let config = InputParserConfig::default();
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let elements = parser.parse(html, &config).unwrap();

        // Should have bullet points
        let texts: Vec<_> = elements
            .iter()
            .filter_map(|e| {
                if let ContentElement::Text(t) = e {
                    Some(&t.text)
                } else {
                    None
                }
            })
            .collect();

        assert!(texts.iter().any(|t| t.contains('•')));
    }

    #[test]
    fn test_numeric_entity() {
        let decoded = decode_html_entity("&#65;");
        assert_eq!(decoded, "A");

        let decoded_hex = decode_html_entity("&#x41;");
        assert_eq!(decoded_hex, "A");
    }

    #[test]
    fn test_parse_tag() {
        let (name, closing) = parse_tag("p");
        assert_eq!(name, "p");
        assert!(!closing);

        let (name, closing) = parse_tag("/p");
        assert_eq!(name, "p");
        assert!(closing);

        let (name, closing) = parse_tag("div class=\"test\"");
        assert_eq!(name, "div");
        assert!(!closing);
    }
}
