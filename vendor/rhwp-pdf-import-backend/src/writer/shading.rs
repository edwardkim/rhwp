//! Shading (gradient) support for PDF generation.
//!
//! This module provides builders for PDF shading patterns including:
//! - Linear gradients (Type 2 - Axial)
//! - Radial gradients (Type 3 - Radial)
//! - Function-based shadings (Type 1)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::shading::{LinearGradientBuilder, GradientStop};
//! use pdf_oxide::layout::Color;
//!
//! let gradient = LinearGradientBuilder::new()
//!     .from(0.0, 0.0)
//!     .to(100.0, 100.0)
//!     .add_stop(0.0, Color::red())
//!     .add_stop(1.0, Color::blue())
//!     .build();
//! ```

use crate::layout::Color;
use crate::object::Object;
use std::collections::HashMap;

/// Helper to create a string key for dictionary
fn key(s: &str) -> String {
    s.to_string()
}

/// A color stop in a gradient.
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// Position along the gradient (0.0 to 1.0)
    pub position: f32,
    /// Color at this position
    pub color: Color,
}

impl GradientStop {
    /// Create a new gradient stop.
    pub fn new(position: f32, color: Color) -> Self {
        Self {
            position: position.clamp(0.0, 1.0),
            color,
        }
    }
}

/// Builder for linear (axial) gradients.
///
/// Creates PDF Type 2 (Axial) shading that interpolates colors
/// along a line between two points.
#[derive(Debug, Clone)]
pub struct LinearGradientBuilder {
    /// Start point (x0, y0)
    start: (f32, f32),
    /// End point (x1, y1)
    end: (f32, f32),
    /// Color stops
    stops: Vec<GradientStop>,
    /// Extend before start point
    extend_start: bool,
    /// Extend after end point
    extend_end: bool,
    /// Color space (default: DeviceRGB)
    color_space: ColorSpace,
}

/// Color spaces for gradients.
#[derive(Debug, Clone, Copy, Default)]
pub enum ColorSpace {
    /// RGB color space
    #[default]
    DeviceRGB,
    /// CMYK color space
    DeviceCMYK,
    /// Grayscale
    DeviceGray,
}

impl ColorSpace {
    /// Get the PDF name for this color space.
    pub fn as_pdf_name(&self) -> &'static [u8] {
        match self {
            ColorSpace::DeviceRGB => b"DeviceRGB",
            ColorSpace::DeviceCMYK => b"DeviceCMYK",
            ColorSpace::DeviceGray => b"DeviceGray",
        }
    }

    /// Get the number of components for this color space.
    pub fn components(&self) -> usize {
        match self {
            ColorSpace::DeviceRGB => 3,
            ColorSpace::DeviceCMYK => 4,
            ColorSpace::DeviceGray => 1,
        }
    }
}

impl Default for LinearGradientBuilder {
    fn default() -> Self {
        Self {
            start: (0.0, 0.0),
            end: (100.0, 0.0),
            stops: Vec::new(),
            extend_start: true,
            extend_end: true,
            color_space: ColorSpace::DeviceRGB,
        }
    }
}

impl LinearGradientBuilder {
    /// Create a new linear gradient builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the start point of the gradient.
    pub fn from(mut self, x: f32, y: f32) -> Self {
        self.start = (x, y);
        self
    }

    /// Set the end point of the gradient.
    pub fn to(mut self, x: f32, y: f32) -> Self {
        self.end = (x, y);
        self
    }

    /// Add a color stop.
    pub fn add_stop(mut self, position: f32, color: Color) -> Self {
        self.stops.push(GradientStop::new(position, color));
        self
    }

    /// Set whether to extend the gradient before the start point.
    pub fn extend_start(mut self, extend: bool) -> Self {
        self.extend_start = extend;
        self
    }

    /// Set whether to extend the gradient after the end point.
    pub fn extend_end(mut self, extend: bool) -> Self {
        self.extend_end = extend;
        self
    }

    /// Set both extend flags.
    pub fn extend(self, extend: bool) -> Self {
        self.extend_start(extend).extend_end(extend)
    }

    /// Build a simple two-color gradient.
    pub fn two_color(start_color: Color, end_color: Color) -> Self {
        Self::new()
            .add_stop(0.0, start_color)
            .add_stop(1.0, end_color)
    }

    /// Build the shading dictionary as a PDF Object.
    ///
    /// Returns a tuple of (shading_dict, function_dict) where function_dict
    /// may be None for simple two-color gradients.
    pub fn build(&self) -> (Object, Option<Object>) {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // ShadingType 2 = Axial (linear gradient)
        dict.insert(key("ShadingType"), Object::Integer(2));

        // Color space
        dict.insert(
            key("ColorSpace"),
            Object::Name(String::from_utf8_lossy(self.color_space.as_pdf_name()).to_string()),
        );

        // Coords [x0 y0 x1 y1]
        dict.insert(
            key("Coords"),
            Object::Array(vec![
                Object::Real(self.start.0 as f64),
                Object::Real(self.start.1 as f64),
                Object::Real(self.end.0 as f64),
                Object::Real(self.end.1 as f64),
            ]),
        );

        // Extend [before after]
        dict.insert(
            key("Extend"),
            Object::Array(vec![
                Object::Boolean(self.extend_start),
                Object::Boolean(self.extend_end),
            ]),
        );

        // Build the function
        let (function, extra_functions) = self.build_function();
        dict.insert(key("Function"), function);

        (Object::Dictionary(dict), extra_functions)
    }

