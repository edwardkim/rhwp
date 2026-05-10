//! 수식 SVG 렌더러
//!
//! LayoutBox를 SVG 요소로 변환한다.
//! 생성된 SVG 조각은 `<g>` 요소 내부에 포함된다.

use super::layout::*;
use super::symbols::{DecoKind, FontStyleKind};
use super::ast::MatrixStyle;

/// 수식 전용 font-family
/// 순서: Latin Modern Math (LaTeX 설치 시) → STIX Two Text (Mac/STIX 설치 시) → STIX Two Math → Times New Roman (Windows 기본) → serif
/// Cambria Math 는 Windows 에서 "볼드 인상" 을 유발해 제외. Pretendard 는 산세리프라 수식 부적합으로 제외. (Task #280)
const EQ_FONT_FAMILY: &str = " font-family=\"'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif\"";

/// 수식을 SVG 조각 문자열로 렌더링
///
/// 진입점 default: italic=true (hwpeq 변수 기본 스타일). FontStyle::Roman(`rm`)
/// 적용 영역에서는 자식 렌더링 시 italic=false 로 전환된다.
pub fn render_equation_svg(layout: &LayoutBox, color: &str, base_font_size: f64) -> String {
    let mut svg = String::new();
    render_box(&mut svg, layout, 0.0, 0.0, color, base_font_size, true, false);
    svg
}

