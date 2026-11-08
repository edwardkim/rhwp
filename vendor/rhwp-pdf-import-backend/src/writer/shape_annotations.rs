//! Shape annotations (Line, Square, Circle, Polygon, PolyLine) for PDF generation.
//!
//! This module provides support for shape annotations per PDF spec:
//! - Line: Section 12.5.6.7
//! - Square/Circle: Section 12.5.6.8
//! - Polygon/PolyLine: Section 12.5.6.9
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{LineAnnotation, ShapeAnnotation, PolygonAnnotation};
//! use pdf_oxide::geometry::Rect;
//! use pdf_oxide::annotation_types::LineEndingStyle;
//!
//! // Draw a line with arrow
//! let line = LineAnnotation::new((100.0, 100.0), (200.0, 200.0))
//!     .with_line_endings(LineEndingStyle::OpenArrow, LineEndingStyle::None);
//!
//! // Draw a rectangle
//! let rect = ShapeAnnotation::square(Rect::new(72.0, 600.0, 100.0, 80.0))
//!     .with_stroke_color(0.0, 0.0, 1.0)  // Blue border
//!     .with_fill_color(0.9, 0.9, 1.0);   // Light blue fill
//!
//! // Draw a polygon
//! let polygon = PolygonAnnotation::polygon(vec![(100.0, 100.0), (150.0, 150.0), (100.0, 150.0)]);
//! ```

use crate::annotation_types::{
    AnnotationColor, AnnotationFlags, BorderEffect, BorderStyleType, LineEndingStyle,
};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

// ============================================================================
// Line Annotation
// ============================================================================

/// A Line annotation per PDF spec Section 12.5.6.7.
///
/// Displays a single straight line on the page.
#[derive(Debug, Clone)]
pub struct LineAnnotation {
    /// Start point (x, y)
    pub start: (f64, f64),
    /// End point (x, y)
    pub end: (f64, f64),
    /// Line ending styles (start, end)
    pub line_endings: (LineEndingStyle, LineEndingStyle),
    /// Stroke color
    pub color: Option<AnnotationColor>,
    /// Interior color (for filled line endings)
    pub interior_color: Option<AnnotationColor>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
    /// Border style type
    pub border_style: Option<BorderStyleType>,
    /// Line width
    pub line_width: Option<f32>,
    /// Leader line length (positive = extends from endpoints)
    pub leader_line: Option<f64>,
    /// Leader line offset (distance from endpoints)
    pub leader_line_offset: Option<f64>,
    /// Leader line extension (beyond leader line)
    pub leader_line_extension: Option<f64>,
    /// Whether to show caption
    pub caption: bool,
    /// Caption content
    pub contents: Option<String>,
    /// Caption positioning (Inline or Top)
    pub caption_position: CaptionPosition,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

/// Caption position for line annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptionPosition {
    /// Caption centered inside the line
    #[default]
    Inline,
    /// Caption above the line
    Top,
}

impl CaptionPosition {
    /// Get PDF name.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            Self::Inline => "Inline",
            Self::Top => "Top",
        }
    }
}

impl LineAnnotation {
    /// Create a new line annotation.
    pub fn new(start: (f64, f64), end: (f64, f64)) -> Self {
        Self {
            start,
            end,
            line_endings: (LineEndingStyle::None, LineEndingStyle::None),
            color: Some(AnnotationColor::black()),
            interior_color: None,
            opacity: None,
            border_style: None,
            line_width: Some(1.0),
            leader_line: None,
            leader_line_offset: None,
            leader_line_extension: None,
            caption: false,
            contents: None,
            caption_position: CaptionPosition::Inline,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
        }
    }

    /// Create a line with arrow at the end.
    pub fn arrow(start: (f64, f64), end: (f64, f64)) -> Self {
        Self::new(start, end).with_line_endings(LineEndingStyle::None, LineEndingStyle::OpenArrow)
    }

    /// Create a double-headed arrow line.
    pub fn double_arrow(start: (f64, f64), end: (f64, f64)) -> Self {
        Self::new(start, end)
            .with_line_endings(LineEndingStyle::OpenArrow, LineEndingStyle::OpenArrow)
    }

    /// Create a dimension line (with leader lines).
    pub fn dimension(start: (f64, f64), end: (f64, f64), leader_length: f64) -> Self {
        Self::new(start, end)
            .with_line_endings(LineEndingStyle::OpenArrow, LineEndingStyle::OpenArrow)
            .with_leader_line(leader_length)
    }

    /// Set line endings.
    pub fn with_line_endings(mut self, start: LineEndingStyle, end: LineEndingStyle) -> Self {
        self.line_endings = (start, end);
        self
    }

