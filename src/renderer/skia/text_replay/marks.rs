//! 편집 화면의 공백·탭·문단 끝 표시를 본문 출력과 별도로 그린다.
use super::super::font_lookup::{legacy_typeface_for_style, match_system_family_style};
use super::SkiaTextReplay;
use crate::renderer::{render_tree::BoundingBox, TextStyle};
use skia_safe::{font, Color, Font, FontStyle, Paint};

pub(super) fn draw(
    replay: &SkiaTextReplay<'_>,
    text: &str,
    bbox: BoundingBox,
    style: &TextStyle,
    baseline: f64,
    rotation: f64,
    is_marker: bool,
    is_para_end: bool,
    is_line_break_end: bool,
    layout_positions: Option<&[f64]>,
) {
    let canvas = replay.canvas;
    let output_options = replay.output_options;
    if !output_options.show_paragraph_marks && !output_options.show_control_codes {
        return;
    }
    let font_size = style.effective_font_size() as f32;
    let make_mark_font = |size: f32| {
        let mut font = match_system_family_style(
            replay.font_mgr,
            replay.system_families,
            "DejaVu Sans",
            FontStyle::normal(),
        )
        .or_else(|| legacy_typeface_for_style(replay.font_mgr, FontStyle::normal()))
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
    if !text.is_empty() && !is_marker {
        let char_positions =
            crate::renderer::replay_positions_or_compute(text, style, layout_positions);
        let visual = crate::renderer::text_direction::VisualTextPositions::for_style(
            text,
            &char_positions,
            style,
        );
        let origin = |index: usize| {
            visual
                .as_ref()
                .and_then(|p| p.span_origin(index, index + 1))
                .unwrap_or_else(|| char_positions.get(index).copied().unwrap_or(0.0))
        };
        for (index, ch) in text.chars().enumerate() {
            if ch == ' ' {
                let x = bbox.x + origin(index);
                let next_x = if index + 1 < char_positions.len() {
                    x + char_positions[index + 1] - char_positions[index]
                } else {
                    bbox.x + bbox.width
                };
                let mark_x = ((x + next_x) / 2.0) as f32 - font_size * 0.125;
                canvas.draw_str("\u{2228}", (mark_x, y as f32), &font, &mark_paint);
            } else if ch == '\t' {
                let mark_x = bbox.x as f32 + origin(index) as f32;
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
}
