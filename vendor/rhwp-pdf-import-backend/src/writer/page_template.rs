//! Page templates for headers, footers, and page numbers.
//!
//! This module provides reusable page templates with placeholder support
//! for adding consistent headers and footers across documents.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{PageTemplate, Artifact, Placeholder};
//!
//! let template = PageTemplate::new()
//!     .header(Artifact::center("My Document"))
//!     .footer(Artifact::right("{page} of {pages}"));
//! ```

use super::font_manager::FontWeight;

/// Placeholder tokens that can be used in headers and footers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placeholder {
    /// Current page number (1-indexed)
    PageNumber,
    /// Total number of pages
    TotalPages,
    /// Current date (formatted as YYYY-MM-DD)
    Date,
    /// Current time (formatted as HH:MM)
    Time,
    /// Document title (from metadata)
    Title,
    /// Document author (from metadata)
    Author,
}

impl Placeholder {
    /// Get the placeholder token string.
    pub fn token(&self) -> &'static str {
        match self {
            Placeholder::PageNumber => "{page}",
            Placeholder::TotalPages => "{pages}",
            Placeholder::Date => "{date}",
            Placeholder::Time => "{time}",
            Placeholder::Title => "{title}",
            Placeholder::Author => "{author}",
        }
    }

    /// Parse placeholder tokens from a string.
    pub fn parse_all(text: &str) -> Vec<(usize, Placeholder)> {
        let mut placeholders = Vec::new();

        for ph in [
            Placeholder::PageNumber,
            Placeholder::TotalPages,
            Placeholder::Date,
            Placeholder::Time,
            Placeholder::Title,
            Placeholder::Author,
        ] {
            let token = ph.token();
            let mut start = 0;
            while let Some(pos) = text[start..].find(token) {
                placeholders.push((start + pos, ph));
                start += pos + token.len();
            }
        }

        // Sort by position
        placeholders.sort_by_key(|(pos, _)| *pos);
        placeholders
    }
}

/// Text alignment for header/footer content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ArtifactAlignment {
    /// Align to the left margin
    Left,
    /// Center horizontally
    #[default]
    Center,
    /// Align to the right margin
    Right,
}

/// Style configuration for header/footer text.
#[derive(Debug, Clone)]
pub struct ArtifactStyle {
    /// Font name
    pub font_name: String,
    /// Font size in points
    pub font_size: f32,
    /// Font weight
    pub font_weight: FontWeight,
    /// Text color (RGB, 0.0-1.0)
    pub color: (f32, f32, f32),
    /// Whether to draw a separator line
    pub separator_line: bool,
    /// Separator line width
    pub separator_width: f32,
}

impl Default for ArtifactStyle {
    fn default() -> Self {
        Self {
            font_name: "Helvetica".to_string(),
            font_size: 10.0,
            font_weight: FontWeight::Normal,
            color: (0.0, 0.0, 0.0), // Black
            separator_line: false,
            separator_width: 0.5,
        }
    }
}

impl ArtifactStyle {
    /// Create a new default style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the font.
    pub fn font(mut self, name: impl Into<String>, size: f32) -> Self {
        self.font_name = name.into();
        self.font_size = size;
        self
    }

    /// Set bold weight.
    pub fn bold(mut self) -> Self {
        self.font_weight = FontWeight::Bold;
        self
    }

    /// Set text color.
    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = (r, g, b);
        self
    }

    /// Enable separator line.
    pub fn with_separator(mut self, width: f32) -> Self {
        self.separator_line = true;
        self.separator_width = width;
        self
    }
}

/// A single positioned text element in a header or footer.
#[derive(Debug, Clone)]
pub struct ArtifactElement {
    /// The text content (may include placeholders)
    pub text: String,
    /// Horizontal alignment
    pub alignment: ArtifactAlignment,
    /// Style for this element
    pub style: Option<ArtifactStyle>,
}

impl ArtifactElement {
    /// Create a new element with default center alignment.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            alignment: ArtifactAlignment::Center,
            style: None,
        }
    }

    /// Create a left-aligned element.
    pub fn left(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            alignment: ArtifactAlignment::Left,
            style: None,
        }
    }

    /// Create a center-aligned element.
    pub fn center(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            alignment: ArtifactAlignment::Center,
            style: None,
        }
    }

    /// Create a right-aligned element.
    pub fn right(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            alignment: ArtifactAlignment::Right,
            style: None,
        }
    }

    /// Set the style for this element.
    pub fn with_style(mut self, style: ArtifactStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Resolve placeholders in the text.
    pub fn resolve(&self, context: &PlaceholderContext) -> String {
        let mut result = self.text.clone();

        result = result.replace(Placeholder::PageNumber.token(), &context.page_number.to_string());
        result = result.replace(Placeholder::TotalPages.token(), &context.total_pages.to_string());
        result = result.replace(Placeholder::Date.token(), &context.date);
        result = result.replace(Placeholder::Time.token(), &context.time);
        result = result.replace(Placeholder::Title.token(), &context.title);
        result = result.replace(Placeholder::Author.token(), &context.author);

        result
    }
}

