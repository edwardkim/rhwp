//! Text annotations (sticky notes) for PDF generation.
//!
//! This module provides support for text annotations per PDF spec Section 12.5.6.4.
//! Text annotations display an icon that, when opened, shows a pop-up window
//! containing text in a standard font.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::TextAnnotation;
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a sticky note
//! let note = TextAnnotation::new(
//!     Rect::new(72.0, 720.0, 24.0, 24.0),
//!     "This is a comment",
//! ).with_icon(TextAnnotationIcon::Comment)
//!  .with_color(1.0, 1.0, 0.0); // Yellow
//! ```

use crate::annotation_types::{AnnotationColor, AnnotationFlags, TextAnnotationIcon};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A text annotation (sticky note).
///
/// Per PDF spec Section 12.5.6.4, a text annotation represents a sticky note
/// attached to a point in the PDF document. When closed, the annotation
/// displays an icon; when open, it displays a pop-up window.
#[derive(Debug, Clone)]
pub struct TextAnnotation {
    /// Bounding rectangle for the icon
    pub rect: Rect,
    /// Text contents of the note
    pub contents: String,
    /// Icon to display when closed
    pub icon: TextAnnotationIcon,
    /// Whether the annotation should be initially open
    pub open: bool,
    /// Annotation color (affects icon appearance)
    pub color: Option<AnnotationColor>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
    /// Author/creator of the annotation
    pub author: Option<String>,
    /// Subject of the annotation
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Creation date (PDF date format)
    pub creation_date: Option<String>,
    /// Modification date (PDF date format)
    pub modification_date: Option<String>,
    /// State of the annotation (for review workflows)
    pub state: Option<String>,
    /// State model (Marked, Review)
    pub state_model: Option<String>,
}

impl TextAnnotation {
    /// Create a new text annotation (sticky note).
    ///
    /// # Arguments
    ///
    /// * `rect` - Bounding rectangle for the icon (typically small, e.g., 24x24)
    /// * `contents` - The text content of the note
    pub fn new(rect: Rect, contents: impl Into<String>) -> Self {
        Self {
            rect,
            contents: contents.into(),
            icon: TextAnnotationIcon::Note,
            open: false,
            color: Some(AnnotationColor::yellow()), // Default yellow
            opacity: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
            creation_date: None,
            modification_date: None,
            state: None,
            state_model: None,
        }
    }

