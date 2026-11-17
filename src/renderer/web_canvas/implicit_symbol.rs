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
    match shape {
        crate::renderer::implicit_symbol::InkShape::Ellipse => {
            let _ = ctx.ellipse(x + cx, y + cy, rx, ry, 0.0, 0.0, std::f64::consts::TAU);
        }
        crate::renderer::implicit_symbol::InkShape::Rectangle => {
            ctx.rect(x + cx - rx, y + cy - ry, 2.0 * rx, 2.0 * ry);
        }
    }
    ctx.fill();
    if stroke {
        ctx.stroke();
    }
    true
}
