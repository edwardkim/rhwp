//! 인쇄 래스터의 가는 직교 실선을 한 장치 픽셀 안에서 적분한다.
//!
//! 원본 선폭·색·줄/표 좌표는 바꾸지 않는다. 1px보다 가는 선을 두 픽셀에
//! 나누면 각 coverage가 절반이 되어 연한 표 선이 끊겨 보인다. 실제 raster
//! transform에서 법선 방향만 최대 반 픽셀 옮기고 같은 폭으로 칠한다.

use crate::paint::RenderProfile;

use super::render_tree::LineNode;
use super::{ArrowStyle, LineRenderType, StrokeDash};

pub(crate) fn raster_line_endpoints(
    line: &LineNode,
    matrix: [f64; 6],
    profile: RenderProfile,
    paint_width: f64,
) -> (f64, f64, f64, f64) {
    let original = (line.x1, line.y1, line.x2, line.y2);
    let style = &line.style;
    if !matches!(profile, RenderProfile::Print | RenderProfile::HighQuality)
        || style.line_type != LineRenderType::Single
        || style.compound_component
        || style.dash != StrokeDash::Solid
        || style.start_arrow != ArrowStyle::None
        || style.end_arrow != ArrowStyle::None
        || style.shadow.is_some()
        || !paint_width.is_finite()
        || paint_width <= 0.0
        || !style.width.is_finite()
        || style.width <= 0.0
        || ![line.x1, line.y1, line.x2, line.y2]
            .into_iter()
            .chain(matrix)
            .all(f64::is_finite)
    {
        return original;
    }
    let [a, b, c, d, e, f] = matrix;
    // 회전/기울임에서 두 법선 축을 따로 반올림하지 않는다. 축 반전과
    // 90도 회전은 같은 장치 직교선이며 역변환으로 원래 좌표계에 돌려준다.
    if !((b == 0.0 && c == 0.0) || (a == 0.0 && d == 0.0)) {
        return original;
    }
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant == 0.0 {
        return original;
    }
    let (dx, dy) = (line.x2 - line.x1, line.y2 - line.y1);
    let (nx, ny) = if dx != 0.0 && dy == 0.0 {
        (0.0, 1.0)
    } else if dx == 0.0 && dy != 0.0 {
        (1.0, 0.0)
    } else {
        return original;
    };
    let device_width = paint_width * (a * nx + c * ny).hypot(b * nx + d * ny);
    if !(0.0..1.0).contains(&device_width) || device_width == 0.0 {
        return original;
    }
    let (px, py) = (a * line.x1 + c * line.y1 + e, b * line.x1 + d * line.y1 + f);
    let (ex, ey) = (a * dx + c * dy, b * dx + d * dy);
    if ![px, py, ex, ey].into_iter().all(f64::is_finite) {
        return original;
    }
    let (shift_x, shift_y) = if ex != 0.0 && ey == 0.0 {
        (0.0, py.floor() + 0.5 - py)
    } else if ex == 0.0 && ey != 0.0 {
        (px.floor() + 0.5 - px, 0.0)
    } else {
        return original;
    };
    let local_x = (d * shift_x - c * shift_y) / determinant;
    let local_y = (-b * shift_x + a * shift_y) / determinant;
    let adjusted = (
        line.x1 + local_x,
        line.y1 + local_y,
        line.x2 + local_x,
        line.y2 + local_y,
    );
    if [adjusted.0, adjusted.1, adjusted.2, adjusted.3]
        .into_iter()
        .all(f64::is_finite)
    {
        adjusted
    } else {
        original
    }
}