    /// Set stroke color (RGB).
    pub fn with_stroke_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set interior color for filled line endings (RGB).
    pub fn with_fill_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.interior_color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set line width.
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = Some(width);
        self
    }

    /// Set border style.
    pub fn with_border_style(mut self, style: BorderStyleType) -> Self {
        self.border_style = Some(style);
        self
    }

    /// Set leader line length.
    pub fn with_leader_line(mut self, length: f64) -> Self {
        self.leader_line = Some(length);
        self
    }

    /// Set leader line offset.
    pub fn with_leader_offset(mut self, offset: f64) -> Self {
        self.leader_line_offset = Some(offset);
        self
    }

    /// Set caption text.
    pub fn with_caption(mut self, text: impl Into<String>) -> Self {
        self.caption = true;
        self.contents = Some(text.into());
        self
    }

    /// Set caption position.
    pub fn with_caption_position(mut self, position: CaptionPosition) -> Self {
        self.caption_position = position;
        self
    }

    /// Set opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Set author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set subject.
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Calculate bounding rectangle.
    pub fn calculate_rect(&self) -> Rect {
        let min_x = self.start.0.min(self.end.0);
        let max_x = self.start.0.max(self.end.0);
        let min_y = self.start.1.min(self.end.1);
        let max_y = self.start.1.max(self.end.1);

        // Add margin for line endings
        let margin = 10.0;
        Rect::new(
            (min_x - margin) as f32,
            (min_y - margin) as f32,
            (max_x - min_x + 2.0 * margin) as f32,
            (max_y - min_y + 2.0 * margin) as f32,
        )
    }

    /// Build the annotation dictionary.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Line".to_string()));

        // Rectangle
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

        // Line coordinates (L entry) - required for Line annotations
        dict.insert(
            "L".to_string(),
            Object::Array(vec![
                Object::Real(self.start.0),
                Object::Real(self.start.1),
                Object::Real(self.end.0),
                Object::Real(self.end.1),
            ]),
        );

        // Line endings (LE entry)
        if self.line_endings != (LineEndingStyle::None, LineEndingStyle::None) {
            dict.insert(
                "LE".to_string(),
                Object::Array(vec![
                    Object::Name(self.line_endings.0.pdf_name().to_string()),
                    Object::Name(self.line_endings.1.pdf_name().to_string()),
                ]),
            );
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Color (C entry) - stroke color
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

        // Interior color (IC entry)
        if let Some(ref color) = self.interior_color {
            if let Some(color_array) = color.to_array() {
                if !color_array.is_empty() {
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
        }

        // Opacity
        if let Some(opacity) = self.opacity {
            dict.insert("CA".to_string(), Object::Real(opacity as f64));
        }

        // Border style (BS entry)
        if self.border_style.is_some() || self.line_width.is_some() {
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
            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Leader line (LL entry)
        if let Some(ll) = self.leader_line {
            dict.insert("LL".to_string(), Object::Real(ll));
        }

        // Leader line offset (LLO entry)
        if let Some(llo) = self.leader_line_offset {
            dict.insert("LLO".to_string(), Object::Real(llo));
        }

        // Leader line extension (LLE entry)
        if let Some(lle) = self.leader_line_extension {
            dict.insert("LLE".to_string(), Object::Real(lle));
        }

        // Caption (Cap entry)
        if self.caption {
            dict.insert("Cap".to_string(), Object::Boolean(true));
        }

        // Caption position (CP entry)
        if self.caption && self.caption_position != CaptionPosition::Inline {
            dict.insert(
                "CP".to_string(),
                Object::Name(self.caption_position.pdf_name().to_string()),
            );
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
}

// ============================================================================
// Shape Annotation (Square/Circle)
// ============================================================================

/// Shape type for Square/Circle annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    /// Square/Rectangle
    Square,
    /// Circle/Ellipse
    Circle,
}

impl ShapeType {
    /// Get PDF subtype name.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            Self::Square => "Square",
            Self::Circle => "Circle",
        }
    }
}

/// A Shape annotation (Square or Circle) per PDF spec Section 12.5.6.8.
#[derive(Debug, Clone)]
pub struct ShapeAnnotation {
    /// Bounding rectangle
    pub rect: Rect,
    /// Shape type
    pub shape_type: ShapeType,
    /// Stroke color
    pub color: Option<AnnotationColor>,
    /// Interior (fill) color
    pub interior_color: Option<AnnotationColor>,
    /// Opacity
    pub opacity: Option<f32>,
    /// Border style
    pub border_style: Option<BorderStyleType>,
    /// Border width
    pub border_width: Option<f32>,
    /// Border effect
    pub border_effect: Option<BorderEffect>,
    /// Rectangle differences (inner content area)
    pub rect_differences: Option<[f32; 4]>,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

impl ShapeAnnotation {
    /// Create a new shape annotation.
    pub fn new(rect: Rect, shape_type: ShapeType) -> Self {
        Self {
            rect,
            shape_type,
            color: Some(AnnotationColor::black()),
            interior_color: None,
            opacity: None,
            border_style: None,
            border_width: Some(1.0),
            border_effect: None,
            rect_differences: None,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
        }
    }

    /// Create a square/rectangle annotation.
    pub fn square(rect: Rect) -> Self {
        Self::new(rect, ShapeType::Square)
    }

    /// Create a circle/ellipse annotation.
    pub fn circle(rect: Rect) -> Self {
        Self::new(rect, ShapeType::Circle)
    }

    /// Set stroke color (RGB).
    pub fn with_stroke_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set fill color (RGB).
    pub fn with_fill_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.interior_color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set no fill (transparent interior).
    pub fn with_no_fill(mut self) -> Self {
        self.interior_color = None;
        self
    }

    /// Set border width.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = Some(width);
        self
    }

    /// Set border style.
    pub fn with_border_style(mut self, style: BorderStyleType) -> Self {
        self.border_style = Some(style);
        self
    }

    /// Set border effect (cloudy effect).
    pub fn with_border_effect(mut self, effect: BorderEffect) -> Self {
        self.border_effect = Some(effect);
        self
    }

    /// Set opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Set content/comment.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set subject.
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

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name(self.shape_type.pdf_name().to_string()));

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

        // Interior color (IC entry)
        if let Some(ref color) = self.interior_color {
            if let Some(color_array) = color.to_array() {
                if !color_array.is_empty() {
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
        }

        // Opacity
        if let Some(opacity) = self.opacity {
            dict.insert("CA".to_string(), Object::Real(opacity as f64));
        }

        // Border style (BS entry)
        if self.border_style.is_some() || self.border_width.is_some() {
            let mut bs = HashMap::new();
            bs.insert("Type".to_string(), Object::Name("Border".to_string()));
            if let Some(width) = self.border_width {
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
            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Border effect (BE entry)
        if let Some(ref be) = self.border_effect {
            let mut be_dict = HashMap::new();
            be_dict.insert("S".to_string(), Object::Name(be.style.pdf_name().to_string()));
            if be.intensity > 0.0 {
                be_dict.insert("I".to_string(), Object::Real(be.intensity as f64));
            }
            dict.insert("BE".to_string(), Object::Dictionary(be_dict));
        }

        // Rectangle differences (RD entry)
        if let Some(rd) = self.rect_differences {
            dict.insert(
                "RD".to_string(),
                Object::Array(vec![
                    Object::Real(rd[0] as f64),
                    Object::Real(rd[1] as f64),
                    Object::Real(rd[2] as f64),
                    Object::Real(rd[3] as f64),
                ]),
            );
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
}

// ============================================================================
// Polygon/PolyLine Annotation
// ============================================================================

/// Polygon type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonType {
    /// Closed polygon
    Polygon,
    /// Open polyline
    PolyLine,
}

impl PolygonType {
    /// Get PDF subtype name.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            Self::Polygon => "Polygon",
            Self::PolyLine => "PolyLine",
        }
    }
}

/// A Polygon or PolyLine annotation per PDF spec Section 12.5.6.9.
#[derive(Debug, Clone)]
pub struct PolygonAnnotation {
    /// Vertices as (x, y) coordinate pairs
    pub vertices: Vec<(f64, f64)>,
    /// Whether this is a closed polygon or open polyline
    pub polygon_type: PolygonType,
    /// Line endings for PolyLine (start, end)
    pub line_endings: Option<(LineEndingStyle, LineEndingStyle)>,
    /// Stroke color
    pub color: Option<AnnotationColor>,
    /// Interior (fill) color (for Polygon)
    pub interior_color: Option<AnnotationColor>,
    /// Opacity
    pub opacity: Option<f32>,
    /// Border style
    pub border_style: Option<BorderStyleType>,
    /// Border width
    pub border_width: Option<f32>,
    /// Border effect
    pub border_effect: Option<BorderEffect>,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
}

impl PolygonAnnotation {
    /// Create a closed polygon.
    pub fn polygon(vertices: Vec<(f64, f64)>) -> Self {
        Self {
            vertices,
            polygon_type: PolygonType::Polygon,
            line_endings: None,
            color: Some(AnnotationColor::black()),
            interior_color: None,
            opacity: None,
            border_style: None,
            border_width: Some(1.0),
            border_effect: None,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
        }
    }

    /// Create an open polyline.
    pub fn polyline(vertices: Vec<(f64, f64)>) -> Self {
        Self {
            vertices,
            polygon_type: PolygonType::PolyLine,
            line_endings: None,
            color: Some(AnnotationColor::black()),
            interior_color: None,
            opacity: None,
            border_style: None,
            border_width: Some(1.0),
            border_effect: None,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
        }
    }

    /// Set line endings (for PolyLine only).
    pub fn with_line_endings(mut self, start: LineEndingStyle, end: LineEndingStyle) -> Self {
        self.line_endings = Some((start, end));
        self
    }

    /// Set stroke color (RGB).
    pub fn with_stroke_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set fill color (RGB).
    pub fn with_fill_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.interior_color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set no fill.
    pub fn with_no_fill(mut self) -> Self {
        self.interior_color = None;
        self
    }

    /// Set border width.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = Some(width);
        self
    }

    /// Set border style.
    pub fn with_border_style(mut self, style: BorderStyleType) -> Self {
        self.border_style = Some(style);
        self
    }

    /// Set border effect.
    pub fn with_border_effect(mut self, effect: BorderEffect) -> Self {
        self.border_effect = Some(effect);
        self
    }

    /// Set opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Set contents/comment.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set subject.
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Calculate bounding rectangle from vertices.
    pub fn calculate_rect(&self) -> Rect {
        if self.vertices.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for (x, y) in &self.vertices {
            min_x = min_x.min(*x);
            max_x = max_x.max(*x);
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }

        // Add small margin
        let margin = 5.0;
        Rect::new(
            (min_x - margin) as f32,
            (min_y - margin) as f32,
            (max_x - min_x + 2.0 * margin) as f32,
            (max_y - min_y + 2.0 * margin) as f32,
        )
    }

    /// Build the annotation dictionary.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name(self.polygon_type.pdf_name().to_string()));

        // Rectangle (calculated from vertices)
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

        // Vertices (required)
        let vertices: Vec<Object> = self
            .vertices
            .iter()
            .flat_map(|(x, y)| vec![Object::Real(*x), Object::Real(*y)])
            .collect();
        dict.insert("Vertices".to_string(), Object::Array(vertices));

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Line endings (for PolyLine)
        if let Some((start, end)) = &self.line_endings {
            if self.polygon_type == PolygonType::PolyLine {
                dict.insert(
                    "LE".to_string(),
                    Object::Array(vec![
                        Object::Name(start.pdf_name().to_string()),
                        Object::Name(end.pdf_name().to_string()),
                    ]),
                );
            }
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

        // Interior color (IC entry)
        if let Some(ref color) = self.interior_color {
            if let Some(color_array) = color.to_array() {
                if !color_array.is_empty() {
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
        }

        // Opacity
        if let Some(opacity) = self.opacity {
            dict.insert("CA".to_string(), Object::Real(opacity as f64));
        }

        // Border style (BS entry)
        if self.border_style.is_some() || self.border_width.is_some() {
            let mut bs = HashMap::new();
            bs.insert("Type".to_string(), Object::Name("Border".to_string()));
            if let Some(width) = self.border_width {
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
            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Border effect (BE entry)
        if let Some(ref be) = self.border_effect {
            let mut be_dict = HashMap::new();
            be_dict.insert("S".to_string(), Object::Name(be.style.pdf_name().to_string()));
            if be.intensity > 0.0 {
                be_dict.insert("I".to_string(), Object::Real(be.intensity as f64));
            }
            dict.insert("BE".to_string(), Object::Dictionary(be_dict));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation_types::BorderEffectStyle;

    // ========== Line Annotation Tests ==========

    #[test]
    fn test_line_annotation_new() {
        let line = LineAnnotation::new((100.0, 100.0), (200.0, 200.0));
        assert_eq!(line.start, (100.0, 100.0));
        assert_eq!(line.end, (200.0, 200.0));
        assert_eq!(line.line_endings, (LineEndingStyle::None, LineEndingStyle::None));
    }

    #[test]
    fn test_line_annotation_arrow() {
        let line = LineAnnotation::arrow((0.0, 0.0), (100.0, 100.0));
        assert_eq!(line.line_endings.1, LineEndingStyle::OpenArrow);
    }

    #[test]
    fn test_line_annotation_double_arrow() {
        let line = LineAnnotation::double_arrow((0.0, 0.0), (100.0, 100.0));
        assert_eq!(line.line_endings.0, LineEndingStyle::OpenArrow);
        assert_eq!(line.line_endings.1, LineEndingStyle::OpenArrow);
    }

    #[test]
    fn test_line_annotation_build() {
        let line = LineAnnotation::new((100.0, 200.0), (300.0, 400.0))
            .with_stroke_color(1.0, 0.0, 0.0)
            .with_line_endings(LineEndingStyle::None, LineEndingStyle::ClosedArrow);

        let dict = line.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Line".to_string())));
        assert!(dict.contains_key("L")); // Line coordinates
        assert!(dict.contains_key("LE")); // Line endings
        assert!(dict.contains_key("C")); // Color
    }

    #[test]
    fn test_line_with_caption() {
        let line = LineAnnotation::new((100.0, 100.0), (200.0, 100.0))
            .with_caption("10 cm")
            .with_caption_position(CaptionPosition::Top);

        assert!(line.caption);
        assert_eq!(line.contents, Some("10 cm".to_string()));

        let dict = line.build(&[]);
        assert_eq!(dict.get("Cap"), Some(&Object::Boolean(true)));
        assert_eq!(dict.get("CP"), Some(&Object::Name("Top".to_string())));
    }

    // ========== Shape Annotation Tests ==========

    #[test]
    fn test_shape_annotation_square() {
        let rect = Rect::new(72.0, 600.0, 100.0, 80.0);
        let shape = ShapeAnnotation::square(rect);

        assert_eq!(shape.shape_type, ShapeType::Square);
    }

    #[test]
    fn test_shape_annotation_circle() {
        let rect = Rect::new(72.0, 600.0, 100.0, 100.0);
        let shape = ShapeAnnotation::circle(rect);

        assert_eq!(shape.shape_type, ShapeType::Circle);
    }

    #[test]
    fn test_shape_annotation_build() {
        let rect = Rect::new(100.0, 500.0, 150.0, 100.0);
        let shape = ShapeAnnotation::square(rect)
            .with_stroke_color(0.0, 0.0, 1.0)
            .with_fill_color(0.8, 0.8, 1.0)
            .with_border_width(2.0);

        let dict = shape.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Square".to_string())));
        assert!(dict.contains_key("C")); // Stroke color
        assert!(dict.contains_key("IC")); // Interior color
        assert!(dict.contains_key("BS")); // Border style
    }

    #[test]
    fn test_shape_annotation_circle_build() {
        let rect = Rect::new(200.0, 400.0, 80.0, 80.0);
        let shape = ShapeAnnotation::circle(rect).with_stroke_color(1.0, 0.0, 0.0);

        let dict = shape.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Circle".to_string())));
    }

    // ========== Polygon Annotation Tests ==========

    #[test]
    fn test_polygon_annotation_triangle() {
        let vertices = vec![(100.0, 100.0), (150.0, 200.0), (50.0, 200.0)];
        let polygon = PolygonAnnotation::polygon(vertices.clone());

        assert_eq!(polygon.vertices, vertices);
        assert_eq!(polygon.polygon_type, PolygonType::Polygon);
    }

    #[test]
    fn test_polyline_annotation() {
        let vertices = vec![(100.0, 100.0), (200.0, 150.0), (300.0, 100.0)];
        let polyline = PolygonAnnotation::polyline(vertices);

        assert_eq!(polyline.polygon_type, PolygonType::PolyLine);
    }

    #[test]
    fn test_polygon_build() {
        let vertices = vec![(100.0, 100.0), (200.0, 100.0), (150.0, 200.0)];
        let polygon = PolygonAnnotation::polygon(vertices)
            .with_stroke_color(0.0, 0.5, 0.0)
            .with_fill_color(0.8, 1.0, 0.8);

        let dict = polygon.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Polygon".to_string())));
        assert!(dict.contains_key("Vertices"));
        assert!(dict.contains_key("C")); // Stroke color
        assert!(dict.contains_key("IC")); // Interior color
    }

    #[test]
    fn test_polyline_with_line_endings() {
        let vertices = vec![(100.0, 100.0), (200.0, 150.0), (300.0, 100.0)];
        let polyline = PolygonAnnotation::polyline(vertices)
            .with_line_endings(LineEndingStyle::Circle, LineEndingStyle::OpenArrow);

        let dict = polyline.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("PolyLine".to_string())));
        assert!(dict.contains_key("LE")); // Line endings
    }

    #[test]
    fn test_polygon_calculate_rect() {
        let vertices = vec![(50.0, 50.0), (150.0, 50.0), (100.0, 150.0)];
        let polygon = PolygonAnnotation::polygon(vertices);

        let rect = polygon.calculate_rect();

        // Should contain all vertices with margin
        assert!(rect.x < 50.0);
        assert!(rect.y < 50.0);
        assert!(rect.x + rect.width > 150.0);
        assert!(rect.y + rect.height > 150.0);
    }

    // ========== Additional Coverage Tests ==========

    // --- CaptionPosition tests ---

    #[test]
    fn test_caption_position_pdf_name() {
        assert_eq!(CaptionPosition::Inline.pdf_name(), "Inline");
        assert_eq!(CaptionPosition::Top.pdf_name(), "Top");
    }

    #[test]
    fn test_caption_position_default() {
        let pos = CaptionPosition::default();
        assert_eq!(pos, CaptionPosition::Inline);
    }

    // --- ShapeType tests ---

    #[test]
    fn test_shape_type_pdf_name() {
        assert_eq!(ShapeType::Square.pdf_name(), "Square");
        assert_eq!(ShapeType::Circle.pdf_name(), "Circle");
    }

    // --- PolygonType tests ---

    #[test]
    fn test_polygon_type_pdf_name() {
        assert_eq!(PolygonType::Polygon.pdf_name(), "Polygon");
        assert_eq!(PolygonType::PolyLine.pdf_name(), "PolyLine");
    }

    // --- LineAnnotation builder method tests ---

    #[test]
    fn test_line_annotation_dimension() {
        let line = LineAnnotation::dimension((50.0, 50.0), (200.0, 50.0), 20.0);
        assert_eq!(line.line_endings.0, LineEndingStyle::OpenArrow);
        assert_eq!(line.line_endings.1, LineEndingStyle::OpenArrow);
        assert_eq!(line.leader_line, Some(20.0));
    }

    #[test]
    fn test_line_annotation_with_stroke_color() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_stroke_color(1.0, 0.0, 0.0);
        match line.color {
            Some(AnnotationColor::Rgb(r, g, b)) => {
                assert!((r - 1.0).abs() < 0.001);
                assert!((g - 0.0).abs() < 0.001);
                assert!((b - 0.0).abs() < 0.001);
            },
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_line_annotation_with_fill_color() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_fill_color(0.0, 1.0, 0.0);
        assert!(line.interior_color.is_some());
    }

    #[test]
    fn test_line_annotation_with_line_width() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_line_width(3.0);
        assert_eq!(line.line_width, Some(3.0));
    }

    #[test]
    fn test_line_annotation_with_border_style() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0))
            .with_border_style(BorderStyleType::Dashed);
        assert_eq!(line.border_style, Some(BorderStyleType::Dashed));
    }

    #[test]
    fn test_line_annotation_with_leader_line() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_leader_line(15.0);
        assert_eq!(line.leader_line, Some(15.0));
    }

    #[test]
    fn test_line_annotation_with_leader_offset() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_leader_offset(5.0);
        assert_eq!(line.leader_line_offset, Some(5.0));
    }

    #[test]
    fn test_line_annotation_with_opacity() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_opacity(0.5);
        assert_eq!(line.opacity, Some(0.5));
    }

    #[test]
    fn test_line_annotation_opacity_clamped() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_opacity(1.5);
        assert_eq!(line.opacity, Some(1.0)); // Clamped to 1.0

        let line2 = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_opacity(-0.5);
        assert_eq!(line2.opacity, Some(0.0)); // Clamped to 0.0
    }

    #[test]
    fn test_line_annotation_with_author() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_author("John");
        assert_eq!(line.author, Some("John".to_string()));
    }

    #[test]
    fn test_line_annotation_with_subject() {
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_subject("Measurement");
        assert_eq!(line.subject, Some("Measurement".to_string()));
    }

    #[test]
    fn test_line_annotation_with_flags() {
        let flags = AnnotationFlags::new(AnnotationFlags::PRINT | AnnotationFlags::LOCKED);
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_flags(flags);
        assert_eq!(line.flags.bits(), AnnotationFlags::PRINT | AnnotationFlags::LOCKED);
    }

    #[test]
    fn test_line_annotation_calculate_rect() {
        let line = LineAnnotation::new((50.0, 100.0), (200.0, 300.0));
        let rect = line.calculate_rect();
        // Should encompass both points with margin
        assert!(rect.x < 50.0);
        assert!(rect.y < 100.0);
        assert!(rect.x as f64 + rect.width as f64 > 200.0);
        assert!(rect.y as f64 + rect.height as f64 > 300.0);
    }

    #[test]
    fn test_line_annotation_build_full() {
        let line = LineAnnotation::new((100.0, 200.0), (300.0, 400.0))
            .with_stroke_color(1.0, 0.0, 0.0)
            .with_fill_color(0.0, 1.0, 0.0)
            .with_line_width(2.0)
            .with_border_style(BorderStyleType::Solid)
            .with_opacity(0.8)
            .with_leader_line(10.0)
            .with_leader_offset(5.0)
            .with_caption("Test caption")
            .with_caption_position(CaptionPosition::Top)
            .with_author("Author")
            .with_subject("Subject")
            .with_line_endings(LineEndingStyle::OpenArrow, LineEndingStyle::ClosedArrow);

        let dict = line.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Line".to_string())));
        assert!(dict.contains_key("L"));
        assert!(dict.contains_key("LE"));
        assert!(dict.contains_key("C"));
        assert!(dict.contains_key("IC"));
        assert!(dict.contains_key("CA"));
        assert!(dict.contains_key("BS"));
        assert!(dict.contains_key("LL"));
        assert!(dict.contains_key("LLO"));
        assert!(dict.contains_key("Cap"));
        assert!(dict.contains_key("CP"));
        assert!(dict.contains_key("Contents"));
        assert!(dict.contains_key("T")); // Author
        assert!(dict.contains_key("Subj")); // Subject
        assert!(dict.contains_key("F")); // Flags
    }

    #[test]
    fn test_line_annotation_build_leader_line_extension() {
        let mut line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0));
        line.leader_line_extension = Some(5.0);
        let dict = line.build(&[]);
        assert_eq!(dict.get("LLE"), Some(&Object::Real(5.0)));
    }

    #[test]
    fn test_line_annotation_build_no_caption_cp() {
        // Without caption enabled, CP should not appear
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0))
            .with_caption_position(CaptionPosition::Top);
        // caption is false
        let dict = line.build(&[]);
        assert!(!dict.contains_key("Cap"));
        assert!(!dict.contains_key("CP"));
    }

    #[test]
    fn test_line_annotation_build_caption_inline_no_cp() {
        // With caption and inline position, CP should not appear
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_caption("Test");
        // caption_position defaults to Inline
        let dict = line.build(&[]);
        assert!(dict.contains_key("Cap"));
        assert!(!dict.contains_key("CP")); // Inline is default, don't write CP
    }

    #[test]
    fn test_line_annotation_build_all_border_styles() {
        for style in &[
            BorderStyleType::Solid,
            BorderStyleType::Dashed,
            BorderStyleType::Beveled,
            BorderStyleType::Inset,
            BorderStyleType::Underline,
        ] {
            let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0)).with_border_style(*style);
            let dict = line.build(&[]);
            assert!(dict.contains_key("BS"));
            match dict.get("BS") {
                Some(Object::Dictionary(bs)) => {
                    assert!(bs.contains_key("S"));
                },
                _ => panic!("Expected BS dictionary"),
            }
        }
    }

    #[test]
    fn test_line_annotation_build_no_line_endings() {
        // Default line endings are None/None, so LE should not appear
        let line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0));
        let dict = line.build(&[]);
        assert!(!dict.contains_key("LE"));
    }

    // --- ShapeAnnotation builder method tests ---

    #[test]
    fn test_shape_annotation_with_no_fill() {
        let shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0))
            .with_fill_color(1.0, 0.0, 0.0)
            .with_no_fill();
        assert!(shape.interior_color.is_none());
    }

    #[test]
    fn test_shape_annotation_with_border_width() {
        let shape =
            ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_border_width(3.0);
        assert_eq!(shape.border_width, Some(3.0));
    }

    #[test]
    fn test_shape_annotation_with_border_style() {
        let shape = ShapeAnnotation::circle(Rect::new(0.0, 0.0, 100.0, 100.0))
            .with_border_style(BorderStyleType::Dashed);
        assert_eq!(shape.border_style, Some(BorderStyleType::Dashed));
    }

    #[test]
    fn test_shape_annotation_with_border_effect() {
        let effect = BorderEffect {
            style: BorderEffectStyle::Cloudy,
            intensity: 1.5,
        };
        let shape =
            ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_border_effect(effect);
        assert!(shape.border_effect.is_some());
    }

    #[test]
    fn test_shape_annotation_with_opacity() {
        let shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_opacity(0.5);
        assert_eq!(shape.opacity, Some(0.5));
    }

    #[test]
    fn test_shape_annotation_opacity_clamped() {
        let shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_opacity(2.0);
        assert_eq!(shape.opacity, Some(1.0));
    }

    #[test]
    fn test_shape_annotation_with_contents() {
        let shape =
            ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_contents("A note");
        assert_eq!(shape.contents, Some("A note".to_string()));
    }

    #[test]
    fn test_shape_annotation_with_author() {
        let shape = ShapeAnnotation::circle(Rect::new(0.0, 0.0, 100.0, 100.0)).with_author("Jane");
        assert_eq!(shape.author, Some("Jane".to_string()));
    }

    #[test]
    fn test_shape_annotation_with_subject() {
        let shape =
            ShapeAnnotation::circle(Rect::new(0.0, 0.0, 100.0, 100.0)).with_subject("Review");
        assert_eq!(shape.subject, Some("Review".to_string()));
    }

    #[test]
    fn test_shape_annotation_with_flags() {
        let flags = AnnotationFlags::new(AnnotationFlags::HIDDEN);
        let shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_flags(flags);
        assert_eq!(shape.flags.bits(), AnnotationFlags::HIDDEN);
    }

    #[test]
    fn test_shape_annotation_build_full() {
        let effect = BorderEffect {
            style: BorderEffectStyle::Cloudy,
            intensity: 1.0,
        };
        let shape = ShapeAnnotation::square(Rect::new(50.0, 50.0, 200.0, 150.0))
            .with_stroke_color(0.0, 0.0, 1.0)
            .with_fill_color(0.9, 0.9, 1.0)
            .with_border_width(2.0)
            .with_border_style(BorderStyleType::Solid)
            .with_border_effect(effect)
            .with_opacity(0.7)
            .with_contents("Comment text")
            .with_author("Author")
            .with_subject("Subject");

        let dict = shape.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Square".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("C"));
        assert!(dict.contains_key("IC"));
        assert!(dict.contains_key("CA"));
        assert!(dict.contains_key("BS"));
        assert!(dict.contains_key("BE"));
        assert!(dict.contains_key("Contents"));
        assert!(dict.contains_key("T"));
        assert!(dict.contains_key("Subj"));
        assert!(dict.contains_key("F"));
    }

    #[test]
    fn test_shape_annotation_build_border_effect_no_intensity() {
        let effect = BorderEffect {
            style: BorderEffectStyle::None,
            intensity: 0.0,
        };
        let shape =
            ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0)).with_border_effect(effect);

        let dict = shape.build(&[]);
        match dict.get("BE") {
            Some(Object::Dictionary(be)) => {
                // Intensity is 0.0, should not appear
                assert!(!be.contains_key("I"));
            },
            _ => panic!("Expected BE dictionary"),
        }
    }

    #[test]
    fn test_shape_annotation_rect_differences() {
        let mut shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0));
        shape.rect_differences = Some([5.0, 5.0, 5.0, 5.0]);

        let dict = shape.build(&[]);
        assert!(dict.contains_key("RD"));
        match dict.get("RD") {
            Some(Object::Array(arr)) => assert_eq!(arr.len(), 4),
            _ => panic!("Expected RD array"),
        }
    }

    #[test]
    fn test_shape_annotation_build_all_border_styles() {
        for style in &[
            BorderStyleType::Solid,
            BorderStyleType::Dashed,
            BorderStyleType::Beveled,
            BorderStyleType::Inset,
            BorderStyleType::Underline,
        ] {
            let shape = ShapeAnnotation::circle(Rect::new(0.0, 0.0, 100.0, 100.0))
                .with_border_style(*style);
            let dict = shape.build(&[]);
            assert!(dict.contains_key("BS"));
        }
    }

    // --- PolygonAnnotation builder method tests ---

    #[test]
    fn test_polygon_with_stroke_color() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0), (50.0, 100.0)])
            .with_stroke_color(1.0, 0.0, 0.0);
        assert!(polygon.color.is_some());
    }

    #[test]
    fn test_polygon_with_fill_color() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0), (50.0, 100.0)])
            .with_fill_color(0.0, 1.0, 0.0);
        assert!(polygon.interior_color.is_some());
    }

    #[test]
    fn test_polygon_with_no_fill() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0), (50.0, 100.0)])
            .with_fill_color(1.0, 0.0, 0.0)
            .with_no_fill();
        assert!(polygon.interior_color.is_none());
    }

    #[test]
    fn test_polygon_with_border_width() {
        let polygon =
            PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0)]).with_border_width(3.0);
        assert_eq!(polygon.border_width, Some(3.0));
    }

    #[test]
    fn test_polygon_with_border_style() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0)])
            .with_border_style(BorderStyleType::Beveled);
        assert_eq!(polygon.border_style, Some(BorderStyleType::Beveled));
    }

    #[test]
    fn test_polygon_with_border_effect() {
        let effect = BorderEffect {
            style: BorderEffectStyle::Cloudy,
            intensity: 2.0,
        };
        let polygon =
            PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0)]).with_border_effect(effect);
        assert!(polygon.border_effect.is_some());
    }

    #[test]
    fn test_polygon_with_opacity() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0)]).with_opacity(0.3);
        assert_eq!(polygon.opacity, Some(0.3));
    }

    #[test]
    fn test_polygon_opacity_clamped() {
        let polygon = PolygonAnnotation::polygon(vec![]).with_opacity(-1.0);
        assert_eq!(polygon.opacity, Some(0.0));
    }

    #[test]
    fn test_polygon_with_contents() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0)]).with_contents("Note");
        assert_eq!(polygon.contents, Some("Note".to_string()));
    }

    #[test]
    fn test_polygon_with_author() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0)]).with_author("Bob");
        assert_eq!(polygon.author, Some("Bob".to_string()));
    }

    #[test]
    fn test_polygon_with_subject() {
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0)]).with_subject("Area");
        assert_eq!(polygon.subject, Some("Area".to_string()));
    }

    #[test]
    fn test_polygon_with_flags() {
        let flags = AnnotationFlags::new(AnnotationFlags::READ_ONLY);
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0)]).with_flags(flags);
        assert_eq!(polygon.flags.bits(), AnnotationFlags::READ_ONLY);
    }

    #[test]
    fn test_polygon_calculate_rect_empty() {
        let polygon = PolygonAnnotation::polygon(vec![]);
        let rect = polygon.calculate_rect();
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }

    #[test]
    fn test_polygon_build_full() {
        let effect = BorderEffect {
            style: BorderEffectStyle::Cloudy,
            intensity: 1.5,
        };
        let polygon =
            PolygonAnnotation::polygon(vec![(100.0, 100.0), (200.0, 100.0), (150.0, 200.0)])
                .with_stroke_color(0.0, 0.0, 1.0)
                .with_fill_color(0.8, 0.8, 1.0)
                .with_border_width(2.0)
                .with_border_style(BorderStyleType::Dashed)
                .with_border_effect(effect)
                .with_opacity(0.9)
                .with_contents("Comment")
                .with_author("Author")
                .with_subject("Subject");

        let dict = polygon.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Polygon".to_string())));
        assert!(dict.contains_key("Vertices"));
        assert!(dict.contains_key("C"));
        assert!(dict.contains_key("IC"));
        assert!(dict.contains_key("CA"));
        assert!(dict.contains_key("BS"));
        assert!(dict.contains_key("BE"));
        assert!(dict.contains_key("Contents"));
        assert!(dict.contains_key("T"));
        assert!(dict.contains_key("Subj"));
        assert!(dict.contains_key("F"));
    }

    #[test]
    fn test_polyline_build_full() {
        let polyline = PolygonAnnotation::polyline(vec![(0.0, 0.0), (50.0, 100.0), (100.0, 0.0)])
            .with_line_endings(LineEndingStyle::Square, LineEndingStyle::Diamond)
            .with_stroke_color(1.0, 0.0, 0.0);

        let dict = polyline.build(&[]);

        assert_eq!(dict.get("Subtype"), Some(&Object::Name("PolyLine".to_string())));
        assert!(dict.contains_key("LE"));
        assert!(dict.contains_key("Vertices"));
    }

    #[test]
    fn test_polygon_line_endings_ignored_for_polygon() {
        // Line endings should only appear for PolyLine, not Polygon
        let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 0.0), (50.0, 100.0)])
            .with_line_endings(LineEndingStyle::OpenArrow, LineEndingStyle::ClosedArrow);

        let dict = polygon.build(&[]);
        // LE should NOT be in the dict for Polygon type
        assert!(!dict.contains_key("LE"));
    }

    #[test]
    fn test_polygon_build_all_border_styles() {
        for style in &[
            BorderStyleType::Solid,
            BorderStyleType::Dashed,
            BorderStyleType::Beveled,
            BorderStyleType::Inset,
            BorderStyleType::Underline,
        ] {
            let polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 100.0)])
                .with_border_style(*style);
            let dict = polygon.build(&[]);
            assert!(dict.contains_key("BS"));
        }
    }

    #[test]
    fn test_polygon_build_border_effect_no_intensity() {
        let effect = BorderEffect {
            style: BorderEffectStyle::None,
            intensity: 0.0,
        };
        let polygon =
            PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 100.0)]).with_border_effect(effect);
        let dict = polygon.build(&[]);
        match dict.get("BE") {
            Some(Object::Dictionary(be)) => {
                assert!(!be.contains_key("I")); // intensity is 0
            },
            _ => panic!("Expected BE dictionary"),
        }
    }

    #[test]
    fn test_polygon_build_vertices_array() {
        let polygon = PolygonAnnotation::polygon(vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)]);
        let dict = polygon.build(&[]);
        match dict.get("Vertices") {
            Some(Object::Array(arr)) => {
                // 3 vertices x 2 coords = 6 values
                assert_eq!(arr.len(), 6);
                assert_eq!(arr[0], Object::Real(10.0));
                assert_eq!(arr[1], Object::Real(20.0));
                assert_eq!(arr[4], Object::Real(50.0));
                assert_eq!(arr[5], Object::Real(60.0));
            },
            _ => panic!("Expected Vertices array"),
        }
    }

    #[test]
    fn test_line_annotation_build_l_entry() {
        let line = LineAnnotation::new((10.0, 20.0), (30.0, 40.0));
        let dict = line.build(&[]);
        match dict.get("L") {
            Some(Object::Array(arr)) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr[0], Object::Real(10.0));
                assert_eq!(arr[1], Object::Real(20.0));
                assert_eq!(arr[2], Object::Real(30.0));
                assert_eq!(arr[3], Object::Real(40.0));
            },
            _ => panic!("Expected L array"),
        }
    }

    #[test]
    fn test_shape_annotation_circle_rect_values() {
        let rect = Rect::new(100.0, 200.0, 50.0, 50.0);
        let shape = ShapeAnnotation::circle(rect);
        let dict = shape.build(&[]);
        match dict.get("Rect") {
            Some(Object::Array(arr)) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr[0], Object::Real(100.0));
                assert_eq!(arr[1], Object::Real(200.0));
                assert_eq!(arr[2], Object::Real(150.0)); // x + width
                assert_eq!(arr[3], Object::Real(250.0)); // y + height
            },
            _ => panic!("Expected Rect array"),
        }
    }

    #[test]
    fn test_shape_annotation_no_color() {
        let mut shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0));
        shape.color = None;
        let dict = shape.build(&[]);
        assert!(!dict.contains_key("C"));
    }

    #[test]
    fn test_line_annotation_no_color() {
        let mut line = LineAnnotation::new((0.0, 0.0), (100.0, 100.0));
        line.color = None;
        let dict = line.build(&[]);
        assert!(!dict.contains_key("C"));
    }

    #[test]
    fn test_polygon_no_color() {
        let mut polygon = PolygonAnnotation::polygon(vec![(0.0, 0.0), (100.0, 100.0)]);
        polygon.color = None;
        let dict = polygon.build(&[]);
        assert!(!dict.contains_key("C"));
    }

    #[test]
    fn test_shape_annotation_flags_zero() {
        let mut shape = ShapeAnnotation::square(Rect::new(0.0, 0.0, 100.0, 100.0));
        shape.flags = AnnotationFlags::new(0);
        let dict = shape.build(&[]);
        assert!(!dict.contains_key("F"));
    }
}
