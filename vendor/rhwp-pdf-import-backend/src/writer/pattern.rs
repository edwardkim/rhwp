//! Pattern support for PDF generation.
//!
//! This module provides builders for PDF pattern resources:
//! - Tiling patterns (Type 1) - repeating content
//! - Shading patterns (Type 2) - gradient-based patterns
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::pattern::{TilingPatternBuilder, PatternPaintType};
//! use pdf_oxide::layout::Color;
//!
//! // Create a striped pattern
//! let pattern = TilingPatternBuilder::new()
//!     .bbox(0.0, 0.0, 10.0, 10.0)
//!     .x_step(10.0)
//!     .y_step(10.0)
//!     .colored()
//!     .content(|builder| {
//!         builder
//!             .set_fill_color(1.0, 0.0, 0.0)
//!             .rect(0.0, 0.0, 5.0, 10.0)
//!             .fill();
//!     })
//!     .build();
//! ```

use crate::layout::Color;
use crate::object::Object;
use std::collections::HashMap;

/// Helper to create a string key for dictionary
fn key(s: &str) -> String {
    s.to_string()
}

/// Pattern paint type.
#[derive(Debug, Clone, Copy, Default)]
pub enum PatternPaintType {
    /// Colored pattern - colors specified in pattern content
    #[default]
    Colored = 1,
    /// Uncolored pattern - color specified when pattern is used
    Uncolored = 2,
}

/// Pattern tiling type.
#[derive(Debug, Clone, Copy, Default)]
pub enum PatternTilingType {
    /// Constant spacing - pattern cell spacing is constant
    #[default]
    ConstantSpacing = 1,
    /// No distortion - cell is adjusted to device pixels without distortion
    NoDistortion = 2,
    /// Constant spacing and faster tiling
    ConstantSpacingFaster = 3,
}

/// Builder for tiling patterns (Type 1).
///
/// Tiling patterns paint a cell that is replicated at fixed intervals
/// to fill the area to be painted.
#[derive(Debug, Clone)]
pub struct TilingPatternBuilder {
    /// Bounding box of the pattern cell
    bbox: (f32, f32, f32, f32),
    /// Horizontal spacing between pattern cells
    x_step: f32,
    /// Vertical spacing between pattern cells
    y_step: f32,
    /// Paint type
    paint_type: PatternPaintType,
    /// Tiling type
    tiling_type: PatternTilingType,
    /// Pattern content stream
    content: Vec<u8>,
    /// Pattern matrix (optional transformation)
    matrix: Option<[f32; 6]>,
    /// Resources needed by the pattern
    resources: HashMap<Vec<u8>, Object>,
}

impl Default for TilingPatternBuilder {
    fn default() -> Self {
        Self {
            bbox: (0.0, 0.0, 10.0, 10.0),
            x_step: 10.0,
            y_step: 10.0,
            paint_type: PatternPaintType::Colored,
            tiling_type: PatternTilingType::ConstantSpacing,
            content: Vec::new(),
            matrix: None,
            resources: HashMap::new(),
        }
    }
}

impl TilingPatternBuilder {
    /// Create a new tiling pattern builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bounding box of the pattern cell.
    pub fn bbox(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.bbox = (x, y, width, height);
        self
    }

    /// Set the horizontal step (spacing).
    pub fn x_step(mut self, step: f32) -> Self {
        self.x_step = step;
        self
    }

    /// Set the vertical step (spacing).
    pub fn y_step(mut self, step: f32) -> Self {
        self.y_step = step;
        self
    }

    /// Set both steps at once.
    pub fn step(self, x: f32, y: f32) -> Self {
        self.x_step(x).y_step(y)
    }

    /// Set as colored pattern (colors in pattern content).
    pub fn colored(mut self) -> Self {
        self.paint_type = PatternPaintType::Colored;
        self
    }

    /// Set as uncolored pattern (color specified at use time).
    pub fn uncolored(mut self) -> Self {
        self.paint_type = PatternPaintType::Uncolored;
        self
    }

    /// Set the tiling type.
    pub fn tiling_type(mut self, tiling: PatternTilingType) -> Self {
        self.tiling_type = tiling;
        self
    }

