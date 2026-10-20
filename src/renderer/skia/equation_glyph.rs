//! 측정 단계에서 발행한 수식 변수 윤곽·합성 획·장평을 그대로 소비한다.

use crate::renderer::equation::{layout::FontGlyphPaint, GlyphOutlineCommand as Command};
use skia_safe::{paint, Canvas, Color, Paint, PathBuilder};

pub(super) fn draw_outline(
    canvas: &Canvas,
    commands: &[Command],
    x: f64,
    y: f64,
    color: Color,
    style: Option<FontGlyphPaint>,
) {
    let mut path = PathBuilder::new();
    for command in commands {
        match *command {
            Command::Move((a, b)) => {
                path.move_to((a as f32, b as f32));
            }
            Command::Line((a, b)) => {
                path.line_to((a as f32, b as f32));
            }
            Command::Cubic([(a, b), (c, d), (e, f)]) => {
                path.cubic_to(
                    (a as f32, b as f32),
                    (c as f32, d as f32),
                    (e as f32, f as f32),
                );
            }
            Command::Close => {
                path.close();
            }
        }
    }
    let path = path.detach();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::Fill);
    paint.set_color(color);
    canvas.save();
    canvas.translate((x as f32, y as f32));
    if let Some(style) = style {
        canvas.scale((style.scale_x as f32, 1.0));
    }
    canvas.draw_path(&path, &paint);
    if let Some(style) = style {
        paint.set_style(paint::Style::Stroke);
        // 인쇄 em 획을 화면 최소 굵기로 늘리지 않는다.
        paint.set_stroke_width(style.stroke_width as f32);
        paint.set_stroke_miter(10.0);
        canvas.draw_path(&path, &paint);
    }
    canvas.restore();
}
