//! FreeText annotations (text boxes) for PDF generation.
//!
//! This module provides support for FreeText annotations per PDF spec Section 12.5.6.6.
//! FreeText annotations display text directly on the page without a pop-up window.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::FreeTextAnnotation;
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a text box
//! let textbox = FreeTextAnnotation::new(
//!     Rect::new(72.0, 700.0, 200.0, 50.0),
//!     "This is displayed directly on the page",
//! ).with_font("Helvetica", 12.0)
//!  .with_alignment(TextAlignment::Center);
//! ```

use crate::annotation_types::{
    AnnotationColor, AnnotationFlags, BorderStyleType, FreeTextIntent, LineEndingStyle,
    TextAlignment,
};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A FreeText annotation (text box displayed on page).
///
/// Per PDF spec Section 12.5.6.6, a FreeText annotation displays text directly
/// on the page, unlike Text annotations which display a pop-up window.
#[derive(Debug, Clone)]
pub struct FreeTextAnnotation {
    /// Bounding rectangle for the text box
    pub rect: Rect,
    /// Text contents
    pub contents: String,
    /// Default appearance string (required) - specifies font and color
    /// Format: "/FontName size Tf r g b rg" e.g. "/Helv 12 Tf 0 0 0 rg"
    pub default_appearance: String,
    /// Text alignment (0=left, 1=center, 2=right)
    pub alignment: TextAlignment,
    /// Intent (FreeText, FreeTextCallout, FreeTextTypeWriter)
    pub intent: FreeTextIntent,
    /// Callout line coordinates for callout annotations
    /// For 2-point: [x1, y1, x2, y2]
    /// For 3-point: [x1, y1, x2, y2, x3, y3] (knee point)
    pub callout_line: Option<Vec<f64>>,
    /// Line ending style for callout line
    pub line_ending: Option<LineEndingStyle>,
    /// Rich text content (XHTML)
    pub rich_text: Option<String>,
    /// Default style string (CSS-like)
    pub default_style: Option<String>,
    /// Annotation color (border/text color)
    pub color: Option<AnnotationColor>,
    /// Interior color (background fill)
    pub interior_color: Option<AnnotationColor>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
    /// Border style
    pub border_style: Option<BorderStyleType>,
    /// Border width
    pub border_width: Option<f32>,
    /// Rectangle differences (inner margins)
    /// [left, top, right, bottom] in points from rect edges
    pub rect_differences: Option<[f32; 4]>,
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

impl FreeTextAnnotation {
    /// Create a new FreeText annotation with default appearance.
    ///
    /// # Arguments
    ///
    /// * `rect` - Bounding rectangle for the text box
    /// * `contents` - The text content to display
    pub fn new(rect: Rect, contents: impl Into<String>) -> Self {
        Self {
            rect,
            contents: contents.into(),
            default_appearance: "/Helv 12 Tf 0 0 0 rg".to_string(),
            alignment: TextAlignment::Left,
            intent: FreeTextIntent::FreeText,
            callout_line: None,
            line_ending: None,
            rich_text: None,
            default_style: None,
            color: None,
            interior_color: Some(AnnotationColor::white()), // White background by default
            opacity: None,
            border_style: None,
            border_width: None,
            rect_differences: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
            creation_date: None,
            modification_date: None,
        }
    }

    /// Create a callout annotation (text box with leader line).
    ///
    /// # Arguments
    ///
    /// * `rect` - Bounding rectangle for the text box
    /// * `contents` - The text content
    /// * `callout` - Callout line points [x1,y1, x2,y2] or [x1,y1, x2,y2, x3,y3]
    pub fn callout(rect: Rect, contents: impl Into<String>, callout: Vec<f64>) -> Self {
        Self::new(rect, contents)
            .with_intent(FreeTextIntent::FreeTextCallout)
            .with_callout_line(callout)
    }

    /// Create a typewriter annotation (plain text without border).
    pub fn typewriter(rect: Rect, contents: impl Into<String>) -> Self {
        Self::new(rect, contents).with_intent(FreeTextIntent::FreeTextTypeWriter)
    }

