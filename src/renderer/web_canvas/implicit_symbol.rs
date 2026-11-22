//! 일반 글자·양각·음각·그림자 모두 같은 원본 기호 윤곽을 소비한다.

use crate::renderer::implicit_symbol::InkShape;
use crate::renderer::TextStyle;
use web_sys::{CanvasRenderingContext2d, CanvasWindingRule};

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
    let (cx, cy) = (x + cx, y + cy);
    let ellipse = |rx: f64, ry: f64| {
        ctx.move_to(cx + rx, cy);
        let _ = ctx.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::TAU);
    };
    let rect = |rx: f64, ry: f64| ctx.rect(cx - rx, cy - ry, 2.0 * rx, 2.0 * ry);
    ctx.begin_path();
    let rule = match shape {
        InkShape::Ellipse => {
            ellipse(rx, ry);
            CanvasWindingRule::Nonzero
        }
        InkShape::Rectangle => {
            rect(rx, ry);
            CanvasWindingRule::Nonzero
        }
        InkShape::Ring { inner } => {
            ellipse(rx, ry);
            ellipse(rx * inner, ry * inner);
            CanvasWindingRule::Evenodd
        }
        InkShape::Frame { inner } => {
            rect(rx, ry);
            rect(rx * inner, ry * inner);
            CanvasWindingRule::Evenodd
        }
    };
    ctx.fill_with_canvas_winding_rule(rule);
    if stroke {
        ctx.stroke();
    }
    true
}
