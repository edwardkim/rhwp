//! Text markup annotations for PDF generation.
//!
//! This module provides support for text markup annotations per PDF spec Section 12.5.6.10:
//! - Highlight (yellow marker effect)
//! - Underline (line under text)
//! - StrikeOut (line through text)
//! - Squiggly (wavy underline)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{TextMarkupAnnotation, TextMarkupType};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a highlight annotation
//! let highlight = TextMarkupAnnotation::highlight(
//!     Rect::new(72.0, 720.0, 100.0, 12.0),
//!     vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]],
//! ).with_color(1.0, 1.0, 0.0); // Yellow
//! ```

use crate::annotation_types::{AnnotationColor, AnnotationFlags, TextMarkupType};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A text markup annotation (Highlight, Underline, StrikeOut, Squiggly).
///
/// Per PDF spec Section 12.5.6.10, text markup annotations highlight text
/// using visual effects like colored backgrounds or lines.
///
/// QuadPoints are required and define the quadrilateral(s) encompassing the text.
/// Each quad is 8 numbers: x1,y1, x2,y2, x3,y3, x4,y4 (counter-clockwise from bottom-left).
#[derive(Debug, Clone)]
pub struct TextMarkupAnnotation {
    /// Bounding rectangle (derived from quad points)
    pub rect: Rect,
    /// Type of text markup (Highlight, Underline, StrikeOut, Squiggly)
    pub markup_type: TextMarkupType,
    /// QuadPoints defining the text area (required)
    /// Each quad is 8 f64 values: [x1,y1, x2,y2, x3,y3, x4,y4]
    pub quad_points: Vec<[f64; 8]>,
    /// Annotation color (affects the markup appearance)
    pub color: Option<AnnotationColor>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
    /// Text contents (description or alt text)
    pub contents: Option<String>,
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
}

impl TextMarkupAnnotation {
    /// Create a new text markup annotation.
    ///
    /// # Arguments
    ///
    /// * `markup_type` - Type of markup (Highlight, Underline, StrikeOut, Squiggly)
    /// * `rect` - Bounding rectangle
    /// * `quad_points` - QuadPoints defining the text area
    pub fn new(markup_type: TextMarkupType, rect: Rect, quad_points: Vec<[f64; 8]>) -> Self {
        Self {
            rect,
            markup_type,
            quad_points,
            color: None,
            opacity: None,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
            creation_date: None,
            modification_date: None,
        }
    }

    /// Create a highlight annotation (yellow marker effect).
    pub fn highlight(rect: Rect, quad_points: Vec<[f64; 8]>) -> Self {
        let mut annot = Self::new(TextMarkupType::Highlight, rect, quad_points);
        // Default yellow color for highlights
        annot.color = Some(AnnotationColor::Rgb(1.0, 1.0, 0.0));
        annot
    }

    /// Create an underline annotation.
    pub fn underline(rect: Rect, quad_points: Vec<[f64; 8]>) -> Self {
        let mut annot = Self::new(TextMarkupType::Underline, rect, quad_points);
        // Default red color for underline
        annot.color = Some(AnnotationColor::Rgb(1.0, 0.0, 0.0));
        annot
    }

    /// Create a strikeout annotation.
    pub fn strikeout(rect: Rect, quad_points: Vec<[f64; 8]>) -> Self {
        let mut annot = Self::new(TextMarkupType::StrikeOut, rect, quad_points);
        // Default red color for strikeout
        annot.color = Some(AnnotationColor::Rgb(1.0, 0.0, 0.0));
        annot
    }

    /// Create a squiggly underline annotation.
    pub fn squiggly(rect: Rect, quad_points: Vec<[f64; 8]>) -> Self {
        let mut annot = Self::new(TextMarkupType::Squiggly, rect, quad_points);
        // Default red color for squiggly
        annot.color = Some(AnnotationColor::Rgb(1.0, 0.0, 0.0));
        annot
    }

    /// Create from a simple rectangle (generates quad points automatically).
    ///
    /// This is a convenience method when you have a single rectangular area.
    pub fn from_rect(markup_type: TextMarkupType, rect: Rect) -> Self {
        let quad = [
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            (rect.y + rect.height) as f64,
            rect.x as f64,
            (rect.y + rect.height) as f64,
        ];
        Self::new(markup_type, rect, vec![quad])
    }

