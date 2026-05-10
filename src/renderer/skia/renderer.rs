use skia_safe::{
    font, paint, surfaces, Canvas, Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint,
    PathBuilder, PathEffect, RRect, Rect, Typeface,
};
use std::collections::HashMap;

use crate::error::HwpError;
use crate::model::image::ImageEffect;
use crate::model::style::UnderlineType;
use crate::model::ColorRef;
use crate::paint::{LayerNode, LayerNodeKind, LayerOutputOptions, PageLayerTree, PaintOp};
use crate::renderer::composer::{decode_pua_overlap_number, pua_to_display_text};
use crate::renderer::layer_renderer::{
    LayerRasterRenderer, LayerRenderResult, RasterOutputFormat, RasterRenderOptions,
    RasterRenderOutput,
};
use crate::renderer::layout::{compute_char_positions, split_into_clusters};
use crate::renderer::{svg_arc_to_beziers, LineStyle, PathCommand, ShapeStyle, StrokeDash};

use super::equation_conv::render_equation;
use super::image_conv::{draw_image_bytes, draw_svg_fragment, ImageSampling};

pub struct SkiaLayerRenderer {
    font_mgr: FontMgr,
    /// 사용자 지정 폰트 디렉토리에서 미리 로드한 폰트 캐시.
    /// key = primary face name (Typeface::family_name), value = Typeface.
    /// SVG 의 `--font-path` 와 같은 패턴으로 ttfs 디렉토리의 한컴 전용 폰트 (HY견명조 등) 도 사용 가능.
    custom_typefaces: HashMap<String, Typeface>,
}

impl SkiaLayerRenderer {
    pub fn new() -> Self {
        Self {
            font_mgr: FontMgr::default(),
            custom_typefaces: HashMap::new(),
        }
    }

