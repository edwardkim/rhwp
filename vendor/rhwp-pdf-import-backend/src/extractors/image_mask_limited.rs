//! Explicit 1-bit masks for request-bounded, independent IR pictures.
//! PDF Reference 1.7 §4.8.5, pp.350–351: sample 0 paints the base image under
//! Decode [0 1]; [1 0] reverses it. Both images map to the same unit square.

use super::{
    limited_decode_ranges, resolve_image_object, sample_buffer, ColorSpace, Error, ImageData,
    PdfImage, PixelFormat, Result,
};
use crate::{document::PdfDocument, import_limits::ImportBudget, object::Object};

pub(super) struct Mask {
    width: u32,
    height: u32,
    stride: usize,
    reversed: bool,
    bits: Vec<u8>,
}

impl Mask {
    pub(super) fn read(
        doc: &PdfDocument,
        object: Option<&Object>,
        budget: &ImportBudget,
    ) -> Result<Option<Self>> {
        let Some(object) = object else {
            return Ok(None);
        };
        let object = resolve_image_object(Some(doc), object)?;
        if matches!(*object, Object::Null) {
            return Ok(None);
        }
        let Object::Stream { dict, .. } = &*object else {
            return Err(Error::Unsupported("limited image import colour-key Mask".into()));
        };
        let entry = |key: &str| {
            dict.get(key)
                .map(|v| resolve_image_object(Some(doc), v))
                .transpose()
        };
        if entry("ImageMask")?.as_deref().and_then(Object::as_bool) != Some(true)
            || !matches!(
                entry("BitsPerComponent")?.as_deref(),
                None | Some(Object::Null | Object::Integer(1))
            )
        {
            return Err(Error::Image("explicit Mask must be a 1-bit image mask".into()));
        }
        for key in ["Mask", "SMask"] {
            if entry(key)?.is_some_and(|v| !matches!(*v, Object::Null)) {
                return Err(Error::Unsupported("nested explicit image mask".into()));
            }
        }
        if entry("Interpolate")?
            .is_some_and(|v| !matches!(*v, Object::Null | Object::Boolean(false)))
        {
            return Err(Error::Unsupported("interpolated explicit image mask".into()));
        }
        let dimension = |key: &str| -> Result<u32> {
            entry(key)?
                .as_deref()
                .and_then(Object::as_integer)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|&n| n != 0)
                .ok_or_else(|| Error::Image(format!("invalid explicit Mask /{key}")))
        };
        let width = dimension("Width")?;
        let height = dimension("Height")?;
        let stride = (width as usize).div_ceil(8);
        let expected = stride
            .checked_mul(height as usize)
            .filter(|&n| n <= budget.limits.stream_bytes)
            .ok_or_else(|| budget.reject("explicit mask sample bytes"))?;
        // Bound expanded sampling work as well as packed bytes, including a
        // mask whose resolution differs from the base image.
        if u64::from(width) * u64::from(height) > 16_000_000 {
            return Err(budget.reject("explicit mask pixels"));
        }
        let decode = limited_decode_ranges(Some(doc), dict.get("Decode"), 1, budget)?;
        let reversed = match decode.as_deref() {
            None | Some([(0.0, 1.0)]) => false,
            Some([(1.0, 0.0)]) => true,
            _ => return Err(Error::Image("invalid explicit Mask Decode".into())),
        };
        // Only lossless packed samples implemented here. In particular a
        // pass-through JPEG/JBIG2/CCITT buffer is never mistaken for mask bits.
        match dict.get("Filter") {
            None | Some(Object::Null) => {},
            Some(Object::Name(name)) if name == "FlateDecode" => {},
            Some(Object::Array(names))
                if names.len() == 1 && names[0].as_name() == Some("FlateDecode") => {},
            _ => return Err(Error::Unsupported("limited explicit Mask filter".into())),
        }
        if entry("DecodeParms")?.is_some_and(|v| !matches!(*v, Object::Null)) {
            return Err(Error::Unsupported("limited explicit Mask DecodeParms".into()));
        }
        let bits = object.decode_stream_data_with_budget(budget)?;
        if bits.len() != expected {
            return Err(Error::Image("explicit Mask sample length differs from dimensions".into()));
        }
        Ok(Some(Self {
            width,
            height,
            stride,
            reversed,
            bits,
        }))
    }

    pub(super) fn apply(self, image: &mut PdfImage, budget: &ImportBudget) -> Result<()> {
        let count = (image.width as usize)
            .checked_mul(image.height as usize)
            .filter(|&n| n > 0)
            .ok_or_else(|| budget.reject("masked image geometry"))?;
        let size = count
            .checked_mul(4)
            .ok_or_else(|| budget.reject("masked RGBA size"))?;
        let rgb = match &image.data {
            ImageData::Jpeg(_) => PdfImage::into_rgb_with_budget(
                image.decode_jpeg_with_budget(budget)?,
                Some(budget),
            )?,
            ImageData::Raw { pixels, format } if image.bits_per_component == 8 => {
                image.raw_rgb_with_budget(pixels, *format, budget)?
            },
            _ if image.ccitt_params.is_some() => PdfImage::into_rgb_with_budget(
                image.decode_ccitt_with_budget(budget)?,
                Some(budget),
            )?,
            _ => return Err(Error::Unsupported("limited masked base image sample depth".into())),
        };
        let mut rgba = sample_buffer(size, Some(budget))?;
        for y in 0..image.height {
            budget.check()?;
            // Nearest samples at pixel centres in the common image unit square.
            let my = ((2 * u64::from(y) + 1) * u64::from(self.height)
                / (2 * u64::from(image.height))) as usize;
            for x in 0..image.width {
                let mx = ((2 * u64::from(x) + 1) * u64::from(self.width)
                    / (2 * u64::from(image.width))) as usize;
                let bit = (self.bits[my * self.stride + mx / 8] >> (7 - mx % 8)) & 1;
                let alpha = if (bit == 0) != self.reversed { 255 } else { 0 };
                let pixel = (y as usize * image.width as usize + x as usize) * 3;
                rgba.extend_from_slice(&rgb.as_raw()[pixel..pixel + 3]);
                rgba.push(alpha);
            }
        }
        image.data = ImageData::Raw {
            pixels: rgba,
            format: PixelFormat::RGBA,
        };
        image.color_space = ColorSpace::DeviceRGB;
        image.bits_per_component = 8;
        image.ccitt_params = None;
        image.icc_profile = None;
        image.samples_are_raw = false;
        Ok(())
    }
}
