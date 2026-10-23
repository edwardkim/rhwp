//! 단색 Print 사각형도 공통 장치 채움 영역을 소비한다.

use crate::renderer::render_tree::{BoundingBox, RectangleNode};
use skia_safe::{Canvas, Rect};

pub(super) fn bounds(
    canvas: &Canvas,
    bbox: BoundingBox,
    rectangle: &RectangleNode,
    print_raster: bool,
) -> Option<Rect> {
    if !print_raster || !crate::paint::device_fill::eligible(rectangle) {
        return None;
    }
    let matrix = canvas.local_to_device_as_3x3();
    if matrix.has_perspective() {
        return None;
    }
    let mapped = crate::paint::device_fill::bounds(
        bbox,
        [
            matrix.scale_x(),
            matrix.skew_y(),
            matrix.skew_x(),
            matrix.scale_y(),
            matrix.translate_x(),
            matrix.translate_y(),
        ]
        .map(f64::from),
    )?;
    Some(Rect::from_xywh(
        mapped.x as f32,
        mapped.y as f32,
        mapped.width as f32,
        mapped.height as f32,
    ))
}
