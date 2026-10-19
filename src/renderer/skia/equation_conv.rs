use skia_safe::{
    font, paint, Canvas, Color, Font, FontMgr, FontStyle, Paint, PathBuilder, Typeface,
};
use std::collections::HashMap;

use super::font_catalog::TypefaceCatalog;
use super::font_lookup::{match_supplied_family, match_system_family_style, SystemFontFamilies};
mod glyphs;
use glyphs::{draw_stretch_bracket, draw_text, draw_text_with_family};

use crate::renderer::equation::ast::MatrixStyle;
use crate::renderer::equation::layout::{
    is_integral_symbol, LayoutBox, LayoutKind, BIG_OP_SCALE, INTEGRAL_SCALE, SCRIPT_SCALE,
};
use crate::renderer::equation::symbols::{DecoKind, FontStyleKind};

const EQ_FONT_FAMILY: &str =
    "Latin Modern Math, STIX Two Math, Cambria Math, DejaVu Math TeX Gyre, DejaVu Sans, Times New Roman, serif";

/// Font resources supplied to body text and every recursive equation branch.
pub(super) struct EquationFonts<'a> {
    pub(super) font_mgr: &'a FontMgr,
    pub(super) system_families: &'a SystemFontFamilies,
    pub(super) custom_typefaces: &'a TypefaceCatalog,
    pub(super) bundled_typefaces: &'a TypefaceCatalog,
}

pub(super) fn render_equation(
    canvas: &Canvas,
    fonts: &EquationFonts<'_>,
    layout: &LayoutBox,
    origin_x: f64,
    origin_y: f64,
    color: u32,
    base_font_size: f64,
) {
    render_box(
        canvas,
        fonts,
        layout,
        origin_x,
        origin_y,
        colorref_to_skia(color, 1.0),
        base_font_size,
        true,
        false,
    );
}

