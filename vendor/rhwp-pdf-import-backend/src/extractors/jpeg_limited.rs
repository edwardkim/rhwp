//! RGB/Gray JPEG decoding with a request context at actual reservations.
//! CMYK retains its separate Adobe-aware decoder in `images`.
use crate::error::{Error, Result};
use crate::import_limits::ImportBudget;
use std::io::Read;
use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
use zune_jpeg_limited::{allocation::AllocationBudget, JpegDecoder};

struct Request<'a>(&'a ImportBudget);

impl AllocationBudget for Request<'_> {
    fn reserve(&self, bytes: usize, _kind: &'static str) -> bool {
        self.0.charge_codec_allocation(bytes).is_ok()
    }

    fn reject(&self) {
        self.0.reject("JPEG allocation");
    }
}

pub(super) fn decode(
    bytes: &[u8],
    width: u32,
    height: u32,
    budget: &ImportBudget,
) -> Result<image::DynamicImage> {
    budget.check()?;
    let mut input = Vec::new();
    let copied = super::jpeg_input::Input::new(bytes, budget).read_to_end(&mut input);
    budget.check()?;
    copied.map_err(|_| Error::Image("bounded JPEG input rejected".into()))?;
    // Match image 0.25.10's non-strict decoding and output color policy.
    let options = DecoderOptions::default()
        .set_strict_mode(false)
        .set_max_width(width as usize)
        .set_max_height(height as usize);
    let request = Request(budget);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(input.as_slice()), options);
    decoder
        .set_allocation_budget(&request)
        .map_err(|_| budget.reject("JPEG request context"))?;
    let header = decoder.decode_headers();
    budget.check()?;
    header.map_err(|_| Error::Image("bounded JPEG header rejected".into()))?;
    if decoder.dimensions() != Some((width as usize, height as usize)) {
        return Err(Error::Image("JPEG dimensions differ from PDF declaration".into()));
    }
    let color = match decoder.input_colorspace() {
        Some(
            color @ (ColorSpace::RGB | ColorSpace::RGBA | ColorSpace::Luma | ColorSpace::LumaA),
        ) => color,
        _ => ColorSpace::RGB,
    };
    decoder.set_options(decoder.options().jpeg_set_out_colorspace(color));
    let size = decoder
        .output_buffer_size()
        .filter(|&size| size <= budget.limits.stream_bytes)
        .ok_or_else(|| budget.reject("JPEG decoded pixel buffer"))?;
    budget.charge_codec_allocation(size)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(size)
        .map_err(|_| budget.reject("JPEG decoded pixel allocation"))?;
    pixels.resize(size, 0);
    let decoded = decoder.decode_into(&mut pixels);
    budget.check()?;
    decoded.map_err(|_| Error::Image("bounded JPEG decode failed".into()))?;
    match color {
        ColorSpace::Luma => {
            image::GrayImage::from_raw(width, height, pixels).map(image::DynamicImage::ImageLuma8)
        },
        ColorSpace::LumaA => image::GrayAlphaImage::from_raw(width, height, pixels)
            .map(image::DynamicImage::ImageLumaA8),
        ColorSpace::RGB => {
            image::RgbImage::from_raw(width, height, pixels).map(image::DynamicImage::ImageRgb8)
        },
        ColorSpace::RGBA => {
            image::RgbaImage::from_raw(width, height, pixels).map(image::DynamicImage::ImageRgba8)
        },
        _ => unreachable!(),
    }
    .ok_or_else(|| Error::Image("invalid JPEG pixel buffer".into()))
}
