//! JPEG 2000 (`/JPXDecode`) image decoding via hayro-jpeg2000.
//!
//! ISO 32000-1 §7.4.9: a `/JPXDecode` stream is a JPEG 2000 codestream — either a
//! raw J2K codestream or a JP2-boxed file. hayro-jpeg2000 handles both. This decodes
//! the codestream to interleaved 8-bit-per-component samples; the caller maps the
//! component count to a colour space and applies `/Decode`, `/SMask`, etc.
//!
//! Feature-gated (`jpeg2000`): when the feature is off the call site returns the
//! existing `UnsupportedFilter` error rather than panicking.

#[cfg(feature = "jpeg2000")]
use crate::error::Error;
use crate::error::Result;

/// Pass-through filter for `/JPXDecode`.
///
/// Like `DCTDecode`/`JBIG2Decode`, the JPEG 2000 codestream is not decompressed
/// by the generic filter pipeline — it is handed to the image extractor, which
/// decodes it with hayro-jpeg2000 (`decode_jpx`). So this decoder returns its input
/// unchanged. It is always available (even without the `jpeg2000` feature) so the
/// pipeline can surface the codestream; the extractor's feature-gated path then
/// either decodes it or returns a typed `UnsupportedFilter` error.
pub struct JpxDecoder;

impl super::StreamDecoder for JpxDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        Ok(input.to_vec())
    }

    fn name(&self) -> &str {
        "JPXDecode"
    }
}

/// A decoded JPEG 2000 image: interleaved 8-bit samples plus component count.
#[cfg(feature = "jpeg2000")]
pub struct JpxImage {
    /// `width * height * num_components` bytes, component-interleaved (row-major).
    pub samples: Vec<u8>,
    pub num_components: u8,
    /// Colour channels from JP2 metadata, excluding an encoded opacity channel.
    pub colour_components: u8,
    /// Uniform source depth, or None for mixed component precisions.
    pub source_bit_depth: Option<u8>,
    /// Dimensions of the samples actually decoded. When `decode_jpx_at` was
    /// given a target resolution these may be smaller than the `/Width` and
    /// `/Height` of the image dictionary, so callers must take the geometry
    /// from here rather than from the dictionary.
    pub width: u32,
    pub height: u32,
}

/// Decode a JP2/J2K codestream to interleaved 8-bit-per-component samples.
///
/// hayro-jpeg2000 yields one f32 plane per component (normalized to the component's
/// bit depth); `DecodedImage::data_u8()` interleaves these to 8-bit samples.
/// Components are assumed to share the image dimensions (no chroma subsampling) —
/// the common case for PDF image XObjects; a subsampled component is rejected with a
/// typed error rather than producing misaligned output.
#[cfg(all(feature = "jpeg2000", test))]
/// Decode a JP2/J2K codestream, optionally at no more than `target` resolution.
///
/// JPEG 2000 is inherently multi-resolution: the codestream stores successive
/// resolution levels, so a smaller image can be produced by decoding fewer of
/// them rather than by decoding everything and shrinking afterwards. When the
/// caller knows the image will be drawn smaller than its stored size — which
/// on a page it usually is — this avoids materialising the full-resolution
/// samples at all.
///
/// That difference is not marginal. One page in the wild carries a
/// 12608 x 16806 JPX image; decoding it at full resolution peaks at 11.3 GB,
/// which no browser tab or phone will survive, for a picture that is painted
/// into a fraction of that.
///
/// `target` is a hint: the decoder picks the smallest resolution level that
/// still covers it, so the result is never smaller than requested and may be
/// larger.
pub fn decode_jpx_at(bytes: &[u8], target: Option<(u32, u32)>) -> Result<JpxImage> {
    decode_jpx_with(bytes, target, true, None, None)
}