    /// Create a comment annotation (Comment icon).
    pub fn comment(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::Comment)
    }

    /// Create a note annotation (Note icon).
    pub fn note(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::Note)
    }

    /// Create a help annotation (Help icon).
    pub fn help(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::Help)
    }

    /// Create a key annotation (Key icon).
    pub fn key(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::Key)
    }

    /// Create an insert annotation (Insert icon, for insertion markers).
    pub fn insert(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::Insert)
    }

    /// Create a paragraph annotation (Paragraph icon).
    pub fn paragraph(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::Paragraph)
    }

    /// Create a new paragraph annotation (NewParagraph icon).
    pub fn new_paragraph(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_icon(TextAnnotationIcon::NewParagraph)
    }

    /// Set the icon.
    pub fn with_icon(mut self, icon: TextAnnotationIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Set whether the annotation should be initially open.
    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Set the color (RGB).
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set the color from an AnnotationColor.
    pub fn with_annotation_color(mut self, color: AnnotationColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the opacity (0.0 = transparent, 1.0 = opaque).
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
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

    /// Set the annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set the state (for review workflows).
    ///
    /// Common states: "Accepted", "Rejected", "Cancelled", "Completed", "None"
    pub fn with_state(mut self, state: impl Into<String>, model: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self.state_model = Some(model.into());
        self
    }

    /// Build the annotation dictionary for PDF output.
    ///
    /// # Arguments
    ///
    /// * `_page_refs` - Page references (not used for text annotations, but kept for API consistency)
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Text".to_string()));

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

        // Contents
        dict.insert("Contents".to_string(), Object::text_string(&self.contents));

        // Icon name
        dict.insert("Name".to_string(), Object::Name(self.icon.pdf_name().to_string()));

        // Open state
        dict.insert("Open".to_string(), Object::Boolean(self.open));

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
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

        // Opacity
        if let Some(opacity) = self.opacity {
            dict.insert("CA".to_string(), Object::Real(opacity as f64));
        }

        // Author (T entry)
        if let Some(ref author) = self.author {
            dict.insert("T".to_string(), Object::text_string(author));
        }

        // Subject
        if let Some(ref subject) = self.subject {
            dict.insert("Subj".to_string(), Object::text_string(subject));
        }

        // Creation date
        if let Some(ref date) = self.creation_date {
            dict.insert("CreationDate".to_string(), Object::text_string(date));
        }

        // Modification date
        if let Some(ref date) = self.modification_date {
            dict.insert("M".to_string(), Object::text_string(date));
        }

        // State (for review workflows)
        if let Some(ref state) = self.state {
            dict.insert("State".to_string(), Object::text_string(state));
        }
        if let Some(ref model) = self.state_model {
            dict.insert("StateModel".to_string(), Object::text_string(model));
        }

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_annotation_new() {
        let rect = Rect::new(72.0, 720.0, 24.0, 24.0);
        let note = TextAnnotation::new(rect, "Test note");

        assert_eq!(note.contents, "Test note");
        assert!(matches!(note.icon, TextAnnotationIcon::Note));
        assert!(!note.open);
        assert!(note.color.is_some());
    }

    #[test]
    fn test_text_annotation_comment() {
        let rect = Rect::new(72.0, 720.0, 24.0, 24.0);
        let comment = TextAnnotation::comment(rect, "This is a comment");

        assert!(matches!(comment.icon, TextAnnotationIcon::Comment));
    }

    #[test]
    fn test_text_annotation_icons() {
        let rect = Rect::new(72.0, 720.0, 24.0, 24.0);

        assert!(matches!(TextAnnotation::note(rect, "").icon, TextAnnotationIcon::Note));
        assert!(matches!(TextAnnotation::help(rect, "").icon, TextAnnotationIcon::Help));
        assert!(matches!(TextAnnotation::key(rect, "").icon, TextAnnotationIcon::Key));
        assert!(matches!(TextAnnotation::insert(rect, "").icon, TextAnnotationIcon::Insert));
        assert!(matches!(
            TextAnnotation::paragraph(rect, "").icon,
            TextAnnotationIcon::Paragraph
        ));
        assert!(matches!(
            TextAnnotation::new_paragraph(rect, "").icon,
            TextAnnotationIcon::NewParagraph
        ));
    }

    #[test]
    fn test_text_annotation_build() {
        let rect = Rect::new(72.0, 720.0, 24.0, 24.0);
        let note = TextAnnotation::new(rect, "Important note")
            .with_icon(TextAnnotationIcon::Comment)
            .with_open(true)
            .with_color(1.0, 0.8, 0.0) // Orange
            .with_author("Reviewer")
            .with_subject("Review");

        let dict = note.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Text".to_string())));
        assert_eq!(dict.get("Name"), Some(&Object::Name("Comment".to_string())));
        assert_eq!(dict.get("Open"), Some(&Object::Boolean(true)));
        assert!(dict.contains_key("Contents"));
        assert!(dict.contains_key("C")); // Color
        assert!(dict.contains_key("T")); // Author
        assert!(dict.contains_key("Subj")); // Subject
    }

    #[test]
    fn test_text_annotation_fluent_builder() {
        let rect = Rect::new(100.0, 500.0, 20.0, 20.0);
        let note = TextAnnotation::new(rect, "Test")
            .with_icon(TextAnnotationIcon::Help)
            .with_open(true)
            .with_opacity(0.8)
            .with_author("Author")
            .with_subject("Subject")
            .with_state("Accepted", "Review");

        assert!(matches!(note.icon, TextAnnotationIcon::Help));
        assert!(note.open);
        assert_eq!(note.opacity, Some(0.8));
        assert_eq!(note.author, Some("Author".to_string()));
        assert_eq!(note.subject, Some("Subject".to_string()));
        assert_eq!(note.state, Some("Accepted".to_string()));
        assert_eq!(note.state_model, Some("Review".to_string()));
    }
}
