//! Bounded Native fills using the shared upstream gradient geometry.

use super::renderer::colorref_to_skia;
use crate::renderer::{render_tree::BoundingBox, GradientFillInfo, PatternFillInfo, ShapeStyle};
use skia_safe::{paint, surfaces, FilterMode, MipmapMode, Paint, Rect, SamplingOptions, TileMode};

pub(super) fn shape_fill(
    bounds: &BoundingBox,
    style: &ShapeStyle,
    gradient: Option<&GradientFillInfo>,
    antialias: bool,
) -> Option<Paint> {
    // Keep P47's admission/stop bound while consuming upstream's center/stop
    // expansion and physical axis for backgrounds and every shape alike.
    let raster_bounds = Rect::from_xywh(
        bounds.x as f32,
        bounds.y as f32,
        bounds.width as f32,
        bounds.height as f32,
    );
    if let Some(mut paint) = gradient
        .filter(|gradient| {
            (2..=4096).contains(&gradient.colors.len())
                && raster_bounds.is_finite()
                && !raster_bounds.is_empty()
        })
        .and_then(|gradient| super::fill::gradient_paint(gradient, bounds, style.opacity))
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

/// 무늬 한 주기를 이루는 픽셀 타일의 한 변. 100dpi 8-dot 무늬다.
const PATTERN_TILE_DOTS: i32 = 8;

/// 무늬 타일에서 무늬색으로 칠할 칸인지. Studio CanvasKit(`drawPatternFill`)과 같은 표다.
fn pattern_ink(hatch: crate::model::style::HatchStyle, x: i32, y: i32) -> bool {
    use crate::model::style::HatchStyle;
    let horizontal = y == 0;
    let vertical = x == 0;
    let rising = (x + y) % PATTERN_TILE_DOTS == 0;
    let falling = x == y;
    match hatch {
        HatchStyle::Horizontal => horizontal,
        HatchStyle::Vertical => vertical,
        HatchStyle::Falling => falling,
        HatchStyle::Rising => rising,
        HatchStyle::Cross => horizontal || vertical,
        HatchStyle::DiagonalCross => rising || falling,
    }
}

fn make_pattern_shader(pattern: &PatternFillInfo) -> Option<skia_safe::Shader> {
    let hatch = crate::model::style::HatchStyle::from_pattern_type(pattern.pattern_type);
    let mut surface = surfaces::raster_n32_premul((PATTERN_TILE_DOTS, PATTERN_TILE_DOTS))?;
    let canvas = surface.canvas();

    let mut background = Paint::default();
    background.set_anti_alias(false);
    background.set_style(paint::Style::Fill);
    background.set_color(colorref_to_skia(pattern.background_color, 1.0));
    let side = PATTERN_TILE_DOTS as f32;
    canvas.draw_rect(Rect::from_xywh(0.0, 0.0, side, side), &background);

    let mut foreground = Paint::default();
    foreground.set_anti_alias(false);
    foreground.set_style(paint::Style::Fill);
    foreground.set_color(colorref_to_skia(pattern.pattern_color, 1.0));
    if let Some(hatch) = hatch {
        for y in 0..PATTERN_TILE_DOTS {
            for x in 0..PATTERN_TILE_DOTS {
                if pattern_ink(hatch, x, y) {
                    canvas.draw_rect(Rect::from_xywh(x as f32, y as f32, 1.0, 1.0), &foreground);
                }
            }
        }
    }

    // 레이아웃이 정한 한 주기(576HU)를 쪽 원점에 고정해 반복한다. SVG·웹 캔버스와 같다.
    let scale = if pattern.tile_size.is_finite() && pattern.tile_size > 0.0 {
        (pattern.tile_size / PATTERN_TILE_DOTS as f64) as f32
    } else {
        1.0
    };
    let matrix = skia_safe::Matrix::new_all(
        scale,
        0.0,
        -scale / 2.0,
        0.0,
        scale,
        -scale / 2.0,
        0.0,
        0.0,
        1.0,
    );
    surface.image_snapshot().to_shader(
        Some((TileMode::Repeat, TileMode::Repeat)),
        SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
        Some(&matrix),
    )
}
