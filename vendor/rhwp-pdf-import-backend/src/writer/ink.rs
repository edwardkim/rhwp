//! Ink annotations (freehand drawing) for PDF generation.
//!
//! This module provides support for Ink annotations per PDF spec Section 12.5.6.13.
//! Ink annotations represent freehand "scribbles" composed of one or more disjoint paths.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::InkAnnotation;
//!
//! // Create a freehand drawing with multiple strokes
//! let ink = InkAnnotation::new()
//!     .add_stroke(vec![(100.0, 100.0), (150.0, 120.0), (200.0, 100.0)])
//!     .add_stroke(vec![(100.0, 150.0), (200.0, 150.0)])
//!     .with_stroke_color(1.0, 0.0, 0.0)  // Red
//!     .with_line_width(2.0);
//! ```

use crate::annotation_types::{AnnotationColor, AnnotationFlags, BorderStyleType};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// An Ink annotation (freehand drawing) per PDF spec Section 12.5.6.13.
///
/// Ink annotations represent freehand "scribbles" composed of one or more
/// disjoint paths (strokes). Each stroke is a sequence of points.
#[derive(Debug, Clone)]
pub struct InkAnnotation {
    /// List of strokes, each stroke is a list of (x, y) points
    pub strokes: Vec<Vec<(f64, f64)>>,
    /// Stroke color
    pub color: Option<AnnotationColor>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
    /// Border style
    pub border_style: Option<BorderStyleType>,
    /// Line width
    pub line_width: Option<f32>,
    /// Dash pattern for dashed strokes
    pub dash_pattern: Option<Vec<f32>>,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
}

