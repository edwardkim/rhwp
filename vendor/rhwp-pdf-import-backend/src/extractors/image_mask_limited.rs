//! Request-bounded image masks. PDF Reference 1.7 §§4.8.5, 7.5.4:
//! explicit stencils have inverse polarity to DeviceGray soft-mask alpha.
//! Matte preblending is undone in the source colour space, before conversion.

use super::{
    limited_decode_ranges, sample_buffer, ColorSpace, Error, ImageData, PdfImage, PixelFormat,
    Result,
};
use crate::{document::PdfDocument, import_limits::ImportBudget, object::Object};
use std::borrow::Cow;
use std::collections::HashMap;

// Resolve chains with an explicit request depth bound as well as the loader's
// cumulative visit accounting. A cycle cannot start an unbounded recovery loop.
pub(super) fn resolve<'a>(
    doc: &PdfDocument,
    object: &'a Object,
    budget: &ImportBudget,
) -> Result<Cow<'a, Object>> {
    let mut object = Cow::Borrowed(object);
    let mut depth = 0;
    while let Some(reference) = object.as_reference() {
        budget.check()?;
        if depth >= budget.limits.reference_depth {
            return Err(budget.reject("image mask reference depth"));
        }
        object = Cow::Owned(doc.load_object(reference)?);
        depth += 1;
    }
    Ok(object)
}

pub(super) enum Mask {
    Stream(StreamMask),
    ColourKey(super::image_mask_limited_extra::ColourKey),
}

impl Mask {
    pub(super) fn interpolation(&self) -> Option<bool> {
        match self {
            Self::Stream(mask) => Some(mask.interpolate),
            Self::ColourKey(_) => None,
        }
    }
    pub(super) fn read(
        doc: &PdfDocument,
        object: &Object,
        color_spaces: Option<&HashMap<String, Object>>,
        budget: &ImportBudget,
    ) -> Result<Option<Self>> {
        let base = object
            .as_dict()
            .ok_or_else(|| Error::Image("image must be a stream".into()))?;
        let soft = base
            .get("SMask")
            .map(|o| resolve(doc, o, budget))
            .transpose()?;
        if soft.as_deref().is_none_or(|o| matches!(o, Object::Null)) {
            if let Some(value) = base.get("Mask") {
                let value = resolve(doc, value, budget)?;
                if let Object::Array(ranges) = &*value {
                    return super::image_mask_limited_extra::ColourKey::read(
                        doc,
                        object,
                        ranges,
                        color_spaces,
                        budget,
                    )
                    .map(|m| Some(Self::ColourKey(m)));
                }
            }
        }
        StreamMask::read(doc, base, color_spaces, budget).map(|m| m.map(Self::Stream))
    }

    pub(super) fn apply(self, image: &mut PdfImage, budget: &ImportBudget) -> Result<()> {
        match self {
            Self::Stream(mask) => mask.apply(image, budget),
            Self::ColourKey(mask) => mask.apply(image, budget),
        }
    }
}

pub(super) struct StreamMask {
    width: u32,
    height: u32,
    stride: usize,
    bpc: usize,
    decode: (f32, f32),
    soft: bool,
    interpolate: bool,
    samples: Vec<u8>,
    matte: Option<[f32; 3]>,
    jpeg_decode: Option<[(f32, f32); 3]>,
    other_source_decode: bool,
}

