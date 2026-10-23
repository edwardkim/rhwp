//! Canvas2D의 목적 영역도 같은 Rust 장치 결과를 사용한다.

use crate::renderer::render_tree::{BoundingBox, ImageNode};
use web_sys::CanvasRenderingContext2d;

pub(super) fn bounds(
    context: &CanvasRenderingContext2d,
    bbox: BoundingBox,
    image: &ImageNode,
    print: bool,
) -> BoundingBox {
    if !print || !crate::paint::device_image::eligible(image) {
        return bbox;
    }
    context
        .get_transform()
        .ok()
        .and_then(|m| {
            crate::paint::device_image::bounds(bbox, [m.a(), m.b(), m.c(), m.d(), m.e(), m.f()])
        })
        .unwrap_or(bbox)
}
