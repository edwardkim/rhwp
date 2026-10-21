//! Source-sample colour keys and current-colour stencils for bounded import.
//! PDF Reference 1.7 4.8.5: Mask ranges precede Decode and colour conversion.
use super::{
    image_mask_limited::resolve, sample_buffer, ColorSpace, Error, ImageData, PdfImage,
    PixelFormat, Result,
};
use crate::{
    document::PdfDocument,
    import_limits::ImportBudget,
    object::{Object, ObjectRef},
};
use std::collections::HashMap;

fn dimension(
    doc: &PdfDocument,
    dict: &HashMap<String, Object>,
    key: &str,
    budget: &ImportBudget,
) -> Result<usize> {
    let v = dict
        .get(key)
        .ok_or_else(|| Error::Image(format!("missing image /{key}")))?;
    let v = resolve(doc, v, budget)?;
    v.as_integer()
        .and_then(|n| usize::try_from(n).ok())
        .filter(|&n| n > 0 && n <= u32::MAX as usize)
        .ok_or_else(|| Error::Image(format!("invalid image /{key}")))
}

/// Only byte-stream filters produce packed PDF samples here. Image codecs
/// require their own audited source-plane adapter; encoded bytes are not pixels.
pub(super) fn samples(
    doc: &PdfDocument,
    object: &Object,
    expected: usize,
    budget: &ImportBudget,
) -> Result<Vec<u8>> {
    if expected > budget.limits.stream_bytes {
        return Err(budget.reject("mask sample bytes"));
    }
    let dict = object
        .as_dict()
        .ok_or_else(|| Error::Image("image must be a stream".into()))?;
    if let Some(filter) = dict.get("Filter") {
        let filter = resolve(doc, filter, budget)?;
        let allowed = |o: &Object| {
            matches!(o, Object::Null)
                || o.as_name().is_some_and(|n| {
                    matches!(
                        n,
                        "FlateDecode"
                            | "ASCIIHexDecode"
                            | "ASCII85Decode"
                            | "LZWDecode"
                            | "RunLengthDecode"
                    )
                })
        };
        let valid = match &*filter {
            Object::Array(names) => names.iter().all(allowed),
            o => allowed(o),
        };
        if !valid {
            return Err(Error::Unsupported("bounded source-sample mask codec".into()));
        }
    }
    let bytes = doc.decode_stream_with_encryption(object, ObjectRef::new(0, 0))?;
    if bytes.len() != expected {
        return Err(Error::Image("image sample length differs from packed rows".into()));
    }
    Ok(bytes)
}

fn layout(
    width: usize,
    height: usize,
    components: usize,
    bpc: usize,
    budget: &ImportBudget,
) -> Result<(usize, usize)> {
    let count = width
        .checked_mul(height)
        .filter(|&n| n <= 16_000_000)
        .ok_or_else(|| budget.reject("mask pixels"))?;
    let row = width
        .checked_mul(components)
        .and_then(|n| n.checked_mul(bpc))
        .map(|n| n.div_ceil(8))
        .ok_or_else(|| budget.reject("mask packed row"))?;
    let bytes = row
        .checked_mul(height)
        .filter(|&n| n <= budget.limits.stream_bytes)
        .ok_or_else(|| budget.reject("mask packed samples"))?;
    Ok((count, bytes))
}

pub(super) struct ColourKey {
    width: usize,
    height: usize,
    alpha: Vec<u8>,
}