    /// Set the pattern transformation matrix.
    pub fn matrix(mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        self.matrix = Some([a, b, c, d, e, f]);
        self
    }

    /// Set the raw content stream.
    pub fn content_bytes(mut self, content: Vec<u8>) -> Self {
        self.content = content;
        self
    }

    /// Build the pattern dictionary and content stream.
    ///
    /// Returns (dictionary, content_stream_bytes).
    pub fn build(&self) -> (Object, Vec<u8>) {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // Type is always Pattern
        dict.insert(key("Type"), Object::Name("Pattern".to_string()));

        // PatternType 1 = Tiling
        dict.insert(key("PatternType"), Object::Integer(1));

        // PaintType
        dict.insert(key("PaintType"), Object::Integer(self.paint_type as i64));

        // TilingType
        dict.insert(key("TilingType"), Object::Integer(self.tiling_type as i64));

        // BBox
        dict.insert(
            key("BBox"),
            Object::Array(vec![
                Object::Real(self.bbox.0 as f64),
                Object::Real(self.bbox.1 as f64),
                Object::Real(self.bbox.2 as f64),
                Object::Real(self.bbox.3 as f64),
            ]),
        );

        // XStep and YStep
        dict.insert(key("XStep"), Object::Real(self.x_step as f64));
        dict.insert(key("YStep"), Object::Real(self.y_step as f64));

        // Matrix (optional)
        if let Some(m) = &self.matrix {
            dict.insert(
                key("Matrix"),
                Object::Array(m.iter().map(|&v| Object::Real(v as f64)).collect()),
            );
        }

        // Resources (if needed) - convert keys
        if !self.resources.is_empty() {
            let converted: HashMap<String, Object> = self
                .resources
                .iter()
                .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), v.clone()))
                .collect();
            dict.insert(key("Resources"), Object::Dictionary(converted));
        }

        (Object::Dictionary(dict), self.content.clone())
    }
}

/// Builder for shading patterns (Type 2).
///
/// Shading patterns use a shading dictionary to define a gradient fill.
#[derive(Debug, Clone, Default)]
pub struct ShadingPatternBuilder {
    /// Reference to shading dictionary (will be indirect reference)
    shading_id: Option<u32>,
    /// Pattern transformation matrix
    matrix: Option<[f32; 6]>,
    /// ExtGState for the pattern
    ext_gstate_id: Option<u32>,
}

impl ShadingPatternBuilder {
    /// Create a new shading pattern builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the shading object ID (will be referenced indirectly).
    pub fn shading_id(mut self, id: u32) -> Self {
        self.shading_id = Some(id);
        self
    }

    /// Set the pattern transformation matrix.
    pub fn matrix(mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        self.matrix = Some([a, b, c, d, e, f]);
        self
    }

    /// Set the ExtGState object ID.
    pub fn ext_gstate_id(mut self, id: u32) -> Self {
        self.ext_gstate_id = Some(id);
        self
    }

    /// Build the pattern dictionary.
    pub fn build(&self) -> Object {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // Type is always Pattern
        dict.insert(key("Type"), Object::Name("Pattern".to_string()));

        // PatternType 2 = Shading
        dict.insert(key("PatternType"), Object::Integer(2));

        // Shading (indirect reference) - caller must set this
        // We'll put a placeholder that the caller should replace
        if let Some(id) = self.shading_id {
            dict.insert(key("Shading"), Object::Reference(crate::object::ObjectRef::new(id, 0)));
        }

        // Matrix (optional)
        if let Some(m) = &self.matrix {
            dict.insert(
                key("Matrix"),
                Object::Array(m.iter().map(|&v| Object::Real(v as f64)).collect()),
            );
        }

        // ExtGState (optional)
        if let Some(id) = self.ext_gstate_id {
            dict.insert(key("ExtGState"), Object::Reference(crate::object::ObjectRef::new(id, 0)));
        }

        Object::Dictionary(dict)
    }
}

/// Predefined pattern presets.
pub struct PatternPresets;

impl PatternPresets {
    /// Create horizontal stripes pattern content.
    pub fn horizontal_stripes(
        width: f32,
        _height: f32,
        stripe_height: f32,
        color: Color,
    ) -> Vec<u8> {
        format!(
            "{} {} {} rg\n0 0 {} {} re\nf\n",
            color.r, color.g, color.b, width, stripe_height
        )
        .into_bytes()
    }

