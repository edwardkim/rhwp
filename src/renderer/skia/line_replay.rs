//! Native parallel shafts. Arrow geometry remains shared across output backends.

use crate::renderer::render_tree::LineNode;
use crate::renderer::LineRenderType;
use skia_safe::{Canvas, Paint};

/// 현재 canvas 변환에서 장치 한 칸에 해당하는 local 길이. 벡터 출력이거나 변환이 퇴화하면 `None`.
fn raster_device_unit(canvas: &Canvas, raster_target: bool) -> Option<f32> {
    if !raster_target {
        return None;
    }
    let inverse = canvas.local_to_device_as_3x3().invert()?;
    let x_unit = inverse.map_vector((1.0, 0.0)).length();
    let y_unit = inverse.map_vector((0.0, 1.0)).length();
    let device_unit = x_unit.min(y_unit);
    (device_unit.is_finite() && device_unit > 0.0).then_some(device_unit)
}

/// raster 출력의 획은 장치 한 칸보다 가늘게 그리지 않는다(PDF raster 의 얇은 선 규칙).
/// 조판 좌표·선언 굵기·점선 간격은 바꾸지 않고 현재 장치 변환만 사용한다.
pub(super) fn floor_stroke_to_device(canvas: &Canvas, paint: &mut Paint, raster_target: bool) {
    if let Some(device_unit) = raster_device_unit(canvas, raster_target) {
        paint.set_stroke_width(paint.stroke_width().max(device_unit));
    }
}

pub(super) fn draw_line(
    canvas: &Canvas,
    line: &LineNode,
    paint: &Paint,
    antialias: bool,
    raster_target: bool,
) {
    let (x1, y1, x2, y2) = (line.x1, line.y1, line.x2, line.y2);
    let width = line.style.width.max(0.5);
    let segments: &[(f64, f64)] = match line.style.line_type {
        LineRenderType::Single => {
            let mut paint = paint.clone();
            paint.set_anti_alias(antialias);
            canvas.draw_line((x1 as f32, y1 as f32), (x2 as f32, y2 as f32), &paint);
            return;
        }
        LineRenderType::Double => &[(0.30, -0.35), (0.30, 0.35)],
        LineRenderType::ThickThinDouble => &[(0.4, -0.30), (0.2, 0.40)],
        LineRenderType::ThinThickDouble => &[(0.2, -0.40), (0.4, 0.30)],
        LineRenderType::ThinThickThinTriple => &[(0.15, -0.425), (0.30, 0.0), (0.15, 0.425)],
    };
    let (dx, dy) = (x2 - x1, y2 - y1);
    let length = dx.hypot(dy);
    if length > 0.001 && length.is_finite() {
        for (width_ratio, offset_ratio) in segments {
            let mut segment = paint.clone();
            segment.set_anti_alias(antialias);
            segment.set_stroke_width((width * width_ratio).max(0.3) as f32);
            floor_stroke_to_device(canvas, &mut segment, raster_target);
            let offset = width * offset_ratio;
            let (ox, oy) = (-dy / length * offset, dx / length * offset);
            canvas.draw_line(
                ((x1 + ox) as f32, (y1 + oy) as f32),
                ((x2 + ox) as f32, (y2 + oy) as f32),
                &segment,
            );
        }
    }
}
