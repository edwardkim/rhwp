//! Extended Graphics State (ExtGState) for transparency and blend modes.
//!
//! This module provides builders for PDF graphics state resources including:
//! - Transparency (fill and stroke alpha)
//! - Blend modes
//! - Soft masks
//! - Other graphics state parameters
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::graphics_state::ExtGStateBuilder;
//!
//! let gs = ExtGStateBuilder::new()
//!     .fill_alpha(0.5)
//!     .stroke_alpha(0.8)
//!     .blend_mode(BlendMode::Multiply)
//!     .build();
//! ```

use super::content_stream::BlendMode;
use crate::object::Object;
use std::collections::HashMap;

/// Helper to create a string key for dictionary
fn key(s: &str) -> String {
    s.to_string()
}

/// Builder for Extended Graphics State dictionaries.
///
/// ExtGState is used to set transparency, blend modes, and other
/// graphics state parameters that can't be set with simple operators.
#[derive(Debug, Clone, Default)]
pub struct ExtGStateBuilder {
    /// Fill alpha (ca) - 0.0 to 1.0
    fill_alpha: Option<f32>,
    /// Stroke alpha (CA) - 0.0 to 1.0
    stroke_alpha: Option<f32>,
    /// Blend mode (BM)
    blend_mode: Option<BlendMode>,
    /// Overprint mode for stroke (OP)
    overprint_stroke: Option<bool>,
    /// Overprint mode for fill (op)
    overprint_fill: Option<bool>,
    /// Overprint mode (OPM)
    overprint_mode: Option<i32>,
    /// Line width (LW)
    line_width: Option<f32>,
    /// Line cap style (LC)
    line_cap: Option<i32>,
    /// Line join style (LJ)
    line_join: Option<i32>,
    /// Miter limit (ML)
    miter_limit: Option<f32>,
    /// Dash pattern (D)
    dash_pattern: Option<(Vec<f32>, f32)>,
    /// Flatness tolerance (FL)
    flatness: Option<f32>,
    /// Smoothness tolerance (SM)
    smoothness: Option<f32>,
    /// Alpha source flag (AIS)
    alpha_source: Option<bool>,
    /// Text knockout flag (TK)
    text_knockout: Option<bool>,
    /// Soft mask (SMask) - reference to soft mask dictionary
    soft_mask: Option<SoftMask>,
}

/// Soft mask configuration for transparency effects.
#[derive(Debug, Clone)]
pub enum SoftMask {
    /// No soft mask (None)
    None,
    /// Soft mask from transparency group
    Group {
        /// Transparency group XObject reference
        group_ref: String,
        /// Subtype (Alpha or Luminosity)
        subtype: SoftMaskSubtype,
        /// Backdrop color (optional)
        backdrop: Option<Vec<f32>>,
        /// Transfer function (optional)
        transfer: Option<String>,
    },
}

/// Soft mask subtype.
#[derive(Debug, Clone, Copy)]
pub enum SoftMaskSubtype {
    /// Use alpha values from the group
    Alpha,
    /// Use luminosity values from the group
    Luminosity,
}

