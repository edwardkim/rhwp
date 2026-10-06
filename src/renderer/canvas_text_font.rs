//! Shared font setup for horizontal positioned Canvas2D text and its provider.
//! This does not select an exact fallback face or authorize another backend.

use super::{canvas_font_family_chain, condensed_ratio_draw_params, TextStyle};

#[derive(Debug, Clone)]
pub struct CanvasTextFont {
    descriptor: String,
    old_hangul_descriptor: String,
    draw_size: f64,
    baseline: f64,
    horizontal_scale: f64,
    layout_ratio: f64,
}

impl CanvasTextFont {
    /// Preserve the positioned painter's existing defaults, script geometry,
    /// condensed-ratio rule, family chain and three-decimal CSS size verbatim.
    /// Registration must reject invalid styles; painting keeps its old fallback.
    pub fn for_positioned_text(style: &TextStyle, baseline: f64) -> Self {
        let base_size = if style.font_size > 0.0 {
            style.font_size
        } else {
            12.0
        };
        let (size, baseline) = style.script_draw_metrics(base_size, baseline);
        let (draw_size, horizontal_scale) = condensed_ratio_draw_params(size, style.ratio);
        let family = canvas_font_family_chain(&style.font_family);
        let prefix = format!(
            "{}{}{:.3}px ",
            if style.italic { "italic " } else { "" },
            if style.bold { "bold " } else { "" },
            draw_size,
        );
        Self {
            descriptor: format!("{prefix}{family}"),
            old_hangul_descriptor: format!("{prefix}'Source Han Serif K Old Hangul', {family}"),
            draw_size,
            baseline,
            horizontal_scale,
            layout_ratio: if style.ratio > 0.0 { style.ratio } else { 1.0 },
        }
    }

    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    pub fn old_hangul_descriptor(&self) -> &str {
        &self.old_hangul_descriptor
    }

    pub fn draw_size(&self) -> f64 {
        self.draw_size
    }

    pub fn baseline(&self) -> f64 {
        self.baseline
    }

    pub fn horizontal_scale(&self) -> f64 {
        self.horizontal_scale
    }

    /// Convert measureText at the *actual CSS size* to the pre-ratio advance
    /// consumed by layout. Then layout_advance = measured * paint_scale exactly.
    /// Do not scale by nominal_size / rounded_CSS_size: that would undo CSS
    /// rounding (or optical-size-dependent metrics) which the painter retains.
    pub fn natural_advance(&self, measured_advance: f64) -> Option<f64> {
        if !measured_advance.is_finite()
            || measured_advance < 0.0
            || !self.draw_size.is_finite()
            || self.draw_size < 0.0005 // CSS {:.3} would become 0.000px
            || !self.layout_ratio.is_finite()
            || !self.horizontal_scale.is_finite()
        {
            return None;
        }
        let advance = measured_advance * (self.horizontal_scale / self.layout_ratio);
        (advance.is_finite() && advance >= 0.0).then_some(advance)
    }
}