    /// 사용자 지정 폰트 디렉토리 (ttfs 등) 의 폰트를 로드하여 Skia 가 직접 사용 가능하게 한다.
    /// SVG 의 `--font-path` 와 동일한 패턴.
    pub fn with_font_paths(mut self, font_paths: &[std::path::PathBuf]) -> Self {
        let mut search_dirs: Vec<std::path::PathBuf> = font_paths.to_vec();
        for dir in &["ttfs/hwp", "ttfs/windows", "ttfs"] {
            search_dirs.push(std::path::PathBuf::from(dir));
        }
        for dir in &search_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_lowercase());
                    if !matches!(ext.as_deref(), Some("ttf") | Some("otf") | Some("ttc")) {
                        continue;
                    }
                    if let Ok(data) = std::fs::read(&path) {
                        let skia_data = skia_safe::Data::new_copy(&data);
                        if let Some(typeface) = self.font_mgr.new_from_data(&skia_data, None) {
                            let family = typeface.family_name();
                            self.custom_typefaces.entry(family).or_insert(typeface);
                        }
                    }
                }
            }
        }
        self
    }

    pub fn render_raster_with_options(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        if let Some(dpi) = options.dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                return Err(HwpError::RenderError(format!("invalid raster dpi: {dpi}")));
            }
        }
        if options.format != RasterOutputFormat::Png {
            return Err(HwpError::RenderError(
                "Skia raster renderer currently supports PNG output".to_string(),
            ));
        }

        let raster_dimension = |value: f64, label: &str| -> LayerRenderResult<i32> {
            if !value.is_finite() || value <= 0.0 {
                return Err(HwpError::RenderError(format!(
                    "invalid page {label}: {value}"
                )));
            }
            if !options.scale.is_finite() || options.scale <= 0.0 {
                return Err(HwpError::RenderError(format!(
                    "invalid raster scale: {}",
                    options.scale
                )));
            }
            if options.max_dimension <= 0 {
                return Err(HwpError::RenderError(format!(
                    "invalid raster max dimension: {}",
                    options.max_dimension
                )));
            }
            let scaled = (value * options.scale).ceil();
            if !scaled.is_finite() || scaled <= 0.0 || scaled > options.max_dimension as f64 {
                return Err(HwpError::RenderError(format!(
                    "raster {label} out of range: {scaled}"
                )));
            }
            Ok(scaled as i32)
        };
        let width = raster_dimension(tree.page_width, "width")?;
        let height = raster_dimension(tree.page_height, "height")?;
        if options.max_pixels == 0 {
            return Err(HwpError::RenderError(
                "invalid raster max pixel count: 0".to_string(),
            ));
        }
        let pixel_count = (width as u64)
            .checked_mul(height as u64)
            .ok_or_else(|| HwpError::RenderError("raster pixel count overflow".to_string()))?;
        if pixel_count > options.max_pixels {
            return Err(HwpError::RenderError(format!(
                "raster pixel count out of range: {pixel_count}"
            )));
        }

        let mut surface = surfaces::raster_n32_premul((width, height))
            .ok_or_else(|| HwpError::RenderError("Skia raster surface 생성 실패".to_string()))?;
        let canvas = surface.canvas();
        let clear_color = if let Some(color) = options.background_color {
            colorref_to_skia(color, 1.0)
        } else if options.transparent {
            Color::from_argb(0, 0, 0, 0)
        } else {
            Color::WHITE
        };
        canvas.clear(clear_color);
        if options.scale != 1.0 {
            canvas.scale((options.scale as f32, options.scale as f32));
        }
        self.render_node(canvas, &tree.root, &tree.output_options);

        let image = surface.image_snapshot();
        let data = image
            .encode(None, EncodedImageFormat::PNG, None)
            .ok_or_else(|| HwpError::RenderError("Skia PNG 인코딩 실패".to_string()))?;
        Ok(RasterRenderOutput {
            bytes: data.as_bytes().to_vec(),
            format: RasterOutputFormat::Png,
            width,
            height,
            dpi: options.dpi,
            color_space: options.color_space,
        })
    }

    fn render_node(&self, canvas: &Canvas, node: &LayerNode, output_options: &LayerOutputOptions) {
        let clip_enabled = output_options.clip_enabled;
        let apply_dash = |paint: &mut Paint, dash: StrokeDash| {
            let base_width = paint.stroke_width().max(1.0);
            let intervals: Option<[f32; 6]> = match dash {
                StrokeDash::Solid => None,
                StrokeDash::Dash => Some([6.0, 3.0, 0.0, 0.0, 0.0, 0.0]),
                StrokeDash::Dot => Some([2.0, 2.0, 0.0, 0.0, 0.0, 0.0]),
                StrokeDash::DashDot => Some([6.0, 3.0, 2.0, 3.0, 0.0, 0.0]),
                StrokeDash::DashDotDot => Some([6.0, 3.0, 2.0, 3.0, 2.0, 3.0]),
            };
            if let Some(intervals) = intervals {
                let intervals = intervals
                    .into_iter()
                    .filter(|value| *value > 0.0)
                    .map(|value| value * base_width)
                    .collect::<Vec<_>>();
                if let Some(effect) = PathEffect::dash(&intervals, 0.0) {
                    paint.set_path_effect(effect);
                }
            }
        };
        let make_fill_paint = |style: &ShapeStyle| -> Option<Paint> {
            let color = style
                .pattern
                .map(|pattern| pattern.background_color)
                .or(style.fill_color)?;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Fill);
            paint.set_color(colorref_to_skia(color, style.opacity as f32));
            Some(paint)
        };
        let make_stroke_paint = |style: &ShapeStyle| -> Option<Paint> {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Stroke);
            paint.set_stroke_width(if style.stroke_width > 0.0 {
                style.stroke_width as f32
            } else {
                1.0
            });
            paint.set_color(colorref_to_skia(style.stroke_color?, style.opacity as f32));
            apply_dash(&mut paint, style.stroke_dash);
            Some(paint)
        };
        let make_line_paint = |style: &LineStyle| {
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
        };
        let draw_placeholder = |bbox: crate::renderer::render_tree::BoundingBox, label: &str| {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return;
            }
            let rect = Rect::from_xywh(
                bbox.x as f32,
                bbox.y as f32,
                bbox.width as f32,
                bbox.height as f32,
            );
            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(Color::from_argb(48, 96, 96, 96));
            canvas.draw_rect(rect, &fill);
            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(Color::from_argb(160, 96, 96, 96));
            canvas.draw_rect(rect, &stroke);
            let mut font = Font::default();
            font.set_size(10.0);
            let mut text = Paint::default();
            text.set_anti_alias(true);
            text.set_color(Color::from_argb(220, 64, 64, 64));
            canvas.draw_str(
                label,
                (bbox.x as f32 + 4.0, (bbox.y + bbox.height / 2.0) as f32),
                &font,
                &text,
            );
        };
        let draw_image = |data: &[u8],
                          bbox: crate::renderer::render_tree::BoundingBox,
                          fill_mode,
                          original_size,
                          crop,
                          effect| {
            draw_image_bytes(
                canvas,
                data,
                bbox.x as f32,
                bbox.y as f32,
                bbox.width as f32,
                bbox.height as f32,
                fill_mode,
                original_size,
                crop,
                effect,
                ImageSampling::linear(),
            );
        };
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
                let font_size = if style.font_size > 0.0 {
                    style.font_size as f32
                } else {
                    12.0
                };
                let font_style = match (style.bold, style.italic) {
                    (true, true) => FontStyle::bold_italic(),
                    (true, false) => FontStyle::bold(),
                    (false, true) => FontStyle::italic(),
                    (false, false) => FontStyle::normal(),
                };
                let mut families = Vec::new();
                if !style.font_family.trim().is_empty() {
                    families.push(style.font_family.as_str());
                }
                // 한글 fallback (CJK glyph 미보유 폰트로 fallback 시 사각형 방지).
                // SVG 경로의 CSS font chain 과 동일한 한글 폴백 폰트 순서.
                families.extend([
                    "Noto Sans KR",
                    "Noto Serif KR",
                    "Noto Sans CJK KR",
                    "Noto Serif CJK KR",
                    "Nanum Gothic",
                    "Nanum Myeongjo",
                    "Malgun Gothic",
                    "맑은 고딕",
                    "Batang",
                    "바탕",
                    "Apple SD Gothic Neo",
                    "AppleMyungjo",
                    "DejaVu Sans",
                    "Arial",
                    "sans-serif",
                ]);
                // 1) 사용자 지정 폰트 (--font-path) 우선 검색
                // 2) 시스템 FontMgr 검색 (한글 fallback chain 포함)
                // 3) 마지막 fallback (legacy_make_typeface)
                //
                // 모든 후보를 chain 으로 보존 — char 단위 fallback 에 사용.
                let typeface_chain: Vec<Typeface> = {
                    let mut chain: Vec<Typeface> = Vec::new();
                    let mut seen: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    let mut push = |chain: &mut Vec<Typeface>,
                                    seen: &mut std::collections::HashSet<String>,
                                    tf: Typeface| {
                        let key = tf.family_name();
                        if seen.insert(key) {
                            chain.push(tf);
                        }
                    };
                    for family in &families {
                        if let Some(tf) = self.custom_typefaces.get(*family).cloned() {
                            push(&mut chain, &mut seen, tf);
                        }
                    }
                    for family in &families {
                        if let Some(tf) = self.font_mgr.match_family_style(family, font_style) {
                            push(&mut chain, &mut seen, tf);
                        }
                    }
                    if let Some(tf) = self.font_mgr.legacy_make_typeface(None::<&str>, font_style) {
                        push(&mut chain, &mut seen, tf);
                    }
                    chain
                };
                let primary_typeface = typeface_chain.first().cloned();
                let font_for_text = |sample: &str, size: f32| -> Option<Font> {
                    let visible_char = sample.chars().find(|ch| !ch.is_whitespace());
                    if let Some(ch) = visible_char {
                        let codepoint = ch as i32;
                        if let Some(tf) = typeface_chain
                            .iter()
                            .find(|tf| tf.unichar_to_glyph(codepoint) != 0)
                            .cloned()
                        {
                            let mut font = Font::new(tf, size);
                            font.set_edging(font::Edging::AntiAlias);
                            return Some(font);
                        }
                        return None;
                    }
                    if let Some(tf) = primary_typeface.clone() {
                        let mut font = Font::new(tf, size);
                        font.set_edging(font::Edging::AntiAlias);
                        Some(font)
                    } else {
                        let mut font = Font::default();
                        font.set_size(size);
                        font.set_edging(font::Edging::AntiAlias);
                        Some(font)
                    }
                };
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

                if let Some(overlap) = char_overlap {
                    let chars: Vec<char> = text.chars().collect();
                    if chars.is_empty() {
                        if effective_rotation != 0.0 {
                            canvas.restore();
                        }
                        return;
                    }

                    let size_ratio = if overlap.inner_char_size > 0 {
                        overlap.inner_char_size as f32 / 100.0
                    } else {
                        1.0
                    };
                    let inner_size = (font_size * size_ratio).max(1.0);
                    let box_size = font_size.max(1.0);
                    let is_combined = decode_pua_overlap_number(&chars);
                    let effective_border = if overlap.border_type == 0 && is_combined.is_some() {
                        1
                    } else {
                        overlap.border_type
                    };
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
                    } else {
                        let char_advance = if chars.len() > 1 {
                            bbox.width as f32 / chars.len() as f32
                        } else {
                            box_size
                        };
                        for (index, ch) in chars.iter().enumerate() {
                            let display = {
                                let codepoint = *ch as u32;
                                if (0x2460..=0x2473).contains(&codepoint) {
                                    (codepoint - 0x2460 + 1).to_string()
                                } else if let Some(display) = pua_to_display_text(*ch) {
                                    display
                                } else {
                                    ch.to_string()
                                }
                            };
                            draw_overlap_box(
                                &display,
                                bbox.x as f32 + index as f32 * char_advance + box_size / 2.0,
                                (bbox.y + bbox.height / 2.0) as f32,
                            );
                        }
                    }
                    if effective_rotation != 0.0 {
                        canvas.restore();
                    }
                    return;
                }

                let char_positions = compute_char_positions(text, style);
                let clusters = split_into_clusters(text);
                let text_width = *char_positions.last().unwrap_or(&0.0) as f32;
                let ratio = if style.ratio > 0.0 {
                    style.ratio as f32
                } else {
                    1.0
                };
                let has_ratio = (ratio - 1.0).abs() > 0.01;
                let shade_rgb = style.shade_color & 0x00FF_FFFF;
                if shade_rgb != 0x00FF_FFFF && shade_rgb != 0 && text_width > 0.0 {
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

                let suppress_dash_leader_line = !matches!(style.underline, UnderlineType::None);
                let dash_run_groups: Vec<(usize, usize)> = {
                    let mut groups = Vec::new();
                    let mut run_start: Option<usize> = None;
                    for (idx, (_, cluster)) in clusters.iter().enumerate() {
                        if cluster == "-" {
                            if run_start.is_none() {
                                run_start = Some(idx);
                            }
                        } else if let Some(start) = run_start.take() {
                            if idx - start >= 3 {
                                groups.push((start, idx));
                            }
                        }
                    }
                    if let Some(start) = run_start {
                        if clusters.len() - start >= 3 {
                            groups.push((start, clusters.len()));
                        }
                    }
                    groups
                };
                let cluster_in_dash_run = |cluster_idx: usize| -> Option<(f32, f32)> {
                    for &(start, end) in &dash_run_groups {
                        if cluster_idx == start {
                            let start_char_idx = clusters[start].0;
                            let last = &clusters[end - 1];
                            let end_char_idx = last.0 + last.1.chars().count();
                            let x1 = char_positions.get(start_char_idx).copied().unwrap_or(0.0);
                            let x2 = char_positions
                                .get(end_char_idx)
                                .copied()
                                .unwrap_or_else(|| *char_positions.last().unwrap_or(&0.0));
                            return Some((x1 as f32, x2 as f32));
                        }
                        if cluster_idx > start && cluster_idx < end {
                            return Some((f32::NAN, f32::NAN));
                        }
                    }
                    None
                };
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
                    for (cluster_idx, (char_idx, cluster)) in clusters.iter().enumerate() {
                        if cluster == " " || cluster == "\t" || cluster == "\u{2007}" {
                            continue;
                        }
                        if cluster.starts_with(|ch: char| {
                            ch < '\u{0020}' && !matches!(ch, '\t' | '\n' | '\r')
                        }) {
                            continue;
                        }
                        if let Some((x1_rel, x2_rel)) = cluster_in_dash_run(cluster_idx) {
                            if x1_rel.is_finite() && !suppress_dash_leader_line {
                                draw_styled_line(
                                    bbox.x as f32 + x1_rel + dx,
                                    y as f32 - font_size * 0.32 + dy,
                                    bbox.x as f32 + x2_rel + dx,
                                    color,
                                    (font_size * 0.07).max(0.5),
                                    &[],
                                    false,
                                );
                            }
                            continue;
                        }
                        if is_middle_dot(cluster) {
                            let advance = cluster_advance(*char_idx, cluster);
                            let cx = bbox.x as f32
                                + char_positions.get(*char_idx).copied().unwrap_or(0.0) as f32
                                + advance / 2.0
                                + dx;
                            let cy = y as f32 - font_size * 0.35 + dy;
                            let mut dot_paint = Paint::default();
                            dot_paint.set_anti_alias(true);
                            dot_paint.set_style(paint::Style::Fill);
                            dot_paint.set_color(color);
                            canvas.draw_circle((cx, cy), font_size * 0.08, &dot_paint);
                            continue;
                        }
                        if let Some(font) = font_for_text(cluster, font_size) {
                            let char_x = bbox.x as f32
                                + char_positions.get(*char_idx).copied().unwrap_or(0.0) as f32
                                + dx;
                            let char_y = y as f32 + dy;
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

                if !matches!(style.underline, UnderlineType::None) && text_width > 0.0 {
                    let color = if style.underline_color != 0 {
                        colorref_to_skia(style.underline_color, 1.0)
                    } else {
                        colorref_to_skia(style.color, 1.0)
                    };
                    let line_y = match style.underline {
                        UnderlineType::Top => y as f32 - font_size + 1.0,
                        _ => y as f32 + 2.0,
                    };
                    draw_line_shape(
                        bbox.x as f32,
                        line_y,
                        bbox.x as f32 + text_width,
                        color,
                        style.underline_shape,
                    );
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
                    let x2 = bbox.x as f32 + leader.end_x as f32;
                    let line_y = y as f32 - font_size * 0.35;
                    let color = colorref_to_skia(style.color, 1.0);
                    match leader.fill_type {
                        1 => draw_styled_line(x1, line_y, x2, color, 0.5, &[], false),
                        2 => draw_styled_line(x1, line_y, x2, color, 0.5, &[3.0, 3.0], false),
                        3 => draw_styled_line(x1, line_y, x2, color, 1.0, &[0.1, 3.0], true),
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
                let mut font = self
                    .font_mgr
                    .match_family_style("DejaVu Sans", FontStyle::normal())
                    .or_else(|| {
                        self.font_mgr
                            .legacy_make_typeface(None::<&str>, FontStyle::normal())
                    })
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
            mark_paint.set_color(Color::from_argb(255, 74, 144, 217));
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
                let char_positions = compute_char_positions(text, style);
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
        let open_shape_transform =
            |transform: crate::renderer::render_tree::ShapeTransform,
             bbox: &crate::renderer::render_tree::BoundingBox| {
                canvas.save();
                let cx = (bbox.x + bbox.width / 2.0) as f32;
                let cy = (bbox.y + bbox.height / 2.0) as f32;
                if transform.horz_flip {
                    canvas.translate((cx * 2.0, 0.0));
                    canvas.scale((-1.0, 1.0));
                }
                if transform.vert_flip {
                    canvas.translate((0.0, cy * 2.0));
                    canvas.scale((1.0, -1.0));
                }
                if transform.rotation != 0.0 {
                    canvas.rotate(transform.rotation as f32, Some((cx, cy).into()));
                }
            };

        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.render_node(canvas, child, output_options);
                }
            }
            LayerNodeKind::ClipRect { clip, child, .. } => {
                if !clip_enabled {
                    self.render_node(canvas, child, output_options);
                    return;
                }
                canvas.save();
                canvas.clip_rect(
                    Rect::from_xywh(
                        clip.x as f32,
                        clip.y as f32,
                        clip.width as f32,
                        clip.height as f32,
                    ),
                    None,
                    Some(true),
                );
                self.render_node(canvas, child, output_options);
                canvas.restore();
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    match op {
                        PaintOp::PageBackground { bbox, background } => {
                            let rect = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(color) = background
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .or(background.background_color)
                            {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_style(paint::Style::Fill);
                                paint.set_color(colorref_to_skia(color, 1.0));
                                canvas.draw_rect(rect, &paint);
                            }
                            if let Some(image) = &background.image {
                                draw_image(
                                    &image.data,
                                    *bbox,
                                    Some(image.fill_mode),
                                    None,
                                    None,
                                    ImageEffect::RealPic,
                                );
                            }
                            if let Some(color) = background.border_color {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_style(paint::Style::Stroke);
                                paint.set_stroke_width(if background.border_width > 0.0 {
                                    background.border_width as f32
                                } else {
                                    1.0
                                });
                                paint.set_color(colorref_to_skia(color, 1.0));
                                canvas.draw_rect(rect, &paint);
                            }
                        }
                        PaintOp::TextRun { bbox, run } => {
                            draw_text(
                                &run.text,
                                *bbox,
                                &run.style,
                                run.baseline,
                                run.rotation,
                                run.is_vertical,
                                run.char_overlap.as_ref(),
                            );
                            let is_marker = !matches!(
                                run.field_marker,
                                crate::renderer::render_tree::FieldMarkerType::None
                            );
                            draw_text_marks(
                                &run.text,
                                *bbox,
                                &run.style,
                                run.baseline,
                                run.rotation,
                                run.is_vertical,
                                is_marker,
                                run.is_para_end,
                                run.is_line_break_end,
                            );
                        }
                        PaintOp::FootnoteMarker { bbox, marker } => {
                            let style = crate::renderer::TextStyle {
                                font_family: marker.font_family.clone(),
                                font_size: (marker.base_font_size * 0.55).max(7.0),
                                color: marker.color,
                                ..Default::default()
                            };
                            draw_text(
                                &marker.text,
                                *bbox,
                                &style,
                                bbox.height * 0.4,
                                0.0,
                                false,
                                None,
                            );
                        }
                        PaintOp::Line { bbox, line } => {
                            if line.transform.has_transform() {
                                open_shape_transform(line.transform, bbox);
                            }
                            canvas.draw_line(
                                (line.x1 as f32, line.y1 as f32),
                                (line.x2 as f32, line.y2 as f32),
                                &make_line_paint(&line.style),
                            );
                            if line.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Rectangle { bbox, rect } => {
                            if rect.transform.has_transform() {
                                open_shape_transform(rect.transform, bbox);
                            }
                            let sk_rect = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(fill) = rect
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        rect.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&rect.style))
                            {
                                if rect.corner_radius > 0.0 {
                                    canvas.draw_round_rect(
                                        sk_rect,
                                        rect.corner_radius as f32,
                                        rect.corner_radius as f32,
                                        &fill,
                                    );
                                } else {
                                    canvas.draw_rect(sk_rect, &fill);
                                }
                            }
                            if let Some(stroke) = make_stroke_paint(&rect.style) {
                                if rect.corner_radius > 0.0 {
                                    canvas.draw_round_rect(
                                        sk_rect,
                                        rect.corner_radius as f32,
                                        rect.corner_radius as f32,
                                        &stroke,
                                    );
                                } else {
                                    canvas.draw_rect(sk_rect, &stroke);
                                }
                            }
                            if rect.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Ellipse { bbox, ellipse } => {
                            if ellipse.transform.has_transform() {
                                open_shape_transform(ellipse.transform, bbox);
                            }
                            let oval = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(fill) = ellipse
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        ellipse.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&ellipse.style))
                            {
                                canvas.draw_oval(oval, &fill);
                            }
                            if let Some(stroke) = make_stroke_paint(&ellipse.style) {
                                canvas.draw_oval(oval, &stroke);
                            }
                            if ellipse.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Path { bbox, path } => {
                            if path.transform.has_transform() {
                                open_shape_transform(path.transform, bbox);
                            }
                            let mut builder = PathBuilder::new();
                            let mut current = (0.0, 0.0);
                            for command in &path.commands {
                                match *command {
                                    PathCommand::MoveTo(x, y) => {
                                        builder.move_to((x as f32, y as f32));
                                        current = (x, y);
                                    }
                                    PathCommand::LineTo(x, y) => {
                                        builder.line_to((x as f32, y as f32));
                                        current = (x, y);
                                    }
                                    PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                                        builder.cubic_to(
                                            (x1 as f32, y1 as f32),
                                            (x2 as f32, y2 as f32),
                                            (x as f32, y as f32),
                                        );
                                        current = (x, y);
                                    }
                                    PathCommand::ArcTo(
                                        rx,
                                        ry,
                                        rotation,
                                        large_arc,
                                        sweep,
                                        x,
                                        y,
                                    ) => {
                                        for segment in svg_arc_to_beziers(
                                            current.0, current.1, rx, ry, rotation, large_arc,
                                            sweep, x, y,
                                        ) {
                                            if let PathCommand::CurveTo(x1, y1, x2, y2, ex, ey) =
                                                segment
                                            {
                                                builder.cubic_to(
                                                    (x1 as f32, y1 as f32),
                                                    (x2 as f32, y2 as f32),
                                                    (ex as f32, ey as f32),
                                                );
                                                current = (ex, ey);
                                            }
                                        }
                                    }
                                    PathCommand::ClosePath => {
                                        builder.close();
                                    }
                                }
                            }
                            let sk_path = builder.detach();
                            if let Some(fill) = path
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        path.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&path.style))
                            {
                                canvas.draw_path(&sk_path, &fill);
                            }
                            if let Some(stroke) = make_stroke_paint(&path.style) {
                                canvas.draw_path(&sk_path, &stroke);
                            }
                            if path.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Image { bbox, image } => {
                            if image.transform.has_transform() {
                                open_shape_transform(image.transform, bbox);
                            }
                            if let Some(data) = image.data.as_deref() {
                                draw_image(
                                    data,
                                    *bbox,
                                    image.fill_mode,
                                    image.original_size,
                                    image.crop,
                                    image.effect,
                                );
                            } else {
                                draw_placeholder(*bbox, "image");
                            }
                            if image.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Equation { bbox, equation } => {
                            canvas.save();
                            let scale_x = if equation.layout_box.width > 0.0 && bbox.width > 0.0 {
                                bbox.width / equation.layout_box.width
                            } else {
                                1.0
                            };
                            if (scale_x - 1.0).abs() > 0.01 {
                                canvas.translate((bbox.x as f32, bbox.y as f32));
                                canvas.scale((scale_x as f32, 1.0));
                                render_equation(
                                    canvas,
                                    &self.font_mgr,
                                    &equation.layout_box,
                                    0.0,
                                    0.0,
                                    equation.color,
                                    equation.font_size,
                                );
                            } else {
                                render_equation(
                                    canvas,
                                    &self.font_mgr,
                                    &equation.layout_box,
                                    bbox.x,
                                    bbox.y,
                                    equation.color,
                                    equation.font_size,
                                );
                            }
                            canvas.restore();
                        }
                        PaintOp::FormObject { bbox, form } => {
                            self.draw_form_control(canvas, *bbox, form);
                        }
                        PaintOp::Placeholder { bbox, placeholder } => {
                            draw_placeholder(*bbox, placeholder.label.as_str());
                        }
                        PaintOp::RawSvg { bbox, raw } => {
                            if !draw_svg_fragment(
                                canvas,
                                raw.svg.as_str(),
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                                ImageSampling::linear(),
                            ) {
                                draw_placeholder(*bbox, "svg");
                            }
                        }
                    }
                }
            }
        }
    }
}