impl ExtGStateBuilder {
    /// Create a new ExtGState builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set fill alpha (opacity for fill operations).
    ///
    /// Value should be between 0.0 (fully transparent) and 1.0 (fully opaque).
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.fill_alpha = Some(alpha.clamp(0.0, 1.0));
        self
    }

    /// Set stroke alpha (opacity for stroke operations).
    pub fn stroke_alpha(mut self, alpha: f32) -> Self {
        self.stroke_alpha = Some(alpha.clamp(0.0, 1.0));
        self
    }

    /// Set both fill and stroke alpha to the same value.
    pub fn alpha(self, alpha: f32) -> Self {
        self.fill_alpha(alpha).stroke_alpha(alpha)
    }

    /// Set blend mode.
    pub fn blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = Some(mode);
        self
    }

    /// Set overprint for stroke operations.
    pub fn overprint_stroke(mut self, enabled: bool) -> Self {
        self.overprint_stroke = Some(enabled);
        self
    }

    /// Set overprint for fill operations.
    pub fn overprint_fill(mut self, enabled: bool) -> Self {
        self.overprint_fill = Some(enabled);
        self
    }

    /// Set line width in graphics state.
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = Some(width);
        self
    }

    /// Set flatness tolerance.
    pub fn flatness(mut self, flatness: f32) -> Self {
        self.flatness = Some(flatness);
        self
    }

    /// Set soft mask.
    pub fn soft_mask(mut self, mask: SoftMask) -> Self {
        self.soft_mask = Some(mask);
        self
    }

    /// Remove soft mask (set to None).
    pub fn no_soft_mask(mut self) -> Self {
        self.soft_mask = Some(SoftMask::None);
        self
    }

    /// Build the ExtGState dictionary as a PDF Object.
    pub fn build(&self) -> Object {
        let mut dict: HashMap<String, Object> = HashMap::new();

        // Type is always ExtGState
        dict.insert(key("Type"), Object::Name("ExtGState".to_string()));

        if let Some(alpha) = self.fill_alpha {
            dict.insert(key("ca"), Object::Real(alpha as f64));
        }

        if let Some(alpha) = self.stroke_alpha {
            dict.insert(key("CA"), Object::Real(alpha as f64));
        }

        if let Some(ref mode) = self.blend_mode {
            dict.insert(key("BM"), Object::Name(mode.as_pdf_name().to_string()));
        }

        if let Some(op) = self.overprint_stroke {
            dict.insert(key("OP"), Object::Boolean(op));
        }

        if let Some(op) = self.overprint_fill {
            dict.insert(key("op"), Object::Boolean(op));
        }

        if let Some(opm) = self.overprint_mode {
            dict.insert(key("OPM"), Object::Integer(opm as i64));
        }

        if let Some(lw) = self.line_width {
            dict.insert(key("LW"), Object::Real(lw as f64));
        }

        if let Some(lc) = self.line_cap {
            dict.insert(key("LC"), Object::Integer(lc as i64));
        }

        if let Some(lj) = self.line_join {
            dict.insert(key("LJ"), Object::Integer(lj as i64));
        }

        if let Some(ml) = self.miter_limit {
            dict.insert(key("ML"), Object::Real(ml as f64));
        }

        if let Some((ref pattern, phase)) = self.dash_pattern {
            let arr = vec![
                Object::Array(pattern.iter().map(|&v| Object::Real(v as f64)).collect()),
                Object::Real(phase as f64),
            ];
            dict.insert(key("D"), Object::Array(arr));
        }

        if let Some(fl) = self.flatness {
            dict.insert(key("FL"), Object::Real(fl as f64));
        }

        if let Some(sm) = self.smoothness {
            dict.insert(key("SM"), Object::Real(sm as f64));
        }

        if let Some(ais) = self.alpha_source {
            dict.insert(key("AIS"), Object::Boolean(ais));
        }

        if let Some(tk) = self.text_knockout {
            dict.insert(key("TK"), Object::Boolean(tk));
        }

        if let Some(ref mask) = self.soft_mask {
            match mask {
                SoftMask::None => {
                    dict.insert(key("SMask"), Object::Name("None".to_string()));
                },
                SoftMask::Group {
                    group_ref: _,
                    subtype,
                    backdrop,
                    transfer: _,
                } => {
                    let mut smask_dict: HashMap<String, Object> = HashMap::new();
                    smask_dict.insert(key("Type"), Object::Name("Mask".to_string()));
                    smask_dict.insert(
                        key("S"),
                        Object::Name(match subtype {
                            SoftMaskSubtype::Alpha => "Alpha".to_string(),
                            SoftMaskSubtype::Luminosity => "Luminosity".to_string(),
                        }),
                    );
                    // G (group reference) would need to be an indirect reference
                    // For now, we'll use a placeholder - the caller should set this up
                    if let Some(ref bc) = backdrop {
                        smask_dict.insert(
                            key("BC"),
                            Object::Array(bc.iter().map(|&v| Object::Real(v as f64)).collect()),
                        );
                    }
                    dict.insert(key("SMask"), Object::Dictionary(smask_dict));
                },
            }
        }

        Object::Dictionary(dict)
    }
}

/// Predefined transparency effects.
impl ExtGStateBuilder {
    /// Create a semi-transparent state (50% opacity).
    pub fn semi_transparent() -> Self {
        Self::new().alpha(0.5)
    }

    /// Create a multiply blend effect.
    pub fn multiply() -> Self {
        Self::new().blend_mode(BlendMode::Multiply)
    }

    /// Create a screen blend effect.
    pub fn screen() -> Self {
        Self::new().blend_mode(BlendMode::Screen)
    }