fn render_box(
    svg: &mut String,
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
                render_box(svg, child, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Text(text) => {
            let text_x = x;
            let text_y = y + lb.baseline;
            let esc = escape_xml(text);
            let fi = fs;
            // CJK/한글 텍스트는 이탤릭 없이 렌더링 (수학 변수명만 이탤릭).
            // FontStyle::Roman(`rm` 적용)으로 italic=false 가 전달된 경우에도 이탤릭을 적용하지 않는다.
            let has_cjk = text.chars().any(|c| matches!(c,
                '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}'
            ));
            let italic_attr = if !has_cjk && italic { " font-style=\"italic\"" } else { "" };
            let weight_attr = if bold { " font-weight=\"bold\"" } else { "" };
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}{}{}>{}</text>\n",
                text_x, text_y, fi, color, italic_attr, weight_attr, EQ_FONT_FAMILY, esc,
            ));
        }
        LayoutKind::Number(text) => {
            let text_x = x;
            let text_y = y + lb.baseline;
            let esc = escape_xml(text);
            let fi = fs;
            let style_attr = if bold { " font-weight=\"bold\"" } else { "" };
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}{}>{}</text>\n",
                text_x, text_y, fi, color, style_attr, EQ_FONT_FAMILY, esc,
            ));
        }
        LayoutKind::Symbol(text) => {
            let text_x = x + lb.width / 2.0;
            let text_y = y + lb.baseline;
            let esc = escape_xml(text);
            let fi = fs;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\" text-anchor=\"middle\"{}>{}</text>\n",
                text_x, text_y, fi, color, EQ_FONT_FAMILY, esc,
            ));
        }
        LayoutKind::MathSymbol(text) => {
            let text_x = x;
            let text_y = y + lb.baseline;
            let esc = escape_xml(text);
            // 적분 기호: layout에서 BIG_OP_SCALE이 적용된 높이를 font-size로 사용
            let fi = if super::layout::is_integral_symbol(text) { lb.height } else { fs };
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                text_x, text_y, fi, color, EQ_FONT_FAMILY, esc,
            ));
        }
        LayoutKind::Function(name) => {
            let text_x = x;
            let text_y = y + lb.baseline;
            let esc = escape_xml(name);
            let fi = fs;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                text_x, text_y, fi, color, EQ_FONT_FAMILY, esc,
            ));
        }
        LayoutKind::Fraction { numer, denom } => {
            // 분자
            render_box(svg, numer, x, y, color, fs, italic, bold);
            // 분수선 — baseline에서 axis_height 위에 배치
            let line_y = y + lb.baseline - fs * super::layout::AXIS_HEIGHT;
            let line_thick = fs * 0.04;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                x + fs * 0.05, line_y,
                x + lb.width - fs * 0.05, line_y,
                color, line_thick,
            ));
            // 분모
            render_box(svg, denom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Atop { top, bottom } => {
            render_box(svg, top, x, y, color, fs, italic, bold);
            render_box(svg, bottom, x, y, color, fs, italic, bold);
        }
        LayoutKind::Sqrt { index, body } => {
            // √ 기호
            let sign_h = lb.height;
            let body_left = x + body.x - fs * 0.1;
            let sign_x = x;
            // V 모양 경로
            let v_top = y;
            let v_mid_x = body_left - fs * 0.15;
            let v_mid_y = y + sign_h;
            let v_start_x = v_mid_x - fs * 0.3;
            let v_start_y = y + sign_h * 0.6;
            let tick_x = v_start_x - fs * 0.1;
            let tick_y = v_start_y - fs * 0.05;

            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                tick_x, tick_y,
                v_start_x, v_start_y,
                v_mid_x, v_mid_y,
                body_left, v_top,
                x + lb.width, v_top,
                color, fs * 0.04,
            ));

            // 인덱스 (있으면)
            if let Some(idx) = index {
                render_box(svg, idx, sign_x, y, color, fs * super::layout::SCRIPT_SCALE, false, false);
            }

            // 본체
            render_box(svg, body, x, y, color, fs, italic, bold);
        }
        LayoutKind::Superscript { base, sup } => {
            render_box(svg, base, x, y, color, fs, italic, bold);
            render_box(svg, sup, x, y, color, fs * super::layout::SCRIPT_SCALE, italic, bold);
        }
        LayoutKind::Subscript { base, sub } => {
            render_box(svg, base, x, y, color, fs, italic, bold);
            render_box(svg, sub, x, y, color, fs * super::layout::SCRIPT_SCALE, italic, bold);
        }
        LayoutKind::SubSup { base, sub, sup } => {
            render_box(svg, base, x, y, color, fs, italic, bold);
            render_box(svg, sub, x, y, color, fs * super::layout::SCRIPT_SCALE, italic, bold);
            render_box(svg, sup, x, y, color, fs * super::layout::SCRIPT_SCALE, italic, bold);
        }
        LayoutKind::BigOp { symbol, sub, sup } => {
            let op_fs = fs * super::layout::BIG_OP_SCALE;
            let is_integral = super::layout::is_integral_symbol(symbol);
            let esc = escape_xml(symbol);

            if is_integral {
                // 적분: 기호는 왼쪽, 첨자는 오른쪽 위/아래 (nolimits)
                let op_x = x;
                let op_y = y + op_fs * 0.8;
                svg.push_str(&format!(
                    "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                    op_x, op_y, op_fs, color, EQ_FONT_FAMILY, esc,
                ));
            } else {
                // ∑, ∏ 등: 기호는 중앙, 첨자는 위/아래 (limits)
                let sup_h = sup.as_ref().map(|b| b.height + fs * 0.05).unwrap_or(0.0);
                let op_x = x + (lb.width - estimate_op_width(symbol, op_fs)) / 2.0;
                let op_y = y + sup_h + op_fs * 0.8;
                svg.push_str(&format!(
                    "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                    op_x, op_y, op_fs, color, EQ_FONT_FAMILY, esc,
                ));
            }
            // 위/아래 첨자: LayoutBox의 자식 좌표로 배치
            if let Some(sup_box) = sup {
                render_box(svg, sup_box, x, y, color, fs * super::layout::SCRIPT_SCALE, false, false);
            }
            if let Some(sub_box) = sub {
                render_box(svg, sub_box, x, y, color, fs * super::layout::SCRIPT_SCALE, false, false);
            }
        }
        LayoutKind::Limit { is_upper, sub } => {
            let name = if *is_upper { "Lim" } else { "lim" };
            let fi = fs;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                x, y + fi * 0.8, fi, color, EQ_FONT_FAMILY, name,
            ));
            if let Some(sub_box) = sub {
                render_box(svg, sub_box, x, y, color, fs * super::layout::SCRIPT_SCALE, false, false);
            }
        }
        LayoutKind::Matrix { cells, style } => {
            // 괄호
            let bracket_chars = match style {
                MatrixStyle::Paren => ("(", ")"),
                MatrixStyle::Bracket => ("[", "]"),
                MatrixStyle::Vert => ("|", "|"),
                MatrixStyle::Plain => ("", ""),
            };
            if !bracket_chars.0.is_empty() {
                draw_stretch_bracket(svg, bracket_chars.0, x, y, fs * 0.3, lb.height, color, fs);
                draw_stretch_bracket(svg, bracket_chars.1, x + lb.width - fs * 0.3, y, fs * 0.3, lb.height, color, fs);
            }
            // 셀 내용
            for row in cells {
                for cell in row {
                    render_box(svg, cell, x, y, color, fs, italic, bold);
                }
            }
        }
        LayoutKind::Rel { arrow, over, under } => {
            render_box(svg, over, x, y, color, fs, italic, bold);
            render_box(svg, arrow, x, y, color, fs, italic, bold);
            if let Some(u) = under {
                render_box(svg, u, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::EqAlign { rows } => {
            for (left, right) in rows {
                render_box(svg, left, x, y, color, fs, italic, bold);
                render_box(svg, right, x, y, color, fs, italic, bold);
            }
        }
        LayoutKind::Paren { left, right, body } => {
            // 텍스트 높이 파렌(`(`, `)`)은 폰트 글리프로 렌더, 그 외는 path. (Task #283)
            let paren_w = fs * 0.333;
            let use_glyph = lb.height <= fs * 1.2;
            // 왼쪽 괄호
            if !left.is_empty() {
                if use_glyph && (left == "(" || left == ")") {
                    svg.push_str(&format!(
                        "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                        x, y + lb.baseline, fs, color, EQ_FONT_FAMILY, escape_xml(left),
                    ));
                } else {
                    draw_stretch_bracket(svg, left, x, y, paren_w, lb.height, color, fs);
                }
            }
            // 본체
            render_box(svg, body, x, y, color, fs, italic, bold);
            // 오른쪽 괄호
            if !right.is_empty() {
                let right_x = x + lb.width - paren_w;
                if use_glyph && (right == "(" || right == ")") {
                    svg.push_str(&format!(
                        "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\"{}>{}</text>\n",
                        right_x, y + lb.baseline, fs, color, EQ_FONT_FAMILY, escape_xml(right),
                    ));
                } else {
                    draw_stretch_bracket(svg, right, right_x, y, paren_w, lb.height, color, fs);
                }
            }
        }
        LayoutKind::Decoration { kind, body } => {
            render_box(svg, body, x, y, color, fs, italic, bold);
            let deco_y = y + fs * 0.05;
            let mid_x = x + body.x + body.width / 2.0;
            draw_decoration(svg, *kind, mid_x, deco_y, body.width, color, fs);
        }
        LayoutKind::FontStyle { style, body } => {
            let (new_italic, new_bold) = match style {
                FontStyleKind::Roman | FontStyleKind::SansSerif | FontStyleKind::Monospace => (false, false),
                FontStyleKind::Italic => (true, bold),
                FontStyleKind::Bold => (italic, true),
                FontStyleKind::Blackboard => (false, true),
                FontStyleKind::Calligraphy | FontStyleKind::Fraktur => (false, false),
            };
            render_box(svg, body, x, y, color, fs, new_italic, new_bold);
        }
        LayoutKind::Space(_) | LayoutKind::Newline | LayoutKind::Empty => {}
    }
}

fn font_size_from_box(lb: &LayoutBox, base_fs: f64) -> f64 {
    // 박스 높이에서 폰트 크기 추정 (baseline 비율로)
    if lb.height > 0.0 {
        lb.height
    } else {
        base_fs
    }
}

fn estimate_op_width(text: &str, fs: f64) -> f64 {
    text.chars().count() as f64 * fs * 0.6
}

/// 늘림 괄호 렌더링
fn draw_stretch_bracket(svg: &mut String, bracket: &str, x: f64, y: f64, w: f64, h: f64, color: &str, fs: f64) {
    let mid_x = x + w / 2.0;
    let stroke_w = fs * 0.04;

    match bracket {
        "(" => {
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x + w * 0.2, y,
                x, y + h / 2.0,
                mid_x + w * 0.2, y + h,
                color, stroke_w,
            ));
        }
        ")" => {
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - w * 0.2, y,
                x + w, y + h / 2.0,
                mid_x - w * 0.2, y + h,
                color, stroke_w,
            ));
        }
        "[" => {
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x + w * 0.2, y,
                mid_x - w * 0.2, y,
                mid_x - w * 0.2, y + h,
                mid_x + w * 0.2, y + h,
                color, stroke_w,
            ));
        }
        "]" => {
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - w * 0.2, y,
                mid_x + w * 0.2, y,
                mid_x + w * 0.2, y + h,
                mid_x - w * 0.2, y + h,
                color, stroke_w,
            ));
        }
        "{" => {
            let qh = h / 4.0;
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x + w * 0.2, y,
                mid_x - w * 0.1, y,
                mid_x - w * 0.1, y + qh,
                mid_x - w * 0.1, y + qh * 2.0,
                mid_x - w * 0.3, y + qh * 2.0,
                mid_x - w * 0.1, y + qh * 2.0,
                mid_x - w * 0.1, y + qh * 3.0,
                mid_x - w * 0.1, y + h,
                mid_x + w * 0.2, y + h,
                color, stroke_w,
            ));
        }
        "}" => {
            let qh = h / 4.0;
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - w * 0.2, y,
                mid_x + w * 0.1, y,
                mid_x + w * 0.1, y + qh,
                mid_x + w * 0.1, y + qh * 2.0,
                mid_x + w * 0.3, y + qh * 2.0,
                mid_x + w * 0.1, y + qh * 2.0,
                mid_x + w * 0.1, y + qh * 3.0,
                mid_x + w * 0.1, y + h,
                mid_x - w * 0.2, y + h,
                color, stroke_w,
            ));
        }
        "|" => {
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x, y, mid_x, y + h, color, stroke_w,
            ));
        }
        _ => {
            // 기타 문자 (⌈, ⌉, ⌊, ⌋ 등)은 텍스트로 렌더링
            let esc = escape_xml(bracket);
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{:.2}\" fill=\"{}\" text-anchor=\"middle\"{}>{}</text>\n",
                mid_x, y + h * 0.7, h, color, EQ_FONT_FAMILY, esc,
            ));
        }
    }
}