impl StreamMask {
    pub(super) fn read(
        doc: &PdfDocument,
        base: &HashMap<String, Object>,
        color_spaces: Option<&HashMap<String, Object>>,
        budget: &ImportBudget,
    ) -> Result<Option<Self>> {
        // Table 4.39: a non-null SMask overrides Mask; never multiply their
        // alphas or reject an ignored colour-key mask instead of using SMask.
        let soft = base
            .get("SMask")
            .map(|v| resolve(doc, v, budget))
            .transpose()?;
        let (object, from_smask) = match soft {
            Some(object) if !matches!(*object, Object::Null) => (object, true),
            _ => match base.get("Mask") {
                Some(object) => (resolve(doc, object, budget)?, false),
                None => return Ok(None),
            },
        };
        if matches!(*object, Object::Null) {
            return Ok(None);
        }
        let Object::Stream { dict, .. } = &*object else {
            return Err(if !from_smask && matches!(*object, Object::Array(_)) {
                Error::Unsupported("limited image import colour-key Mask".into())
            } else {
                Error::Image("image mask must be a stream".into())
            });
        };
        let entry = |key: &str| dict.get(key).map(|v| resolve(doc, v, budget)).transpose();
        let stencil = entry("ImageMask")?;
        // Some writers put an ordinary grayscale image under Mask. PDFium's
        // StartLoadMaskDIB reads that image as alpha, not as an inverse stencil.
        // Keep true ImageMask polarity and malformed flags on the strict path.
        let gray_mask = !from_smask
            && stencil
                .as_deref()
                .is_none_or(|v| matches!(v, Object::Null | Object::Boolean(false)))
            && entry("ColorSpace")?.as_deref().and_then(Object::as_name) == Some("DeviceGray");
        let soft = from_smask || gray_mask;
        let bpc = entry("BitsPerComponent")?;
        let bpc = if soft {
            if entry("Subtype")?.as_deref().and_then(Object::as_name) != Some("Image")
                || entry("Type")?
                    .is_some_and(|v| !matches!(*v, Object::Null) && v.as_name() != Some("XObject"))
                || stencil.is_some_and(|v| !matches!(*v, Object::Null | Object::Boolean(false)))
                || entry("ColorSpace")?.as_deref().and_then(Object::as_name) != Some("DeviceGray")
            {
                return Err(Error::Image(
                    "SMask requires a DeviceGray image, not a stencil".into(),
                ));
            }
            match bpc.as_deref().and_then(Object::as_integer) {
                Some(n @ (1 | 2 | 4 | 8)) => n as usize,
                Some(16) => return Err(Error::Unsupported("limited 16-bit SMask".into())),
                _ => return Err(Error::Image("invalid SMask BitsPerComponent".into())),
            }
        } else {
            if stencil.as_deref().and_then(Object::as_bool) != Some(true)
                || !matches!(bpc.as_deref(), None | Some(Object::Null | Object::Integer(1)))
            {
                return Err(Error::Image("explicit Mask must be a 1-bit image mask".into()));
            }
            1
        };
        for key in ["Mask", "SMask"] {
            if entry(key)?.is_some_and(|v| !matches!(*v, Object::Null)) {
                return Err(Error::Unsupported("nested image mask".into()));
            }
        }
        let interpolate = match entry("Interpolate")?.as_deref() {
            None | Some(Object::Null | Object::Boolean(false)) => false,
            Some(Object::Boolean(true)) => true,
            _ => return Err(Error::Image("invalid mask Interpolate flag".into())),
        };
        let dimension = |key: &str| -> Result<u32> {
            entry(key)?
                .as_deref()
                .and_then(Object::as_integer)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|&n| n != 0)
                .ok_or_else(|| Error::Image(format!("invalid image mask /{key}")))
        };
        let width = dimension("Width")?;
        let height = dimension("Height")?;
        let stride = (width as usize)
            .checked_mul(bpc)
            .map(|n| n.div_ceil(8))
            .ok_or_else(|| budget.reject("mask row size"))?;
        let expected = stride
            .checked_mul(height as usize)
            .filter(|&n| n <= budget.limits.stream_bytes)
            .ok_or_else(|| budget.reject("mask sample bytes"))?;
        if u64::from(width) * u64::from(height) > 64_000_000 {
            return Err(budget.reject("mask pixels"));
        }
        let ranges = limited_decode_ranges(Some(doc), dict.get("Decode"), 1, budget)?;
        let decode = ranges.as_deref().map_or((0.0, 1.0), |v| v[0]);
        if !soft && !matches!(decode, (0.0, 1.0) | (1.0, 0.0)) {
            return Err(Error::Image("invalid explicit Mask Decode".into()));
        }
        let matte = match entry("Matte")? {
            None => None,
            Some(v) if matches!(*v, Object::Null) => None,
            // PDFium only unblends Matte for SMask. A grayscale Mask uses its
            // decoded samples directly; it must not alter the base RGB values.
            Some(_) if gray_mask => None,
            Some(v) if soft => {
                // Do not undo preblending after an ICC/CMYK conversion or an
                // Indexed palette lookup: those are not source RGB samples.
                let cs = base
                    .get("ColorSpace")
                    .ok_or_else(|| Error::Image("missing base ColorSpace".into()))?;
                let cs = resolve(doc, cs, budget)?;
                let cs = match cs
                    .as_name()
                    .and_then(|name| color_spaces.and_then(|map| map.get(name)))
                {
                    Some(resource) => resolve(doc, resource, budget)?,
                    None => cs,
                };
                let components = match cs.as_name() {
                    Some("DeviceRGB") => 3,
                    Some("DeviceGray") => 1,
                    _ => {
                        return Err(Error::Unsupported("limited Matte source colour space".into()))
                    },
                };
                let values = v
                    .as_array()
                    .filter(|a| a.len() == components)
                    .ok_or_else(|| {
                        Error::Image(
                            "Matte component count differs from source colour space".into(),
                        )
                    })?;
                let mut matte = [0.0; 3];
                for (i, value) in values.iter().enumerate() {
                    let value = resolve(doc, value, budget)?;
                    matte[i] = value
                        .as_integer()
                        .map(|n| n as f64)
                        .or_else(|| value.as_real())
                        .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
                        .ok_or_else(|| Error::Image("invalid Matte colour component".into()))?
                        as f32;
                }
                if components == 1 {
                    matte = [matte[0]; 3];
                }
                Some(matte)
            },
            Some(_) => return Err(Error::Image("Matte requires SMask".into())),
        };
        // DecodeParms belongs to the filter pipeline, including Flate/LZW
        // predictors. The bounded stream decoder validates and applies it;
        // absent Predictor means 1, so other predictor entries are inert.
        // The raw extractor folds Decode into samples; its JPEG pass-through
        // intentionally does not. Carry it to the JPEG composition boundary.
        let jpeg_decode = if base
            .get("Decode")
            .is_some_and(|v| !matches!(v, Object::Null))
        {
            let cs = base
                .get("ColorSpace")
                .ok_or_else(|| Error::Image("missing base ColorSpace".into()))?;
            let cs = resolve(doc, cs, budget)?;
            let cs = match cs
                .as_name()
                .and_then(|name| color_spaces.and_then(|map| map.get(name)))
            {
                Some(resource) => resolve(doc, resource, budget)?,
                None => cs,
            };
            match cs.as_name() {
                Some("DeviceRGB" | "DeviceGray") => {
                    let n = if cs.as_name() == Some("DeviceGray") {
                        1
                    } else {
                        3
                    };
                    let ranges = limited_decode_ranges(Some(doc), base.get("Decode"), n, budget)?;
                    ranges.map(|r| {
                        if n == 1 {
                            [r[0]; 3]
                        } else {
                            [r[0], r[1], r[2]]
                        }
                    })
                },
                _ => None, // Non-JPEG paths already apply their own Decode.
            }
        } else {
            None
        };
        let other_source_decode = base
            .get("Decode")
            .is_some_and(|v| !matches!(v, Object::Null))
            && jpeg_decode.is_none();
        // Byte filters and opted-in codec adapters supply packed source rows.
        // Encoded image bytes must never reach the raw sample reader.
        let samples = super::image_mask_limited_extra::samples(doc, &object, expected, budget)?;
        Ok(Some(Self {
            width,
            height,
            stride,
            bpc,
            decode,
            soft,
            interpolate,
            samples,
            matte,
            jpeg_decode,
            other_source_decode,
        }))
    }

    fn alpha(&self, x: usize, y: usize, max: u8) -> f64 {
        let offset = x * self.bpc;
        let raw = (self.samples[y * self.stride + offset / 8] >> (8 - self.bpc - offset % 8)) & max;
        let value = (f64::from(self.decode.0)
            + f64::from(raw) * (f64::from(self.decode.1) - f64::from(self.decode.0))
                / f64::from(max))
        .clamp(0.0, 1.0);
        if self.soft {
            value
        } else {
            1.0 - value
        }
    }

    pub(super) fn apply(self, image: &mut PdfImage, budget: &ImportBudget) -> Result<()> {
        if self.matte.is_some() && (self.width != image.width || self.height != image.height) {
            return Err(Error::Image("Matte requires matching base and SMask dimensions".into()));
        }
        let width = image.width.max(self.width);
        let height = image.height.max(self.height);
        let count = (width as usize)
            .checked_mul(height as usize)
            .filter(|&n| n > 0 && n <= 64_000_000)
            .ok_or_else(|| budget.reject("masked image geometry"))?;
        let size = count
            .checked_mul(4)
            .ok_or_else(|| budget.reject("masked RGBA size"))?;
        let jpeg = matches!(image.data.as_ref(), ImageData::Jpeg(_));
        if jpeg && self.other_source_decode {
            return Err(Error::Unsupported("limited masked JPEG Decode colour space".into()));
        }
        let rgb: Cow<'_, [u8]> = match image.data.as_ref() {
            ImageData::Raw {
                pixels,
                format: PixelFormat::RGB,
            } if image.bits_per_component == 8 => {
                let expected = (image.width as usize)
                    .checked_mul(image.height as usize)
                    .and_then(|n| n.checked_mul(3));
                if expected != Some(pixels.len()) {
                    return Err(Error::Image("invalid masked RGB sample length".into()));
                }
                Cow::Borrowed(pixels)
            },
            ImageData::Jpeg(_) => Cow::Owned(
                PdfImage::into_rgb_with_budget(
                    image.decode_jpeg_with_budget(budget)?,
                    Some(budget),
                )?
                .into_raw(),
            ),
            ImageData::Raw { pixels, format } if image.bits_per_component == 8 => Cow::Owned(
                image
                    .raw_rgb_with_budget(pixels, *format, budget)?
                    .into_raw(),
            ),
            _ if image.ccitt_params.is_some() => Cow::Owned(
                PdfImage::into_rgb_with_budget(
                    image.decode_ccitt_with_budget(budget)?,
                    Some(budget),
                )?
                .into_raw(),
            ),
            _ => return Err(Error::Unsupported("limited masked base image sample depth".into())),
        };
        let mut rgba = sample_buffer(size, Some(budget))?;
        let max = ((1 << self.bpc) - 1) as u8;
        for y in 0..height {
            budget.check()?;
            // Pixel centres in the common image unit square; no change to CTM,
            // clipping, bounding boxes or final image ownership/placement.
            let my = ((2 * u64::from(y) + 1) * u64::from(self.height) / (2 * u64::from(height)))
                as usize;
            let by = ((2 * u64::from(y) + 1) * u64::from(image.height) / (2 * u64::from(height)))
                as usize;
            for x in 0..width {
                let mx = ((2 * u64::from(x) + 1) * u64::from(self.width) / (2 * u64::from(width)))
                    as usize;
                let bx = ((2 * u64::from(x) + 1) * u64::from(image.width) / (2 * u64::from(width)))
                    as usize;
                let alpha = if self.interpolate {
                    // ISO 32000-1 8.9.5.3 allows the reader's interpolation
                    // algorithm. Bilinear sample-centre mapping, with edge
                    // extension, needs no additional image-sized allocation.
                    let fx = ((f64::from(x) + 0.5) * f64::from(self.width) / f64::from(width)
                        - 0.5)
                        .clamp(0.0, f64::from(self.width - 1));
                    let fy = ((f64::from(y) + 0.5) * f64::from(self.height) / f64::from(height)
                        - 0.5)
                        .clamp(0.0, f64::from(self.height - 1));
                    let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
                    let (x1, y1) = (
                        (x0 + 1).min(self.width as usize - 1),
                        (y0 + 1).min(self.height as usize - 1),
                    );
                    let (dx, dy) = (fx - x0 as f64, fy - y0 as f64);
                    let upper = self.alpha(x0, y0, max) * (1.0 - dx) + self.alpha(x1, y0, max) * dx;
                    let lower = self.alpha(x0, y1, max) * (1.0 - dx) + self.alpha(x1, y1, max) * dx;
                    upper * (1.0 - dy) + lower * dy
                } else {
                    self.alpha(mx, my, max)
                };
                let pixel = (by * image.width as usize + bx) * 3;
                for c in 0..3 {
                    let mut value = f64::from(rgb[pixel + c]);
                    if let Some(ranges) = self.jpeg_decode.as_ref().filter(|_| jpeg) {
                        value = (f64::from(ranges[c].0)
                            + value / 255.0 * (f64::from(ranges[c].1) - f64::from(ranges[c].0)))
                        .clamp(0.0, 1.0)
                            * 255.0;
                    }
                    let value = match self.matte {
                        Some(m) if alpha > 0.0 => {
                            f64::from(m[c]) * 255.0 + (value - f64::from(m[c]) * 255.0) / alpha
                        },
                        Some(_) => 0.0, // RGB is irrelevant when fully transparent.
                        None => value,
                    };
                    rgba.push(value.round().clamp(0.0, 255.0) as u8);
                }
                rgba.push((alpha * 255.0).round() as u8);
            }
        }
        image.data = std::sync::Arc::new(ImageData::Raw {
            pixels: rgba,
            format: PixelFormat::RGBA,
        });
        image.width = width;
        image.height = height;
        image.color_space = ColorSpace::DeviceRGB;
        image.bits_per_component = 8;
        image.ccitt_params = None;
        image.icc_profile = None;
        image.samples_are_raw = false;
        Ok(())
    }
}
