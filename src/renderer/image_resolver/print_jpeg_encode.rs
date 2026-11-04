//! 독립 한컴 Print 표본의 불투명 raster 압축 전달.
//!
//! 17/18/24/31색은 무손실, 32/256색은 JPEG 4:2:0으로 전달됐다.
//! 품질은 원본 폭에 대한 96dpi 인쇄 폭의 비율로 정해지며 Y 배율과 무관하다.
//! 편집 원본을 바꾸지 않고 배치별 출력 payload만 만든다.

use std::collections::HashSet;

use jpeg_encoder::{ChromaSubsamplingMethod, ColorType, Encoder, SamplingFactor};

use super::{decode_image_with_format_limited, detect_image_mime_type, memoized, Conversion};
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

pub(crate) fn print_jpeg_payload(
    image: &ImageNode,
    bbox: BoundingBox,
    dpi: f64,
) -> Option<ResolvedImagePayload> {
    if image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.0
        || image.content_inset.is_some()
        || !matches!(
            image.fill_mode,
            None | Some(ImageFillMode::FitToSize | ImageFillMode::Total)
        )
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    let data = image.data.as_deref()?;
    if super::png_sampling::pdf_source(data) {
        return None;
    }
    match detect_image_mime_type(data) {
        "image/bmp" | "image/png" | "image/gif" => (),
        _ => return None,
    }
    let header = crate::renderer::image_header::canvaskit_encoded_image_header(data)
        .filter(|header| header.is_within_decode_limits())?;
    let (width, height) = (header.width, header.height);
    if let Some(crop) = image.crop {
        let source = crate::renderer::svg::compute_image_crop_src(
            crop,
            super::image_crop_reference(image),
            f64::from(width),
            f64::from(height),
        );
        if source != (0.0, 0.0, f64::from(width), f64::from(height)) {
            return None;
        }
    }
    let target = image.transform.effective_image_bbox(&bbox);
    let nominal_width = (target.width * crate::renderer::DEFAULT_DPI / dpi).floor();
    if !nominal_width.is_finite() || nominal_width <= 0.0 {
        return None;
    }
    let quality = (75.0 * nominal_width / f64::from(width))
        .floor()
        .clamp(35.0, 75.0) as u8;
    let expand = image.hancom2024_print_bitmap;
    let bytes = encode_print_jpeg(data, quality, expand)?;
    Some(ResolvedImagePayload {
        data: bytes,
        mime: "image/jpeg",
        // 품질은 배치에 종속된다. BinData 신원으로 다른 폭의 결과를 재사용하지 않는다.
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

/// 중첩 메타파일 자원도 동일한 팔레트·양자화·전달 마커를 소비한다.
/// 최초 사용 품질을 고르는 책임과 현재 배치의 프레임 책임은 호출자에게 있다.
pub(crate) fn encode_print_jpeg(data: &[u8], quality: u8, expand: bool) -> Option<Vec<u8>> {
    if !(35..=75).contains(&quality) {
        return None;
    }
    let format = match detect_image_mime_type(data) {
        "image/bmp" => image::ImageFormat::Bmp,
        "image/png" => image::ImageFormat::Png,
        "image/gif" => image::ImageFormat::Gif,
        _ => return None,
    };
    let header = crate::renderer::image_header::canvaskit_encoded_image_header(data)
        .filter(|header| header.is_within_decode_limits())?;
    let (width, height) = (header.width, header.height);
    memoized(Conversion::PrintJpeg { quality, expand }, data, || {
        let mut decoded = decode_image_with_format_limited(data, format)?.to_rgba8();
        // RGB555 전달과 JPEG 품질은 서로 다른 팔레트 경계를 소비한다.
        // 변환 전 원본에서 큰 경계까지 세고 알파 검사는 모든 픽셀에서 유지한다.
        let mut colors = HashSet::with_capacity(1025);
        for pixel in decoded.pixels() {
            if pixel[3] != 255 {
                return None;
            }
            if colors.len() <= 1024 {
                colors.insert([pixel[0], pixel[1], pixel[2]]);
            }
        }
        if colors.len() < 32 {
            return None;
        }
        let quality = if expand {
            super::print_codec::quality_for_palette(quality, colors.len())
        } else {
            quality
        };
        colors.extend([[0, 0, 0], [255, 255, 255]]);
        let reconstruction = expand && colors.len() > 256;
        if reconstruction {
            // 257색 이상 전달은 RGB555 장치의 4×4 ordered dither를 거친다.
            // 단순 c + (c >> 5)는 평균 명도만 근사하며 저명도 타일의 실제
            // JPEG AC 성분을 잃는다. 독립 1..255 타일의 decoder pixels로 확인했다.
            const BAYER: [[u8; 4]; 4] =
                [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
            for (x, y, pixel) in decoded.enumerate_pixels_mut() {
                for channel in &mut pixel.0[..3] {
                    // 독립 125..129행의 원본들은 y-height 위상을 따른다.
                    // 배치 좌표가 아니라 DIB scan surface의 원본 높이가 정한다.
                    let threshold =
                        BAYER[(y as usize + 4 - height as usize % 4) % 4][x as usize % 4];
                    let value = (*channel & 0xf8)
                        .saturating_add(u8::from(2 * (*channel & 7) > threshold) * 8)
                        .min(248);
                    *channel = value.saturating_add(value >> 5);
                }
            }
        }
        let rgb = image::DynamicImage::ImageRgba8(decoded).to_rgb8();
        let mut output = Vec::new();
        let mut encoder = Encoder::new(&mut output, quality);
        encoder.set_sampling_factor(SamplingFactor::F_2_2);
        encoder.set_chroma_subsampling_method(ChromaSubsamplingMethod::Average);
        encoder
            .encode(
                rgb.as_raw(),
                width.try_into().ok()?,
                height.try_into().ok()?,
                ColorType::Rgb,
            )
            .ok()?;
        super::png_sampling::mark_print_jpeg(&mut output, reconstruction);
        Some(output)
    })
}