    /// Build the interpolation function for the gradient.
    fn build_function(&self) -> (Object, Option<Object>) {
        // Sort stops by position
        let mut stops = self.stops.clone();
        stops.sort_by(|a, b| crate::utils::safe_float_cmp(a.position, b.position));

        // If no stops, use black to white
        if stops.is_empty() {
            stops = vec![
                GradientStop::new(0.0, Color::black()),
                GradientStop::new(1.0, Color::white()),
            ];
        }

        // If only one stop, duplicate it
        if stops.len() == 1 {
            let color = stops[0].color;
            stops = vec![GradientStop::new(0.0, color), GradientStop::new(1.0, color)];
        }

        // Simple two-color gradient uses Type 2 (exponential interpolation)
        if stops.len() == 2 && stops[0].position == 0.0 && stops[1].position == 1.0 {
            return (self.build_type2_function(&stops[0].color, &stops[1].color), None);
        }

        // Multi-stop gradient uses Type 3 (stitching function)
        self.build_type3_function(&stops)
    }

    /// Build a Type 2 (exponential interpolation) function.
    fn build_type2_function(&self, c0: &Color, c1: &Color) -> Object {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // FunctionType 2 = Exponential interpolation
        dict.insert(key("FunctionType"), Object::Integer(2));

        // Domain [0 1]
        dict.insert(key("Domain"), Object::Array(vec![Object::Real(0.0), Object::Real(1.0)]));

        // C0 = start color
        dict.insert(
            key("C0"),
            Object::Array(vec![
                Object::Real(c0.r as f64),
                Object::Real(c0.g as f64),
                Object::Real(c0.b as f64),
            ]),
        );

        // C1 = end color
        dict.insert(
            key("C1"),
            Object::Array(vec![
                Object::Real(c1.r as f64),
                Object::Real(c1.g as f64),
                Object::Real(c1.b as f64),
            ]),
        );

        // N = exponent (1.0 for linear)
        dict.insert(key("N"), Object::Real(1.0));

        Object::Dictionary(dict)
    }

    /// Build a Type 3 (stitching) function for multi-stop gradients.
    fn build_type3_function(&self, stops: &[GradientStop]) -> (Object, Option<Object>) {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // FunctionType 3 = Stitching
        dict.insert(key("FunctionType"), Object::Integer(3));

        // Domain [0 1]
        dict.insert(key("Domain"), Object::Array(vec![Object::Real(0.0), Object::Real(1.0)]));

        // Build sub-functions for each segment
        let mut functions = Vec::new();
        let mut bounds = Vec::new();
        let mut encode = Vec::new();

        for i in 0..stops.len() - 1 {
            let c0 = &stops[i].color;
            let c1 = &stops[i + 1].color;
            functions.push(self.build_type2_function(c0, c1));

            if i < stops.len() - 2 {
                bounds.push(Object::Real(stops[i + 1].position as f64));
            }

            encode.push(Object::Real(0.0));
            encode.push(Object::Real(1.0));
        }

        dict.insert(key("Functions"), Object::Array(functions));
        dict.insert(key("Bounds"), Object::Array(bounds));
        dict.insert(key("Encode"), Object::Array(encode));

        (Object::Dictionary(dict), None)
    }
}

/// Builder for radial gradients.
///
/// Creates PDF Type 3 (Radial) shading that interpolates colors
/// between two circles.
#[derive(Debug, Clone)]
pub struct RadialGradientBuilder {
    /// Inner circle center and radius (x0, y0, r0)
    inner: (f32, f32, f32),
    /// Outer circle center and radius (x1, y1, r1)
    outer: (f32, f32, f32),
    /// Color stops
    stops: Vec<GradientStop>,
    /// Extend before start
    extend_start: bool,
    /// Extend after end
    extend_end: bool,
}

impl Default for RadialGradientBuilder {
    fn default() -> Self {
        Self {
            inner: (50.0, 50.0, 0.0),
            outer: (50.0, 50.0, 50.0),
            stops: Vec::new(),
            extend_start: true,
            extend_end: true,
        }
    }
}

impl RadialGradientBuilder {
    /// Create a new radial gradient builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the inner circle (center and radius).
    pub fn inner_circle(mut self, cx: f32, cy: f32, radius: f32) -> Self {
        self.inner = (cx, cy, radius);
        self
    }

    /// Set the outer circle (center and radius).
    pub fn outer_circle(mut self, cx: f32, cy: f32, radius: f32) -> Self {
        self.outer = (cx, cy, radius);
        self
    }

    /// Create a centered radial gradient.
    pub fn centered(cx: f32, cy: f32, radius: f32) -> Self {
        Self::new()
            .inner_circle(cx, cy, 0.0)
            .outer_circle(cx, cy, radius)
    }

