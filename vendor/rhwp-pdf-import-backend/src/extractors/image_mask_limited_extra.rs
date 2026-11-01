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
    if let Some(samples) = codec_samples(doc, object, dict, expected, budget)? {
        return Ok(samples);
    }
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
        .filter(|&n| n <= 64_000_000)
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
        } = image.samples_mut(budget)?
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

/// Encoded image bytes must be decoded before source-sample mask operations.
/// ISO 32000-1 7.4.8/7.4.9: JBIG2 is bilevel; JPX samples precede PDF Decode.
fn codec_samples(
    doc: &PdfDocument,
    object: &Object,
    dict: &HashMap<String, Object>,
    expected: usize,
    budget: &ImportBudget,
) -> Result<Option<Vec<u8>>> {
    let Some(filter) = dict.get("Filter") else {
        return Ok(None);
    };
    let filter = resolve(doc, filter, budget)?;
    let last = match &*filter {
        Object::Array(filters) => filters.last(),
        other => Some(other),
    };
    let last = last.map(|f| resolve(doc, f, budget)).transpose()?;
    let name = last.as_deref().and_then(Object::as_name);
    if !matches!(name, Some("JBIG2Decode" | "JPXDecode" | "DCTDecode")) {
        return Ok(None);
    }
    let width = dimension(doc, dict, "Width", budget)? as u32;
    let height = dimension(doc, dict, "Height", budget)? as u32;
    if name == Some("DCTDecode") {
        let bpc = dict
            .get("BitsPerComponent")
            .map(|v| resolve(doc, v, budget))
            .transpose()?;
        if bpc.as_deref().and_then(Object::as_integer) != Some(8) {
            return Err(Error::Unsupported("JPEG source-mask bit depth".into()));
        }
        // The stream decoder removes wrapping byte filters and decrypts the
        // stream; DCT remains encoded until the allocation-budgeted decoder.
        let bytes = doc.decode_stream_with_encryption(object, ObjectRef::new(0, 0))?;
        let decoded = super::super::jpeg_limited::decode(&bytes, width, height, budget)?;
        let pixels = match decoded {
            image::DynamicImage::ImageLuma8(p) => p.into_raw(),
            image::DynamicImage::ImageRgb8(p) => p.into_raw(),
            _ => return Err(Error::Unsupported("JPEG mask source components".into())),
        };
        if pixels.len() != expected {
            return Err(Error::Image("JPEG source samples differ from PDF declaration".into()));
        }
        return Ok(Some(pixels));
    }
    #[cfg(feature = "jbig2")]
    if name == Some("JBIG2Decode") {
        let bpc = dict
            .get("BitsPerComponent")
            .map(|v| resolve(doc, v, budget))
            .transpose()?;
        if bpc
            .as_deref()
            .and_then(Object::as_integer)
            .is_some_and(|n| n != 1)
        {
            return Err(Error::Image("JBIG2 source samples require one bit".into()));
        }
        let packed = super::decode_jbig2_stencil(object, None, dict, Some(doc), width, height)?;
        if packed.len() != expected {
            return Err(Error::Image("JBIG2 packed mask length".into()));
        }
        return Ok(Some(packed));
    }
    #[cfg(feature = "jpeg2000")]
    if name == Some("JPXDecode") {
        return crate::codec_resource::decode(doc.import_budget(), || {
            use rhwp_codec_resource as resource;
            let bpc = dict
                .get("BitsPerComponent")
                .map(|v| resolve(doc, v, budget))
                .transpose()?;
            let bpc = bpc.as_deref().and_then(Object::as_integer).unwrap_or(8);
            if !matches!(bpc, 1 | 2 | 4 | 8) {
                return Err(Error::Unsupported("JPX source-mask bit depth".into()));
            }
            let colors = dict
                .get("ColorSpace")
                .map(|v| resolve(doc, v, budget))
                .transpose()?;
            let indexed = colors
                .as_deref()
                .and_then(Object::as_array)
                .and_then(|v| v.first())
                .and_then(Object::as_name)
                == Some("Indexed");
            let bytes = doc.decode_stream_with_encryption(object, ObjectRef::new(0, 0))?;
            let image = if indexed {
                crate::decoders::jpx::decode_jpx_for_pdf(
                    &bytes,
                    None,
                    false,
                    Some((width, height)),
                    Some(1),
                )?
            } else {
                crate::decoders::jpx::decode_jpx_for_pdf(
                    &bytes,
                    None,
                    true,
                    Some((width, height)),
                    if bpc == 8 {
                        u8::try_from(expected / (width as usize * height as usize)).ok()
                    } else {
                        None
                    },
                )?
            };
            if image.width != width || image.height != height {
                return Err(Error::Image("JPX mask dimensions differ from PDF".into()));
            }
            if image.source_bit_depth != Some(bpc as u8) {
                return Err(Error::Unsupported(
                    "JPX source-mask depth differs from packed PDF sample contract".into(),
                ));
            }
            let components = image.num_components as usize;
            let row_samples = resource::product(&[width as usize, components])
                .map_err(|_| budget.reject("JPX mask row"))?;
            let row_bytes = resource::product(&[row_samples, bpc as usize])
                .map_err(|_| budget.reject("JPX mask row bits"))?
                .div_ceil(8);
            if resource::product(&[row_bytes, height as usize])
                .map_err(|_| budget.reject("JPX mask size"))?
                != expected
            {
                return Err(Error::Image(
                    "JPX mask sample count differs from PDF colour space".into(),
                ));
            }
            let mut packed = resource::filled(0u8, expected)
                .map_err(|_| budget.reject("JPX mask allocation"))?;
            resource::work(image.samples.len()).map_err(|_| budget.reject("JPX mask work"))?;
            let max = (1u16 << bpc) - 1;
            for (y, row) in image.samples.chunks_exact(row_samples).enumerate() {
                for (x, &sample) in row.iter().enumerate() {
                    let value = if indexed {
                        u16::from(sample)
                    } else {
                        (u16::from(sample) * max + 127) / 255
                    };
                    if value > max {
                        return Err(Error::Image("JPX palette index exceeds PDF bit depth".into()));
                    }
                    let offset = x * bpc as usize;
                    packed[y * row_bytes + offset / 8] |=
                        (value as u8) << (8 - bpc as usize - offset % 8);
                }
            }
            Ok(Some(packed))
        });
    }
    Err(Error::UnsupportedFilter(format!(
        "{} disabled at build time",
        name.unwrap_or("image codec")
    )))
}
