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
    let Some((cx, cy, rx, ry)) = crate::renderer::implicit_symbol::ink(sources, index, style)
    else {
        return false;
    };
    ctx.begin_path();
    let _ = ctx.ellipse(x + cx, y + cy, rx, ry, 0.0, 0.0, std::f64::consts::TAU);
    ctx.fill();
    if stroke {
        ctx.stroke();
    }
    true
}
