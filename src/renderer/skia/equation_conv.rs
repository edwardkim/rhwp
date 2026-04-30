use skia_safe::{font, paint, Canvas, Color, Font, FontMgr, FontStyle, Paint, PathBuilder};

use crate::renderer::equation::ast::MatrixStyle;
use crate::renderer::equation::layout::{
    is_integral_symbol, LayoutBox, LayoutKind, AXIS_HEIGHT, BIG_OP_SCALE, SCRIPT_SCALE,
};
use crate::renderer::equation::symbols::{DecoKind, FontStyleKind};

const EQ_FONT_FAMILY: &str =
    "Latin Modern Math, STIX Two Math, Cambria Math, DejaVu Sans, Times New Roman, serif";

pub fn render_equation(
    canvas: &Canvas,
    font_mgr: &FontMgr,
    layout: &LayoutBox,
    origin_x: f64,
    origin_y: f64,
    color: u32,
    base_font_size: f64,
) {
    render_box(
        canvas,
        font_mgr,
        layout,
        origin_x,
        origin_y,
        colorref_to_skia(color, 1.0),
        base_font_size,
        false,
        false,
    );
}

fn render_box(
    canvas: &Canvas,
    font_mgr: &FontMgr,
    lb: &LayoutBox,
    parent_x: f64,
    parent_y: f64,
    color: Color,
    fs: f64,
    italic: bool,
    bold: bool,
) {
    let x = parent_x + lb.x;
    let y = parent_y + lb.y;

    match &lb.kind {
        LayoutKind::Row(children) => {
            for child in children {
                render_box(canvas, font_mgr, child, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Text(text) => {
            draw_text(
                canvas,
                font_mgr,
                text,
                x,
                y + lb.baseline,
                font_size_from_box(lb, fs),
                true,
                bold,
                color,
                false,
            );
        }
        LayoutKind::Number(text) => {
            draw_text(
                canvas,
                font_mgr,
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
                font_mgr,
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
        LayoutKind::MathSymbol(text) => {
            let font_size = if is_integral_symbol(text) {
                lb.height
            } else {
                font_size_from_box(lb, fs)
            };
            draw_text(
                canvas,
                font_mgr,
                text,
                x,
                y + lb.baseline,
                font_size,
                false,
                false,
                color,
                false,
            );
        }
        LayoutKind::Function(name) => {
            draw_text(
                canvas,
                font_mgr,
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
        LayoutKind::Fraction { numer, denom } => {
            render_box(canvas, font_mgr, numer, x, y, color, fs, italic, bold);
            let line_y = y + lb.baseline - fs * AXIS_HEIGHT;
            canvas.draw_line(
                ((x + fs * 0.05) as f32, line_y as f32),
                ((x + lb.width - fs * 0.05) as f32, line_y as f32),
                &stroke_paint(color, fs * 0.04),
            );
            render_box(canvas, font_mgr, denom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Atop { top, bottom } => {
            render_box(canvas, font_mgr, top, x, y, color, fs, italic, bold);
            render_box(canvas, font_mgr, bottom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Sqrt { index, body } => {
            let sign_h = lb.height;
            let body_left = x + body.x - fs * 0.1;
            let sign_x = x;
            let v_top = y;
            let v_mid_x = body_left - fs * 0.15;
            let v_mid_y = y + sign_h;
            let v_start_x = v_mid_x - fs * 0.3;
            let v_start_y = y + sign_h * 0.6;
            let tick_x = v_start_x - fs * 0.1;
            let tick_y = v_start_y - fs * 0.05;

            let mut path = PathBuilder::new();
            path.move_to((tick_x as f32, tick_y as f32));
            path.line_to((v_start_x as f32, v_start_y as f32));
            path.line_to((v_mid_x as f32, v_mid_y as f32));
            path.line_to((body_left as f32, v_top as f32));
            path.line_to(((x + lb.width) as f32, v_top as f32));
            canvas.draw_path(&path.detach(), &stroke_paint(color, fs * 0.04));

            if let Some(index) = index {
                render_box(
                    canvas,
                    font_mgr,
                    index,
                    sign_x,
                    y,
                    color,
                    fs * SCRIPT_SCALE,
                    false,
                    false,
                );
            }
            render_box(canvas, font_mgr, body, x, y, color, fs, italic, bold);
        }
        LayoutKind::Superscript { base, sup } => {
            render_box(canvas, font_mgr, base, x, y, color, fs, italic, bold);
            render_box(
                canvas,
                font_mgr,
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
            render_box(canvas, font_mgr, base, x, y, color, fs, italic, bold);
            render_box(
                canvas,
                font_mgr,
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
            render_box(canvas, font_mgr, base, x, y, color, fs, italic, bold);
            render_box(
                canvas,
                font_mgr,
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
                font_mgr,
                sup,
                x,
                y,
                color,
                fs * SCRIPT_SCALE,
                italic,
                bold,
            );
        }
        LayoutKind::BigOp { symbol, sub, sup } => {
            let op_fs = fs * BIG_OP_SCALE;
            let (op_x, op_y) = if is_integral_symbol(symbol) {
                (x, y + op_fs * 0.8)
            } else {
                let sup_h = sup.as_ref().map(|b| b.height + fs * 0.05).unwrap_or(0.0);
                (
                    x + (lb.width - estimate_op_width(symbol, op_fs)) / 2.0,
                    y + sup_h + op_fs * 0.8,
                )
            };
            draw_text(
                canvas, font_mgr, symbol, op_x, op_y, op_fs, false, false, color, false,
            );
            if let Some(sup) = sup {
                render_box(
                    canvas,
                    font_mgr,
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
                    font_mgr,
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
        LayoutKind::Limit { is_upper, sub } => {
            let name = if *is_upper { "Lim" } else { "lim" };
            draw_text(
                canvas,
                font_mgr,
                name,
                x,
                y + fs * 0.8,
                fs,
                false,
                false,
                color,
                false,
            );
            if let Some(sub) = sub {
                render_box(
                    canvas,
                    font_mgr,
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
        LayoutKind::Matrix { cells, style } => {
            let bracket_chars = match style {
                MatrixStyle::Paren => ("(", ")"),
                MatrixStyle::Bracket => ("[", "]"),
                MatrixStyle::Vert => ("|", "|"),
                MatrixStyle::Plain => ("", ""),
            };
            if !bracket_chars.0.is_empty() {
                draw_stretch_bracket(
                    canvas,
                    font_mgr,
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
                    font_mgr,
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
                    render_box(canvas, font_mgr, cell, x, y, color, fs, italic, bold);
                }
            }
        }
        LayoutKind::Rel { arrow, over, under } => {
            render_box(canvas, font_mgr, over, x, y, color, fs, italic, bold);
            render_box(canvas, font_mgr, arrow, x, y, color, fs, italic, bold);
            if let Some(under) = under {
                render_box(canvas, font_mgr, under, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::EqAlign { rows } => {
            for (left, right) in rows {
                render_box(canvas, font_mgr, left, x, y, color, fs, italic, bold);
                render_box(canvas, font_mgr, right, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Paren { left, right, body } => {
            let paren_w = fs * 0.333;
            let use_glyph = lb.height <= fs * 1.2;
            if !left.is_empty() {
                if use_glyph && (left == "(" || left == ")") {
                    draw_text(
                        canvas,
                        font_mgr,
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
                        canvas, font_mgr, left, x, y, paren_w, lb.height, color, fs,
                    );
                }
            }
            render_box(canvas, font_mgr, body, x, y, color, fs, italic, bold);
            if !right.is_empty() {
                let right_x = x + lb.width - paren_w;
                if use_glyph && (right == "(" || right == ")") {
                    draw_text(
                        canvas,
                        font_mgr,
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
                        canvas, font_mgr, right, right_x, y, paren_w, lb.height, color, fs,
                    );
                }
            }
        }
        LayoutKind::Decoration { kind, body } => {
            render_box(canvas, font_mgr, body, x, y, color, fs, italic, bold);
            let deco_y = y + fs * 0.05;
            let mid_x = x + body.x + body.width / 2.0;
            draw_decoration(canvas, *kind, mid_x, deco_y, body.width, color, fs);
        }
        LayoutKind::FontStyle { style, body } => {
            let (new_italic, new_bold) = match style {
                FontStyleKind::Roman => (false, false),
                FontStyleKind::Italic => (true, bold),
                FontStyleKind::Bold => (italic, true),
            };
            render_box(
                canvas, font_mgr, body, x, y, color, fs, new_italic, new_bold,
            );
        }
        LayoutKind::Space(_) | LayoutKind::Newline | LayoutKind::Empty => {}
    }
}

fn draw_text(
    canvas: &Canvas,
    font_mgr: &FontMgr,
    text: &str,
    x: f64,
    baseline_y: f64,
    font_size: f64,
    italic: bool,
    bold: bool,
    color: Color,
    centered: bool,
) {
    if text.is_empty() {
        return;
    }
    let font_style = match (bold, italic) {
        (true, true) => FontStyle::bold_italic(),
        (true, false) => FontStyle::bold(),
        (false, true) => FontStyle::italic(),
        (false, false) => FontStyle::normal(),
    };
    let typeface = EQ_FONT_FAMILY
        .split(',')
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .find_map(|family| font_mgr.match_family_style(family, font_style))
        .or_else(|| font_mgr.legacy_make_typeface(None::<&str>, font_style));
    let mut font = if let Some(typeface) = typeface {
        Font::new(typeface, font_size as f32)
    } else {
        let mut font = Font::default();
        font.set_size(font_size as f32);
        font
    };
    font.set_edging(font::Edging::AntiAlias);

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(color);

    let draw_x = if centered {
        let (width, _) = font.measure_str(text, Some(&paint));
        x - f64::from(width) / 2.0
    } else {
        x
    };
    canvas.draw_str(text, (draw_x as f32, baseline_y as f32), &font, &paint);
}

fn draw_stretch_bracket(
    canvas: &Canvas,
    font_mgr: &FontMgr,
    bracket: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: Color,
    fs: f64,
) {
    let mid_x = x + w / 2.0;
    let paint = stroke_paint(color, fs * 0.04);

    match bracket {
        "(" => {
            let mut path = PathBuilder::new();
            path.move_to(((mid_x + w * 0.2) as f32, y as f32));
            path.quad_to(
                (x as f32, (y + h / 2.0) as f32),
                ((mid_x + w * 0.2) as f32, (y + h) as f32),
            );
            canvas.draw_path(&path.detach(), &paint);
        }
        ")" => {
            let mut path = PathBuilder::new();
            path.move_to(((mid_x - w * 0.2) as f32, y as f32));
            path.quad_to(
                ((x + w) as f32, (y + h / 2.0) as f32),
                ((mid_x - w * 0.2) as f32, (y + h) as f32),
            );
            canvas.draw_path(&path.detach(), &paint);
        }
        "[" => {
            let mut path = PathBuilder::new();
            path.move_to(((mid_x + w * 0.2) as f32, y as f32));
            path.line_to(((mid_x - w * 0.2) as f32, y as f32));
            path.line_to(((mid_x - w * 0.2) as f32, (y + h) as f32));
            path.line_to(((mid_x + w * 0.2) as f32, (y + h) as f32));
            canvas.draw_path(&path.detach(), &paint);
        }
        "]" => {
            let mut path = PathBuilder::new();
            path.move_to(((mid_x - w * 0.2) as f32, y as f32));
            path.line_to(((mid_x + w * 0.2) as f32, y as f32));
            path.line_to(((mid_x + w * 0.2) as f32, (y + h) as f32));
            path.line_to(((mid_x - w * 0.2) as f32, (y + h) as f32));
            canvas.draw_path(&path.detach(), &paint);
        }
        "{" => {
            let qh = h / 4.0;
            let mut path = PathBuilder::new();
            path.move_to(((mid_x + w * 0.2) as f32, y as f32));
            path.quad_to(
                ((mid_x - w * 0.1) as f32, y as f32),
                ((mid_x - w * 0.1) as f32, (y + qh) as f32),
            );
            path.quad_to(
                ((mid_x - w * 0.1) as f32, (y + qh * 2.0) as f32),
                ((mid_x - w * 0.3) as f32, (y + qh * 2.0) as f32),
            );
            path.quad_to(
                ((mid_x - w * 0.1) as f32, (y + qh * 2.0) as f32),
                ((mid_x - w * 0.1) as f32, (y + qh * 3.0) as f32),
            );
            path.quad_to(
                ((mid_x - w * 0.1) as f32, (y + h) as f32),
                ((mid_x + w * 0.2) as f32, (y + h) as f32),
            );
            canvas.draw_path(&path.detach(), &paint);
        }
        "}" => {
            let qh = h / 4.0;
            let mut path = PathBuilder::new();
            path.move_to(((mid_x - w * 0.2) as f32, y as f32));
            path.quad_to(
                ((mid_x + w * 0.1) as f32, y as f32),
                ((mid_x + w * 0.1) as f32, (y + qh) as f32),
            );
            path.quad_to(
                ((mid_x + w * 0.1) as f32, (y + qh * 2.0) as f32),
                ((mid_x + w * 0.3) as f32, (y + qh * 2.0) as f32),
            );
            path.quad_to(
                ((mid_x + w * 0.1) as f32, (y + qh * 2.0) as f32),
                ((mid_x + w * 0.1) as f32, (y + qh * 3.0) as f32),
            );
            path.quad_to(
                ((mid_x + w * 0.1) as f32, (y + h) as f32),
                ((mid_x - w * 0.2) as f32, (y + h) as f32),
            );
            canvas.draw_path(&path.detach(), &paint);
        }
        "|" => {
            canvas.draw_line(
                (mid_x as f32, y as f32),
                (mid_x as f32, (y + h) as f32),
                &paint,
            );
        }
        _ => {
            draw_text(
                canvas,
                font_mgr,
                bracket,
                mid_x,
                y + h * 0.7,
                h,
                false,
                false,
                color,
                true,
            );
        }
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
        DecoKind::Vec => {
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
    if lb.height > 0.0 {
        lb.height
    } else {
        base_fs
    }
}

fn estimate_op_width(text: &str, fs: f64) -> f64 {
    text.chars().count() as f64 * fs * 0.6
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
