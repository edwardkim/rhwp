use skia_safe::{
    gradient_shader::{Gradient, GradientColors, Interpolation as GradientInterpolation},
    paint, shaders, surfaces, Color, Color4f, FilterMode, Font, FontHinting, FontMgr, FontStyle,
    MipmapMode, Paint, Point, Rect, SamplingOptions, TileMode,
};

use crate::renderer::{
    generic_fallback, GradientFillInfo, LineStyle, PatternFillInfo, ShapeStyle, StrokeDash,
    TextStyle,
};

pub fn colorref_to_skia(color: u32, alpha_scale: f32) -> Color {
    let b = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let a = (255.0 * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_argb(a, r, g, b)
}

pub fn make_fill_paint(
    bounds: Rect,
    style: &ShapeStyle,
    gradient: Option<&GradientFillInfo>,
) -> Option<Paint> {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    if let Some(shader) = make_fill_shader(bounds, style.pattern.as_ref(), gradient) {
        paint.set_alpha_f(style.opacity as f32);
        paint.set_shader(shader);
        return Some(paint);
    }
    let fill_color = style.fill_color?;
    paint.set_color(colorref_to_skia(fill_color, style.opacity as f32));
    Some(paint)
}

pub fn make_background_fill_paint(
    bounds: Rect,
    background_color: Option<u32>,
    gradient: Option<&GradientFillInfo>,
) -> Option<Paint> {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    if let Some(shader) = make_fill_shader(bounds, None, gradient) {
        paint.set_shader(shader);
        return Some(paint);
    }
    let background_color = background_color?;
    paint.set_color(colorref_to_skia(background_color, 1.0));
    Some(paint)
}

pub fn make_stroke_paint(style: &ShapeStyle) -> Option<Paint> {
    let stroke_color = style.stroke_color?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Stroke);
    paint.set_stroke_width(if style.stroke_width > 0.0 {
        style.stroke_width as f32
    } else {
        1.0
    });
    paint.set_color(colorref_to_skia(stroke_color, style.opacity as f32));
    apply_dash(&mut paint, style.stroke_dash);
    Some(paint)
}

pub fn make_line_paint(style: &LineStyle) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Stroke);
    paint.set_stroke_width(if style.width > 0.0 {
        style.width as f32
    } else {
        1.0
    });
    paint.set_color(colorref_to_skia(style.color, 1.0));
    apply_dash(&mut paint, style.dash);
    paint
}

