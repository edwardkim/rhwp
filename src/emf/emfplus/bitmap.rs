//! 표준 압축 bitmap의 실제 encoded 형식·크기·알파를 보존한다.

use std::io::Cursor;

use base64::Engine;

use super::{player::Budget, stream::u32_at};
use crate::renderer::image_header::{
    canvaskit_encoded_image_header, CanvasKitEncodedImageFormat, CANVASKIT_MAX_IMAGE_DIMENSION,
    CANVASKIT_MAX_IMAGE_PIXELS,
};

pub(super) fn compressed(data: &[u8], budget: &mut Budget) -> Option<String> {
    if u32_at(data, 24)? != 1 {
        return None;
    }
    // MS-EMFPLUS 2.2.2.2: 압축 그림의 선언 Width/Height/Stride/PixelFormat은 미정이다.
    // 해당 0 값을 거절하거나 문서의 외곽 크기로 대체하지 않는다.
    let bytes = data.get(28..)?;
    let header = canvaskit_encoded_image_header(bytes)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    let (format, mime) = match header.format {
        CanvasKitEncodedImageFormat::Png => (image::ImageFormat::Png, "image/png"),
        CanvasKitEncodedImageFormat::Jpeg => (image::ImageFormat::Jpeg, "image/jpeg"),
        _ => return None,
    };
    let reserve = bytes
        .len()
        .checked_add(2)?
        .checked_div(3)?
        .checked_mul(4)?
        .checked_add(256)?;
    budget.reserve_object_bytes(reserve)?;

    // 헤더만 유효한 잘린 그림을 완전한 EMF+ 재생으로 보고하지 않는다.
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(CANVASKIT_MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(CANVASKIT_MAX_IMAGE_DIMENSION);
    limits.max_alloc = CANVASKIT_MAX_IMAGE_PIXELS.checked_mul(4);
    reader.limits(limits);
    let decoded = reader.decode().ok()?;
    if decoded.width() != header.width || decoded.height() != header.height {
        return None;
    }
    let (w, h) = (header.width, header.height);
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\"><image width=\"{w}\" height=\"{h}\" href=\"data:{mime};base64,{encoded}\"/></svg>"
    ))
}
