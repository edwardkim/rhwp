//! 인쇄 래스터의 가는 직교 실선에 최소 한 장치 픽셀의 점유 폭을 적용한다.
//!
//! PDFium RasterizeStroke의 최소 장치 폭과 독립 Print의 실제 coverage를 따른다.
//! 원본 줄/표 좌표·색·선폭 속성은 그대로 두고 현재 raster transform의
//! 법선 배율로 paint 폭을 계산하고 가는 선의 coverage를 장치 픽셀 중심에
//! 모은다. 레이아웃 좌표와 SVG/PDF 속성은 수정하지 않는다.

use crate::paint::RenderProfile;

use super::render_tree::LineNode;
use super::{ArrowStyle, LineRenderType, StrokeDash};

pub(crate) fn raster_line_geometry(
    line: &LineNode,
    matrix: [f64; 6],
    profile: RenderProfile,
    paint_width: f64,
) -> (f64, f64, f64, f64, f64) {
    let original = (line.x1, line.y1, line.x2, line.y2, paint_width);
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
    let normal_scale = (a * nx + c * ny).hypot(b * nx + d * ny);
    let device_width = paint_width * normal_scale;
    if !(0.0..1.0).contains(&device_width) || device_width == 0.0 {
        return original;
    }
    let raster_width = 1.0 / normal_scale;
    if !raster_width.is_finite() || raster_width <= 0.0 {
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
        raster_width,
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
