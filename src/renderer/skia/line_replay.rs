//! Native parallel shafts. Arrow geometry remains shared across output backends.

use crate::renderer::render_tree::LineNode;
use crate::renderer::LineRenderType;
use skia_safe::{Canvas, Paint};

pub(super) fn draw_line(canvas: &Canvas, line: &LineNode, paint: &Paint, antialias: bool) {
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
