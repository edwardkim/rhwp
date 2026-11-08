//! 한컴 2024 Print의 보이는 팔레트와 RGB555 bitmap 전달.
//!
//! BMP·PNG·JPEG·2020/2024 Print의 독립 대조는 stage2025,
//! 흑백 예약색·indexed 형식의 추가 경계는 stage2101에 있다. 그림 효과 합성은 별도 계약이다.

use std::collections::HashSet;
use std::io::Cursor;

use super::print_window::SourceWindow;
use super::{decode_image_with_format_limited, detect_image_mime_type, memoized, Conversion};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

#[path = "print_jpeg.rs"]
mod print_jpeg;
pub(super) use print_jpeg::decode as decode_print_bitmap;
#[cfg(target_arch = "wasm32")]
pub(crate) use print_jpeg::decode_replay as decode_print_jpeg_replay;

/// 전체 JPEG 채움은 독립 Print의 IJG 원본 RGB를 압축한다.
/// Picture의 Intel IDCT·RGB555 전달이나 최종 출력의 재색변환으로 대체하지 않는다.
pub(super) fn decode_jpeg_fill_rgb(data: &[u8]) -> Option<image::RgbImage> {
    if !data.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    print_jpeg::decode_jpeg(data, true).map(image::DynamicImage::into_rgb8)
}

fn is_print_bitmap_candidate(image: &ImageNode) -> bool {
    image.hancom2024_print_bitmap
        && !image
            .data
            .as_deref()
            .is_some_and(super::png_sampling::pdf_source)
        && image.effect == ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image
            .data
            .as_deref()
            .is_some_and(|data| format(data).is_some())
}

pub(crate) fn print_bitmap_payload(
    image: &ImageNode,
    in_shape_group: bool,
) -> Option<ResolvedImagePayload> {
    if !is_print_bitmap_candidate(image) {
        return None;
    }
    let source = image.data.as_deref()?;
    let header = super::canvaskit_encoded_image_header(source)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    let window = SourceWindow::for_image(image, header.width, header.height)?;
    // 자연 크기·반복 채움은 crop 이후의 반복 단위와 원점을 별도로 소비한다.
    if window.contained && window.is_partial() && image.fill_mode.is_none() {
        let bounds = window.bounds;
        let data = memoized(
            Conversion::Hancom2024BitmapWindow {
                bounds,
                group: in_shape_group,
            },
            source,
            || {
                let (mut visible, quantized) = decode_transfer_pixels_for_group(
                    source,
                    format(source)?,
                    Some(bounds),
                    in_shape_group,
                )?;
                if quantized {
                    let (width, height) = visible.dimensions();
                    super::print_rgb555::quantize(visible.as_mut(), width, height)?;
                }
                to_png(visible, source, quantized)
            },
        )?;
        return Some(ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::OutputDeviceWindow,
            suppress_effects: false,
        });
    }
    if !requires_print_bitmap_transfer(image) {
        return None;
    }
    Some(ResolvedImagePayload {
        data: bitmap_bytes_to_png(source, in_shape_group)?,
        mime: "image/png",
        kind: ResolvedImageKind::FormatConverted,
        suppress_effects: false,
    })
}

/// 한컴 Print는 잘린 뒤 보이는 원본 픽셀과 흑백 예약색으로 팔레트를 결정한다.
/// 전체 원본의 색 수만 쓰면 같은 BinData를 다른 창으로 쓸 때 색 전달이 달라진다.
/// 창을 담은 payload는 원본 bin key 대신 그 바이트로 전달한다.
pub(super) fn requires_print_bitmap_transfer(image: &ImageNode) -> bool {
    if !is_print_bitmap_candidate(image) {
        return false;
    }
    let Some(data) = image.data.as_deref() else {
        return false;
    };
    let Some(header) = crate::renderer::image_header::canvaskit_encoded_image_header(data)
        .filter(|header| header.is_within_decode_limits())
    else {
        return false;
    };
    let Some(window) = SourceWindow::for_image(image, header.width, header.height) else {
        return false;
    };
    let bounds = window.bounds;
    memoized(Conversion::Hancom2024BitmapPalette { bounds }, data, || {
        let decoded = decode_image_with_format_limited(data, format(data)?)?.to_rgba8();
        let pixels = (bounds[1]..bounds[3]).flat_map(|y| {
            let decoded = &decoded;
            (bounds[0]..bounds[2]).map(move |x| decoded.get_pixel(x, y))
        });
        exceeds_palette(pixels).then(Vec::new)
    })
    .is_some()
}

