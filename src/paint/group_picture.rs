//! 묶음 자식 그림의 원본 자원과 자르기 프레임을 공통 paint로 발행한다.

use crate::paint::ResolvedImagePayload;
use crate::renderer::image_resolver::{
    detect_image_mime_type, image_crop_reference, print_device_image_payload, print_jpeg_payload,
};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

/// 한컴 Print에서 묶음의 그림은 전체 비트맵을 유지하고 물리 프레임으로 자른다.
/// 회전·채움·바깥 원본 영역 등은 기존 경로에 남긴다.
pub(super) fn project(
    image: &ImageNode,
    frame: BoundingBox,
    dpi: f64,
) -> Option<(BoundingBox, ImageNode, ResolvedImagePayload)> {
    if !image.hancom2024_print_bitmap
        || image.fill_mode.is_some()
        || image.content_inset.is_some()
        || image.transform.has_transform()
        || ![frame.x, frame.y, frame.width, frame.height]
            .into_iter()
            .all(f64::is_finite)
        || frame.width <= 0.0
        || frame.height <= 0.0
    {
        return None;
    }
    let (left, top, right, bottom) = image.crop?;
    let (width, height) = image_crop_reference(image)?;
    if left < 0
        || top < 0
        || right <= left
        || bottom <= top
        || i64::from(right) > i64::from(width)
        || i64::from(bottom) > i64::from(height)
        || (left == 0
            && top == 0
            && i64::from(right) == i64::from(width)
            && i64::from(bottom) == i64::from(height))
    {
        return None;
    }
    let crop_width = f64::from(right - left);
    let crop_height = f64::from(bottom - top);
    let full = BoundingBox::new(
        frame.x - frame.width * f64::from(left) / crop_width,
        frame.y - frame.height * f64::from(top) / crop_height,
        frame.width * f64::from(width) / crop_width,
        frame.height * f64::from(height) / crop_height,
    );
    if ![full.x, full.y, full.width, full.height]
        .into_iter()
        .all(f64::is_finite)
    {
        return None;
    }
    let mut original = image.clone();
    original.crop = None;
    original.original_size_hu = None;
    original.original_size = None;
    let payload = print_jpeg_payload(&original, full, dpi).or_else(|| {
        if original
            .data
            .as_deref()
            .is_none_or(|data| detect_image_mime_type(data) != "image/jpeg")
        {
            return None;
        }
        let width_points = full.width * 72.0 / dpi;
        let height_points = full.height * 72.0 / dpi;
        print_device_image_payload(&original, width_points, (width_points, height_points), true)
    })?;
    Some((full, original, payload))
}
