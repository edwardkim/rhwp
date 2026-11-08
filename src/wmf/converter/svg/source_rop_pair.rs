//! 같은 원본/목적 창의 연속 OR/AND 마스크를 단일 투명 그림으로 내린다.
//!
//! 모든 목적 픽셀 값에 대해 상수 또는 항등인 경우만 alpha로 표현한다.
//! 흰색 자체를 투명색으로 취급하지 않으며 원본 WMF 바이트는 보존한다.

use super::device_context::BlitDestRect;
use crate::wmf::{converter::Bitmap, parser::TernaryRasterOperation};

pub(super) struct SourceBlit<'a> {
    pub record: usize,
    pub element: usize,
    pub operation: TernaryRasterOperation,
    pub destination: BlitDestRect,
    pub source: [i16; 4],
    pub bitmap: &'a Bitmap,
}

pub(super) struct PendingSourcePair {
    record: usize,
    element: usize,
    operation: TernaryRasterOperation,
    destination: BlitDestRect,
    source: [i16; 4],
    pixels: image::RgbaImage,
}

pub(super) struct ComposedPair {
    pub previous_element: usize,
    pub png: Vec<u8>,
}

impl PendingSourcePair {
    pub(super) fn observe(
        pending: &mut Option<Self>,
        blit: SourceBlit<'_>,
    ) -> Option<ComposedPair> {
        let previous = pending.take();
        if !matches!(
            blit.operation,
            TernaryRasterOperation::SRCAND | TernaryRasterOperation::SRCPAINT
        ) {
            return None;
        }
        let pixels = crate::renderer::image_resolver::decode_image_with_format_limited(
            blit.bitmap.as_slice(),
            image::ImageFormat::Bmp,
        )?
        .to_rgba8();
        // 64MiB의 단일 대기 표본만 보유한다. 잘린 원본은 전체 그림 계약으로 합치지 않는다.
        if u64::from(pixels.width()) * u64::from(pixels.height()) > 16_777_216
            || blit.source
                != [
                    0,
                    0,
                    i16::try_from(pixels.width()).ok()?,
                    i16::try_from(pixels.height()).ok()?,
                ]
        {
            return None;
        }
        if let Some(first) = previous.filter(|first| {
            first.record.checked_add(1) == Some(blit.record)
                && first.destination == blit.destination
                && first.source == blit.source
                && first.operation != blit.operation
        }) {
            if let Some(composed) = compose(
                &first.pixels,
                &pixels,
                first.operation == TernaryRasterOperation::SRCPAINT,
            ) {
                let mut output = std::io::Cursor::new(Vec::new());
                image::DynamicImage::ImageRgba8(composed)
                    .write_to(&mut output, image::ImageFormat::Png)
                    .ok()?;
                return Some(ComposedPair {
                    previous_element: first.element,
                    png: output.into_inner(),
                });
            }
        }
        *pending = Some(Self {
            record: blit.record,
            element: blit.element,
            operation: blit.operation,
            destination: blit.destination,
            source: blit.source,
            pixels,
        });
        None
    }
}

/// (D | A) & B 또는 (D & A) | B를 모든 D에 대해 증명한다.
fn compose(
    first: &image::RgbaImage,
    second: &image::RgbaImage,
    or_first: bool,
) -> Option<image::RgbaImage> {
    if first.dimensions() != second.dimensions() {
        return None;
    }
    let mut result = image::RgbaImage::new(first.width(), first.height());
    for ((first, second), output) in first.pixels().zip(second.pixels()).zip(result.pixels_mut()) {
        if first[3] != 255 || second[3] != 255 {
            return None;
        }
        let mut constant = [0; 3];
        let mut identities = 0;
        let mut constants = 0;
        for channel in 0..3 {
            let (variable, value) = if or_first {
                (
                    second[channel] & !first[channel],
                    second[channel] & first[channel],
                )
            } else {
                (first[channel] & !second[channel], second[channel])
            };
            if variable == 255 && value == 0 {
                identities += 1;
            } else if variable == 0 {
                constants += 1;
                constant[channel] = value;
            } else {
                return None;
            }
        }
        if identities == 3 {
            *output = image::Rgba([0, 0, 0, 0]);
        } else if constants == 3 {
            *output = image::Rgba([constant[0], constant[1], constant[2], 255]);
        } else {
            return None;
        }
    }
    Some(result)
}