/// 장식 렌더링
fn draw_decoration(svg: &mut String, kind: DecoKind, mid_x: f64, y: f64, width: f64, color: &str, fs: f64) {
    let stroke_w = fs * 0.03;
    let half_w = width / 2.0;

    match kind {
        DecoKind::Hat => {
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - half_w * 0.6, y + fs * 0.15,
                mid_x, y,
                mid_x + half_w * 0.6, y + fs * 0.15,
                color, stroke_w,
            ));
        }
        DecoKind::Bar | DecoKind::Overline => {
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - half_w, y + fs * 0.05,
                mid_x + half_w, y + fs * 0.05,
                color, stroke_w,
            ));
        }
        DecoKind::Vec => {
            // 오른쪽 화살표
            let arrow_y = y + fs * 0.05;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - half_w, arrow_y,
                mid_x + half_w, arrow_y,
                color, stroke_w,
            ));
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x + half_w - fs * 0.1, arrow_y - fs * 0.06,
                mid_x + half_w, arrow_y,
                mid_x + half_w - fs * 0.1, arrow_y + fs * 0.06,
                color, stroke_w,
            ));
        }
        DecoKind::Tilde => {
            let ty = y + fs * 0.08;
            svg.push_str(&format!(
                "<path d=\"M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - half_w * 0.6, ty,
                mid_x - half_w * 0.2, ty - fs * 0.08,
                mid_x, ty,
                mid_x + half_w * 0.2, ty + fs * 0.08,
                mid_x + half_w * 0.6, ty,
                color, stroke_w,
            ));
        }
        DecoKind::Dot => {
            svg.push_str(&format!(
                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>\n",
                mid_x, y + fs * 0.06, fs * 0.03, color,
            ));
        }
        DecoKind::DDot => {
            let gap = fs * 0.1;
            svg.push_str(&format!(
                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>\n",
                mid_x - gap, y + fs * 0.06, fs * 0.03, color,
            ));
            svg.push_str(&format!(
                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>\n",
                mid_x + gap, y + fs * 0.06, fs * 0.03, color,
            ));
        }
        DecoKind::Underline | DecoKind::Under => {
            // 아래선은 y 위치를 body 아래로 옮김 (여기서는 위치만 표시)
            // 실제로는 body 높이를 알아야 하지만, 여기서는 근사치 사용
            let uy = y + fs * 1.1;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - half_w, uy, mid_x + half_w, uy, color, stroke_w,
            ));
        }
        _ => {
            // Check, Acute, Grave, Dyad, Arch, StrikeThrough 등 간략 처리
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n",
                mid_x - half_w * 0.5, y + fs * 0.1,
                mid_x + half_w * 0.5, y + fs * 0.1,
                color, stroke_w,
            ));
        }
    }
}

