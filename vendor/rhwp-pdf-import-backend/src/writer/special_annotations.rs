//! Special annotations for PDF generation.
//!
//! This module provides support for special PDF annotation types:
//! - Popup (Section 12.5.6.14): Pop-up windows for other annotations
//! - Caret (Section 12.5.6.11): Insertion point markers
//! - FileAttachment (Section 12.5.6.15): Embedded file attachments
//! - Redact (Section 12.5.6.23): Redaction annotations
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{CaretAnnotation, RedactAnnotation};
//!
//! // Create a caret for text insertion
//! let caret = CaretAnnotation::new(Rect::new(100.0, 700.0, 10.0, 10.0))
//!     .with_symbol(CaretSymbol::Paragraph);
//!
//! // Create a redact annotation
//! let redact = RedactAnnotation::new(Rect::new(72.0, 500.0, 200.0, 20.0))
//!     .with_overlay_text("REDACTED");
//! ```

use crate::annotation_types::{AnnotationColor, AnnotationFlags};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

// ============================================================================
// Popup Annotation (Section 12.5.6.14)
// ============================================================================

/// A Popup annotation per PDF spec Section 12.5.6.14.
///
/// Popup annotations do not appear alone; they are associated with a markup
/// annotation and are used to display text in a pop-up window.
#[derive(Debug, Clone)]
pub struct PopupAnnotation {
    /// Bounding rectangle for the popup window
    pub rect: Rect,
    /// Whether the popup is initially open
    pub open: bool,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

impl PopupAnnotation {
    /// Create a new popup annotation.
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            open: false,
            flags: AnnotationFlags::empty(),
        }
    }

    /// Create a popup that is initially open.
    pub fn open(rect: Rect) -> Self {
        Self {
            rect,
            open: true,
            flags: AnnotationFlags::empty(),
        }
    }

    /// Set whether the popup is initially open.
    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Build the annotation dictionary.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Popup".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // Open flag
        dict.insert("Open".to_string(), Object::Boolean(self.open));

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        dict
    }

    /// Get the bounding rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

impl Default for PopupAnnotation {
    fn default() -> Self {
        Self::new(Rect::new(0.0, 0.0, 200.0, 150.0))
    }
}

// ============================================================================
// Caret Annotation (Section 12.5.6.11)
// ============================================================================

/// Caret symbol type for caret annotations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CaretSymbol {
    /// No symbol (default)
    #[default]
    None,
    /// Paragraph symbol (¶)
    Paragraph,
}

impl CaretSymbol {
    /// Get the PDF name for this symbol.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            CaretSymbol::None => "None",
            CaretSymbol::Paragraph => "P",
        }
    }
}

/// A Caret annotation per PDF spec Section 12.5.6.11.
///
/// Caret annotations mark a position in the document text where content
/// should be inserted or where a correction is needed.
#[derive(Debug, Clone)]
pub struct CaretAnnotation {
    /// Bounding rectangle for the caret
    pub rect: Rect,
    /// Symbol to display (None or Paragraph)
    pub symbol: CaretSymbol,
    /// Optional rectangle difference for the caret
    pub rd: Option<(f64, f64, f64, f64)>,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Color
    pub color: Option<AnnotationColor>,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

impl CaretAnnotation {
    /// Create a new caret annotation.
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            symbol: CaretSymbol::None,
            rd: None,
            contents: None,
            author: None,
            subject: None,
            color: Some(AnnotationColor::Rgb(0.0, 0.0, 1.0)), // Blue by default
            flags: AnnotationFlags::printable(),
        }
    }

    /// Create a caret with paragraph symbol.
    pub fn paragraph(rect: Rect) -> Self {
        Self {
            symbol: CaretSymbol::Paragraph,
            ..Self::new(rect)
        }
    }

    /// Set the caret symbol.
    pub fn with_symbol(mut self, symbol: CaretSymbol) -> Self {
        self.symbol = symbol;
        self
    }

    /// Set the rectangle difference (RD).
    pub fn with_rd(mut self, left: f64, bottom: f64, right: f64, top: f64) -> Self {
        self.rd = Some((left, bottom, right, top));
        self
    }

    /// Set contents/comment.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the subject.
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the color.
    pub fn with_color(mut self, color: AnnotationColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Build the annotation dictionary.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Caret".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // Symbol (Sy)
        dict.insert("Sy".to_string(), Object::Name(self.symbol.pdf_name().to_string()));

        // Rectangle Difference (RD)
        if let Some((l, b, r, t)) = self.rd {
            dict.insert(
                "RD".to_string(),
                Object::Array(vec![
                    Object::Real(l),
                    Object::Real(b),
                    Object::Real(r),
                    Object::Real(t),
                ]),
            );
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Color
        if let Some(ref color) = self.color {
            if let Some(color_array) = color.to_array() {
                if !color_array.is_empty() {
                    dict.insert(
                        "C".to_string(),
                        Object::Array(
                            color_array
                                .into_iter()
                                .map(|v| Object::Real(v as f64))
                                .collect(),
                        ),
                    );
                }
            }
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Author
        if let Some(ref author) = self.author {
            dict.insert("T".to_string(), Object::text_string(author));
        }

        // Subject
        if let Some(ref subject) = self.subject {
            dict.insert("Subj".to_string(), Object::text_string(subject));
        }

        dict
    }

    /// Get the bounding rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

impl Default for CaretAnnotation {
    fn default() -> Self {
        Self::new(Rect::new(0.0, 0.0, 10.0, 10.0))
    }
}

// ============================================================================
// FileAttachment Annotation (Section 12.5.6.15)
// ============================================================================

/// Icon type for file attachment annotations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FileAttachmentIcon {
    /// Push pin icon (default)
    #[default]
    PushPin,
    /// Graph/chart icon
    Graph,
    /// Paperclip icon
    Paperclip,
    /// Tag icon
    Tag,
}

impl FileAttachmentIcon {
    /// Get the PDF name for this icon.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            FileAttachmentIcon::PushPin => "PushPin",
            FileAttachmentIcon::Graph => "Graph",
            FileAttachmentIcon::Paperclip => "Paperclip",
            FileAttachmentIcon::Tag => "Tag",
        }
    }
}