    /// Set the color.
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

    /// Set the contents (description text).
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

    /// Set the annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Build the annotation dictionary for PDF output.
    ///
    /// # Arguments
    ///
    /// * `_page_refs` - Page references (not used for text markup, but kept for API consistency)
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert(
            "Subtype".to_string(),
            Object::Name(self.markup_type.subtype().pdf_name().to_string()),
        );

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

        // QuadPoints (required for text markup)
        let quad_array: Vec<Object> = self
            .quad_points
            .iter()
            .flat_map(|quad| quad.iter().map(|&v| Object::Real(v)))
            .collect();
        dict.insert("QuadPoints".to_string(), Object::Array(quad_array));

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

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
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

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_annotation() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]];

        let highlight = TextMarkupAnnotation::highlight(rect, quads);

        assert!(matches!(highlight.markup_type, TextMarkupType::Highlight));
        assert!(matches!(highlight.color, Some(AnnotationColor::Rgb(1.0, 1.0, 0.0))));
    }

    #[test]
    fn test_underline_annotation() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]];

        let underline = TextMarkupAnnotation::underline(rect, quads);

        assert!(matches!(underline.markup_type, TextMarkupType::Underline));
    }

    #[test]
    fn test_strikeout_annotation() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]];

        let strikeout = TextMarkupAnnotation::strikeout(rect, quads);

        assert!(matches!(strikeout.markup_type, TextMarkupType::StrikeOut));
    }

    #[test]
    fn test_squiggly_annotation() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]];

        let squiggly = TextMarkupAnnotation::squiggly(rect, quads);

        assert!(matches!(squiggly.markup_type, TextMarkupType::Squiggly));
    }

    #[test]
    fn test_from_rect() {
        let rect = Rect::new(100.0, 200.0, 50.0, 20.0);
        let highlight = TextMarkupAnnotation::from_rect(TextMarkupType::Highlight, rect);

        assert_eq!(highlight.quad_points.len(), 1);
        let quad = &highlight.quad_points[0];
        assert_eq!(quad[0], 100.0); // x1
        assert_eq!(quad[1], 200.0); // y1
        assert_eq!(quad[2], 150.0); // x2
    }

    #[test]
    fn test_build_highlight() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]];

        let highlight = TextMarkupAnnotation::highlight(rect, quads)
            .with_opacity(0.5)
            .with_contents("Important text")
            .with_author("Test User");

        let dict = highlight.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Highlight".to_string())));
        assert!(dict.contains_key("QuadPoints"));
        assert!(dict.contains_key("C")); // Color
        assert!(dict.contains_key("CA")); // Opacity
        assert!(dict.contains_key("Contents"));
        assert!(dict.contains_key("T")); // Author
    }

    #[test]
    fn test_quad_points_serialization() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![
            [72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0],
            [72.0, 700.0, 172.0, 700.0, 172.0, 712.0, 72.0, 712.0],
        ];

        let highlight = TextMarkupAnnotation::highlight(rect, quads);
        let dict = highlight.build(&[]);

        if let Some(Object::Array(quad_array)) = dict.get("QuadPoints") {
            assert_eq!(quad_array.len(), 16); // 2 quads × 8 values
        } else {
            panic!("QuadPoints should be an array");
        }
    }

    #[test]
    fn test_fluent_builder() {
        let rect = Rect::new(72.0, 720.0, 100.0, 12.0);
        let quads = vec![[72.0, 720.0, 172.0, 720.0, 172.0, 732.0, 72.0, 732.0]];

        let highlight = TextMarkupAnnotation::highlight(rect, quads)
            .with_color(0.5, 0.8, 0.2)
            .with_opacity(0.7)
            .with_contents("Note")
            .with_author("Reviewer")
            .with_subject("Important");

        assert!(matches!(highlight.color, Some(AnnotationColor::Rgb(0.5, 0.8, 0.2))));
        assert_eq!(highlight.opacity, Some(0.7));
        assert_eq!(highlight.contents, Some("Note".to_string()));
        assert_eq!(highlight.author, Some("Reviewer".to_string()));
        assert_eq!(highlight.subject, Some("Important".to_string()));
    }
}
