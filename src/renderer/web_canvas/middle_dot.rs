//! 가운뎃점은 SVG·Skia와 같은 전진 상자 중앙의 점으로 그린다.
//!
//! 독립 한컴 Print의 HCI Poppy Type3 점은 0.384em 전진 안에서
//! 가로 중심이 0.192em이다. 대체 글리프를 반폭으로 줄이면 이 중심과
//! 윤곽을 보존하지 못한다. 전진에는 이미 장평·자간이 반영되어 있다.

use web_sys::CanvasRenderingContext2d;

pub(super) fn paint(
    ctx: &CanvasRenderingContext2d,
    positions: &[f64],
    char_index: usize,
    x: f64,
    y: f64,
    font_size: f64,
    stroke: bool,
) {
    let advance = positions
        .get(char_index + 1)
        .zip(positions.get(char_index))
        .map_or(0.0, |(end, start)| end - start);
    let center_x = x + advance / 2.0;
    let center_y = y - font_size * super::super::render_tree::MIDDLE_DOT_CY_OFFSET_EM;
    let radius = font_size * super::super::render_tree::MIDDLE_DOT_RADIUS_EM;
    ctx.begin_path();
    let _ = ctx.arc(center_x, center_y, radius, 0.0, std::f64::consts::TAU);
    ctx.fill();
    if stroke {
        ctx.stroke();
    }
}

pub(super) fn paint_onepoint(
    ctx: &CanvasRenderingContext2d,
    style: &crate::renderer::TextStyle,
    font_size: f64,
    x: f64,
    y: f64,
    stroke: bool,
) -> bool {
    let Some(ink) = super::super::render_tree::fullwidth_onepoint_ink(style, font_size) else {
        return false;
    };
    ctx.begin_path();
    let _ = ctx.ellipse(
        x + ink.center_x,
        y - ink.above_baseline,
        ink.radius_x,
        ink.radius_y,
        0.0,
        0.0,
        std::f64::consts::TAU,
    );
    ctx.fill();
    if stroke {
        ctx.stroke();
    }
    true
}