/// A FileAttachment annotation per PDF spec Section 12.5.6.15.
///
/// File attachment annotations contain a reference to an embedded file.
#[derive(Debug, Clone)]
pub struct FileAttachmentAnnotation {
    /// Bounding rectangle for the icon
    pub rect: Rect,
    /// Icon to display
    pub icon: FileAttachmentIcon,
    /// File name
    pub file_name: String,
    /// Description of the file
    pub description: Option<String>,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Color
    pub color: Option<AnnotationColor>,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

impl FileAttachmentAnnotation {
    /// Create a new file attachment annotation.
    pub fn new(rect: Rect, file_name: impl Into<String>) -> Self {
        Self {
            rect,
            icon: FileAttachmentIcon::PushPin,
            file_name: file_name.into(),
            description: None,
            contents: None,
            author: None,
            color: None,
            flags: AnnotationFlags::printable(),
        }
    }

    /// Set the icon.
    pub fn with_icon(mut self, icon: FileAttachmentIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Set the file description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set contents/comment.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the color.
    pub fn with_color(mut self, color: AnnotationColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Build the annotation dictionary.
    ///
    /// Note: This creates a basic file specification. The actual file embedding
    /// requires additional structure in the PDF that should be handled by PdfWriter.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("FileAttachment".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // Icon name (Name)
        dict.insert("Name".to_string(), Object::Name(self.icon.pdf_name().to_string()));

        // File Specification (FS) - simplified version
        let mut fs_dict = HashMap::new();
        fs_dict.insert("Type".to_string(), Object::Name("Filespec".to_string()));
        fs_dict.insert("F".to_string(), Object::text_string(&self.file_name));
        if let Some(ref desc) = self.description {
            fs_dict.insert("Desc".to_string(), Object::text_string(desc));
        }
        dict.insert("FS".to_string(), Object::Dictionary(fs_dict));

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Color
        if let Some(ref color) = self.color {
            if let Some(color_array) = color.to_array() {
                if !color_array.is_empty() {
                    dict.insert(
                        "C".to_string(),
                        Object::Array(
                            color_array
                                .into_iter()
                                .map(|v| Object::Real(v as f64))
                                .collect(),
                        ),
                    );
                }
            }
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Author
        if let Some(ref author) = self.author {
            dict.insert("T".to_string(), Object::text_string(author));
        }

        dict
    }

    /// Get the bounding rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

// ============================================================================
// Redact Annotation (Section 12.5.6.23)
// ============================================================================

/// A Redact annotation per PDF spec Section 12.5.6.23.
///
/// Redact annotations mark content that is intended to be permanently removed.
/// When applied, the marked content is deleted and optionally replaced with
/// overlay text or a colored box.
#[derive(Debug, Clone)]
pub struct RedactAnnotation {
    /// Bounding rectangle
    pub rect: Rect,
    /// QuadPoints defining precise redaction areas
    pub quad_points: Option<Vec<[f64; 8]>>,
    /// Interior color (fill color after redaction)
    pub interior_color: Option<AnnotationColor>,
    /// Overlay text to display after redaction
    pub overlay_text: Option<String>,
    /// Alignment of overlay text (0=left, 1=center, 2=right)
    pub overlay_text_alignment: i32,
    /// Default appearance for overlay text
    pub default_appearance: Option<String>,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

impl RedactAnnotation {
    /// Create a new redact annotation.
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            quad_points: None,
            interior_color: Some(AnnotationColor::black()),
            overlay_text: None,
            overlay_text_alignment: 0,
            default_appearance: None,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::new(AnnotationFlags::PRINT | AnnotationFlags::LOCKED),
        }
    }

    /// Create a redact annotation with quad points.
    pub fn with_quads(rect: Rect, quad_points: Vec<[f64; 8]>) -> Self {
        Self {
            quad_points: Some(quad_points),
            ..Self::new(rect)
        }
    }

    /// Set the interior/fill color.
    pub fn with_interior_color(mut self, color: AnnotationColor) -> Self {
        self.interior_color = Some(color);
        self
    }

    /// Set the overlay text.
    pub fn with_overlay_text(mut self, text: impl Into<String>) -> Self {
        self.overlay_text = Some(text.into());
        self
    }

    /// Set the overlay text alignment (0=left, 1=center, 2=right).
    pub fn with_overlay_text_alignment(mut self, alignment: i32) -> Self {
        self.overlay_text_alignment = alignment.clamp(0, 2);
        self
    }

    /// Set the default appearance string.
    pub fn with_default_appearance(mut self, da: impl Into<String>) -> Self {
        self.default_appearance = Some(da.into());
        self
    }

    /// Set contents/comment.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the subject.
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Build the annotation dictionary.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Redact".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // QuadPoints
        if let Some(ref quads) = self.quad_points {
            let mut all_points = Vec::new();
            for quad in quads {
                for point in quad {
                    all_points.push(Object::Real(*point));
                }
            }
            dict.insert("QuadPoints".to_string(), Object::Array(all_points));
        }

        // Interior color (IC)
        if let Some(ref color) = self.interior_color {
            if let Some(color_array) = color.to_array() {
                dict.insert(
                    "IC".to_string(),
                    Object::Array(
                        color_array
                            .into_iter()
                            .map(|v| Object::Real(v as f64))
                            .collect(),
                    ),
                );
            }
        }

        // Overlay text
        if let Some(ref text) = self.overlay_text {
            dict.insert("OverlayText".to_string(), Object::text_string(text));
        }

        // Overlay text alignment (Q)
        if self.overlay_text_alignment != 0 {
            dict.insert("Q".to_string(), Object::Integer(self.overlay_text_alignment as i64));
        }

        // Default appearance
        if let Some(ref da) = self.default_appearance {
            dict.insert("DA".to_string(), Object::text_string(da));
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Author
        if let Some(ref author) = self.author {
            dict.insert("T".to_string(), Object::text_string(author));
        }

        // Subject
        if let Some(ref subject) = self.subject {
            dict.insert("Subj".to_string(), Object::text_string(subject));
        }

        dict
    }

    /// Get the bounding rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

impl Default for RedactAnnotation {
    fn default() -> Self {
        Self::new(Rect::new(0.0, 0.0, 100.0, 20.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Popup tests
    #[test]
    fn test_popup_new() {
        let rect = Rect::new(300.0, 500.0, 200.0, 150.0);
        let popup = PopupAnnotation::new(rect);

        assert_eq!(popup.rect, rect);
        assert!(!popup.open);
    }

    #[test]
    fn test_popup_open() {
        let rect = Rect::new(300.0, 500.0, 200.0, 150.0);
        let popup = PopupAnnotation::open(rect);

        assert!(popup.open);
    }

    #[test]
    fn test_popup_build() {
        let popup = PopupAnnotation::new(Rect::new(100.0, 100.0, 200.0, 150.0));

        let dict = popup.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Popup".to_string())));
        assert_eq!(dict.get("Open"), Some(&Object::Boolean(false)));
    }

    // Caret tests
    #[test]
    fn test_caret_new() {
        let rect = Rect::new(100.0, 700.0, 10.0, 10.0);
        let caret = CaretAnnotation::new(rect);

        assert_eq!(caret.rect, rect);
        assert_eq!(caret.symbol, CaretSymbol::None);
    }

    #[test]
    fn test_caret_paragraph() {
        let rect = Rect::new(100.0, 700.0, 10.0, 10.0);
        let caret = CaretAnnotation::paragraph(rect);

        assert_eq!(caret.symbol, CaretSymbol::Paragraph);
    }

    #[test]
    fn test_caret_symbol_names() {
        assert_eq!(CaretSymbol::None.pdf_name(), "None");
        assert_eq!(CaretSymbol::Paragraph.pdf_name(), "P");
    }

    #[test]
    fn test_caret_build() {
        let caret =
            CaretAnnotation::new(Rect::new(100.0, 700.0, 10.0, 10.0)).with_contents("Insert here");

        let dict = caret.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Caret".to_string())));
        assert_eq!(dict.get("Sy"), Some(&Object::Name("None".to_string())));
        assert!(dict.contains_key("Contents"));
    }

    #[test]
    fn test_caret_fluent_builder() {
        let caret = CaretAnnotation::new(Rect::new(100.0, 700.0, 10.0, 10.0))
            .with_symbol(CaretSymbol::Paragraph)
            .with_contents("Insert paragraph")
            .with_author("Editor");

        assert_eq!(caret.symbol, CaretSymbol::Paragraph);
        assert_eq!(caret.contents, Some("Insert paragraph".to_string()));
        assert_eq!(caret.author, Some("Editor".to_string()));
    }

    // FileAttachment tests
    #[test]
    fn test_file_attachment_new() {
        let rect = Rect::new(72.0, 600.0, 24.0, 24.0);
        let file = FileAttachmentAnnotation::new(rect, "document.pdf");

        assert_eq!(file.rect, rect);
        assert_eq!(file.file_name, "document.pdf");
        assert_eq!(file.icon, FileAttachmentIcon::PushPin);
    }

    #[test]
    fn test_file_attachment_icons() {
        assert_eq!(FileAttachmentIcon::PushPin.pdf_name(), "PushPin");
        assert_eq!(FileAttachmentIcon::Graph.pdf_name(), "Graph");
        assert_eq!(FileAttachmentIcon::Paperclip.pdf_name(), "Paperclip");
        assert_eq!(FileAttachmentIcon::Tag.pdf_name(), "Tag");
    }

    #[test]
    fn test_file_attachment_build() {
        let file = FileAttachmentAnnotation::new(Rect::new(72.0, 600.0, 24.0, 24.0), "report.xlsx")
            .with_icon(FileAttachmentIcon::Paperclip)
            .with_description("Monthly report");

        let dict = file.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("FileAttachment".to_string())));
        assert_eq!(dict.get("Name"), Some(&Object::Name("Paperclip".to_string())));
        assert!(dict.contains_key("FS"));
    }

    // Redact tests
    #[test]
    fn test_redact_new() {
        let rect = Rect::new(72.0, 500.0, 200.0, 20.0);
        let redact = RedactAnnotation::new(rect);

        assert_eq!(redact.rect, rect);
        assert!(redact.interior_color.is_some());
    }

    #[test]
    fn test_redact_with_overlay() {
        let redact = RedactAnnotation::new(Rect::new(72.0, 500.0, 200.0, 20.0))
            .with_overlay_text("REDACTED")
            .with_overlay_text_alignment(1); // Center

        assert_eq!(redact.overlay_text, Some("REDACTED".to_string()));
        assert_eq!(redact.overlay_text_alignment, 1);
    }

    #[test]
    fn test_redact_build() {
        let redact = RedactAnnotation::new(Rect::new(72.0, 500.0, 200.0, 20.0))
            .with_overlay_text("CONFIDENTIAL")
            .with_interior_color(AnnotationColor::black());

        let dict = redact.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Redact".to_string())));
        assert!(dict.contains_key("IC"));
        assert!(dict.contains_key("OverlayText"));
    }

    #[test]
    fn test_redact_with_quads() {
        let rect = Rect::new(72.0, 500.0, 200.0, 20.0);
        let quads = vec![[72.0, 500.0, 272.0, 500.0, 272.0, 520.0, 72.0, 520.0]];
        let redact = RedactAnnotation::with_quads(rect, quads);

        assert!(redact.quad_points.is_some());

        let dict = redact.build(&[]);
        assert!(dict.contains_key("QuadPoints"));
    }
}