/// Decode a JP2/J2K codestream to its palette *indices*, one byte per pixel.
///
/// For an image whose dictionary declares an `/Indexed` colour space, the
/// codestream's single component is an index into the dictionary's lookup
/// table, and §7.4.9 (`docs/spec/pdf.md`:3143) has the dictionary win: "If
/// present, it shall determine how the image samples are interpreted, and
/// the colour space specifications in the JPEG2000 data shall be ignored". A JP2 file may carry a palette box of its own for
/// the same indices; resolving through it yields three colour components
/// where the dictionary expects one, and the first of them — the red
/// channel — then stands in for the whole pixel. That is how a page whose
/// palette is four shades of blue came out `R = G = B`.
///
/// The samples are the raw index values, not rescaled to 8 bits, so they can
/// be looked up in the dictionary's table directly. Indices wider than a byte
/// are refused: an `/Indexed` table is indexed by 0..`hival` and "`hival`
/// shall be no greater than 255" (§8.6.6.3, pdf.md:10992), so a component
/// deeper than 8 bits cannot be one.
#[cfg(all(feature = "jpeg2000", test))]
pub fn decode_jpx_indices_at(bytes: &[u8], target: Option<(u32, u32)>) -> Result<JpxImage> {
    decode_jpx_with(bytes, target, false, None, None)
}

#[cfg(feature = "jpeg2000")]
pub(crate) fn decode_jpx_for_pdf(
    bytes: &[u8],
    target: Option<(u32, u32)>,
    palette: bool,
    expected: Option<(u32, u32)>,
    pdf_colour_components: Option<u8>,
) -> Result<JpxImage> {
    decode_jpx_with(bytes, target, palette, expected, pdf_colour_components)
}

