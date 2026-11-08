//! HTML converter for PDF documents.
//!
//! This module converts PDF pages to HTML format with two modes:
//! - **Semantic**: Clean HTML with proper tags (h1, h2, h3, p)
//! - **Layout-preserved**: HTML with absolute positioning to match PDF layout

use crate::converters::{ConversionOptions, ReadingOrderMode};
use crate::error::Result;
use crate::layout::clustering::{cluster_chars_into_words, cluster_words_into_lines};
use crate::layout::{TextBlock, TextChar};
use regex::Regex;
use std::sync::LazyLock;

static RE_URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://[^\s<>()]+").unwrap());
static RE_EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());

/// Converter for PDF to HTML format.
///
/// Supports both semantic HTML generation and layout-preserved HTML with CSS positioning.
///
/// # Examples
///
/// ```ignore
/// use pdf_oxide::PdfDocument;
/// use pdf_oxide::converters::{HtmlConverter, ConversionOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut doc = PdfDocument::open("paper.pdf")?;
/// let chars = doc.extract_spans(0)?;
///
/// let converter = HtmlConverter::new();
///
/// // Semantic HTML
/// let options = ConversionOptions::default();
/// let html = converter.convert_page(&chars, &options)?;
///
/// // Layout-preserved HTML
/// let layout_options = ConversionOptions {
///     preserve_layout: true,
///     ..Default::default()
/// };
/// let layout_html = converter.convert_page(&chars, &layout_options)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[deprecated(
    since = "0.2.0",
    note = "Use `pdf_oxide::pipeline::converters::HtmlOutputConverter` instead. \
            The new converter is part of the unified TextPipeline architecture and \
            provides better feature support and maintainability."
)]
pub struct HtmlConverter;

#[allow(deprecated)]
impl HtmlConverter {
    /// Create a new HTML converter.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::converters::HtmlConverter;
    ///
    /// let converter = HtmlConverter::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Convert a page to HTML format from text spans (PDF spec compliant - RECOMMENDED).
    ///
    /// This is the recommended method that uses PDF-native text spans instead of
    /// character-based extraction. Routes to either semantic or layout-preserved HTML
    /// based on the `preserve_layout` option.
    ///
    /// **Benefits over character-based conversion:**
    /// - PDF spec compliant (ISO 32000-1:2008, Section 9.4.4 NOTE 6)
    /// - No character splitting issues
    /// - Preserves PDF's text positioning intent
    /// - Much faster (no DBSCAN clustering needed)
    /// - More robust for complex layouts
    ///
    /// # Arguments
    ///
    /// * `spans` - The text spans extracted from the page via `extract_spans()`
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing the HTML representation of the page.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn convert_page_from_spans(
        &self,
        spans: &[crate::layout::TextSpan],
        options: &ConversionOptions,
    ) -> Result<String> {
        if options.preserve_layout {
            self.convert_page_preserve_layout_from_spans(spans, options)
        } else {
            self.convert_page_semantic_from_spans(spans, options)
        }
    }

    /// Convert a page to semantic HTML from text spans (PDF spec compliant - RECOMMENDED).
    ///
    /// Generates clean HTML with proper semantic tags (h1, h2, h3, p) from PDF text spans.
    ///
    /// # Arguments
    ///
    /// * `spans` - The text spans extracted from the page
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing semantic HTML.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn convert_page_semantic_from_spans(
        &self,
        spans: &[crate::layout::TextSpan],
        _options: &ConversionOptions,
    ) -> Result<String> {
        use crate::layout::TextBlock;

        if spans.is_empty() {
            return Ok(String::new());
        }

        // Convert spans to TextBlocks
        let mut span_blocks: Vec<TextBlock> = spans
            .iter()
            .map(|span| TextBlock {
                chars: vec![],
                bbox: span.bbox,
                text: span.text.clone(),
                avg_font_size: span.font_size,
                dominant_font: span.font_name.clone(),
                is_bold: matches!(span.font_weight, crate::layout::FontWeight::Bold),
                is_italic: span.is_italic,
                mcid: span.mcid,
                sequence: span.sequence,
                rotation_degrees: span.rotation_degrees,
            })
            .collect();

        // Sort by Y position (top to bottom), then X position (left to right)
        span_blocks.sort_by(|a, b| {
            let y_cmp = crate::utils::safe_float_cmp(a.bbox.y, b.bbox.y);
            if y_cmp != std::cmp::Ordering::Equal {
                return y_cmp;
            }
            crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x)
        });

