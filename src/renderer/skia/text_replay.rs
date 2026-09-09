use std::collections::HashMap;

use skia_safe::{
    font, paint, Canvas, Color, Font, FontMgr, FontStyle, Paint, PathEffect, Rect, Typeface,
};

use crate::model::style::UnderlineType;
use crate::paint::LayerOutputOptions;
use crate::renderer::composer::{
    char_overlap_display_text, char_overlap_size_ratio, decode_pua_overlap_number,
    expand_pua_render_text, CharOverlapInfo,
};
use crate::renderer::layout::{forces_halfwidth_cjk_quote, split_into_clusters};
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::{boxed_pua_char_overlap_semantics, clamp_tab_leader_end_x, TextStyle};

use super::font_lookup::{
    legacy_typeface_for_style, match_system_family_style, select_typeface_for_character,
    text_typeface_candidates, SystemFontFamilies,
};
use super::renderer::colorref_to_skia;

pub(super) struct SkiaTextReplay<'a> {
    pub(super) canvas: &'a Canvas,
    pub(super) font_mgr: &'a FontMgr,
    pub(super) custom_typefaces: &'a HashMap<String, Typeface>,
    pub(super) bundled_typefaces: &'a HashMap<String, Typeface>,
    pub(super) system_families: &'a SystemFontFamilies,
    pub(super) output_options: &'a LayerOutputOptions,
}

