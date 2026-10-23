//! Canvas2D는 사각형 채움의 장치 경계를 별도로 추측하지 않는다.

use crate::renderer::render_tree::{BoundingBox, RectangleNode};
use web_sys::CanvasRenderingContext2d;

pub(super) fn bounds(
    context: &CanvasRenderingContext2d,
    bbox: BoundingBox,
    rectangle: &RectangleNode,
    print: bool,
) -> BoundingBox {
    if !print || !crate::paint::device_fill::eligible(rectangle) {
        return bbox;
    }
    context
        .get_transform()
        .ok()
        .and_then(|matrix| {
            crate::paint::device_fill::bounds(
                bbox,
                [
                    matrix.a(),
                    matrix.b(),
                    matrix.c(),
                    matrix.d(),
                    matrix.e(),
                    matrix.f(),
                ],
            )
        })
        .unwrap_or(bbox)
}
