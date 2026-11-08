//! Watermark annotations for PDF generation.
//!
//! This module provides support for Watermark annotations per PDF spec Section 12.5.6.22.
//! Watermark annotations are used to represent graphics that should appear as part
//! of the page content, but are defined as annotations for flexibility.
//!
//! # Features
//!
//! - Text-based watermarks with customizable font, size, color
//! - Rotation support for diagonal watermarks
//! - Opacity control for transparency
//! - FixedPrint mode for print-only or screen-only watermarks
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::WatermarkAnnotation;
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a diagonal "CONFIDENTIAL" watermark
//! let watermark = WatermarkAnnotation::new("CONFIDENTIAL")
//!     .with_rect(Rect::new(100.0, 300.0, 400.0, 200.0))
//!     .with_rotation(45.0)
//!     .with_opacity(0.3)
//!     .with_color(0.8, 0.0, 0.0)
//!     .with_font("Helvetica", 48.0);
//!
//! // Create a print-only watermark
//! let print_only = WatermarkAnnotation::new("DRAFT")
//!     .with_rect(rect)
//!     .fixed_print(true);
//! ```

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A Watermark annotation per PDF spec Section 12.5.6.22.
///
/// Watermark annotations display text or graphics intended to appear
/// as page background content. They support:
/// - Custom text with font and size
/// - Rotation for diagonal placement
/// - Transparency via opacity
/// - FixedPrint for print-only behavior
#[derive(Debug, Clone)]
pub struct WatermarkAnnotation {
    /// The watermark text
    text: String,
    /// Bounding rectangle
    rect: Rect,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    opacity: f32,
    /// Rotation angle in degrees (counter-clockwise)
    rotation: f32,
    /// Font name
    font_name: String,
    /// Font size in points
    font_size: f32,
    /// Text color (RGB, 0.0-1.0)
    color: (f32, f32, f32),
    /// FixedPrint dictionary settings
    fixed_print: Option<FixedPrintSettings>,
    /// Annotation flags
    flags: AnnotationFlags,
    /// Contents/comment
    contents: Option<String>,
}

/// Settings for FixedPrint behavior.
///
/// FixedPrint controls how the watermark appears when printing vs on screen.
#[derive(Debug, Clone)]
pub struct FixedPrintSettings {
    /// Horizontal translation in default user space units
    pub h: f32,
    /// Vertical translation in default user space units
    pub v: f32,
}

impl Default for FixedPrintSettings {
    fn default() -> Self {
        Self { h: 0.0, v: 0.0 }
    }
}

impl Default for WatermarkAnnotation {
    fn default() -> Self {
        Self {
            text: String::new(),
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            opacity: 0.3,
            rotation: 0.0,
            font_name: "Helvetica".to_string(),
            font_size: 48.0,
            color: (0.5, 0.5, 0.5), // Gray
            fixed_print: None,
            flags: AnnotationFlags::new(AnnotationFlags::PRINT | AnnotationFlags::READ_ONLY),
            contents: None,
        }
    }
}

impl WatermarkAnnotation {
    /// Create a new watermark annotation with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Create a "CONFIDENTIAL" watermark with default styling.
    pub fn confidential() -> Self {
        Self::new("CONFIDENTIAL")
            .with_color(0.8, 0.0, 0.0)
            .with_rotation(45.0)
    }

    /// Create a "DRAFT" watermark with default styling.
    pub fn draft() -> Self {
        Self::new("DRAFT")
            .with_color(0.5, 0.5, 0.5)
            .with_rotation(45.0)
    }

    /// Create a "SAMPLE" watermark with default styling.
    pub fn sample() -> Self {
        Self::new("SAMPLE")
            .with_color(0.0, 0.5, 0.0)
            .with_rotation(45.0)
    }

    /// Create a "DO NOT COPY" watermark with default styling.
    pub fn do_not_copy() -> Self {
        Self::new("DO NOT COPY")
            .with_color(0.8, 0.0, 0.0)
            .with_rotation(45.0)
    }

    /// Set the bounding rectangle for the watermark.
    pub fn with_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    /// Set the opacity (0.0 = transparent, 1.0 = opaque).
    ///
    /// Default is 0.3 for a subtle watermark effect.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set the rotation angle in degrees (counter-clockwise).
    ///
    /// Common values: 45.0 for diagonal, 0.0 for horizontal.
    pub fn with_rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Set the font name and size.
    pub fn with_font(mut self, name: impl Into<String>, size: f32) -> Self {
        self.font_name = name.into();
        self.font_size = size;
        self
    }

