//! 잘린 불투명 비트맵의 Print 원본 창과 공통 재표본화.

use super::{
    detect_image_mime_type, memoized, print_alpha_2024 as print_alpha, print_codec,
    print_prefilter, print_tone, Conversion,
};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

/// 창의 정수 경계와 출력 크기는 각각 원본 좌표와 창의 길이에서 구한다.
/// 재표본화 필터는 실제 원본 안에 있는 창 바깥 픽셀도 소비한다.
pub(super) fn payload(image: &ImageNode, width_points: f64) -> Option<ResolvedImagePayload> {
    if !image.hancom2024_print_bitmap
        || image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.0
        || image.fill_mode.is_some()
        || image.content_inset.is_some()
        || image.transform.has_transform()
        || !width_points.is_finite()
        || width_points <= 0.0
    {
        return None;
    }
    let source = image.data.as_deref()?;
    if super::png_sampling::pdf_source(source) {
        return None;
    }
    let format = match detect_image_mime_type(source) {
        "image/bmp" => image::ImageFormat::Bmp,
        "image/png" => image::ImageFormat::Png,
        _ => return None,
    };
    let header = super::canvaskit_encoded_image_header(source)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    let (left, top, right, bottom) = image.crop?;
    let (ref_width, ref_height) = super::image_crop_reference(image)?;
    if ref_width == 0
        || ref_height == 0
        || left < 0
        || top < 0
        || right <= left
        || bottom <= top
        || i64::from(right) > i64::from(ref_width)
        || i64::from(bottom) > i64::from(ref_height)
    {
        return None;
    }
    let edge = |value: i32, pixels: u32, reference: u32| {
        (i64::from(value) * i64::from(pixels) / i64::from(reference)) as u32
    };
    let bounds = [
        edge(left, header.width, ref_width),
        edge(top, header.height, ref_height),
        edge(right, header.width, ref_width),
        edge(bottom, header.height, ref_height),
    ];
    let width = edge(right - left, header.width, ref_width);
    let height = edge(bottom - top, header.height, ref_height);
    if width == 0
        || height == 0
        || bounds[0] >= bounds[2]
        || bounds[1] >= bounds[3]
        || (bounds[2] - bounds[0] == width && bounds[3] - bounds[1] == height)
    {
        return None;
    }
    let (visible, quantized) = print_tone::decode_transfer_pixels(source, format, Some(bounds))?;
    let requested = print_codec::requested_quality(width_points, width);
    let quality = if requested > 70 {
        print_codec::quality_for_palette(
            requested,
            print_codec::palette_len(&image::DynamicImage::ImageRgba8(visible).to_rgb8()),
        )
    } else {
        requested
    };
    let data = memoized(
        Conversion::Hancom2024ResampledBitmapWindow {
            bounds,
            width,
            height,
            quality,
        },
        source,
        || {
            let rgba = super::decode_image_with_format_limited(source, format)?.to_rgba8();
            let resized = print_prefilter::resize_opaque_window(&rgba, bounds, (width, height))?;
            print_alpha::encode_planes_with_palette(resized, quality, quantized, |channel, _| {
                channel
            })
        },
    )?;
    Some(ResolvedImagePayload {
        data,
        mime: "image/png",
        kind: ResolvedImageKind::OutputDeviceWindow,
        suppress_effects: false,
    })
}