impl LayerRasterRenderer for SkiaLayerRenderer {
    fn render_raster(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        self.render_raster_with_options(tree, options)
    }
}

impl SkiaLayerRenderer {
    fn make_form_font(&self, size: f32) -> Font {
        let style = FontStyle::default();
        let cjk_families = ["Malgun Gothic", "맑은 고딕", "NanumGothic", "나눔고딕", "AppleGothic"];
        for family in &cjk_families {
            if let Some(tf) = self.custom_typefaces.get(*family).cloned() {
                return Font::new(tf, size);
            }
            if let Some(tf) = self.font_mgr.match_family_style(family, style) {
                return Font::new(tf, size);
            }
        }
        if let Some(tf) = self.font_mgr.legacy_make_typeface(None::<&str>, style) {
            return Font::new(tf, size);
        }
        let mut f = Font::default();
        f.set_size(size);
        f
    }

    fn draw_form_control(
        &self,
        canvas: &Canvas,
        bbox: crate::renderer::render_tree::BoundingBox,
        form: &crate::renderer::render_tree::FormObjectNode,
    ) {
        use crate::model::control::FormType;

        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return;
        }

        let x = bbox.x as f32;
        let y = bbox.y as f32;
        let w = bbox.width as f32;
        let h = bbox.height as f32;
        let rect = Rect::from_xywh(x, y, w, h);

