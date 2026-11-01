//! 원본 좌표와 분리된 Print 장치의 공통 이미지 압축 전달.

use std::collections::HashSet;

use jpeg_encoder::{ChromaSubsamplingMethod, ColorType, Encoder, SamplingFactor};

use super::{detect_image_mime_type, memoized, Conversion};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

/// 기본 Print 압축의 고색수 품질 판정은 원본 팔레트와 요청 품질을 소비한다.
/// 사전 RGB555 변환이나 다른 인코더 진입점이 이 판정을 대신하지 않는다.
pub(super) fn quality_for_palette(quality: u8, source_colors: usize) -> u8 {
    if source_colors > 1024 && quality > 70 {
        (f64::from(quality) * 0.7) as u8
    } else {
        quality
    }
}

pub(super) fn requested_quality(width_points: f64, pixel_width: u32) -> u8 {
    ((width_points as f32 / pixel_width as f32 * 100.0) as i32).clamp(35, 75) as u8
}

/// 압축 형식과 출력 세대를 확인한 뒤 동일한 장치 표본 계약을 게시한다.
pub(crate) fn print_area_sampling(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> bool {
    if (resolved.is_some_and(|payload| {
        payload.kind == ResolvedImageKind::OutputDeviceWindow
            && super::png_sampling::print_separate_alpha(&payload.data)
    })) || (super::print_alpha_2024::eligible(image)
        && resolved.is_some_and(|payload| super::png_sampling::print_separate_alpha(&payload.data)))
        || crate::paint::metafile_picture::is_device_picture(image, resolved)
        || super::print_picture_resample::is_device_picture(image, resolved)
    {
        return true;
    }
    // 영역 맞춤 채움은 이미 검증한 전체 원본·장치 사각형 계약을 재사용한다.
    // 일반 PNG Picture의 팔레트별 재구성과 반복·부분 창은 이 경로로 넓히지 않는다.
    let bitmap_fill = crate::paint::device_image::eligible(image);
    let accepts_mime = |mime| mime == "image/jpeg" || (bitmap_fill && mime == "image/png");
    image.hancom2024_print_bitmap
        && image.effect == ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image.opacity == 1.0
        && (image.fill_mode.is_none() || bitmap_fill)
        && !image.transform.has_transform()
        && !image
            .data
            .as_deref()
            .is_some_and(|data| super::png_sampling::pdf_source(data))
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
                    .is_some_and(|data| accepts_mime(detect_image_mime_type(data)))
            },
            |payload| accepts_mime(payload.mime),
        )
}