    /// Set the text color (RGB, 0.0-1.0).
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
        self
    }

    /// Enable FixedPrint mode for print-only watermarks.
    ///
    /// When enabled, the watermark appears only when printing,
    /// not on screen display.
    pub fn fixed_print(mut self, enabled: bool) -> Self {
        if enabled {
            self.fixed_print = Some(FixedPrintSettings::default());
        } else {
            self.fixed_print = None;
        }
        self
    }

    /// Set FixedPrint with custom translation.
    pub fn with_fixed_print(mut self, h: f32, v: f32) -> Self {
        self.fixed_print = Some(FixedPrintSettings { h, v });
        self
    }

    /// Set the annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set contents/comment for the annotation.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Get the watermark text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the bounding rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Build the appearance stream content for the watermark.
    fn build_appearance_stream(&self) -> Vec<u8> {
        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Set opacity via graphics state (if not fully opaque)
        if self.opacity < 1.0 {
            stream.push_str("/GS0 gs\n");
        }

        // Calculate center of rect for rotation
        let cx = self.rect.width / 2.0;
        let cy = self.rect.height / 2.0;

        // Apply rotation if needed
        if self.rotation != 0.0 {
            let rad = self.rotation.to_radians();
            let cos_r = rad.cos();
            let sin_r = rad.sin();

            // Translate to center, rotate, translate back
            stream.push_str(&format!("1 0 0 1 {} {} cm\n", cx, cy));
            stream.push_str(&format!(
                "{:.6} {:.6} {:.6} {:.6} 0 0 cm\n",
                cos_r, sin_r, -sin_r, cos_r
            ));
            stream.push_str(&format!("1 0 0 1 {} {} cm\n", -cx, -cy));
        }

        // Begin text
        stream.push_str("BT\n");

        // Set font
        stream.push_str(&format!("/F1 {} Tf\n", self.font_size));

        // Set text color
        stream
            .push_str(&format!("{:.3} {:.3} {:.3} rg\n", self.color.0, self.color.1, self.color.2));

        // Calculate text position (centered in rect)
        // Approximate text width based on font size and character count
        let approx_width = self.text.len() as f32 * self.font_size * 0.5;
        let text_x = (self.rect.width - approx_width) / 2.0;
        let text_y = (self.rect.height - self.font_size) / 2.0;

        stream.push_str(&format!("{:.2} {:.2} Td\n", text_x.max(0.0), text_y.max(0.0)));

        // Show text
        stream.push_str(&format!("({}) Tj\n", escape_pdf_string(&self.text)));

        // End text
        stream.push_str("ET\n");

        // Restore graphics state
        stream.push_str("Q\n");

        stream.into_bytes()
    }

    /// Build the annotation dictionary for PDF output.
    pub fn build(&self, page_ref: ObjectRef) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Watermark".to_string()));

        // Page reference
        dict.insert("P".to_string(), Object::Reference(page_ref));

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

        // Flags
        let flags = self.flags.bits();
        if flags != 0 {
            dict.insert("F".to_string(), Object::Integer(flags as i64));
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // FixedPrint dictionary
        if let Some(ref fp) = self.fixed_print {
            let mut fp_dict = HashMap::new();
            fp_dict.insert("Type".to_string(), Object::Name("FixedPrint".to_string()));
            fp_dict.insert("H".to_string(), Object::Real(fp.h as f64));
            fp_dict.insert("V".to_string(), Object::Real(fp.v as f64));
            dict.insert("FixedPrint".to_string(), Object::Dictionary(fp_dict));
        }

        // Appearance stream
        let ap_stream = self.build_appearance_stream();
        let ap_stream_len = ap_stream.len();

        let mut ap_stream_dict = HashMap::new();
        ap_stream_dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
        ap_stream_dict.insert("Subtype".to_string(), Object::Name("Form".to_string()));
        ap_stream_dict.insert(
            "BBox".to_string(),
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(self.rect.width as f64),
                Object::Real(self.rect.height as f64),
            ]),
        );
        ap_stream_dict.insert("Length".to_string(), Object::Integer(ap_stream_len as i64));

        // Resources for appearance stream
        let mut resources = HashMap::new();

        // Font resource
        let mut font_dict = HashMap::new();
        let mut f1 = HashMap::new();
        f1.insert("Type".to_string(), Object::Name("Font".to_string()));
        f1.insert("Subtype".to_string(), Object::Name("Type1".to_string()));
        f1.insert("BaseFont".to_string(), Object::Name(self.font_name.clone()));
        font_dict.insert("F1".to_string(), Object::Dictionary(f1));
        resources.insert("Font".to_string(), Object::Dictionary(font_dict));

        // ExtGState for opacity
        if self.opacity < 1.0 {
            let mut gs_dict = HashMap::new();
            let mut gs0 = HashMap::new();
            gs0.insert("Type".to_string(), Object::Name("ExtGState".to_string()));
            gs0.insert("CA".to_string(), Object::Real(self.opacity as f64));
            gs0.insert("ca".to_string(), Object::Real(self.opacity as f64));
            gs_dict.insert("GS0".to_string(), Object::Dictionary(gs0));
            resources.insert("ExtGState".to_string(), Object::Dictionary(gs_dict));
        }

        ap_stream_dict.insert("Resources".to_string(), Object::Dictionary(resources));

        // Create appearance dictionary with Normal state
        let mut ap_dict = HashMap::new();
        ap_dict.insert(
            "N".to_string(),
            Object::Stream {
                dict: ap_stream_dict,
                data: crate::import_limits::StreamData::from(ap_stream),
            },
        );

        dict.insert("AP".to_string(), Object::Dictionary(ap_dict));

        dict
    }
}