fn render_box(
    canvas: &Canvas,
    fonts: &EquationFonts<'_>,
    lb: &LayoutBox,
    parent_x: f64,
    parent_y: f64,
    color: Color,
    fs: f64,
    italic: bool,
    bold: bool,
) {
    let fs = if lb.font_size > 0.0 { lb.font_size } else { fs };
    let x = parent_x + lb.x;
    let y = parent_y + lb.y;

    match &lb.kind {
        LayoutKind::HancomGlyphs(run) => {
            let family = run.font_family;
            let typeface =
                match_supplied_family(fonts.custom_typefaces, family, FontStyle::normal())
                    .or_else(|| {
                        match_supplied_family(fonts.bundled_typefaces, family, FontStyle::normal())
                    })
                    .or_else(|| {
                        match_system_family_style(
                            fonts.font_mgr,
                            fonts.system_families,
                            family,
                            FontStyle::normal(),
                        )
                    });
            let mut font = if let Some(typeface) = typeface {
                Font::new(typeface, fs as f32)
            } else {
                let mut font = Font::default();
                font.set_size(fs as f32);
                font
            };
            font.set_edging(font::Edging::AntiAlias);
            font.set_hinting(skia_safe::FontHinting::None);
            font.set_subpixel(true);
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(color);
            for ((glyph, offset), shear) in run.glyphs.chars().zip(&run.positions).zip(&run.shears)
            {
                font.set_skew_x(-(*shear as f32));
                canvas.draw_str(
                    glyph.to_string(),
                    (
                        (x + offset) as f32,
                        (y + lb.baseline + run.baseline_shift) as f32,
                    ),
                    &font,
                    &paint,
                );
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                render_box(canvas, fonts, child, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Text(text) => {
            if crate::renderer::equation::text_has_cjk(text)
                && (lb.width
                    - crate::renderer::equation::layout::estimate_text_width(
                        text,
                        font_size_from_box(lb, fs),
                        false,
                    ))
                .abs()
                    > 0.01
            {
                let font_size = font_size_from_box(lb, fs);
                for (ch, offset) in
                    crate::renderer::equation::positioned_cjk_text(text, font_size, lb.width)
                {
                    draw_text(
                        canvas,
                        fonts,
                        &ch.to_string(),
                        x + offset,
                        y + lb.baseline,
                        font_size,
                        false,
                        bold,
                        color,
                        false,
                    );
                }
                return;
            }
            draw_text(
                canvas,
                fonts,
                text,
                x,
                y + lb.baseline,
                font_size_from_box(lb, fs),
                italic && !crate::renderer::equation::layout::is_styled_variable(text),
                bold && !crate::renderer::equation::layout::is_styled_variable(text),
                color,
                false,
            );
        }
        LayoutKind::Number(text) => {
            draw_text(
                canvas,
                fonts,
                text,
                x,
                y + lb.baseline,
                font_size_from_box(lb, fs),
                false,
                bold,
                color,
                false,
            );
        }
        LayoutKind::Symbol(text) => {
            draw_text(
                canvas,
                fonts,
                text,
                x + lb.width / 2.0,
                y + lb.baseline,
                font_size_from_box(lb, fs),
                false,
                false,
                color,
                true,
            );
        }
        LayoutKind::FontGlyph {
            text,
            font_family,
            outline,
            paint,
        } => {
            if let Some(commands) = outline {
                super::equation_glyph::draw_outline(canvas, commands, x, y, color, *paint);
                return;
            }
            draw_text_with_family(
                canvas,
                fonts,
                text,
                x,
                y + lb.baseline,
                fs,
                false,
                false,
                color,
                false,
                Some(font_family),
            );
        }
        LayoutKind::MathSymbol(text) => {
            // Task #1317: 적분 기호(∫)는 폰트 text 가 아닌 stroke path 로 렌더(geom SSOT,
            // svg/canvas 와 동일). 그 외 MathSymbol 은 text 렌더.
            if is_integral_symbol(text) {
                draw_integral(canvas, x, y, fs, color);
            } else {
                draw_text(
                    canvas,
                    fonts,
                    text,
                    x,
                    y + lb.baseline,
                    font_size_from_box(lb, fs),
                    false,
                    false,
                    color,
                    false,
                );
            }
        }
        LayoutKind::Function(name) => {
            draw_text(
                canvas,
                fonts,
                name,
                x,
                y + lb.baseline,
                font_size_from_box(lb, fs),
                false,
                false,
                color,
                false,
            );
        }
        LayoutKind::Fraction {
            numer,
            denom,
            axis_height,
            rule_thickness,
        } => {
            render_box(canvas, fonts, numer, x, y, color, fs, italic, bold);
            let line_y = y + lb.baseline - axis_height;
            canvas.draw_line(
                ((x + fs * 0.05) as f32, line_y as f32),
                ((x + lb.width - fs * 0.05) as f32, line_y as f32),
                &stroke_paint(color, *rule_thickness),
            );
            render_box(canvas, fonts, denom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Atop { top, bottom } => {
            render_box(canvas, fonts, top, x, y, color, fs, italic, bold);
            render_box(canvas, fonts, bottom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Sqrt { index, body } => {
            let geometry = crate::renderer::equation::radical_outline::geometry(lb, body, fs);
            let mut path = PathBuilder::new();
            for command in &geometry.outline {
                use crate::renderer::equation::GlyphOutlineCommand as Command;
                match *command {
                    Command::Move((a, b)) => {
                        path.move_to(((x + a) as f32, (y + b) as f32));
                    }
                    Command::Line((a, b)) => {
                        path.line_to(((x + a) as f32, (y + b) as f32));
                    }
                    Command::Cubic([(a, b), (c, d), (e, f)]) => {
                        path.cubic_to(
                            ((x + a) as f32, (y + b) as f32),
                            ((x + c) as f32, (y + d) as f32),
                            ((x + e) as f32, (y + f) as f32),
                        );
                    }
                    Command::Close => {
                        path.close();
                    }
                }
            }
            canvas.draw_path(&path.detach(), &fill_paint(color));
            canvas.draw_line(
                (
                    (x + geometry.bar_start.0) as f32,
                    (y + geometry.bar_start.1) as f32,
                ),
                (
                    (x + geometry.bar_end.0) as f32,
                    (y + geometry.bar_end.1) as f32,
                ),
                &stroke_paint(color, geometry.rule_thickness),
            );

            if let Some(index) = index {
                render_box(
                    canvas,
                    fonts,
                    index,
                    x,
                    y,
                    color,
                    fs * crate::renderer::equation::layout::RADICAL_DEGREE_SCALE,
                    false,
                    false,
                );
            }
            render_box(canvas, fonts, body, x, y, color, fs, italic, bold);
        }
        LayoutKind::Superscript { base, sup } => {
            render_box(canvas, fonts, base, x, y, color, fs, italic, bold);
            render_box(
                canvas,
                fonts,
                sup,
                x,
                y,
                color,
                fs * SCRIPT_SCALE,
                italic,
                bold,
            );
        }
        LayoutKind::Subscript { base, sub } => {
            render_box(canvas, fonts, base, x, y, color, fs, italic, bold);
            render_box(
                canvas,
                fonts,
                sub,
                x,
                y,
                color,
                fs * SCRIPT_SCALE,
                italic,
                bold,
            );
        }
        LayoutKind::SubSup { base, sub, sup } => {
            render_box(canvas, fonts, base, x, y, color, fs, italic, bold);
            render_box(
                canvas,
                fonts,
                sub,
                x,
                y,
                color,
                fs * SCRIPT_SCALE,
                italic,
                bold,
            );
            render_box(
                canvas,
                fonts,
                sup,
                x,
                y,
                color,
                fs * SCRIPT_SCALE,
                italic,
                bold,
            );
        }
        LayoutKind::BigOp {
            symbol,
            sub,
            sup,
            op_font_size,
            op_x,
            op_y,
            op_baseline,
            ..
        } => {
            let is_integral = is_integral_symbol(symbol);
            // 생산 단계에서 확정한 표시 글자 크기와 원점을 소비한다.
            let op_fs = *op_font_size;
            if is_integral {
                // Task #1317: 적분 기호는 stroke path 로 렌더(geom SSOT).
                draw_integral(canvas, x + op_x, y + op_y, fs, color);
            } else {
                let op_x = x + op_x;
                let op_y = y + op_baseline;
                draw_text(
                    canvas, fonts, symbol, op_x, op_y, op_fs, false, false, color, false,
                );
            }
            if let Some(sup) = sup {
                render_box(
                    canvas,
                    fonts,
                    sup,
                    x,
                    y,
                    color,
                    fs * SCRIPT_SCALE,
                    false,
                    false,
                );
            }
            if let Some(sub) = sub {
                render_box(
                    canvas,
                    fonts,
                    sub,
                    x,
                    y,
                    color,
                    fs * SCRIPT_SCALE,
                    false,
                    false,
                );
            }
        }
        LayoutKind::Limit {
            is_upper,
            name,
            name_font_size,
            sub_font_size,
            sub,
        } => {
            if let Some(name) = name {
                render_box(canvas, fonts, name, x, y, color, name.height, false, false);
            } else {
                let name = if *is_upper { "Lim" } else { "lim" };
                draw_text(
                    canvas,
                    fonts,
                    name,
                    x,
                    y + lb.baseline,
                    *name_font_size,
                    false,
                    false,
                    color,
                    false,
                );
            }
            if let Some(sub) = sub {
                render_box(
                    canvas,
                    fonts,
                    sub,
                    x,
                    y,
                    color,
                    *sub_font_size,
                    false,
                    false,
                );
            }
        }
        LayoutKind::Matrix { cells, style } => {
            let bracket_chars = match style {
                MatrixStyle::Paren => ("(", ")"),
                MatrixStyle::Bracket => ("[", "]"),
                MatrixStyle::Vert => ("|", "|"),
                MatrixStyle::Plain | MatrixStyle::Ladder | MatrixStyle::ShortLadder => ("", ""),
            };
            if !bracket_chars.0.is_empty() {
                draw_stretch_bracket(
                    canvas,
                    fonts,
                    bracket_chars.0,
                    x,
                    y,
                    fs * 0.3,
                    lb.height,
                    color,
                    fs,
                );
                draw_stretch_bracket(
                    canvas,
                    fonts,
                    bracket_chars.1,
                    x + lb.width - fs * 0.3,
                    y,
                    fs * 0.3,
                    lb.height,
                    color,
                    fs,
                );
            }
            for row in cells {
                for cell in row {
                    render_box(canvas, fonts, cell, x, y, color, fs, italic, bold);
                }
            }
        }
        LayoutKind::Rel { arrow, over, under } => {
            render_box(canvas, fonts, over, x, y, color, fs, italic, bold);
            render_box(canvas, fonts, arrow, x, y, color, fs, italic, bold);
            if let Some(under) = under {
                render_box(canvas, fonts, under, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::EqAlign { rows } => {
            for (left, right) in rows {
                render_box(canvas, fonts, left, x, y, color, fs, italic, bold);
                render_box(canvas, fonts, right, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Paren {
            left,
            right,
            left_width,
            right_width,
            delimiter_height,
            glyph_delimiters,
            left_outline,
            right_outline,
            body,
        } => {
            let use_glyph = *glyph_delimiters;
            if !left.is_empty() {
                if let Some(commands) = left_outline {
                    super::equation_glyph::draw_outline(canvas, commands, x, y, color, None);
                } else if use_glyph && (left == "(" || left == ")") {
                    draw_text(
                        canvas,
                        fonts,
                        left,
                        x,
                        y + lb.baseline,
                        fs,
                        false,
                        false,
                        color,
                        false,
                    );
                } else {
                    draw_stretch_bracket(
                        canvas,
                        fonts,
                        left,
                        x,
                        y,
                        *left_width,
                        *delimiter_height,
                        color,
                        fs,
                    );
                }
            }
            render_box(canvas, fonts, body, x, y, color, fs, italic, bold);
            if !right.is_empty() {
                let right_x = x + lb.width - right_width;
                if let Some(commands) = right_outline {
                    super::equation_glyph::draw_outline(canvas, commands, right_x, y, color, None);
                } else if use_glyph && (right == "(" || right == ")") {
                    draw_text(
                        canvas,
                        fonts,
                        right,
                        right_x,
                        y + lb.baseline,
                        fs,
                        false,
                        false,
                        color,
                        false,
                    );
                } else {
                    draw_stretch_bracket(
                        canvas,
                        fonts,
                        right,
                        right_x,
                        y,
                        *right_width,
                        *delimiter_height,
                        color,
                        fs,
                    );
                }
            }
        }
        LayoutKind::Decoration {
            kind,
            body,
            negation,
            outline,
        } => {
            render_box(canvas, fonts, body, x, y, color, fs, italic, bold);
            if let Some(commands) = outline {
                super::equation_glyph::draw_outline(canvas, commands, x, y, color, None);
            } else if *kind == DecoKind::StrikeThrough {
                let g = negation.clone().unwrap_or_else(|| {
                    crate::renderer::equation::negation_geometry::geometry(body)
                });
                canvas.draw_line(
                    ((x + g.start.0) as f32, (y + g.start.1) as f32),
                    ((x + g.end.0) as f32, (y + g.end.1) as f32),
                    &stroke_paint(color, g.thickness),
                );
            } else if *kind == DecoKind::Box {
                canvas.draw_rect(
                    skia_safe::Rect::from_xywh(
                        x as f32,
                        y as f32,
                        lb.width as f32,
                        lb.height as f32,
                    ),
                    &stroke_paint(color, fs * 0.03),
                );
            } else {
                let deco_y = y + fs * 0.05;
                let mid_x = x + body.x + body.width / 2.0;
                draw_decoration(canvas, *kind, mid_x, deco_y, body.width, color, fs);
            }
        }
        LayoutKind::Color { rgb, body } => {
            // RGB 범위는 해당 하위 수식에만 적용하며 형제의 원래 색상은 보존한다.
            render_box(
                canvas,
                fonts,
                body,
                x,
                y,
                Color::from_rgb(rgb[0], rgb[1], rgb[2]),
                fs,
                italic,
                bold,
            );
        }
        LayoutKind::FontStyle { style, body } => {
            let (new_italic, new_bold) = match style {
                FontStyleKind::Roman | FontStyleKind::SansSerif | FontStyleKind::Monospace => {
                    (false, false)
                }
                FontStyleKind::Italic => (true, bold),
                FontStyleKind::Bold => (italic, true),
                FontStyleKind::Blackboard => (false, true),
                FontStyleKind::Calligraphy | FontStyleKind::Fraktur => (false, false),
            };
            render_box(canvas, fonts, body, x, y, color, fs, new_italic, new_bold);
        }
        LayoutKind::Space(_) | LayoutKind::Newline | LayoutKind::Empty => {}
    }
}

fn draw_decoration(
    canvas: &Canvas,
    kind: DecoKind,
    mid_x: f64,
    y: f64,
    width: f64,
    color: Color,
    fs: f64,
) {
    let half_w = width / 2.0;
    let paint = stroke_paint(color, fs * 0.03);

    match kind {
        DecoKind::Hat => {
            let mut path = PathBuilder::new();
            path.move_to(((mid_x - half_w * 0.6) as f32, (y + fs * 0.15) as f32));
            path.line_to((mid_x as f32, y as f32));
            path.line_to(((mid_x + half_w * 0.6) as f32, (y + fs * 0.15) as f32));
            canvas.draw_path(&path.detach(), &paint);
        }
        DecoKind::Bar | DecoKind::Overline => {
            canvas.draw_line(
                ((mid_x - half_w) as f32, (y + fs * 0.05) as f32),
                ((mid_x + half_w) as f32, (y + fs * 0.05) as f32),
                &paint,
            );
        }
        DecoKind::Vec | DecoKind::Dyad => {
            let arrow_y = y + fs * 0.05;
            canvas.draw_line(
                ((mid_x - half_w) as f32, arrow_y as f32),
                ((mid_x + half_w) as f32, arrow_y as f32),
                &paint,
            );
            let mut head = PathBuilder::new();
            head.move_to((
                (mid_x + half_w - fs * 0.1) as f32,
                (arrow_y - fs * 0.06) as f32,
            ));
            head.line_to(((mid_x + half_w) as f32, arrow_y as f32));
            head.line_to((
                (mid_x + half_w - fs * 0.1) as f32,
                (arrow_y + fs * 0.06) as f32,
            ));
            canvas.draw_path(&head.detach(), &paint);
            if kind == DecoKind::Dyad {
                let mut head = PathBuilder::new();
                head.move_to((
                    (mid_x - half_w + fs * 0.1) as f32,
                    (arrow_y - fs * 0.06) as f32,
                ));
                head.line_to(((mid_x - half_w) as f32, arrow_y as f32));
                head.line_to((
                    (mid_x - half_w + fs * 0.1) as f32,
                    (arrow_y + fs * 0.06) as f32,
                ));
                canvas.draw_path(&head.detach(), &paint);
            }
        }
        DecoKind::Tilde => {
            let ty = y + fs * 0.08;
            let mut path = PathBuilder::new();
            path.move_to(((mid_x - half_w * 0.6) as f32, ty as f32));
            path.quad_to(
                ((mid_x - half_w * 0.2) as f32, (ty - fs * 0.08) as f32),
                (mid_x as f32, ty as f32),
            );
            path.quad_to(
                ((mid_x + half_w * 0.2) as f32, (ty + fs * 0.08) as f32),
                ((mid_x + half_w * 0.6) as f32, ty as f32),
            );
            canvas.draw_path(&path.detach(), &paint);
        }
        DecoKind::Dot => {
            canvas.draw_circle(
                (mid_x as f32, (y + fs * 0.06) as f32),
                (fs * 0.03) as f32,
                &fill_paint(color),
            );
        }
        DecoKind::DDot => {
            let gap = fs * 0.1;
            let fill = fill_paint(color);
            canvas.draw_circle(
                ((mid_x - gap) as f32, (y + fs * 0.06) as f32),
                (fs * 0.03) as f32,
                &fill,
            );
            canvas.draw_circle(
                ((mid_x + gap) as f32, (y + fs * 0.06) as f32),
                (fs * 0.03) as f32,
                &fill,
            );
        }
        DecoKind::Underline | DecoKind::Under => {
            let underline_y = y + fs * 1.1;
            canvas.draw_line(
                ((mid_x - half_w) as f32, underline_y as f32),
                ((mid_x + half_w) as f32, underline_y as f32),
                &paint,
            );
        }
        _ => {
            canvas.draw_line(
                ((mid_x - half_w * 0.5) as f32, (y + fs * 0.1) as f32),
                ((mid_x + half_w * 0.5) as f32, (y + fs * 0.1) as f32),
                &paint,
            );
        }
    }
}

fn font_size_from_box(lb: &LayoutBox, base_fs: f64) -> f64 {
    if lb.font_size > 0.0 {
        lb.font_size
    } else {
        base_fs
    }
}

/// 적분 기호(∫)를 stroke path 로 렌더 (Task #1317).
///
/// SVG·Canvas2D와 같은 공개 적분 윤곽 및 공통 ink 영역을 소비한다.
fn draw_integral(canvas: &Canvas, x: f64, y: f64, fs: f64, color: Color) {
    use crate::renderer::equation::integral_outline::{commands, Command};
    let mut path = PathBuilder::new();
    for command in commands(x, y, fs) {
        match command {
            Command::Move((x, y)) => {
                path.move_to((x as f32, y as f32));
            }
            Command::Line((x, y)) => {
                path.line_to((x as f32, y as f32));
            }
            Command::Cubic([(a, b), (c, d), (e, f)]) => {
                path.cubic_to(
                    (a as f32, b as f32),
                    (c as f32, d as f32),
                    (e as f32, f as f32),
                );
            }
            Command::Close => {
                path.close();
            }
        }
    }
    canvas.draw_path(&path.detach(), &fill_paint(color));
}

fn fill_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(color);
    paint
}

fn stroke_paint(color: Color, width: f64) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Stroke);
    paint.set_stroke_width(width.max(0.5) as f32);
    paint.set_color(color);
    paint
}

fn colorref_to_skia(color: u32, alpha_scale: f32) -> Color {
    let b = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let a = (255.0 * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_argb(a, r, g, b)
}