/// XML 특수문자 이스케이프
fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(ch),
        }
    }
    result
}

/// 수식 color(0x00BBGGRR)를 SVG 색상 문자열(#rrggbb)로 변환
pub fn eq_color_to_svg(color: u32) -> String {
    let r = color & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = (color >> 16) & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::equation::parser::EqParser;
    use crate::renderer::equation::tokenizer::tokenize;
    use crate::renderer::equation::layout::EqLayout;

    fn render_eq(script: &str) -> String {
        let tokens = tokenize(script);
        let ast = EqParser::new(tokens).parse();
        let layout = EqLayout::new(20.0).layout(&ast);
        render_equation_svg(&layout, "#000000", 20.0)
    }

    #[test]
    fn test_simple_text_svg() {
        let svg = render_eq("abc");
        assert!(svg.contains("<text"));
        assert!(svg.contains("abc"));
    }

    #[test]
    fn test_fraction_svg() {
        let svg = render_eq("a over b");
        assert!(svg.contains("<text")); // 분자/분모 텍스트
        assert!(svg.contains("<line")); // 분수선
    }

    #[test]
    fn test_atop_svg_has_no_fraction_line() {
        let svg = render_eq("a atop b");
        assert!(svg.contains("<text"));
        assert!(!svg.contains("<line"));
        let y_values: Vec<&str> = svg
            .lines()
            .filter_map(|line| line.split(" y=\"").nth(1))
            .filter_map(|rest| rest.split('"').next())
            .collect();
        assert_eq!(
            y_values.len(),
            2,
            "ATOP은 위/아래 텍스트 2개를 렌더링해야 함: {}",
            svg
        );
        assert_ne!(
            y_values[0], y_values[1],
            "ATOP은 두 항을 세로로 배치해야 함: {}",
            svg
        );
    }

    #[test]
    fn test_paren_svg() {
        // 텍스트 높이 파렌은 글리프로 렌더 (Task #283)
        let svg = render_eq("LEFT ( a RIGHT )");
        assert!(svg.contains("<text")); // 내용 + 글리프 파렌
        assert!(!svg.contains("<path")); // path 파렌 아님
    }

    #[test]
    fn test_paren_stretch_svg() {
        // 스트레치 파렌(분수 감쌈)은 path 유지 (Task #283)
        let svg = render_eq("LEFT ( a over b RIGHT )");
        assert!(svg.contains("<path")); // 스트레치 괄호
        assert!(svg.contains("<line")); // 분수선
    }

    #[test]
    fn test_eq01_svg() {
        let svg = render_eq(
            "평점=입찰가격평가~배점한도 TIMES LEFT ( {최저입찰가격} over {해당입찰가격} RIGHT )"
        );
        assert!(svg.contains("평점"));
        assert!(svg.contains("×")); // TIMES → ×
        assert!(svg.contains("<line")); // 분수선
        assert!(svg.contains("<path")); // 괄호
    }

    // Task #488: rm/it 폰트 스타일 적용 검증

    #[test]
    fn test_default_text_is_italic() {
        // hwpeq 기본: 라틴 변수는 italic
        let svg = render_eq("K");
        assert!(svg.contains("font-style=\"italic\""), "기본 변수는 italic: {}", svg);
    }

    #[test]
    fn test_rm_disables_italic() {
        // rm K (직립체): italic 미적용
        let svg = render_eq("rm K");
        assert!(!svg.contains("font-style=\"italic\""), "rm 적용 시 italic 없음: {}", svg);
        assert!(svg.contains(">K<"));
    }

    #[test]
    fn test_rm_prefix_form_disables_italic() {
        // rmK (공백 없는 prefix 형태): italic 미적용
        let svg = render_eq("rmK");
        assert!(!svg.contains("font-style=\"italic\""), "rmK 적용 시 italic 없음: {}", svg);
        assert!(svg.contains(">K<"));
        // rm prefix 자체가 토큰으로 분리되었으므로 raw "rmK" 가 SVG 텍스트로 남지 않아야 함
        assert!(!svg.contains(">rmK<"));
    }

    #[test]
    fn test_rm_compound_chemical_symbol() {
        // rmCa: 두 글자 화학 기호도 한 토큰으로 묶여 italic 미적용
        let svg = render_eq("rmCa");
        assert!(!svg.contains("font-style=\"italic\""));
        assert!(svg.contains(">Ca<"));
    }

    #[test]
    fn test_it_keeps_italic() {
        // it K (이탤릭 명시): italic 적용
        let svg = render_eq("it K");
        assert!(svg.contains("font-style=\"italic\""));
        assert!(svg.contains(">K<"));
    }

    #[test]
    fn test_cjk_never_italic() {
        // 한글은 default italic=true 영역에서도 italic 미적용
        let svg = render_eq("평점");
        assert!(!svg.contains("font-style=\"italic\""), "CJK는 italic 미적용: {}", svg);
        assert!(svg.contains("평점"));
    }
}
