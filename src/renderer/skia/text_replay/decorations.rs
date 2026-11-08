//! 기존 밑줄·취소선·탭 리더의 Native 재생. 글리프 윤곽 보정과 계산을 분리한다.
use skia_safe::{paint, Canvas, Color, Paint, PathEffect};

pub(super) fn draw_styled_line(
    canvas: &Canvas,
    (x1, y): (f32, f32),
    x2: f32,
    color: Color,
    width: f32,
    dash: &[f32],
    round: bool,
) {
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
}

pub(super) fn draw_line_shape(canvas: &Canvas, x1: f32, y: f32, x2: f32, color: Color, shape: u8) {
    let draw_styled_line = |x1, y, x2, color, width, dash: &[f32], round| {
        self::draw_styled_line(canvas, (x1, y), x2, color, width, dash, round)
    };
    match shape {
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
        11 | 12 => {
            if ![x1, y, x2].iter().all(|value| value.is_finite()) || x2 <= x1 {
                return;
            }
            if x2 - x1 > 6.0 * 4096.0 {
                draw_styled_line(x1, y, x2, color, 1.0, &[], false);
                return;
            }
            let offsets: &[f32] = if shape == 12 { &[-1.0, 1.0] } else { &[0.0] };
            for offset in offsets {
                let mut path = skia_safe::PathBuilder::new();
                path.move_to((x1, y + offset));
                let segments = ((x2 - x1) / 6.0).ceil() as usize;
                for index in 0..segments {
                    let left = x1 + index as f32 * 6.0;
                    let right = (left + 6.0).min(x2);
                    let control_y = y + offset + if index % 2 == 0 { -1.5 } else { 1.5 };
                    path.quad_to(((left + right) / 2.0, control_y), (right, y + offset));
                }
                let mut wave = Paint::default();
                wave.set_anti_alias(true);
                wave.set_style(paint::Style::Stroke);
                wave.set_stroke_width(if shape == 12 { 0.5 } else { 0.7 });
                wave.set_color(color);
                canvas.draw_path(&path.detach(), &wave);
            }
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
    }
}
