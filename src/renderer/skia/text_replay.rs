use skia_safe::{font, paint, Canvas, Color, Font, FontMgr, FontStyle, Paint, Rect, Typeface};

use crate::model::style::UnderlineType;
use crate::paint::LayerOutputOptions;
use crate::renderer::composer::{
    char_overlap_display_text, char_overlap_size_ratio, decode_pua_overlap_number,
    expand_pua_render_text_in_style, CharOverlapInfo,
};
use crate::renderer::layout::split_into_clusters;
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::{boxed_pua_char_overlap_semantics, clamp_tab_leader_end_x, TextStyle};

use super::font_catalog::TypefaceCatalog;
use super::font_lookup::{
    font_style_for_text, match_supplied_family, match_system_family_style,
    select_typeface_for_character, text_typeface_candidates, SystemFontFamilies,
};
use super::renderer::colorref_to_skia;
use super::synthetic_typeface::{draw_synthetic_bold_paths, unemboldened_typeface};

pub(super) mod fonts;
mod lines;
mod marks;
mod monochrome;
mod overlap;

pub(super) struct SkiaTextReplay<'a> {
    pub(super) canvas: &'a Canvas,
    pub(super) raster_target: bool,
    pub(super) geometric_text: bool,
    pub(super) hft_outline_enabled: bool,
    pub(super) font_mgr: &'a FontMgr,
    pub(super) text_shaper: &'a std::cell::OnceCell<Option<skia_safe::Shaper>>,
    pub(super) custom_typefaces: &'a TypefaceCatalog,
    pub(super) bundled_typefaces: &'a TypefaceCatalog,
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
        let draw_text =
            |text: &str,
             bbox: crate::renderer::render_tree::BoundingBox,
             style: &crate::renderer::TextStyle,
             baseline: f64,
             rotation: f64,
             _is_vertical: bool,
             char_overlap: Option<&crate::renderer::composer::CharOverlapInfo>| {
                if text.is_empty() && style.tab_leaders.is_empty() {
                    return;
                }
                let base_font_size = style.effective_font_size();
                // [#2771] 위첨자/아래첨자를 SVG/Canvas/HTML 과 동일한 계약(0.7 배
                // 글꼴 + baseline 이동)으로 그린다. 종전 skia 경로는 첨자 분기가
                // 아예 없어 본문과 같은 크기·같은 baseline 으로 그렸다.
                // baseline 이동은 아래 `y` 계산에서 함께 적용한다.
                let (draw_font_size, _) = style.script_draw_metrics(base_font_size, 0.0);
                let font_size = draw_font_size as f32;
                let font_style = font_style_for_text(style);
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
                    &crate::renderer::native_font_family_for_style(style),
                    font_style,
                );
                let hft_outline = (self.hft_outline_enabled
                    && crate::renderer::hft_outline::supports_style(style))
                .then(|| {
                    let family = crate::renderer::hft_outline::first_family(&style.font_family);
                    let face = match_supplied_family(self.custom_typefaces, family, font_style)
                        .or_else(|| {
                            match_system_family_style(
                                self.font_mgr,
                                self.system_families,
                                family,
                                font_style,
                            )
                        })
                        .or_else(|| {
                            match_supplied_family(self.bundled_typefaces, family, font_style)
                        })?;
                    super::hft_outline::HftOutlineReplay::new(&face)
                })
                .flatten();
                let font_for_text = |sample: &str, size: f32| {
                    if let Some(family) =
                        crate::renderer::measured_glyph_supplier::family(style, sample)
                    {
                        let (_, chain) = text_typeface_candidates(
                            self.font_mgr,
                            self.system_families,
                            self.custom_typefaces,
                            self.bundled_typefaces,
                            family,
                            font_style,
                        );
                        return fonts::for_text(&chain, sample, size, style, self.font_mgr, self.geometric_text);
                    }
                    if crate::renderer::pua_glyph::candidate(sample).is_some() {
                        if let Some(primary) = typeface_chain.first() {
                            let font = Font::new(primary.typeface.clone(), size);
                            if let Some(glyph) = fonts::compatibility_glyph(&font, sample) {
                                return fonts::for_text(
                                    &typeface_chain[..1],
                                    &glyph.to_string(),
                                    size,
                                    style,
                                    self.font_mgr,
                                    self.geometric_text,
                                );
                            }
                        }
                    }
                    fonts::for_text(&typeface_chain, sample, size, style, self.font_mgr, self.geometric_text)
                };
                let baseline_y = if baseline > 0.0 {
                    bbox.y + baseline
                } else {
                    bbox.y + bbox.height
                };
                // [#2771] 첨자 baseline 이동 (위 0.3em / 아래 0.15em). 비첨자는 항등.
                let (_, y) = style.script_draw_metrics(base_font_size, baseline_y);
                // Vertical layout already emits upright glyph bboxes or an
                // explicit sideways rotation; applying another 90° rotates
                // upright glyphs and double-rotates sideways runs.
                let effective_rotation = rotation;
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

                let metric_source = text;
                let text = expand_pua_render_text_in_style(text, style);
                let text = text.as_str();
                let metric_text =
                    crate::renderer::composer::display_projection_metric_text(metric_source, text);
                let char_positions = crate::renderer::replay_positions_or_compute(
                    metric_text,
                    style,
                    layout_positions,
                );
                let fit_positions = crate::renderer::glyph_fit_positions(
                    metric_text,
                    style,
                    layout_positions,
                    &char_positions,
                );
                let visual_positions =
                    crate::renderer::text_direction::VisualTextPositions::for_style(
                        text,
                        &char_positions,
                        style,
                    );
                let span_x = |start: usize, end: usize| {
                    visual_positions
                        .as_ref()
                        .and_then(|positions| positions.span_origin(start, end))
                        .unwrap_or_else(|| char_positions.get(start).copied().unwrap_or(0.0))
                };
                let clusters = split_into_clusters(text);
                let trailing = text.chars().rev().take_while(|ch| *ch == ' ').count();
                let trim = trim_trailing_spaces.min(trailing);
                let text_width = char_positions
                    .get(text.chars().count().saturating_sub(trim))
                    .copied()
                    .unwrap_or_else(|| *char_positions.last().unwrap_or(&0.0))
                    as f32;
                // 장평은 가로에만 적용한다. 세로 크기와 공통 그리기 비율을 함께 소비한다.
                let (font_size, ratio) = {
                    let (fs, r) = style.ratio_draw_params(font_size as f64);
                    (fs as f32, r as f32)
                };
                let has_ratio = (ratio - 1.0).abs() > 0.01;
                let shaped_spans = if suppress_glyphs {
                    Vec::new()
                } else {
                    super::text_shaping::prepare_shaped_spans(
                        &clusters,
                        visual_positions.as_ref(),
                        self.text_shaper,
                        self.font_mgr,
                        |text| font_for_text(text, font_size),
                    )
                };
                let segments = visual_positions
                    .as_ref()
                    .map(|p| p.segments(text.chars().count().saturating_sub(trim)))
                    .unwrap_or_else(|| vec![(0.0, f64::from(text_width))]);
                if !suppress_glyphs
                    && crate::model::color::char_shade(style.shade_color).is_some()
                    && text_width > 0.0
                {
                    let mut shade = Paint::default();
                    shade.set_anti_alias(true);
                    shade.set_style(paint::Style::Fill);
                    shade.set_color(colorref_to_skia(style.shade_color, 1.0));
                    for &(left, right) in &segments {
                        canvas.draw_rect(
                            Rect::from_xywh(
                                bbox.x as f32 + left as f32,
                                y as f32 - font_size,
                                (right - left) as f32,
                                font_size * 1.2,
                            ),
                            &shade,
                        );
                    }
                }

                let lines = lines::TextLines { canvas };
                // 연속 하이픈은 낱글자로 그리되 원 획이 저장 간격보다 넓어
                // 겹치는 경우에는 낱글자별 짧은 획을 사용한다.
                let cluster_advance = |char_idx: usize, cluster: &str| -> f32 {
                    let end = char_idx + cluster.chars().count();
                    if end < char_positions.len() {
                        (char_positions[end] - char_positions[char_idx]) as f32
                    } else {
                        0.0
                    }
                };
                let is_middle_dot = crate::renderer::render_tree::is_vector_middle_dot;
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
                    let mut spans = shaped_spans.iter().peekable();
                    for (cluster_index, (char_idx, cluster)) in clusters.iter().enumerate() {
                        if spans.peek().is_some_and(|span| cluster_index >= span.end) {
                            spans.next();
                        }
                        if let Some(span) = spans.peek().filter(|span| cluster_index >= span.start)
                        {
                            if cluster_index == span.start {
                                // 공백·원문 전진은 유지하고 blob은 방향 구간의 실제 표시 원점에 그린다.
                                let (end_start, end_text) = &clusters[span.end - 1];
                                let end = end_start + end_text.chars().count();
                                let char_x = bbox.x as f32 + span_x(*char_idx, end) as f32 + dx;
                                canvas.save();
                                canvas.translate((char_x, y as f32 + dy + span.baseline_offset));
                                if has_ratio {
                                    canvas.scale((ratio, 1.0));
                                }
                                canvas.draw_text_blob(&span.blob, (0.0, 0.0), &text_paint);
                                canvas.restore();
                            }
                            continue;
                        }
                        let painted = visual_positions
                            .as_ref()
                            .map(|p| p.mirrored(cluster, *char_idx))
                            .unwrap_or(std::borrow::Cow::Borrowed(cluster.as_str()));
                        let cluster = painted.as_ref();
                        // 공백·개체 마커는 원문 위치와 장식을 보존하되 글리프를 그리지 않는다.
                        // U+FFFC의 실제 개체는 별도 paint op가 소유한다. 문자 인덱스를
                        // 제거하면 뒤 cluster의 저장 위치가 달라지므로 그대로 건너뛴다.
                        if cluster.starts_with('\u{FFFC}')
                            || cluster.chars().all(char::is_whitespace)
                        {
                            continue;
                        }
                        let cluster_offset = span_x(*char_idx, *char_idx + cluster.chars().count());
                        let char_x = bbox.x as f32 + cluster_offset as f32 + dx;
                        if cluster == "-" {
                            if let Some((start, end, y_offset, stroke)) =
                                crate::renderer::overlapping_dash_leader_segment(
                                    text,
                                    style,
                                    *char_idx,
                                    &char_positions,
                                    f64::from(font_size),
                                )
                            {
                                let line_y = y as f32 + y_offset as f32 + dy;
                                let mut line_paint = Paint::default();
                                line_paint.set_anti_alias(true);
                                line_paint.set_color(color);
                                line_paint.set_style(paint::Style::Stroke);
                                line_paint.set_stroke_width(stroke as f32);
                                canvas.draw_line(
                                    (char_x + start as f32, line_y),
                                    (char_x + end as f32, line_y),
                                    &line_paint,
                                );
                                continue;
                            }
                        }
                        if cluster.starts_with(|ch: char| {
                            ch < '\u{0020}' && !matches!(ch, '\t' | '\n' | '\r')
                        }) {
                            continue;
                        }
                        if let Some((cx, cy, rx, ry)) =
                            crate::renderer::legacy_hft_bullet_geometry(cluster, style)
                        {
                            let left = bbox.x + cluster_offset + cx + f64::from(dx);
                            let top = y + cy + f64::from(dy);
                            canvas.draw_oval(
                                skia_safe::Rect::from_xywh(
                                    (left - rx) as f32,
                                    (top - ry) as f32,
                                    (2.0 * rx) as f32,
                                    (2.0 * ry) as f32,
                                ),
                                &text_paint,
                            );
                            continue;
                        }
                        if cluster == "․" {
                            if let Some(ink) = crate::renderer::render_tree::fullwidth_onepoint_ink(
                                style,
                                font_size as f64,
                            ) {
                                let cx = bbox.x
                                    + char_positions.get(*char_idx).copied().unwrap_or(0.0)
                                    + ink.center_x
                                    + dx as f64;
                                let cy = y + dy as f64 - ink.above_baseline;
                                canvas.draw_oval(
                                    Rect::from_xywh(
                                        (cx - ink.radius_x) as f32,
                                        (cy - ink.radius_y) as f32,
                                        (2.0 * ink.radius_x) as f32,
                                        (2.0 * ink.radius_y) as f32,
                                    ),
                                    &text_paint,
                                );
                                continue;
                            }
                        }
                        if is_middle_dot(cluster) {
                            let advance = cluster_advance(*char_idx, cluster);
                            let cx = bbox.x as f32 + cluster_offset as f32 + advance / 2.0 + dx;
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
                                let baseline_y = y as f32 + dy;
                                if let Some(g) = crate::renderer::boxed_digit_geometry::single_digit(
                                    number,
                                    f64::from(font_size),
                                ) {
                                    let family = format!(
                                        "{},{}",
                                        crate::renderer::boxed_digit_geometry::number_face(),
                                        style.font_family
                                    );
                                    let (_, number_faces) = text_typeface_candidates(
                                        self.font_mgr,
                                        self.system_families,
                                        self.custom_typefaces,
                                        self.bundled_typefaces,
                                        &family,
                                        FontStyle::normal(),
                                    );
                                    if let Some(face) = select_typeface_for_character(
                                        &number_faces,
                                        char::from_digit(number, 10).unwrap(),
                                    ) {
                                        let mut number_font =
                                            Font::new(face.typeface.clone(), font_size);
                                        number_font.set_edging(font::Edging::AntiAlias);
                                        fonts::preserve_geometry(
                                            &mut number_font,
                                            self.geometric_text,
                                        );
                                        let mut frame_paint = Paint::default();
                                        frame_paint.set_anti_alias(true);
                                        frame_paint.set_style(paint::Style::Stroke);
                                        frame_paint.set_stroke_width(g.stroke_width as f32);
                                        frame_paint.set_color(color);
                                        canvas.draw_rect(
                                            Rect::from_xywh(
                                                char_x + g.frame_x as f32,
                                                baseline_y + g.frame_y as f32,
                                                g.frame_width as f32,
                                                g.frame_height as f32,
                                            ),
                                            &frame_paint,
                                        );
                                        canvas.save();
                                        canvas.translate((
                                            char_x + g.number_x as f32,
                                            baseline_y + g.number_y as f32,
                                        ));
                                        canvas.scale((
                                            g.number_scale_x as f32,
                                            g.number_scale_y as f32,
                                        ));
                                        canvas.draw_str(
                                            number.to_string(),
                                            (0.0, 0.0),
                                            &number_font,
                                            &text_paint,
                                        );
                                        canvas.restore();
                                        continue;
                                    }
                                }
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
                        if let Some(mut font) = font_for_text(cluster, font_size) {
                            // 독립 Print의 regular 프로그램 합성 굵기는 같은 윤곽선의
                            // fill+stroke(0.02em)다. 실제 bold face와 outline pass는 보존한다.
                            let regular = (style.bold && stroke_width == 0.0)
                                .then(|| unemboldened_typeface(self.font_mgr, &font.typeface()))
                                .flatten();
                            let synthetic_bold = regular.is_some();
                            let bold_paint = regular.map(|typeface| {
                                font.set_typeface(typeface);
                                font.set_embolden(false);
                                let mut paint = text_paint.clone();
                                paint.set_style(paint::Style::StrokeAndFill);
                                paint.set_stroke_width(font_size * 0.02);
                                paint.set_stroke_miter(10.0);
                                paint
                            });
                            let glyph_paint = bold_paint.as_ref().unwrap_or(&text_paint);
                            let char_y = y as f32 + dy;
                            let compatibility =
                                fonts::compatibility_glyph(&font, cluster).map(|ch| ch.to_string());
                            let glyph_text = compatibility.as_deref().unwrap_or(cluster);
                            // 측정에서 폭을 소유한 경로가 paint의 축소도 결정한다.
                            // Unicode 범위만으로 이미 좁은 자연 글리프를 다시 줄이지 않는다.
                            let needs_halfwidth_scale = !has_ratio
                                && cluster.chars().next().is_some_and(|ch| {
                                    matches!(
                                        ch,
                                        '\u{2018}'
                                            ..='\u{2027}' | '\u{00B7}' | '\u{300C}' | '\u{300D}'
                                    ) && matches!(
                                        crate::renderer::layout::char_width_decision(
                                            &[ch],
                                            &[1],
                                            0,
                                            style,
                                            cluster.chars().count() == 1,
                                        )
                                        .width_source,
                                        "metricNarrowPunctuationOverlay"
                                            | "metricHalfwidthPunctuationOverlay"
                                    )
                                })
                                && !has_ratio;
                            if needs_halfwidth_scale {
                                let end = *char_idx + cluster.chars().count();
                                let advance = char_positions.get(end).copied().unwrap_or(0.0)
                                    - char_positions.get(*char_idx).copied().unwrap_or(0.0);
                                let scale = crate::renderer::halfwidth_punct_paint_scale(
                                    advance,
                                    f64::from(font.measure_str(glyph_text, None).0),
                                ) as f32;
                                if synthetic_bold {
                                    // 글자 윤곽선만 반각으로 하고 사용자 좌표 획은 유지한다.
                                    font.set_scale_x(scale);
                                    if !draw_synthetic_bold_paths(
                                        canvas,
                                        glyph_text,
                                        &font,
                                        (char_x, char_y),
                                        glyph_paint,
                                        self.raster_target,
                                    ) {
                                        canvas.draw_str(
                                            glyph_text,
                                            (char_x, char_y),
                                            &font,
                                            glyph_paint,
                                        );
                                    }
                                    continue;
                                }
                                canvas.save();
                                canvas.translate((char_x, char_y));
                                canvas.scale((scale, 1.0));
                                canvas.draw_str(glyph_text, (0.0, 0.0), &font, glyph_paint);
                                canvas.restore();
                                continue;
                            }
                            // 위치와 윤곽 폭을 분리한 공통 경계를 Canvas와 같이 쓴다.
                            // 양수 자간·굵기 tracking은 글리프를 늘리지 않으며,
                            // 확정 원점은 char_positions에 그대로 남는다.
                            let pin_ascii_advance = cluster.chars().any(|ch| {
                                ch.is_ascii_alphanumeric()
                                    || (style.fixed_glyph_advances && ch.is_ascii_graphic())
                            });
                            let (fit_scale, fit_dx) = fit_positions
                                .get(*char_idx)
                                .and_then(|start| {
                                    let end = *char_idx + cluster.chars().count();
                                    let advance = fit_positions.get(end)? - start;
                                    let (natural_width, ink) = font.measure_str(glyph_text, None);
                                    crate::renderer::canvas_cluster_fit(
                                        style,
                                        cluster,
                                        advance,
                                        f64::from(natural_width * ratio),
                                        Some(f64::from(ink.width() * ratio)),
                                        pin_ascii_advance,
                                    )
                                })
                                .map_or((1.0, 0.0), |(scale, dx)| (scale as f32, dx as f32));
                            if synthetic_bold {
                                // 글리프의 장평·원본 폭만 맞추고 합성 획의 사용자 좌표 폭은 유지한다.
                                font.set_scale_x(ratio * fit_scale);
                                if !draw_synthetic_bold_paths(
                                    canvas,
                                    glyph_text,
                                    &font,
                                    (char_x + fit_dx, char_y),
                                    glyph_paint,
                                    self.raster_target,
                                ) {
                                    canvas.draw_str(
                                        glyph_text,
                                        (char_x + fit_dx, char_y),
                                        &font,
                                        glyph_paint,
                                    );
                                }
                                continue;
                            }
                            if has_ratio
                                || (fit_scale - 1.0).abs() > f32::EPSILON
                                || fit_dx.abs() > f32::EPSILON
                            {
                                canvas.save();
                                canvas.translate((char_x + fit_dx, char_y));
                                canvas.scale((ratio * fit_scale, 1.0));
                                if !hft_outline.as_ref().is_some_and(|outline| {
                                    outline.draw(canvas, glyph_text, &font, (0.0, 0.0), glyph_paint)
                                }) {
                                    canvas.draw_str(glyph_text, (0.0, 0.0), &font, glyph_paint);
                                }
                                canvas.restore();
                            } else if !hft_outline.as_ref().is_some_and(|outline| {
                                outline.draw(
                                    canvas,
                                    glyph_text,
                                    &font,
                                    (char_x, char_y),
                                    glyph_paint,
                                )
                            }) {
                                canvas.draw_str(glyph_text, (char_x, char_y), &font, glyph_paint);
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

                for &(left, right) in &segments {
                    let start_x = bbox.x as f32 + left as f32;
                    let end_x = bbox.x as f32 + right as f32;
                    if !matches!(style.underline, UnderlineType::None) && text_width > 0.0 {
                        // COLORREF 0 은 미지정이 아니라 검정 — svg.rs 와 같은 계약.
                        let color = colorref_to_skia(style.underline_color, 1.0);
                        // [#5730] 아래 밑줄은 기준선 + 0.17em (한글 2022 프로브 실측) —
                        // 이중/삼중선(shape 7~10)은 em 비례 실측표를 따른다.
                        // SVG 백엔드(renderer/svg.rs)와 같은 표(text_decoration)를 소비한다.
                        let top = crate::renderer::text_decoration::top_underline_geometry(
                            style,
                            font_size as f64,
                        );
                        let multi = crate::renderer::text_decoration::underline_multi_lines(
                            style.underline_shape,
                        )
                        .filter(|_| style.underline != UnderlineType::Top);
                        if let Some(top) = top {
                            lines.top_underline(start_x, y as f32, end_x, color, &top, left);
                        } else if let Some(multi_lines) = multi {
                            for (dy_em, width_em) in multi_lines {
                                lines.styled(
                                    start_x,
                                    y as f32 + font_size * *dy_em as f32,
                                    end_x,
                                    color,
                                    (font_size * *width_em as f32).max(0.3),
                                    &[],
                                    false,
                                );
                            }
                        } else {
                            let line_y = match style.underline {
                                UnderlineType::Top => y as f32 - font_size + 1.0,
                                _ => y as f32
                                    + font_size
                                        * crate::renderer::text_decoration::UNDERLINE_BASELINE_RATIO
                                            as f32,
                            };
                            lines.shape(start_x, line_y, end_x, color, style.underline_shape);
                        }
                    }
                    if style.strikethrough && text_width > 0.0 {
                        let color = if style.strike_color != 0 {
                            colorref_to_skia(style.strike_color, 1.0)
                        } else {
                            colorref_to_skia(style.color, 1.0)
                        };
                        lines.shape(
                            start_x,
                            y as f32 - font_size * 0.3,
                            end_x,
                            color,
                            style.strike_shape,
                        );
                    }
                }
                if let Some(geometry) = crate::renderer::emphasis::geometry(
                    text,
                    &char_positions,
                    style,
                    font_size as f64,
                ) {
                    let mark_style = TextStyle {
                        font_family: geometry.font_family.to_owned(),
                        bold: geometry.bold,
                        ..TextStyle::default()
                    };
                    let (_, faces) = text_typeface_candidates(
                        self.font_mgr,
                        self.system_families,
                        self.custom_typefaces,
                        self.bundled_typefaces,
                        geometry.font_family,
                        font_style_for_text(&mark_style),
                    );
                    if let Some(mut font) = fonts::for_text(
                        &faces,
                        geometry.glyph,
                        font_size,
                        &mark_style,
                        self.font_mgr,
                    ) {
                        font.set_scale_x(geometry.ratio as f32);
                        font.set_skew_x((geometry.skew_x * geometry.ratio) as f32);
                        let mut paint = Paint::default();
                        paint.set_anti_alias(true);
                        paint.set_color(colorref_to_skia(style.color, 1.0));
                        for x in geometry.positions {
                            canvas.draw_str(
                                geometry.glyph,
                                ((bbox.x + x) as f32, (y + geometry.baseline_offset) as f32),
                                &font,
                                &paint,
                            );
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
                        1 => lines.styled(x1, line_y, x2, color, 0.5, &[], false),
                        2 => lines.styled(x1, line_y, x2, color, 0.5, &[3.0, 3.0], false),
                        3 => {
                            if let Some(dots) = crate::renderer::tab_leader::dot_geometry(
                                font_size as f64,
                                leader.start_x,
                                leader_end_x,
                            ) {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_color(color);
                                for center in dots.centers {
                                    canvas.draw_circle(
                                        ((bbox.x + center) as f32, line_y),
                                        dots.radius as f32,
                                        &paint,
                                    );
                                }
                                continue;
                            }
                            let (w, dash, gap) =
                                crate::renderer::render_tree::tab_dot_leader_stroke(
                                    font_size as f64,
                                );
                            lines.styled(
                                x1,
                                line_y,
                                x2,
                                color,
                                w as f32,
                                &[dash as f32, gap as f32],
                                true,
                            )
                        }
                        4 => lines.styled(x1, line_y, x2, color, 0.5, &[6.0, 2.0, 1.0, 2.0], false),
                        5 => lines.styled(
                            x1,
                            line_y,
                            x2,
                            color,
                            0.5,
                            &[6.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                            false,
                        ),
                        6 => lines.styled(x1, line_y, x2, color, 0.5, &[8.0, 4.0], false),
                        7 => {
                            let (w, dash, gap) =
                                crate::renderer::render_tree::tab_circle_leader_stroke();
                            lines.styled(
                                x1,
                                line_y,
                                x2,
                                color,
                                w as f32,
                                &[dash as f32, gap as f32],
                                true,
                            )
                        }
                        8 => {
                            lines.styled(x1, line_y - 1.0, x2, color, 0.3, &[], false);
                            lines.styled(x1, line_y + 1.0, x2, color, 0.3, &[], false);
                        }
                        9 => {
                            lines.styled(x1, line_y - 1.2, x2, color, 0.3, &[], false);
                            lines.styled(x1, line_y + 0.8, x2, color, 0.8, &[], false);
                        }
                        10 => {
                            lines.styled(x1, line_y - 0.8, x2, color, 0.8, &[], false);
                            lines.styled(x1, line_y + 1.2, x2, color, 0.3, &[], false);
                        }
                        11 => {
                            lines.styled(x1, line_y - 2.0, x2, color, 0.3, &[], false);
                            lines.styled(x1, line_y, x2, color, 0.8, &[], false);
                            lines.styled(x1, line_y + 2.0, x2, color, 0.3, &[], false);
                        }
                        _ => lines.styled(x1, line_y, x2, color, 0.5, &[1.0, 2.0], false),
                    }
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
            marks::draw(
                self,
                text,
                bbox,
                style,
                baseline,
                rotation,
                is_marker,
                is_para_end,
                is_line_break_end,
                layout_positions,
            );
        }
    }
}