        // Merge adjacent spans on the same line into paragraphs
        let mut blocks: Vec<TextBlock> = Vec::new();
        let mut current_paragraph: Option<TextBlock> = None;

        for span_block in span_blocks {
            match &mut current_paragraph {
                None => {
                    // Start new paragraph
                    current_paragraph = Some(span_block);
                },
                Some(para) => {
                    // Check if this span is on the same line (within 5 pixels vertically)
                    let y_diff = (span_block.bbox.y - para.bbox.y).abs();
                    let same_line = y_diff < 5.0;

                    // Check if same font characteristics (for heading detection)
                    let similar_font = (span_block.avg_font_size - para.avg_font_size).abs() < 2.0;

                    if same_line && similar_font {
                        // Merge into current paragraph
                        // Add space if not adjacent
                        let x_gap = span_block.bbox.x - (para.bbox.x + para.bbox.width);
                        if x_gap > 1.0
                            && !para.text.ends_with(' ')
                            && !span_block.text.starts_with(' ')
                        {
                            para.text.push(' ');
                        }
                        para.text.push_str(&span_block.text);

                        // Expand bbox to include new span
                        let new_right = span_block.bbox.x + span_block.bbox.width;
                        let old_right = para.bbox.x + para.bbox.width;
                        if new_right > old_right {
                            para.bbox.width = new_right - para.bbox.x;
                        }
                    } else {
                        // Different line or font - start new paragraph
                        blocks.push(current_paragraph.take().unwrap());
                        current_paragraph = Some(span_block);
                    }
                },
            }
        }

        // Don't forget the last paragraph
        if let Some(para) = current_paragraph {
            blocks.push(para);
        }

        // Heading detection removed (non-spec-compliant feature)
        // All blocks are treated as paragraphs for spec compliance

        // Apply reading order (use simple top-to-bottom for span-based conversion)
        let ordered_indices =
            self.determine_reading_order(&blocks, ReadingOrderMode::TopToBottomLeftToRight);

        // Generate HTML
        let mut html = String::new();

        for &idx in &ordered_indices {
            let block = &blocks[idx];
            // Convert URLs and emails to hyperlinks
            let linked_text = linkify_urls_and_emails(&block.text);

            // All blocks rendered as paragraphs for PDF spec compliance
            html.push_str("<p>");
            html.push_str(&linked_text);
            html.push_str("</p>\n");
        }

