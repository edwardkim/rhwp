//! 수식 Canvas 렌더러
//!
//! LayoutBox를 HTML5 Canvas 2D API로 직접 렌더링한다.
//! WASM 환경에서만 컴파일된다.

use super::ast::MatrixStyle;
use super::layout::*;
use super::symbols::{DecoKind, FontStyleKind};
use web_sys::CanvasRenderingContext2d;

/// 수식을 Canvas에 렌더링
pub fn render_equation_canvas(
    ctx: &CanvasRenderingContext2d,
    layout: &LayoutBox,
    origin_x: f64,
    origin_y: f64,
    color: &str,
    base_font_size: f64,
) {
    // 진입점 default: italic=true (hwpeq 변수 기본 스타일).
    // FontStyle::Roman(`rm`) 적용 영역에서는 자식 렌더링 시 italic=false 로 전환된다.
    render_box(
        ctx,
        layout,
        origin_x,
        origin_y,
        color,
        base_font_size,
        true,
        false,
    );
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
    let fs = if lb.font_size > 0.0 { lb.font_size } else { fs };
    let x = parent_x + lb.x;
    let y = parent_y + lb.y;

    match &lb.kind {
        LayoutKind::HancomGlyphs(run) => {
            ctx.set_font(&format!("{:.2}px {}", fs, run.font_family));
            ctx.set_fill_style_str(color);
            for ((glyph, offset), shear) in run.glyphs.chars().zip(&run.positions).zip(&run.shears)
            {
                ctx.save();
                let _ = ctx.transform(
                    1.0,
                    0.0,
                    -shear,
                    1.0,
                    x + offset,
                    y + lb.baseline + run.baseline_shift,
                );
                let _ = ctx.fill_text(&glyph.to_string(), 0.0, 0.0);
                ctx.restore();
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                render_box(ctx, child, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Text(text) => {
            // [Issue #900] SVG 경로 (svg_render.rs:43, commit 5a6f9a87 / Task #142) 와
            // 동기화 — font_size_from_box(lb.height) 는 복합 박스 (Limit, BigOp 등)
            // 의 전체 높이를 font-size 로 오용. 부모 전달 fs 사용.
            let fi = fs;
            // CJK 문자는 italic 미적용. FontStyle::Roman(`rm`)으로 italic=false 가
            // 전달된 경우에도 italic 미적용 (svg_render.rs Text arm 과 동일 정책).
            let has_cjk = super::text_has_cjk(text);
            if has_cjk {
                ctx.set_font(&format!(
                    "{}{:.1}px {}",
                    if bold { "bold " } else { "" },
                    fi,
                    super::CJK_EQUATION_FONT_FAMILY
                ));
            } else {
                let styled = is_styled_variable(text);
                set_font(ctx, fi, italic && !styled, bold && !styled);
            }
            ctx.set_fill_style_str(color);
            if has_cjk && (lb.width - estimate_text_width(text, fi, false)).abs() > 0.01 {
                for (ch, offset) in super::positioned_cjk_text(text, fi, lb.width) {
                    let _ = ctx.fill_text(&ch.to_string(), x + offset, y + lb.baseline);
                }
            } else {
                let _ = ctx.fill_text(text, x, y + lb.baseline);
            }
        }
        LayoutKind::Number(text) => {
            // [Issue #900] svg_render.rs Number arm 과 동기화 — fs 사용.
            let fi = fs;
            set_font(ctx, fi, false, bold);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(text, x, y + lb.baseline);
        }
        LayoutKind::Symbol(text) => {
            // [Issue #900] svg_render.rs Symbol arm 과 동기화 — fs 사용.
            let fi = fs;
            set_font(ctx, fi, false, false);
            ctx.set_fill_style_str(color);
            ctx.set_text_align("center");
            let _ = ctx.fill_text(text, x + lb.width / 2.0, y + lb.baseline);
            ctx.set_text_align("start");
        }
        LayoutKind::FontGlyph {
            text,
            font_family,
            outline,
            paint,
        } => {
            if let Some(commands) = outline {
                ctx.save();
                let _ = ctx.translate(x, y);
                if let Some(p) = paint {
                    let _ = ctx.scale(p.scale_x, 1.0);
                }
                ctx.set_fill_style_str(color);
                ctx.begin_path();
                for command in commands {
                    use super::GlyphOutlineCommand as Command;
                    match *command {
                        Command::Move((a, b)) => ctx.move_to(a, b),
                        Command::Line((a, b)) => ctx.line_to(a, b),
                        Command::Cubic([(a, b), (c, d), (e, f)]) => {
                            ctx.bezier_curve_to(a, b, c, d, e, f)
                        }
                        Command::Close => ctx.close_path(),
                    }
                }
                ctx.fill();
                if let Some(p) = paint {
                    ctx.set_stroke_style_str(color);
                    ctx.set_line_width(p.stroke_width);
                    ctx.set_line_join("miter");
                    ctx.set_miter_limit(10.0);
                    ctx.stroke();
                }
                ctx.restore();
                return;
            }
            ctx.set_font(&format!("{fs:.1}px {font_family}"));
            ctx.set_fill_style_str(color);
            ctx.set_text_align("start");
            let _ = ctx.fill_text(text, x, y + lb.baseline);
        }
        LayoutKind::MathSymbol(text) => {
            // [Issue #900] svg_render.rs MathSymbol arm 과 동기화 (commit 292dbbef).
            // Task #1317: 적분 기호(∫)는 폰트 text 가 아닌 stroke path 로 렌더(geom SSOT,
            // svg_render.rs 와 동일). 그 외 MathSymbol 은 부모 전달 fs 로 text 렌더.
            if super::layout::is_integral_symbol(text) {
                draw_integral(ctx, x, y, fs, color);
            } else {
                set_font(ctx, fs, false, false);
                ctx.set_fill_style_str(color);
                let _ = ctx.fill_text(text, x, y + lb.baseline);
            }
        }
        LayoutKind::Function(name) => {
            // [Issue #900] svg_render.rs Function arm 과 동기화 — fs 사용.
            let fi = fs;
            set_font(ctx, fi, false, false);
            ctx.set_fill_style_str(color);
            let _ = ctx.fill_text(name, x, y + lb.baseline);
        }
        LayoutKind::Fraction {
            numer,
            denom,
            axis_height,
            rule_thickness,
            rule_start,
            rule_end,
        } => {
            render_box(ctx, numer, x, y, color, fs, italic, bold);
            // 분수선 — baseline에서 axis_height 위에 배치 (SVG 경로와 동일)
            let line_y = y + lb.baseline - axis_height;
            let line_thick = *rule_thickness;
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(line_thick);
            ctx.begin_path();
            ctx.move_to(x + rule_start, line_y);
            ctx.line_to(x + rule_end, line_y);
            ctx.stroke();
            render_box(ctx, denom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Atop { top, bottom } => {
            render_box(ctx, top, x, y, color, fs, italic, bold);
            render_box(ctx, bottom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Sqrt { index, body } => {
            let geometry = super::radical_outline::geometry(lb, body, fs);
            ctx.set_fill_style_str(color);
            ctx.begin_path();
            for command in &geometry.outline {
                use super::GlyphOutlineCommand as Command;
                match *command {
                    Command::Move((a, b)) => ctx.move_to(x + a, y + b),
                    Command::Line((a, b)) => ctx.line_to(x + a, y + b),
                    Command::Cubic([(a, b), (c, d), (e, f)]) => {
                        ctx.bezier_curve_to(x + a, y + b, x + c, y + d, x + e, y + f)
                    }
                    Command::Close => ctx.close_path(),
                }
            }
            ctx.fill();
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(geometry.rule_thickness);
            ctx.begin_path();
            ctx.move_to(x + geometry.bar_start.0, y + geometry.bar_start.1);
            ctx.line_to(x + geometry.bar_end.0, y + geometry.bar_end.1);
            ctx.stroke();

            if let Some(idx) = index {
                render_box(
                    ctx,
                    idx,
                    x,
                    y,
                    color,
                    fs * RADICAL_DEGREE_SCALE,
                    false,
                    false,
                );
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
            // [Issue #900] svg_render.rs BigOp arm 과 동기화 (commit 292dbbef).
            // 적분 (∫, ∮ 등): 기호 좌측, 첨자 우상단/우하단 (nolimits)
            // 그 외 (∑, ∏ 등): 기호 중앙, 첨자 위/아래 (limits)
            let is_integral = super::layout::is_integral_symbol(symbol);
            // 생산 단계에서 확정한 표시 글자 크기와 원점을 소비한다.
            let op_fs = *op_font_size;
            set_font(ctx, op_fs, false, false);
            ctx.set_fill_style_str(color);
            if is_integral {
                // Task #1317: 적분 기호는 stroke path 로 렌더(geom SSOT).
                draw_integral(ctx, x + op_x, y + op_y, fs, color);
            } else {
                let op_x = x + op_x;
                let op_y = y + op_baseline;
                let _ = ctx.fill_text(symbol, op_x, op_y);
            }
            // 위/아래 첨자 — LayoutBox 자식 좌표 사용 (적분/일반 공통)
            if let Some(sup_box) = sup {
                render_box(ctx, sup_box, x, y, color, fs * SCRIPT_SCALE, false, false);
            }
            if let Some(sub_box) = sub {
                render_box(ctx, sub_box, x, y, color, fs * SCRIPT_SCALE, false, false);
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
                render_box(ctx, name, x, y, color, name.height, false, false);
            } else {
                // 일반 글꼴은 첨자를 포함한 전체 상자 높이를 글꼴 크기로 쓰지 않는다.
                let name = if *is_upper { "Lim" } else { "lim" };
                set_font(ctx, *name_font_size, false, false);
                ctx.set_fill_style_str(color);
                let _ = ctx.fill_text(name, x, y + lb.baseline);
            }
            if let Some(sub_box) = sub {
                render_box(ctx, sub_box, x, y, color, *sub_font_size, false, false);
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
                draw_stretch_bracket(ctx, bracket_chars.0, x, y, fs * 0.3, lb.height, color, fs);
                draw_stretch_bracket(
                    ctx,
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
            // 텍스트 높이 파렌(`(`, `)`)은 폰트 글리프로 렌더, 그 외는 path. (Task #283)
            let use_glyph = *glyph_delimiters;
            if !left.is_empty() {
                if let Some(commands) = left_outline {
                    draw_delimiter_outline(ctx, commands, x, y, color);
                } else if use_glyph && (left == "(" || left == ")") {
                    set_font(ctx, fs, false, false);
                    ctx.set_fill_style_str(color);
                    let _ = ctx.fill_text(left, x, y + lb.baseline);
                } else {
                    draw_stretch_bracket(
                        ctx,
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
            render_box(ctx, body, x, y, color, fs, italic, bold);
            if !right.is_empty() {
                let right_x = x + lb.width - right_width;
                if let Some(commands) = right_outline {
                    draw_delimiter_outline(ctx, commands, right_x, y, color);
                } else if use_glyph && (right == "(" || right == ")") {
                    set_font(ctx, fs, false, false);
                    ctx.set_fill_style_str(color);
                    let _ = ctx.fill_text(right, right_x, y + lb.baseline);
                } else {
                    draw_stretch_bracket(
                        ctx,
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
            render_box(ctx, body, x, y, color, fs, italic, bold);
            if let Some(commands) = outline {
                draw_delimiter_outline(ctx, commands, x, y, color);
            } else if *kind == DecoKind::StrikeThrough {
                let g = negation
                    .clone()
                    .unwrap_or_else(|| super::negation_geometry::geometry(body));
                ctx.set_stroke_style_str(color);
                ctx.set_line_width(g.thickness);
                ctx.begin_path();
                ctx.move_to(x + g.start.0, y + g.start.1);
                ctx.line_to(x + g.end.0, y + g.end.1);
                ctx.stroke();
            } else if *kind == DecoKind::Box {
                ctx.set_stroke_style_str(color);
                ctx.set_line_width(fs * 0.03);
                ctx.stroke_rect(x, y, lb.width, lb.height);
            } else {
                let deco_y = y + fs * 0.05;
                let mid_x = x + body.x + body.width / 2.0;
                draw_decoration(ctx, *kind, mid_x, deco_y, body.width, color, fs);
            }
        }
        LayoutKind::Color { rgb, body } => {
            let body_color = format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);
            render_box(ctx, body, x, y, &body_color, fs, italic, bold);
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
            render_box(ctx, body, x, y, color, fs, new_italic, new_bold);
        }
        LayoutKind::Space(_) | LayoutKind::Newline | LayoutKind::Empty => {}
    }
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

/// 적분 기호(∫)를 stroke path 로 렌더 (Task #1317).
///
/// SVG·Skia와 같은 공개 적분 윤곽 및 공통 ink 영역을 소비한다.
fn draw_integral(ctx: &CanvasRenderingContext2d, x: f64, y: f64, fs: f64, color: &str) {
    use super::integral_outline::{commands, Command};
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    for command in commands(x, y, fs) {
        match command {
            Command::Move((x, y)) => ctx.move_to(x, y),
            Command::Line((x, y)) => ctx.line_to(x, y),
            Command::Cubic([(a, b), (c, d), (e, f)]) => ctx.bezier_curve_to(a, b, c, d, e, f),
            Command::Close => ctx.close_path(),
        }
    }
    ctx.fill();
}

/// 늘림 괄호 렌더링
fn draw_delimiter_outline(
    ctx: &CanvasRenderingContext2d,
    commands: &[super::GlyphOutlineCommand],
    x: f64,
    y: f64,
    color: &str,
) {
    use super::GlyphOutlineCommand as Command;
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    for command in commands {
        match *command {
            Command::Move((a, b)) => ctx.move_to(x + a, y + b),
            Command::Line((a, b)) => ctx.line_to(x + a, y + b),
            Command::Cubic([(a, b), (c, d), (e, f)]) => {
                ctx.bezier_curve_to(x + a, y + b, x + c, y + d, x + e, y + f)
            }
            Command::Close => ctx.close_path(),
        }
    }
    ctx.fill();
}

#[allow(clippy::too_many_arguments)]
fn draw_stretch_bracket(
    ctx: &CanvasRenderingContext2d,
    bracket: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: &str,
    fs: f64,
) {
    if matches!(bracket, "{" | "}") {
        use super::GlyphOutlineCommand as Command;
        ctx.set_fill_style_str(color);
        ctx.begin_path();
        for command in super::brace_stretch::commands(x, y, w, h, fs, bracket == "}") {
            match command {
                Command::Move((a, b)) => ctx.move_to(a, b),
                Command::Line((a, b)) => ctx.line_to(a, b),
                Command::Cubic([(a, b), (c, d), (e, f)]) => ctx.bezier_curve_to(a, b, c, d, e, f),
                Command::Close => ctx.close_path(),
            }
        }
        ctx.fill();
        return;
    }
    let mid_x = x + w / 2.0;
    let stroke_w = if matches!(bracket, "(" | ")") {
        (fs * 0.042).max(0.48)
    } else {
        fs * 0.04
    };
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(stroke_w);

    match bracket {
        "(" => {
            ctx.begin_path();
            ctx.set_line_cap("round");
            ctx.move_to(x + w * 0.9, y);
            ctx.bezier_curve_to(
                x + w * 0.05,
                y + h * 0.18,
                x + w * 0.05,
                y + h * 0.82,
                x + w * 0.9,
                y + h,
            );
            ctx.stroke();
            ctx.set_line_cap("butt");
        }
        ")" => {
            ctx.begin_path();
            ctx.set_line_cap("round");
            ctx.move_to(x + w * 0.1, y);
            ctx.bezier_curve_to(
                x + w * 0.95,
                y + h * 0.18,
                x + w * 0.95,
                y + h * 0.82,
                x + w * 0.1,
                y + h,
            );
            ctx.stroke();
            ctx.set_line_cap("butt");
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
    kind: DecoKind,
    mid_x: f64,
    y: f64,
    width: f64,
    color: &str,
    fs: f64,
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
        DecoKind::Vec | DecoKind::Dyad => {
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
            if kind == DecoKind::Dyad {
                ctx.begin_path();
                ctx.move_to(mid_x - half_w + fs * 0.1, arrow_y - fs * 0.06);
                ctx.line_to(mid_x - half_w, arrow_y);
                ctx.line_to(mid_x - half_w + fs * 0.1, arrow_y + fs * 0.06);
                ctx.stroke();
            }
        }
        DecoKind::Tilde => {
            let ty = y + fs * 0.08;
            ctx.begin_path();
            ctx.move_to(mid_x - half_w * 0.6, ty);
            ctx.quadratic_curve_to(mid_x - half_w * 0.2, ty - fs * 0.08, mid_x, ty);
            ctx.quadratic_curve_to(
                mid_x + half_w * 0.2,
                ty + fs * 0.08,
                mid_x + half_w * 0.6,
                ty,
            );
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
            let _ = ctx.arc(
                mid_x - gap,
                y + fs * 0.06,
                fs * 0.03,
                0.0,
                std::f64::consts::TAU,
            );
            ctx.fill();
            ctx.begin_path();
            let _ = ctx.arc(
                mid_x + gap,
                y + fs * 0.06,
                fs * 0.03,
                0.0,
                std::f64::consts::TAU,
            );
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