    /// Create an overlay blend effect.
    pub fn overlay() -> Self {
        Self::new().blend_mode(BlendMode::Overlay)
    }

    /// Create a difference blend effect.
    pub fn difference() -> Self {
        Self::new().blend_mode(BlendMode::Difference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext_gstate_builder_alpha() {
        let gs = ExtGStateBuilder::new()
            .fill_alpha(0.5)
            .stroke_alpha(0.8)
            .build();

        if let Object::Dictionary(dict) = gs {
            assert!(dict.contains_key("ca"));
            assert!(dict.contains_key("CA"));
            assert!(dict.contains_key("Type"));
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_ext_gstate_builder_blend_mode() {
        let gs = ExtGStateBuilder::new()
            .blend_mode(BlendMode::Multiply)
            .build();

        if let Object::Dictionary(dict) = gs {
            assert!(dict.contains_key("BM"));
            if let Some(Object::Name(name)) = dict.get("BM") {
                assert_eq!(name, "Multiply");
            }
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_predefined_effects() {
        let gs = ExtGStateBuilder::semi_transparent().build();
        if let Object::Dictionary(dict) = gs {
            assert!(dict.contains_key("ca"));
            assert!(dict.contains_key("CA"));
        }

        let gs = ExtGStateBuilder::multiply().build();
        if let Object::Dictionary(dict) = gs {
            assert!(dict.contains_key("BM"));
        }
    }

    // ---- Tests for alpha clamping ----

    #[test]
    fn test_fill_alpha_clamped_above() {
        let gs = ExtGStateBuilder::new().fill_alpha(2.0).build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Real(val)) = dict.get("ca") {
                assert!((*val - 1.0).abs() < f64::EPSILON, "fill_alpha should be clamped to 1.0");
            } else {
                panic!("Expected Real for ca");
            }
        }
    }

    #[test]
    fn test_fill_alpha_clamped_below() {
        let gs = ExtGStateBuilder::new().fill_alpha(-0.5).build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Real(val)) = dict.get("ca") {
                assert!((*val).abs() < f64::EPSILON, "fill_alpha should be clamped to 0.0");
            } else {
                panic!("Expected Real for ca");
            }
        }
    }

    #[test]
    fn test_stroke_alpha_clamped_above() {
        let gs = ExtGStateBuilder::new().stroke_alpha(1.5).build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Real(val)) = dict.get("CA") {
                assert!((*val - 1.0).abs() < f64::EPSILON, "stroke_alpha should be clamped to 1.0");
            } else {
                panic!("Expected Real for CA");
            }
        }
    }

    // ---- Tests for alpha() convenience method ----

    #[test]
    fn test_alpha_sets_both() {
        let gs = ExtGStateBuilder::new().alpha(0.3).build();
        if let Object::Dictionary(dict) = gs {
            assert!(dict.contains_key("ca"));
            assert!(dict.contains_key("CA"));
            if let (Some(Object::Real(fill)), Some(Object::Real(stroke))) =
                (dict.get("ca"), dict.get("CA"))
            {
                assert!((*fill - 0.3).abs() < 0.01);
                assert!((*stroke - 0.3).abs() < 0.01);
            }
        }
    }

    // ---- Tests for overprint settings ----

    #[test]
    fn test_overprint_stroke() {
        let gs = ExtGStateBuilder::new().overprint_stroke(true).build();
        if let Object::Dictionary(dict) = gs {
            assert_eq!(dict.get("OP"), Some(&Object::Boolean(true)));
        }
    }

    #[test]
    fn test_overprint_fill() {
        let gs = ExtGStateBuilder::new().overprint_fill(false).build();
        if let Object::Dictionary(dict) = gs {
            assert_eq!(dict.get("op"), Some(&Object::Boolean(false)));
        }
    }

    // ---- Tests for line width and flatness ----

