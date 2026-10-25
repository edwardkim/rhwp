//! Print 전용의 제한된 JPEG 표본 변환. 화면·PDF import의 기본 decoder는 바꾸지 않는다.

use std::cell::Cell;

use zune_print::allocation::AllocationBudget;
use zune_print::zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
use zune_print::JpegDecoder;

#[path = "print_jpeg_idct.rs"]
mod idct;

struct Budget(Cell<usize>);

impl AllocationBudget for Budget {
    fn reserve(&self, bytes: usize, _kind: &'static str) -> bool {
        let Some(remaining) = self.0.get().checked_sub(bytes) else {
            return false;
        };
        self.0.set(remaining);
        true
    }

    fn reject(&self) {}
}

pub(crate) fn decode(data: &[u8], format: image::ImageFormat) -> Option<image::DynamicImage> {
    if format != image::ImageFormat::Jpeg {
        return super::super::decode_image_with_format_limited(data, format);
    }
    decode_jpeg(data, false)
}

/// 최종 Print JPEG 재생. 원본 사진의 Intel IDCT·RGB555 변환과 구분한다.
#[cfg(target_arch = "wasm32")]
pub(crate) fn decode_replay(data: &[u8]) -> Option<image::DynamicImage> {
    if !data.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    decode_jpeg(data, true)
}

pub(super) fn decode_jpeg(data: &[u8], final_replay: bool) -> Option<image::DynamicImage> {
    let header = super::super::canvaskit_encoded_image_header(data)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    let budget = Budget(Cell::new(
        usize::try_from(crate::renderer::image_header::CANVASKIT_MAX_IMAGE_PIXELS.checked_mul(4)?)
            .ok()?,
    ));
    let options = DecoderOptions::default()
        .set_max_width(crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION as usize)
        .set_max_height(crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION as usize)
        .set_use_unsafe(false)
        .jpeg_set_out_colorspace(ColorSpace::RGB);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(data), options);
    decoder.set_allocation_budget(&budget).ok()?;
    decoder
        .set_transforms(
            if final_replay {
                zune_print::transforms::ijg_replay_idct
            } else {
                idct::transform
            },
            zune_print::transforms::ijg_fancy_upsampling,
        )
        .ok()?;
    decoder.decode_headers().ok()?;
    if !matches!(
        decoder.input_colorspace()?,
        ColorSpace::RGB | ColorSpace::YCbCr | ColorSpace::Luma
    ) {
        drop(decoder);
        return super::super::decode_image_with_format_limited(data, image::ImageFormat::Jpeg);
    }
    let ycbcr = final_replay && decoder.input_colorspace()? == ColorSpace::YCbCr;
    if ycbcr {
        decoder.set_options(options.jpeg_set_out_colorspace(ColorSpace::YCbCr));
    }
    let (width, height) = decoder.dimensions()?;
    if width != header.width as usize || height != header.height as usize {
        return None;
    }
    let length = decoder.output_buffer_size()?;
    if length != width.checked_mul(height)?.checked_mul(3)?
        || !budget.reserve(length, "Print RGB output")
    {
        return None;
    }
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(length).ok()?;
    pixels.resize(length, 0);
    decoder.decode_into(&mut pixels).ok()?;
    if ycbcr {
        zune_print::transforms::ijg_replay_ycbcr_to_rgb(&mut pixels);
    }
    Some(image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(
        header.width,
        header.height,
        pixels,
    )?))
}