fn exceeds_palette<'a>(pixels: impl Iterator<Item = &'a image::Rgba<u8>>) -> bool {
    const PALETTE_CAPACITY: usize = 1 << 8;
    let mut colors = HashSet::with_capacity(PALETTE_CAPACITY + 1);
    colors.extend([[0, 0, 0], [255, 255, 255]]);
    for pixel in pixels {
        colors.insert([pixel[0], pixel[1], pixel[2]]);
        if colors.len() > PALETTE_CAPACITY {
            return true;
        }
    }
    false
}

pub(super) fn bitmap_bytes_to_png(data: &[u8], in_shape_group: bool) -> Option<Vec<u8>> {
    memoized(
        Conversion::Hancom2024Bitmap {
            group: in_shape_group,
        },
        data,
        || {
            let (mut decoded, quantized) =
                decode_transfer_pixels_for_group(data, format(data)?, None, in_shape_group)?;
            // 한컴의 팔레트 전달은 흑백 두 예약색을 포함한 RGB 색 수로 구분한다.
            // 원본이 255/256색이어도 빠진 예약색을 더하면 팔레트 용량을 넘을 수 있다.
            // 알파는 별도의 투명도이며 RGB 팔레트의 색 수에 합산하지 않는다.
            if !quantized {
                return None;
            }
            let (width, height) = decoded.dimensions();
            super::print_rgb555::quantize(decoded.as_mut(), width, height)?;
            to_png(decoded, data, true)
        },
    )
}

/// 변환 전에 원본 팔레트를 판정하고, 선택한 같은 창만 Print에 전달한다.
pub(super) fn decode_transfer_pixels(
    data: &[u8],
    format: image::ImageFormat,
    bounds: Option<[u32; 4]>,
) -> Option<(image::RgbaImage, bool)> {
    decode_transfer_pixels_for_group(data, format, bounds, false)
}

fn decode_transfer_pixels_for_group(
    data: &[u8],
    format: image::ImageFormat,
    bounds: Option<[u32; 4]>,
    in_shape_group: bool,
) -> Option<(image::RgbaImage, bool)> {
    let visible = |decoded: image::DynamicImage| {
        let rgba = decoded.to_rgba8();
        match bounds {
            Some([left, top, right, bottom]) => {
                image::imageops::crop_imm(&rgba, left, top, right - left, bottom - top).to_image()
            }
            None => rgba,
        }
    };
    let mut rgba = visible(decode_image_with_format_limited(data, format)?);
    let quantized = exceeds_palette(rgba.pixels());
    if quantized && format == image::ImageFormat::Jpeg {
        // 팔레트 분류용 버퍼와 Print 디코딩 버퍼를 동시에 보존하지 않는다.
        drop(rgba);
        rgba = visible(if in_shape_group {
            print_jpeg::decode_jpeg(data, true)?
        } else {
            decode_print_bitmap(data, format)?
        });
    }
    Some((rgba, quantized))
}

fn format(data: &[u8]) -> Option<image::ImageFormat> {
    match detect_image_mime_type(data) {
        "image/bmp" => Some(image::ImageFormat::Bmp),
        "image/png" => Some(image::ImageFormat::Png),
        "image/jpeg" => Some(image::ImageFormat::Jpeg),
        _ => None,
    }
}

fn to_png(rgba: image::RgbaImage, source: &[u8], quantized: bool) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
        .ok()?;
    if quantized {
        super::png_sampling::mark_print_bitmap(&mut output);
    }
    super::png_sampling::preserve(source, &mut output);
    Some(output)
}
