use skia_safe::{paint, Color, Font, FontHinting, FontMgr, FontStyle, Paint};

use crate::renderer::{generic_fallback, LineStyle, ShapeStyle, StrokeDash, TextStyle};

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
            if candidate.is_empty()
                || family_candidates
                    .iter()
                    .any(|existing: &String| existing == candidate)
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
        for candidate in [
            "Noto Serif CJK KR",
            "NanumMyeongjo",
            "DejaVu Serif",
            "serif",
        ] {
            if family_candidates
                .iter()
                .any(|existing| existing == candidate)
            {
                continue;
            }
            family_candidates.push(candidate.to_string());
        }
    } else {
        for candidate in [
            "Noto Sans CJK KR",
            "NanumGothic",
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

    let probe_char = sample_text
        .chars()
        .find(|ch| !ch.is_whitespace() && !ch.is_ascii());

    let mut matched = None;
    for candidate in &family_candidates {
        let typeface = if let Some(probe_char) = probe_char {
            if matches!(
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
            ) {
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
}
