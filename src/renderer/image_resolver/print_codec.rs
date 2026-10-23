//! 원본 좌표와 분리된 Print 장치의 공통 이미지 압축 전달.

use std::collections::HashSet;

use jpeg_encoder::{ChromaSubsamplingMethod, ColorType, Encoder, SamplingFactor};

use super::{decode_image_with_format_limited, detect_image_mime_type, memoized, Conversion};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

/// 압축 형식과 출력 세대를 확인한 뒤 동일한 장치 표본 계약을 게시한다.
pub(crate) fn print_area_sampling(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> bool {
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
        && resolved.map_or_else(
            || {
                image
                    .data
                    .as_deref()
                    .is_some_and(|data| detect_image_mime_type(data) == "image/jpeg")
            },
            |payload| payload.mime == "image/jpeg",
        )
}

/// 폭은 최종 paint frame의 point 단위다. 논리 bbox에서 별도로 추정하지 않는다.
pub(crate) fn print_device_image_payload(
    image: &ImageNode,
    width_points: f64,
) -> Option<ResolvedImagePayload> {
    if !image.hancom2024_print_bitmap
        || image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.0
        || image.fill_mode.is_some()
        || !width_points.is_finite()
        || width_points <= 0.0
    {
        return None;
    }
    let source = image.data.as_deref()?;
    if !matches!(detect_image_mime_type(source), "image/bmp" | "image/png")
        || super::png_sampling::interpolation(source).is_some()
    {
        return None;
    }
    let header = super::canvaskit_encoded_image_header(source)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    // 부분 자르기는 source 영역의 정수 전송과 압축 순서를 함께 보존해야 한다.
    // 전체 원본의 압축을 한 뒤 clip하는 것으로 그 계약을 대신하지 않는다.
    if let Some((left, top, right, bottom)) = image.crop {
        let (width, height) = super::image_crop_reference(image)?;
        if left != 0
            || top != 0
            || i64::from(right) != i64::from(width)
            || i64::from(bottom) != i64::from(height)
        {
            return None;
        }
    }
    let quality = ((width_points as f32 / header.width as f32 * 100.0) as i32).clamp(35, 75) as u8;
    // 기존 색·형식 resolver의 결과를 압축하므로 색 전달을 다시 적용하지 않는다.
    let base = super::resolve_image_payload(image)?;
    let data = memoized(
        Conversion::Hancom2024PrintJpeg { quality },
        &base.data,
        || encode(&base, quality),
    )?;
    Some(ResolvedImagePayload {
        data,
        mime: "image/jpeg",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

fn encode(payload: &ResolvedImagePayload, quality: u8) -> Option<Vec<u8>> {
    let format = match payload.mime {
        "image/png" => image::ImageFormat::Png,
        "image/bmp" => image::ImageFormat::Bmp,
        _ => return None,
    };
    let rgba = decode_image_with_format_limited(&payload.data, format)?.to_rgba8();
    let (width, height) = (
        u16::try_from(rgba.width()).ok()?,
        u16::try_from(rgba.height()).ok()?,
    );
    if rgba.as_raw().len() / 4 * 3 < 4000 || rgba.pixels().any(|pixel| pixel[3] != 255) {
        return None;
    }
    let mut colors = HashSet::with_capacity(1025);
    for pixel in rgba.pixels() {
        colors.insert([pixel[0], pixel[1], pixel[2]]);
        if colors.len() > 1024 {
            break;
        }
    }
    if colors.len() < 32 {
        return None;
    }
    let quality = if colors.len() > 1024 && quality > 70 {
        (f64::from(quality) * 0.7) as u8
    } else {
        quality
    };
    let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
    let mut data = Vec::new();
    let mut encoder = Encoder::new(&mut data, quality);
    encoder.set_sampling_factor(SamplingFactor::F_2_2);
    encoder.set_chroma_subsampling_method(ChromaSubsamplingMethod::Average);
    // 컴포넌트별 순차 scan 대신 baseline interleaved JPEG를 공통 전달한다.
    // 실제 Canvas 이미지 decoder의 순차 scan 색 오류를 피하며 DCT·양자화는 유지한다.
    encoder.set_optimized_huffman_tables(false);
    encoder
        .encode(rgb.as_raw(), width, height, ColorType::Rgb)
        .ok()?;
    Some(data)
}