        let bg_color = parse_css_color(&form.back_color).unwrap_or(Color::from_rgb(240, 240, 240));
        let fg_color = parse_css_color(&form.fore_color).unwrap_or(Color::from_rgb(0, 0, 0));
        let border_color = Color::from_rgb(160, 160, 160);

    match form.form_type {
        FormType::PushButton => {
            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(bg_color);
            let rrect = RRect::new_rect_xy(rect, 3.0, 3.0);
            canvas.draw_rrect(rrect, &fill);

            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(border_color);
            canvas.draw_rrect(rrect, &stroke);

            let label = if form.caption.is_empty() { &form.name } else { &form.caption };
            if !label.is_empty() {
                let font = self.make_form_font((h * 0.45).clamp(8.0, 14.0));
                let mut tp = Paint::default();
                tp.set_anti_alias(true);
                tp.set_color(fg_color);
                let text_w = font.measure_str(label, Some(&tp)).0;
                let tx = x + (w - text_w) / 2.0;
                let ty = y + h / 2.0 + font.size() * 0.35;
                canvas.draw_str(label, (tx, ty), &font, &tp);
            }
        }
        FormType::CheckBox => {
            let box_size = h.min(w).min(14.0);
            let bx = x + 2.0;
            let by = y + (h - box_size) / 2.0;
            let box_rect = Rect::from_xywh(bx, by, box_size, box_size);

            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(bg_color);
            canvas.draw_rect(box_rect, &fill);

            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(border_color);
            canvas.draw_rect(box_rect, &stroke);

            if form.value != 0 {
                let mut check = Paint::default();
                check.set_anti_alias(true);
                check.set_style(paint::Style::Stroke);
                check.set_stroke_width(2.0);
                check.set_color(fg_color);
                check.set_stroke_cap(paint::Cap::Round);
                let cx = bx + box_size * 0.2;
                let cy = by + box_size * 0.55;
                let mx = bx + box_size * 0.4;
                let my = by + box_size * 0.75;
                let ex = bx + box_size * 0.8;
                let ey = by + box_size * 0.25;
                let mut builder = PathBuilder::new();
                builder.move_to((cx, cy));
                builder.line_to((mx, my));
                builder.line_to((ex, ey));
                let path = builder.detach();
                canvas.draw_path(&path, &check);
            }

            if !form.caption.is_empty() {
                let font = self.make_form_font((h * 0.6).clamp(8.0, 13.0));
                let mut tp = Paint::default();
                tp.set_anti_alias(true);
                tp.set_color(fg_color);
                let tx = bx + box_size + 4.0;
                let ty = y + h / 2.0 + font.size() * 0.35;
                canvas.draw_str(&form.caption, (tx, ty), &font, &tp);
            }
        }
        FormType::RadioButton => {
            let r = h.min(w).min(14.0) / 2.0;
            let cx = x + 2.0 + r;
            let cy = y + h / 2.0;

            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(bg_color);
            canvas.draw_circle((cx, cy), r, &fill);

            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(border_color);
            canvas.draw_circle((cx, cy), r, &stroke);

            if form.value != 0 {
                let mut dot = Paint::default();
                dot.set_anti_alias(true);
                dot.set_style(paint::Style::Fill);
                dot.set_color(fg_color);
                canvas.draw_circle((cx, cy), r * 0.5, &dot);
            }

            if !form.caption.is_empty() {
                let font = self.make_form_font((h * 0.6).clamp(8.0, 13.0));
                let mut tp = Paint::default();
                tp.set_anti_alias(true);
                tp.set_color(fg_color);
                let tx = cx + r + 4.0;
                let ty = y + h / 2.0 + font.size() * 0.35;
                canvas.draw_str(&form.caption, (tx, ty), &font, &tp);
            }
        }
        FormType::ComboBox => {
            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(bg_color);
            canvas.draw_rect(rect, &fill);

            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(border_color);
            canvas.draw_rect(rect, &stroke);

            // 드롭다운 화살표 영역
            let arrow_w = h.min(20.0);
            let ax = x + w - arrow_w;
            let arrow_rect = Rect::from_xywh(ax, y, arrow_w, h);
            let mut abg = Paint::default();
            abg.set_anti_alias(true);
            abg.set_style(paint::Style::Fill);
            abg.set_color(bg_color);
            canvas.draw_rect(arrow_rect, &abg);
            canvas.draw_line((ax, y), (ax, y + h), &stroke);

            // 화살표 삼각형
            let mut arrow = Paint::default();
            arrow.set_anti_alias(true);
            arrow.set_style(paint::Style::Fill);
            arrow.set_color(Color::from_rgb(80, 80, 80));
            let acx = ax + arrow_w / 2.0;
            let acy = y + h / 2.0;
            let as_ = (arrow_w * 0.25).min(5.0);
            let mut builder = PathBuilder::new();
            builder.move_to((acx - as_, acy - as_ * 0.5));
            builder.line_to((acx + as_, acy - as_ * 0.5));
            builder.line_to((acx, acy + as_ * 0.5));
            builder.close();
            let path = builder.detach();
            canvas.draw_path(&path, &arrow);

            if !form.text.is_empty() {
                let font = self.make_form_font((h * 0.55).clamp(8.0, 13.0));
                let mut tp = Paint::default();
                tp.set_anti_alias(true);
                tp.set_color(fg_color);
                let tx = x + 4.0;
                let ty = y + h / 2.0 + font.size() * 0.35;
                canvas.draw_str(&form.text, (tx, ty), &font, &tp);
            }
        }
        FormType::Edit => {
            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(bg_color);
            canvas.draw_rect(rect, &fill);

            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(border_color);
            canvas.draw_rect(rect, &stroke);

            if !form.text.is_empty() {
                let font = self.make_form_font((h * 0.55).clamp(8.0, 13.0));
                let mut tp = Paint::default();
                tp.set_anti_alias(true);
                tp.set_color(fg_color);
                let tx = x + 4.0;
                let ty = y + h / 2.0 + font.size() * 0.35;
                canvas.draw_str(&form.text, (tx, ty), &font, &tp);
            }
        }
    }
    }
}