pub fn make_font(text_style: &TextStyle, font_mgr: &FontMgr, sample_text: &str) -> Font {
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

    let mut family_candidates = Vec::new();
    for candidate_list in [
        text_style.font_family.as_str(),
        generic_fallback(&text_style.font_family),
    ] {
        for candidate in candidate_list.split(',') {
            let candidate = candidate.trim().trim_matches('\'').trim_matches('"');
            if candidate.is_empty() {
                continue;
            }

            for alias in match candidate {
                "함초롬바탕" => vec!["함초롬바탕", "HCR Batang"],
                "함초롬돋움" => vec!["함초롬돋움", "HCR Dotum"],
                "함초롱바탕" => vec!["함초롱바탕", "HCR Batang"],
                "함초롱돋움" => vec!["함초롱돋움", "HCR Dotum"],
                "한컴바탕" => vec!["한컴바탕", "함초롬바탕", "HCR Batang"],
                "한컴돋움" => vec!["한컴돋움", "함초롬돋움", "HCR Dotum"],
                "맑은 고딕" => vec!["맑은 고딕", "Malgun Gothic"],
                "바탕" => vec!["바탕", "Batang"],
                "돋움" => vec!["돋움", "Dotum"],
                "굴림" => vec!["굴림", "Gulim"],
                "굴림체" => vec!["굴림체", "GulimChe"],
                "바탕체" => vec!["바탕체", "BatangChe"],
                "궁서" => vec!["궁서", "Gungsuh"],
                "궁서체" => vec!["궁서체", "GungsuhChe"],
                _ => vec![candidate],
            } {
                if family_candidates
                    .iter()
                    .any(|existing: &String| existing == alias)
                {
                    continue;
                }
                family_candidates.push(alias.to_string());
            }
        }
    }

    let probe_char = sample_text
        .chars()
        .find(|ch| !ch.is_whitespace() && !ch.is_ascii());
    let needs_currency_fallback = probe_char
        .is_some_and(|ch| matches!(ch, '\u{20A9}' | '\u{20AC}' | '\u{00A3}' | '\u{00A5}'));
    let needs_symbol_fallback = probe_char.is_some_and(|ch| {
        matches!(
            ch,
            '\u{2460}'..='\u{24FF}' | '\u{25A0}'..='\u{25FF}' | '\u{2600}'..='\u{27BF}'
        )
    });

    if needs_currency_fallback {
        for candidate in [
            "Malgun Gothic",
            "맑은 고딕",
            "Apple SD Gothic Neo",
            "Noto Sans CJK KR",
            "NanumGothic",
            "Noto Sans KR",
            "Pretendard",
            "DejaVu Sans",
            "sans-serif",
        ] {
            if family_candidates
                .iter()
                .any(|existing| existing == candidate)
            {
                continue;
            }
            family_candidates.push(candidate.to_string());
        }
    } else if needs_symbol_fallback {
        for candidate in [
            "Noto Sans CJK KR",
            "NanumGothic",
            "Noto Sans KR",
            "DejaVu Sans",
            "OpenSymbol",
            "Segoe UI Symbol",
            "sans-serif",
        ] {
            if family_candidates
                .iter()
                .any(|existing| existing == candidate)
            {
                continue;
            }
            family_candidates.push(candidate.to_string());
        }
    }

    if family_candidates
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case("monospace"))
    {
        for candidate in [
            "D2Coding",
            "NanumGothicCoding",
            "Noto Sans Mono",
            "DejaVu Sans Mono",
            "monospace",
        ] {
            if family_candidates
                .iter()
                .any(|existing| existing == candidate)
            {
                continue;
            }
            family_candidates.push(candidate.to_string());
        }
    } else if family_candidates
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case("serif"))
    {
        let serif_fallbacks: &[&str] = if probe_char.is_some() {
            &[
                "Batang",
                "바탕",
                "AppleMyungjo",
                "Noto Serif CJK KR",
                "NanumMyeongjo",
                "Noto Serif KR",
                "DejaVu Serif",
                "serif",
            ]
        } else {
            &[
                "DejaVu Serif",
                "Batang",
                "바탕",
                "AppleMyungjo",
                "Noto Serif CJK KR",
                "NanumMyeongjo",
                "Noto Serif KR",
                "serif",
            ]
        };
        for candidate in serif_fallbacks {
            if family_candidates
                .iter()
                .any(|existing| existing == candidate)
            {
                continue;
            }
            family_candidates.push((*candidate).to_string());
        }
    } else {
        for candidate in [
            "Malgun Gothic",
            "맑은 고딕",
            "Apple SD Gothic Neo",
            "Noto Sans CJK KR",
            "NanumGothic",
            "Noto Sans KR",
            "Pretendard",
            "DejaVu Sans",
            "sans-serif",
        ] {
            if family_candidates
                .iter()
                .any(|existing| existing == candidate)
            {
                continue;
            }
            family_candidates.push(candidate.to_string());
        }
    }

    let mut matched = None;
    for candidate in &family_candidates {
        let typeface = if let Some(probe_char) = probe_char {
            if needs_symbol_fallback
                || needs_currency_fallback
                || matches!(
                    candidate.as_str(),
                    "D2Coding"
                        | "NanumGothicCoding"
                        | "Noto Sans Mono"
                        | "DejaVu Sans Mono"
                        | "Noto Serif CJK KR"
                        | "NanumMyeongjo"
                        | "DejaVu Serif"
                        | "Noto Sans CJK KR"
                        | "NanumGothic"
                        | "DejaVu Sans"
                        | "serif"
                        | "sans-serif"
                        | "monospace"
                )
            {
                font_mgr.match_family_style_character(
                    candidate,
                    font_style,
                    &["ko", "en"],
                    probe_char as i32,
                )
            } else {
                font_mgr.match_family_style(candidate, font_style)
            }
        } else {
            font_mgr.match_family_style(candidate, font_style)
        };
        let Some(typeface) = typeface else {
            continue;
        };
        if sample_text.chars().any(|ch| !ch.is_whitespace()) {
            let probe_font = Font::new(typeface.clone(), font_size);
            let glyphs = probe_font.text_to_glyphs_vec(sample_text);
            if glyphs.len() != sample_text.chars().count()
                || glyphs
                    .iter()
                    .zip(sample_text.chars())
                    .any(|(&glyph, ch)| !ch.is_whitespace() && glyph == 0)
            {
                continue;
            }
        }
        let family_name = typeface.family_name();
        if matches!(candidate.as_str(), "serif" | "sans-serif" | "monospace")
            || family_name == *candidate
            || family_name.eq_ignore_ascii_case(candidate)
        {
            matched = Some(typeface);
            break;
        }
    }
    let matched = matched.or_else(|| font_mgr.legacy_make_typeface(None::<&str>, font_style));

    let mut font = if let Some(typeface) = matched {
        Font::new(typeface, font_size)
    } else {
        let mut font = Font::default();
        font.set_size(font_size);
        font
    };

    font.set_edging(skia_safe::font::Edging::AntiAlias);
    font.set_hinting(FontHinting::None);
    font.set_subpixel(false);
    font.set_linear_metrics(true);
    font.set_baseline_snap(false);
    font.set_scale_x(if text_style.ratio > 0.0 {
        text_style.ratio as f32
    } else {
        1.0
    });
    font
}

