//! 가로 Canvas2D 텍스트와 실측 provider의 공통 글꼴 설정.

use super::{canvas_font_family_chain, condensed_ratio_draw_params, TextStyle};

pub(super) fn font_family_chain(style: &TextStyle) -> String {
    let fallback = canvas_font_family_chain(&style.font_family);
    if !style.bold {
        return fallback;
    }
    let primary = style.font_family.split(',').next().unwrap_or("").trim();
    let aliases = super::font_face_aliases::local_bold_aliases(primary);
    if aliases.is_empty() {
        return fallback;
    }
    let actual_bold = aliases
        .iter()
        // local()의 full face 이름은 Canvas의 CSS family와 다르다.
        // 실제 family에서 요청한 bold weight를 선택해야 같은 프로그램을 쓴다.
        .filter_map(|family| super::base_family_without_weight_suffix(family))
        .map(|family| format!("\"{}\"", family.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{actual_bold}, {fallback}")
}

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
    /// 글자 크기·첨자·장평·CSS 소수점 규칙을 실측과 paint에서 공유한다.
    /// 굵게는 실제 Bold 후보를 먼저 찾고, 없으면 기존 family 체인을 사용한다.
    pub fn for_positioned_text(style: &TextStyle, baseline: f64) -> Self {
        let base_size = if style.font_size > 0.0 {
            style.font_size
        } else {
            12.0
        };
        let (size, baseline) = style.script_draw_metrics(base_size, baseline);
        let (draw_size, horizontal_scale) = condensed_ratio_draw_params(size, style.ratio);
        let family = font_family_chain(style);
        let prefix = format!(
            "{}{}{:.3}px ",
            if style.italic { "italic " } else { "" },
            if style.bold { "bold " } else { "" },
            draw_size,
        );
        Self {
            descriptor: format!("{prefix}{family}"),
            old_hangul_descriptor: format!("{prefix}'RHWP Old Hangul', {family}"),
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