impl InkAnnotation {
    /// Create a new empty ink annotation.
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
            color: Some(AnnotationColor::black()),
            opacity: None,
            border_style: None,
            line_width: Some(1.0),
            dash_pattern: None,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
            creation_date: None,
            modification_date: None,
        }
    }

    /// Create an ink annotation with a single stroke.
    pub fn with_stroke(stroke: Vec<(f64, f64)>) -> Self {
        let mut ink = Self::new();
        ink.strokes.push(stroke);
        ink
    }

    /// Create an ink annotation from multiple strokes.
    pub fn with_strokes(strokes: Vec<Vec<(f64, f64)>>) -> Self {
        Self {
            strokes,
            ..Self::new()
        }
    }

    /// Add a stroke to the annotation.
    pub fn add_stroke(mut self, stroke: Vec<(f64, f64)>) -> Self {
        self.strokes.push(stroke);
        self
    }

    /// Set the stroke color (RGB).
    pub fn with_stroke_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set the stroke color from AnnotationColor.
    pub fn with_color(mut self, color: AnnotationColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the line width.
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = Some(width);
        self
    }

    /// Set the border style.
    pub fn with_border_style(mut self, style: BorderStyleType) -> Self {
        self.border_style = Some(style);
        self
    }

    /// Set a dashed stroke pattern.
    pub fn with_dash_pattern(mut self, pattern: Vec<f32>) -> Self {
        self.dash_pattern = Some(pattern);
        self.border_style = Some(BorderStyleType::Dashed);
        self
    }

    /// Set the opacity (0.0 = transparent, 1.0 = opaque).
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
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

    /// Set the annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Calculate the bounding rectangle from all strokes.
    pub fn calculate_rect(&self) -> Rect {
        if self.strokes.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for stroke in &self.strokes {
            for (x, y) in stroke {
                min_x = min_x.min(*x);
                max_x = max_x.max(*x);
                min_y = min_y.min(*y);
                max_y = max_y.max(*y);
            }
        }

        // Add margin for line width
        let margin = self.line_width.unwrap_or(1.0) as f64 + 2.0;
        Rect::new(
            (min_x - margin) as f32,
            (min_y - margin) as f32,
            (max_x - min_x + 2.0 * margin) as f32,
            (max_y - min_y + 2.0 * margin) as f32,
        )
    }

    /// Build the annotation dictionary for PDF output.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Ink".to_string()));

        // Rectangle (calculated from strokes)
        let rect = self.calculate_rect();
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(rect.x as f64),
                Object::Real(rect.y as f64),
                Object::Real((rect.x + rect.width) as f64),
                Object::Real((rect.y + rect.height) as f64),
            ]),
        );

        // InkList (required) - array of stroke arrays
        let ink_list: Vec<Object> = self
            .strokes
            .iter()
            .map(|stroke| {
                let points: Vec<Object> = stroke
                    .iter()
                    .flat_map(|(x, y)| vec![Object::Real(*x), Object::Real(*y)])
                    .collect();
                Object::Array(points)
            })
            .collect();
        dict.insert("InkList".to_string(), Object::Array(ink_list));

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Stroke color (C entry)
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

        // Border style (BS entry)
        if self.border_style.is_some() || self.line_width.is_some() || self.dash_pattern.is_some() {
            let mut bs = HashMap::new();
            bs.insert("Type".to_string(), Object::Name("Border".to_string()));

            if let Some(width) = self.line_width {
                bs.insert("W".to_string(), Object::Real(width as f64));
            }

            if let Some(ref style) = self.border_style {
                let style_char = match style {
                    BorderStyleType::Solid => "S",
                    BorderStyleType::Dashed => "D",
                    BorderStyleType::Beveled => "B",
                    BorderStyleType::Inset => "I",
                    BorderStyleType::Underline => "U",
                };
                bs.insert("S".to_string(), Object::Name(style_char.to_string()));
            }

            if let Some(ref pattern) = self.dash_pattern {
                bs.insert(
                    "D".to_string(),
                    Object::Array(pattern.iter().map(|&v| Object::Real(v as f64)).collect()),
                );
            }

            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Author
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

impl Default for InkAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ink_annotation_new() {
        let ink = InkAnnotation::new();
        assert!(ink.strokes.is_empty());
        assert!(ink.color.is_some());
        assert_eq!(ink.line_width, Some(1.0));
    }

    #[test]
    fn test_ink_with_single_stroke() {
        let stroke = vec![(100.0, 100.0), (150.0, 120.0), (200.0, 100.0)];
        let ink = InkAnnotation::with_stroke(stroke.clone());

        assert_eq!(ink.strokes.len(), 1);
        assert_eq!(ink.strokes[0], stroke);
    }

    #[test]
    fn test_ink_with_multiple_strokes() {
        let strokes = vec![
            vec![(100.0, 100.0), (150.0, 120.0)],
            vec![(200.0, 200.0), (250.0, 220.0)],
        ];
        let ink = InkAnnotation::with_strokes(strokes.clone());

        assert_eq!(ink.strokes.len(), 2);
    }

    #[test]
    fn test_ink_add_stroke() {
        let ink = InkAnnotation::new()
            .add_stroke(vec![(100.0, 100.0), (150.0, 120.0)])
            .add_stroke(vec![(200.0, 200.0), (250.0, 220.0)]);

        assert_eq!(ink.strokes.len(), 2);
    }

    #[test]
    fn test_ink_calculate_rect() {
        let ink = InkAnnotation::new()
            .add_stroke(vec![(100.0, 100.0), (200.0, 200.0)])
            .with_line_width(2.0);

        let rect = ink.calculate_rect();

        // Should encompass all points with margin
        assert!(rect.x < 100.0);
        assert!(rect.y < 100.0);
        assert!(rect.x + rect.width > 200.0);
        assert!(rect.y + rect.height > 200.0);
    }

    #[test]
    fn test_ink_build() {
        let ink = InkAnnotation::new()
            .add_stroke(vec![(100.0, 100.0), (150.0, 120.0), (200.0, 100.0)])
            .with_stroke_color(1.0, 0.0, 0.0)
            .with_line_width(2.0);

        let dict = ink.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Ink".to_string())));
        assert!(dict.contains_key("InkList"));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("C")); // Color
        assert!(dict.contains_key("BS")); // Border style
    }

    #[test]
    fn test_ink_with_dashed_stroke() {
        let ink = InkAnnotation::new()
            .add_stroke(vec![(100.0, 100.0), (200.0, 100.0)])
            .with_dash_pattern(vec![3.0, 2.0]);

        assert_eq!(ink.dash_pattern, Some(vec![3.0, 2.0]));
        assert!(matches!(ink.border_style, Some(BorderStyleType::Dashed)));

        let dict = ink.build(&[]);
        assert!(dict.contains_key("BS"));
    }

    #[test]
    fn test_ink_fluent_builder() {
        let ink = InkAnnotation::new()
            .add_stroke(vec![(100.0, 100.0), (200.0, 200.0)])
            .with_stroke_color(0.0, 0.0, 1.0)
            .with_line_width(3.0)
            .with_opacity(0.8)
            .with_author("Artist")
            .with_subject("Drawing")
            .with_contents("A simple line");

        assert_eq!(ink.line_width, Some(3.0));
        assert_eq!(ink.opacity, Some(0.8));
        assert_eq!(ink.author, Some("Artist".to_string()));
        assert_eq!(ink.subject, Some("Drawing".to_string()));
        assert_eq!(ink.contents, Some("A simple line".to_string()));
    }

    #[test]
    fn test_ink_empty_strokes() {
        let ink = InkAnnotation::new();
        let rect = ink.calculate_rect();

        // Empty strokes should produce a zero-size rect
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }
}
