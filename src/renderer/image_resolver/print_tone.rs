//! 한컴 2024 Print의 보이는 팔레트와 RGB555 bitmap 전달.
//!
//! BMP·PNG·JPEG·2020/2024 Print의 독립 대조는 stage2025,
//! 흑백 예약색·indexed 형식의 추가 경계는 stage2101에 있다. 그림 효과 합성은 별도 계약이다.

use std::collections::HashSet;
use std::io::Cursor;

use super::{decode_image_with_format_limited, detect_image_mime_type, memoized, Conversion};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

fn is_print_bitmap_candidate(image: &ImageNode) -> bool {
    image.hancom2024_print_bitmap
        && image.effect == ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image
            .data
            .as_deref()
            .is_some_and(|data| matches!(detect_image_mime_type(data), "image/bmp" | "image/png"))
}

pub(crate) fn print_bitmap_payload(image: &ImageNode) -> Option<ResolvedImagePayload> {
    if !requires_print_bitmap_transfer(image) {
        return None;
    }
    Some(ResolvedImagePayload {
        data: bitmap_bytes_to_png(image.data.as_deref()?)?,
        mime: "image/png",
        kind: ResolvedImageKind::FormatConverted,
        suppress_effects: false,
    })
}

/// 한컴 Print는 잘린 뒤 보이는 원본 픽셀과 흑백 예약색으로 팔레트를 결정한다.
/// 전체 원본의 색 수만 쓰면 같은 BinData를 다른 창으로 쓸 때 색 전달이 달라진다.
/// 전달 여부만 창별로 메모하고, 실제 색 변환 바이트는 원본별로 공유한다.
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
    let (width, height) = (header.width, header.height);
    let bounds = if let Some((left, top, right, bottom)) = image.crop {
        if right.checked_sub(left).is_none() || bottom.checked_sub(top).is_none() {
            return false;
        }
        let reference = super::image_crop_reference(image).filter(|&(w, h)| w > 0 && h > 0);
        if let Some((ref_width, ref_height)) = reference {
            // 원본 정수 비율로 계산해 정확한 끝점이 부동소수점 반올림 때문에
            // 한 픽셀 안으로 들어오지 않게 한다. 임의 epsilon은 사용하지 않는다.
            let edge = |value: i32, pixels: u32, reference: u32, ceil: bool| {
                let numerator = i64::from(value) * i64::from(pixels);
                let reference = i64::from(reference);
                let whole = numerator.div_euclid(reference);
                let rounded = whole + i64::from(ceil && numerator.rem_euclid(reference) != 0);
                rounded.clamp(0, i64::from(pixels)) as u32
            };
            [
                edge(left, width, ref_width, true),
                edge(top, height, ref_height, true),
                edge(right, width, ref_width, false),
                edge(bottom, height, ref_height, false),
            ]
        } else {
            // imgDim이 없는 원본은 실제 배치와 같은 적응 좌표계 해소를 사용한다.
            let (x, y, w, h) = crate::renderer::svg::compute_image_crop_src(
                (left, top, right, bottom),
                None,
                f64::from(width),
                f64::from(height),
            );
            [
                x.ceil().clamp(0.0, f64::from(width)) as u32,
                y.ceil().clamp(0.0, f64::from(height)) as u32,
                (x + w).floor().clamp(0.0, f64::from(width)) as u32,
                (y + h).floor().clamp(0.0, f64::from(height)) as u32,
            ]
        }
    } else {
        [0, 0, width, height]
    };
    if bounds[0] >= bounds[2] || bounds[1] >= bounds[3] {
        return false;
    }
    memoized(Conversion::Hancom2024BitmapPalette { bounds }, data, || {
        let format = match detect_image_mime_type(data) {
            "image/bmp" => image::ImageFormat::Bmp,
            "image/png" => image::ImageFormat::Png,
            _ => return None,
        };
        let decoded = decode_image_with_format_limited(data, format)?.to_rgba8();
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

pub(super) fn bitmap_bytes_to_png(data: &[u8]) -> Option<Vec<u8>> {
    memoized(Conversion::Hancom2024Bitmap, data, || {
        let format = match detect_image_mime_type(data) {
            "image/bmp" => image::ImageFormat::Bmp,
            "image/png" => image::ImageFormat::Png,
            _ => return None,
        };
        let mut decoded = decode_image_with_format_limited(data, format)?.to_rgba8();
        // 한컴의 팔레트 전달은 흑백 두 예약색을 포함한 RGB 색 수로 구분한다.
        // 원본이 255/256색이어도 빠진 예약색을 더하면 팔레트 용량을 넘을 수 있다.
        // 알파는 별도의 투명도이며 RGB 팔레트의 색 수에 합산하지 않는다.
        if !exceeds_palette(decoded.pixels()) {
            return None;
        }
        let (width, height) = decoded.dimensions();
        super::print_rgb555::quantize(decoded.as_mut(), width, height)?;
        let mut output = Vec::new();
        image::DynamicImage::ImageRgba8(decoded)
            .write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
            .ok()?;
        super::png_sampling::mark_print_bitmap(&mut output);
        super::png_sampling::preserve(data, &mut output);
        Some(output)
    })
}
