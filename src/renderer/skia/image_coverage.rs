//! 실제 raster 행렬에서 공통 Print 그림 목적 영역을 소비한다.

use crate::renderer::render_tree::{BoundingBox, ImageNode};
use skia_safe::Canvas;

pub(super) fn bounds(
    canvas: &Canvas,
    bbox: BoundingBox,
    image: &ImageNode,
    print: bool,
) -> BoundingBox {
    if !print || !crate::paint::device_image::eligible(image) {
        return bbox;
    }
    let m = canvas.local_to_device_as_3x3();
    if m.has_perspective() {
        return bbox;
    }
    crate::paint::device_image::bounds(
        bbox,
        [
            m.scale_x(),
            m.skew_y(),
            m.skew_x(),
            m.scale_y(),
            m.translate_x(),
            m.translate_y(),
        ]
        .map(f64::from),
    )
    .unwrap_or(bbox)
}