/// Escape special characters in PDF string.
fn escape_pdf_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '\\' => result.push_str("\\\\"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_new() {
        let wm = WatermarkAnnotation::new("TEST");
        assert_eq!(wm.text(), "TEST");
        assert_eq!(wm.opacity, 0.3);
        assert_eq!(wm.rotation, 0.0);
    }

    #[test]
    fn test_watermark_confidential() {
        let wm = WatermarkAnnotation::confidential();
        assert_eq!(wm.text(), "CONFIDENTIAL");
        assert_eq!(wm.rotation, 45.0);
        assert_eq!(wm.color, (0.8, 0.0, 0.0));
    }

    #[test]
    fn test_watermark_with_settings() {
        let wm = WatermarkAnnotation::new("DRAFT")
            .with_rect(Rect::new(100.0, 200.0, 400.0, 200.0))
            .with_opacity(0.5)
            .with_rotation(30.0)
            .with_font("Courier", 36.0)
            .with_color(0.0, 0.0, 1.0);

        assert_eq!(wm.opacity, 0.5);
        assert_eq!(wm.rotation, 30.0);
        assert_eq!(wm.font_name, "Courier");
        assert_eq!(wm.font_size, 36.0);
        assert_eq!(wm.color, (0.0, 0.0, 1.0));
    }

    #[test]
    fn test_watermark_fixed_print() {
        let wm = WatermarkAnnotation::new("PRINT ONLY").fixed_print(true);
        assert!(wm.fixed_print.is_some());

        let wm2 = WatermarkAnnotation::new("NO PRINT").fixed_print(false);
        assert!(wm2.fixed_print.is_none());
    }

    #[test]
    fn test_watermark_build() {
        let wm = WatermarkAnnotation::new("TEST").with_rect(Rect::new(100.0, 200.0, 300.0, 100.0));

        let page_ref = ObjectRef::new(5, 0);
        let dict = wm.build(page_ref);

        assert!(dict.contains_key("Type"));
        assert!(dict.contains_key("Subtype"));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("AP"));

        if let Some(Object::Name(subtype)) = dict.get("Subtype") {
            assert_eq!(subtype, "Watermark");
        }
    }

    #[test]
    fn test_watermark_with_fixed_print_build() {
        let wm = WatermarkAnnotation::new("PRINT ONLY")
            .with_rect(Rect::new(0.0, 0.0, 612.0, 792.0))
            .with_fixed_print(10.0, 20.0);

        let page_ref = ObjectRef::new(5, 0);
        let dict = wm.build(page_ref);

        assert!(dict.contains_key("FixedPrint"));

        if let Some(Object::Dictionary(fp)) = dict.get("FixedPrint") {
            assert!(fp.contains_key("H"));
            assert!(fp.contains_key("V"));
        }
    }

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("hello"), "hello");
        assert_eq!(escape_pdf_string("test(1)"), "test\\(1\\)");
        assert_eq!(escape_pdf_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_opacity_clamping() {
        let wm1 = WatermarkAnnotation::new("TEST").with_opacity(-0.5);
        assert_eq!(wm1.opacity, 0.0);

        let wm2 = WatermarkAnnotation::new("TEST").with_opacity(1.5);
        assert_eq!(wm2.opacity, 1.0);
    }
}