pub fn make_text_paint(text_style: &TextStyle) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(colorref_to_skia(text_style.color, 1.0));
    paint
}

fn make_fill_shader(
    bounds: Rect,
    pattern: Option<&PatternFillInfo>,
    gradient: Option<&GradientFillInfo>,
) -> Option<skia_safe::Shader> {
    if let Some(gradient) = gradient {
        if let Some(shader) = make_gradient_shader(gradient, bounds) {
            return Some(shader);
        }
    }
    pattern.and_then(make_pattern_shader)
}

fn make_gradient_shader(info: &GradientFillInfo, bounds: Rect) -> Option<skia_safe::Shader> {
    if info.colors.len() < 2 {
        return None;
    }

    let colors: Vec<Color4f> = info
        .colors
        .iter()
        .copied()
        .map(|color| Color4f::from(colorref_to_skia(color, 1.0)))
        .collect();
    let positions: Vec<f32> = if info.positions.len() == colors.len() {
        info.positions
            .iter()
            .map(|position| position.clamp(0.0, 1.0) as f32)
            .collect()
    } else {
        Vec::new()
    };
    let colors = if positions.is_empty() {
        GradientColors::new_evenly_spaced(&colors, TileMode::Clamp, None)
    } else {
        GradientColors::new(&colors, Some(&positions), TileMode::Clamp, None)
    };
    let shader_gradient = Gradient::new(colors, GradientInterpolation::default());

    if matches!(info.gradient_type, 2..=4) {
        let center = Point::new(
            bounds.left + bounds.width() * (info.center_x as f32 / 100.0),
            bounds.top + bounds.height() * (info.center_y as f32 / 100.0),
        );
        let radius = bounds.width().max(bounds.height()) / 2.0;
        return shaders::radial_gradient((center, radius.max(1.0)), &shader_gradient, None);
    }

    let (start, end) = angle_to_gradient_points(info.angle, bounds);
    shaders::linear_gradient((start, end), &shader_gradient, None)
}

