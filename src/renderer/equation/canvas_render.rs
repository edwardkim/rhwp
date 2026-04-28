//! 수식 Canvas 렌더러
//!
//! LayoutBox를 HTML5 Canvas 2D API로 직접 렌더링한다.
//! WASM 환경에서만 컴파일된다.

use web_sys::CanvasRenderingContext2d;
use super::layout::*;
use super::symbols::{DecoKind, FontStyleKind};
use super::ast::MatrixStyle;

/// 수식을 Canvas에 렌더링
pub fn render_equation_canvas(
    ctx: &CanvasRenderingContext2d,
    layout: &LayoutBox,
    origin_x: f64,
    origin_y: f64,
    color: &str,
    base_font_size: f64,
) {
    render_box(ctx, layout, origin_x, origin_y, color, base_font_size, false, false);
}

fn render_box(
    ctx: &CanvasRenderingContext2d,
    lb: &LayoutBox,
    parent_x: f64,
    parent_y: f64,
    color: &str,
    fs: f64,
    italic: bool,
    bold: bool,
) {
    let x = parent_x + lb.x;
    let y = parent_y + lb.y;

    match &lb.kind {
        LayoutKind::Row(children) => {
            for child in children {
                render_box(ctx, child, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Text(text) => {
            let fi = font_size_from_box(lb, fs);
            let has_cjk = text.chars().any(|c| matches!(c,
                '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}'
            ));
            set_font(ctx, fi, !has_cjk, bold);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(text, x, y + lb.baseline);
        }
        LayoutKind::Number(text) => {
            let fi = font_size_from_box(lb, fs);
            set_font(ctx, fi, false, bold);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(text, x, y + lb.baseline);
        }
        LayoutKind::Symbol(text) => {
            let fi = font_size_from_box(lb, fs);
            set_font(ctx, fi, false, false);
            ctx.set_fill_style_str(color);
            ctx.set_text_align("center");
            let _ = ctx.fill_text(text, x + lb.width / 2.0, y + lb.baseline);
            ctx.set_text_align("start");
        }
        LayoutKind::MathSymbol(text) => {
            let fi = font_size_from_box(lb, fs);
            set_font(ctx, fi, false, false);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(text, x, y + lb.baseline);
        }
        LayoutKind::Function(name) => {
            let fi = font_size_from_box(lb, fs);
            set_font(ctx, fi, false, false);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(name, x, y + lb.baseline);
        }
        LayoutKind::Fraction { numer, denom } => {
            render_box(ctx, numer, x, y, color, fs, italic, bold);
            // 분수선 — baseline에서 axis_height 위에 배치 (SVG 경로와 동일)
            let line_y = y + lb.baseline - fs * super::layout::AXIS_HEIGHT;
            let line_thick = fs * 0.04;
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(line_thick);
            ctx.begin_path();
            ctx.move_to(x + fs * 0.05, line_y);
            ctx.line_to(x + lb.width - fs * 0.05, line_y);
            ctx.stroke();
            render_box(ctx, denom, x, y, color, fs, italic, bold);
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

            ctx.set_stroke_style_str(color);
            ctx.set_line_width(fs * 0.04);
            ctx.begin_path();
            ctx.move_to(tick_x, tick_y);
            ctx.line_to(v_start_x, v_start_y);
            ctx.line_to(v_mid_x, v_mid_y);
            ctx.line_to(body_left, v_top);
            ctx.line_to(x + lb.width, v_top);
            ctx.stroke();

            if let Some(idx) = index {
                render_box(ctx, idx, sign_x, y, color, fs * SCRIPT_SCALE, false, false);
            }
            render_box(ctx, body, x, y, color, fs, italic, bold);
        }
        LayoutKind::Superscript { base, sup } => {
            render_box(ctx, base, x, y, color, fs, italic, bold);
            render_box(ctx, sup, x, y, color, fs * SCRIPT_SCALE, italic, bold);
        }
        LayoutKind::Subscript { base, sub } => {
            render_box(ctx, base, x, y, color, fs, italic, bold);
            render_box(ctx, sub, x, y, color, fs * SCRIPT_SCALE, italic, bold);
        }
        LayoutKind::SubSup { base, sub, sup } => {
            render_box(ctx, base, x, y, color, fs, italic, bold);
            render_box(ctx, sub, x, y, color, fs * SCRIPT_SCALE, italic, bold);
            render_box(ctx, sup, x, y, color, fs * SCRIPT_SCALE, italic, bold);
        }
        LayoutKind::BigOp { symbol, sub, sup } => {
            let op_fs = fs * BIG_OP_SCALE;
            let sup_h = sup.as_ref().map(|b| b.height + fs * 0.05).unwrap_or(0.0);
            let op_x = x + (lb.width - estimate_op_width(symbol, op_fs)) / 2.0;
            let op_y = y + sup_h + op_fs * 0.8;
            set_font(ctx, op_fs, false, false);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(symbol, op_x, op_y);
            if let Some(sup_box) = sup {
                render_box(ctx, sup_box, x, y, color, fs * SCRIPT_SCALE, false, false);
            }
            if let Some(sub_box) = sub {
                render_box(ctx, sub_box, x, y, color, fs * SCRIPT_SCALE, false, false);
            }
        }
        LayoutKind::Limit { is_upper, sub } => {
            // [Task PR #396 후속] SVG 경로 (svg_render.rs::Limit) 와 동일하게 base font_size 사용.
            // font_size_from_box(lb, fs) 는 lb.height 를 사용하는데, Limit 의 lb 는 "lim + 첨자"
            // 전체 높이라 base 의 1.5~2 배가 되어 lim 글자가 비정상으로 커지는 정황.
            let name = if *is_upper { "Lim" } else { "lim" };
            let fi = fs;
            set_font(ctx, fi, false, false);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(name, x, y + fi * 0.8);
            if let Some(sub_box) = sub {
                render_box(ctx, sub_box, x, y, color, fs * SCRIPT_SCALE, false, false);
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
                draw_stretch_bracket(ctx, bracket_chars.0, x, y, fs * 0.3, lb.height, color, fs);
                draw_stretch_bracket(ctx, bracket_chars.1, x + lb.width - fs * 0.3, y, fs * 0.3, lb.height, color, fs);
            }
            for row in cells {
                for cell in row {
                    render_box(ctx, cell, x, y, color, fs, italic, bold);
                }
            }
        }
        LayoutKind::Rel { arrow, over, under } => {
            render_box(ctx, over, x, y, color, fs, italic, bold);
            render_box(ctx, arrow, x, y, color, fs, italic, bold);
            if let Some(u) = under {
                render_box(ctx, u, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::EqAlign { rows } => {
            for (left, right) in rows {
                render_box(ctx, left, x, y, color, fs, italic, bold);
                render_box(ctx, right, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Paren { left, right, body } => {
            // 텍스트 높이 파렌(`(`, `)`)은 폰트 글리프로 렌더, 그 외는 path. (Task #283)
            let paren_w = fs * 0.333;
            let use_glyph = lb.height <= fs * 1.2;
            if !left.is_empty() {
                if use_glyph && (left == "(" || left == ")") {
                    set_font(ctx, fs, false, false);
                    ctx.set_fill_style_str(color);
                    let _ = ctx.fill_text(left, x, y + lb.baseline);
                } else {
                    draw_stretch_bracket(ctx, left, x, y, paren_w, lb.height, color, fs);
                }
            }
            render_box(ctx, body, x, y, color, fs, italic, bold);
            if !right.is_empty() {
                let right_x = x + lb.width - paren_w;
                if use_glyph && (right == "(" || right == ")") {
                    set_font(ctx, fs, false, false);
                    ctx.set_fill_style_str(color);
                    let _ = ctx.fill_text(right, right_x, y + lb.baseline);
                } else {
                    draw_stretch_bracket(ctx, right, right_x, y, paren_w, lb.height, color, fs);
                }
            }
        }
        LayoutKind::Decoration { kind, body } => {
            render_box(ctx, body, x, y, color, fs, italic, bold);
            let deco_y = y + fs * 0.05;
            let mid_x = x + body.x + body.width / 2.0;
            draw_decoration(ctx, *kind, mid_x, deco_y, body.width, color, fs);
        }
        LayoutKind::FontStyle { style, body } => {
            let (new_italic, new_bold) = match style {
                FontStyleKind::Roman => (false, false),
                FontStyleKind::Italic => (true, bold),
                FontStyleKind::Bold => (italic, true),
            };
            render_box(ctx, body, x, y, color, fs, new_italic, new_bold);
        }
        LayoutKind::Space(_) | LayoutKind::Newline | LayoutKind::Empty => {}
    }
}

fn font_size_from_box(lb: &LayoutBox, base_fs: f64) -> f64 {
    if lb.height > 0.0 { lb.height } else { base_fs }
}

fn estimate_op_width(text: &str, fs: f64) -> f64 {
    text.chars().count() as f64 * fs * 0.6
}

fn set_font(ctx: &CanvasRenderingContext2d, size: f64, italic: bool, bold: bool) {
    let style = if italic { "italic " } else { "" };
    let weight = if bold { "bold " } else { "" };
    // svg_render.rs 의 EQ_FONT_FAMILY 와 동일 스택 유지. (Task #280)
    ctx.set_font(&format!(
        "{}{}{:.1}px 'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif",
        style, weight, size,
    ));
}

/// 늘림 괄호 렌더링
fn draw_stretch_bracket(
    ctx: &CanvasRenderingContext2d,
    bracket: &str, x: f64, y: f64, w: f64, h: f64, color: &str, fs: f64,
) {
    let mid_x = x + w / 2.0;
    let stroke_w = fs * 0.04;
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(stroke_w);

    match bracket {
        "(" => {
            ctx.begin_path();
            ctx.move_to(mid_x + w * 0.2, y);
            let _ = ctx.quadratic_curve_to(x, y + h / 2.0, mid_x + w * 0.2, y + h);
            ctx.stroke();
        }
        ")" => {
            ctx.begin_path();
            ctx.move_to(mid_x - w * 0.2, y);
            let _ = ctx.quadratic_curve_to(x + w, y + h / 2.0, mid_x - w * 0.2, y + h);
            ctx.stroke();
        }
        "[" => {
            ctx.begin_path();
            ctx.move_to(mid_x + w * 0.2, y);
            ctx.line_to(mid_x - w * 0.2, y);
            ctx.line_to(mid_x - w * 0.2, y + h);
            ctx.line_to(mid_x + w * 0.2, y + h);
            ctx.stroke();
        }
        "]" => {
            ctx.begin_path();
            ctx.move_to(mid_x - w * 0.2, y);
            ctx.line_to(mid_x + w * 0.2, y);
            ctx.line_to(mid_x + w * 0.2, y + h);
            ctx.line_to(mid_x - w * 0.2, y + h);
            ctx.stroke();
        }
        "{" => {
            let qh = h / 4.0;
            ctx.begin_path();
            ctx.move_to(mid_x + w * 0.2, y);
            let _ = ctx.quadratic_curve_to(mid_x - w * 0.1, y, mid_x - w * 0.1, y + qh);
            let _ = ctx.quadratic_curve_to(mid_x - w * 0.1, y + qh * 2.0, mid_x - w * 0.3, y + qh * 2.0);
            let _ = ctx.quadratic_curve_to(mid_x - w * 0.1, y + qh * 2.0, mid_x - w * 0.1, y + qh * 3.0);
            let _ = ctx.quadratic_curve_to(mid_x - w * 0.1, y + h, mid_x + w * 0.2, y + h);
            ctx.stroke();
        }
        "}" => {
            let qh = h / 4.0;
            ctx.begin_path();
            ctx.move_to(mid_x - w * 0.2, y);
            let _ = ctx.quadratic_curve_to(mid_x + w * 0.1, y, mid_x + w * 0.1, y + qh);
            let _ = ctx.quadratic_curve_to(mid_x + w * 0.1, y + qh * 2.0, mid_x + w * 0.3, y + qh * 2.0);
            let _ = ctx.quadratic_curve_to(mid_x + w * 0.1, y + qh * 2.0, mid_x + w * 0.1, y + qh * 3.0);
            let _ = ctx.quadratic_curve_to(mid_x + w * 0.1, y + h, mid_x - w * 0.2, y + h);
            ctx.stroke();
        }
        "|" => {
            ctx.begin_path();
            ctx.move_to(mid_x, y);
            ctx.line_to(mid_x, y + h);
            ctx.stroke();
        }
        _ => {
            // 기타 괄호: 텍스트로 렌더링
            set_font(ctx, h, false, false);
            ctx.set_fill_style_str(color);
            ctx.set_text_align("center");
            let _ = ctx.fill_text(bracket, mid_x, y + h * 0.7);
            ctx.set_text_align("start");
        }
    }
}

/// 장식 렌더링
fn draw_decoration(
    ctx: &CanvasRenderingContext2d,
    kind: DecoKind, mid_x: f64, y: f64, width: f64, color: &str, fs: f64,
) {
    let stroke_w = fs * 0.03;
    let half_w = width / 2.0;
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(stroke_w);

    match kind {
        DecoKind::Hat => {
            ctx.begin_path();
            ctx.move_to(mid_x - half_w * 0.6, y + fs * 0.15);
            ctx.line_to(mid_x, y);
            ctx.line_to(mid_x + half_w * 0.6, y + fs * 0.15);
            ctx.stroke();
        }
        DecoKind::Bar | DecoKind::Overline => {
            ctx.begin_path();
            ctx.move_to(mid_x - half_w, y + fs * 0.05);
            ctx.line_to(mid_x + half_w, y + fs * 0.05);
            ctx.stroke();
        }
        DecoKind::Vec => {
            let arrow_y = y + fs * 0.05;
            ctx.begin_path();
            ctx.move_to(mid_x - half_w, arrow_y);
            ctx.line_to(mid_x + half_w, arrow_y);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(mid_x + half_w - fs * 0.1, arrow_y - fs * 0.06);
            ctx.line_to(mid_x + half_w, arrow_y);
            ctx.line_to(mid_x + half_w - fs * 0.1, arrow_y + fs * 0.06);
            ctx.stroke();
        }
        DecoKind::Tilde => {
            let ty = y + fs * 0.08;
            ctx.begin_path();
            ctx.move_to(mid_x - half_w * 0.6, ty);
            let _ = ctx.quadratic_curve_to(mid_x - half_w * 0.2, ty - fs * 0.08, mid_x, ty);
            let _ = ctx.quadratic_curve_to(mid_x + half_w * 0.2, ty + fs * 0.08, mid_x + half_w * 0.6, ty);
            ctx.stroke();
        }
        DecoKind::Dot => {
            ctx.set_fill_style_str(color);
            ctx.begin_path();
            let _ = ctx.arc(mid_x, y + fs * 0.06, fs * 0.03, 0.0, std::f64::consts::TAU);
            ctx.fill();
        }
        DecoKind::DDot => {
            let gap = fs * 0.1;
            ctx.set_fill_style_str(color);
            ctx.begin_path();
            let _ = ctx.arc(mid_x - gap, y + fs * 0.06, fs * 0.03, 0.0, std::f64::consts::TAU);
            ctx.fill();
            ctx.begin_path();
            let _ = ctx.arc(mid_x + gap, y + fs * 0.06, fs * 0.03, 0.0, std::f64::consts::TAU);
            ctx.fill();
        }
        DecoKind::Underline | DecoKind::Under => {
            let uy = y + fs * 1.1;
            ctx.begin_path();
            ctx.move_to(mid_x - half_w, uy);
            ctx.line_to(mid_x + half_w, uy);
            ctx.stroke();
        }
        _ => {
            ctx.begin_path();
            ctx.move_to(mid_x - half_w * 0.5, y + fs * 0.1);
            ctx.line_to(mid_x + half_w * 0.5, y + fs * 0.1);
            ctx.stroke();
        }
    }
}