    /// Add a color stop.
    pub fn add_stop(mut self, position: f32, color: Color) -> Self {
        self.stops.push(GradientStop::new(position, color));
        self
    }

    /// Set extend flags.
    pub fn extend(mut self, start: bool, end: bool) -> Self {
        self.extend_start = start;
        self.extend_end = end;
        self
    }

    /// Build the shading dictionary.
    pub fn build(&self) -> Object {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // ShadingType 3 = Radial
        dict.insert(key("ShadingType"), Object::Integer(3));

        // Color space
        dict.insert(key("ColorSpace"), Object::Name("DeviceRGB".to_string()));

        // Coords [x0 y0 r0 x1 y1 r1]
        dict.insert(
            key("Coords"),
            Object::Array(vec![
                Object::Real(self.inner.0 as f64),
                Object::Real(self.inner.1 as f64),
                Object::Real(self.inner.2 as f64),
                Object::Real(self.outer.0 as f64),
                Object::Real(self.outer.1 as f64),
                Object::Real(self.outer.2 as f64),
            ]),
        );

        // Extend
        dict.insert(
            key("Extend"),
            Object::Array(vec![
                Object::Boolean(self.extend_start),
                Object::Boolean(self.extend_end),
            ]),
        );

        // Build function (reuse linear gradient logic)
        let linear = LinearGradientBuilder {
            stops: self.stops.clone(),
            ..Default::default()
        };
        let (function, _) = linear.build_function();
        dict.insert(key("Function"), function);

        Object::Dictionary(dict)
    }
}

/// Predefined gradient presets.
pub struct GradientPresets;

impl GradientPresets {
    /// Black to white horizontal gradient.
    pub fn grayscale() -> LinearGradientBuilder {
        LinearGradientBuilder::two_color(Color::black(), Color::white())
    }

    /// Red to blue gradient.
    pub fn red_to_blue() -> LinearGradientBuilder {
        LinearGradientBuilder::two_color(
            Color::new(1.0, 0.0, 0.0), // Red
            Color::new(0.0, 0.0, 1.0), // Blue
        )
    }

    /// Rainbow gradient.
    pub fn rainbow() -> LinearGradientBuilder {
        LinearGradientBuilder::new()
            .add_stop(0.0, Color::new(1.0, 0.0, 0.0)) // Red
            .add_stop(0.17, Color::new(1.0, 0.5, 0.0)) // Orange
            .add_stop(0.33, Color::new(1.0, 1.0, 0.0)) // Yellow
            .add_stop(0.5, Color::new(0.0, 1.0, 0.0)) // Green
            .add_stop(0.67, Color::new(0.0, 0.0, 1.0)) // Blue
            .add_stop(0.83, Color::new(0.29, 0.0, 0.51)) // Indigo
            .add_stop(1.0, Color::new(0.56, 0.0, 1.0)) // Violet
    }

    /// Sunset gradient (orange to purple).
    pub fn sunset() -> LinearGradientBuilder {
        LinearGradientBuilder::new()
            .add_stop(
                0.0,
                Color {
                    r: 1.0,
                    g: 0.6,
                    b: 0.0,
                },
            )
            .add_stop(
                0.5,
                Color {
                    r: 1.0,
                    g: 0.3,
                    b: 0.3,
                },
            )
            .add_stop(
                1.0,
                Color {
                    r: 0.5,
                    g: 0.0,
                    b: 0.5,
                },
            )
    }

    /// Ocean gradient (light blue to dark blue).
    pub fn ocean() -> LinearGradientBuilder {
        LinearGradientBuilder::new()
            .add_stop(
                0.0,
                Color {
                    r: 0.5,
                    g: 0.8,
                    b: 1.0,
                },
            )
            .add_stop(
                1.0,
                Color {
                    r: 0.0,
                    g: 0.2,
                    b: 0.5,
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_gradient_two_color() {
        let gradient = LinearGradientBuilder::new()
            .from(0.0, 0.0)
            .to(100.0, 0.0)
            .add_stop(0.0, Color::new(1.0, 0.0, 0.0))
            .add_stop(1.0, Color::new(0.0, 0.0, 1.0))
            .build();

        if let (Object::Dictionary(dict), _) = gradient {
            assert!(dict.contains_key("ShadingType"));
            assert!(dict.contains_key("Coords"));
            assert!(dict.contains_key("Function"));
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_radial_gradient() {
        let gradient = RadialGradientBuilder::centered(50.0, 50.0, 50.0)
            .add_stop(0.0, Color::white())
            .add_stop(1.0, Color::black())
            .build();

        if let Object::Dictionary(dict) = gradient {
            assert!(dict.contains_key("ShadingType"));
            if let Some(Object::Integer(st)) = dict.get("ShadingType") {
                assert_eq!(*st, 3); // Radial shading
            }
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_gradient_presets() {
        let _ = GradientPresets::grayscale().build();
        let _ = GradientPresets::rainbow().build();
        let _ = GradientPresets::sunset().build();
    }
}