#[cfg(feature = "jpeg2000")]
fn decode_jpx_with(
    bytes: &[u8],
    target: Option<(u32, u32)>,
    resolve_palette_indices: bool,
    expected: Option<(u32, u32)>,
    pdf_colour_components: Option<u8>,
) -> Result<JpxImage> {
    use hayro_jpeg2000::{DecodeSettings, DecoderContext, Image};

    let settings = DecodeSettings {
        target_resolution: target,
        resolve_palette_indices,
        strict: expected.is_some(),
        pdf_colour_components,
    };

    let image = Image::new(bytes, &settings).map_err(|e| {
        Error::UnsupportedFilter(format!("JPXDecode: JPEG 2000 decode failed: {e:?}"))
    })?;

    if image.num_components() > 5 {
        return Err(Error::Unsupported("JPX encoded component count exceeds PDF support".into()));
    }
    let colour_components = image.color_space().num_channels();
    let width = image.width();
    let height = image.height();
    if expected.is_some_and(|dims| dims != (width, height)) {
        return Err(Error::Image("JPX dimensions differ from PDF image".into()));
    }
    let npix = rhwp_codec_resource::product(&[width as usize, height as usize])
        .map_err(|_| Error::Image("JPX dimensions overflow".into()))?;
    if expected.is_some() && npix > 16_000_000 {
        return Err(Error::Image("JPX image exceeds pixel limit".into()));
    }

    let mut ctx = DecoderContext::default();
    let decoded = image.decode(&mut ctx).map_err(|e| {
        Error::UnsupportedFilter(format!("JPXDecode: JPEG 2000 decode failed: {e:?}"))
    })?;

    let comps = decoded.components();
    if comps.is_empty() {
        return Err(Error::UnsupportedFilter(
            "JPXDecode: JPEG 2000 image has no components".to_string(),
        ));
    }
    let source_bit_depth = comps
        .iter()
        .all(|c| c.bit_depth() == comps[0].bit_depth())
        .then(|| comps[0].bit_depth());
    let num_components = comps.len();
    if num_components > 5 {
        return Err(Error::Unsupported(
            "JPX PDF component count exceeds supported colour+alpha".into(),
        ));
    }

    // Palette indices are wanted as they are: the decoder's own interleave
    // would rescale a 4-bit index to the 8-bit range (0..15 → 0..255), and a
    // lookup table cannot be read with that.
    if !resolve_palette_indices {
        if !matches!(comps.len(), 1 | 2)
            || comps[0].bit_depth() > 8
            || comps.iter().any(|c| c.samples().len() != npix)
        {
            return Err(Error::Image(
                "JPX Indexed requires index and optional full-size alpha".into(),
            ));
        }
        let mut samples = rhwp_codec_resource::capacity(
            rhwp_codec_resource::product(&[npix, comps.len()])
                .map_err(|_| Error::Image("JPX index size".into()))?,
        )
        .map_err(|_| Error::Image("JPX index allocation".into()))?;
        rhwp_codec_resource::work(npix).map_err(|_| Error::Image("JPX index work".into()))?;
        for index in 0..npix {
            samples.push(comps[0].samples()[index].round().clamp(0.0, 255.0) as u8);
            if comps.len() == 2 {
                let maximum = ((1u64 << comps[1].bit_depth()) - 1) as f32;
                samples.push(
                    (comps[1].samples()[index] * 255.0 / maximum)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                );
            }
        }
        return Ok(JpxImage {
            samples,
            num_components: comps.len() as u8,
            colour_components: 1,
            source_bit_depth,
            width,
            height,
        });
    }

    // Fast path: every component is full-resolution (the common case) → use the
    // decoder's own interleave.
    if comps.iter().all(|c| c.samples().len() == npix) {
        return Ok(JpxImage {
            samples: decoded
                .try_data_u8()
                .map_err(|e| Error::Image(format!("JPX samples: {e}")))?,
            num_components: num_components as u8,
            colour_components,
            source_bit_depth,
            width,
            height,
        });
    }

    // Chroma-subsampled path (WS1.7). hayro-jpeg2000 0.4 does not expose
    // per-component dimensions, so only the unambiguous 2×2 (4:2:0) case is
    // recovered: a component with ⌈w/2⌉·⌈h/2⌉ samples is nearest-upsampled to
    // full resolution; any other ratio (or non-8-bit depth, where the f32→u8
    // scaling would differ) stays unsupported rather than guessing. Components
    // are then interleaved manually since `data_u8` assumes equal plane sizes.
    let (w, h) = (width as usize, height as usize);
    let (sw, sh) = (width.div_ceil(2) as usize, height.div_ceil(2) as usize);
    let mut planes: Vec<Vec<u8>> = rhwp_codec_resource::capacity(num_components)
        .map_err(|_| Error::Image("JPX planes allocation".into()))?;
    for (ci, comp) in comps.iter().enumerate() {
        if comp.bit_depth() != 8 {
            return Err(Error::UnsupportedFilter(format!(
                "JPXDecode: subsampled component {ci} with {}-bit depth not supported",
                comp.bit_depth()
            )));
        }
        let s = comp.samples();
        let plane = if s.len() == npix {
            rhwp_codec_resource::collect(s.iter().map(|&v| v.round().clamp(0.0, 255.0) as u8))
                .map_err(|_| Error::Image("JPX plane allocation".into()))?
        } else if s.len() == sw * sh {
            upsample_nearest_u8(s, sw, sh, w, h)?
        } else {
            return Err(Error::UnsupportedFilter(format!(
                "JPXDecode: subsampled component {ci} ({} samples) — only 2×2 (4:2:0) \
                 subsampling of a {width}×{height} image is supported",
                s.len()
            )));
        };
        planes.push(plane);
    }

    let count = rhwp_codec_resource::product(&[npix, num_components])
        .map_err(|_| Error::Image("JPX sample size".into()))?;
    let mut samples = rhwp_codec_resource::filled(0u8, count)
        .map_err(|_| Error::Image("JPX samples allocation".into()))?;
    for (ci, plane) in planes.iter().enumerate() {
        for (i, &px) in plane.iter().enumerate() {
            samples[i * num_components + ci] = px;
        }
    }
    Ok(JpxImage {
        samples,
        width,
        height,
        num_components: num_components as u8,
        colour_components,
        source_bit_depth,
    })
}