    /// Create vertical stripes pattern content.
    pub fn vertical_stripes(_width: f32, height: f32, stripe_width: f32, color: Color) -> Vec<u8> {
        format!(
            "{} {} {} rg\n0 0 {} {} re\nf\n",
            color.r, color.g, color.b, stripe_width, height
        )
        .into_bytes()
    }

    /// Create a checkerboard pattern content.
    pub fn checkerboard(size: f32, color1: Color, color2: Color) -> Vec<u8> {
        format!(
            "{} {} {} rg\n0 0 {} {} re\nf\n{} {} {} rg\n{} 0 {} {} re\n0 {} {} {} re\nf\n",
            color1.r,
            color1.g,
            color1.b,
            size * 2.0,
            size * 2.0,
            color2.r,
            color2.g,
            color2.b,
            size,
            size,
            size,
            size,
            size,
            size
        )
        .into_bytes()
    }

    /// Create a dot pattern content.
    pub fn dots(spacing: f32, radius: f32, color: Color) -> Vec<u8> {
        // Approximate circle with Bézier curves
        let k = radius * 0.552_284_8;
        let cx = spacing / 2.0;
        let cy = spacing / 2.0;

        format!(
            "{} {} {} rg\n\
             {} {} m\n\
             {} {} {} {} {} {} c\n\
             {} {} {} {} {} {} c\n\
             {} {} {} {} {} {} c\n\
             {} {} {} {} {} {} c\n\
             f\n",
            color.r,
            color.g,
            color.b,
            cx + radius,
            cy,
            cx + radius,
            cy + k,
            cx + k,
            cy + radius,
            cx,
            cy + radius,
            cx - k,
            cy + radius,
            cx - radius,
            cy + k,
            cx - radius,
            cy,
            cx - radius,
            cy - k,
            cx - k,
            cy - radius,
            cx,
            cy - radius,
            cx + k,
            cy - radius,
            cx + radius,
            cy - k,
            cx + radius,
            cy
        )
        .into_bytes()
    }

    /// Create diagonal lines pattern content.
    pub fn diagonal_lines(size: f32, line_width: f32, color: Color) -> Vec<u8> {
        format!(
            "{} {} {} RG\n{} w\n0 0 m\n{} {} l\nS\n",
            color.r, color.g, color.b, line_width, size, size
        )
        .into_bytes()
    }

    /// Create a crosshatch pattern content.
    pub fn crosshatch(size: f32, line_width: f32, color: Color) -> Vec<u8> {
        format!(
            "{} {} {} RG\n{} w\n\
             0 0 m\n{} {} l\nS\n\
             {} 0 m\n0 {} l\nS\n",
            color.r, color.g, color.b, line_width, size, size, size, size
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiling_pattern_builder() {
        let content =
            PatternPresets::horizontal_stripes(10.0, 10.0, 5.0, Color::new(1.0, 0.0, 0.0));
        let (dict, _content) = TilingPatternBuilder::new()
            .bbox(0.0, 0.0, 10.0, 10.0)
            .step(10.0, 10.0)
            .colored()
            .content_bytes(content)
            .build();

        if let Object::Dictionary(d) = dict {
            assert!(d.contains_key("PatternType"));
            if let Some(Object::Integer(pt)) = d.get("PatternType") {
                assert_eq!(*pt, 1);
            }
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_shading_pattern_builder() {
        let dict = ShadingPatternBuilder::new()
            .shading_id(5)
            .matrix(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
            .build();

        if let Object::Dictionary(d) = dict {
            assert!(d.contains_key("PatternType"));
            if let Some(Object::Integer(pt)) = d.get("PatternType") {
                assert_eq!(*pt, 2);
            }
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_pattern_presets() {
        let _ = PatternPresets::horizontal_stripes(10.0, 10.0, 5.0, Color::new(0.0, 0.0, 1.0));
        let _ = PatternPresets::checkerboard(5.0, Color::white(), Color::black());
        let _ = PatternPresets::dots(10.0, 2.0, Color::new(1.0, 0.0, 0.0));
        let _ = PatternPresets::diagonal_lines(10.0, 0.5, Color::black());
    }
}