    /// Set the font and size for the default appearance.
    ///
    /// Common font names: Helvetica (Helv), Times-Roman (TiRo), Courier (Cour)
    pub fn with_font(mut self, font_name: &str, size: f32) -> Self {
        // Map common names to standard PDF names
        let pdf_font = match font_name.to_lowercase().as_str() {
            "helvetica" | "helv" | "arial" => "Helv",
            "times" | "times-roman" | "tiro" => "TiRo",
            "courier" | "cour" => "Cour",
            "symbol" => "Symb",
            "zapfdingbats" | "zapf" => "ZaDb",
            _ => font_name,
        };
        // Keep existing color or default to black
        let color_spec = if let Some(ref color) = self.color {
            match color {
                AnnotationColor::Rgb(r, g, b) => format!("{} {} {} rg", r, g, b),
                AnnotationColor::Gray(g) => format!("{} g", g),
                AnnotationColor::Cmyk(c, m, y, k) => format!("{} {} {} {} k", c, m, y, k),
                AnnotationColor::None => "0 0 0 rg".to_string(),
            }
        } else {
            "0 0 0 rg".to_string()
        };
        self.default_appearance = format!("/{} {} Tf {}", pdf_font, size, color_spec);
        self
    }

    /// Set the text alignment.
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the intent.
    pub fn with_intent(mut self, intent: FreeTextIntent) -> Self {
        self.intent = intent;
        self
    }

    /// Set callout line coordinates.
    pub fn with_callout_line(mut self, callout: Vec<f64>) -> Self {
        self.callout_line = Some(callout);
        self
    }

    /// Set line ending style for callout.
    pub fn with_line_ending(mut self, ending: LineEndingStyle) -> Self {
        self.line_ending = Some(ending);
        self
    }

    /// Set the text color (RGB).
    pub fn with_text_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some(AnnotationColor::Rgb(r, g, b));
        // Update default appearance with new color
        self = self.with_font("Helvetica", 12.0); // Re-apply to update DA
        self
    }

