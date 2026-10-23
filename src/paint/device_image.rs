//! Print bitmap 채움의 그림 영역과 표본 재구성은 별도 계약이다.

use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::renderer::image_header::{canvaskit_encoded_image_header, CanvasKitEncodedImageFormat};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

pub(crate) fn eligible(image: &ImageNode) -> bool {
    image.hancom2024_print_bitmap
        && matches!(
            image.fill_mode,
            Some(ImageFillMode::FitToSize | ImageFillMode::Total)
        )
        && image.effect == ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image.opacity == 1.0
        && image.crop.is_none()
        && image.content_inset.is_none()
        && !image.transform.has_transform()
        && image
            .data
            .as_deref()
            .and_then(canvaskit_encoded_image_header)
            .is_some_and(|header| {
                header.is_within_decode_limits()
                    && matches!(
                        header.format,
                        CanvasKitEncodedImageFormat::Bmp
                            | CanvasKitEncodedImageFormat::Png
                            | CanvasKitEncodedImageFormat::Jpeg
                    )
            })
}

/// 최종 image matrix의 외접 영역을 목적 사각형으로 되돌린다.
/// clip과 같은 외접 연산을 공유하며 그림 내부의 표본·색·alpha는 변경하지 않는다.
pub(crate) fn bounds(bbox: BoundingBox, matrix: [f64; 6]) -> Option<BoundingBox> {
    super::device_clip::bitmap_fill(bbox, matrix)
}