/// 폭은 최종 paint frame의 point 단위다. 논리 bbox에서 별도로 추정하지 않는다.
pub(crate) fn print_device_image_payload(
    image: &ImageNode,
    width_points: f64,
    logical_size_points: (f64, f64),
    in_shape_group: bool,
) -> Option<ResolvedImagePayload> {
    // 묶음의 원본 그림은 자식 창이 아니라 전체 자원을 Print에 싣는 독립 계약이다.
    // 묶음 재생의 clip·행렬은 기존 경로를 보존한다.
    if !in_shape_group {
        if let Some(payload) = super::print_bitmap_window::payload(image, width_points) {
            return Some(payload);
        }
    }
    if let Some(payload) =
        super::print_picture_resample::payload(image, logical_size_points, width_points)
    {
        return Some(payload);
    }
    if let Some(payload) = super::print_alpha_2024::payload(image, width_points) {
        return Some(payload);
    }
    // 전체 JPEG 채움의 원본 RGB 계약은 Picture의 RGB555 전달과 다르다.
    // 같은 공통 진입점에서 자원을 만들고 모든 backend가 그 결과를 소비한다.
    if crate::paint::device_image::eligible(image)
        && image
            .data
            .as_deref()
            .is_some_and(|data| detect_image_mime_type(data) == "image/jpeg")
    {
        return super::print_fill_jpeg::payload(image, width_points);
    }
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
    if !matches!(
        detect_image_mime_type(source),
        "image/bmp" | "image/png" | "image/jpeg"
    ) || super::png_sampling::pdf_source(source)
    {
        return None;
    }
    let header = super::canvaskit_encoded_image_header(source)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    // 색 변환과 압축이 같은 창을 소비하며, 색 전달을 다시 적용하지 않는다.
    let base = super::resolve_image_payload_for_print(image, in_shape_group).unwrap_or_else(|| {
        ResolvedImagePayload {
            data: source.to_vec(),
            mime: detect_image_mime_type(source),
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }
    });
    if image.crop.is_some() && base.kind != ResolvedImageKind::OutputDeviceWindow {
        let (left, top, right, bottom) = image.crop?;
        let (width, height) = super::image_crop_reference(image)?;
        if left != 0
            || top != 0
            || i64::from(right) != i64::from(width)
            || i64::from(bottom) != i64::from(height)
        {
            return None;
        }
    }
    let output_header = super::canvaskit_encoded_image_header(&base.data)?;
    if !output_header.is_within_decode_limits() {
        return None;
    }
    let quality = requested_quality(width_points, output_header.width);
    // 압축 전의 같은 가시 창에서 색 수를 센다. base가 이미 RGB555 PNG면
    // 그 변환으로 줄어든 팔레트가 원본의 고색수 품질 계약을 바꾸지 않아야 한다.
    let quality = if quality > 70 {
        let format = match detect_image_mime_type(source) {
            "image/bmp" => image::ImageFormat::Bmp,
            "image/png" => image::ImageFormat::Png,
            "image/jpeg" => image::ImageFormat::Jpeg,
            _ => return None,
        };
        let window =
            super::print_window::SourceWindow::for_image(image, header.width, header.height)?;
        let (visible, _) =
            super::print_tone::decode_transfer_pixels(source, format, Some(window.bounds))?;
        let mut colors = HashSet::with_capacity(1025);
        for pixel in visible.pixels() {
            colors.insert([pixel[0], pixel[1], pixel[2]]);
            if colors.len() > 1024 {
                break;
            }
        }
        quality_for_palette(quality, colors.len())
    } else {
        quality
    };
    let data = memoized(
        Conversion::Hancom2024PrintJpeg { quality },
        &base.data,
        || encode(&base, quality),
    )?;
    Some(ResolvedImagePayload {
        data,
        mime: "image/jpeg",
        kind: if base.kind == ResolvedImageKind::OutputDeviceWindow {
            ResolvedImageKind::OutputDeviceWindow
        } else {
            ResolvedImageKind::OutputDevice
        },
        suppress_effects: false,
    })
}

fn encode(payload: &ResolvedImagePayload, quality: u8) -> Option<Vec<u8>> {
    let format = match payload.mime {
        "image/png" => image::ImageFormat::Png,
        "image/bmp" => image::ImageFormat::Bmp,
        "image/jpeg" => image::ImageFormat::Jpeg,
        _ => return None,
    };
    let (rgba, _) = super::print_tone::decode_transfer_pixels(&payload.data, format, None)?;
    if rgba.as_raw().len() / 4 * 3 < 4000 || rgba.pixels().any(|pixel| pixel[3] != 255) {
        return None;
    }
    encode_rgb(image::DynamicImage::ImageRgba8(rgba).to_rgb8(), quality)
}

pub(super) fn palette_len(rgb: &image::RgbImage) -> usize {
    let mut colors = HashSet::with_capacity(1025);
    for pixel in rgb.pixels() {
        colors.insert([pixel[0], pixel[1], pixel[2]]);
        if colors.len() > 1024 {
            break;
        }
    }
    colors.len()
}

/// RGB의 압축만 공유한다. 원본 색의 디코딩·RGB555 여부는 호출 계약이 결정한다.
pub(super) fn encode_rgb(rgb: image::RgbImage, quality: u8) -> Option<Vec<u8>> {
    let (width, height) = (
        u16::try_from(rgb.width()).ok()?,
        u16::try_from(rgb.height()).ok()?,
    );
    if rgb.as_raw().len() < 4000 || palette_len(&rgb) < 32 {
        return None;
    }
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