    /// Set the background (interior) color (RGB).
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.interior_color = Some(AnnotationColor::Rgb(r, g, b));
        self
    }

    /// Set no background (transparent).
    pub fn with_transparent_background(mut self) -> Self {
        self.interior_color = None;
        self
    }

    /// Set the border style.
    pub fn with_border(mut self, style: BorderStyleType, width: f32) -> Self {
        self.border_style = Some(style);
        self.border_width = Some(width);
        self
    }

    /// Set no border.
    pub fn with_no_border(mut self) -> Self {
        self.border_width = Some(0.0);
        self
    }

    /// Set inner padding/margins (rectangle differences).
    pub fn with_padding(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.rect_differences = Some([left, top, right, bottom]);
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

    /// Set rich text content (XHTML format).
    pub fn with_rich_text(mut self, xhtml: impl Into<String>) -> Self {
        self.rich_text = Some(xhtml.into());
        self
    }

    /// Build the annotation dictionary for PDF output.
    ///
    /// # Arguments
    ///
    /// * `_page_refs` - Page references (not used for FreeText, kept for API consistency)
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("FreeText".to_string()));

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

        // Default Appearance (required for FreeText)
        dict.insert("DA".to_string(), Object::text_string(&self.default_appearance));

        // Text alignment (Q entry)
        if self.alignment != TextAlignment::Left {
            dict.insert("Q".to_string(), Object::Integer(self.alignment.to_pdf_int() as i64));
        }

        // Intent (IT entry) - only if not default
        if self.intent != FreeTextIntent::FreeText {
            dict.insert("IT".to_string(), Object::Name(self.intent.pdf_name().to_string()));
        }

        // Callout line (CL entry)
        if let Some(ref callout) = self.callout_line {
            dict.insert(
                "CL".to_string(),
                Object::Array(callout.iter().map(|&v| Object::Real(v)).collect()),
            );
        }

        // Line ending (LE entry) for callout
        if let Some(ref ending) = self.line_ending {
            dict.insert("LE".to_string(), Object::Name(ending.pdf_name().to_string()));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Color (C entry) - border/text color
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

        // Interior color (IC entry) - background fill
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

        // Opacity (CA entry)
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

        // Rich text (RC entry)
        if let Some(ref rc) = self.rich_text {
            dict.insert("RC".to_string(), Object::text_string(rc));
        }

        // Default style (DS entry)
        if let Some(ref ds) = self.default_style {
            dict.insert("DS".to_string(), Object::text_string(ds));
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
    fn test_freetext_new() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let ft = FreeTextAnnotation::new(rect, "Test text");

        assert_eq!(ft.contents, "Test text");
        assert!(matches!(ft.alignment, TextAlignment::Left));
        assert!(matches!(ft.intent, FreeTextIntent::FreeText));
        assert!(ft.default_appearance.contains("/Helv"));
    }

    #[test]
    fn test_freetext_callout() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let callout = vec![100.0, 600.0, 150.0, 650.0, 172.0, 700.0];
        let ft = FreeTextAnnotation::callout(rect, "Callout text", callout.clone());

        assert!(matches!(ft.intent, FreeTextIntent::FreeTextCallout));
        assert_eq!(ft.callout_line, Some(callout));
    }

    #[test]
    fn test_freetext_typewriter() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let ft = FreeTextAnnotation::typewriter(rect, "Typed text");

        assert!(matches!(ft.intent, FreeTextIntent::FreeTextTypeWriter));
    }

    #[test]
    fn test_freetext_with_font() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let ft = FreeTextAnnotation::new(rect, "Text").with_font("Courier", 14.0);

        assert!(ft.default_appearance.contains("/Cour"));
        assert!(ft.default_appearance.contains("14"));
    }

    #[test]
    fn test_freetext_with_alignment() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let ft = FreeTextAnnotation::new(rect, "Centered").with_alignment(TextAlignment::Center);

        assert!(matches!(ft.alignment, TextAlignment::Center));

        let dict = ft.build(&[]);
        assert_eq!(dict.get("Q"), Some(&Object::Integer(1)));
    }

    #[test]
    fn test_freetext_build() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let ft = FreeTextAnnotation::new(rect, "Test content")
            .with_font("Helvetica", 12.0)
            .with_alignment(TextAlignment::Right)
            .with_background_color(1.0, 1.0, 0.8) // Light yellow
            .with_border(BorderStyleType::Solid, 1.0)
            .with_author("Author");

        let dict = ft.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("FreeText".to_string())));
        assert!(dict.contains_key("DA")); // Default Appearance
        assert_eq!(dict.get("Q"), Some(&Object::Integer(2))); // Right alignment
        assert!(dict.contains_key("IC")); // Interior color
        assert!(dict.contains_key("BS")); // Border style
        assert!(dict.contains_key("T")); // Author
    }

    #[test]
    fn test_freetext_callout_build() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let callout = vec![50.0, 650.0, 72.0, 700.0];
        let ft = FreeTextAnnotation::callout(rect, "Note", callout)
            .with_line_ending(LineEndingStyle::OpenArrow);

        let dict = ft.build(&[]);

        assert_eq!(dict.get("IT"), Some(&Object::Name("FreeTextCallout".to_string())));
        assert!(dict.contains_key("CL"));
        assert_eq!(dict.get("LE"), Some(&Object::Name("OpenArrow".to_string())));
    }

    #[test]
    fn test_freetext_fluent_builder() {
        let rect = Rect::new(100.0, 500.0, 150.0, 30.0);
        let ft = FreeTextAnnotation::new(rect, "Text")
            .with_font("Times", 10.0)
            .with_alignment(TextAlignment::Center)
            .with_background_color(0.9, 0.9, 1.0)
            .with_opacity(0.9)
            .with_padding(5.0, 5.0, 5.0, 5.0)
            .with_author("Reviewer")
            .with_subject("Comment");

        assert!(matches!(ft.alignment, TextAlignment::Center));
        assert_eq!(ft.opacity, Some(0.9));
        assert_eq!(ft.rect_differences, Some([5.0, 5.0, 5.0, 5.0]));
        assert_eq!(ft.author, Some("Reviewer".to_string()));
        assert_eq!(ft.subject, Some("Comment".to_string()));
    }

    #[test]
    fn test_freetext_no_border() {
        let rect = Rect::new(72.0, 700.0, 200.0, 50.0);
        let ft = FreeTextAnnotation::new(rect, "Text").with_no_border();

        assert_eq!(ft.border_width, Some(0.0));
    }
}
