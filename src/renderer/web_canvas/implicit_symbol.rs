//! 일반 글자·양각·음각·그림자 모두 같은 원본 기호 윤곽을 소비한다.

use crate::renderer::TextStyle;
use web_sys::CanvasRenderingContext2d;

pub(super) fn paint(
    ctx: &CanvasRenderingContext2d,
    sources: Option<&[char]>,
    index: usize,
    style: &TextStyle,
    x: f64,
    y: f64,
    stroke: bool,
) -> bool {
    let Some((shape, cx, cy, rx, ry)) =
        crate::renderer::implicit_symbol::ink(sources, index, style)
    else {
        return false;
    };
    ctx.begin_path();
    if shape.is_ellipse() {
        let _ = ctx.ellipse(x + cx, y + cy, rx, ry, 0.0, 0.0, std::f64::consts::TAU);
        if let Some(scale) = shape.inner_scale() {
            ctx.move_to(x + cx + rx * scale, y + cy);
            let _ = ctx.ellipse_with_anticlockwise(
                x + cx,
                y + cy,
                rx * scale,
                ry * scale,
                0.0,
                0.0,
                -std::f64::consts::TAU,
                true,
            );
        }
    } else {
        ctx.rect(x + cx - rx, y + cy - ry, 2.0 * rx, 2.0 * ry);
        if let Some(scale) = shape.inner_scale() {
            let (ix, iy) = (rx * scale, ry * scale);
            ctx.rect(x + cx + ix, y + cy - iy, -2.0 * ix, 2.0 * iy);
        }
    }
    ctx.fill();
    if stroke {
        ctx.stroke();
    }
    true
}