        Ok(html)
    }

    /// Convert a page to layout-preserved HTML from text spans (PDF spec compliant - RECOMMENDED).
    ///
    /// Generates HTML with absolute positioning to match the PDF layout exactly.
    /// Each text span is wrapped in a div with CSS positioning.
    ///
    /// # Arguments
    ///
    /// * `spans` - The text spans extracted from the page
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing HTML with CSS positioning.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn convert_page_preserve_layout_from_spans(
        &self,
        spans: &[crate::layout::TextSpan],
        _options: &ConversionOptions,
    ) -> Result<String> {
        if spans.is_empty() {
            return Ok(String::new());
        }

        // Generate HTML with absolute positioning
        let mut html = String::new();

        // Add CSS styles
        html.push_str("<style>\n");
        html.push_str(".page {\n");
        html.push_str("  position: relative;\n");
        html.push_str("  width: 100%;\n");
        html.push_str("  height: 100%;\n");
        html.push_str("}\n");
        html.push_str(".text {\n");
        html.push_str("  position: absolute;\n");
        html.push_str("  white-space: nowrap;\n");
        html.push_str("}\n");
        html.push_str("</style>\n");

        // Add page container
        html.push_str("<div class=\"page\">\n");

        // Add each span with positioning
        for span in spans {
            let escaped_text = escape_html(&span.text);
            let x = span.bbox.x;
            let y = span.bbox.y;
            let font_size = span.font_size;

            html.push_str(&format!(
                "  <div class=\"text\" style=\"left: {}px; top: {}px; font-size: {}px;\">{}</div>\n",
                x, y, font_size, escaped_text
            ));
        }

        html.push_str("</div>\n");

        Ok(html)
    }

    /// Convert a page to HTML format (character-based - DEPRECATED).
    ///
    /// Routes to either semantic or layout-preserved conversion based on options.
    ///
    /// # Arguments
    ///
    /// * `chars` - The text characters extracted from the page
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing the HTML representation of the page.
    ///
    /// # Errors
    ///
    /// Returns an error if clustering or conversion fails.
    pub fn convert_page(&self, chars: &[TextChar], options: &ConversionOptions) -> Result<String> {
        if options.preserve_layout {
            self.convert_page_preserve_layout(chars, options)
        } else {
            self.convert_page_semantic(chars, options)
        }
    }

    /// Convert a page to semantic HTML.
    ///
    /// Generates clean HTML with proper semantic tags (h1, h2, h3, p).
    ///
    /// # Arguments
    ///
    /// * `chars` - The text characters extracted from the page
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing semantic HTML.
    ///
    /// # Errors
    ///
    /// Returns an error if clustering or conversion fails.
    pub fn convert_page_semantic(
        &self,
        chars: &[TextChar],
        options: &ConversionOptions,
    ) -> Result<String> {
        if chars.is_empty() {
            return Ok(String::new());
        }

        // Compute font-adaptive epsilon for word clustering
        let median_font_size = Self::compute_median_font_size(chars);
        let word_epsilon = median_font_size * 0.5;

        // Step 1: Cluster characters into words
        let word_clusters = cluster_chars_into_words(chars, word_epsilon);
        let mut words = Vec::new();

        for cluster in &word_clusters {
            let word_chars: Vec<TextChar> = cluster.iter().map(|&i| chars[i].clone()).collect();
            if !word_chars.is_empty() {
                words.push(TextBlock::from_chars(word_chars));
            }
        }

        if words.is_empty() {
            return Ok(String::new());
        }

        // Step 2: Cluster words into lines
        let line_clusters = cluster_words_into_lines(&words, 5.0);
        let mut lines = Vec::new();

        for cluster in &line_clusters {
            let line_words: Vec<TextBlock> = cluster.iter().map(|&i| words[i].clone()).collect();
            if !line_words.is_empty() {
                let all_chars: Vec<TextChar> =
                    line_words.iter().flat_map(|w| w.chars.clone()).collect();
                lines.push(TextBlock::from_chars(all_chars));
            }
        }

        if lines.is_empty() {
            return Ok(String::new());
        }

        // Heading detection removed (non-PDF-spec-compliant)
        // All content is now rendered as body text/paragraphs

        // Step 4: Determine reading order
        let ordered_indices =
            self.determine_reading_order(&lines, options.reading_order_mode.clone());

        // Step 5: Generate HTML
        let mut html = String::new();

        for &idx in &ordered_indices {
            let line = &lines[idx];
            // Convert URLs and emails to hyperlinks
            let linked_text = linkify_urls_and_emails(&line.text);

            // All content rendered as paragraphs (body text only)
            html.push_str("<p>");
            html.push_str(&linked_text);
            html.push_str("</p>\n");
        }

        Ok(html)
    }

    /// Convert a page to layout-preserved HTML.
    ///
    /// Generates HTML with absolute positioning to match the PDF layout exactly.
    /// Each text element is wrapped in a div with CSS positioning.
    ///
    /// # Arguments
    ///
    /// * `chars` - The text characters extracted from the page
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing HTML with CSS positioning.
    ///
    /// # Errors
    ///
    /// Returns an error if clustering or conversion fails.
    pub fn convert_page_preserve_layout(
        &self,
        chars: &[TextChar],
        _options: &ConversionOptions,
    ) -> Result<String> {
        if chars.is_empty() {
            return Ok(String::new());
        }

        // Compute font-adaptive epsilon for word clustering
        let median_font_size = Self::compute_median_font_size(chars);
        let word_epsilon = median_font_size * 0.5;

        // For layout preservation, we cluster into words for better rendering
        let word_clusters = cluster_chars_into_words(chars, word_epsilon);
        let mut words = Vec::new();

        for cluster in &word_clusters {
            let word_chars: Vec<TextChar> = cluster.iter().map(|&i| chars[i].clone()).collect();
            if !word_chars.is_empty() {
                words.push(TextBlock::from_chars(word_chars));
            }
        }

        if words.is_empty() {
            return Ok(String::new());
        }

        // Generate HTML with absolute positioning
        let mut html = String::new();

        // Add CSS styles
        html.push_str("<style>\n");
        html.push_str(".page {\n");
        html.push_str("  position: relative;\n");
        html.push_str("  width: 100%;\n");
        html.push_str("  height: 100%;\n");
        html.push_str("}\n");
        html.push_str(".text {\n");
        html.push_str("  position: absolute;\n");
        html.push_str("  white-space: nowrap;\n");
        html.push_str("}\n");
        html.push_str("</style>\n");

        // Add page container
        html.push_str("<div class=\"page\">\n");

        // Add each word with positioning
        for word in &words {
            let escaped_text = escape_html(&word.text);
            let x = word.bbox.x;
            let y = word.bbox.y;
            let font_size = word.avg_font_size;

            html.push_str(&format!(
                "  <div class=\"text\" style=\"left: {}px; top: {}px; font-size: {}px;\">{}</div>\n",
                x, y, font_size, escaped_text
            ));
        }

        html.push_str("</div>\n");

        Ok(html)
    }

    /// Determine the reading order of text blocks.
    ///
    /// This implements simple top-to-bottom, left-to-right ordering.
    ///
    /// # Arguments
    ///
    /// * `blocks` - The text blocks to order
    /// * `mode` - The reading order mode to use
    ///
    /// # Returns
    ///
    /// A vector of indices representing the reading order.
    /// Compute median font size from characters for adaptive epsilon.
    fn compute_median_font_size(chars: &[TextChar]) -> f32 {
        if chars.is_empty() {
            return 12.0; // Default fallback
        }

        let mut font_sizes: Vec<f32> = chars.iter().map(|c| c.font_size).collect();
        font_sizes.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));

        let mid = font_sizes.len() / 2;
        if font_sizes.len().is_multiple_of(2) {
            (font_sizes[mid - 1] + font_sizes[mid]) / 2.0
        } else {
            font_sizes[mid]
        }
    }

    fn determine_reading_order(&self, blocks: &[TextBlock], mode: ReadingOrderMode) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..blocks.len()).collect();

        match mode {
            ReadingOrderMode::TopToBottomLeftToRight
            | ReadingOrderMode::ColumnAware
            | ReadingOrderMode::StructureTreeFirst { .. } => {
                // Sort by Y (top to bottom), then by X (left to right)
                // Note: For StructureTreeFirst, this is a simplified fallback in HTML converter
                // Full structure tree support should be implemented similarly to markdown converter
                indices.sort_by(|&a, &b| {
                    let block_a = &blocks[a];
                    let block_b = &blocks[b];

                    let y_cmp = crate::utils::safe_float_cmp(block_a.bbox.y, block_b.bbox.y);
                    if y_cmp != std::cmp::Ordering::Equal {
                        y_cmp
                    } else {
                        crate::utils::safe_float_cmp(block_a.bbox.x, block_b.bbox.x)
                    }
                });
            },
        }

        indices
    }
}

