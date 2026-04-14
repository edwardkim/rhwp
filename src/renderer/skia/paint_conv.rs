use skia_safe::{paint, Color, Font, FontHinting, FontMgr, FontStyle, Paint};

use crate::renderer::{
    generic_fallback, LineStyle, ShapeStyle, StrokeDash, TextStyle,
};

pub fn colorref_to_skia(color: u32, alpha_scale: f32) -> Color {
    let b = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let a = (255.0 * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_argb(a, r, g, b)
}

pub fn make_fill_paint(style: &ShapeStyle) -> Option<Paint> {
    let fill_color = style.fill_color?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(colorref_to_skia(fill_color, style.opacity as f32));
    Some(paint)
}

pub fn make_stroke_paint(style: &ShapeStyle) -> Option<Paint> {
    let stroke_color = style.stroke_color?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Stroke);
    paint.set_stroke_width(style.stroke_width.max(1.0) as f32);
    paint.set_color(colorref_to_skia(stroke_color, style.opacity as f32));
    apply_dash(&mut paint, style.stroke_dash);
    Some(paint)
}

pub fn make_line_paint(style: &LineStyle) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Stroke);
    paint.set_stroke_width(style.width.max(1.0) as f32);
    paint.set_color(colorref_to_skia(style.color, 1.0));
    apply_dash(&mut paint, style.dash);
    paint
}

pub fn make_font(text_style: &TextStyle, font_mgr: &FontMgr) -> Font {
    let font_size = if text_style.font_size > 0.0 {
        text_style.font_size as f32
    } else {
        12.0
    };
    let font_style = match (text_style.bold, text_style.italic) {
        (true, true) => FontStyle::bold_italic(),
        (true, false) => FontStyle::bold(),
        (false, true) => FontStyle::italic(),
        (false, false) => FontStyle::normal(),
    };

    let matched = if text_style.font_family.is_empty() {
        None
    } else {
        font_mgr
            .match_family_style(&text_style.font_family, font_style)
            .or_else(|| font_mgr.match_family_style(generic_fallback(&text_style.font_family), font_style))
    };

    let mut font = if let Some(typeface) = matched {
        Font::new(typeface, font_size)
    } else {
        let mut font = Font::default();
        font.set_size(font_size);
        font
    };

    font.set_edging(skia_safe::font::Edging::AntiAlias);
    font.set_hinting(FontHinting::Slight);
    font.set_embolden(text_style.bold);
    font.set_scale_x(if text_style.ratio > 0.0 {
        text_style.ratio as f32
    } else {
        1.0
    });
    if text_style.italic {
        font.set_skew_x(-0.2);
    }
    font
}

pub fn make_text_paint(text_style: &TextStyle) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(colorref_to_skia(text_style.color, 1.0));
    paint
}

fn apply_dash(paint: &mut Paint, dash: StrokeDash) {
    let intervals: Option<[f32; 6]> = match dash {
        StrokeDash::Solid => None,
        StrokeDash::Dash => Some([6.0, 3.0, 0.0, 0.0, 0.0, 0.0]),
        StrokeDash::Dot => Some([2.0, 2.0, 0.0, 0.0, 0.0, 0.0]),
        StrokeDash::DashDot => Some([6.0, 3.0, 2.0, 3.0, 0.0, 0.0]),
        StrokeDash::DashDotDot => Some([6.0, 3.0, 2.0, 3.0, 2.0, 3.0]),
    };
    if let Some(intervals) = intervals {
        let trimmed: Vec<f32> = intervals.into_iter().filter(|value| *value > 0.0).collect();
        if let Some(effect) = skia_safe::PathEffect::dash(&trimmed, 0.0) {
            paint.set_path_effect(effect);
        }
    }
}