/// A header or footer artifact definition.
#[derive(Debug, Clone, Default)]
pub struct Artifact {
    /// Left-aligned element
    pub left: Option<ArtifactElement>,
    /// Center-aligned element
    pub center: Option<ArtifactElement>,
    /// Right-aligned element
    pub right: Option<ArtifactElement>,
    /// Default style for all elements
    pub style: ArtifactStyle,
    /// Vertical offset from page edge (points)
    pub offset: f32,
}

/// Alias for Header
pub type Header = Artifact;
/// Alias for Footer
pub type Footer = Artifact;

impl Artifact {
    /// Create a new empty artifact.
    pub fn new() -> Self {
        Self {
            offset: 36.0, // Half inch from edge
            ..Default::default()
        }
    }

    /// Create with a single left-aligned element.
    pub fn left(text: impl Into<String>) -> Self {
        let mut hf = Self::new();
        hf.left = Some(ArtifactElement::left(text));
        hf
    }

    /// Create with a single centered element.
    pub fn center(text: impl Into<String>) -> Self {
        let mut hf = Self::new();
        hf.center = Some(ArtifactElement::center(text));
        hf
    }

    /// Create with a single right-aligned element.
    pub fn right(text: impl Into<String>) -> Self {
        let mut hf = Self::new();
        hf.right = Some(ArtifactElement::right(text));
        hf
    }

    /// Set the left element.
    pub fn with_left(mut self, text: impl Into<String>) -> Self {
        self.left = Some(ArtifactElement::left(text));
        self
    }

    /// Set the center element.
    pub fn with_center(mut self, text: impl Into<String>) -> Self {
        self.center = Some(ArtifactElement::center(text));
        self
    }

    /// Set the right element.
    pub fn with_right(mut self, text: impl Into<String>) -> Self {
        self.right = Some(ArtifactElement::right(text));
        self
    }

    /// Set the default style.
    pub fn with_style(mut self, style: ArtifactStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the offset from page edge.
    pub fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Check if this header/footer has any content.
    pub fn is_empty(&self) -> bool {
        self.left.is_none() && self.center.is_none() && self.right.is_none()
    }

    /// Get all elements as positioned items.
    pub fn elements(&self) -> Vec<&ArtifactElement> {
        let mut elements = Vec::new();
        if let Some(ref e) = self.left {
            elements.push(e);
        }
        if let Some(ref e) = self.center {
            elements.push(e);
        }
        if let Some(ref e) = self.right {
            elements.push(e);
        }
        elements
    }
}

/// Context for resolving placeholders.
#[derive(Debug, Clone)]
pub struct PlaceholderContext {
    /// Current page number (1-indexed)
    pub page_number: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Current date
    pub date: String,
    /// Current time
    pub time: String,
    /// Document title
    pub title: String,
    /// Document author
    pub author: String,
}

impl PlaceholderContext {
    /// Create a new context with current date/time.
    pub fn new(page_number: usize, total_pages: usize) -> Self {
        let now = chrono::Local::now();
        Self {
            page_number,
            total_pages,
            date: now.format("%Y-%m-%d").to_string(),
            time: now.format("%H:%M").to_string(),
            title: String::new(),
            author: String::new(),
        }
    }

    /// Set the document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the document author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }
}

impl Default for PlaceholderContext {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

/// A complete page template with header and footer.
#[derive(Debug, Clone, Default)]
pub struct PageTemplate {
    /// Header definition
    pub header: Option<Artifact>,
    /// Footer definition
    pub footer: Option<Artifact>,
    /// Whether to skip header/footer on first page
    pub skip_first_page: bool,
    /// Optional different template for first page
    pub first_page_header: Option<Artifact>,
    /// Optional different footer for first page
    pub first_page_footer: Option<Artifact>,
    /// Left margin (points)
    pub margin_left: f32,
    /// Right margin (points)
    pub margin_right: f32,
}

impl PageTemplate {
    /// Create a new empty page template.
    pub fn new() -> Self {
        Self {
            margin_left: 72.0,  // 1 inch
            margin_right: 72.0, // 1 inch
            ..Default::default()
        }
    }

    /// Set the header.
    pub fn header(mut self, header: Artifact) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the footer.
    pub fn footer(mut self, footer: Artifact) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Set to skip header/footer on first page.
    pub fn skip_first_page(mut self) -> Self {
        self.skip_first_page = true;
        self
    }

    /// Set a different header for the first page.
    pub fn first_page_header(mut self, header: Artifact) -> Self {
        self.first_page_header = Some(header);
        self
    }

    /// Set a different footer for the first page.
    pub fn first_page_footer(mut self, footer: Artifact) -> Self {
        self.first_page_footer = Some(footer);
        self
    }