impl SkiaTextReplay<'_> {
    pub(super) fn draw_text(
        &self,
        text: &str,
        bbox: BoundingBox,
        style: &TextStyle,
        baseline: f64,
        rotation: f64,
        is_vertical: bool,
        char_overlap: Option<&CharOverlapInfo>,
        is_marker: bool,
        is_para_end: bool,
        is_line_break_end: bool,
        layout_positions: Option<&[f64]>,
        trim_trailing_spaces: usize,
        suppress_glyphs: bool,
        render_marks: bool,
    ) {
        let canvas = self.canvas;
        let output_options = self.output_options;
        let draw_text =
            |text: &str,
             bbox: crate::renderer::render_tree::BoundingBox,
             style: &crate::renderer::TextStyle,
             baseline: f64,
             rotation: f64,
             is_vertical: bool,
             char_overlap: Option<&crate::renderer::composer::CharOverlapInfo>| {
                if text.is_empty() && style.tab_leaders.is_empty() {
                    return;
                }
                let base_font_size = if style.font_size > 0.0 {
                    style.font_size
                } else {
                    12.0
                };
                // [#2771] 위첨자/아래첨자를 SVG/Canvas/HTML 과 동일한 계약(0.7 배
                // 글꼴 + baseline 이동)으로 그린다. 종전 skia 경로는 첨자 분기가
                // 아예 없어 본문과 같은 크기·같은 baseline 으로 그렸다.
                // baseline 이동은 아래 `y` 계산에서 함께 적용한다.
                let (draw_font_size, _) = style.script_draw_metrics(base_font_size, 0.0);
                let font_size = draw_font_size as f32;
                let font_style = match (style.bold, style.italic) {
                    (true, true) => FontStyle::bold_italic(),
                    (true, false) => FontStyle::bold(),
                    (false, true) => FontStyle::italic(),
                    (false, false) => FontStyle::normal(),
                };
                // 1) 사용자 지정 폰트 (--font-path) 우선 검색
                // 2) 시스템 FontMgr 검색 (한글 fallback chain 포함)
                // 3) 마지막 fallback (legacy_make_typeface)
                //
                // 모든 후보를 chain 으로 보존 — char 단위 fallback 에 사용.
                // 후보와 glyph 선택은 opt-in font decision trace도 그대로 사용한다.
                let (_, typeface_chain) = text_typeface_candidates(
                    self.font_mgr,
                    self.system_families,
                    self.custom_typefaces,
                    self.bundled_typefaces,
                    &style.font_family,
                    font_style,
                );
                let primary_typeface = typeface_chain
                    .first()
                    .map(|candidate| candidate.typeface.clone());
                // 한글은 bold face 가 없는 폰트(휴먼명조 등 단일 400 페이스)에
                // 동일 정규 페이스 + stroke 로 합성 굵게를 적용한다 (오라클
                // PDF 실측: 굵은 헤더가 정규 휴먼명조 임베드로 방출). custom
                // typeface 는 스타일 무시 단일 페이스라 여기서 embolden 으로
                // 합성한다. 시스템 매칭이 진짜 bold 페이스를 반환한 경우는
                // weight 조건에서 제외돼 이중 굵게가 없다.
                let want_synthetic_bold = style.bold;
                let finish_font = |tf: Typeface, size: f32| -> Font {
                    let is_bold_face = *tf.font_style().weight() >= 600;
                    let mut font = Font::new(tf, size);
                    font.set_edging(font::Edging::AntiAlias);
                    if want_synthetic_bold && !is_bold_face {
                        font.set_embolden(true);
                    }
                    font
                };
                let font_for_text = |sample: &str, size: f32| -> Option<Font> {
                    if let Some(ch) = sample.chars().find(|ch| !ch.is_whitespace()) {
                        if let Some(candidate) = select_typeface_for_character(&typeface_chain, ch)
                        {
                            return Some(finish_font(candidate.typeface.clone(), size));
                        }
                        return None;
                    }
                    if let Some(tf) = primary_typeface.clone() {
                        Some(finish_font(tf, size))
                    } else {
                        let mut font = Font::default();
                        font.set_size(size);
                        font.set_edging(font::Edging::AntiAlias);
                        Some(font)
                    }
                };
                let baseline_y = if baseline > 0.0 {
                    bbox.y + baseline
                } else {
                    bbox.y + bbox.height
                };
                // [#2771] 첨자 baseline 이동 (위 0.3em / 아래 0.15em). 비첨자는 항등.
                let (_, y) = style.script_draw_metrics(base_font_size, baseline_y);
                let effective_rotation = if is_vertical {
                    rotation + 90.0
                } else {
                    rotation
                };
                if effective_rotation != 0.0 {
                    canvas.save();
                    canvas.rotate(
                        effective_rotation as f32,
                        Some(
                            (
                                (bbox.x + bbox.width / 2.0) as f32,
                                (bbox.y + bbox.height / 2.0) as f32,
                            )
                                .into(),
                        ),
                    );
                }

                if let Some(overlap) = char_overlap {
                    let chars: Vec<char> = text.chars().collect();
                    if chars.is_empty() {
                        if effective_rotation != 0.0 {
                            canvas.restore();
                        }
                        return;
                    }

                    let box_size = font_size.max(1.0);
                    let is_combined = decode_pua_overlap_number(&chars);
                    let boxed_pua = boxed_pua_char_overlap_semantics(&chars, overlap.border_type);
                    let effective_border = boxed_pua
                        .map(|(_, border_type)| border_type)
                        .unwrap_or_else(|| {
                            if overlap.border_type == 0 && is_combined.is_some() {
                                1
                            } else {
                                overlap.border_type
                            }
                        });
                    // charSz 는 "테두리 내부" 글자 비율 — SVG/CanvasKit 과 같은 규칙을 쓴다 (#4085).
                    let size_ratio =
                        char_overlap_size_ratio(effective_border, overlap.inner_char_size) as f32;
                    let inner_size = (font_size * size_ratio).max(1.0);
                    let is_reversed = effective_border == 2 || effective_border == 4;
                    let is_circle = effective_border == 1 || effective_border == 2;
                    let is_rect = effective_border == 3 || effective_border == 4;
                    let fill_color = if is_reversed {
                        Color::BLACK
                    } else {
                        Color::TRANSPARENT
                    };
                    let text_color = if is_reversed {
                        Color::WHITE
                    } else {
                        colorref_to_skia(style.color, 1.0)
                    };
                    let stroke_color = colorref_to_skia(style.color, 1.0);
                    let mut shape_paint = Paint::default();
                    shape_paint.set_anti_alias(true);
                    let mut stroke_paint = Paint::default();
                    stroke_paint.set_anti_alias(true);
                    stroke_paint.set_style(paint::Style::Stroke);
                    stroke_paint.set_stroke_width(0.8);
                    stroke_paint.set_color(stroke_color);
                    let mut text_paint = Paint::default();
                    text_paint.set_anti_alias(true);
                    text_paint.set_color(text_color);
                    let draw_overlap_text = |display: &str, cx: f32, cy: f32| {
                        if let Some(font) = font_for_text(display, inner_size) {
                            let width = font.measure_str(display, Some(&text_paint)).0;
                            canvas.draw_str(
                                display,
                                (cx - width / 2.0, cy + inner_size * 0.35),
                                &font,
                                &text_paint,
                            );
                        }
                    };
                    let mut draw_overlap_box = |display: &str, cx: f32, cy: f32| {
                        if is_circle {
                            shape_paint.set_style(paint::Style::Fill);
                            shape_paint.set_color(fill_color);
                            if is_reversed {
                                canvas.draw_circle((cx, cy), box_size / 2.0, &shape_paint);
                            }
                            canvas.draw_circle((cx, cy), box_size / 2.0, &stroke_paint);
                        } else if is_rect {
                            let rect = Rect::from_xywh(
                                cx - box_size / 2.0,
                                cy - box_size / 2.0,
                                box_size,
                                box_size,
                            );
                            shape_paint.set_style(paint::Style::Fill);
                            shape_paint.set_color(fill_color);
                            if is_reversed {
                                canvas.draw_rect(rect, &shape_paint);
                            }
                            canvas.draw_rect(rect, &stroke_paint);
                        }
                        draw_overlap_text(display, cx, cy);
                    };

                    if let Some(number) = is_combined {
                        draw_overlap_box(
                            &number,
                            (bbox.x + bbox.width / 2.0) as f32,
                            (bbox.y + bbox.height / 2.0) as f32,
                        );
                    } else if chars.len() > 1 {
                        let cx = (bbox.x + bbox.width / 2.0) as f32;
                        let cy = (bbox.y + bbox.height / 2.0) as f32;
                        if is_circle {
                            shape_paint.set_style(paint::Style::Fill);
                            shape_paint.set_color(fill_color);
                            if is_reversed {
                                canvas.draw_circle((cx, cy), box_size / 2.0, &shape_paint);
                            }
                            canvas.draw_circle((cx, cy), box_size / 2.0, &stroke_paint);
                        } else if is_rect {
                            let rect = Rect::from_xywh(
                                cx - box_size / 2.0,
                                cy - box_size / 2.0,
                                box_size,
                                box_size,
                            );
                            shape_paint.set_style(paint::Style::Fill);
                            shape_paint.set_color(fill_color);
                            if is_reversed {
                                canvas.draw_rect(rect, &shape_paint);
                            }
                            canvas.draw_rect(rect, &stroke_paint);
                        }

                        for ch in chars.iter() {
                            let display = char_overlap_display_text(*ch, is_circle || is_rect);
                            draw_overlap_text(&display, cx, cy);
                        }
                    } else {
                        for (index, ch) in chars.iter().enumerate() {
                            let display = if let Some((number, _)) = boxed_pua {
                                number.to_string()
                            } else {
                                char_overlap_display_text(*ch, is_circle || is_rect)
                            };
                            draw_overlap_box(
                                &display,
                                bbox.x as f32 + index as f32 * box_size + box_size / 2.0,
                                (bbox.y + bbox.height / 2.0) as f32,
                            );
                        }
                    }
                    if effective_rotation != 0.0 {
                        canvas.restore();
                    }
                    return;
                }

                let text = expand_pua_render_text(text);
                let text = text.as_str();
                let char_positions =
                    crate::renderer::replay_positions_or_compute(text, style, layout_positions);
                let clusters = split_into_clusters(text);
                let trailing = text.chars().rev().take_while(|ch| *ch == ' ').count();
                let trim = trim_trailing_spaces.min(trailing);
                let text_width = char_positions
                    .get(text.chars().count().saturating_sub(trim))
                    .copied()
                    .unwrap_or_else(|| *char_positions.last().unwrap_or(&0.0))
                    as f32;
                // [#5821] 압축 장평은 세로도 √r — SSOT 는 condensed_ratio_draw_params.
                let (font_size, ratio) = {
                    let (fs, r) =
                        crate::renderer::condensed_ratio_draw_params(font_size as f64, style.ratio);
                    (fs as f32, r as f32)
                };
                let has_ratio = (ratio - 1.0).abs() > 0.01;
                if !suppress_glyphs
                    && crate::model::color::char_shade(style.shade_color).is_some()
                    && text_width > 0.0
                {
                    let mut shade = Paint::default();
                    shade.set_anti_alias(true);
                    shade.set_style(paint::Style::Fill);
                    shade.set_color(colorref_to_skia(style.shade_color, 1.0));
                    canvas.draw_rect(
                        Rect::from_xywh(
                            bbox.x as f32,
                            y as f32 - font_size,
                            text_width,
                            font_size * 1.2,
                        ),
                        &shade,
                    );
                }

                let draw_styled_line = |x1: f32,
                                        y: f32,
                                        x2: f32,
                                        color: Color,
                                        width: f32,
                                        dash: &[f32],
                                        round: bool| {
                    if x2 <= x1 {
                        return;
                    }
                    let mut line_paint = Paint::default();
                    line_paint.set_anti_alias(true);
                    line_paint.set_style(paint::Style::Stroke);
                    line_paint.set_stroke_width(width);
                    line_paint.set_color(color);
                    if round {
                        line_paint.set_stroke_cap(paint::Cap::Round);
                    }
                    if !dash.is_empty() {
                        if let Some(effect) = PathEffect::dash(dash, 0.0) {
                            line_paint.set_path_effect(effect);
                        }
                    }
                    canvas.draw_line((x1, y), (x2, y), &line_paint);
                };
                let draw_line_shape =
                    |x1: f32, y: f32, x2: f32, color: Color, shape: u8| match shape {
                        7 => {
                            draw_styled_line(x1, y - 1.0, x2, color, 0.7, &[], false);
                            draw_styled_line(x1, y + 1.0, x2, color, 0.7, &[], false);
                        }
                        8 => {
                            draw_styled_line(x1, y - 1.2, x2, color, 0.5, &[], false);
                            draw_styled_line(x1, y + 0.8, x2, color, 1.2, &[], false);
                        }
                        9 => {
                            draw_styled_line(x1, y - 0.8, x2, color, 1.2, &[], false);
                            draw_styled_line(x1, y + 1.2, x2, color, 0.5, &[], false);
                        }
                        10 => {
                            draw_styled_line(x1, y - 1.5, x2, color, 0.5, &[], false);
                            draw_styled_line(x1, y, x2, color, 0.5, &[], false);
                            draw_styled_line(x1, y + 1.5, x2, color, 0.5, &[], false);
                        }
                        1 => draw_styled_line(x1, y, x2, color, 1.0, &[3.0, 3.0], false),
                        2 => draw_styled_line(x1, y, x2, color, 1.0, &[1.0, 2.0], false),
                        3 => draw_styled_line(x1, y, x2, color, 1.0, &[6.0, 2.0, 1.0, 2.0], false),
                        4 => draw_styled_line(
                            x1,
                            y,
                            x2,
                            color,
                            1.0,
                            &[6.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                            false,
                        ),
                        5 => draw_styled_line(x1, y, x2, color, 1.0, &[8.0, 4.0], false),
                        6 => draw_styled_line(x1, y, x2, color, 1.0, &[0.1, 2.5], true),
                        _ => draw_styled_line(x1, y, x2, color, 1.0, &[], false),
                    };

                // [#5804] 3+ 연속 '-' 를 단일 가로선으로 대체하던 처리(Task #352)를 걷어냈다.
                // 한글 2022 정본은 하이픈을 낱글자 글리프로 그리고, 그 탄력 분배는 이미
                // 레이아웃이 `extra_dash_advance` 로 만들어 `char_positions` 에 담는다.
                // svg.rs 와 같은 결정이다.
                let cluster_advance = |char_idx: usize, cluster: &str| -> f32 {
                    let end = char_idx + cluster.chars().count();
                    if end < char_positions.len() {
                        (char_positions[end] - char_positions[char_idx]) as f32
                    } else {
                        0.0
                    }
                };
                let is_middle_dot = |cluster: &str| cluster == "\u{00B7}";
                let draw_text_pass = |color: Color, stroke_width: f32, dx: f32, dy: f32| {
                    let mut text_paint = Paint::default();
                    text_paint.set_anti_alias(true);
                    text_paint.set_color(color);
                    if stroke_width > 0.0 {
                        text_paint.set_style(paint::Style::Stroke);
                        text_paint.set_stroke_width(stroke_width);
                    } else {
                        text_paint.set_style(paint::Style::Fill);
                    }
                    for (char_idx, cluster) in clusters.iter() {
                        if cluster == " " || cluster == "\t" || cluster == "\u{2007}" {
                            continue;
                        }
                        if cluster.starts_with(|ch: char| {
                            ch < '\u{0020}' && !matches!(ch, '\t' | '\n' | '\r')
                        }) {
                            continue;
                        }
                        if is_middle_dot(cluster) {
                            let advance = cluster_advance(*char_idx, cluster);
                            let cx = bbox.x as f32
                                + char_positions.get(*char_idx).copied().unwrap_or(0.0) as f32
                                + advance / 2.0
                                + dx;
                            let cy = y as f32
                                - font_size
                                    * crate::renderer::render_tree::MIDDLE_DOT_CY_OFFSET_EM as f32
                                + dy;
                            let mut dot_paint = Paint::default();
                            dot_paint.set_anti_alias(true);
                            dot_paint.set_style(paint::Style::Fill);
                            dot_paint.set_color(color);
                            canvas.draw_circle(
                                (cx, cy),
                                font_size
                                    * crate::renderer::render_tree::MIDDLE_DOT_RADIUS_EM as f32,
                                &dot_paint,
                            );
                            continue;
                        }
                        // [#6127] 한컴 사각 안 숫자(U+F02B1~F02C4) 평문 폴백 —
                        // web_canvas·SVG 와 같은 bounded vector 합성(상자 0.72em,
                        // 숫자 0.5em). raw PUA 는 함초롬 확장 글꼴이 없으면 빈칸.
                        if cluster.chars().count() == 1 {
                            if let Some(number) = cluster
                                .chars()
                                .next()
                                .and_then(crate::renderer::boxed_pua_number)
                            {
                                let char_x = bbox.x as f32
                                    + char_positions.get(*char_idx).copied().unwrap_or(0.0) as f32
                                    + dx;
                                let baseline_y = y as f32 + dy;
                                let box_size = (font_size * 0.72).max(1.0);
                                let box_y = baseline_y - font_size * 0.76;
                                let mut box_paint = Paint::default();
                                box_paint.set_anti_alias(true);
                                box_paint.set_style(paint::Style::Stroke);
                                box_paint.set_stroke_width((font_size * 0.04).max(0.6));
                                box_paint.set_color(color);
                                canvas.draw_rect(
                                    skia_safe::Rect::from_xywh(char_x, box_y, box_size, box_size),
                                    &box_paint,
                                );
                                let number_str = number.to_string();
                                let number_size = (font_size * 0.5).max(1.0);
                                if let Some(number_font) = font_for_text(&number_str, number_size) {
                                    let width =
                                        number_font.measure_str(&number_str, Some(&text_paint)).0;
                                    canvas.draw_str(
                                        &number_str,
                                        (
                                            char_x + (box_size - width) / 2.0,
                                            box_y + box_size * 0.72,
                                        ),
                                        &number_font,
                                        &text_paint,
                                    );
                                }
                                continue;
                            }
                        }
                        if let Some(font) = font_for_text(cluster, font_size) {
                            let char_x = bbox.x as f32
                                + char_positions.get(*char_idx).copied().unwrap_or(0.0) as f32
                                + dx;
                            let char_y = y as f32 + dy;
                            // 반각 강제 구두점: 측정은 반각(0.3~0.5em)인데 폰트
                            // 글리프가 전각인 문자(휴먼명조 U+2018 등)를 그대로
                            // 그리면 다음 글자와 겹친다. web_canvas 와 동일하게
                            // 0.5× 수평 축소로 반각 공간에 배치 (한글은 자체
                            // 내장 협폭 글리프로 렌더 — 오라클 PDF Type3 실측).
                            let needs_halfwidth_scale = cluster.chars().next().is_some_and(|ch| {
                                matches!(ch, '\u{2018}'..='\u{2027}')
                                    || forces_halfwidth_cjk_quote(
                                        &style.font_family,
                                        style.bold,
                                        style.italic,
                                        ch,
                                        style.font_size,
                                    )
                            }) && !has_ratio;
                            if needs_halfwidth_scale {
                                canvas.save();
                                canvas.translate((char_x, char_y));
                                canvas.scale((0.5, 1.0));
                                canvas.draw_str(cluster, (0.0, 0.0), &font, &text_paint);
                                canvas.restore();
                                continue;
                            }
                            if has_ratio {
                                canvas.save();
                                canvas.translate((char_x, char_y));
                                canvas.scale((ratio, 1.0));
                                canvas.draw_str(cluster, (0.0, 0.0), &font, &text_paint);
                                canvas.restore();
                            } else {
                                canvas.draw_str(cluster, (char_x, char_y), &font, &text_paint);
                            }
                        }
                    }
                };

                if !suppress_glyphs {
                    if style.shadow_type > 0 {
                        draw_text_pass(
                            colorref_to_skia(style.shadow_color, 1.0),
                            0.0,
                            style.shadow_offset_x as f32,
                            style.shadow_offset_y as f32,
                        );
                    }
                    if style.outline_type > 0 {
                        draw_text_pass(
                            colorref_to_skia(style.color, 1.0),
                            (font_size * 0.08).max(0.8),
                            0.0,
                            0.0,
                        );
                    }
                    if style.emboss {
                        draw_text_pass(Color::WHITE, 0.0, -1.0, -1.0);
                        draw_text_pass(Color::from_argb(255, 96, 96, 96), 0.0, 1.0, 1.0);
                    } else if style.engrave {
                        draw_text_pass(Color::from_argb(255, 96, 96, 96), 0.0, -1.0, -1.0);
                        draw_text_pass(Color::WHITE, 0.0, 1.0, 1.0);
                    }
                    draw_text_pass(colorref_to_skia(style.color, 1.0), 0.0, 0.0, 0.0);
                }

                if !matches!(style.underline, UnderlineType::None) && text_width > 0.0 {
                    // COLORREF 0 은 미지정이 아니라 검정 — svg.rs 와 같은 계약.
                    let color = colorref_to_skia(style.underline_color, 1.0);
                    // [#5730] 아래 밑줄은 기준선 + 0.17em (한글 2022 프로브 실측) —
                    // 이중/삼중선(shape 7~10)은 em 비례 실측표를 따른다.
                    // SVG 백엔드(renderer/svg.rs)와 같은 표(text_decoration)를 소비한다.
                    let multi = if matches!(style.underline, UnderlineType::Top) {
                        None
                    } else {
                        crate::renderer::text_decoration::underline_multi_lines(
                            style.underline_shape,
                        )
                    };
                    if let Some(lines) = multi {
                        for (dy_em, width_em) in lines {
                            draw_styled_line(
                                bbox.x as f32,
                                y as f32 + font_size * *dy_em as f32,
                                bbox.x as f32 + text_width,
                                color,
                                (font_size * *width_em as f32).max(0.3),
                                &[],
                                false,
                            );
                        }
                    } else {
                        let line_y =
                            match style.underline {
                                UnderlineType::Top => y as f32 - font_size + 1.0,
                                _ => y as f32
                                    + font_size
                                        * crate::renderer::text_decoration::UNDERLINE_BASELINE_RATIO
                                            as f32,
                            };
                        draw_line_shape(
                            bbox.x as f32,
                            line_y,
                            bbox.x as f32 + text_width,
                            color,
                            style.underline_shape,
                        );
                    }
                }
                if style.strikethrough && text_width > 0.0 {
                    let color = if style.strike_color != 0 {
                        colorref_to_skia(style.strike_color, 1.0)
                    } else {
                        colorref_to_skia(style.color, 1.0)
                    };
                    draw_line_shape(
                        bbox.x as f32,
                        y as f32 - font_size * 0.3,
                        bbox.x as f32 + text_width,
                        color,
                        style.strike_shape,
                    );
                }
                if style.emphasis_dot > 0 {
                    let dot = match style.emphasis_dot {
                        1 => "●",
                        2 => "○",
                        3 => "ˇ",
                        4 => "˜",
                        5 => "･",
                        6 => "˸",
                        _ => "",
                    };
                    if !dot.is_empty() {
                        let dot_size = font_size * 0.3;
                        let dot_y = y as f32 - font_size * 1.05;
                        if let Some(font) = font_for_text(dot, dot_size) {
                            let mut dot_paint = Paint::default();
                            dot_paint.set_anti_alias(true);
                            dot_paint.set_color(colorref_to_skia(style.color, 1.0));
                            for cx in &char_positions[..char_positions.len().saturating_sub(1)] {
                                canvas.draw_str(
                                    dot,
                                    (bbox.x as f32 + *cx as f32 + font_size * ratio * 0.5, dot_y),
                                    &font,
                                    &dot_paint,
                                );
                            }
                        }
                    }
                }
                for leader in &style.tab_leaders {
                    if leader.fill_type == 0 {
                        continue;
                    }
                    let x1 = bbox.x as f32 + leader.start_x as f32;
                    let leader_end_x =
                        clamp_tab_leader_end_x(text, &char_positions, leader, font_size as f64);
                    let x2 = bbox.x as f32 + leader_end_x as f32;
                    let line_y = y as f32 - font_size * 0.35;
                    let color = colorref_to_skia(style.color, 1.0);
                    match leader.fill_type {
                        1 => draw_styled_line(x1, line_y, x2, color, 0.5, &[], false),
                        2 => draw_styled_line(x1, line_y, x2, color, 0.5, &[3.0, 3.0], false),
                        3 => {
                            // 점선 — 두께·간격은 폰트 크기를 따른다 (svg.rs 와 같은 출처).
                            let (w, dash, gap) =
                                crate::renderer::render_tree::tab_dot_leader_stroke(
                                    font_size as f64,
                                );
                            draw_styled_line(
                                x1,
                                line_y,
                                x2,
                                color,
                                w as f32,
                                &[dash as f32, gap as f32],
                                true,
                            )
                        }
                        4 => draw_styled_line(
                            x1,
                            line_y,
                            x2,
                            color,
                            0.5,
                            &[6.0, 2.0, 1.0, 2.0],
                            false,
                        ),
                        5 => draw_styled_line(
                            x1,
                            line_y,
                            x2,
                            color,
                            0.5,
                            &[6.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                            false,
                        ),
                        6 => draw_styled_line(x1, line_y, x2, color, 0.5, &[8.0, 4.0], false),
                        7 => draw_styled_line(x1, line_y, x2, color, 0.7, &[0.1, 2.5], true),
                        8 => {
                            draw_styled_line(x1, line_y - 1.0, x2, color, 0.3, &[], false);
                            draw_styled_line(x1, line_y + 1.0, x2, color, 0.3, &[], false);
                        }
                        9 => {
                            draw_styled_line(x1, line_y - 1.2, x2, color, 0.3, &[], false);
                            draw_styled_line(x1, line_y + 0.8, x2, color, 0.8, &[], false);
                        }
                        10 => {
                            draw_styled_line(x1, line_y - 0.8, x2, color, 0.8, &[], false);
                            draw_styled_line(x1, line_y + 1.2, x2, color, 0.3, &[], false);
                        }
                        11 => {
                            draw_styled_line(x1, line_y - 2.0, x2, color, 0.3, &[], false);
                            draw_styled_line(x1, line_y, x2, color, 0.8, &[], false);
                            draw_styled_line(x1, line_y + 2.0, x2, color, 0.3, &[], false);
                        }
                        _ => draw_styled_line(x1, line_y, x2, color, 0.5, &[1.0, 2.0], false),
                    }
                }
                if effective_rotation != 0.0 {
                    canvas.restore();
                }
            };
        let draw_text_marks = |text: &str,
                               bbox: crate::renderer::render_tree::BoundingBox,
                               style: &crate::renderer::TextStyle,
                               baseline: f64,
                               rotation: f64,
                               is_vertical: bool,
                               is_marker: bool,
                               is_para_end: bool,
                               is_line_break_end: bool| {
            if !output_options.show_paragraph_marks && !output_options.show_control_codes {
                return;
            }
            let font_size = if style.font_size > 0.0 {
                style.font_size as f32
            } else {
                12.0
            };
            let make_mark_font = |size: f32| {
                let mut font = match_system_family_style(
                    self.font_mgr,
                    self.system_families,
                    "DejaVu Sans",
                    FontStyle::normal(),
                )
                .or_else(|| legacy_typeface_for_style(self.font_mgr, FontStyle::normal()))
                .map(|tf| Font::new(tf, size))
                .unwrap_or_else(|| {
                    let mut font = Font::default();
                    font.set_size(size);
                    font
                });
                font.set_edging(font::Edging::AntiAlias);
                font
            };
            let font = make_mark_font(font_size * 0.5);
            let mut mark_paint = Paint::default();
            mark_paint.set_anti_alias(true);
            mark_paint.set_color(Color::from_argb(255, 0, 102, 255));
            let y = if baseline > 0.0 {
                bbox.y + baseline
            } else {
                bbox.y + bbox.height
            };
            let effective_rotation = if is_vertical {
                rotation + 90.0
            } else {
                rotation
            };
            if effective_rotation != 0.0 {
                canvas.save();
                canvas.rotate(
                    effective_rotation as f32,
                    Some(
                        (
                            (bbox.x + bbox.width / 2.0) as f32,
                            (bbox.y + bbox.height / 2.0) as f32,
                        )
                            .into(),
                    ),
                );
            }
            if !text.is_empty() && !is_marker {
                let char_positions =
                    crate::renderer::replay_positions_or_compute(text, style, layout_positions);
                for (index, ch) in text.chars().enumerate() {
                    if ch == ' ' {
                        let x = bbox.x + char_positions.get(index).copied().unwrap_or(0.0);
                        let next_x = if index + 1 < char_positions.len() {
                            bbox.x + char_positions[index + 1]
                        } else {
                            bbox.x + bbox.width
                        };
                        let mark_x = ((x + next_x) / 2.0) as f32 - font_size * 0.125;
                        canvas.draw_str("\u{2228}", (mark_x, y as f32), &font, &mark_paint);
                    } else if ch == '\t' {
                        let mark_x = bbox.x as f32
                            + char_positions.get(index).copied().unwrap_or(0.0) as f32;
                        canvas.draw_str("\u{2192}", (mark_x, y as f32), &font, &mark_paint);
                    }
                }
            }
            if is_para_end || is_line_break_end {
                let end_font = make_mark_font(font_size);
                let mark = if is_line_break_end {
                    "\u{2193}"
                } else {
                    "\u{21B5}"
                };
                let mark_x = if text.is_empty() {
                    bbox.x as f32
                } else {
                    (bbox.x + bbox.width) as f32
                };
                canvas.draw_str(mark, (mark_x, y as f32), &end_font, &mark_paint);
            }
            if effective_rotation != 0.0 {
                canvas.restore();
            }
        };

        draw_text(
            text,
            bbox,
            style,
            baseline,
            rotation,
            is_vertical,
            char_overlap,
        );
        if render_marks {
            draw_text_marks(
                text,
                bbox,
                style,
                baseline,
                rotation,
                is_vertical,
                is_marker,
                is_para_end,
                is_line_break_end,
            );
        }
    }
}
