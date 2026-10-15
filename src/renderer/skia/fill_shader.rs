//! Native gradient and pattern fills shared by all shape replay paths.

use super::renderer::colorref_to_skia;
use crate::renderer::{render_tree::BoundingBox, GradientFillInfo, PatternFillInfo, ShapeStyle};
use skia_safe::{paint, surfaces, FilterMode, MipmapMode, Paint, Rect, SamplingOptions, TileMode};

pub(super) fn shape_fill(
    bounds: &BoundingBox,
    style: &ShapeStyle,
    gradient: Option<&GradientFillInfo>,
    antialias: bool,
) -> Option<Paint> {
    // Consume the same center/stop expansion and physical axis as upstream
    // rectangle backgrounds for every shape, including ellipses and paths.
    if let Some(mut paint) =
        gradient.and_then(|gradient| super::fill::gradient_paint(gradient, bounds, style.opacity))
    {
        paint.set_anti_alias(antialias);
        return Some(paint);
    }
    let mut paint = Paint::default();
    paint.set_anti_alias(antialias);
    if let Some(shader) = style.pattern.as_ref().and_then(make_pattern_shader) {
        paint.set_alpha_f(style.opacity.clamp(0.0, 1.0) as f32);
        paint.set_shader(shader);
    } else {
        // Retain the current renderer's deterministic malformed/single-stop
        // fallback rather than removing a previously visible shape.
        let color = gradient
            .and_then(|gradient| gradient.colors.first().copied())
            .or(style.pattern.map(|pattern| pattern.background_color))
            .or(style.fill_color)?;
        paint.set_color(colorref_to_skia(color, style.opacity as f32));
    }
    Some(paint)
}

fn make_pattern_shader(pattern: &PatternFillInfo) -> Option<skia_safe::Shader> {
    let mut surface = surfaces::raster_n32_premul((6, 6))?;
    let canvas = surface.canvas();

    let mut background = Paint::default();
    background.set_anti_alias(false);
    background.set_style(paint::Style::Fill);
    background.set_color(colorref_to_skia(pattern.background_color, 1.0));
    canvas.draw_rect(Rect::from_xywh(0.0, 0.0, 6.0, 6.0), &background);

    let mut foreground = Paint::default();
    foreground.set_anti_alias(true);
    foreground.set_style(paint::Style::Stroke);
    foreground.set_stroke_width(1.0);
    foreground.set_color(colorref_to_skia(pattern.pattern_color, 1.0));

    match pattern.pattern_type {
        0 => {
            canvas.draw_line((0.0, 3.0), (6.0, 3.0), &foreground);
        }
        1 => {
            canvas.draw_line((3.0, 0.0), (3.0, 6.0), &foreground);
        }
        2 => {
            canvas.draw_line((6.0, 0.0), (0.0, 6.0), &foreground);
        }
        3 => {
            canvas.draw_line((0.0, 0.0), (6.0, 6.0), &foreground);
        }
        4 => {
            canvas.draw_line((3.0, 0.0), (3.0, 6.0), &foreground);
            canvas.draw_line((0.0, 3.0), (6.0, 3.0), &foreground);
        }
        5 => {
            canvas.draw_line((0.0, 0.0), (6.0, 6.0), &foreground);
            canvas.draw_line((6.0, 0.0), (0.0, 6.0), &foreground);
        }
        _ => {}
    }

    surface.image_snapshot().to_shader(
        Some((TileMode::Repeat, TileMode::Repeat)),
        SamplingOptions::new(FilterMode::Nearest, MipmapMode::None),
        None,
    )
}
