//! 수식 Skia의 실제 face 선택·텍스트 표시와 늘인 괄호 표시.
use super::super::font_lookup::{
    select_typeface_for_character, text_family_candidates, typeface_candidates_for_families,
};
use super::{stroke_paint, EquationFonts, EQ_FONT_FAMILY};
use skia_safe::{font, paint, Canvas, Color, Font, FontStyle, Paint, PathBuilder};

pub(super) fn draw_text(
    canvas: &Canvas,
    fonts: &EquationFonts<'_>,
    text: &str,
    x: f64,
    baseline_y: f64,
    font_size: f64,
    italic: bool,
    bold: bool,
    color: Color,
    centered: bool,
) {
    draw_text_with_family(
        canvas, fonts, text, x, baseline_y, font_size, italic, bold, color, centered, None,
    );
}

pub(super) fn draw_text_with_family(
    canvas: &Canvas,
    fonts: &EquationFonts<'_>,
    text: &str,
    x: f64,
    baseline_y: f64,
    font_size: f64,
    italic: bool,
    bold: bool,
    color: Color,
    centered: bool,
    explicit_family: Option<&str>,
) {
    if text.is_empty() {
        return;
    }
    // 수식의 한글은 SVG/Canvas와 같은 명조 대체 글꼴과 정체 스타일을 쓴다.
    let has_cjk = crate::renderer::equation::text_has_cjk(text);
    let family = explicit_family.unwrap_or(if has_cjk {
        crate::renderer::equation::CJK_EQUATION_FONT_FAMILY
    } else {
        EQ_FONT_FAMILY
    });
    let font_style = match (bold, italic && !has_cjk) {
        (true, true) => FontStyle::bold_italic(),
        (true, false) => FontStyle::bold(),
        (false, true) => FontStyle::italic(),
        (false, false) => FontStyle::normal(),
    };
    let mut families: Vec<String> = family
        .split(',')
        .map(str::trim)
        .map(str::to_owned)
        .collect();
    let postscript_requests = families.clone();
    // Match the web equation's serif CJK fallback before general body fallbacks.
    if has_cjk {
        families.extend(
            [
                "AppleMyungjo",
                "Noto Serif KR",
                "Noto Serif CJK KR",
                "Nanum Myeongjo",
            ]
            .map(str::to_owned),
        );
    }
    for fallback in text_family_candidates("") {
        if !families.contains(&fallback) {
            families.push(fallback);
        }
    }
    let chain = typeface_candidates_for_families(
        fonts.font_mgr,
        fonts.system_families,
        fonts.custom_typefaces,
        fonts.bundled_typefaces,
        &families,
        &postscript_requests,
        font_style,
    );
    // Preserve shaping within consecutive characters using the same face. Use
    // these exact runs for measurement and paint, including centered symbols.
    let mut runs: Vec<(Option<usize>, String)> = Vec::new();
    for ch in text.chars() {
        let selected = select_typeface_for_character(&chain, ch).or_else(|| chain.first());
        let index = selected.and_then(|candidate| {
            chain
                .iter()
                .position(|entry| std::ptr::eq(entry, candidate))
        });
        if let Some((_, run)) = runs.last_mut().filter(|(previous, _)| *previous == index) {
            run.push(ch);
        } else {
            runs.push((index, ch.to_string()));
        }
    }
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(color);
    let runs: Vec<_> = runs
        .into_iter()
        .map(|(index, text)| {
            let mut face = index
                .map(|index| Font::new(chain[index].typeface.clone(), font_size as f32))
                .unwrap_or_default();
            face.set_size(font_size as f32);
            face.set_edging(font::Edging::AntiAlias);
            let (width, _) = face.measure_str(&text, Some(&paint));
            (face, text, f64::from(width))
        })
        .collect();
    let mut draw_x = if centered {
        x - runs.iter().map(|(_, _, width)| width).sum::<f64>() / 2.0
    } else {
        x
    };
    for (face, text, width) in runs {
        canvas.draw_str(&text, (draw_x as f32, baseline_y as f32), &face, &paint);
        draw_x += width;
    }
}

pub(super) fn draw_stretch_bracket(
    canvas: &Canvas,
    fonts: &EquationFonts<'_>,
    bracket: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: Color,
    fs: f64,
) {
    if matches!(bracket, "{" | "}") {
        let commands =
            crate::renderer::equation::brace_stretch::commands(x, y, w, h, fs, bracket == "}");
        super::super::equation_glyph::draw_outline(canvas, &commands, 0.0, 0.0, color, None);
        return;
    }
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
                fonts,
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