fn angle_to_gradient_points(angle: i16, bounds: Rect) -> (Point, Point) {
    let x = bounds.left;
    let y = bounds.top;
    let width = bounds.width();
    let height = bounds.height();
    let angle = ((angle % 360 + 360) % 360) as f32;

    match angle as i32 {
        0 => (Point::new(x, y), Point::new(x, y + height)),
        45 => (Point::new(x, y), Point::new(x + width, y + height)),
        90 => (Point::new(x, y), Point::new(x + width, y)),
        135 => (Point::new(x, y + height), Point::new(x + width, y)),
        180 => (Point::new(x, y + height), Point::new(x, y)),
        225 => (Point::new(x + width, y + height), Point::new(x, y)),
        270 => (Point::new(x + width, y), Point::new(x, y)),
        315 => (Point::new(x + width, y), Point::new(x, y + height)),
        _ => {
            let radians = angle.to_radians();
            let sin_angle = radians.sin();
            let cos_angle = radians.cos();
            let center_x = x + width / 2.0;
            let center_y = y + height / 2.0;
            (
                Point::new(
                    center_x - sin_angle * width / 2.0,
                    center_y - cos_angle * height / 2.0,
                ),
                Point::new(
                    center_x + sin_angle * width / 2.0,
                    center_y + cos_angle * height / 2.0,
                ),
            )
        }
    }
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

#[cfg(test)]
mod tests {
    use super::make_font;
    use crate::renderer::TextStyle;
    use skia_safe::{FontMgr, FontStyle};

    #[test]
    fn resolves_deterministic_generic_fallback_families() {
        let font_mgr = FontMgr::default();

        let mono_family = make_font(
            &TextStyle {
                font_family: "바탕체".to_string(),
                font_size: 12.0,
                ..Default::default()
            },
            &font_mgr,
            "A",
        )
        .typeface()
        .family_name();
        let sans_family = make_font(
            &TextStyle {
                font_family: "한컴 윤고딕 230".to_string(),
                font_size: 12.0,
                ..Default::default()
            },
            &font_mgr,
            "A",
        )
        .typeface()
        .family_name();
        let hangul_sans_family = make_font(
            &TextStyle {
                font_family: "한컴 윤고딕 230".to_string(),
                font_size: 12.0,
                ..Default::default()
            },
            &font_mgr,
            "표",
        )
        .typeface()
        .family_name();

        assert!(
            matches!(
                mono_family.as_str(),
                "D2Coding" | "NanumGothicCoding" | "Noto Sans Mono" | "DejaVu Sans Mono"
            ),
            "unexpected mono fallback family: {mono_family}"
        );
        assert!(
            matches!(
                sans_family.as_str(),
                "Noto Sans CJK KR" | "NanumGothic" | "DejaVu Sans" | "Arial"
            ),
            "unexpected sans fallback family: {sans_family}"
        );
        if font_mgr
            .match_family_style("Noto Sans CJK KR", FontStyle::normal())
            .is_some()
        {
            assert_eq!(
                hangul_sans_family, "Noto Sans CJK KR",
                "unexpected Hangul sans fallback family: {hangul_sans_family}"
            );
        } else if font_mgr
            .match_family_style("NanumGothic", FontStyle::normal())
            .is_some()
        {
            assert_eq!(
                hangul_sans_family, "NanumGothic",
                "unexpected Hangul sans fallback family: {hangul_sans_family}"
            );
        }
    }

    #[test]
    fn prioritizes_ascii_and_hangul_serif_fallbacks_differently() {
        let font_mgr = FontMgr::default();

        let ascii_family = make_font(
            &TextStyle {
                font_family: "serif".to_string(),
                font_size: 12.0,
                ..Default::default()
            },
            &font_mgr,
            "n",
        )
        .typeface()
        .family_name();
        let hangul_family = make_font(
            &TextStyle {
                font_family: "serif".to_string(),
                font_size: 12.0,
                ..Default::default()
            },
            &font_mgr,
            "표",
        )
        .typeface()
        .family_name();

        if font_mgr
            .match_family_style("DejaVu Serif", FontStyle::normal())
            .is_some()
        {
            assert_eq!(
                ascii_family, "DejaVu Serif",
                "unexpected ASCII serif fallback family: {ascii_family}"
            );
        }
        if font_mgr
            .match_family_style("Noto Serif CJK KR", FontStyle::normal())
            .is_some()
        {
            assert!(
                matches!(
                    hangul_family.as_str(),
                    "Noto Serif CJK KR" | "NanumMyeongjo"
                ),
                "unexpected Hangul serif fallback family: {hangul_family}"
            );
        } else if font_mgr
            .match_family_style("NanumMyeongjo", FontStyle::normal())
            .is_some()
        {
            assert_eq!(
                hangul_family, "NanumMyeongjo",
                "unexpected Hangul serif fallback family: {hangul_family}"
            );
        }
    }
}
