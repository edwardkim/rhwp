//! Print의 단색 사각형은 clip과 다른 정수 채움 영역을 사용한다.
//
// Rectangle arithmetic adapted from PDFium CFX_RenderDevice::DrawPath.
// Copyright 2016 The PDFium Authors. Original copyright 2014 Foxit Software Inc.
// BSD-style license: licenses/pdfium/LICENSE.

use crate::renderer::render_tree::{BoundingBox, RectangleNode};

pub(crate) fn eligible(rect: &RectangleNode) -> bool {
    rect.corner_radius == 0.0
        && rect.gradient.is_none()
        && rect.style.fill_color.is_some()
        && rect.style.stroke_color.is_none()
        && rect.style.opacity == 1.0
        && rect.style.pattern.is_none()
        && rect.style.shadow.is_none()
}

fn axis(start: f32, end: f32) -> Option<(f64, f64)> {
    if !start.is_finite() || !end.is_finite() || end < start {
        return None;
    }
    let mut first = start.floor();
    let mut last = end.ceil();
    let length = (end - start).ceil().max(1.0);
    if first == last {
        last += 1.0;
    }
    if last - first >= length + 1.0 {
        if start - first > last - end {
            first += 1.0;
        } else {
            last -= 1.0;
        }
    }
    (first >= i32::MIN as f32 && last < i32::MAX as f32 && last > first)
        .then_some((f64::from(first), f64::from(last)))
}

/// 실제 행렬을 사용해 float32 끝점과 채움 영역을 모든 backend에 공유한다.
pub(crate) fn bounds(bbox: BoundingBox, [a, b, c, d, e, f]: [f64; 6]) -> Option<BoundingBox> {
    if ![bbox.x, bbox.y, bbox.width, bbox.height, a, b, c, d, e, f]
        .into_iter()
        .all(f64::is_finite)
        || bbox.width <= 0.0
        || bbox.height <= 0.0
        || a <= 0.0
        || d <= 0.0
        || b != 0.0
        || c != 0.0
    {
        return None;
    }
    let (x, y, w, h) = (
        bbox.x as f32,
        bbox.y as f32,
        bbox.width as f32,
        bbox.height as f32,
    );
    let (sx, sy, tx, ty) = (a as f32, d as f32, e as f32, f as f32);
    let (left, right) = axis(sx * x + tx, sx * (x + w) + tx)?;
    let (top, bottom) = axis(sy * y + ty, sy * (y + h) + ty)?;
    let mapped = BoundingBox::new(
        (left - e) / a,
        (top - f) / d,
        (right - left) / a,
        (bottom - top) / d,
    );
    [mapped.x, mapped.y, mapped.width, mapped.height]
        .into_iter()
        .all(f64::is_finite)
        .then_some(mapped)
}