impl ColourKey {
    pub(super) fn read(
        doc: &PdfDocument,
        object: &Object,
        ranges: &[Object],
        spaces: Option<&HashMap<String, Object>>,
        budget: &ImportBudget,
    ) -> Result<Self> {
        let dict = object
            .as_dict()
            .ok_or_else(|| Error::Image("image must be a stream".into()))?;
        let cs = dict
            .get("ColorSpace")
            .ok_or_else(|| Error::Image("missing colour-key ColorSpace".into()))?;
        let cs = resolve(doc, cs, budget)?;
        let cs = match cs.as_name().and_then(|n| spaces.and_then(|s| s.get(n))) {
            Some(v) => resolve(doc, v, budget)?,
            None => cs,
        };
        let components = match &*cs {
            Object::Name(n) if n == "DeviceGray" => 1,
            Object::Name(n) if n == "DeviceRGB" => 3,
            Object::Name(n) if n == "DeviceCMYK" => 4,
            Object::Array(a) if a.first().and_then(Object::as_name) == Some("Indexed") => 1,
            _ => return Err(Error::Unsupported("bounded colour-key source colour space".into())),
        };
        let depth = dict
            .get("BitsPerComponent")
            .ok_or_else(|| Error::Image("missing colour-key depth".into()))?;
        let depth = resolve(doc, depth, budget)?;
        let bpc = match depth.as_integer() {
            Some(n @ (1 | 2 | 4 | 8 | 16)) => n as usize,
            _ => return Err(Error::Image("invalid colour-key BitsPerComponent".into())),
        };
        if ranges.len() != components * 2 {
            return Err(Error::Image(
                "colour-key range count differs from source components".into(),
            ));
        }
        let maximum = (1u32 << bpc) - 1;
        let mut pairs = [(0u32, 0u32); 4];
        for (i, pair) in ranges.chunks_exact(2).enumerate() {
            let number = |o: &Object| -> Result<u32> {
                resolve(doc, o, budget)?
                    .as_integer()
                    .and_then(|n| u32::try_from(n).ok())
                    .filter(|&n| n <= maximum)
                    .ok_or_else(|| {
                        Error::Image("colour-key bound outside source sample range".into())
                    })
            };
            pairs[i] = (number(&pair[0])?, number(&pair[1])?);
            if pairs[i].0 > pairs[i].1 {
                return Err(Error::Image("reversed colour-key range".into()));
            }
        }
        let width = dimension(doc, dict, "Width", budget)?;
        let height = dimension(doc, dict, "Height", budget)?;
        let (count, expected) = layout(width, height, components, bpc, budget)?;
        let raw = samples(doc, object, expected, budget)?;
        let stride = expected / height;
        let mut alpha = sample_buffer(count, Some(budget))?;
        for i in 0..count {
            if i % 4096 == 0 {
                budget.check()?;
            }
            let row = (i / width) * stride;
            let pixel = (i % width) * components;
            let masked = (0..components).all(|c| {
                let bit = (pixel + c) * bpc;
                let byte = row + bit / 8;
                let sample = match bpc {
                    16 => u32::from(u16::from_be_bytes([raw[byte], raw[byte + 1]])),
                    8 => u32::from(raw[byte]),
                    _ => (u32::from(raw[byte]) >> (8 - bpc - bit % 8)) & maximum,
                };
                pairs[c].0 <= sample && sample <= pairs[c].1
            });
            alpha.push(if masked { 0 } else { 255 });
        }
        Ok(Self {
            width,
            height,
            alpha,
        })
    }

    pub(super) fn apply(self, image: &mut PdfImage, budget: &ImportBudget) -> Result<()> {
        if image.width as usize != self.width || image.height as usize != self.height {
            return Err(Error::Image("colour-key decoded geometry changed".into()));
        }
        image.apply_opacity(1., budget)?;
        let ImageData::Raw {
            pixels,
            format: PixelFormat::RGBA,
        } = &mut image.data
        else {
            unreachable!("opacity produces RGBA")
        };
        for (i, (pixel, alpha)) in pixels.chunks_exact_mut(4).zip(self.alpha).enumerate() {
            if i % 4096 == 0 {
                budget.check()?;
            }
            pixel[3] = ((u16::from(pixel[3]) * u16::from(alpha) + 127) / 255) as u8;
        }
        Ok(())
    }
}

pub(super) fn stencil(
    doc: &PdfDocument,
    object: &Object,
    colour: Option<(f32, f32, f32)>,
    budget: &ImportBudget,
) -> Result<PdfImage> {
    let dict = object
        .as_dict()
        .ok_or_else(|| Error::Image("stencil must be a stream".into()))?;
    for key in ["ColorSpace", "Mask", "SMask"] {
        if dict
            .get(key)
            .map(|o| resolve(doc, o, budget))
            .transpose()?
            .as_deref()
            .is_some_and(|o| !matches!(o, Object::Null))
        {
            return Err(Error::Image(format!("stencil must not contain /{key}")));
        }
    }
    if let Some(depth) = dict.get("BitsPerComponent") {
        if !matches!(&*resolve(doc, depth, budget)?, Object::Null | Object::Integer(1)) {
            return Err(Error::Image("stencil must have one-bit samples".into()));
        }
    }
    let decode = super::limited_decode_ranges(Some(doc), dict.get("Decode"), 1, budget)?;
    let reverse = match decode.as_deref() {
        None | Some([(0., 1.)]) => false,
        Some([(1., 0.)]) => true,
        _ => return Err(Error::Image("invalid stencil Decode".into())),
    };
    let (r, g, b) =
        colour.ok_or_else(|| Error::Unsupported("bounded stencil current colour".into()))?;
    let rgb = [r, g, b].map(|v| (v.clamp(0., 1.) * 255.).round() as u8);
    let width = dimension(doc, dict, "Width", budget)?;
    let height = dimension(doc, dict, "Height", budget)?;
    let (count, expected) = layout(width, height, 1, 1, budget)?;
    let raw = samples(doc, object, expected, budget)?;
    let stride = width.div_ceil(8);
    let mut pixels = sample_buffer(
        count
            .checked_mul(4)
            .ok_or_else(|| budget.reject("stencil RGBA"))?,
        Some(budget),
    )?;
    for i in 0..count {
        if i % 4096 == 0 {
            budget.check()?;
        }
        let bit = raw[(i / width) * stride + (i % width) / 8] >> (7 - (i % width) % 8) & 1;
        pixels.extend_from_slice(&rgb);
        pixels.push(if (bit == 0) != reverse { 255 } else { 0 });
    }
    Ok(PdfImage::new(
        width as u32,
        height as u32,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels,
            format: PixelFormat::RGBA,
        },
    ))
}