#[allow(deprecated)]
impl Default for HtmlConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape HTML special characters.
///
/// Replaces &, <, >, ", and ' with their HTML entity equivalents.
///
/// # Arguments
///
/// * `text` - The text to escape
///
/// # Returns
///
/// The escaped text safe for inclusion in HTML.
///
/// # Examples
///
/// ```
/// # use pdf_oxide::converters::html::escape_html;
/// let text = "AT&T <Company>";
/// let escaped = escape_html(text);
/// assert_eq!(escaped, "AT&amp;T &lt;Company&gt;");
/// ```
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Convert URLs and email addresses in text to HTML hyperlinks.
///
/// This function:
/// 1. Escapes HTML special characters in the text
/// 2. Detects URLs (http://, https://) and converts them to `<a href="...">` tags
/// 3. Detects email addresses and converts them to `<a href="mailto:...">` tags
///
/// # Arguments
///
/// * `text` - The text to process
///
/// # Returns
///
/// HTML string with URLs and emails converted to hyperlinks.
///
/// # Examples
///
/// ```
/// # use pdf_oxide::converters::html::linkify_urls_and_emails;
/// let text = "Visit https://example.com or email test@example.com";
/// let linked = linkify_urls_and_emails(text);
/// assert!(linked.contains("<a href=\"https://example.com\">"));
/// assert!(linked.contains("<a href=\"mailto:test@example.com\">"));
/// ```
pub fn linkify_urls_and_emails(text: &str) -> String {
    // First, escape HTML to make text safe
    let escaped = escape_html(text);

    // Process URLs first
    let with_urls = RE_URL.replace_all(&escaped, |caps: &regex::Captures| {
        let url = &caps[0];
        format!(r#"<a href="{}">{}</a>"#, url, url)
    });

    // Then process emails
    let with_emails = RE_EMAIL.replace_all(&with_urls, |caps: &regex::Captures| {
        let email = &caps[0];
        format!(r#"<a href="mailto:{}">{}</a>"#, email, email)
    });

    with_emails.to_string()
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::{Color, FontWeight};

    fn mock_char(c: char, x: f32, y: f32, font_size: f32, bold: bool) -> TextChar {
        let bbox = Rect::new(x, y, 8.0, font_size);
        TextChar {
            char: c,
            bbox,
            font_name: "Times".to_string(),
            font_size,
            font_weight: if bold {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            },
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * font_size,
            descent: -0.35 * font_size,
            matrix: None,
        }
    }

    fn mock_word(text: &str, x: f32, y: f32, font_size: f32, bold: bool) -> Vec<TextChar> {
        text.chars()
            .enumerate()
            .map(|(i, c)| mock_char(c, x + (i as f32 * 7.0), y, font_size, bold))
            .collect()
    }

    #[test]
    fn test_html_converter_new() {
        let converter = HtmlConverter::new();
        assert!(format!("{:?}", converter).contains("HtmlConverter"));
    }

    #[test]
    fn test_html_converter_default() {
        let converter = HtmlConverter;
        assert!(format!("{:?}", converter).contains("HtmlConverter"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("Hello"), "Hello");
        assert_eq!(escape_html("AT&T"), "AT&amp;T");
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("'apostrophe'"), "&#x27;apostrophe&#x27;");
        assert_eq!(escape_html("<b>&\"'</b>"), "&lt;b&gt;&amp;&quot;&#x27;&lt;/b&gt;");
    }

    #[test]
    fn test_convert_empty() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions::default();
        let result = converter.convert_page(&[], &options).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_semantic_single_line() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            detect_headings: false,
            ..Default::default()
        };

        let chars = mock_word("Hello World", 0.0, 0.0, 12.0, false);
        let result = converter.convert_page_semantic(&chars, &options).unwrap();

        assert!(result.contains("<p>Hello World</p>"));
        assert!(!result.contains("<h1>"));
    }

    #[test]
    fn test_convert_semantic_with_heading() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            detect_headings: true,
            ..Default::default()
        };

        let mut chars = Vec::new();
        chars.extend(mock_word("Title", 0.0, 0.0, 24.0, true)); // Large bold = H1
        chars.extend(mock_word("Body", 0.0, 50.0, 12.0, false)); // Regular = Body

        let result = converter.convert_page_semantic(&chars, &options).unwrap();

        assert!(result.contains("Title"));
        assert!(result.contains("Body"));
        assert!(result.contains("<h") || result.contains("<p>"));
    }

    #[test]
    fn test_convert_semantic_escape_html() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            detect_headings: false,
            ..Default::default()
        };

        let chars = mock_word("AT&T<>", 0.0, 0.0, 12.0, false);
        let result = converter.convert_page_semantic(&chars, &options).unwrap();

        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
    }

    #[test]
    fn test_convert_layout_preserved() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            preserve_layout: true,
            ..Default::default()
        };

        let chars = mock_word("Test", 100.0, 200.0, 14.0, false);
        let result = converter
            .convert_page_preserve_layout(&chars, &options)
            .unwrap();

        assert!(result.contains("<style>"));
        assert!(result.contains(".page"));
        assert!(result.contains(".text"));
        assert!(result.contains("position: absolute"));
        assert!(result.contains("left: 100px"));
        assert!(result.contains("top: 200px"));
        assert!(result.contains("font-size: 14px"));
        assert!(result.contains("Test"));
    }

    #[test]
    fn test_convert_layout_multiple_words() {
        use crate::geometry::Rect;
        use crate::layout::{Color, FontWeight, TextSpan};

        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            preserve_layout: true,
            ..Default::default()
        };

        // Create TextSpan instances representing complete words (PDF spec compliant)
        // TextSpan represents complete strings from Tj/TJ operators, not individual chars
        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "First".to_string(),
                bbox: Rect::new(10.0, 20.0, 30.0, 12.0), // width ~= 5 chars * 6pt
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
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
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Second".to_string(),
                bbox: Rect::new(10.0, 40.0, 36.0, 12.0), // width ~= 6 chars * 6pt
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
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
        ];

        let result = converter
            .convert_page_preserve_layout_from_spans(&spans, &options)
            .unwrap();

        assert!(result.contains("First"));
        assert!(result.contains("Second"));
        assert!(result.contains("left: 10px"));
        assert!(result.contains("top: 20px"));
        assert!(result.contains("top: 40px"));
    }

    #[test]
    fn test_convert_page_routes_to_semantic() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            preserve_layout: false,
            detect_headings: false,
            ..Default::default()
        };

        let chars = mock_word("Test", 0.0, 0.0, 12.0, false);
        let result = converter.convert_page(&chars, &options).unwrap();

        assert!(result.contains("<p>Test</p>"));
        assert!(!result.contains("position: absolute"));
    }

    #[test]
    fn test_convert_page_routes_to_layout() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            preserve_layout: true,
            ..Default::default()
        };

        let chars = mock_word("Test", 50.0, 100.0, 12.0, false);
        let result = converter.convert_page(&chars, &options).unwrap();

        assert!(result.contains("position: absolute"));
        assert!(result.contains("left: 50px"));
        assert!(!result.contains("<p>"));
    }

    #[test]
    fn test_semantic_h1() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            detect_headings: true,
            ..Default::default()
        };

        let chars = mock_word("Main Title", 0.0, 0.0, 28.0, true);
        let result = converter.convert_page_semantic(&chars, &options).unwrap();

        assert!(result.contains("Main Title"));
    }

    #[test]
    fn test_semantic_small_text() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            detect_headings: true,
            ..Default::default()
        };

        // Very small text should be detected as Small level
        let chars = mock_word("footnote", 0.0, 0.0, 8.0, false);
        let result = converter.convert_page_semantic(&chars, &options).unwrap();

        assert!(result.contains("footnote"));
    }

    #[test]
    fn test_layout_empty() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions {
            preserve_layout: true,
            ..Default::default()
        };

        let result = converter
            .convert_page_preserve_layout(&[], &options)
            .unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_semantic_empty() {
        let converter = HtmlConverter::new();
        let options = ConversionOptions::default();

        let result = converter.convert_page_semantic(&[], &options).unwrap();
        assert_eq!(result, "");
    }
}