fn parse_css_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::from_rgb(r, g, b))
}

fn colorref_to_skia(color: ColorRef, alpha_scale: f32) -> Color {
    let b = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let a = (255.0 * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_argb(a, r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::FormType;
    use crate::model::style::{ImageFillMode, UnderlineType};
    use crate::paint::{CacheHint, GroupKind, LayerNode, LayerOutputOptions};
    use crate::renderer::composer::CharOverlapInfo;
    use crate::renderer::equation::ast::EqNode;
    use crate::renderer::equation::layout::EqLayout;
    use crate::renderer::render_tree::{
        BoundingBox, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
        PageBackgroundImage, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode,
        RectangleNode, TextRunNode,
    };
    use crate::renderer::{GradientFillInfo, PatternFillInfo, TabLeaderInfo, TextStyle};
    use image::{ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn decode_rgba(bytes: &[u8]) -> image::RgbaImage {
        image::load_from_memory(bytes)
            .expect("decode png")
            .to_rgba8()
    }

    fn assert_channel(pixel: image::Rgba<u8>, channel: usize, min: u8, max: u8) {
        assert!(
            pixel[channel] >= min && pixel[channel] <= max,
            "pixel={pixel:?}, channel={channel}, expected {min}..={max}"
        );
    }

    fn count_ink(image: &image::RgbaImage) -> usize {
        image.pixels().filter(|pixel| pixel[3] > 0).count()
    }

    fn solid_png(color: [u8; 4]) -> Vec<u8> {
        let image = RgbaImage::from_pixel(2, 2, Rgba(color));
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("encode png");
        cursor.into_inner()
    }

    fn split_png(
        width: u32,
        height: u32,
        first: [u8; 4],
        second: [u8; 4],
        vertical: bool,
    ) -> Vec<u8> {
        let mut image = RgbaImage::from_pixel(width, height, Rgba(first));
        for y in 0..height {
            for x in 0..width {
                let second_half = if vertical {
                    y >= height / 2
                } else {
                    x >= width / 2
                };
                if second_half {
                    image.put_pixel(x, y, Rgba(second));
                }
            }
        }
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("encode png");
        cursor.into_inner()
    }

    fn solid_rect_tree(
        page_width: f64,
        page_height: f64,
        bbox: BoundingBox,
        fill_color: ColorRef,
    ) -> PageLayerTree {
        let style = ShapeStyle {
            fill_color: Some(fill_color),
            ..Default::default()
        };
        PageLayerTree::new(
            page_width,
            page_height,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, page_width, page_height),
                None,
                vec![PaintOp::Rectangle {
                    bbox,
                    rect: RectangleNode::new(0.0, style, None),
                }],
            ),
        )
    }

    #[test]
    fn renders_png_for_basic_layer_tree() {
        let tree = solid_rect_tree(
            32.0,
            24.0,
            BoundingBox::new(4.0, 4.0, 16.0, 12.0),
            0x000000ff,
        );

        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render png");

        assert_eq!(output.format, RasterOutputFormat::Png);
        assert_eq!(output.width, 32);
        assert_eq!(output.height, 24);
        assert_eq!(&output.bytes[..8], b"\x89PNG\r\n\x1a\n");
        let decoded = image::load_from_memory(&output.bytes).expect("decode png");
        assert_eq!(decoded.width(), 32);
        assert_eq!(decoded.height(), 24);
    }

    #[test]
    fn raster_options_scale_output_size() {
        let tree = PageLayerTree::new(
            10.0,
            12.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 12.0), None, vec![]),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    scale: 2.0,
                    transparent: false,
                    ..Default::default()
                },
            )
            .expect("render scaled png");

        assert_eq!(output.width, 20);
        assert_eq!(output.height, 24);
    }

    #[test]
    fn preserves_colorref_channel_order_in_pixels() {
        let tree = solid_rect_tree(12.0, 12.0, BoundingBox::new(2.0, 2.0, 8.0, 8.0), 0x000000ff);
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render red rect");
        let image = decode_rgba(&output.bytes);
        let pixel = image.get_pixel(4, 4);

        assert!(pixel[0] > 220, "red channel should be high: {pixel:?}");
        assert!(pixel[1] < 32, "green channel should be low: {pixel:?}");
        assert!(pixel[2] < 32, "blue channel should be low: {pixel:?}");
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn clears_transparent_by_default_and_opaque_when_requested() {
        let tree = PageLayerTree::new(
            4.0,
            4.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 4.0, 4.0), None, vec![]),
        );
        let renderer = SkiaLayerRenderer::new();
        let transparent = renderer
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render transparent");
        let opaque = renderer
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    transparent: false,
                    ..Default::default()
                },
            )
            .expect("render opaque");

        assert_eq!(decode_rgba(&transparent.bytes).get_pixel(0, 0)[3], 0);
        assert_eq!(
            decode_rgba(&opaque.bytes).get_pixel(0, 0).0,
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn output_options_control_clip_rect_replay() {
        let style = ShapeStyle {
            fill_color: Some(0x000000ff),
            ..Default::default()
        };
        let child = LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            None,
            vec![PaintOp::Rectangle {
                bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                rect: RectangleNode::new(0.0, style, None),
            }],
        );
        let clipped = PageLayerTree::new(
            20.0,
            20.0,
            LayerNode::clip_rect(
                BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                None,
                BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                child.clone(),
                crate::paint::ClipKind::Generic,
            ),
        );
        let unclipped = clipped.clone().with_output_options(LayerOutputOptions {
            clip_enabled: false,
            ..Default::default()
        });
        let renderer = SkiaLayerRenderer::new();
        let clipped_png = renderer
            .render_raster_with_options(&clipped, RasterRenderOptions::default())
            .expect("render clipped");
        let unclipped_png = renderer
            .render_raster_with_options(&unclipped, RasterRenderOptions::default())
            .expect("render unclipped");
        let clipped = decode_rgba(&clipped_png.bytes);
        let unclipped = decode_rgba(&unclipped_png.bytes);

        assert_eq!(clipped.get_pixel(15, 15)[3], 0);
        assert_eq!(unclipped.get_pixel(15, 15)[3], 255);
    }

    #[test]
    fn rejects_invalid_raster_options_before_surface_creation() {
        let tree = PageLayerTree::new(
            10.0,
            10.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
        );
        let renderer = SkiaLayerRenderer::new();

        let invalid_scale = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                scale: 0.0,
                ..Default::default()
            },
        );
        assert!(invalid_scale.is_err());

        let invalid_dpi = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                dpi: Some(0.0),
                ..Default::default()
            },
        );
        assert!(invalid_dpi.is_err());

        let oversized = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_dimension: 8,
                ..Default::default()
            },
        );
        assert!(oversized.is_err());

        let too_many_pixels = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_pixels: 99,
                ..Default::default()
            },
        );
        assert!(too_many_pixels.is_err());

        let invalid_pixel_budget = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_pixels: 0,
                ..Default::default()
            },
        );
        assert!(invalid_pixel_budget.is_err());
    }

    #[test]
    fn raster_output_preserves_metadata_and_background_color() {
        let tree = PageLayerTree::new(
            3.0,
            2.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 3.0, 2.0), None, vec![]),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    dpi: Some(144.0),
                    background_color: Some(0x0000ff00),
                    ..Default::default()
                },
            )
            .expect("render with metadata");
        let image = decode_rgba(&output.bytes);
        let pixel = *image.get_pixel(0, 0);

        assert_eq!(output.dpi, Some(144.0));
        assert_eq!(
            output.color_space,
            crate::renderer::layer_renderer::RasterColorSpace::Srgb
        );
        assert_channel(pixel, 0, 0, 16);
        assert_channel(pixel, 1, 220, 255);
        assert_channel(pixel, 2, 0, 16);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn rejects_invalid_page_dimensions() {
        let renderer = SkiaLayerRenderer::new();
        let zero_width = PageLayerTree::new(
            0.0,
            10.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 0.0, 10.0), None, vec![]),
        );
        let nan_height = PageLayerTree::new(
            10.0,
            f64::NAN,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
        );

        assert!(renderer
            .render_raster_with_options(&zero_width, RasterRenderOptions::default())
            .is_err());
        assert!(renderer
            .render_raster_with_options(&nan_height, RasterRenderOptions::default())
            .is_err());
    }

    #[test]
    fn renders_page_background_fill_border_and_image() {
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::PageBackground {
                    bbox: BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    background: PageBackgroundNode {
                        background_color: Some(0x0000ff00),
                        border_color: Some(0x00ff0000),
                        border_width: 2.0,
                        gradient: None,
                        image: None,
                    },
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render background");
        let image = decode_rgba(&output.bytes);
        let fill = *image.get_pixel(4, 4);
        let border = *image.get_pixel(0, 0);

        assert_channel(fill, 0, 0, 32);
        assert_channel(fill, 1, 180, 255);
        assert_channel(fill, 2, 0, 32);
        assert_eq!(fill[3], 255);
        assert_channel(border, 0, 0, 64);
        assert_channel(border, 1, 0, 64);
        assert_channel(border, 2, 180, 255);
        assert_eq!(border[3], 255);

        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::PageBackground {
                    bbox: BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    background: PageBackgroundNode {
                        background_color: None,
                        border_color: None,
                        border_width: 0.0,
                        gradient: None,
                        image: Some(PageBackgroundImage {
                            data: solid_png([0, 0, 255, 255]),
                            fill_mode: ImageFillMode::FitToSize,
                        }),
                    },
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render background image");
        let image = decode_rgba(&output.bytes);
        let pixel = *image.get_pixel(4, 4);

        assert_channel(pixel, 0, 0, 32);
        assert_channel(pixel, 1, 0, 32);
        assert_channel(pixel, 2, 220, 255);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn renders_shape_fallback_fills_for_gradient_pattern_ellipse_path_and_line() {
        let gradient = GradientFillInfo {
            gradient_type: 1,
            angle: 0,
            center_x: 0,
            center_y: 0,
            colors: vec![0x00ff0000, 0x000000ff],
            positions: vec![0.0, 1.0],
        };
        let gradient_rect = RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(0x000000ff),
                ..Default::default()
            },
            Some(Box::new(gradient)),
        );
        let pattern_rect = RectangleNode::new(
            0.0,
            ShapeStyle {
                pattern: Some(PatternFillInfo {
                    pattern_type: 1,
                    pattern_color: 0x000000ff,
                    background_color: 0x0000ff00,
                }),
                ..Default::default()
            },
            None,
        );
        let ellipse = crate::renderer::render_tree::EllipseNode::new(
            ShapeStyle {
                fill_color: Some(0x000000ff),
                ..Default::default()
            },
            None,
        );
        let path = PathNode::new(
            vec![
                PathCommand::MoveTo(2.0, 24.0),
                PathCommand::LineTo(12.0, 24.0),
                PathCommand::LineTo(12.0, 34.0),
                PathCommand::LineTo(2.0, 34.0),
                PathCommand::ClosePath,
            ],
            ShapeStyle {
                fill_color: Some(0x00ff0000),
                ..Default::default()
            },
            None,
        );
        let line = crate::renderer::render_tree::LineNode::new(
            18.0,
            30.0,
            34.0,
            30.0,
            LineStyle {
                color: 0x000000ff,
                width: 3.0,
                ..Default::default()
            },
        );
        let tree = PageLayerTree::new(
            40.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 40.0, 40.0),
                None,
                vec![
                    PaintOp::Rectangle {
                        bbox: BoundingBox::new(2.0, 2.0, 10.0, 8.0),
                        rect: gradient_rect,
                    },
                    PaintOp::Rectangle {
                        bbox: BoundingBox::new(16.0, 2.0, 10.0, 8.0),
                        rect: pattern_rect,
                    },
                    PaintOp::Ellipse {
                        bbox: BoundingBox::new(2.0, 12.0, 10.0, 10.0),
                        ellipse,
                    },
                    PaintOp::Path {
                        bbox: BoundingBox::new(2.0, 24.0, 10.0, 10.0),
                        path,
                    },
                    PaintOp::Line {
                        bbox: BoundingBox::new(18.0, 28.0, 16.0, 4.0),
                        line,
                    },
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render shapes");
        let image = decode_rgba(&output.bytes);
        let gradient_pixel = *image.get_pixel(4, 4);
        let pattern_pixel = *image.get_pixel(18, 4);
        let ellipse_pixel = *image.get_pixel(7, 17);
        let path_pixel = *image.get_pixel(7, 29);
        let line_pixel = *image.get_pixel(24, 30);

        assert_channel(gradient_pixel, 2, 180, 255);
        assert_channel(pattern_pixel, 1, 180, 255);
        assert_channel(ellipse_pixel, 0, 180, 255);
        assert_channel(path_pixel, 2, 180, 255);
        assert_channel(line_pixel, 0, 180, 255);
    }

    #[test]
    fn renders_arc_path_segments_as_ink() {
        let path = PathNode::new(
            vec![
                PathCommand::MoveTo(4.0, 12.0),
                PathCommand::ArcTo(8.0, 8.0, 0.0, false, true, 20.0, 12.0),
            ],
            ShapeStyle {
                stroke_color: Some(0x000000ff),
                stroke_width: 2.0,
                ..Default::default()
            },
            None,
        );
        let tree = PageLayerTree::new(
            24.0,
            18.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 24.0, 18.0),
                None,
                vec![PaintOp::Path {
                    bbox: BoundingBox::new(4.0, 4.0, 16.0, 12.0),
                    path,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render arc path");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 8);
    }

    #[test]
    fn renders_valid_images_and_invalid_image_placeholders() {
        let tree = PageLayerTree::new(
            20.0,
            10.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 20.0, 10.0),
                None,
                vec![
                    PaintOp::Image {
                        bbox: BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                        image: ImageNode::new(1, Some(solid_png([0, 0, 255, 255]))),
                    },
                    PaintOp::Image {
                        bbox: BoundingBox::new(10.0, 0.0, 8.0, 8.0),
                        image: ImageNode::new(2, Some(vec![1, 2, 3, 4])),
                    },
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render images");
        let image = decode_rgba(&output.bytes);
        let valid = *image.get_pixel(4, 4);
        let invalid_placeholder = *image.get_pixel(12, 4);

        assert_channel(valid, 2, 220, 255);
        assert!(invalid_placeholder[3] > 0);
    }

    #[test]
    fn renders_cropped_image_source_rects() {
        let mut node = ImageNode::new(
            1,
            Some(split_png(4, 4, [255, 0, 0, 255], [0, 0, 255, 255], true)),
        );
        node.crop = Some((0, 2, 4, 4));
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::Image {
                    bbox: BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    image: node,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render cropped image");
        let image = decode_rgba(&output.bytes);
        let pixel = *image.get_pixel(4, 4);

        assert_channel(pixel, 0, 0, 48);
        assert_channel(pixel, 2, 180, 255);
    }

    #[test]
    fn renders_tiled_images_using_original_size() {
        let mut node = ImageNode::new(
            1,
            Some(split_png(8, 4, [255, 0, 0, 255], [0, 255, 0, 255], false)),
        );
        node.fill_mode = Some(ImageFillMode::TileAll);
        node.original_size = Some((8.0, 4.0));
        let tree = PageLayerTree::new(
            16.0,
            4.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 16.0, 4.0),
                None,
                vec![PaintOp::Image {
                    bbox: BoundingBox::new(0.0, 0.0, 16.0, 4.0),
                    image: node,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render tiled image");
        let image = decode_rgba(&output.bytes);
        let first_tile_left = *image.get_pixel(2, 2);
        let second_tile_left = *image.get_pixel(10, 2);
        let first_tile_right = *image.get_pixel(6, 2);
        let second_tile_right = *image.get_pixel(14, 2);

        assert_channel(first_tile_left, 0, 180, 255);
        assert_channel(second_tile_left, 0, 180, 255);
        assert_channel(first_tile_right, 1, 180, 255);
        assert_channel(second_tile_right, 1, 180, 255);
    }

    #[test]
    fn applies_grayscale_image_effect() {
        let mut node = ImageNode::new(1, Some(solid_png([255, 0, 0, 255])));
        node.effect = ImageEffect::GrayScale;
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::Image {
                    bbox: BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    image: node,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render grayscale image");
        let pixel = *decode_rgba(&output.bytes).get_pixel(4, 4);
        let max_channel = pixel[0].max(pixel[1]).max(pixel[2]);
        let min_channel = pixel[0].min(pixel[1]).min(pixel[2]);

        assert!(max_channel.abs_diff(min_channel) <= 2, "pixel={pixel:?}");
        assert!(pixel[0] > 40 && pixel[0] < 140, "pixel={pixel:?}");
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn ignores_invalid_image_rects() {
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::Image {
                    bbox: BoundingBox::new(f64::NAN, 0.0, 8.0, 8.0),
                    image: ImageNode::new(1, Some(solid_png([255, 0, 0, 255]))),
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render invalid image rect");
        let image = decode_rgba(&output.bytes);

        assert_eq!(count_ink(&image), 0);
    }

    #[test]
    fn renders_text_and_footnote_marker_as_ink() {
        let run = TextRunNode {
            text: "A".to_string(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 20.0,
            field_marker: Default::default(),
        };
        let marker = FootnoteMarkerNode {
            number: 1,
            text: "1)".to_string(),
            base_font_size: 18.0,
            font_family: String::new(),
            color: 0x00000000,
            section_index: 0,
            para_index: 0,
            control_index: 0,
        };
        let tree = PageLayerTree::new(
            64.0,
            32.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 32.0),
                None,
                vec![
                    PaintOp::TextRun {
                        bbox: BoundingBox::new(4.0, 4.0, 24.0, 24.0),
                        run,
                    },
                    PaintOp::FootnoteMarker {
                        bbox: BoundingBox::new(32.0, 4.0, 24.0, 24.0),
                        marker,
                    },
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render text");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_char_overlap_text_run_as_ink() {
        let run = TextRunNode {
            text: "①".to_string(),
            style: TextStyle {
                font_size: 20.0,
                color: 0x00000000,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: Some(CharOverlapInfo {
                border_type: 1,
                inner_char_size: 90,
            }),
            border_fill_id: 0,
            baseline: 22.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            40.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 40.0, 40.0),
                None,
                vec![PaintOp::TextRun {
                    bbox: BoundingBox::new(8.0, 8.0, 24.0, 24.0),
                    run,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render char overlap");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_tab_leader_for_empty_text_run() {
        let run = TextRunNode {
            text: String::new(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                tab_leaders: vec![TabLeaderInfo {
                    start_x: 8.0,
                    end_x: 72.0,
                    fill_type: 1,
                }],
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 22.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            88.0,
            36.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 88.0, 36.0),
                None,
                vec![PaintOp::TextRun {
                    bbox: BoundingBox::new(4.0, 4.0, 80.0, 28.0),
                    run,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render tab leader");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_output_control_marks_as_ink() {
        let run = TextRunNode {
            text: " \t".to_string(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: true,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 22.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            72.0,
            36.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 72.0, 36.0),
                None,
                vec![PaintOp::TextRun {
                    bbox: BoundingBox::new(4.0, 4.0, 60.0, 28.0),
                    run,
                }],
            ),
        )
        .with_output_options(LayerOutputOptions {
            show_control_codes: true,
            ..Default::default()
        });
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render control marks");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_decorated_text_as_ink() {
        let run = TextRunNode {
            text: "A".to_string(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                underline: UnderlineType::Bottom,
                strikethrough: true,
                emphasis_dot: 1,
                shade_color: 0x0000ffff,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 24.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            48.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 48.0, 40.0),
                None,
                vec![PaintOp::TextRun {
                    bbox: BoundingBox::new(8.0, 8.0, 32.0, 28.0),
                    run,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render decorated text");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_equation_layout_as_colored_ink() {
        let font_size = 18.0;
        let layout_box = EqLayout::new(font_size).layout(&EqNode::Fraction {
            numer: Box::new(EqNode::Text("a".to_string())),
            denom: Box::new(EqNode::Text("b".to_string())),
        });
        let equation = EquationNode {
            svg_content: String::new(),
            layout_box,
            color_str: "#ff0000".to_string(),
            color: 0x000000ff,
            font_size,
            section_index: Some(0),
            para_index: Some(0),
            control_index: Some(0),
            cell_index: None,
            cell_para_index: None,
        };
        let tree = PageLayerTree::new(
            64.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 48.0),
                None,
                vec![PaintOp::Equation {
                    bbox: BoundingBox::new(6.0, 6.0, 44.0, 32.0),
                    equation,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render equation");
        let image = decode_rgba(&output.bytes);
        let red_ink = image
            .pixels()
            .filter(|pixel| pixel[0] > 160 && pixel[1] < 96 && pixel[2] < 96 && pixel[3] > 0)
            .count();

        assert!(
            red_ink > 0,
            "equation should render using its configured color"
        );
    }

    #[test]
    fn renders_atop_equation_layout_as_colored_ink() {
        let font_size = 18.0;
        let layout_box = EqLayout::new(font_size).layout(&EqNode::Atop {
            top: Box::new(EqNode::Text("a".to_string())),
            bottom: Box::new(EqNode::Text("b".to_string())),
        });
        let equation = EquationNode {
            svg_content: String::new(),
            layout_box,
            color_str: "#00aa00".to_string(),
            color: 0x0000aa00,
            font_size,
            section_index: Some(0),
            para_index: Some(0),
            control_index: Some(0),
            cell_index: None,
            cell_para_index: None,
        };
        let tree = PageLayerTree::new(
            64.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 48.0),
                None,
                vec![PaintOp::Equation {
                    bbox: BoundingBox::new(6.0, 6.0, 44.0, 32.0),
                    equation,
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render atop equation");
        let image = decode_rgba(&output.bytes);
        let green_ink = image
            .pixels()
            .filter(|pixel| pixel[0] < 96 && pixel[1] > 96 && pixel[2] < 96 && pixel[3] > 0)
            .count();

        assert!(
            green_ink > 0,
            "atop equation should render using its configured color"
        );
    }

    #[test]
    fn renders_placeholder_style_ops_as_ink() {
        let form = FormObjectNode {
            form_type: FormType::PushButton,
            caption: "OK".to_string(),
            text: String::new(),
            fore_color: "#000000".to_string(),
            back_color: "#ffffff".to_string(),
            value: 0,
            enabled: true,
            section_index: 0,
            para_index: 0,
            control_index: 0,
            name: "button".to_string(),
            cell_location: None,
        };
        let tree = PageLayerTree::new(
            48.0,
            16.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 48.0, 16.0),
                None,
                vec![
                    PaintOp::Placeholder {
                        bbox: BoundingBox::new(0.0, 0.0, 14.0, 14.0),
                        placeholder: PlaceholderNode {
                            fill_color: 0,
                            stroke_color: 0,
                            label: "ph".to_string(),
                        },
                    },
                    PaintOp::RawSvg {
                        bbox: BoundingBox::new(16.0, 0.0, 14.0, 14.0),
                        raw: RawSvgNode {
                            svg: "<invalid".to_string(),
                        },
                    },
                    PaintOp::FormObject {
                        bbox: BoundingBox::new(32.0, 0.0, 14.0, 14.0),
                        form,
                    },
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render placeholders");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 40);
    }

    #[test]
    fn renders_raw_svg_fragment_as_colored_ink() {
        let tree = PageLayerTree::new(
            32.0,
            24.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 32.0, 24.0),
                None,
                vec![PaintOp::RawSvg {
                    bbox: BoundingBox::new(4.0, 4.0, 18.0, 12.0),
                    raw: RawSvgNode {
                        svg: "<rect x=\"0\" y=\"0\" width=\"18\" height=\"12\" fill=\"#00ff00\"/>"
                            .to_string(),
                    },
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render raw svg");
        let image = decode_rgba(&output.bytes);
        let green_ink = image
            .pixels()
            .filter(|pixel| pixel[0] < 48 && pixel[1] > 180 && pixel[2] < 48 && pixel[3] > 0)
            .count();

        assert!(
            green_ink > 100,
            "raw SVG fragment should render as green ink"
        );
    }

    #[test]
    fn raw_svg_replay_does_not_load_external_file_hrefs() {
        let external_path = std::env::temp_dir().join(format!(
            "rhwp-skia-raw-svg-external-{}.png",
            std::process::id()
        ));
        std::fs::write(&external_path, solid_png([255, 0, 0, 255])).expect("write external png");
        let external_href = external_path.to_string_lossy();
        let tree = PageLayerTree::new(
            32.0,
            24.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 32.0, 24.0),
                None,
                vec![PaintOp::RawSvg {
                    bbox: BoundingBox::new(4.0, 4.0, 20.0, 16.0),
                    raw: RawSvgNode {
                        svg: format!(
                            "<image href=\"{}\" x=\"0\" y=\"0\" width=\"20\" height=\"16\"/>",
                            external_href
                        ),
                    },
                }],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render raw svg with external href");
        let _ = std::fs::remove_file(&external_path);
        let image = decode_rgba(&output.bytes);
        let red_ink = image
            .pixels()
            .filter(|pixel| pixel[0] > 180 && pixel[1] < 48 && pixel[2] < 48 && pixel[3] > 0)
            .count();

        assert_eq!(red_ink, 0, "raw SVG replay must not load file hrefs");
    }

    #[test]
    fn group_children_replay_in_order() {
        let red = LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 12.0, 12.0),
            None,
            vec![PaintOp::Rectangle {
                bbox: BoundingBox::new(0.0, 0.0, 12.0, 12.0),
                rect: RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color: Some(0x000000ff),
                        ..Default::default()
                    },
                    None,
                ),
            }],
        );
        let blue = LayerNode::leaf(
            BoundingBox::new(3.0, 3.0, 6.0, 6.0),
            None,
            vec![PaintOp::Rectangle {
                bbox: BoundingBox::new(3.0, 3.0, 6.0, 6.0),
                rect: RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color: Some(0x00ff0000),
                        ..Default::default()
                    },
                    None,
                ),
            }],
        );
        let tree = PageLayerTree::new(
            12.0,
            12.0,
            LayerNode::group(
                BoundingBox::new(0.0, 0.0, 12.0, 12.0),
                None,
                vec![red, blue],
                CacheHint::None,
                GroupKind::Generic,
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render group");
        let image = decode_rgba(&output.bytes);
        let center = *image.get_pixel(6, 6);

        assert_channel(center, 0, 0, 64);
        assert_channel(center, 1, 0, 64);
        assert_channel(center, 2, 180, 255);
        assert_eq!(center[3], 255);
    }
}