/// Nearest-neighbour upsample of an `sw×sh` f32 sample plane to `fw×fh` u8.
#[cfg(feature = "jpeg2000")]
fn upsample_nearest_u8(sub: &[f32], sw: usize, sh: usize, fw: usize, fh: usize) -> Result<Vec<u8>> {
    let count = rhwp_codec_resource::product(&[fw, fh])
        .map_err(|_| Error::Image("JPX upsample size".into()))?;
    let mut out = rhwp_codec_resource::filled(0u8, count)
        .map_err(|_| Error::Image("JPX upsample allocation".into()))?;
    for y in 0..fh {
        let sy = (y * sh / fh).min(sh.saturating_sub(1));
        for x in 0..fw {
            let sx = (x * sw / fw).min(sw.saturating_sub(1));
            out[y * fw + x] = sub[sy * sw + sx].round().clamp(0.0, 255.0) as u8;
        }
    }
    Ok(out)
}

#[cfg(all(test, feature = "jpeg2000"))]
mod tests {
    use super::{decode_jpx_at, upsample_nearest_u8};

    /// Original synthetic 816x1056 DeviceGray geometry/resolution control.
    /// The lost upstream scan is not reconstructed or used as a visual baseline.
    const SYNTHETIC_GRAY_JP2: &[u8] = include_bytes!("testdata/gray-816x1056.jp2");

    /// WS1.7: nearest-neighbour upsample of a 2×2 subsampled plane to 4×4 —
    /// each source sample fills its 2×2 output block.
    #[test]
    fn upsample_nearest_2x2_to_4x4() {
        // 2×2 plane: [10 20 / 30 40]
        let sub = [10.0f32, 20.0, 30.0, 40.0];
        let out = upsample_nearest_u8(&sub, 2, 2, 4, 4).unwrap();
        assert_eq!(
            out,
            vec![
                10, 10, 20, 20, //
                10, 10, 20, 20, //
                30, 30, 40, 40, //
                30, 30, 40, 40,
            ]
        );
    }

    /// Odd full dimensions (⌈w/2⌉ source): upsample 2×2 → 3×3 clamps at edges.
    #[test]
    fn upsample_nearest_2x2_to_3x3() {
        let sub = [1.0f32, 2.0, 3.0, 4.0];
        let out = upsample_nearest_u8(&sub, 2, 2, 3, 3).unwrap();
        assert_eq!(out.len(), 9);
        assert_eq!(out[0], 1); // (0,0)
        assert_eq!(out[8], 4); // (2,2) → source (1,1)
    }

    /// A target resolution decodes fewer levels, and the reported geometry
    /// follows the samples rather than the codestream's full size.
    ///
    /// This is what keeps an oversized image affordable: decoding a
    /// 12608 x 16806 JPX in full peaks at 11.0 GB, which no browser tab or
    /// phone survives, for a picture painted into a fraction of that.
    #[test]
    fn test_target_resolution_decodes_a_smaller_image() {
        let full = decode_jpx_at(SYNTHETIC_GRAY_JP2, None).expect("decode at full size");
        assert_eq!((full.width, full.height), (816, 1056), "test premise");

        let small = decode_jpx_at(SYNTHETIC_GRAY_JP2, Some((200, 260)))
            .expect("decode at a reduced resolution");

        // Never smaller than asked for — the decoder picks the smallest level
        // that still covers the target ...
        assert!(
            small.width >= 200 && small.height >= 260,
            "reduced decode {}x{} is below the requested 200x260",
            small.width,
            small.height
        );
        // ... but genuinely smaller than the full image, which is the point:
        // without this the full-resolution samples are always materialised.
        assert!(
            small.width < full.width && small.height < full.height,
            "target resolution had no effect: still {}x{}",
            small.width,
            small.height
        );
        // The geometry must describe the samples actually returned, or the
        // caller reads the buffer with the wrong row stride.
        assert_eq!(
            small.samples.len(),
            small.width as usize * small.height as usize * usize::from(small.num_components),
            "reported geometry does not match the sample count"
        );
    }

    #[test]
    fn decode_jpx_grayscale() {
        let img = decode_jpx_at(SYNTHETIC_GRAY_JP2, None).expect("decode JP2 codestream");

        assert_eq!(img.num_components, 1);
        assert_eq!(img.samples.len(), 816 * 1056);

        // The independently generated grayscale control is not one flat value.
        let first = img.samples[0];
        assert!(
            img.samples.iter().any(|&b| b != first),
            "decoded image is uniformly flat — decode likely failed"
        );
    }
}