    #[test]
    fn test_line_width() {
        let gs = ExtGStateBuilder::new().line_width(2.5).build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Real(val)) = dict.get("LW") {
                assert!((*val - 2.5).abs() < 0.01);
            } else {
                panic!("Expected Real for LW");
            }
        }
    }

    #[test]
    fn test_flatness() {
        let gs = ExtGStateBuilder::new().flatness(50.0).build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Real(val)) = dict.get("FL") {
                assert!((*val - 50.0).abs() < 0.01);
            } else {
                panic!("Expected Real for FL");
            }
        }
    }

    // ---- Tests for soft mask ----

    #[test]
    fn test_no_soft_mask() {
        let gs = ExtGStateBuilder::new().no_soft_mask().build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Name(name)) = dict.get("SMask") {
                assert_eq!(name, "None");
            } else {
                panic!("Expected Name(\"None\") for SMask");
            }
        }
    }

    #[test]
    fn test_soft_mask_group_alpha() {
        let gs = ExtGStateBuilder::new()
            .soft_mask(SoftMask::Group {
                group_ref: "G1".to_string(),
                subtype: SoftMaskSubtype::Alpha,
                backdrop: None,
                transfer: None,
            })
            .build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Dictionary(smask_dict)) = dict.get("SMask") {
                assert_eq!(smask_dict.get("S"), Some(&Object::Name("Alpha".to_string())));
                assert_eq!(smask_dict.get("Type"), Some(&Object::Name("Mask".to_string())));
                assert!(!smask_dict.contains_key("BC"));
            } else {
                panic!("Expected Dictionary for SMask");
            }
        }
    }

    #[test]
    fn test_soft_mask_group_luminosity_with_backdrop() {
        let gs = ExtGStateBuilder::new()
            .soft_mask(SoftMask::Group {
                group_ref: "G2".to_string(),
                subtype: SoftMaskSubtype::Luminosity,
                backdrop: Some(vec![1.0, 0.5, 0.0]),
                transfer: None,
            })
            .build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Dictionary(smask_dict)) = dict.get("SMask") {
                assert_eq!(smask_dict.get("S"), Some(&Object::Name("Luminosity".to_string())));
                assert!(smask_dict.contains_key("BC"));
                if let Some(Object::Array(bc)) = smask_dict.get("BC") {
                    assert_eq!(bc.len(), 3);
                }
            } else {
                panic!("Expected Dictionary for SMask");
            }
        }
    }

    // ---- Tests for predefined effects ----

    #[test]
    fn test_screen_effect() {
        let gs = ExtGStateBuilder::screen().build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Name(name)) = dict.get("BM") {
                assert_eq!(name, "Screen");
            } else {
                panic!("Expected BM with Screen");
            }
        }
    }

    #[test]
    fn test_overlay_effect() {
        let gs = ExtGStateBuilder::overlay().build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Name(name)) = dict.get("BM") {
                assert_eq!(name, "Overlay");
            } else {
                panic!("Expected BM with Overlay");
            }
        }
    }

    #[test]
    fn test_difference_effect() {
        let gs = ExtGStateBuilder::difference().build();
        if let Object::Dictionary(dict) = gs {
            if let Some(Object::Name(name)) = dict.get("BM") {
                assert_eq!(name, "Difference");
            } else {
                panic!("Expected BM with Difference");
            }
        }
    }

    // ---- Tests for default builder ----

    #[test]
    fn test_default_builder_only_has_type() {
        let gs = ExtGStateBuilder::new().build();
        if let Object::Dictionary(dict) = gs {
            // Should only have "Type" key since nothing else was set
            assert_eq!(dict.len(), 1);
            assert_eq!(dict.get("Type"), Some(&Object::Name("ExtGState".to_string())));
        }
    }

    // ---- Tests for chaining ----

    #[test]
    fn test_builder_chaining_many_properties() {
        let gs = ExtGStateBuilder::new()
            .fill_alpha(0.7)
            .stroke_alpha(0.9)
            .blend_mode(BlendMode::Overlay)
            .overprint_stroke(true)
            .overprint_fill(false)
            .line_width(1.0)
            .flatness(100.0)
            .no_soft_mask()
            .build();
        if let Object::Dictionary(dict) = gs {
            assert!(dict.contains_key("ca"));
            assert!(dict.contains_key("CA"));
            assert!(dict.contains_key("BM"));
            assert!(dict.contains_key("OP"));
            assert!(dict.contains_key("op"));
            assert!(dict.contains_key("LW"));
            assert!(dict.contains_key("FL"));
            assert!(dict.contains_key("SMask"));
            assert!(dict.contains_key("Type"));
        } else {
            panic!("Expected Dictionary");
        }
    }

    // ---- Tests for soft_mask method ----

    #[test]
    fn test_soft_mask_none_variant() {
        let gs = ExtGStateBuilder::new().soft_mask(SoftMask::None).build();
        if let Object::Dictionary(dict) = gs {
            assert_eq!(dict.get("SMask"), Some(&Object::Name("None".to_string())));
        }
    }
}
