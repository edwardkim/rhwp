//! 한컴 2024 Print의 색 bitmap과 별도 이진 alpha 마스크 전달.
//!
//! 원본 PNG의 straight RGB를 보존한다. 색·마스크는 각각 장치 표본을 낸 뒤
//! 합성하므로, backend의 PNG 디코딩으로 투명 픽셀 아래 색을 먼저 잃지 않는다.

#[path = "print_alpha_core.rs"]
mod core;
use core::{bayer_rank, print_color};

use std::io::Cursor;

use super::{memoized, print_codec, print_tone, Conversion};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

pub(super) fn eligible(image: &ImageNode) -> bool {
    image.hancom2024_print_bitmap
        && image.effect == ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image.opacity == 1.0
        && image.fill_mode.is_none()
        && !image.transform.has_transform()
        && image.crop.is_none_or(|(left, top, right, bottom)| {
            super::image_crop_reference(image).is_some_and(|(width, height)| {
                left == 0
                    && top == 0
                    && i64::from(right) == i64::from(width)
                    && i64::from(bottom) == i64::from(height)
            })
        })
        && image.data.as_deref().is_some_and(|data| {
            data.starts_with(b"\x89PNG\r\n\x1a\n") && !super::png_sampling::pdf_source(data)
        })
}

pub(super) fn payload(image: &ImageNode, width_points: f64) -> Option<ResolvedImagePayload> {
    if !eligible(image) || !width_points.is_finite() || width_points <= 0.0 {
        return None;
    }
    let source = image.data.as_deref()?;
    let header = super::canvaskit_encoded_image_header(source)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    let quality = print_codec::requested_quality(width_points, header.width);
    let data = memoized(Conversion::Hancom2024PrintAlpha { quality }, source, || {
        let (rgba, quantized) =
            print_tone::decode_transfer_pixels(source, image::ImageFormat::Png, None)?;
        // 팔레트 bitmap과 불투명 PNG는 기존 색 전달 계약을 유지한다.
        if !quantized || rgba.pixels().all(|pixel| pixel[3] == 255) {
            return None;
        }
        let quality = print_codec::quality_for_palette(
            quality,
            print_codec::palette_len(&image::DynamicImage::ImageRgba8(rgba.clone()).to_rgb8()),
        );
        encode_planes(rgba, quality, print_color)
    })?;
    Some(ResolvedImagePayload {
        data,
        mime: "image/png",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

/// 색과 coverage를 따로 표본화하는 공통 payload를 생성한다.
/// 복원한 straight RGB와 원본 PArgb의 색 전달 계약은 호출자가 구분한다.
pub(super) fn encode_planes(
    rgba: image::RgbaImage,
    quality: u8,
    color: impl Fn(u8, u8) -> u8,
) -> Option<Vec<u8>> {
    encode_planes_with_palette(rgba, quality, true, color)
}

pub(super) fn encode_planes_with_palette(
    mut rgba: image::RgbaImage,
    quality: u8,
    quantize: bool,
    color: impl Fn(u8, u8) -> u8,
) -> Option<Vec<u8>> {
    let (width, height) = rgba.dimensions();
    let mut mask = Vec::with_capacity(rgba.as_raw().len() / 4);
    for (x, y, pixel) in rgba.enumerate_pixels_mut() {
        let alpha = pixel[3];
        let opaque = alpha == 255 || alpha > bayer_rank(x, y);
        mask.push(if opaque { 255 } else { 0 });
        for channel in &mut pixel.0[..3] {
            *channel = if opaque { color(*channel, alpha) } else { 255 };
        }
        pixel[3] = 255;
    }
    if quantize {
        super::print_rgb555::quantize(rgba.as_mut(), width, height)?;
    }
    let jpeg = print_codec::encode_rgb(image::DynamicImage::ImageRgba8(rgba).to_rgb8(), quality)?;
    let pixels = print_tone::decode_jpeg_fill_rgb(&jpeg)?.into_raw();
    let mut combined = Vec::with_capacity(mask.len() * 4);
    for (rgb, alpha) in pixels.chunks_exact(3).zip(mask) {
        combined.extend_from_slice(rgb);
        combined.push(alpha);
    }
    let rgba = image::RgbaImage::from_raw(width, height, combined)?;
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .ok()?;
    super::png_sampling::mark_print_separate_alpha(&mut bytes);
    Some(bytes)
}