    /// Set margins.
    pub fn margins(mut self, left: f32, right: f32) -> Self {
        self.margin_left = left;
        self.margin_right = right;
        self
    }

    /// Get the header for a specific page.
    pub fn get_header(&self, page_number: usize) -> Option<&Artifact> {
        if page_number == 1 {
            if self.skip_first_page && self.first_page_header.is_none() {
                return None;
            }
            self.first_page_header.as_ref().or(self.header.as_ref())
        } else {
            self.header.as_ref()
        }
    }

    /// Get the footer for a specific page.
    pub fn get_footer(&self, page_number: usize) -> Option<&Artifact> {
        if page_number == 1 {
            if self.skip_first_page && self.first_page_footer.is_none() {
                return None;
            }
            self.first_page_footer.as_ref().or(self.footer.as_ref())
        } else {
            self.footer.as_ref()
        }
    }

    /// Check if the template has any content.
    pub fn is_empty(&self) -> bool {
        self.header.is_none()
            && self.footer.is_none()
            && self.first_page_header.is_none()
            && self.first_page_footer.is_none()
    }
}

/// Common page number format patterns.
pub struct PageNumberFormat;

impl PageNumberFormat {
    /// "Page X" format
    pub fn page_x() -> String {
        "Page {page}".to_string()
    }

    /// "Page X of Y" format
    pub fn page_x_of_y() -> String {
        "Page {page} of {pages}".to_string()
    }

    /// "X / Y" format
    pub fn x_slash_y() -> String {
        "{page} / {pages}".to_string()
    }

    /// "X" format (just the number)
    pub fn number_only() -> String {
        "{page}".to_string()
    }

    /// "- X -" format
    pub fn dashed() -> String {
        "- {page} -".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_tokens() {
        assert_eq!(Placeholder::PageNumber.token(), "{page}");
        assert_eq!(Placeholder::TotalPages.token(), "{pages}");
        assert_eq!(Placeholder::Date.token(), "{date}");
    }

    #[test]
    fn test_placeholder_parse() {
        let text = "Page {page} of {pages}";
        let placeholders = Placeholder::parse_all(text);

        assert_eq!(placeholders.len(), 2);
        assert_eq!(placeholders[0].1, Placeholder::PageNumber);
        assert_eq!(placeholders[1].1, Placeholder::TotalPages);
    }

    #[test]
    fn test_hf_element_resolve() {
        let element = ArtifactElement::center("Page {page} of {pages}");
        let context = PlaceholderContext::new(5, 10);

        let resolved = element.resolve(&context);
        assert_eq!(resolved, "Page 5 of 10");
    }

    #[test]
    fn test_artifact_creation() {
        let hf = Artifact::new()
            .with_left("Document Title")
            .with_right("{page}");

        assert!(hf.left.is_some());
        assert!(hf.center.is_none());
        assert!(hf.right.is_some());
    }

    #[test]
    fn test_page_template() {
        let template = PageTemplate::new()
            .header(Artifact::center("My Document"))
            .footer(Artifact::right("{page} of {pages}"));

        assert!(template.header.is_some());
        assert!(template.footer.is_some());
        assert!(!template.is_empty());
    }

    #[test]
    fn test_skip_first_page() {
        let template = PageTemplate::new()
            .header(Artifact::center("Header"))
            .skip_first_page();

        assert!(template.get_header(1).is_none());
        assert!(template.get_header(2).is_some());
    }

    #[test]
    fn test_first_page_template() {
        let template = PageTemplate::new()
            .header(Artifact::center("Regular Header"))
            .first_page_header(Artifact::center("Title Page"));

        let first = template.get_header(1).unwrap();
        let second = template.get_header(2).unwrap();

        assert_eq!(first.center.as_ref().unwrap().text, "Title Page");
        assert_eq!(second.center.as_ref().unwrap().text, "Regular Header");
    }

    #[test]
    fn test_page_number_formats() {
        assert_eq!(PageNumberFormat::page_x(), "Page {page}");
        assert_eq!(PageNumberFormat::page_x_of_y(), "Page {page} of {pages}");
        assert_eq!(PageNumberFormat::x_slash_y(), "{page} / {pages}");
    }

    #[test]
    fn test_hf_style() {
        let style = ArtifactStyle::new()
            .font("Times-Roman", 12.0)
            .bold()
            .color(0.5, 0.5, 0.5)
            .with_separator(1.0);

        assert_eq!(style.font_name, "Times-Roman");
        assert_eq!(style.font_size, 12.0);
        assert!(matches!(style.font_weight, FontWeight::Bold));
        assert!(style.separator_line);
    }

    #[test]
    fn test_placeholder_context_with_metadata() {
        let context = PlaceholderContext::new(1, 10)
            .with_title("My Document")
            .with_author("John Doe");

        let element = ArtifactElement::center("{title} by {author}");
        let resolved = element.resolve(&context);

        assert_eq!(resolved, "My Document by John Doe");
    }
}
