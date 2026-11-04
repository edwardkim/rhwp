//! Image extraction from PDF XObject resources.
//!
//! This module provides functionality to extract images from PDF documents,
//! including JPEG pass-through for DCT-encoded images and raw pixel decoding
//! for other image types.
//!
//! Phase 5

use crate::cache::MutexExt;
use crate::error::{Error, Result};
use crate::extractors::ccitt_bilevel;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::borrow::Cow;
use std::cmp::min;
use std::path::Path;
use std::sync::{Arc, OnceLock};

#[path = "image_mask_limited.rs"]
mod image_mask_limited;
#[path = "image_mask_limited_extra.rs"]
mod image_mask_limited_extra;

#[path = "image_stencil_tiles.rs"]
mod image_stencil_tiles;

/// A PDF image with metadata and pixel data.
///
/// Represents an image extracted from a PDF, including dimensions,
/// color space information, and the actual image data (either JPEG
/// or raw pixels).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PdfImage {
    /// Image width in pixels
    width: u32,
    /// Image height in pixels
    height: u32,
    /// Color space of the image
    color_space: ColorSpace,
    /// Bits per color component (typically 8)
    bits_per_component: u8,
    /// Image data (JPEG or raw pixels)
    #[serde(
        skip_serializing_if = "ImageData::is_empty",
        serialize_with = "serialize_image_data"
    )]
    data: Arc<ImageData>,
    #[serde(skip)]
    png_cache: PngCache,
    /// Optional bounding box in PDF user space (v0.3.14)
    bbox: Option<Rect>,
    /// Rotation in degrees (v0.3.14)
    rotation_degrees: i32,
    /// Transformation matrix (v0.3.14)
    matrix: [f32; 6],
    /// CCITT decompression parameters (for 1-bit bilevel images)
    #[serde(skip)]
    ccitt_params: Option<crate::decoders::CcittParams>,
    /// Embedded ICC profile associated with the image's colour space,
    /// if any. For a plain `/ICCBased` image this is the profile from
    /// the array; for an `Indexed` image with an `ICCBased` base this
    /// is the base profile. `None` when the document only used
    /// device-dependent colour. Consumed by `save_as_*` to drive the
    /// CMYK→sRGB conversion through the CMM instead of the §10.3.5
    /// additive-clamp fallback.
    #[serde(skip)]
    icc_profile: Option<std::sync::Arc<crate::color::IccProfile>>,
    /// Rendering intent from the image dictionary's `/Intent`, or the
    /// graphics-state default per ISO 32000-1:2008 §8.6.5.8.
    rendering_intent: crate::color::RenderingIntent,
    /// Whether `data` still holds the image's raw samples, i.e. the values
    /// the image dictionary's own entries are expressed in.
    ///
    /// Consumers that range-test stored samples against dictionary values —
    /// colour-key `/Mask` (§8.9.6.4) — are only meaningful in that space.
    /// Every rescale leaves it: mapping a non-default `/Decode` into the
    /// buffer, expanding an Indexed image to palette RGB, unpacking a
    /// sub-byte sample to the 8-bit range, and collapsing a 16-bit sample to
    /// its high byte. A CCITT buffer stays raw, with its `/Decode` polarity
    /// carried separately in `ccitt_params`.
    #[serde(skip)]
    samples_are_raw: bool,
    /// Whether a non-default `/Decode` array is already mapped into `data`.
    ///
    /// Strictly narrower than `!samples_are_raw`, and not interchangeable
    /// with it: unpacking a sub-byte sample, expanding an Indexed image and
    /// reducing 16-bit samples all leave the raw space *without* folding in
    /// `/Decode`. A consumer that applies `/Decode` itself — separation-plate
    /// routing — must read this one, or it drops a map the image is still
    /// owed (e.g. a `/Decode [1 0]` inversion) on every sub-byte image.
    #[serde(skip)]
    decode_folded_in: bool,
    /// Explicit bounded-import sampling, persisted in the PNG resource.
    interpolate: Option<bool>,
}

// Encoded bytes are an optimization, not part of image value equality.
#[derive(Debug, Clone, Default)]
struct PngCache(Option<Arc<OnceLock<Vec<u8>>>>);
impl PartialEq for PngCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
fn serialize_image_data<S: serde::Serializer>(
    data: &Arc<ImageData>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serde::Serialize::serialize(data.as_ref(), serializer)
}

impl PdfImage {
    /// Create a new PDF image.
    pub fn new(
        width: u32,
        height: u32,
        color_space: ColorSpace,
        bits_per_component: u8,
        data: ImageData,
    ) -> Self {
        Self {
            width,
            height,
            color_space,
            bits_per_component,
            data: Arc::new(data),
            png_cache: PngCache::default(),
            bbox: None,
            rotation_degrees: 0,
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ccitt_params: None,
            icc_profile: None,
            rendering_intent: crate::color::RenderingIntent::default(),
            samples_are_raw: true,
            decode_folded_in: false,
            interpolate: None,
        }
    }

    fn convert_to_srgb_with_budget(
        &mut self,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<()> {
        self.png_cache = PngCache::default();
        let Some(transform) = self.build_icc_transform().filter(|t| t.has_cmm()) else {
            budget.check()?;
            return Ok(());
        };
        budget.check()?;
        let (pixels, format) = match self.data.as_ref() {
            ImageData::Raw {
                pixels,
                format: PixelFormat::RGBA,
            } if self.bits_per_component == 8 => {
                let source_channels = transform.source_n_components() as usize;
                if !matches!(source_channels, 1 | 3) {
                    return Err(Error::Image("ICC RGBA source components".into()));
                }
                let mut rgba = sample_buffer(pixels.len(), Some(budget))?;
                for pixel in pixels.chunks_exact(4) {
                    let rgb = transform
                        .convert_pixel(&pixel[..source_channels])
                        .ok_or_else(|| Error::Image("ICC alpha conversion".into()))?;
                    rgba.extend_from_slice(&rgb);
                    rgba.push(pixel[3]);
                }
                (rgba, PixelFormat::RGBA)
            },
            ImageData::Raw { pixels, format } if self.bits_per_component == 8 => (
                self.raw_rgb_with_budget(pixels, *format, budget)?
                    .into_raw(),
                PixelFormat::RGB,
            ),
            ImageData::Jpeg(_) => {
                let rgb = Self::into_rgb_with_budget(
                    self.decode_jpeg_with_budget(budget)?,
                    Some(budget),
                )?;
                let rgb = if transform.source_n_components() == 4 {
                    rgb
                } else {
                    self.raw_rgb_with_budget(rgb.as_raw(), PixelFormat::RGB, budget)?
                };
                (rgb.into_raw(), PixelFormat::RGB)
            },
            _ => return Err(Error::Unsupported("ICC source sample depth".into())),
        };
        budget.check()?;
        self.data = Arc::new(ImageData::Raw { pixels, format });
        self.color_space = ColorSpace::DeviceRGB;
        self.icc_profile = None;
        self.samples_are_raw = false;
        Ok(())
    }

    fn apply_opacity(
        &mut self,
        opacity: f32,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<()> {
        self.png_cache = PngCache::default();
        if matches!(
            self.data.as_ref(),
            ImageData::Raw {
                format: PixelFormat::RGBA,
                ..
            }
        ) {
            let row_bytes = self.width as usize * 4;
            let ImageData::Raw { pixels, .. } = self.samples_mut(budget)? else {
                unreachable!()
            };
            for row in pixels.chunks_mut(row_bytes) {
                budget.check()?;
                for p in row.chunks_exact_mut(4) {
                    p[3] = (f32::from(p[3]) * opacity).round() as u8;
                }
            }
            return Ok(());
        }
        let rgb: Cow<'_, [u8]> = match self.data.as_ref() {
            ImageData::Raw {
                pixels,
                format: PixelFormat::RGB,
            } if self.bits_per_component == 8 => Cow::Borrowed(pixels),
            ImageData::Jpeg(_) => Cow::Owned(
                Self::into_rgb_with_budget(self.decode_jpeg_with_budget(budget)?, Some(budget))?
                    .into_raw(),
            ),
            ImageData::Raw { pixels, format } if self.bits_per_component == 8 => Cow::Owned(
                self.raw_rgb_with_budget(pixels, *format, budget)?
                    .into_raw(),
            ),
            _ => return Err(Error::Unsupported("limited image opacity sample format".into())),
        };
        let count = (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or_else(|| budget.reject("image opacity geometry"))?;
        if rgb.len() != count * 3 {
            return Err(Error::Image("invalid RGB opacity samples".into()));
        }
        let mut pixels = sample_buffer(
            count
                .checked_mul(4)
                .ok_or_else(|| budget.reject("image opacity bytes"))?,
            Some(budget),
        )?;
        for row in rgb.chunks(self.width as usize * 3) {
            budget.check()?;
            for c in row.chunks_exact(3) {
                pixels.extend_from_slice(&[c[0], c[1], c[2], (opacity * 255.0).round() as u8]);
            }
        }
        self.data = Arc::new(ImageData::Raw {
            pixels,
            format: PixelFormat::RGBA,
        });
        self.bits_per_component = 8;
        self.color_space = ColorSpace::DeviceRGB;
        Ok(())
    }

    /// Whether the stored samples are still in the image's raw sample space
    /// (see the field docs for what leaves it).
    pub fn samples_are_raw(&self) -> bool {
        self.samples_are_raw
    }

    /// Bake a convex page-space clip and affine placement into an RGBA frame.
    /// The rectangle is storage geometry only; pixels outside the real boundary
    /// remain transparent. Axis-aligned editable crops keep their existing path.
    fn apply_affine_clip(
        &mut self,
        ctm: crate::content::Matrix,
        clip: &super::image_clip::ImageClip,
        frame: crate::geometry::Rect,
        interpolate: bool,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<()> {
        let [a, b, c, d, e, f] = [ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f].map(f64::from);
        let determinant = a * d - b * c;
        if !determinant.is_finite() || determinant == 0.0 {
            return Err(Error::Unsupported("singular clipped image matrix".into()));
        }
        let (sw, sh) = (self.width as usize, self.height as usize);
        if sw == 0 || sh == 0 {
            return Err(Error::Image("empty clipped image samples".into()));
        }
        // The clip edge is geometric, even for a one-pixel solid image. Match
        // the product's compound-path raster density, retaining higher source
        // density. This changes storage resolution, never the physical frame.
        let density_x = ((d * sw as f64).hypot(b * sh as f64) / determinant.abs()).max(3.0);
        let density_y = ((c * sw as f64).hypot(a * sh as f64) / determinant.abs()).max(3.0);
        let width = (f64::from(frame.width) * density_x).ceil();
        let height = (f64::from(frame.height) * density_y).ceil();
        if !width.is_finite()
            || !height.is_finite()
            || width < 1.0
            || height < 1.0
            || width > 16_384.0
            || height > 16_384.0
            || width * height > 64_000_000.0
        {
            return Err(budget.reject("clipped image raster dimensions"));
        }
        let (width, height) = (width as usize, height as usize);
        let mask = match clip {
            super::image_clip::ImageClip::Complex(c) => {
                Some(c.raster(frame, width, height, budget)?)
            },
            _ => None,
        };
        let mut output = sample_buffer(width * height * 4, Some(budget))?;
        // Use the existing bounded codec/colour conversion; existing alpha is
        // retained. Both this conversion and the destination are charged.
        self.apply_opacity(1.0, budget)?;
        let ImageData::Raw {
            pixels,
            format: PixelFormat::RGBA,
        } = self.data.as_ref()
        else {
            return Err(Error::Image("clipped image requires RGBA samples".into()));
        };
        if sw.checked_mul(sh).and_then(|n| n.checked_mul(4)) != Some(pixels.len()) {
            return Err(Error::Image("invalid clipped image sample length".into()));
        }
        let sample = |x: isize, y: isize| {
            // PDF interpolation extends edge samples at the unit-square edge.
            let x = x.clamp(0, sw as isize - 1) as usize;
            let y = y.clamp(0, sh as isize - 1) as usize;
            &pixels[(y * sw + x) * 4..(y * sw + x + 1) * 4]
        };
        for y in 0..height {
            budget.check()?;
            let py = f64::from(frame.y)
                + f64::from(frame.height) * (1.0 - (y as f64 + 0.5) / height as f64);
            for x in 0..width {
                let px =
                    f64::from(frame.x) + f64::from(frame.width) * (x as f64 + 0.5) / width as f64;
                let u = (d * (px - e) - c * (py - f)) / determinant;
                let v = (-b * (px - e) + a * (py - f)) / determinant;
                if !(0.0..=1.0).contains(&u)
                    || !(0.0..=1.0).contains(&v)
                    || !clip.contains(crate::geometry::Point::new(px as f32, py as f32))
                {
                    output.extend_from_slice(&[0; 4]);
                } else if !interpolate {
                    output.extend_from_slice(sample(
                        (u * sw as f64).floor() as isize,
                        ((1.0 - v) * sh as f64).floor() as isize,
                    ));
                } else {
                    let (sx, sy) = (u * sw as f64 - 0.5, (1.0 - v) * sh as f64 - 0.5);
                    let (ix, iy) = (sx.floor() as isize, sy.floor() as isize);
                    let (fx, fy) = (sx - sx.floor(), sy - sy.floor());
                    let mut channels = [0.0; 4];
                    for (dx, dy, w) in [
                        (0, 0, (1.0 - fx) * (1.0 - fy)),
                        (1, 0, fx * (1.0 - fy)),
                        (0, 1, (1.0 - fx) * fy),
                        (1, 1, fx * fy),
                    ] {
                        let p = sample(ix + dx, iy + dy);
                        channels[3] += f64::from(p[3]) * w;
                        for k in 0..3 {
                            channels[k] += f64::from(p[k]) * f64::from(p[3]) * w;
                        }
                    }
                    let mut rgba = [0; 4];
                    rgba[3] = channels[3].round() as u8;
                    if channels[3] > 0.0 {
                        for k in 0..3 {
                            rgba[k] = (channels[k] / channels[3]).round() as u8;
                        }
                    }
                    output.extend_from_slice(&rgba);
                }
                if let Some(mask) = &mask {
                    let alpha = output.last_mut().expect("RGBA pixel was emitted");
                    *alpha = ((u16::from(*alpha) * u16::from(mask.data()[y * width + x]) + 127)
                        / 255) as u8;
                }
            }
        }
        self.width = width as u32;
        self.height = height as u32;
        self.data = Arc::new(ImageData::Raw {
            pixels: output,
            format: PixelFormat::RGBA,
        });
        self.color_space = ColorSpace::DeviceRGB;
        self.bits_per_component = 8;
        self.set_bbox(frame);
        self.set_matrix([frame.width, 0.0, 0.0, frame.height, frame.x, frame.y]);
        self.set_rotation_degrees(0);
        Ok(())
    }

    /// Record that the stored samples have been transformed out of the raw
    /// sample space.
    pub(crate) fn set_samples_are_raw(&mut self, raw: bool) {
        self.samples_are_raw = raw;
    }

    /// Whether a non-default `/Decode` is already mapped into the stored
    /// samples, so a consumer that applies `/Decode` itself must not do so
    /// again. Do not substitute `!samples_are_raw()` — see the field docs.
    pub fn decode_folded_in(&self) -> bool {
        self.decode_folded_in
    }

    /// Record that a non-default `/Decode` has been mapped into the stored
    /// samples.
    pub(crate) fn set_decode_folded_in(&mut self, folded: bool) {
        self.decode_folded_in = folded;
    }

    /// Create a new PDF image with spatial metadata (v0.3.14).
    pub fn with_spatial(
        width: u32,
        height: u32,
        color_space: ColorSpace,
        bits_per_component: u8,
        data: ImageData,
        bbox: Rect,
        rotation: i32,
        matrix: [f32; 6],
    ) -> Self {
        Self {
            width,
            height,
            color_space,
            bits_per_component,
            data: Arc::new(data),
            png_cache: PngCache::default(),
            bbox: Some(bbox),
            rotation_degrees: rotation,
            matrix,
            ccitt_params: None,
            icc_profile: None,
            rendering_intent: crate::color::RenderingIntent::default(),
            samples_are_raw: true,
            decode_folded_in: false,
            interpolate: None,
        }
    }

    /// Create a new PDF image with a bounding box (v0.3.12, convenience wrapper).
    pub fn with_bbox(
        width: u32,
        height: u32,
        color_space: ColorSpace,
        bits_per_component: u8,
        data: ImageData,
        bbox: Rect,
    ) -> Self {
        Self::with_spatial(
            width,
            height,
            color_space,
            bits_per_component,
            data,
            bbox,
            0,
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        )
    }

    /// Create a new PDF image with CCITT parameters.
    pub fn with_ccitt_params(
        width: u32,
        height: u32,
        color_space: ColorSpace,
        bits_per_component: u8,
        data: ImageData,
        ccitt_params: crate::decoders::CcittParams,
    ) -> Self {
        Self {
            width,
            height,
            color_space,
            bits_per_component,
            data: Arc::new(data),
            png_cache: PngCache::default(),
            bbox: None,
            rotation_degrees: 0,
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ccitt_params: Some(ccitt_params),
            icc_profile: None,
            rendering_intent: crate::color::RenderingIntent::default(),
            samples_are_raw: true,
            decode_folded_in: false,
            interpolate: None,
        }
    }

    /// Get the image width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the image height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the image color space.
    pub fn color_space(&self) -> &ColorSpace {
        &self.color_space
    }

    /// Get bits per component.
    pub fn bits_per_component(&self) -> u8 {
        self.bits_per_component
    }

    /// Get the image data.
    pub fn data(&self) -> &ImageData {
        self.data.as_ref()
    }

    /// Get the bounding box if available.
    pub fn bbox(&self) -> Option<&Rect> {
        self.bbox.as_ref()
    }

    /// Set the bounding box for this image.
    pub fn set_bbox(&mut self, bbox: Rect) {
        self.bbox = Some(bbox);
    }

    /// Get rotation in degrees.
    pub fn rotation_degrees(&self) -> i32 {
        self.rotation_degrees
    }

    /// Set rotation in degrees.
    pub fn set_rotation_degrees(&mut self, rotation: i32) {
        self.rotation_degrees = rotation;
    }

    /// Get transformation matrix.
    pub fn matrix(&self) -> [f32; 6] {
        self.matrix
    }

    /// Set transformation matrix.
    pub fn set_matrix(&mut self, matrix: [f32; 6]) {
        self.matrix = matrix;
    }

    /// Set CCITT decompression parameters for this image.
    pub fn set_ccitt_params(&mut self, params: crate::decoders::CcittParams) {
        self.png_cache = PngCache::default();
        self.ccitt_params = Some(params);
    }

    /// Get CCITT decompression parameters if available.
    pub fn ccitt_params(&self) -> Option<&crate::decoders::CcittParams> {
        self.ccitt_params.as_ref()
    }

    /// Embedded ICC profile associated with the image, if any.
    pub fn icc_profile(&self) -> Option<&std::sync::Arc<crate::color::IccProfile>> {
        self.icc_profile.as_ref()
    }

    /// Attach an ICC profile (used by extractors; colour conversion
    /// picks it up automatically when present).
    pub fn set_icc_profile(&mut self, profile: std::sync::Arc<crate::color::IccProfile>) {
        self.png_cache = PngCache::default();
        self.icc_profile = Some(profile);
    }

    /// Rendering intent — ISO 32000-1:2008 §8.6.5.8, defaults to
    /// `RelativeColorimetric`.
    pub fn rendering_intent(&self) -> crate::color::RenderingIntent {
        self.rendering_intent
    }

    /// Set the rendering intent (used by extractors when they see an
    /// explicit `/Intent` entry on the image dictionary).
    pub fn set_rendering_intent(&mut self, intent: crate::color::RenderingIntent) {
        self.png_cache = PngCache::default();
        self.rendering_intent = intent;
    }

    /// Build the source→sRGB transform from this image's embedded ICC
    /// profile (if any). Returns `None` when the image uses purely
    /// device-dependent colour, or when no profile was resolved at
    /// extraction time.
    ///
    /// The resulting transform is component-agnostic: callers pick the
    /// matching `Transform::convert_{cmyk,rgb,gray}_*` method based on
    /// the source pixel format. Used by the `decode_cmyk_jpeg_to_rgb_…`,
    /// `cmyk_to_rgb_with_transform`, and `save_raw_as_*` paths.
    fn build_icc_transform(&self) -> Option<std::sync::Arc<crate::color::Transform>> {
        self.icc_profile
            .as_ref()
            .map(|p| crate::color::IccProfile::cached_transform(p, self.rendering_intent))
    }

    /// Save the image as PNG format.
    pub fn save_as_png(&self, path: impl AsRef<Path>) -> Result<()> {
        match self.data.as_ref() {
            ImageData::Jpeg(jpeg_data) => {
                if self.color_space.components() == 4 {
                    let transform = self.build_icc_transform();
                    let rgb =
                        decode_cmyk_jpeg_to_rgb_with_profile(jpeg_data, transform.as_deref())?;
                    let buf = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                        self.width,
                        self.height,
                        rgb,
                    )
                    .ok_or_else(|| Error::Image("Invalid CMYK image dimensions".to_string()))?;
                    buf.save_with_format(path, image::ImageFormat::Png)
                        .map_err(|e| Error::Image(format!("Failed to save PNG: {}", e)))
                } else {
                    save_jpeg_as_png(jpeg_data, path)
                }
            },
            ImageData::Raw { pixels, format } => {
                // Always build the transform if a profile is present; the
                // save helper picks the right convert_* method for the
                // pixel format. RGB/Gray ICCBased samples would otherwise
                // be written as-is, which is wrong when the profile is
                // wide-gamut (Adobe RGB, ProPhoto, …) or a calibrated
                // grayscale other than sRGB gamma.
                let transform = self.build_icc_transform();
                save_raw_as_png(
                    pixels,
                    self.width,
                    self.height,
                    *format,
                    transform.as_deref(),
                    path,
                )
            },
        }
    }

    /// Save the image as JPEG format.
    pub fn save_as_jpeg(&self, path: impl AsRef<Path>) -> Result<()> {
        match self.data.as_ref() {
            // Pass-through for RGB / grayscale JPEGs — viewers handle those
            // uniformly. CMYK JPEGs (4-channel ColorSpace such as DeviceCMYK
            // or ICCBased N=4) must be decoded and re-encoded as RGB because
            // most viewers either fail to open CMYK JPEGs or display them
            // with inverted or washed-out colors. `decode_cmyk_jpeg_to_rgb`
            // pulls CMYK samples via `jpeg-decoder`, inspects the APP14
            // Adobe marker to detect the inverted-channel convention
            // Photoshop / InDesign / WPS write, inverts when present, then
            // does a naive CMYK→RGB conversion (full ICC profile handling
            // is a follow-up).
            ImageData::Jpeg(jpeg_data) => {
                if self.color_space.components() == 4 {
                    let transform = self.build_icc_transform();
                    let rgb =
                        decode_cmyk_jpeg_to_rgb_with_profile(jpeg_data, transform.as_deref())?;
                    let buf = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                        self.width,
                        self.height,
                        rgb,
                    )
                    .ok_or_else(|| Error::Image("Invalid CMYK image dimensions".to_string()))?;
                    buf.save_with_format(path, image::ImageFormat::Jpeg)
                        .map_err(|e| Error::Image(format!("Failed to save JPEG: {}", e)))
                } else {
                    std::fs::write(path, jpeg_data).map_err(Error::from)
                }
            },
            ImageData::Raw { pixels, format } => {
                let transform = self.build_icc_transform();
                save_raw_as_jpeg(
                    pixels,
                    self.width,
                    self.height,
                    *format,
                    transform.as_deref(),
                    path,
                )
            },
        }
    }

    /// Convert image to PNG bytes in memory.
    pub fn to_png_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.encode_png(&mut buffer, None)?;
        Ok(buffer)
    }

    /// Validate real JPEG dimensions before pixel allocation and bound PNG output.
    pub fn to_png_bytes_with_budget(
        &self,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<Vec<u8>> {
        budget.check()?;
        let raw_size = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|n| n.checked_mul(4))
            .filter(|&n| n > 0 && n <= budget.limits.stream_bytes)
            .ok_or_else(|| budget.reject("image pixel buffer"))?;
        if ![1, 2, 4, 8].contains(&self.bits_per_component) {
            return Err(Error::Unsupported("limited import image sample depth".into()));
        }
        if let Some(bytes) = self.png_cache.0.as_ref().and_then(|cache| cache.get()) {
            return budget.copy_bytes(bytes);
        }
        // Legacy raw conversion still needs its conservative work allowance.
        // JPEG reservations now flow through the audited RGB/CMYK decoder hooks,
        // into_rgb_with_budget and the bounded PNG compressor/output writer.
        // Charging an additional imaginary 3*RGBA buffer counted that work twice.
        if self.borrowed_png_samples().is_none()
            && !matches!(self.data.as_ref(), ImageData::Jpeg(_))
        {
            budget.charge_decoded(
                raw_size
                    .checked_mul(3)
                    .ok_or_else(|| budget.reject("image work overflow"))?,
            )?;
        }
        let mut buffer = budget.output();
        let result = self.encode_png(&mut buffer, Some(budget));
        budget.check()?;
        result?;
        let bytes = buffer.finish();
        if let Some(cache) = &self.png_cache.0 {
            let _ = cache.set(bytes);
            return budget.copy_bytes(cache.get().expect("encoded PNG"));
        }
        Ok(bytes)
    }

    fn samples_mut(
        &mut self,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<&mut ImageData> {
        self.png_cache = PngCache::default();
        if Arc::get_mut(&mut self.data).is_none() {
            let copy = match self.data.as_ref() {
                ImageData::Jpeg(bytes) => ImageData::Jpeg(budget.copy_bytes(bytes)?),
                ImageData::Raw { pixels, format } => ImageData::Raw {
                    pixels: budget.copy_bytes(pixels)?,
                    format: *format,
                },
            };
            self.data = Arc::new(copy);
        }
        Ok(Arc::get_mut(&mut self.data).expect("unique pixel copy"))
    }

    // This result is consumed by both budget accounting and encoding. Already
    // decoded samples need no image-sized decode/conversion scratch: PNG feeds
    // borrowed rows into the audited compressor and bounded output writer.
    fn borrowed_png_samples(&self) -> Option<(&[u8], usize)> {
        let ImageData::Raw { pixels, format } = self.data.as_ref() else {
            return None;
        };
        if self.bits_per_component != 8 || self.ccitt_params.is_some() || self.icc_profile.is_some()
        {
            return None;
        }
        let channels = match format {
            PixelFormat::RGBA => 4,
            PixelFormat::RGB => 3,
            PixelFormat::Grayscale
                if matches!(self.color_space, ColorSpace::DeviceGray | ColorSpace::CalGray) =>
            {
                1
            },
            _ => return None,
        };
        let size = (self.width as usize)
            .checked_mul(self.height as usize)?
            .checked_mul(channels)?;
        (pixels.len() == size).then_some((pixels, channels))
    }

    fn encode_png(
        &self,
        buffer: &mut impl std::io::Write,
        budget: Option<&crate::import_limits::ImportBudget>,
    ) -> Result<()> {
        use image::codecs::png::{CompressionType, FilterType, PngEncoder};
        use image::ImageEncoder;
        // `Adaptive` per-scanline filtering (Sub/Up/Average/Paeth) is where PNG's
        // deflate win actually comes from on photographic / scanned content; with
        // `NoFilter` deflate cannot exploit smooth gradients and the stream stays
        // near-raw. This is purely a container-size choice - the decoded pixels
        // are byte-identical either way (PNG is lossless) - so it trades a little
        // encode CPU for large size reductions (measured ~40x on flat scans).
        let mut write_pixels = |pixels: &[u8], color: image::ExtendedColorType| -> Result<()> {
            if let Some(budget) = budget {
                return super::png_limited::encode(
                    buffer,
                    pixels,
                    self.width,
                    self.height,
                    color.channel_count() as usize,
                    budget,
                    self.interpolate,
                );
            }
            PngEncoder::new_with_quality(
                &mut *buffer,
                CompressionType::Default,
                FilterType::Adaptive,
            )
            .write_image(pixels, self.width, self.height, color)
            .map_err(|e| Error::Encode(format!("Failed to encode PNG: {}", e)))
        };

        let transform = self.build_icc_transform();
        let mut write_converted = |pixels: &[u8], channels: usize| -> Result<()> {
            if let Some(transform) = transform.as_ref().filter(|t| t.has_cmm()) {
                let source_channels = if channels == 4 && transform.source_n_components() != 4 {
                    transform.source_n_components() as usize
                } else {
                    channels
                };
                if source_channels != transform.source_n_components() as usize {
                    return Err(Error::Image("ICC components differ from stored samples".into()));
                }
                let alpha = channels == 4 && source_channels != 4;
                let output_channels = if alpha { 4 } else { 3 };
                let count = pixels
                    .len()
                    .checked_div(channels)
                    .and_then(|n| n.checked_mul(output_channels))
                    .ok_or_else(|| Error::Image("ICC output geometry".into()))?;
                let mut converted = sample_buffer(count, budget)?;
                for row in pixels.chunks(self.width as usize * channels) {
                    if let Some(budget) = budget {
                        budget.check()?;
                    }
                    for pixel in row.chunks_exact(channels) {
                        let rgb = transform
                            .convert_pixel(&pixel[..source_channels])
                            .ok_or_else(|| Error::Image("ICC sample conversion".into()))?;
                        converted.extend_from_slice(&rgb);
                        if alpha {
                            converted.push(pixel[3]);
                        }
                    }
                }
                if let Some(budget) = budget {
                    budget.check()?;
                }
                write_pixels(
                    &converted,
                    if alpha {
                        image::ColorType::Rgba8
                    } else {
                        image::ColorType::Rgb8
                    }
                    .into(),
                )
            } else {
                write_pixels(
                    pixels,
                    match channels {
                        1 => image::ColorType::L8,
                        4 => image::ColorType::Rgba8,
                        _ => image::ColorType::Rgb8,
                    }
                    .into(),
                )
            }
        };
        if let Some((pixels, channels)) = self.borrowed_png_samples() {
            let color = match channels {
                1 => image::ColorType::L8,
                3 => image::ColorType::Rgb8,
                4 => image::ColorType::Rgba8,
                _ => unreachable!("validated borrowed PNG samples"),
            };
            return write_converted(pixels, color.channel_count() as usize);
        }
        match self.data.as_ref() {
            ImageData::Raw { pixels, format } => {
                let expected_gray = (self.width as usize)
                    .checked_mul(self.height as usize)
                    .ok_or_else(|| Error::Image("image geometry overflow".into()))?;
                let expected_rgb = expected_gray
                    .checked_mul(3)
                    .ok_or_else(|| Error::Image("image RGB size overflow".into()))?;
                let ccitt_budget = budget.filter(|_| {
                    self.bits_per_component == 1
                        && self.color_space == ColorSpace::DeviceGray
                        && self.ccitt_params.is_some()
                });

                if *format == PixelFormat::RGBA && self.bits_per_component == 8 {
                    if expected_gray.checked_mul(4) != Some(pixels.len()) {
                        return Err(Error::Image("invalid RGBA image length".into()));
                    }
                    write_converted(pixels, 4)?;
                } else if ccitt_budget.is_none()
                    && *format == PixelFormat::Grayscale
                    && matches!(self.color_space, ColorSpace::DeviceGray | ColorSpace::CalGray)
                    && pixels.len() == expected_gray
                {
                    // image 0.25 changed `write_image` to take
                    // `ExtendedColorType` — `ColorType::*` now converts
                    // through `Into`. API-only change, same semantics.
                    write_converted(pixels, 1)?;
                } else if ccitt_budget.is_none()
                    && *format == PixelFormat::RGB
                    && pixels.len() == expected_rgb
                {
                    write_converted(pixels, 3)?;
                } else if *format == PixelFormat::CMYK
                    && self.bits_per_component == 8
                    && transform.as_ref().is_some_and(|t| t.has_cmm())
                {
                    write_converted(pixels, 4)?;
                } else {
                    let dynamic_image = if let Some(budget) = ccitt_budget {
                        self.decode_ccitt_with_budget(budget)?
                    } else if let Some(budget) = budget {
                        if self.bits_per_component != 8 {
                            return Err(Error::Unsupported(
                                "limited import packed colour samples".into(),
                            ));
                        }
                        image::DynamicImage::ImageRgb8(
                            self.raw_rgb_with_budget(pixels, *format, budget)?,
                        )
                    } else {
                        self.to_dynamic_image()?
                    };
                    let rgb = Self::into_rgb_with_budget(dynamic_image, budget)?;
                    // `ImageBuffer::from_raw` accepts a buffer at least as long
                    // as the image needs, so a mis-declared depth can leave the
                    // RGB buffer larger than width×height×3. Reject that here
                    // with a recoverable error instead of handing the encoder a
                    // mismatched buffer, which it asserts on (panicking through
                    // the FFI boundary where callers cannot catch it).
                    if rgb.as_raw().len() != expected_rgb {
                        return Err(Error::Encode(format!(
                            "image buffer length {} does not match {}x{} RGB image ({} expected)",
                            rgb.as_raw().len(),
                            self.width,
                            self.height,
                            expected_rgb
                        )));
                    }
                    write_converted(rgb.as_raw(), 3)?;
                }
            },
            ImageData::Jpeg(_) => {
                let dynamic_image = if let Some(budget) = budget {
                    self.decode_jpeg_with_budget(budget)?
                } else {
                    self.to_dynamic_image()?
                };
                let rgb = Self::into_rgb_with_budget(dynamic_image, budget)?;
                if self.color_space.components() == 4 {
                    write_pixels(rgb.as_raw(), image::ColorType::Rgb8.into())?;
                } else {
                    write_converted(rgb.as_raw(), 3)?;
                }
            },
        }

        Ok(())
    }

    fn raw_rgb_with_budget(
        &self,
        pixels: &[u8],
        format: PixelFormat,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<image::RgbImage> {
        let count = (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or_else(|| budget.reject("image RGB size overflow"))?;
        if count.checked_mul(format.bytes_per_pixel()) != Some(pixels.len()) {
            return Err(Error::Image("invalid raw colour image length".into()));
        }
        let size = count
            .checked_mul(3)
            .ok_or_else(|| budget.reject("image RGB size overflow"))?;
        let mut rgb = sample_buffer(size, Some(budget))?;
        match format {
            PixelFormat::RGB => rgb.extend_from_slice(pixels),
            PixelFormat::RGBA => {
                for pixel in pixels.chunks_exact(4) {
                    rgb.extend_from_slice(&pixel[..3]);
                }
            },
            PixelFormat::Grayscale => {
                for &gray in pixels {
                    rgb.extend_from_slice(&[gray; 3]);
                }
            },
            PixelFormat::CMYK => {
                let transform = self.build_icc_transform();
                for c in pixels.chunks_exact(4) {
                    let value = match &transform {
                        Some(t) => t.convert_cmyk_pixel(c[0], c[1], c[2], c[3]),
                        None => cmyk_pixel_to_rgb(c[0], c[1], c[2], c[3]),
                    };
                    rgb.extend_from_slice(&value);
                }
            },
        }
        if format != PixelFormat::CMYK {
            if let Some(transform) = self.build_icc_transform().filter(|t| t.has_cmm()) {
                let source_channels = transform.source_n_components() as usize;
                for pixel in rgb.chunks_exact_mut(3) {
                    let converted = transform
                        .convert_pixel(&pixel[..source_channels.min(3)])
                        .ok_or_else(|| Error::Image("ICC raw components".into()))?;
                    pixel.copy_from_slice(&converted);
                }
                budget.check()?;
            }
        }
        image::RgbImage::from_raw(self.width, self.height, rgb)
            .ok_or_else(|| Error::Image("invalid RGB image geometry".into()))
    }

    fn into_rgb_with_budget(
        image: image::DynamicImage,
        budget: Option<&crate::import_limits::ImportBudget>,
    ) -> Result<image::RgbImage> {
        let Some(budget) = budget else {
            return Ok(image.to_rgb8());
        };
        // Consume an RGB buffer without cloning it. Other formats allocate one
        // RGB output in into_rgb8; account for that payload before conversion.
        if !matches!(image, image::DynamicImage::ImageRgb8(_)) {
            let size = (image.width() as usize)
                .checked_mul(image.height() as usize)
                .and_then(|n| n.checked_mul(3))
                .filter(|&n| n <= budget.limits.stream_bytes)
                .ok_or_else(|| budget.reject("image RGB size overflow"))?;
            budget.charge_codec_allocation(size)?;
        }
        Ok(image.into_rgb8())
    }

    fn decode_ccitt_with_budget(
        &self,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<image::DynamicImage> {
        let (ImageData::Raw { pixels, .. }, Some(params)) =
            (self.data.as_ref(), &self.ccitt_params)
        else {
            unreachable!()
        };
        let mut params = params.clone();
        if params.rows.is_none_or(|rows| rows == 0) {
            params.rows = Some(self.height);
        }
        if params.columns != self.width || params.rows != Some(self.height) {
            return Err(Error::Image("CCITT dimensions differ from image geometry".into()));
        }
        let mut decoded = crate::decoders::ccitt::decode_with_budget(pixels, &params, budget)?.data;
        if params.black_is_1 {
            for byte in &mut decoded {
                *byte = !*byte;
            }
        }
        let count = (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or_else(|| budget.reject("CCITT grayscale bytes"))?;
        budget.charge_codec_allocation(count)?;
        let mut gray = Vec::new();
        gray.try_reserve_exact(count)
            .map_err(|_| budget.reject("CCITT grayscale allocation"))?;
        let stride = (self.width as usize).div_ceil(8);
        for row in decoded.chunks_exact(stride) {
            for column in 0..self.width as usize {
                gray.push(if (row[column / 8] >> (7 - column % 8)) & 1 == 0 {
                    255
                } else {
                    0
                });
            }
        }
        image::GrayImage::from_raw(self.width, self.height, gray)
            .map(image::DynamicImage::ImageLuma8)
            .ok_or_else(|| Error::Image("invalid bounded CCITT pixel buffer".into()))
    }

    fn decode_jpeg_with_budget(
        &self,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<image::DynamicImage> {
        let ImageData::Jpeg(bytes) = self.data.as_ref() else {
            unreachable!()
        };
        if self.color_space.components() != 4 {
            return super::jpeg_limited::decode(bytes, self.width, self.height, budget);
        }
        let request = super::jpeg_cmyk_limited::Request(budget);
        let decoder = jpeg_decoder_limited::Decoder::new_with_allocation_budget(
            std::io::Cursor::new(bytes),
            &request,
        );
        budget.check()?;
        let mut decoder =
            decoder.map_err(|_| Error::Image("bounded CMYK JPEG constructor failed".into()))?;
        decoder.set_max_decoding_buffer_size(budget.limits.stream_bytes);
        let header = decoder.read_info();
        budget.check()?;
        header.map_err(|_| Error::Image("invalid JPEG header".into()))?;
        let info = decoder
            .info()
            .ok_or_else(|| Error::Image("missing JPEG dimensions".into()))?;
        if u32::from(info.width) != self.width || u32::from(info.height) != self.height {
            return Err(Error::Image("JPEG dimensions differ from PDF declaration".into()));
        }
        let _output_bytes = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|n| n.checked_mul(info.pixel_format.pixel_bytes()))
            .filter(|&n| n <= budget.limits.stream_bytes)
            .ok_or_else(|| budget.reject("JPEG decoded pixel buffer"))?;
        // The decoder now charges the actual output and intermediate reservations.
        let pixels = decoder.decode();
        budget.check()?;
        let pixels = pixels.map_err(|_| Error::Image("bounded JPEG decode failed".into()))?;
        match info.pixel_format {
            jpeg_decoder_limited::PixelFormat::L8 => {
                image::GrayImage::from_raw(self.width, self.height, pixels)
                    .map(image::DynamicImage::ImageLuma8)
            },
            jpeg_decoder_limited::PixelFormat::RGB24 => {
                image::RgbImage::from_raw(self.width, self.height, pixels)
                    .map(image::DynamicImage::ImageRgb8)
            },
            jpeg_decoder_limited::PixelFormat::CMYK32 => {
                let mut cmyk = pixels;
                if cmyk_jpeg_samples_are_inverted(bytes) {
                    for channel in &mut cmyk {
                        *channel = 255 - *channel;
                    }
                }
                Some(image::DynamicImage::ImageRgb8(self.raw_rgb_with_budget(
                    &cmyk,
                    PixelFormat::CMYK,
                    budget,
                )?))
            },
            _ => return Err(Error::Unsupported("limited import JPEG sample depth".into())),
        }
        .ok_or_else(|| Error::Image("invalid JPEG pixel buffer".into()))
    }

    /// Convert image to a base64 data URI for embedding in HTML.
    pub fn to_base64_data_uri(&self) -> Result<String> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        match self.data.as_ref() {
            ImageData::Jpeg(jpeg_data) => {
                let base64_str = STANDARD.encode(jpeg_data);
                Ok(format!("data:image/jpeg;base64,{}", base64_str))
            },
            ImageData::Raw { .. } => {
                let png_bytes = self.to_png_bytes()?;
                let base64_str = STANDARD.encode(&png_bytes);
                Ok(format!("data:image/png;base64,{}", base64_str))
            },
        }
    }

    /// Convert this PDF image to a `DynamicImage`.
    pub fn to_dynamic_image(&self) -> Result<image::DynamicImage> {
        match self.data.as_ref() {
            ImageData::Jpeg(jpeg_data) => {
                if self.color_space.components() == 4 {
                    // 4-component (DeviceCMYK / ICCBased N=4) JPEGs must go
                    // through the Adobe-aware CMYK path. image::load_from_memory
                    // routes them through zune-jpeg's own CMYK->RGB, which does
                    // not honor the APP14 inversion and yields near-black output
                    // (see decode_cmyk_jpeg_to_rgb_with_profile).
                    let transform = self.build_icc_transform();
                    let rgb =
                        decode_cmyk_jpeg_to_rgb_with_profile(jpeg_data, transform.as_deref())?;
                    return image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
                        self.width,
                        self.height,
                        rgb,
                    )
                    .ok_or_else(|| Error::Decode("Invalid CMYK image dimensions".to_string()))
                    .map(image::DynamicImage::ImageRgb8);
                }
                log::debug!(
                    "Decoding JPEG data ({} bytes), starts with: {:02X?}",
                    jpeg_data.len(),
                    &jpeg_data[..min(jpeg_data.len(), 16)]
                );
                image::load_from_memory(jpeg_data)
                    .map_err(|e| Error::Decode(format!("Failed to decode JPEG: {}", e)))
            },
            ImageData::Raw { pixels, format } => {
                if self.bits_per_component == 1
                    && matches!(self.color_space, ColorSpace::DeviceGray)
                    && self.ccitt_params.is_some()
                {
                    // Only genuinely CCITT-filtered images reach here — see
                    // `extract_image_from_xobject`, which sets `ccitt_params`
                    // solely when the XObject's `/Filter` is CCITTFaxDecode.
                    let Some(params) = self.ccitt_params.clone() else {
                        unreachable!("guarded by ccitt_params.is_some() above")
                    };

                    let decompressed = ccitt_bilevel::decompress_ccitt(pixels, &params)?;
                    let grayscale =
                        ccitt_bilevel::bilevel_to_grayscale(&decompressed, self.width, self.height);

                    image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
                        self.width,
                        self.height,
                        grayscale,
                    )
                    .ok_or_else(|| Error::Decode("Invalid image dimensions".to_string()))
                    .map(image::DynamicImage::ImageLuma8)
                } else if self.bits_per_component == 1
                    && matches!(self.color_space, ColorSpace::DeviceGray)
                {
                    // Packed 1-bit DeviceGray rows, `ceil(width / 8)` bytes
                    // each, MSB first. `extract_image_from_xobject` unpacks
                    // sub-byte images to 8-bit samples (with /Decode applied)
                    // before storing, so this branch only serves externally
                    // constructed images; it unpacks with the ISO 32000-1
                    // §8.9.5.2 Table 90 default semantics: sample bit 0 ->
                    // component 0.0 (black), bit 1 -> component 1.0 (white).
                    let row_bytes = (self.width as usize).div_ceil(8);
                    let mut grayscale =
                        Vec::with_capacity(self.width as usize * self.height as usize);
                    for row in 0..self.height as usize {
                        let row_start = row * row_bytes;
                        for col in 0..self.width as usize {
                            let byte_idx = row_start + col / 8;
                            let bit = pixels
                                .get(byte_idx)
                                .map(|b| (b >> (7 - (col % 8))) & 1)
                                .unwrap_or(1);
                            grayscale.push(if bit == 0 { 0x00 } else { 0xFF });
                        }
                    }

                    image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
                        self.width,
                        self.height,
                        grayscale,
                    )
                    .ok_or_else(|| Error::Decode("Invalid image dimensions".to_string()))
                    .map(image::DynamicImage::ImageLuma8)
                } else {
                    match (format, self.color_space) {
                        (PixelFormat::RGBA, _) => {
                            image::RgbaImage::from_raw(self.width, self.height, pixels.clone())
                                .ok_or_else(|| Error::Decode("Invalid RGBA dimensions".into()))
                                .map(image::DynamicImage::ImageRgba8)
                        },
                        (PixelFormat::RGB, ColorSpace::DeviceRGB) => {
                            image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
                                self.width,
                                self.height,
                                pixels.clone(),
                            )
                            .ok_or_else(|| Error::Decode("Invalid image dimensions".to_string()))
                            .map(image::DynamicImage::ImageRgb8)
                        },
                        (PixelFormat::Grayscale, ColorSpace::DeviceGray) => {
                            image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
                                self.width,
                                self.height,
                                pixels.clone(),
                            )
                            .ok_or_else(|| Error::Decode("Invalid image dimensions".to_string()))
                            .map(image::DynamicImage::ImageLuma8)
                        },
                        _ => {
                            let rgb_pixels = match format {
                                PixelFormat::Grayscale => {
                                    pixels.iter().flat_map(|&g| vec![g, g, g]).collect()
                                },
                                PixelFormat::CMYK => cmyk_to_rgb_with_transform(
                                    pixels,
                                    self.build_icc_transform().as_deref(),
                                ),
                                PixelFormat::RGB => pixels.clone(),
                                PixelFormat::RGBA => unreachable!("handled above"),
                            };
                            image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
                                self.width,
                                self.height,
                                rgb_pixels,
                            )
                            .ok_or_else(|| Error::Decode("Invalid image dimensions".to_string()))
                            .map(image::DynamicImage::ImageRgb8)
                        },
                    }
                }
            },
        }
    }
}

/// Image data representation.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum ImageData {
    /// JPEG-encoded image data.
    Jpeg(Vec<u8>),
    /// Raw pixel data with a specified format.
    Raw {
        /// Raw pixel bytes.
        pixels: Vec<u8>,
        /// Pixel format (RGB, RGBA, Grayscale, CMYK).
        format: PixelFormat,
    },
}

impl ImageData {
    /// Returns true if the image data is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            ImageData::Jpeg(data) => data.is_empty(),
            ImageData::Raw { pixels, .. } => pixels.is_empty(),
        }
    }
}

/// PDF color space types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ColorSpace {
    /// RGB color space (3 components).
    DeviceRGB,
    /// Grayscale color space (1 component).
    DeviceGray,
    /// CMYK color space (4 components).
    DeviceCMYK,
    /// Indexed (palette-based) color space.
    Indexed,
    /// Calibrated grayscale.
    CalGray,
    /// Calibrated RGB.
    CalRGB,
    /// CIE L*a*b* color space.
    Lab,
    /// ICC profile-based color space with N components.
    ICCBased(usize),
    /// Separation (spot color) space.
    Separation,
    /// DeviceN (multi-ink) color space.
    DeviceN,
    /// Pattern color space.
    Pattern,
}

impl ColorSpace {
    /// Returns the number of color components for this color space.
    pub fn components(&self) -> usize {
        match self {
            ColorSpace::DeviceGray => 1,
            ColorSpace::DeviceRGB => 3,
            ColorSpace::DeviceCMYK => 4,
            ColorSpace::Indexed => 1,
            ColorSpace::CalGray => 1,
            ColorSpace::CalRGB => 3,
            ColorSpace::Lab => 3,
            ColorSpace::ICCBased(n) => *n,
            ColorSpace::Separation => 1,
            ColorSpace::DeviceN => 4,
            ColorSpace::Pattern => 0,
        }
    }
}

/// Pixel format for raw image data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum PixelFormat {
    /// RGB format (3 bytes per pixel).
    RGB,
    /// RGB with straight alpha (4 bytes per pixel), after explicit masking.
    RGBA,
    /// Grayscale format (1 byte per pixel).
    Grayscale,
    /// CMYK format (4 bytes per pixel).
    CMYK,
}

impl PixelFormat {
    /// Returns the number of bytes per pixel for this format.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Grayscale => 1,
            PixelFormat::RGB => 3,
            PixelFormat::RGBA => 4,
            PixelFormat::CMYK => 4,
        }
    }
}

fn color_space_to_pixel_format(color_space: &ColorSpace) -> PixelFormat {
    match color_space {
        ColorSpace::DeviceGray => PixelFormat::Grayscale,
        ColorSpace::DeviceRGB => PixelFormat::RGB,
        ColorSpace::DeviceCMYK => PixelFormat::CMYK,
        ColorSpace::Indexed => PixelFormat::RGB,
        ColorSpace::CalGray => PixelFormat::Grayscale,
        ColorSpace::CalRGB => PixelFormat::RGB,
        ColorSpace::Lab => PixelFormat::RGB,
        ColorSpace::ICCBased(n) => match n {
            1 => PixelFormat::Grayscale,
            3 => PixelFormat::RGB,
            4 => PixelFormat::CMYK,
            _ => PixelFormat::RGB,
        },
        ColorSpace::Separation => PixelFormat::Grayscale,
        ColorSpace::DeviceN => PixelFormat::CMYK,
        ColorSpace::Pattern => PixelFormat::RGB,
    }
}

/// Parse a ColorSpace name from a PDF object.
pub fn parse_color_space(obj: &crate::object::Object) -> Result<ColorSpace> {
    use crate::object::Object;

    match obj {
        Object::Name(name) => match name.as_str() {
            "DeviceRGB" => Ok(ColorSpace::DeviceRGB),
            "DeviceGray" => Ok(ColorSpace::DeviceGray),
            "DeviceCMYK" => Ok(ColorSpace::DeviceCMYK),
            "Pattern" => Ok(ColorSpace::Pattern),
            other => Err(Error::Image(format!("Unsupported color space: {}", other))),
        },
        Object::Array(arr) if !arr.is_empty() => {
            if let Some(name) = arr[0].as_name() {
                match name {
                    "Indexed" => Ok(ColorSpace::Indexed),
                    "CalGray" => Ok(ColorSpace::CalGray),
                    "CalRGB" => Ok(ColorSpace::CalRGB),
                    "Lab" => Ok(ColorSpace::Lab),
                    "ICCBased" => {
                        let num_components = if arr.len() > 1 {
                            if let Some(stream_dict) = arr[1].as_dict() {
                                stream_dict
                                    .get("N")
                                    .and_then(|obj| match obj {
                                        Object::Integer(n) => Some(*n as usize),
                                        _ => None,
                                    })
                                    .unwrap_or(3)
                            } else {
                                3
                            }
                        } else {
                            3
                        };
                        Ok(ColorSpace::ICCBased(num_components))
                    },
                    "Separation" => Ok(ColorSpace::Separation),
                    "DeviceN" => Ok(ColorSpace::DeviceN),
                    "Pattern" => Ok(ColorSpace::Pattern),
                    other => Err(Error::Image(format!("Unsupported array color space: {}", other))),
                }
            } else {
                Err(Error::Image("Color space array must start with a name".to_string()))
            }
        },
        _ => Err(Error::Image(format!("Invalid color space object: {:?}", obj))),
    }
}

/// True when a 1-bit image's `/Decode` array is `[1 0]` (inverted) rather
/// than the DeviceGray default `[0 1]`. ISO 32000-1:2008 8.9.5.2 Table 90:
/// for a 1-bit component the default Decode maps sample 0 -> black, 1 ->
/// white; `[1 0]` reverses that (0 -> white, 1 -> black). Absent, malformed,
/// or non-inverted arrays are treated as the default (no inversion).
fn decode_array_inverts_1bpc(decode: Option<&crate::object::Object>) -> bool {
    let arr = match decode.and_then(|o| o.as_array()) {
        Some(a) if a.len() == 2 => a,
        _ => return false,
    };
    let as_num =
        |o: &crate::object::Object| o.as_integer().map(|i| i as f64).or_else(|| o.as_real());
    matches!((as_num(&arr[0]), as_num(&arr[1])), (Some(lo), Some(hi)) if lo > hi)
}

/// Per-component `(Dmin, Dmax)` pairs from a `/Decode` array, or `None` when
/// the array is absent or does not hold `2 × ncomp` numbers.
/// The number of inks a `/DeviceN` colour space names (ISO 32000-1 §8.6.6.5,
/// `[/DeviceN names alternate tint]`), or `None` for any other space.
///
/// `ColorSpace::DeviceN` is a unit variant: the parser keeps the tag and drops
/// the name array, so `components()` can only answer a flat 4. Sample geometry
/// needs the true count, and 4 is merely the most common one.
fn devicen_ink_count(cs_obj: &crate::object::Object) -> Option<usize> {
    let arr = cs_obj.as_array()?;
    if !matches!(arr.first(), Some(crate::object::Object::Name(n)) if n == "DeviceN") {
        return None;
    }
    match arr.get(1)? {
        crate::object::Object::Array(names) if !names.is_empty() => Some(names.len()),
        _ => None,
    }
}

fn decode_ranges(decode: Option<&crate::object::Object>, ncomp: usize) -> Option<Vec<(f32, f32)>> {
    let arr = decode.and_then(|o| o.as_array())?;
    if arr.len() != ncomp * 2 {
        return None;
    }
    let as_num =
        |o: &crate::object::Object| o.as_integer().map(|i| i as f64).or_else(|| o.as_real());
    let mut out = Vec::with_capacity(ncomp);
    for pair in arr.as_chunks::<2>().0.iter() {
        out.push((as_num(&pair[0])? as f32, as_num(&pair[1])? as f32));
    }
    Some(out)
}

/// Reserve owned sample/lookup storage before materializing it. This accounts
/// requested payload bytes, not allocator overhead or the entire process heap.
fn sample_buffer<T>(
    count: usize,
    budget: Option<&crate::import_limits::ImportBudget>,
) -> Result<Vec<T>> {
    if let Some(budget) = budget {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .filter(|&n| n <= budget.limits.stream_bytes)
            .ok_or_else(|| budget.reject("image sample buffer"))?;
        budget.charge_codec_allocation(bytes)?;
    }
    let mut output = Vec::new();
    output.try_reserve_exact(count).map_err(|_| match budget {
        Some(budget) => budget.reject("image sample allocation failed"),
        None => Error::Image("image sample allocation failed".into()),
    })?;
    Ok(output)
}

fn limited_decode_ranges(
    doc: Option<&crate::document::PdfDocument>,
    decode: Option<&crate::object::Object>,
    ncomp: usize,
    budget: &crate::import_limits::ImportBudget,
) -> Result<Option<Vec<(f32, f32)>>> {
    let Some(decode) = decode else {
        return Ok(None);
    };
    let decode = resolve_image_object(doc, decode)?;
    if matches!(*decode, crate::object::Object::Null) {
        return Ok(None);
    }
    let arr = decode
        .as_array()
        .filter(|arr| Some(arr.len()) == ncomp.checked_mul(2))
        .ok_or_else(|| Error::Image("invalid image Decode array".into()))?;
    let mut ranges = sample_buffer(ncomp, Some(budget))?;
    for pair in arr.chunks_exact(2) {
        let number = |value: &crate::object::Object| -> Result<f32> {
            let value = resolve_image_object(doc, value)?;
            value
                .as_integer()
                .map(|n| n as f32)
                .or_else(|| value.as_real().map(|n| n as f32))
                .filter(|n| n.is_finite())
                .ok_or_else(|| Error::Image("non-finite or nonnumeric image Decode value".into()))
        };
        ranges.push((number(&pair[0])?, number(&pair[1])?));
    }
    Ok(Some(ranges))
}

/// Interleaved image samples as one 8-bit byte per component with `/Decode`
/// applied (ISO 32000-1 §8.9.5.2): each raw sample maps through
/// `Dmin + raw · (Dmax − Dmin) / (2^bpc − 1)`, then scales to `0..=255`.
/// Sub-byte samples are unpacked from their MSB-first, byte-aligned rows, so
/// the result always holds `width × height × ncomp` bytes.
/// Returns `None` for a `/BitsPerComponent` outside the spec's {1, 2, 4, 8}
/// (Table 89) rather than trusting it as shift arithmetic: a malformed value
/// must surface as a recoverable decode error, never a panic.
fn samples_to_decoded_bytes(
    data: &[u8],
    width: u32,
    height: u32,
    ncomp: usize,
    bpc: u8,
    ranges: Option<&[(f32, f32)]>,
    budget: Option<&crate::import_limits::ImportBudget>,
) -> Result<Option<Vec<u8>>> {
    if !matches!(bpc, 1 | 2 | 4 | 8) || ncomp == 0 {
        return Ok(None);
    }
    let bpc = usize::from(bpc);
    // Sizes come from /Width and /Height, which a hostile file controls, so
    // every product is checked: a wrapped `usize` (reachable on 32-bit wasm
    // with ordinary dimensions) would under-reserve and then grow until the
    // allocator aborts, and an allocation failure aborts the process rather
    // than returning an error.
    let geometry = (width as usize).checked_mul(ncomp).and_then(|row| {
        Some((row, row.checked_mul(height as usize)?, row.checked_mul(bpc)?.div_ceil(8)))
    });
    let Some((samples_per_row, total, row_bytes)) = geometry else {
        return match budget {
            Some(budget) => Err(budget.reject("image sample geometry")),
            None => Ok(None),
        };
    };
    if let Some(budget) = budget {
        let required = row_bytes
            .checked_mul(height as usize)
            .ok_or_else(|| budget.reject("image sample geometry"))?;
        if data.len() < required {
            return Err(Error::Image("truncated image samples".into()));
        }
    }
    // A stream shorter than its declared size is padded, matching what the
    // packed path already did. Refusing it instead would drop the image back
    // to its packed bytes with `/Decode` never applied, which renders the
    // negative of the intended picture — a louder wrong answer than the
    // displaced one padding gives.
    //
    // Refusing was also what bounded this allocation, so bound it directly:
    // the same 256 MiB ceiling `expand_indexed_to_rgb_with_transform` uses.
    /// Hard cap on the unpacked output buffer (256 MiB), matching the
    /// Indexed expander. A legitimate image does not reach it; a hostile
    /// `/Width` × `/Height` does.
    const MAX_UNPACKED_OUTPUT_BYTES: usize = 256 * 1024 * 1024;
    if total > MAX_UNPACKED_OUTPUT_BYTES {
        if let Some(budget) = budget {
            return Err(budget.reject("image sample buffer"));
        }
        // Refusing here is the same fallback the short-stream case was moved
        // off: the caller keeps the packed bytes and any /Decode goes
        // unapplied, which renders the negative of the picture. Nothing in a
        // real corpus reaches this, so the trade is worth it against an
        // adversarial /Width × /Height — but say so rather than letting a
        // wrong-polarity image out silently.
        log::warn!(
            "Refusing to unpack a {total}-byte image buffer (cap {MAX_UNPACKED_OUTPUT_BYTES}); \
             samples stay packed and any /Decode is left unapplied"
        );
        return Ok(None);
    }

    // Per-component lookup instead of float arithmetic per sample: `2^bpc`
    // entries per component, so the inner loop is one table read. The
    // identity mapping stays exact (bpc 1/2/4/8 -> ×255/×85/×17/×1).
    let levels = 1usize << bpc;
    let max = (levels - 1) as f32;
    let mut lut: Vec<[u8; 256]> = sample_buffer(ncomp, budget)?;
    for comp in 0..ncomp {
        let (lo, hi) = ranges.map_or((0.0, 1.0), |r| r[comp]);
        let mut table = [0u8; 256];
        for (raw, slot) in table.iter_mut().enumerate().take(levels) {
            let v = lo + (raw as f32) * (hi - lo) / max;
            *slot = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        lut.push(table);
    }

    if bpc == 8 {
        // Only the declared rows are image samples. Keep the legacy extractor's
        // extra-byte behavior, but do not amplify unrelated stream suffixes in
        // the limited path or fabricate missing samples.
        let data = if budget.is_some() {
            &data[..total]
        } else {
            data
        };
        let mut output = sample_buffer(data.len(), budget)?;
        output.extend(
            data.iter()
                .enumerate()
                .map(|(i, &b)| lut[i % ncomp][b as usize]),
        );
        return Ok(Some(output));
    }
    let mask = (1u32 << bpc) - 1;
    let mut out = sample_buffer(total, budget)?;
    for row in 0..height as usize {
        let row_start = row * row_bytes;
        for s in 0..samples_per_row {
            let bit_offset = s * bpc;
            // Past the end of a short stream the sample reads as zero, which
            // is what the packed path produced before unpacking existed.
            let byte = data.get(row_start + bit_offset / 8).copied().unwrap_or(0);
            // Cannot underflow: bpc < 8 here (the bpc == 8 case returned
            // above) and `bit_offset % 8` is at most 8 - bpc for those depths.
            let shift = 8 - bpc - (bit_offset % 8);
            let raw = ((byte >> shift) as u32) & mask;
            out.push(lut[s % ncomp][raw as usize]);
        }
    }
    Ok(Some(out))
}

/// Extract an image from an XObject stream.
pub fn extract_image_from_xobject(
    doc: Option<&crate::document::PdfDocument>,
    xobject: &crate::object::Object,
    obj_ref: Option<ObjectRef>,
    color_space_map: Option<&std::collections::HashMap<String, crate::object::Object>>,
) -> Result<PdfImage> {
    extract_image_from_xobject_at(doc, xobject, obj_ref, color_space_map, None)
}

/// Extract an image XObject, decoding no larger than `target` where the format
/// allows it.
///
/// JPEG 2000 stores successive resolution levels, so a caller that already
/// knows the image will be painted smaller than its stored size can have the
/// decoder stop early instead of producing full-resolution samples that are
/// immediately resampled away. Images "shall be mapped to the unit square in
/// user space (as are all images)" (`docs/spec/pdf.md`:23985) "regardless of
/// the number of samples in the image" (pdf.md:8475) — the stored sample count
/// does not determine the painted size, so decoding past what is painted buys
/// nothing.
///
/// It matters at the extremes: a real 12608 x 16806 JPX image peaks at 11.3 GB
/// decoded in full, which no browser tab or phone survives, to be painted into
/// a fraction of that.
///
/// `target` is a hint. Formats that cannot decode progressively ignore it, and
/// even JPEG 2000 returns the smallest resolution level that still *covers*
/// the target, so the result is never smaller than asked for. The returned
/// [`PdfImage`] always reports the geometry of the samples it actually holds,
/// which for a reduced decode is below the `/Width` and `/Height` of the
/// dictionary.
pub fn extract_image_from_xobject_at(
    doc: Option<&crate::document::PdfDocument>,
    xobject: &crate::object::Object,
    obj_ref: Option<ObjectRef>,
    color_space_map: Option<&std::collections::HashMap<String, crate::object::Object>>,
    target: Option<(u32, u32)>,
) -> Result<PdfImage> {
    use crate::object::Object;

    // Set only by the JPX branch, which is the one format here that can honour
    // `target`; `None` means the samples match the dictionary's dimensions.
    let mut jpx_decoded_dims: Option<(u32, u32)> = None;

    let dict = xobject
        .as_dict()
        .ok_or_else(|| Error::Image("XObject is not a stream".to_string()))?;

    let subtype = dict
        .get("Subtype")
        .and_then(|obj| obj.as_name())
        .ok_or_else(|| Error::Image("XObject missing /Subtype".to_string()))?;

    if subtype != "Image" {
        return Err(Error::Image(format!("XObject subtype is not Image: {}", subtype)));
    }

    // /Width and /Height may be indirect references (ISO 32000-1 §7.3.10
    // permits any object entry to be an indirect reference); resolve them
    // the same way the /ColorSpace resolution just below does.
    let resolve_int = |obj: &Object| -> Option<i64> {
        if let (Some(d), Some(r)) = (doc, obj.as_reference()) {
            d.load_object(r).ok().and_then(|o| o.as_integer())
        } else {
            obj.as_integer()
        }
    };

    // A dimension is validated, not cast. ISO 32000-1:2008 §8.9.5.1 mandates
    // the image-to-user matrix `[1/w 0 0 -1/h 0 1]`, which is undefined at
    // zero, and Table 89 requires both entries to be positive integers — so a
    // negative or out-of-range value is invalid, not a number to truncate.
    // `as u32` turned `-1` into 4294967295 and `2^32` into 0, both of which
    // then flowed into allocation and sampling arithmetic; the stencil path's
    // `image_mask_layout` already rejects the same shapes with `try_from`.
    let dimension = |key: &str| -> Result<u32> {
        let value = dict
            .get(key)
            .and_then(resolve_int)
            .ok_or_else(|| Error::Image(format!("Image missing /{key}")))?;
        let dimension = u32::try_from(value)
            .map_err(|_| Error::Image(format!("Image /{key} must be a positive integer")))?;
        if dimension == 0 {
            return Err(Error::Image(format!("Image /{key} must be a positive integer")));
        }
        Ok(dimension)
    };

    let width = dimension("Width")?;
    let height = dimension("Height")?;
    // Enumeration and materialization consume the same resolved filter chain.
    let filter_chain = parse_filter_chain(dict, doc)?;
    let has_dct = filter_chain.contains(&PdfFilter::DCTDecode);
    let is_jpeg_only = has_dct && filter_chain.len() == 1;
    let is_jpeg_chain = has_dct && filter_chain.len() > 1;
    let is_jbig2 = filter_chain.contains(&PdfFilter::JBIG2Decode);
    let is_jpx = filter_chain.contains(&PdfFilter::JPXDecode);
    let is_ccitt = filter_chain.contains(&PdfFilter::CCITTFaxDecode);

    // Match the actual storage branch below. JPEG keeps encoded bytes here;
    // its RGB/CMYK decoder, conversions and PNG writer reserve real buffers.
    // A fictitious eight-byte plane is not an individual JPEG allocation.
    let encoded_jpeg = (is_jpeg_only || is_jpeg_chain) && !is_jbig2 && !is_jpx;
    if let Some(budget) = doc.and_then(|doc| doc.import_budget()) {
        budget.check()?;
        if !encoded_jpeg {
            // Preserve the existing conservative preflight for other codecs.
            // This is not an allocation and is not charged as decoded work.
            let _worst_buffer = (width as usize)
                .checked_mul(height as usize)
                .and_then(|n| n.checked_mul(8))
                .filter(|&n| n > 0 && n <= budget.limits.stream_bytes)
                .ok_or_else(|| budget.reject("image extraction buffer"))?;
        }
    }

    let sample_depth = dict
        .get("BitsPerComponent")
        .and_then(resolve_int)
        .unwrap_or(8);
    let sample_budget = doc
        .and_then(|doc| doc.import_budget())
        .map(|budget| &**budget);
    if sample_budget.is_some() && !matches!(sample_depth, 1 | 2 | 4 | 8 | 16) {
        return Err(Error::Image("invalid image BitsPerComponent".into()));
    }
    let bits_per_component = sample_depth as u8;

    // ISO 32000-1:2008 Table 89, /ColorSpace: "Required for images, except
    // those that use the JPXDecode filter … If ColorSpace is absent, the
    // colour space specifications in the JPEG2000 data shall be used."
    //
    // So an absent /ColorSpace is legal on a JPX image, and erroring on it
    // rejected a valid file outright: a page whose only content was one such
    // image rendered completely blank, where MuPDF, pdfium, poppler and
    // Ghostscript all paint it and agree on its tone to within 0.81 of a grey
    // level. `decode_jpx_image` already ignores this value and derives the
    // pixel format from the codestream's own component count, so the
    // placeholder below is never read for that path — it exists only to
    // satisfy the shared prologue.
    let jpx_without_color_space = dict.get("ColorSpace").is_none() && is_jpx;
    let device_rgb = Object::Name("DeviceRGB".to_string());
    let color_space_obj = match dict.get("ColorSpace") {
        Some(cs) => cs,
        None if jpx_without_color_space => &device_rgb,
        None => return Err(Error::Image("Image missing /ColorSpace".to_string())),
    };

    let resolved_color_space = resolve_image_object(doc, color_space_obj)?;
    let resolved_color_space = if doc.is_some() {
        if let Object::Name(ref name) = *resolved_color_space {
            if let Some(map) = color_space_map {
                match map.get(name) {
                    Some(object) => resolve_image_object(doc, object)?,
                    None => resolved_color_space,
                }
            } else {
                resolved_color_space
            }
        } else {
            resolved_color_space
        }
    } else {
        resolved_color_space
    };

    // Preserve identity before the component-count resolver replaces the
    // reference with a Stream. The request cache must see the original key.
    let source_icc_reference = resolved_color_space
        .as_array()
        .filter(|a| a.first().and_then(Object::as_name) == Some("ICCBased"))
        .and_then(|a| a.get(1))
        .and_then(Object::as_reference);

    // For array-form color spaces (e.g. [/ICCBased <ref>], [/Indexed <base> <hi> <palette_ref>])
    // the second element is commonly an indirect reference to the ICC profile
    // stream / palette. `parse_color_space` only inspects the immediate
    // `Object::Stream` dict, so an unresolved reference silently falls back to
    // `N = 3` and a CMYK (N = 4) image is labelled as RGB. Resolve the stream
    // reference here so the component count reflects the real profile.
    let resolved_color_space =
        if let (Some(doc_mut), Object::Array(arr)) = (doc, &*resolved_color_space) {
            // Indexed resolves its base separately; do not clone its lookup
            // string just to dereference that base a second time here.
            if arr.len() > 1 && arr[0].as_name() != Some("Indexed") {
                if let Some(second_ref) = arr[1].as_reference() {
                    if let Ok(resolved_second) = doc_mut.load_object(second_ref) {
                        let mut new_arr = arr.clone();
                        new_arr[1] = resolved_second;
                        Cow::Owned(Object::Array(new_arr))
                    } else {
                        resolved_color_space
                    }
                } else {
                    resolved_color_space
                }
            } else {
                resolved_color_space
            }
        } else {
            resolved_color_space
        };

    let color_space = parse_color_space(&resolved_color_space)?;
    // For Indexed color spaces, resolve the base color space and palette now so we
    // can expand indices to RGB after decoding the stream. Without this, raw
    // Indexed pixel data (1 byte per pixel) is mislabelled as RGB (3 bytes per
    // pixel) and ImageBuffer::from_raw rejects the wrong length. Fail fast if
    // the palette cannot be resolved so the error points at the real root cause
    // instead of the downstream "Invalid RGB image dimensions" symptom.
    let indexed_resolution: Option<IndexedResolution> = if color_space == ColorSpace::Indexed {
        let resolved = resolve_indexed_palette(doc, &resolved_color_space)?;
        if resolved.is_none() {
            return Err(Error::Image("Unable to resolve Indexed color space palette".to_string()));
        }
        resolved
    } else {
        None
    };

    // For a plain (non-Indexed) `[/ICCBased <stream>]` colour space,
    // capture the profile bytes so the CMM can convert through the
    // document's actual source characterisation instead of the
    // §10.3.5 additive-clamp fallback.
    //
    // When the image uses plain `/DeviceCMYK` with no ICC profile of
    // its own, fall back to the document's `/OutputIntents` CMYK
    // profile if one exists — the standard PDF/X assumption per
    // ISO 32000-1:2008 §14.11.5.
    let direct_icc_profile = if matches!(color_space, ColorSpace::ICCBased(_)) {
        match (doc, source_icc_reference) {
            (Some(doc), Some(reference)) => doc.import_icc_profile(reference),
            _ => resolve_icc_profile_from_obj(doc, &resolved_color_space),
        }
    } else if color_space == ColorSpace::DeviceCMYK {
        doc.and_then(|d| d.output_intent_cmyk_profile())
    } else {
        None
    };

    // Per §8.6.5.8, an image dictionary may override the graphics-state
    // rendering intent via `/Intent`. Unrecognised names fall through
    // to `RelativeColorimetric`.
    let rendering_intent = dict
        .get("Intent")
        .and_then(|obj| obj.as_name())
        .map(crate::color::RenderingIntent::from_pdf_name)
        .unwrap_or_default();

    // Bit depth of the buffer actually stored: raised to 8 when a transform
    // below (sub-byte unpack, /Decode application, 16-bit reduction) rewrites
    // the samples as one byte per component.
    let mut stored_bpc = bits_per_component;
    // Cleared when a transform below moves the stored samples out of the raw
    // sample space, so consumers that range-test them against dictionary
    // values (colour-key /Mask) can tell.
    let mut samples_are_raw = true;
    // Set only when a non-default /Decode is actually folded into the stored
    // samples. Tracked apart from `samples_are_raw` because most transforms
    // below leave the raw space without applying /Decode, and a consumer that
    // applies it itself (plate routing) must still do so for those.
    let mut decode_folded_in = false;
    let data = if is_jbig2 {
        let decoded = decode_jbig2_image(xobject, obj_ref, dict, doc, width, height)?;
        // The JBIG2 decoder expands straight to 8-bit gray, bypassing the
        // packed-sample block below that is the only place /Decode is
        // otherwise honoured. ISO 32000-1 8.9.5.2 Table 90 applies to a JBIG2
        // image like any other, and Table 145 explicitly permits /Decode on a
        // soft-mask image -- 11.6.5.3 requires the alpha be derived with the
        // Decode transformation already performed. Scanners in the wild write
        // /Decode [1 0] to normalise the polarity their encoder emitted, so
        // dropping it inverts the mask exactly.
        if decode_array_inverts_1bpc(dict.get("Decode")) {
            decode_folded_in = true;
            match decoded {
                ImageData::Raw { mut pixels, format } => {
                    for b in &mut pixels {
                        *b = !*b;
                    }
                    ImageData::Raw { pixels, format }
                },
                other => other,
            }
        } else {
            decoded
        }
    } else if is_jpx {
        // Palette lookup replaces index samples with RGB, as it does on the
        // raw-sample path below; the buffer then no longer holds the values
        // the dictionary's entries describe.
        let indexed_transform = indexed_resolution.as_ref().and_then(|ir| {
            ir.base_profile
                .clone()
                .map(|p| crate::color::IccProfile::cached_transform(&p, rendering_intent))
        });
        let indexed = indexed_resolution
            .as_ref()
            .map(|ir| (ir, indexed_transform.as_deref()));
        if indexed.is_some() {
            samples_are_raw = false;
        }
        let direct_transform = direct_icc_profile
            .as_ref()
            .map(|p| crate::color::IccProfile::cached_transform(p, rendering_intent));
        let (data, dec_w, dec_h) = decode_jpx_image(
            xobject,
            obj_ref,
            doc,
            &color_space,
            indexed,
            direct_transform.as_deref(),
            target,
        )?;
        if bits_per_component != 8 {
            samples_are_raw = false;
        }
        jpx_decoded_dims = Some((dec_w, dec_h));
        data
    } else if encoded_jpeg {
        let decoded = if let (Some(d), Some(ref_id)) = (doc.as_ref(), obj_ref) {
            d.decode_stream_with_encryption(xobject, ref_id)?
        } else {
            xobject.decode_stream_data()?
        };
        ImageData::Jpeg(decoded)
    } else {
        let decoded_data = if let (Some(d), Some(ref_id)) = (doc.as_ref(), obj_ref) {
            d.decode_stream_with_encryption(xobject, ref_id)?
        } else {
            xobject.decode_stream_data()?
        };
        if let Some(ir) = indexed_resolution.as_ref() {
            let budget = doc.and_then(|doc| doc.import_budget()).map(|b| &**b);
            let decoded_data = if let Some(budget) = budget.filter(|_| is_ccitt) {
                if bits_per_component != 1 {
                    return Err(Error::Image("CCITT requires one-bit image samples".into()));
                }
                let index = filter_chain
                    .iter()
                    .position(|name| *name == PdfFilter::CCITTFaxDecode)
                    .unwrap();
                let params = limited_ccitt_parameters(
                    doc.unwrap(),
                    dict.get("DecodeParms"),
                    index,
                    width,
                    height,
                )?;
                let mut packed =
                    crate::decoders::ccitt::decode_with_budget(&decoded_data, &params, budget)?
                        .data;
                // The decoder returns black=1. PDF's default BlackIs1=false
                // means black=0; /Decode is a separate index mapping below.
                if !params.black_is_1 {
                    for byte in &mut packed {
                        *byte = !*byte;
                    }
                }
                packed
            } else {
                decoded_data
            };
            // Build a Transform if the Indexed base has a profile so
            // palette entries render through the real CMM (when linked).
            let transform = ir
                .base_profile
                .clone()
                .map(|p| crate::color::IccProfile::cached_transform(&p, rendering_intent));
            let indexed_decode = if budget.is_some() {
                limited_indexed_decode(doc, dict.get("Decode"))?
            } else {
                None
            };
            let expanded = expand_indexed_to_rgb_with_context(
                &decoded_data,
                &ir.palette,
                ir.base_fmt,
                width,
                height,
                bits_per_component,
                IndexedDecodeContext {
                    transform: transform.as_deref(),
                    budget,
                    decode: indexed_decode,
                    preblended_opacity: None,
                },
            )?;
            // Palette lookup replaces index samples with RGB, so the buffer
            // is no longer in the space the dictionary's entries describe.
            samples_are_raw = false;
            if budget.is_some() {
                stored_bpc = 8;
                // The expander validated bpc before the shift. Consumers must
                // know that a non-default Decode is already materialized.
                let default_decode = (0.0, ((1u32 << bits_per_component) - 1) as f64);
                decode_folded_in = indexed_decode.is_some_and(|range| range != default_decode);
            }
            ImageData::Raw {
                pixels: expanded,
                format: PixelFormat::RGB,
            }
        } else {
            let pixel_format = color_space_to_pixel_format(&color_space);
            let ncomp = devicen_ink_count(&resolved_color_space)
                .unwrap_or_else(|| color_space.components());
            // ISO 32000-1 §8.9.5.2: BitsPerComponent 16 stores each colour
            // sample as a big-endian 16-bit value. The rest of the image
            // pipeline (and the render target, an 8-bit tiny_skia pixmap)
            // assumes 8-bit samples, so reduce each sample to 8 bits. Without
            // this the raw buffer is twice the expected length; the lenient
            // `ImageBuffer::from_raw` then builds an oversized image whose
            // buffer the PNG encoder rejects with a panic (`assertion
            // left == right failed: Invalid buffer length`).
            //
            // The reduction rounds `v * 255 / 65535` to the nearest 8-bit
            // value rather than dropping the low byte (`v >> 8`, i.e. floor):
            // truncation biases every sample downward by up to ~1 LSB, most
            // visibly darkening near-white highlights. Full u16 precision
            // through extraction is deferred (v0.3.72) — no current consumer
            // benefits, as both the PNG path and the rasteriser are 8-bit.
            let reduced: Vec<u8> = if bits_per_component == 16 {
                // Rescaling 0..65535 to 0..255 leaves the raw sample space
                // exactly as the sub-byte unpack does. The flag clears at the
                // `effective_bpc` check below, which owns the stored-versus-
                // declared depth rule for every codec. A colour-key /Mask
                // states its bounds in the file's 0..65535 space and must not
                // be range-tested against these bytes.
                let samples = if let Some(budget) = sample_budget {
                    let required = (width as usize)
                        .checked_mul(height as usize)
                        .and_then(|n| n.checked_mul(ncomp))
                        .and_then(|n| n.checked_mul(2))
                        .ok_or_else(|| budget.reject("image sample geometry"))?;
                    decoded_data
                        .get(..required)
                        .ok_or_else(|| Error::Image("truncated 16-bit image samples".into()))?
                } else {
                    &decoded_data
                };
                let mut reduced = sample_buffer(samples.len() / 2, sample_budget)?;
                reduced.extend(
                    samples
                        .chunks_exact(2)
                        .map(|sample| reduce_16_to_8(sample[0], sample[1])),
                );
                reduced
            } else {
                decoded_data
            };
            let bpc_after_reduce = if bits_per_component == 16 {
                8
            } else {
                bits_per_component
            };

            // `ColorSpace::DeviceN` does not carry its ink count — the parser
            // discards the name array and `components()` answers a flat 4 — so
            // read the real count here. It sets the unpacker's row geometry:
            // with the wrong value every row starts at the wrong offset, and a
            // stream sized for its true ink count runs out partway down, so
            // the tail of the image is padded with fabricated zeros.
            // The stored buffer is one byte per component at the stride
            // `PixelFormat` declares, and that stride is only ever 1, 3 or 4.
            // A colour space whose component count is anything else — a
            // 2- or 6-ink `/DeviceN`, an `/ICCBased` with N ∉ {1,3,4} — cannot
            // be expressed in it, so unpacking would have to write a buffer
            // whose length contradicts its own format. Leave the stream packed
            // instead: `bits_per_component` stays sub-byte, which is exactly
            // what every consumer already checks before reading samples.
            let representable = ncomp == pixel_format.bytes_per_pixel();
            // CCITT keeps its packed stream — decompression happens lazily
            // and /Decode is folded into `black_is_1` below. Lab samples are
            // not confined to [0, 1], so the linear-to-byte mapping does not
            // apply to them.
            let keep_raw = is_ccitt || color_space == ColorSpace::Lab || !representable;
            let ranges = if let Some(budget) = sample_budget {
                limited_decode_ranges(doc, dict.get("Decode"), ncomp, budget)?
            } else {
                decode_ranges(dict.get("Decode"), ncomp)
            }
            .filter(|r| r.iter().any(|&(lo, hi)| (lo, hi) != (0.0, 1.0)));

            // The stored buffer contract is one 8-bit byte per component
            // (`PixelFormat::bytes_per_pixel`), so sub-byte samples are
            // unpacked here and a non-default /Decode is applied at
            // 1/2/4/8 bpc on this raw-sample path (ISO 32000-1 §8.9.5.2).
            // DCT- and JPX-coded images keep their encoded stream and do
            // not pass through here, so /Decode is not applied to them.
            // Distinguish "did not attempt the unpack" from "attempted it and
            // it was refused" — the None arm below must only act on the
            // second. A CCITT buffer deliberately stays raw with its /Decode
            // polarity carried in `ccitt_params`, so touching it here would
            // apply that mapping twice.
            let attempted_unpack = !(keep_raw || (bpc_after_reduce == 8 && ranges.is_none()));
            let unpacked = if !attempted_unpack {
                None
            } else {
                samples_to_decoded_bytes(
                    &reduced,
                    width,
                    height,
                    ncomp,
                    bpc_after_reduce,
                    ranges.as_deref(),
                    sample_budget,
                )?
            };
            // A `/BitsPerComponent` the spec does not define leaves the
            // stream untouched, so the existing length checks downstream
            // reject it as they did before.
            let pixels = match unpacked {
                Some(samples) => {
                    stored_bpc = 8;
                    // This arm runs only when the buffer was rewritten — the
                    // identity case (8 bpc, no /Decode) took the `None` branch
                    // above — so the samples have left the raw space either by
                    // the ×255/×85/×17 sub-byte rescale or by /Decode itself.
                    // Colour-key /Mask (§8.9.6.4) range-tests against bounds
                    // stated in the original 0..2^bpc−1 space and reads this.
                    samples_are_raw = false;
                    // Plate routing applies /Decode itself, so it needs the
                    // narrower fact: only `ranges` actually folds one in.
                    decode_folded_in = ranges.is_some();
                    samples
                },
                None => {
                    // The unpack was refused — over the size cap, or a
                    // /BitsPerComponent the spec does not define — so the
                    // buffer stays packed and /Decode goes unapplied. For the
                    // one mapping where that is catastrophic rather than
                    // merely approximate, apply it here instead.
                    //
                    // A per-component `[1 0]` at 1 bpc is a pure inversion: on
                    // packed samples it is a byte-wise NOT, needing neither
                    // unpacking nor allocation. Leaving it unapplied renders
                    // the exact negative of the picture, which is what a large
                    // 1-bpc scan with /Decode [1 0] became once it crossed the
                    // cap — the byte-wise NOT handled it at any size before the
                    // unpacking path existed.
                    let mut reduced = reduced;
                    let inverts = ranges
                        .as_deref()
                        .is_some_and(|r| r.iter().all(|&(lo, hi)| lo == 1.0 && hi == 0.0));
                    if attempted_unpack && bpc_after_reduce == 1 && inverts {
                        for byte in &mut reduced {
                            *byte = !*byte;
                        }
                        // Values have left the raw sample space, and /Decode is
                        // now folded in — both facts consumers gate on.
                        samples_are_raw = false;
                        decode_folded_in = true;
                    }
                    reduced
                },
            };
            ImageData::Raw {
                pixels,
                format: pixel_format,
            }
        }
    };

    // JBIG2 decode produces 8-bit-per-channel pixels regardless of the
    // XObject's BitsPerComponent (which is 1).  Override to 8 so that
    // to_dynamic_image() does not try to CCITT-decompress the output.
    // 16-bit samples were collapsed to their high byte above (and an Indexed
    // base always expands to 8-bit RGB), so the stored pixels are 8-bit per
    // component in those cases too.
    let effective_bpc = if is_jbig2 || is_jpx || bits_per_component == 16 {
        8
    } else {
        stored_bpc
    };
    // A stored depth that no longer matches the declared /BitsPerComponent
    // means the samples left the space the dictionary describes: the 16-bit
    // reduce and the JBIG2 decoder's 8-bit output both land here. Cleared
    // centrally so a codec expansion cannot forget the flag — the JBIG2
    // branch did, and reported raw samples at 8 bpc against a declared 1.
    if i64::from(effective_bpc) != i64::from(bits_per_component) {
        samples_are_raw = false;
    }
    // A reduced-resolution JPX decode holds fewer samples than the dictionary
    // advertises; the buffer's geometry is what the samples actually are.
    let (width, height) = jpx_decoded_dims.unwrap_or((width, height));

    let color_space = if jpx_without_color_space {
        match &data {
            ImageData::Raw {
                format: PixelFormat::Grayscale,
                ..
            } => ColorSpace::DeviceGray,
            ImageData::Raw {
                format: PixelFormat::CMYK,
                ..
            } => ColorSpace::DeviceCMYK,
            _ => ColorSpace::DeviceRGB,
        }
    } else {
        color_space
    };
    let mut image = PdfImage::new(width, height, color_space, effective_bpc, data);
    image.set_samples_are_raw(samples_are_raw);
    image.set_decode_folded_in(decode_folded_in);

    // Attach the ICC profile if we found one — prefer the direct ICCBased
    // profile, then fall back to an Indexed base's profile so the CMM has
    // something to work with for palette-backed CMYK/Lab images too.
    if let Some(p) = direct_icc_profile {
        image.set_icc_profile(p);
    }
    if is_jpx
        && matches!(
            image.data.as_ref(),
            ImageData::Raw {
                format: PixelFormat::RGBA,
                ..
            }
        )
    {
        image.icc_profile = None; // JPX embedded alpha is converted before RGBA expansion.
    }
    image.set_rendering_intent(rendering_intent);

    if bits_per_component == 1 && image.color_space == ColorSpace::DeviceGray && is_ccitt {
        // §7.3.10: any object may be written as an indirect reference, and
        // `/DecodeParms` routinely is. `extract_ccitt_params_with_width` reads
        // a dictionary or an array and returns None for a reference, so an
        // unresolved one left `ccitt_params` unset — and with it unset,
        // `to_dynamic_image` skips CCITT decompression entirely and unpacks
        // the still-compressed bytes as though they were packed pixels. A
        // 221-byte codestream standing in for 12,341 bytes of 344x287 image
        // meant everything past the first ~1.8% of the page fell out of
        // bounds and defaulted to white: the page rendered nearly blank at
        // coverage 0.00988, where the ink derivable from the file is 0.07980
        // and all four reference renderers report 0.08004-0.08297.
        let params = if let Some(doc) = doc.filter(|doc| doc.import_budget().is_some()) {
            let index = filter_chain
                .iter()
                .position(|name| *name == PdfFilter::CCITTFaxDecode)
                .unwrap();
            Some(limited_ccitt_parameters(doc, dict.get("DecodeParms"), index, width, height)?)
        } else {
            let decode_parms = dict.get("DecodeParms").and_then(|o| match o {
                Object::Reference(_) => doc.and_then(|d| d.resolve_object(o).ok()),
                other => Some(other.clone()),
            });
            crate::object::extract_ccitt_params_with_width(decode_parms.as_ref(), Some(width))
        };
        if let Some(mut ccitt_params) = params {
            if ccitt_params.rows.is_none() {
                ccitt_params.rows = Some(height);
            }
            // ISO 32000-1 7.4.6 /BlackIs1 and 8.9.5.2 Table 90 /Decode both
            // flip which sample bit means "black" for a 1-bit DeviceGray
            // image, and they are independent, composable mechanisms: a
            // producer that wants inverted polarity may set /BlackIs1 true
            // *or* write /Decode [1 0] on the image XObject instead (both
            // are seen in real-world scanned PDFs). Fold /Decode into the
            // same inversion flag decompress_ccitt already honors so either
            // one alone inverts and both together cancel out.
            if decode_array_inverts_1bpc(dict.get("Decode")) {
                ccitt_params.black_is_1 = !ccitt_params.black_is_1;
            }
            image.set_ccitt_params(ccitt_params);
        }
    }

    Ok(image)
}

fn resolve_image_object<'a>(
    doc: Option<&crate::document::PdfDocument>,
    object: &'a crate::object::Object,
) -> Result<Cow<'a, crate::object::Object>> {
    match (doc, object.as_reference()) {
        (Some(doc), Some(_)) => Ok(Cow::Owned(doc.resolve_object(object)?)),
        _ => Ok(Cow::Borrowed(object)),
    }
}

fn limited_indexed_decode(
    doc: Option<&crate::document::PdfDocument>,
    object: Option<&crate::object::Object>,
) -> Result<Option<(f64, f64)>> {
    let Some(object) = object else {
        return Ok(None);
    };
    let object = resolve_image_object(doc, object)?;
    let array = object
        .as_array()
        .filter(|array| array.len() == 2)
        .ok_or_else(|| Error::Image("invalid Indexed Decode array".into()))?;
    let number = |value: &crate::object::Object| {
        value
            .as_integer()
            .map(|n| n as f64)
            .or_else(|| value.as_real())
            .filter(|n| n.is_finite())
            .ok_or_else(|| Error::Image("invalid Indexed Decode value".into()))
    };
    Ok(Some((number(&array[0])?, number(&array[1])?)))
}

fn limited_ccitt_parameters(
    doc: &crate::document::PdfDocument,
    object: Option<&crate::object::Object>,
    filter_index: usize,
    width: u32,
    height: u32,
) -> Result<crate::decoders::CcittParams> {
    use crate::object::Object;
    let resolved = match object {
        Some(object @ Object::Reference(_)) => Some(doc.resolve_object(object)?),
        _ => None,
    };
    let object = resolved.as_ref().or(object);
    let object = match object {
        Some(Object::Array(array)) => array.get(filter_index),
        _ => object,
    };
    let resolved_entry = match object {
        Some(object @ Object::Reference(_)) => Some(doc.resolve_object(object)?),
        _ => None,
    };
    let dict = match resolved_entry.as_ref().or(object) {
        None | Some(Object::Null) => None,
        Some(Object::Dictionary(dict)) => Some(dict),
        _ => return Err(Error::Image("invalid CCITT DecodeParms".into())),
    };
    let integer = |key: &str, default: i64| -> Result<i64> {
        match dict.and_then(|dict| dict.get(key)) {
            None => Ok(default),
            Some(Object::Integer(value)) => Ok(*value),
            _ => Err(Error::Image(format!("invalid CCITT {key}"))),
        }
    };
    let boolean = |key: &str, default: bool| -> Result<bool> {
        match dict.and_then(|dict| dict.get(key)) {
            None => Ok(default),
            Some(Object::Boolean(value)) => Ok(*value),
            _ => Err(Error::Image(format!("invalid CCITT {key}"))),
        }
    };
    let columns = u32::try_from(integer("Columns", i64::from(width))?)
        .map_err(|_| Error::Image("invalid CCITT Columns".into()))?;
    let rows = u32::try_from(integer("Rows", 0)?)
        .map_err(|_| Error::Image("invalid CCITT Rows".into()))?;
    let rows = if rows == 0 { height } else { rows };
    if columns != width || rows != height {
        return Err(Error::Image("CCITT dimensions differ from image geometry".into()));
    }
    Ok(crate::decoders::CcittParams {
        k: integer("K", 0)?,
        columns,
        rows: Some(rows),
        black_is_1: boolean("BlackIs1", false)?,
        end_of_line: boolean("EndOfLine", false)?,
        encoded_byte_align: boolean("EncodedByteAlign", false)?,
        end_of_block: boolean("EndOfBlock", true)?,
    })
}

/// Extract and parse an `ICCBased` colour-space's profile stream.
///
/// Accepts either a fully-resolved `[/ICCBased <Stream>]` array (the
/// stream is an `Object::Stream` directly), or a `[/ICCBased <Ref>]`
/// array where the second element is a live reference — in that case
/// `doc` must be supplied so we can dereference.
///
/// Returns `None` if:
///   - `cs_obj` isn't an ICCBased array,
///   - the profile stream can't be decoded,
///   - the profile bytes fail ICC header validation, or
///   - the declared `/N` disagrees with the profile header's
///     colourSpace signature (PDF §8.6.5.5 mandates they match).
///
/// No error is returned — callers treat "no profile" as "fall back to
/// device colour space" per §8.6.5.5's /Alternate clause.
pub(crate) fn resolve_icc_profile_from_obj(
    doc: Option<&crate::document::PdfDocument>,
    cs_obj: &crate::object::Object,
) -> Option<std::sync::Arc<crate::color::IccProfile>> {
    use crate::object::Object;

    let Object::Array(arr) = cs_obj else {
        return None;
    };
    if arr.len() < 2 || arr[0].as_name() != Some("ICCBased") {
        return None;
    }

    if let (Object::Reference(reference), Some(doc)) = (&arr[1], doc) {
        return doc.import_icc_profile(*reference);
    }

    // Second element should be a stream (already resolved by the caller
    // in the common path) or a reference we still need to dereference.
    let profile_obj = match (&arr[1], doc) {
        (Object::Stream { .. }, _) => arr[1].clone(),
        (Object::Reference(r), Some(d)) => match d.load_object(*r) {
            Ok(obj) => obj,
            Err(_) => return None,
        },
        _ => return None,
    };

    let Object::Stream { dict, .. } = &profile_obj else {
        return None;
    };
    // `N` is mandatory per PDF 32000-1 §8.6.5.5 Table 66.
    let n = dict
        .get("N")
        .and_then(|obj| obj.as_integer())
        .filter(|n| matches!(*n, 1 | 3 | 4))? as u8;

    let bytes = match doc {
        Some(doc) if doc.import_budget().is_some() => {
            doc.decode_stream_with_encryption(&profile_obj, crate::object::ObjectRef::new(0, 0))
        },
        _ => profile_obj.decode_stream_data(),
    }
    .ok()?;
    let mut profile = crate::color::IccProfile::parse(bytes, n)?;
    profile.bind_budget(doc.and_then(|d| d.import_budget()));
    Some(std::sync::Arc::new(profile))
}

/// Outcome of resolving an `[/Indexed base hival lookup]` colour space:
/// the palette in the base's pixel format, plus the base's ICC profile
/// when the base is `ICCBased`.
pub(crate) struct IndexedResolution {
    pub base_fmt: PixelFormat,
    pub palette: Vec<u8>,
    /// False when palette samples were already colour-converted (currently Lab).
    /// Such samples cannot be unblended in their original component ranges.
    pub retains_source_samples: bool,
    /// `None` for device-dependent bases or bases we already folded
    /// colourimetrically (e.g. Lab, whose palette is rewritten to RGB
    /// before being returned).
    pub base_profile: Option<std::sync::Arc<crate::color::IccProfile>>,
}

/// Resolve an Indexed color space's base color space and palette lookup bytes.
///
/// PDF Indexed color spaces are `[/Indexed base hival lookup]` where `lookup`
/// is either a byte string or a stream of `(hival + 1) * N` bytes (N = number
/// of components in the base color space).
fn resolve_indexed_palette(
    doc: Option<&crate::document::PdfDocument>,
    cs_obj: &crate::object::Object,
) -> Result<Option<IndexedResolution>> {
    use crate::object::Object;

    let Object::Array(arr) = cs_obj else {
        return Ok(None);
    };
    if arr.len() < 4 {
        return Ok(None);
    }

    // Resolve the base color-space object. When it's an array like
    // [/ICCBased <stream_ref>], resolve inner references so
    // parse_color_space can read /N from the ICC stream dict.
    let outer = resolve_image_object(doc, &arr[1])?;
    let source_profile = resolve_icc_profile_from_obj(doc, &outer);
    let base_obj = if let Some(d) = doc {
        if let Object::Array(inner) = &*outer {
            let mut inner = inner.clone();
            for item in inner.iter_mut() {
                if let Some(r) = item.as_reference() {
                    match d.load_object(r) {
                        Ok(resolved) => *item = resolved,
                        Err(_) => d.check_import_budget()?,
                    }
                }
            }
            Cow::Owned(Object::Array(inner))
        } else {
            outer
        }
    } else {
        outer
    };
    let base_cs = parse_color_space(&base_obj)?;
    let base_fmt = color_space_to_pixel_format(&base_cs);
    let n = base_fmt.bytes_per_pixel();

    // When the base is `/ICCBased`, capture the profile bytes so the
    // extractor can later hand them to a CMM. Parse failures reduce to
    // `None` — the decoder then falls back to §10.3.5 CMYK→RGB math as
    // if no profile were present.
    let base_profile = if matches!(base_cs, ColorSpace::ICCBased(_)) {
        source_profile
    } else {
        None
    };

    // hival bounds the valid index range. Resolve via indirect reference if
    // needed; treat invalid / missing values as "unknown" and skip truncation.
    let hival_obj = resolve_image_object(doc, &arr[2])?;
    let hival: Option<usize> = hival_obj.as_integer().and_then(|i| {
        if (0..=255).contains(&i) {
            Some(i as usize)
        } else {
            None
        }
    });

    let budget = doc.and_then(|doc| doc.import_budget());
    if budget.is_some() && hival.is_none() {
        return Err(Error::Image("invalid Indexed hival".into()));
    }
    let expected = hival.map(|h| (h + 1) * n);
    let lookup_obj = resolve_image_object(doc, &arr[3])?;
    let mut palette_bytes = match &*lookup_obj {
        Object::String(s) => match budget {
            Some(budget) => budget.copy_bytes(&s[..s.len().min(expected.unwrap())])?,
            None => s.clone(),
        },
        Object::Stream { .. } => match (doc, arr[3].as_reference()) {
            (Some(doc), Some(reference)) => {
                doc.decode_stream_with_encryption(&lookup_obj, reference)?
            },
            _ => lookup_obj.decode_stream_data()?,
        },
        _ => return Ok(None),
    };
    if palette_bytes.is_empty() {
        return Ok(None);
    }
    if budget.is_some() && expected.is_some_and(|expected| palette_bytes.len() < expected) {
        return Err(Error::Image("truncated Indexed palette".into()));
    }

    // Truncate palette to the logical length implied by hival so that indices
    // greater than hival fall into the out-of-range branch of the expander.
    // Per PDF 32000-1:2008 §8.6.6.3 the lookup is exactly (hival + 1) * N bytes;
    // anything beyond that is stray data that must not be mapped to pixels.
    if let Some(h) = hival {
        let expected = (h + 1).saturating_mul(n);
        if expected > 0 && palette_bytes.len() > expected {
            palette_bytes.truncate(expected);
        }
    }

    // Device-independent colour-space palettes must be converted to
    // RGB before being handed to the expander, which assumes palette
    // bytes are already in the output colour space. Without this step
    // Lab triples are mis-interpreted as raw RGB and render with
    // perceptually wrong colours.
    if matches!(base_cs, ColorSpace::Lab) {
        if let Some(budget) = budget {
            budget.charge_codec_allocation(palette_bytes.len())?;
        }
        let white = extract_lab_whitepoint(&base_obj);
        let rgb_palette = lab_palette_to_rgb(&palette_bytes, white);
        // Lab palettes are now RGB; no base ICC profile to carry through.
        return Ok(Some(IndexedResolution {
            base_fmt: PixelFormat::RGB,
            palette: rgb_palette,
            base_profile: None,
            retains_source_samples: false,
        }));
    }

    Ok(Some(IndexedResolution {
        base_fmt,
        palette: palette_bytes,
        base_profile,
        retains_source_samples: true,
    }))
}

/// Reduce a big-endian 16-bit colour sample (`hi`, `lo` bytes) to 8 bits,
/// rounding `v * 255 / 65535` to the nearest value. Preferred over the crude
/// high-byte drop (`v >> 8`), which floors and biases every sample downward by
/// up to ~1 LSB. `v == 0xFFFF` maps to `255` and `v == 0` to `0` exactly.
#[inline]
fn reduce_16_to_8(hi: u8, lo: u8) -> u8 {
    let v = u16::from_be_bytes([hi, lo]) as u32;
    ((v * 255 + 32_767) / 65_535) as u8
}

/// Expand packed Indexed image indices into RGB bytes using the palette.
///
/// Supports 1, 2, 4, and 8 bit-per-component index streams. Rows are padded
/// to byte boundaries per the PDF spec.
///
/// Returns `Err(Error::Image)` when the requested dimensions would require
/// more than `MAX_INDEXED_OUTPUT_BYTES` to decode, or when the `usize`
/// arithmetic on `width * height * channels` / `width * bpc` overflows,
/// or when the input `raw` buffer is too short to supply every row of the
/// requested height. This is an input-amplification guard for maliciously
/// crafted PDFs that pair tiny streams with extreme Indexed image
/// dimensions — see issue #324.
#[cfg(test)]
fn expand_indexed_to_rgb(
    raw: &[u8],
    palette: &[u8],
    base_fmt: PixelFormat,
    width: u32,
    height: u32,
    bpc: u8,
) -> Result<Vec<u8>> {
    expand_indexed_to_rgb_with_transform(raw, palette, base_fmt, width, height, bpc, None)
}

/// Like [`expand_indexed_to_rgb`] but routes CMYK palette entries
/// through an ICC transform when one is supplied. Used during image
/// extraction when the base colour space is `/ICCBased` with N=4.
#[cfg(any(test, feature = "jpeg2000"))]
#[cfg(test)]
fn expand_indexed_to_rgb_with_transform(
    raw: &[u8],
    palette: &[u8],
    base_fmt: PixelFormat,
    width: u32,
    height: u32,
    bpc: u8,
    transform: Option<&crate::color::Transform>,
) -> Result<Vec<u8>> {
    expand_indexed_to_rgb_with_context(
        raw,
        palette,
        base_fmt,
        width,
        height,
        bpc,
        IndexedDecodeContext {
            transform,
            budget: None,
            decode: None,
            preblended_opacity: None,
        },
    )
}

struct IndexedDecodeContext<'a> {
    transform: Option<&'a crate::color::Transform>,
    budget: Option<&'a crate::import_limits::ImportBudget>,
    decode: Option<(f64, f64)>,
    /// Interleaved JPX samples and stride. Opacity is the last component.
    /// Indexed preblending applies to palette colours, before colour conversion.
    preblended_opacity: Option<(&'a [u8], usize)>,
}

fn expand_indexed_to_rgb_with_context(
    raw: &[u8],
    palette: &[u8],
    base_fmt: PixelFormat,
    width: u32,
    height: u32,
    bpc: u8,
    context: IndexedDecodeContext<'_>,
) -> Result<Vec<u8>> {
    let transform = context.transform;
    /// Hard cap on the decoded output buffer size (256 MiB). Legitimate
    /// Indexed images in real PDFs are several orders of magnitude below
    /// this — the cap only fires on pathological / adversarial inputs
    /// where `width * height` is billions of pixels.
    const MAX_INDEXED_OUTPUT_BYTES: usize = 256 * 1024 * 1024;

    let w = width as usize;
    let h = height as usize;
    let n = base_fmt.bytes_per_pixel();
    if let Some((samples, stride)) = context.preblended_opacity {
        let required = w
            .checked_mul(h)
            .and_then(|pixels| pixels.checked_mul(stride));
        if stride < 2 || required.is_none_or(|length| samples.len() < length) {
            return Err(Error::Image("Indexed opacity sample extent".into()));
        }
        if let Some(budget) = context.budget {
            budget.charge_codec_work(
                w.checked_mul(h)
                    .and_then(|pixels| pixels.checked_mul(n))
                    .ok_or_else(|| budget.reject("Indexed preblend work"))?,
            )?;
        }
    }

    // ISO 32000-2 §8.9.5.1 mandates bpc ∈ {1, 2, 4, 8} for Indexed color
    // spaces. Anything else (0, 3, 5, 6, 7, 9, 12, 16, …) used to be
    // accepted silently — bpc=0 was coerced to 1 and any other value fell
    // through the `read_index` `_ => 0` arm, producing a solid palette-
    // entry-0 image with no error. Reject up front so malformed input is
    // surfaced instead of decoded into nonsense pixels.
    if !matches!(bpc, 1 | 2 | 4 | 8) {
        return Err(Error::Image(format!(
            "Indexed image has invalid /BitsPerComponent {bpc} \
             (PDF spec requires 1, 2, 4, or 8)"
        )));
    }

    // Checked arithmetic for `bytes_per_row = ceil(w * bpc / 8)`.
    let bytes_per_row = w
        .checked_mul(bpc as usize)
        .map(|v| v.div_ceil(8))
        .ok_or_else(|| {
            Error::Image(format!("Indexed image row width overflow: {w} × {bpc} bpc exceeds usize"))
        })?;

    // Checked arithmetic for `w * h * 3` (output always written as RGB).
    let output_bytes = w
        .checked_mul(h)
        .and_then(|v| v.checked_mul(3))
        .ok_or_else(|| {
            Error::Image(format!("Indexed image output size overflow: {w} × {h} × 3 exceeds usize"))
        })?;

    if output_bytes > MAX_INDEXED_OUTPUT_BYTES {
        return Err(Error::Image(format!(
            "Indexed image decode would produce {output_bytes} bytes, \
             exceeds guard limit of {MAX_INDEXED_OUTPUT_BYTES} bytes \
             (width={w}, height={h})"
        )));
    }

    // The decoded index stream must cover every row of the image.
    // Truncated streams used to get silently zero-padded, which lets a
    // malicious PDF pair a 10-byte stream with a 10 000 × 10 000 image
    // and force a ~300 MiB allocation filled with default palette entry
    // 0. Reject that shape up front.
    let required_bytes = bytes_per_row.checked_mul(h).ok_or_else(|| {
        Error::Image(format!(
            "Indexed image required-input size overflow: {bytes_per_row} × {h} exceeds usize"
        ))
    })?;
    if raw.len() < required_bytes {
        return Err(Error::Image(format!(
            "Indexed image index stream truncated: {} bytes available, \
             {} required ({} bytes/row × {} rows)",
            raw.len(),
            required_bytes,
            bytes_per_row,
            h
        )));
    }

    if let Some(budget) = context.budget {
        if output_bytes > budget.limits.stream_bytes {
            return Err(budget.reject("Indexed RGB buffer"));
        }
        budget.charge_codec_allocation(output_bytes)?;
    }
    let mut out = Vec::new();
    out.try_reserve_exact(output_bytes)
        .map_err(|_| match context.budget {
            Some(budget) => budget.reject("Indexed RGB allocation"),
            None => Error::Image("Indexed RGB allocation failed".into()),
        })?;

    let read_index = |row: &[u8], x: usize| -> usize {
        match bpc {
            8 => row.get(x).copied().unwrap_or(0) as usize,
            4 => {
                let byte_idx = x / 2;
                let b = row.get(byte_idx).copied().unwrap_or(0);
                if x.is_multiple_of(2) {
                    (b >> 4) as usize
                } else {
                    (b & 0x0F) as usize
                }
            },
            2 => {
                let byte_idx = x / 4;
                let b = row.get(byte_idx).copied().unwrap_or(0);
                let shift = 6 - (x % 4) * 2;
                ((b >> shift) & 0x03) as usize
            },
            1 => {
                let byte_idx = x / 8;
                let b = row.get(byte_idx).copied().unwrap_or(0);
                let shift = 7 - (x % 8);
                ((b >> shift) & 0x01) as usize
            },
            // Unreachable: bpc is validated to be in {1, 2, 4, 8} above
            // before the closure is called, so this arm only exists to
            // satisfy exhaustiveness on `u8`.
            _ => unreachable!("bpc validated to {{1,2,4,8}} before read_index"),
        }
    };

    for y in 0..h {
        let row_start = y * bytes_per_row;
        let row_end = (row_start + bytes_per_row).min(raw.len());
        let row: &[u8] = if row_start < raw.len() {
            &raw[row_start..row_end]
        } else {
            &[]
        };
        // The palette defines the valid index range, so its last complete
        // entry is `hival` as far as this buffer is concerned.
        let last_entry = if n == 0 {
            0
        } else {
            (palette.len() / n).saturating_sub(1)
        };
        for x in 0..w {
            // ISO 32000-1:2008 §8.6.6.3 (`docs/spec/pdf.md`:11053-11054): the
            // index "should be an integer in the range 0 to hival … if it is
            // outside the range 0 to hival, it shall be adjusted to the
            // nearest value within that range."
            //
            // Adjusted to the nearest value, not treated as an error. Emitting
            // black for an out-of-range index — which is what this did — is a
            // colour the file never asked for, and it darkens the page: on the
            // file named for this case we rendered a mean tone of 222.34 where
            // four engines agree on 231.72–235.49.
            let sample = read_index(row, x);
            let idx = if let Some((low, high)) = context.decode {
                let fraction = sample as f64 / ((1u32 << bpc) - 1) as f64;
                // Weighted endpoints avoid overflowing high-low for finite
                // opposite endpoints. Indexed values round to nearest integer.
                ((1.0 - fraction) * low + fraction * high)
                    .round()
                    .clamp(0.0, last_entry as f64) as usize
            } else {
                sample.min(last_entry)
            };
            let off = idx * n;
            if off + n > palette.len() {
                out.extend_from_slice(&[0, 0, 0]);
                continue;
            }
            let mut straight = [0; 4];
            let colour = if let Some((samples, stride)) = context.preblended_opacity {
                let alpha = samples[(y * w + x) * stride + stride - 1];
                for (output, value) in straight[..n].iter_mut().zip(&palette[off..off + n]) {
                    *output = unassociate_zero_matte(*value, alpha);
                }
                &straight[..n]
            } else {
                &palette[off..off + n]
            };
            if let Some(transform) = transform {
                let converted = transform.convert_pixel(colour);
                if let Some(budget) = context.budget {
                    budget.check()?;
                }
                if let Some(rgb) = converted {
                    out.extend_from_slice(&rgb);
                    continue;
                }
            }
            match base_fmt {
                PixelFormat::RGBA => {
                    return Err(Error::Image("Indexed base cannot contain alpha".into()));
                },
                PixelFormat::RGB => out.extend_from_slice(&colour[..3]),
                PixelFormat::Grayscale => {
                    let g = colour[0];
                    out.push(g);
                    out.push(g);
                    out.push(g);
                },
                PixelFormat::CMYK => {
                    let c = colour[0];
                    let m = colour[1];
                    let y_c = colour[2];
                    let k = colour[3];
                    let [r, g, b] = if let Some(t) = transform {
                        t.convert_cmyk_pixel(c, m, y_c, k)
                    } else {
                        cmyk_pixel_to_rgb(c, m, y_c, k)
                    };
                    out.push(r);
                    out.push(g);
                    out.push(b);
                },
            }
        }
    }
    Ok(out)
}

/// Convert a single DeviceCMYK pixel to RGB.
///
/// Shared conversion math used by both bulk CMYK->RGB and Indexed palette
/// expansion so the two paths cannot drift apart.
///
/// Uses the PROCESS-INK conversion (`color::cmyk_to_rgb`, tetralinear over the
/// 16 measured ink corners of the CMYK cube), NOT the naive additive-then-clamp
/// `R = 1 - min(1, C+K)`. The additive form treats the inks as pure subtractive
/// primaries and so renders 100% K as `#000000` and 100% cyan as `#00FFFF`;
/// DeviceCMYK is a *device* space, so the colour is what the inks look like -
/// 100% K is `#231F20`, 100% cyan `#00ADEF`. This is the same conversion the
/// text/vector paths already use, so a DeviceCMYK image now matches the colour
/// of DeviceCMYK text and fills on the same page. For `/ICCBased` CMYK a real
/// CMM (qcms / lcms2) still takes precedence when a profile is available; this
/// is the no-profile fallback.
pub(crate) fn cmyk_pixel_to_rgb(c: u8, m: u8, y: u8, k: u8) -> [u8; 3] {
    let (r, g, b) = crate::color::cmyk_to_rgb(
        c as f32 / 255.0,
        m as f32 / 255.0,
        y as f32 / 255.0,
        k as f32 / 255.0,
    );
    [
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    ]
}

/// Extract `/WhitePoint` from a Lab colour-space PDF object.
///
/// The object is `[/Lab << /WhitePoint [Xw Yw Zw] >>]`. Returns the
/// whitepoint as `[Xw, Yw, Zw]`, falling back to D65 if absent.
fn extract_lab_whitepoint(cs_obj: &crate::object::Object) -> [f64; 3] {
    const D65: [f64; 3] = [0.9505, 1.0, 1.0890];
    let arr = match cs_obj {
        crate::object::Object::Array(a) => a,
        _ => return D65,
    };
    if arr.len() < 2 {
        return D65;
    }
    let dict = match &arr[1] {
        crate::object::Object::Dictionary(d) => d,
        _ => return D65,
    };
    let wp = match dict.get("WhitePoint") {
        Some(crate::object::Object::Array(a)) if a.len() >= 3 => a,
        _ => return D65,
    };
    let f = |obj: &crate::object::Object| -> Option<f64> {
        match obj {
            crate::object::Object::Real(v) => Some(*v),
            crate::object::Object::Integer(v) => Some(*v as f64),
            _ => None,
        }
    };
    match (f(&wp[0]), f(&wp[1]), f(&wp[2])) {
        (Some(x), Some(y), Some(z)) => [x, y, z],
        _ => D65,
    }
}

/// Convert a Lab-encoded palette to sRGB.
///
/// Each entry is 3 bytes: L* (byte 0), a* (byte 1), b* (byte 2).
/// Decoding per PDF 32000-1:2008 §8.6.5.4:
///   L* = byte_0 / 255.0 × 100.0
///   a* = byte_1 − 128.0   (default /Range [−128 127])
///   b* = byte_2 − 128.0
///
/// Then Lab → XYZ (whitepoint-relative) → sRGB with standard gamma.
pub(crate) fn lab_palette_to_rgb(palette: &[u8], white: [f64; 3]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(palette.len());
    for chunk in palette.chunks(3) {
        if chunk.len() < 3 {
            rgb.extend_from_slice(&[0, 0, 0]);
            continue;
        }
        let [r, g, b] = lab_pixel_to_rgb(chunk[0], chunk[1], chunk[2], white);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    rgb
}

// NOTE: The XYZ→linear-sRGB matrix below assumes a D65 whitepoint. Lab CIEs
// whose `/WhitePoint` is non-D65 (D50 is common in print workflows) would
// strictly need chromatic adaptation (e.g., Bradford) from the source
// whitepoint to D65 before the sRGB matrix. We intentionally omit that for
// now — the vast majority of PDF `/Lab` spaces we encounter are D65 — but
// the caller's `white` is still used to scale `xw, yw, zw` so D65 and
// near-D65 whitepoints produce correct output. Non-D65 spaces will have a
// minor chromatic-adaptation error until this is revisited.
fn lab_pixel_to_rgb(l_byte: u8, a_byte: u8, b_byte: u8, white: [f64; 3]) -> [u8; 3] {
    let l_star = l_byte as f64 / 255.0 * 100.0;
    let a_star = a_byte as f64 - 128.0;
    let b_star = b_byte as f64 - 128.0;

    let fy = (l_star + 16.0) / 116.0;
    let fx = a_star / 500.0 + fy;
    let fz = fy - b_star / 200.0;

    let [xw, yw, zw] = white;
    let x = xw * f_inv(fx);
    let y = yw * f_inv(fy);
    let z = zw * f_inv(fz);

    // XYZ → linear sRGB (D65 matrix, IEC 61966-2-1:1999)
    let r_lin = 3.2406254773 * x - 1.5372079722 * y - 0.4986285987 * z;
    let g_lin = -0.9689307147 * x + 1.8757560609 * y + 0.0415175580 * z;
    let b_lin = 0.0557101204 * x - 0.2040210506 * y + 1.0569959423 * z;

    [srgb_gamma(r_lin), srgb_gamma(g_lin), srgb_gamma(b_lin)]
}

fn f_inv(t: f64) -> f64 {
    const DELTA: f64 = 6.0 / 29.0;
    if t > DELTA {
        t * t * t
    } else {
        3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
    }
}

fn srgb_gamma(lin: f64) -> u8 {
    let v = if lin <= 0.0031308 {
        12.92 * lin
    } else {
        1.055 * lin.powf(1.0 / 2.4) - 0.055
    };
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Convert a raw CMYK byte stream (4 bytes per pixel) to straight RGB bytes
/// (3 bytes per pixel) using the naive per-pixel conversion.
///
/// This is a non-ICC conversion and does not handle Adobe-inverted JPEG CMYK;
/// for JPEG-encoded CMYK streams use `decode_adobe_cmyk_jpeg` instead.
pub fn cmyk_to_rgb(cmyk: &[u8]) -> Vec<u8> {
    cmyk_to_rgb_with_transform(cmyk, None)
}

/// Like [`cmyk_to_rgb`] but routes through an ICC transform when given,
/// and falls through to §10.3.5 otherwise. Used by save_raw_as_* when
/// the source image carries an ICC profile.
pub fn cmyk_to_rgb_with_transform(
    cmyk: &[u8],
    transform: Option<&crate::color::Transform>,
) -> Vec<u8> {
    if let Some(t) = transform {
        return t.convert_cmyk_buffer(cmyk);
    }
    let mut rgb = Vec::with_capacity((cmyk.len() / 4) * 3);
    for chunk in cmyk.as_chunks::<4>().0 {
        let [r, g, b] = cmyk_pixel_to_rgb(chunk[0], chunk[1], chunk[2], chunk[3]);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    rgb
}

/// Decode a CMYK-colourspace JPEG to straight RGB bytes, applying Adobe's
/// Thin wrapper that falls back to the intent-less, profile-less
/// variant — kept as the public, backwards-compatible entry point.
pub fn decode_cmyk_jpeg_to_rgb(jpeg_data: &[u8]) -> Result<Vec<u8>> {
    decode_cmyk_jpeg_to_rgb_with_profile(jpeg_data, None)
}

/// Decode a DeviceCMYK JPEG to raw 8-bpc CMYK samples (W*H*4 bytes,
/// channel order C, M, Y, K). Output is the raw DCT sample plane treated
/// as straight CMYK ink (0 = no ink, 255 = full coverage), matching how
/// poppler / Ghostscript render DCTDecode CMYK streams inside a PDF.
///
/// `jpeg-decoder` 0.3 applies Adobe's `255 - x` inversion to EVERY
/// 4-component JPEG that carries an Adobe APP14 marker
/// (`color_convert_line_cmyk` for `transform = 0`; `color_convert_line_ycck`
/// for `transform = 2`). PDF renderers do not do that inversion - they use
/// the raw DCT samples as the CMYK ink. So whenever an Adobe marker is
/// present, pdf_oxide must undo the decoder's inversion with a second
/// `255 - x` to recover the raw samples:
///
/// - `color_transform = 0` (plain CMYK, Photoshop / Distiller default) -
///   undo the decoder's `color_convert_line_cmyk` inversion.
/// - `color_transform = 2` (YCCK, Adobe Illustrator default) - the decoder
///   ran YCbCr->RGB plus `255 - K`; the same `255 - x` on all four channels
///   recovers the raw CMYK plane.
/// - no APP14 marker - the samples are used as-is (non-Adobe convention),
///   left unchanged to avoid disturbing non-Adobe JPEG handling.
///
/// The jpeg-decoder inversion contract is pinned by
/// `tests/test_jpeg_decoder_cmyk_contract.rs`; the Adobe decode path is
/// covered by `tests/test_cmyk_jpeg_adobe_inversion.rs`. If a future
/// jpeg-decoder release changes its inversion behaviour, those tests fire
/// before real fixtures regress.
///
/// Used by the separation pipeline to route CMYK image channels directly
/// to the matching ink plates without going through a colour-space
/// conversion to RGB and back. Only the rendering feature consumes this;
/// without it the function would be dead code under `-D warnings`.
#[cfg(feature = "rendering")]
pub(crate) fn decode_cmyk_jpeg_to_raw_cmyk(jpeg_data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(jpeg_data));
    let cmyk = decoder
        .decode()
        .map_err(|e| Error::Decode(format!("Failed to decode CMYK JPEG: {}", e)))?;
    let info = decoder
        .info()
        .ok_or_else(|| Error::Decode("JPEG info unavailable".to_string()))?;

    let pixel_count = (info.width as usize) * (info.height as usize);
    let expected = pixel_count * 4;
    if cmyk.len() < expected {
        return Err(Error::Decode(format!(
            "CMYK JPEG decoded {} bytes, expected {}",
            cmyk.len(),
            expected
        )));
    }

    let mut raw = cmyk;
    raw.truncate(expected);
    // jpeg-decoder has already applied a `255 - x` inversion; undo it to
    // recover the raw DCT samples poppler uses as straight CMYK ink.
    //
    // The undo used to run only when an Adobe APP14 marker was present, which
    // is wrong twice over. ISO 32000-1:2008 Table 13 (`docs/spec/pdf.md:2979`)
    // says that when the marker is absent "the default value of ColorTransform
    // shall be 1 if the image has three components and **0 otherwise**" — so
    // for four components, no marker *is* transform 0, the very case the undo
    // handles. And measurement settles it independently: `jpeg-decoder`
    // returns byte-identical samples for the same image with and without the
    // marker, pinned by `tests/test_cmyk_jpeg_without_app14_marker.rs`. A
    // marker-less 4-component JPEG therefore kept the decoder's inversion and
    // rendered as the complement of its ink.
    //
    // Only an explicit marker declaring some other transform opts out.
    if cmyk_jpeg_samples_are_inverted(jpeg_data) {
        for b in raw.iter_mut() {
            *b = 255 - *b;
        }
    }
    Ok(raw)
}

/// Whether `jpeg-decoder`'s output for this 4-component JPEG carries its
/// `255 - x` inversion, so the extractor must undo it.
///
/// True when there is no Adobe APP14 marker (Table 13 makes that
/// `ColorTransform 0` for four components) or when the marker declares
/// transform 0 (plain CMYK) or 2 (YCCK). An explicit marker declaring anything
/// else is taken at its word.
fn cmyk_jpeg_samples_are_inverted(jpeg_data: &[u8]) -> bool {
    matches!(scan_app14_color_transform(jpeg_data), None | Some(0) | Some(2))
}

/// Like [`decode_cmyk_jpeg_to_rgb`] but applies the given ICC transform
/// when provided, falling back to §10.3.5 otherwise. Used internally by
/// `PdfImage::save_as_*` when the source image carries an ICCBased
/// colour space (or when the document's `OutputIntents` supplied a
/// default CMYK profile).
///
/// APP14 handling matches `decode_cmyk_jpeg_to_raw_cmyk`: when an Adobe
/// marker is present (CMYK transform 0 or YCCK transform 2), jpeg-decoder's
/// `255 - x` inversion is undone to recover the raw DCT samples poppler
/// treats as straight CMYK; without a marker the samples pass through.
pub fn decode_cmyk_jpeg_to_rgb_with_profile(
    jpeg_data: &[u8],
    transform: Option<&crate::color::Transform>,
) -> Result<Vec<u8>> {
    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(jpeg_data));
    let cmyk = decoder
        .decode()
        .map_err(|e| Error::Decode(format!("Failed to decode CMYK JPEG: {}", e)))?;
    let info = decoder
        .info()
        .ok_or_else(|| Error::Decode("JPEG info unavailable".to_string()))?;

    let pixel_count = (info.width as usize) * (info.height as usize);
    let expected = pixel_count * 4;
    if cmyk.len() < expected {
        return Err(Error::Decode(format!(
            "CMYK JPEG decoded {} bytes, expected {}",
            cmyk.len(),
            expected
        )));
    }

    // Undo jpeg-decoder's Adobe `255 - x` inversion (applied for any APP14
    // CMYK transform 0 or YCCK transform 2) to recover the raw DCT samples,
    // which poppler / Ghostscript render as straight CMYK ink. No marker ->
    // pass through unchanged.
    let straight_cmyk: Vec<u8> = if cmyk_jpeg_samples_are_inverted(jpeg_data) {
        cmyk[..expected].iter().map(|b| 255 - *b).collect()
    } else {
        cmyk[..expected].to_vec()
    };

    if let Some(t) = transform {
        return Ok(t.convert_cmyk_buffer(&straight_cmyk));
    }

    // §10.3.5 additive-clamp fallback.
    let mut rgb = Vec::with_capacity(pixel_count * 3);
    for chunk in straight_cmyk.as_chunks::<4>().0 {
        let [r, g, b] = cmyk_pixel_to_rgb(chunk[0], chunk[1], chunk[2], chunk[3]);
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    Ok(rgb)
}

/// Walk the JPEG marker stream for an Adobe APP14 ("Adobe") segment and
/// return its `color_transform` byte. The byte is 0 for plain CMYK
/// (Photoshop), 1 for YCbCr (3-channel), 2 for YCCK (Adobe Illustrator).
fn scan_app14_color_transform(jpeg_data: &[u8]) -> Option<u8> {
    let mut i = 0;
    while i + 1 < jpeg_data.len() {
        if jpeg_data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = jpeg_data[i + 1];
        i += 2;
        if marker == 0x00 || marker == 0xFF {
            continue;
        }
        if matches!(marker, 0xD0..=0xD9) || marker == 0x01 {
            continue;
        }
        if i + 1 >= jpeg_data.len() {
            break;
        }
        let seg_len = u16::from_be_bytes([jpeg_data[i], jpeg_data[i + 1]]) as usize;
        if seg_len < 2 || i + seg_len > jpeg_data.len() {
            break;
        }
        if marker == 0xEE && seg_len >= 14 {
            let payload = &jpeg_data[i + 2..i + seg_len];
            if payload.len() >= 12 && payload.starts_with(b"Adobe") {
                return Some(payload[11]);
            }
        }
        if marker == 0xDA {
            break;
        }
        i += seg_len;
    }
    None
}

fn save_jpeg_as_png(jpeg_data: &[u8], path: impl AsRef<Path>) -> Result<()> {
    use image::ImageFormat;
    let img = image::load_from_memory_with_format(jpeg_data, ImageFormat::Jpeg)
        .map_err(|e| Error::Image(format!("Failed to decode JPEG: {}", e)))?;
    img.save_with_format(path, ImageFormat::Png)
        .map_err(|e| Error::Image(format!("Failed to save PNG: {}", e)))
}

/// Decide whether a given ICC transform should actually be applied to a
/// buffer of the given pixel format. The transform was compiled for
/// whatever component count the profile advertised; applying it to a
/// mismatched buffer (e.g. a 4-component CMYK transform to a 3-channel
/// RGB buffer) would produce garbage. A `None` transform, or a
/// transform whose profile components disagree with `format`, is
/// suppressed so the caller falls through to identity / fallback math.
fn icc_matches_format(
    transform: Option<&crate::color::Transform>,
    format: PixelFormat,
) -> Option<&crate::color::Transform> {
    let t = transform?;
    let needed = match format {
        // Mask composition has already converted colour samples to RGB.
        PixelFormat::RGBA => return None,
        PixelFormat::RGB => 3,
        PixelFormat::Grayscale => 1,
        PixelFormat::CMYK => 4,
    };
    if t.source_n_components() == needed {
        Some(t)
    } else {
        None
    }
}

fn save_raw_as_png(
    pixels: &[u8],
    width: u32,
    height: u32,
    format: PixelFormat,
    transform: Option<&crate::color::Transform>,
    path: impl AsRef<Path>,
) -> Result<()> {
    use image::{ImageBuffer, ImageFormat, Luma, Rgb};

    match format {
        PixelFormat::RGBA => {
            let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
                .ok_or_else(|| Error::Image("Invalid RGBA image dimensions".into()))?;
            img.save_with_format(path, ImageFormat::Png)
                .map_err(|e| Error::Image(format!("Failed to save PNG: {e}")))
        },
        PixelFormat::RGB => {
            // RGB source through an ICC profile (Adobe RGB, ProPhoto, wide-
            // gamut cameras) → convert to sRGB before writing. With no
            // profile the bytes are assumed sRGB already and passed through.
            let rgb = match icc_matches_format(transform, format) {
                Some(t) => t.convert_rgb_buffer(pixels),
                None => pixels.to_vec(),
            };
            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb)
                .ok_or_else(|| Error::Image("Invalid RGB image dimensions".to_string()))?;
            img.save_with_format(path, ImageFormat::Png)
                .map_err(|e| Error::Image(format!("Failed to save PNG: {}", e)))
        },
        PixelFormat::Grayscale => {
            // A Gray ICC profile promotes to sRGB RGB; without one the
            // single channel is written as an L8 PNG.
            if let Some(t) = icc_matches_format(transform, format) {
                let rgb = t.convert_gray_buffer(pixels);
                let img =
                    ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb).ok_or_else(|| {
                        Error::Image("Invalid grayscale image dimensions".to_string())
                    })?;
                img.save_with_format(path, ImageFormat::Png)
                    .map_err(|e| Error::Image(format!("Failed to save PNG: {}", e)))
            } else {
                let img = ImageBuffer::<Luma<u8>, _>::from_raw(width, height, pixels.to_vec())
                    .ok_or_else(|| {
                        Error::Image("Invalid grayscale image dimensions".to_string())
                    })?;
                img.save_with_format(path, ImageFormat::Png)
                    .map_err(|e| Error::Image(format!("Failed to save PNG: {}", e)))
            }
        },
        PixelFormat::CMYK => {
            let rgb = cmyk_to_rgb_with_transform(pixels, icc_matches_format(transform, format));
            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb)
                .ok_or_else(|| Error::Image("Invalid CMYK image dimensions".to_string()))?;
            img.save_with_format(path, ImageFormat::Png)
                .map_err(|e| Error::Image(format!("Failed to save PNG: {}", e)))
        },
    }
}

fn save_raw_as_jpeg(
    pixels: &[u8],
    width: u32,
    height: u32,
    format: PixelFormat,
    transform: Option<&crate::color::Transform>,
    path: impl AsRef<Path>,
) -> Result<()> {
    use image::{ImageBuffer, ImageFormat, Luma, Rgb};

    match format {
        PixelFormat::RGBA => Err(Error::Unsupported(
            "JPEG cannot preserve image transparency; save as PNG".into(),
        )),
        PixelFormat::RGB => {
            let rgb = match icc_matches_format(transform, format) {
                Some(t) => t.convert_rgb_buffer(pixels),
                None => pixels.to_vec(),
            };
            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb)
                .ok_or_else(|| Error::Image("Invalid RGB image dimensions".to_string()))?;
            img.save_with_format(path, ImageFormat::Jpeg)
                .map_err(|e| Error::Image(format!("Failed to save JPEG: {}", e)))
        },
        PixelFormat::Grayscale => {
            if let Some(t) = icc_matches_format(transform, format) {
                let rgb = t.convert_gray_buffer(pixels);
                let img =
                    ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb).ok_or_else(|| {
                        Error::Image("Invalid grayscale image dimensions".to_string())
                    })?;
                img.save_with_format(path, ImageFormat::Jpeg)
                    .map_err(|e| Error::Image(format!("Failed to save JPEG: {}", e)))
            } else {
                let img = ImageBuffer::<Luma<u8>, _>::from_raw(width, height, pixels.to_vec())
                    .ok_or_else(|| {
                        Error::Image("Invalid grayscale image dimensions".to_string())
                    })?;
                img.save_with_format(path, ImageFormat::Jpeg)
                    .map_err(|e| Error::Image(format!("Failed to save JPEG: {}", e)))
            }
        },
        PixelFormat::CMYK => {
            let rgb = cmyk_to_rgb_with_transform(pixels, icc_matches_format(transform, format));
            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb)
                .ok_or_else(|| Error::Image("Invalid CMYK image dimensions".to_string()))?;
            img.save_with_format(path, ImageFormat::Jpeg)
                .map_err(|e| Error::Image(format!("Failed to save JPEG: {}", e)))
        },
    }
}

/// Decode a JBIG2-compressed PDF image stream into raw grayscale pixels.
/// Decode a JBIG2 stencil into packed 1-bit rows in the PDF sample convention.
///
/// An explicit `/Mask` is an image mask, and §8.9.6.2 reads its *samples*: a
/// sample of 0 marks the page. Table 12's example writes a JBIG2 image as
/// `/DeviceGray /BitsPerComponent 1`, in which 0 is black — so a black JBIG2
/// pixel is sample 0 and marks the page, which for an explicit mask means the
/// base image shows through there.
///
/// Returns `((width + 7) / 8) * height` bytes, MSB first, which is the layout
/// the stencil loop indexes. Without this the compressed bitstream reached
/// that loop unchanged: almost every sample fell past the end of the buffer
/// and the mask was silently ignored, so a scanned page rendered as the raw
/// grey scan with no text knocked out.
#[cfg(feature = "jbig2")]
pub(crate) fn decode_jbig2_stencil(
    stream: &crate::object::Object,
    obj_ref: Option<ObjectRef>,
    dict: &std::collections::HashMap<String, crate::object::Object>,
    doc: Option<&crate::document::PdfDocument>,
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    let ImageData::Raw { pixels, .. } =
        decode_jbig2_image(stream, obj_ref, dict, doc, width, height)?
    else {
        return Err(Error::Image("JBIG2 stencil decode returned no samples".to_string()));
    };

    let row_bytes = (width as usize).div_ceil(8);
    // 0xFF = every sample 1 = "leave the previous contents unchanged", so a
    // row the decoder did not reach masks nothing out rather than erasing the
    // base image.
    let budget = doc.and_then(|d| d.import_budget());
    crate::codec_resource::decode(budget, || {
        let count = rhwp_codec_resource::product(&[row_bytes, height as usize])
            .map_err(|_| Error::Image("JBIG2 stencil size".into()))?;
        let mut packed = rhwp_codec_resource::filled(0xFFu8, count)
            .map_err(|_| Error::Image("JBIG2 stencil allocation".into()))?;
        rhwp_codec_resource::work(pixels.len())
            .map_err(|_| Error::Image("JBIG2 stencil work".into()))?;
        for (y, row) in pixels.chunks_exact(width as usize).enumerate() {
            for (x, &g) in row.iter().enumerate() {
                if g < 128 {
                    // black -> sample 0
                    packed[y * row_bytes + x / 8] &= !(0x80 >> (x % 8));
                }
            }
        }
        Ok(packed)
    })
}

#[cfg(feature = "jbig2")]
fn decode_jbig2_image(
    xobject: &crate::object::Object,
    obj_ref: Option<ObjectRef>,
    dict: &std::collections::HashMap<String, crate::object::Object>,
    doc: Option<&crate::document::PdfDocument>,
    width: u32,
    height: u32,
) -> Result<ImageData> {
    crate::codec_resource::decode(doc.and_then(|d| d.import_budget()), || {
        use rhwp_codec_resource as resource;
        let jbig2_bytes = if let (Some(d), Some(id)) = (doc, obj_ref) {
            d.decode_stream_with_encryption(xobject, id)?
        } else {
            xobject.decode_stream_data()?
        };

        // DecodeParms follows the same filter position as JBIG2Decode, and may
        // be indirect. A present but broken globals stream must not disappear.
        let mut globals = None;
        if let Some(d) = doc {
            if let Some(parms) = dict.get("DecodeParms") {
                let parms = resolve_image_object(doc, parms)?;
                let selected;
                let parms = if let Some(array) = parms.as_array() {
                    let filters = parse_filter_chain(dict, doc)?;
                    let index = filters
                        .iter()
                        .position(|f| *f == PdfFilter::JBIG2Decode)
                        .unwrap_or(0);
                    selected = array
                        .get(index)
                        .map(|v| resolve_image_object(doc, v))
                        .transpose()?;
                    selected.as_deref()
                } else {
                    Some(&*parms)
                };
                if let Some(value) = parms
                    .and_then(|p| p.as_dict())
                    .and_then(|p| p.get("JBIG2Globals"))
                {
                    let id = value.as_reference().ok_or_else(|| {
                        Error::Image("JBIG2Globals must reference a stream".into())
                    })?;
                    let object = resolve_image_object(doc, value)?;
                    if !matches!(&*object, crate::object::Object::Stream { .. }) {
                        return Err(Error::Image("JBIG2Globals must resolve to a stream".into()));
                    }
                    globals = Some(d.decode_stream_with_encryption(&object, id)?);
                }
            }
        }

        let image = hayro_jbig2::Image::new_embedded(&jbig2_bytes, globals.as_deref())
            .map_err(|e| Error::Image(format!("JBIG2 decode error: {e}")))?;
        if image.width() != width || image.height() != height {
            return Err(Error::Image("JBIG2 dimensions differ from PDF image".into()));
        }
        // The callback cannot return errors. Allocate once, never grow in it,
        // and verify every row and the complete sample count after decode.
        struct Collector {
            pixels: Vec<u8>,
            width: usize,
            x: usize,
            y: usize,
            invalid: bool,
        }
        impl hayro_jbig2::Decoder for Collector {
            fn push_pixel(&mut self, black: bool) {
                self.write(black, 1);
            }
            fn push_pixel_chunk(&mut self, black: bool, chunks: u32) {
                match (chunks as usize).checked_mul(8) {
                    Some(count) => self.write(black, count),
                    None => self.invalid = true,
                }
            }
            fn next_line(&mut self) {
                self.invalid |= self.x != self.width;
                self.x = 0;
                self.y = self.y.saturating_add(1);
            }
        }
        impl Collector {
            fn write(&mut self, black: bool, count: usize) {
                if self.invalid {
                    return;
                }
                let Some(end) = self.x.checked_add(count).filter(|&e| e <= self.width) else {
                    self.invalid = true;
                    return;
                };
                let Some(start) = self
                    .y
                    .checked_mul(self.width)
                    .and_then(|n| n.checked_add(self.x))
                else {
                    self.invalid = true;
                    return;
                };
                let Some(stop) = start.checked_add(count) else {
                    self.invalid = true;
                    return;
                };
                let Some(output) = self.pixels.get_mut(start..stop) else {
                    self.invalid = true;
                    return;
                };
                output.fill(if black { 0 } else { 255 });
                self.x = end;
            }
        }
        let count = resource::product(&[width as usize, height as usize])
            .map_err(|_| Error::Image("JBIG2 sample size overflow".into()))?;
        let pixels = resource::filled(255, count)
            .map_err(|_| Error::Image("JBIG2 sample allocation".into()))?;
        let mut collector = Collector {
            pixels,
            width: width as usize,
            x: 0,
            y: 0,
            invalid: false,
        };
        image
            .decode(&mut collector)
            .map_err(|e| Error::Image(format!("JBIG2 pixel decode error: {e}")))?;
        if collector.invalid || collector.x != 0 || collector.y != height as usize {
            return Err(Error::Image("JBIG2 incomplete or excess pixel rows".into()));
        }
        Ok(ImageData::Raw {
            pixels: collector.pixels,
            format: PixelFormat::Grayscale,
        })
    })
}

#[cfg(not(feature = "jbig2"))]
fn decode_jbig2_image(
    _xobject: &crate::object::Object,
    _obj_ref: Option<ObjectRef>,
    _dict: &std::collections::HashMap<String, crate::object::Object>,
    _doc: Option<&crate::document::PdfDocument>,
    _width: u32,
    _height: u32,
) -> Result<ImageData> {
    Err(Error::UnsupportedFilter(
        "JBIG2Decode disabled: enable RHWP `pdf-jbig2` (backend `jbig2`) at build time".to_string(),
    ))
}

/// Decode a JPEG 2000 (`/JPXDecode`) image stream into raw interleaved samples.
///
/// `JpxDecoder` is a pass-through, so `decode_stream_*` yields the raw JPEG 2000
/// codestream, which hayro-jpeg2000 (`decoders::jpx`) decodes to 8-bit
/// component-interleaved samples.
fn unassociate_zero_matte(value: u8, alpha: u8) -> u8 {
    if alpha == 0 {
        0
    } else {
        ((u32::from(value) * 255 + u32::from(alpha) / 2) / u32::from(alpha)).min(255) as u8
    }
}

#[cfg(feature = "jpeg2000")]
fn decode_jpx_image(
    xobject: &crate::object::Object,
    obj_ref: Option<ObjectRef>,
    doc: Option<&crate::document::PdfDocument>,
    color_space: &ColorSpace,
    indexed: Option<(&IndexedResolution, Option<&crate::color::Transform>)>,
    direct_transform: Option<&crate::color::Transform>,
    target: Option<(u32, u32)>,
) -> Result<(ImageData, u32, u32)> {
    crate::codec_resource::decode(doc.and_then(|d| d.import_budget()), || {
        let dict = xobject
            .as_dict()
            .ok_or_else(|| Error::Image("JPX image dictionary".into()))?;
        let entry = |key: &str| {
            dict.get(key)
                .map(|v| resolve_image_object(doc, v))
                .transpose()
        };
        let smask_entry = entry("SMaskInData")?;
        let smask = match smask_entry.as_deref() {
            None | Some(crate::object::Object::Integer(0)) => 0,
            Some(crate::object::Object::Integer(v @ (1 | 2))) => *v,
            _ => return Err(Error::Image("invalid JPX SMaskInData".into())),
        };
        if smask != 0
            && entry("SMask")?
                .as_deref()
                .is_some_and(|v| !matches!(v, crate::object::Object::Null))
        {
            return Err(Error::Image("JPX SMaskInData forbids an external SMask".into()));
        }
        let dimension = |key: &str| {
            entry(key)?
                .as_deref()
                .and_then(|v| v.as_integer())
                .and_then(|v| u32::try_from(v).ok())
                .filter(|&v| v > 0)
                .ok_or_else(|| Error::Image(format!("invalid JPX {key}")))
        };
        let expected = doc
            .and_then(|d| d.import_budget())
            .map(|_| Ok::<_, Error>((dimension("Width")?, dimension("Height")?)))
            .transpose()?;
        let codestream: Vec<u8> = if let (Some(d), Some(ref_id)) = (doc.as_ref(), obj_ref) {
            d.decode_stream_with_encryption(xobject, ref_id)?
        } else {
            xobject.decode_stream_data()?
        };

        // An /Indexed dictionary space makes the codestream's one component a
        // table index, and §7.4.9 (pdf.md:3143) has the dictionary decide how the
        // samples are read: "the colour space specifications in the JPEG2000 data
        // shall be ignored". A JP2 file can carry a palette of its own for those
        // indices; letting the decoder resolve it produced three colour
        // components for a space that declares one, and the red channel was then
        // taken as a grey level. The indices come back unscaled, one byte each,
        // so the dictionary's table is read at 8 bits per index whatever
        // /BitsPerComponent says the codestream packed them at.
        if let Some((ir, transform)) = indexed {
            if smask == 2 && !ir.retains_source_samples {
                return Err(Error::Unsupported(
                    "preblended JPX Indexed palette requires source-space samples (Lab is unsupported)".into(),
                ));
            }
            let img = crate::decoders::jpx::decode_jpx_for_pdf(
                &codestream,
                target,
                false,
                expected,
                Some(1),
            )?;
            let components = img.num_components as usize;
            if components != if smask == 0 { components } else { 2 } {
                return Err(Error::Image("JPX Indexed mask has no opacity component".into()));
            }
            let mut indices = rhwp_codec_resource::capacity(img.samples.len() / components)
                .map_err(|_| Error::Image("JPX index allocation".into()))?;
            for pixel in img.samples.chunks_exact(components) {
                // ISO 32000-1 11.6.5.3: Indexed preblending affects the colour
                // table, not the indices. Recover palette components before CMM.
                indices.push(pixel[0]);
            }
            let expanded = expand_indexed_to_rgb_with_context(
                &indices,
                &ir.palette,
                ir.base_fmt,
                img.width,
                img.height,
                8,
                IndexedDecodeContext {
                    transform,
                    budget: doc.and_then(|d| d.import_budget()).map(|b| &**b),
                    decode: None,
                    preblended_opacity: (smask == 2).then_some((&img.samples, components)),
                },
            )?;
            if smask == 0 {
                return Ok((
                    ImageData::Raw {
                        pixels: expanded,
                        format: PixelFormat::RGB,
                    },
                    img.width,
                    img.height,
                ));
            }
            let mut rgba = rhwp_codec_resource::capacity(
                rhwp_codec_resource::product(&[indices.len(), 4])
                    .map_err(|_| Error::Image("JPX Indexed alpha size".into()))?,
            )
            .map_err(|_| Error::Image("JPX Indexed alpha allocation".into()))?;
            for (rgb, pixel) in expanded
                .chunks_exact(3)
                .zip(img.samples.chunks_exact(components))
            {
                rgba.extend_from_slice(rgb);
                rgba.push(pixel[1]);
            }
            return Ok((
                ImageData::Raw {
                    pixels: rgba,
                    format: PixelFormat::RGBA,
                },
                img.width,
                img.height,
            ));
        }

        let img = crate::decoders::jpx::decode_jpx_for_pdf(
            &codestream,
            target,
            true,
            expected,
            dict.contains_key("ColorSpace")
                .then_some(color_space.components() as u8),
        )?;

        // A JPX codestream may carry an opacity channel alongside its colour
        // channels — a greyscale image decodes to two components, an RGB one to
        // four. Table 89's /SMaskInData entry (`docs/spec/pdf.md`:14527) governs
        // that channel and defaults to 0:
        //
        //   0  If present, encoded soft-mask image information shall be ignored.
        //
        // So unless the image asks otherwise, the extra channel is dropped and the
        // image is painted from its colour channels alone. Refusing to decode it —
        // which is what an unrecognised component count used to do — loses the
        // whole image: one real file is a single 551x337 grey+alpha JPX covering
        // the page, and rejecting it rendered the page blank.
        //
        // How many of the decoded components are colour comes from the dictionary
        // when it says, per the same table's /ColorSpace entry (pdf.md:14487):
        // "If ColorSpace is present, any colour space specifications in the
        // JPEG2000 data shall be ignored."
        let declared = color_space.components();
        let decoded = usize::from(img.num_components);
        let colour_components = if dict.contains_key("ColorSpace") {
            declared
        } else {
            usize::from(img.colour_components)
        };
        if colour_components == 0 || decoded < colour_components || decoded > colour_components + 1
        {
            return Err(Error::Image("JPX components differ from the PDF colour space".into()));
        }

        if smask != 0 {
            if decoded != colour_components + 1 || !matches!(colour_components, 1 | 3 | 4) {
                return Err(Error::Unsupported(
                    "JPX embedded alpha requires Gray, RGB or CMYK and alpha".into(),
                ));
            }
            let count = rhwp_codec_resource::product(&[img.width as usize, img.height as usize, 4])
                .map_err(|_| Error::Image("JPX alpha size".into()))?;
            let mut rgba = rhwp_codec_resource::filled(0u8, count)
                .map_err(|_| Error::Image("JPX alpha allocation".into()))?;
            for (source, output) in img
                .samples
                .chunks_exact(decoded)
                .zip(rgba.chunks_exact_mut(4))
            {
                let alpha = source[colour_components];
                let mut color = [0; 4];
                for c in 0..colour_components {
                    color[c] = if smask == 2 {
                        unassociate_zero_matte(source[c], alpha)
                    } else {
                        source[c]
                    };
                }
                let rgb = direct_transform
                    .and_then(|t| t.convert_pixel(&color[..colour_components]))
                    .unwrap_or_else(|| match colour_components {
                        1 => [color[0]; 3],
                        3 => [color[0], color[1], color[2]],
                        _ => cmyk_pixel_to_rgb(color[0], color[1], color[2], color[3]),
                    });
                output[..3].copy_from_slice(&rgb);
                output[3] = alpha;
            }
            return Ok((
                ImageData::Raw {
                    pixels: rgba,
                    format: PixelFormat::RGBA,
                },
                img.width,
                img.height,
            ));
        }

        let format = match colour_components {
            1 => PixelFormat::Grayscale,
            3 => PixelFormat::RGB,
            4 => PixelFormat::CMYK,
            n => {
                return Err(Error::UnsupportedFilter(format!(
                    "JPXDecode: unsupported JPEG 2000 component count {n}"
                )))
            },
        };

        // Drop the trailing opacity channel(s) if the decode produced more
        // components than the colour space accounts for.
        let samples = if colour_components == decoded {
            img.samples
        } else {
            let px = img.samples.len() / decoded.max(1);
            let mut out = rhwp_codec_resource::capacity(
                rhwp_codec_resource::product(&[px, colour_components])
                    .map_err(|_| Error::Image("JPX colour size".into()))?,
            )
            .map_err(|_| Error::Image("JPX colour allocation".into()))?;
            rhwp_codec_resource::work(img.samples.len())
                .map_err(|_| Error::Image("JPX colour work".into()))?;
            for i in 0..px {
                let base = i * decoded;
                out.extend_from_slice(&img.samples[base..base + colour_components]);
            }
            log::debug!(
                "JPXDecode: {decoded} components decoded, {colour_components} are colour; \
             dropping the opacity channel per /SMaskInData default 0"
            );
            out
        };

        // The decoder may have chosen a lower resolution level than the `/Width`
        // and `/Height` in the dictionary, so the caller must take the geometry
        // from what came back rather than from the dictionary.
        Ok((
            ImageData::Raw {
                pixels: samples,
                format,
            },
            img.width,
            img.height,
        ))
    })
}

#[cfg(not(feature = "jpeg2000"))]
fn decode_jpx_image(
    _xobject: &crate::object::Object,
    _obj_ref: Option<ObjectRef>,
    _doc: Option<&crate::document::PdfDocument>,
    _color_space: &ColorSpace,
    _indexed: Option<(&IndexedResolution, Option<&crate::color::Transform>)>,
    _direct_transform: Option<&crate::color::Transform>,
    _target: Option<(u32, u32)>,
) -> Result<(ImageData, u32, u32)> {
    Err(Error::UnsupportedFilter(
        "JPXDecode disabled: enable RHWP `pdf-jpeg2000` (backend `jpeg2000`) at build time"
            .to_string(),
    ))
}

/// Expand an inline image's abbreviated dictionary (ISO 32000-1 §8.9.7 Table 91)
/// into the equivalent image XObject dictionary.
///
/// As well as expanding the abbreviations (`/W` → `/Width`, …) this supplies the
/// `/Subtype /Image` that an inline image never carries: §8.9.7 says an inline
/// image's dictionary holds "a subset of the entries in the image dictionary",
/// with the subtype implied by the `BI` operator rather than written out. Callers
/// hand the result to the image-XObject decoder, which *requires* `/Subtype` — so
/// without this the decoder rejects every inline image with "XObject missing
/// /Subtype", and the callers, which use `if let Ok(..)`, drop them SILENTLY.
pub fn expand_inline_image_dict(
    mut dict: std::collections::HashMap<String, crate::object::Object>,
) -> std::collections::HashMap<String, crate::object::Object> {
    use std::collections::HashMap;
    const KEY_ABBREVS: [(&str, &str); 9] = [
        ("W", "Width"),
        ("H", "Height"),
        ("CS", "ColorSpace"),
        ("BPC", "BitsPerComponent"),
        ("F", "Filter"),
        ("DP", "DecodeParms"),
        ("IM", "ImageMask"),
        ("I", "Interpolate"),
        ("D", "Decode"),
    ];
    let mut expanded = HashMap::new();
    // A dictionary carrying BOTH forms of one key (`/F` and `/Filter`) must
    // resolve the same way every run: the abbreviated form wins, matching
    // pdf.js's dict.get("F", "Filter"). Draining the abbreviations first makes
    // that precedence structural; deciding it inside a HashMap iteration made
    // it per-process hash-seed luck.
    for (abbrev, full) in KEY_ABBREVS {
        let value = match dict.remove(abbrev) {
            Some(v) => {
                dict.remove(full);
                Some(v)
            },
            None => dict.remove(full),
        };
        if let Some(v) = value {
            expanded.insert(full.to_string(), v);
        }
    }
    for (key, value) in dict {
        expanded.insert(key, value);
    }
    // §8.9.7 Table 92: inline images abbreviate the VALUES too, not just the
    // keys - `/CS /RGB`, `/F /Fl`. Expanding only the keys leaves the decoder
    // looking at a colour space called "RGB", which it does not know.
    for (key, map) in [
        ("ColorSpace", colorspace_abbrev as fn(&str) -> Option<&'static str>),
        ("Filter", filter_abbrev as fn(&str) -> Option<&'static str>),
    ] {
        if let Some(v) = expanded.remove(key) {
            expanded.insert(key.to_string(), expand_inline_abbrev(v, map));
        }
    }
    // §8.9.7: the subtype is implied by `BI`, never written in the dictionary.
    // The image-XObject decoder requires it, so supply it here. Do not clobber a
    // dictionary that somehow carries one already.
    expanded
        .entry("Subtype".to_string())
        .or_insert_with(|| crate::object::Object::Name("Image".to_string()));
    expanded
}

/// §8.9.7 Table 92 colour-space abbreviations. An unabbreviated name (or a name
/// we do not recognise, e.g. a `/Resources /ColorSpace` entry like `/CS0`) passes
/// through untouched.
fn colorspace_abbrev(name: &str) -> Option<&'static str> {
    match name {
        "G" => Some("DeviceGray"),
        "RGB" => Some("DeviceRGB"),
        "CMYK" => Some("DeviceCMYK"),
        "I" => Some("Indexed"),
        _ => None,
    }
}

/// §8.9.7 Table 92 filter abbreviations.
fn filter_abbrev(name: &str) -> Option<&'static str> {
    match name {
        "AHx" => Some("ASCIIHexDecode"),
        "A85" => Some("ASCII85Decode"),
        "LZW" => Some("LZWDecode"),
        "Fl" => Some("FlateDecode"),
        "RL" => Some("RunLengthDecode"),
        "CCF" => Some("CCITTFaxDecode"),
        "DCT" => Some("DCTDecode"),
        _ => None,
    }
}

/// Rewrite abbreviated names in an inline-image value. Handles a bare name, and
/// an array (a filter chain, or an `[/I /RGB 255 <lookup>]` indexed space, whose
/// BASE name is itself abbreviated).
fn expand_inline_abbrev(
    value: crate::object::Object,
    map: fn(&str) -> Option<&'static str>,
) -> crate::object::Object {
    use crate::object::Object;
    match value {
        Object::Name(n) => match map(&n) {
            Some(full) => Object::Name(full.to_string()),
            None => Object::Name(n),
        },
        Object::Array(items) => Object::Array(
            items
                .into_iter()
                .map(|item| expand_inline_abbrev(item, map))
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod inline_image_dict_tests {
    use super::*;
    use crate::object::Object;
    use std::collections::HashMap;

    fn dict(pairs: &[(&str, Object)]) -> HashMap<String, Object> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// The decoder requires `/Subtype`, and an inline image never carries one
    /// (§8.9.7 - it is implied by `BI`). Without this every inline image was
    /// rejected with "XObject missing /Subtype" and silently dropped.
    #[test]
    fn supplies_the_implied_image_subtype() {
        let out = expand_inline_image_dict(dict(&[
            ("W", Object::Integer(26)),
            ("H", Object::Integer(1)),
        ]));
        assert_eq!(out.get("Subtype"), Some(&Object::Name("Image".to_string())));
        assert_eq!(out.get("Width"), Some(&Object::Integer(26)));
        assert_eq!(out.get("Height"), Some(&Object::Integer(1)));
    }

    /// §8.9.7 Table 92: inline images abbreviate the VALUES too. Expanding only
    /// the keys left the decoder looking at a colour space named "RGB".
    #[test]
    fn expands_abbreviated_colour_space_and_filter_values() {
        let out = expand_inline_image_dict(dict(&[
            ("CS", Object::Name("RGB".to_string())),
            ("F", Object::Name("Fl".to_string())),
        ]));
        assert_eq!(out.get("ColorSpace"), Some(&Object::Name("DeviceRGB".to_string())));
        assert_eq!(out.get("Filter"), Some(&Object::Name("FlateDecode".to_string())));
    }

    #[test]
    fn expands_every_table_92_abbreviation() {
        for (abbr, full) in [
            ("G", "DeviceGray"),
            ("RGB", "DeviceRGB"),
            ("CMYK", "DeviceCMYK"),
            ("I", "Indexed"),
        ] {
            let out = expand_inline_image_dict(dict(&[("CS", Object::Name(abbr.to_string()))]));
            assert_eq!(
                out.get("ColorSpace"),
                Some(&Object::Name(full.to_string())),
                "colour space /{abbr}"
            );
        }
        for (abbr, full) in [
            ("AHx", "ASCIIHexDecode"),
            ("A85", "ASCII85Decode"),
            ("LZW", "LZWDecode"),
            ("Fl", "FlateDecode"),
            ("RL", "RunLengthDecode"),
            ("CCF", "CCITTFaxDecode"),
            ("DCT", "DCTDecode"),
        ] {
            let out = expand_inline_image_dict(dict(&[("F", Object::Name(abbr.to_string()))]));
            assert_eq!(out.get("Filter"), Some(&Object::Name(full.to_string())), "filter /{abbr}");
        }
    }

    /// A filter CHAIN, and an indexed space whose base name is itself abbreviated.
    #[test]
    fn expands_abbreviations_inside_arrays() {
        let out = expand_inline_image_dict(dict(&[
            (
                "F",
                Object::Array(vec![
                    Object::Name("A85".to_string()),
                    Object::Name("Fl".to_string()),
                ]),
            ),
            (
                "CS",
                Object::Array(vec![
                    Object::Name("I".to_string()),
                    Object::Name("RGB".to_string()),
                    Object::Integer(255),
                ]),
            ),
        ]));
        assert_eq!(
            out.get("Filter"),
            Some(&Object::Array(vec![
                Object::Name("ASCII85Decode".to_string()),
                Object::Name("FlateDecode".to_string()),
            ]))
        );
        assert_eq!(
            out.get("ColorSpace"),
            Some(&Object::Array(vec![
                Object::Name("Indexed".to_string()),
                Object::Name("DeviceRGB".to_string()),
                Object::Integer(255),
            ]))
        );
    }

    /// A name we do not recognise (a `/Resources /ColorSpace` entry like `/CS0`,
    /// or an already-full name) must pass through untouched.
    #[test]
    fn leaves_unabbreviated_and_named_spaces_alone() {
        let out = expand_inline_image_dict(dict(&[("CS", Object::Name("CS0".to_string()))]));
        assert_eq!(out.get("ColorSpace"), Some(&Object::Name("CS0".to_string())));
        let out = expand_inline_image_dict(dict(&[("CS", Object::Name("DeviceGray".to_string()))]));
        assert_eq!(out.get("ColorSpace"), Some(&Object::Name("DeviceGray".to_string())));
    }

    /// Both forms of one key in the same dictionary (the pdf-association
    /// duplicate-key fixture does this for /F//Filter, /W//Width, /DP//
    /// DecodeParms): the abbreviated form must win, and deterministically —
    /// before, the winner was HashMap iteration order, a fresh hash seed per
    /// process, and the fixture's image count flapped between runs.
    #[test]
    fn abbreviated_key_beats_its_full_twin() {
        let out = expand_inline_image_dict(dict(&[
            ("F", Object::Name("AHx".to_string())),
            ("Filter", Object::Name("A85".to_string())),
            ("W", Object::Integer(20)),
            ("Width", Object::Integer(999)),
            ("DP", Object::Null),
            ("DecodeParms", Object::Integer(15)),
        ]));
        assert_eq!(out.get("Filter"), Some(&Object::Name("ASCIIHexDecode".to_string())));
        assert_eq!(out.get("Width"), Some(&Object::Integer(20)));
        assert_eq!(out.get("DecodeParms"), Some(&Object::Null));
    }
}

#[cfg(test)]
mod indexed_tests {
    use super::*;

    #[test]
    fn expand_indexed_rgb_8bpc() {
        // 2x2 image, 4 palette entries, each RGB
        let palette = vec![
            0, 0, 0, // index 0 black
            255, 0, 0, // index 1 red
            0, 255, 0, // index 2 green
            0, 0, 255, // index 3 blue
        ];
        let raw = vec![0, 1, 2, 3];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 2, 2, 8).unwrap();
        assert_eq!(out, vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]);
    }

    #[test]
    fn expand_indexed_gray_base_to_rgb() {
        // Base color space is Grayscale, palette is 1 byte per entry
        let palette = vec![10, 128, 255];
        let raw = vec![0, 1, 2];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::Grayscale, 3, 1, 8).unwrap();
        assert_eq!(out, vec![10, 10, 10, 128, 128, 128, 255, 255, 255]);
    }

    #[test]
    fn expand_indexed_out_of_range_index_clamps_to_the_last_entry() {
        // ISO 32000-1:2008 §8.6.6.3 (`docs/spec/pdf.md`:11053-11054): an index
        // "outside the range 0 to hival … shall be adjusted to the nearest
        // value within that range". Adjusted, not zeroed — black is a colour
        // the file never named, and emitting it darkens the image.
        //
        // This previously asserted `[0, 0, 0]` for the out-of-range sample,
        // which pinned behaviour rather than a decision: the comment read only
        // "→ zeroed" and cited nothing.
        let palette = vec![10, 20, 30, 40, 50, 60];
        let raw = vec![0, 5];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 2, 1, 8).unwrap();
        assert_eq!(out, vec![10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn resolve_indexed_palette_truncates_to_hival() {
        use crate::object::Object;
        // [/Indexed /DeviceRGB 1 <inline palette>] — hival = 1, so 2 entries * 3 = 6 bytes.
        // Provide an oversized 12-byte palette; the extra 6 bytes must be dropped so
        // that indices > hival cannot pick up stray lookup data.
        let cs = Object::Array(vec![
            Object::Name("Indexed".to_string()),
            Object::Name("DeviceRGB".to_string()),
            Object::Integer(1),
            Object::String(vec![
                10, 20, 30, // entry 0
                40, 50, 60, // entry 1
                70, 80, 90, // stray — beyond hival
                100, 110, 120,
            ]),
        ]);
        let ir = resolve_indexed_palette(None, &cs).unwrap().unwrap();
        assert_eq!(ir.base_fmt, PixelFormat::RGB);
        assert_eq!(ir.palette, vec![10, 20, 30, 40, 50, 60]);
        assert!(ir.base_profile.is_none(), "DeviceRGB base has no ICC profile");
        let (fmt, palette) = (ir.base_fmt, ir.palette);

        // Index 2 (> hival) must now be treated as out-of-range → black pixel.
        let raw = vec![0, 1, 2];
        let out = expand_indexed_to_rgb(&raw, &palette, fmt, 3, 1, 8).unwrap();
        // Truncated to hival = 1 (two entries), so index 2 clamps to entry 1
        // per §8.6.6.3 rather than being zeroed.
        assert_eq!(out, vec![10, 20, 30, 40, 50, 60, 40, 50, 60]);
    }

    #[test]
    fn expand_indexed_cmyk_base_matches_cmyk_to_rgb() {
        // Palette has a single CMYK entry; expansion must match the shared helper.
        let palette = vec![64, 128, 192, 32];
        let raw = vec![0];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::CMYK, 1, 1, 8).unwrap();
        let expected = cmyk_pixel_to_rgb(64, 128, 192, 32);
        assert_eq!(out, expected.to_vec());
    }

    #[test]
    fn cmyk_pixel_uses_process_inks_not_additive() {
        // DeviceCMYK images convert via the process-ink corners (matching the
        // text/vector paths), NOT the naive additive `1 - min(1, C+K)`. 100% K is
        // the K ink #231F20 (not #000000); process cyan is #00ADEF (not #00FFFF).
        assert_eq!(cmyk_pixel_to_rgb(0, 0, 0, 255), [0x23, 0x1F, 0x20]);
        assert_eq!(cmyk_pixel_to_rgb(255, 0, 0, 0), [0x00, 0xAD, 0xEF]);
        // No ink at all is the paper.
        assert_eq!(cmyk_pixel_to_rgb(0, 0, 0, 0), [255, 255, 255]);
    }

    #[test]
    fn expand_indexed_1bpc_with_row_padding() {
        // 2-entry palette, 5x2 image at 1 bpc. 5 bits → 1 byte per row (3 bits padding).
        // Row 0 indices: 0,1,0,1,0 → top nibble 01010xxx = 0x50
        // Row 1 indices: 1,1,0,0,1 → top nibble 11001xxx = 0xC8
        let palette = vec![10, 20, 30, 200, 210, 220];
        let raw = vec![0x50, 0xC8];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 5, 2, 1).unwrap();
        assert_eq!(
            out,
            vec![
                10, 20, 30, 200, 210, 220, 10, 20, 30, 200, 210, 220, 10, 20, 30, // row 0
                200, 210, 220, 200, 210, 220, 10, 20, 30, 10, 20, 30, 200, 210, 220, // row 1
            ]
        );
    }

    #[test]
    fn expand_indexed_2bpc_with_row_padding() {
        // 4-entry palette, 3x1 image at 2 bpc. 6 bits → 1 byte per row (2 bits padding).
        // indices 0,1,2 → 00 01 10 xx → 0x18
        let palette = vec![
            0, 0, 0, // 0
            10, 20, 30, // 1
            40, 50, 60, // 2
            70, 80, 90, // 3
        ];
        let raw = vec![0x18];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 3, 1, 2).unwrap();
        assert_eq!(out, vec![0, 0, 0, 10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn expand_indexed_4bpc_packs_two_per_byte() {
        // 4x1 image, 4bpc: 2 indices per byte, high nibble first
        let palette = vec![
            0, 0, 0, // 0
            10, 20, 30, // 1
            40, 50, 60, // 2
            70, 80, 90, // 3
        ];
        // indices: 0,1,2,3 → packed: 0x01, 0x23
        let raw = vec![0x01, 0x23];
        let out = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 4, 1, 4).unwrap();
        assert_eq!(out, vec![0, 0, 0, 10, 20, 30, 40, 50, 60, 70, 80, 90]);
    }

    // ---- DoS / hardening guards for #324 ----

    #[test]
    fn expand_indexed_rejects_overflow_dimensions() {
        // Dimensions that overflow usize when computing w * h * 3. Previously
        // Vec::with_capacity(w*h*3) would panic or reserve absurd amounts.
        let palette = vec![0, 0, 0, 255, 0, 0];
        let raw = vec![0, 1];
        let huge = u32::MAX / 2;
        let result = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, huge, huge, 8);
        assert!(result.is_err(), "overflow dimensions must be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("overflow") || err.contains("exceeds"),
            "expected overflow/limit error, got: {err}"
        );
    }

    #[test]
    fn expand_indexed_rejects_truncated_stream() {
        // 10x10 8bpc image requires 100 index bytes. Supplying 10 used to
        // silently zero-pad the remaining rows; now it's an error.
        let palette = vec![10, 20, 30, 40, 50, 60];
        let raw = vec![0; 10];
        let result = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 10, 10, 8);
        assert!(result.is_err(), "truncated stream must be rejected");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("truncated"), "expected truncated error, got: {err}");
    }

    #[test]
    fn expand_indexed_rejects_output_over_cap() {
        // 12 000 × 12 000 × 3 = 432 MB > 256 MB guard. The MAX_INDEXED_OUTPUT_BYTES
        // check fires before we inspect `raw.len()`, so the test doesn't need to
        // allocate a 144 MB stream — an empty buffer is enough to prove the cap
        // rejects the request.
        let palette = vec![0, 0, 0];
        let raw: Vec<u8> = Vec::new();
        let result = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 12_000, 12_000, 8);
        assert!(result.is_err(), "oversized output must be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("guard limit") || err.contains("exceeds"),
            "expected output-size guard error, got: {err}"
        );
    }

    // ---- #338: bpc validation per ISO 32000-2 §8.9.5.1 ----

    #[test]
    fn expand_indexed_rejects_bpc_zero() {
        // bpc = 0 used to be coerced to 1 by `bpc.max(1)`, silently
        // accepting a malformed PDF. Now it must be rejected.
        let palette = vec![0, 0, 0, 255, 0, 0];
        let raw = vec![0xFF];
        let result = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 1, 1, 0);
        assert!(result.is_err(), "bpc=0 must be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("BitsPerComponent") || err.contains("bpc"),
            "expected bpc error, got: {err}"
        );
    }

    #[test]
    fn expand_indexed_rejects_unsupported_bpc() {
        // 3, 5, 6, 7, 9, 12, 16, … are all invalid for Indexed. Previously
        // the `_ => 0` arm in `read_index` silently mapped every pixel to
        // palette entry 0, returning a solid-color image. Now they're
        // rejected up front.
        let palette = vec![0, 0, 0, 255, 0, 0];
        let raw = vec![0xFF];
        for bpc in [3u8, 5, 6, 7, 9, 12, 16] {
            let result = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 1, 1, bpc);
            assert!(result.is_err(), "bpc={bpc} must be rejected");
        }
    }

    #[test]
    fn expand_indexed_accepts_all_spec_bpc_values() {
        // Sanity: 1, 2, 4, 8 must still all work.
        let palette = vec![0, 0, 0, 255, 0, 0, 10, 20, 30, 40, 50, 60];
        let raw = vec![0xFF];
        for bpc in [1u8, 2, 4, 8] {
            let result = expand_indexed_to_rgb(&raw, &palette, PixelFormat::RGB, 1, 1, bpc);
            assert!(result.is_ok(), "bpc={bpc} must be accepted, got {result:?}");
        }
    }

    // Regression test for #336. Per ISO 32000-1 §8.6.6.3, the lookup element of
    // `[/Indexed base hival lookup]` must be either a byte string or a stream.
    // Historical behaviour when it was neither: `resolve_indexed_palette` returned
    // `Ok(None)` and `extract_image_from_xobject` silently fell back to treating
    // the raw 1-byte/pixel index stream as 3-byte/pixel RGB, producing the
    // misleading "Invalid RGB image dimensions" error. The fix returns an
    // explicit `Error::Image("Unable to resolve Indexed color space palette")`.
    #[test]
    fn resolve_indexed_palette_array_lookup_returns_none() {
        use crate::object::Object;
        let cs = Object::Array(vec![
            Object::Name("Indexed".to_string()),
            Object::Name("DeviceRGB".to_string()),
            Object::Integer(1),
            // Lookup as Array-of-Array (not String or Stream) — unresolvable.
            Object::Array(vec![
                Object::Array(vec![Object::Integer(0), Object::Integer(0), Object::Integer(0)]),
                Object::Array(vec![
                    Object::Integer(255),
                    Object::Integer(255),
                    Object::Integer(255),
                ]),
            ]),
        ]);
        assert!(resolve_indexed_palette(None, &cs).unwrap().is_none());
    }

    #[test]
    fn extract_image_errors_when_indexed_lookup_is_array() {
        use crate::object::Object;
        use std::collections::HashMap;

        let mut dict = HashMap::new();
        dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
        dict.insert("Width".to_string(), Object::Integer(2));
        dict.insert("Height".to_string(), Object::Integer(1));
        dict.insert("BitsPerComponent".to_string(), Object::Integer(8));
        dict.insert(
            "ColorSpace".to_string(),
            Object::Array(vec![
                Object::Name("Indexed".to_string()),
                Object::Name("DeviceRGB".to_string()),
                Object::Integer(1),
                Object::Array(vec![Object::Integer(0), Object::Integer(0), Object::Integer(0)]),
            ]),
        );
        let xobject = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(&[0, 1]),
        };

        let err = extract_image_from_xobject(None, &xobject, None, None)
            .expect_err("Indexed with Array lookup must error");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("Unable to resolve Indexed color space palette"),
            "error message should identify palette-resolution failure, got: {msg}"
        );
        assert!(
            !msg.contains("Invalid RGB image dimensions"),
            "must not fall through to misleading RGB-dimension error, got: {msg}"
        );
    }

    // #337 Lab→XYZ→sRGB conversion tests

    #[test]
    fn lab_pixel_mid_gray() {
        // Lab(50, 0, 0) = perceptual mid-gray → sRGB ~(119, 119, 119).
        // Byte encoding: L=128, a=128, b=128.
        let d65: [f64; 3] = [0.9505, 1.0, 1.0890];
        let [r, g, b] = super::lab_pixel_to_rgb(128, 128, 128, d65);
        for (label, v, expected) in [("R", r, 119), ("G", g, 119), ("B", b, 119)] {
            let diff = (v as i32 - expected).abs();
            assert!(diff <= 3, "Lab(50,0,0) {label}: expected ~{expected}, got {v} (Δ={diff})");
        }
    }

    #[test]
    fn lab_pixel_white() {
        // Lab(100, 0, 0) = white → sRGB ~(255, 255, 255).
        // Byte encoding: L=255, a=128, b=128.
        let d65: [f64; 3] = [0.9505, 1.0, 1.0890];
        let [r, g, b] = super::lab_pixel_to_rgb(255, 128, 128, d65);
        for (label, v) in [("R", r), ("G", g), ("B", b)] {
            assert!(v >= 250, "Lab(100,0,0) {label}: expected ~255, got {v}");
        }
    }

    #[test]
    fn lab_pixel_black() {
        // Lab(0, 0, 0) = black → sRGB ~(0, 0, 0).
        // Byte encoding: L=0, a=128, b=128.
        let d65: [f64; 3] = [0.9505, 1.0, 1.0890];
        let [r, g, b] = super::lab_pixel_to_rgb(0, 128, 128, d65);
        for (label, v) in [("R", r), ("G", g), ("B", b)] {
            assert!(v <= 5, "Lab(0,0,0) {label}: expected ~0, got {v}");
        }
    }

    #[test]
    fn lab_pixel_red_tint() {
        // Lab(50, 80, 0) has a strong red-magenta tint.
        // Byte encoding: L=128, a=208 (128+80), b=128.
        let d65: [f64; 3] = [0.9505, 1.0, 1.0890];
        let [r, g, b] = super::lab_pixel_to_rgb(128, 208, 128, d65);
        assert!(r > g + 50, "Lab(50,80,0) should have R >> G: R={r}, G={g}");
        assert!(r > b, "Lab(50,80,0) should have R > B: R={r}, B={b}");
    }

    #[test]
    fn lab_palette_round_trip() {
        // 3-entry Lab palette → RGB palette should have 9 bytes.
        let d65: [f64; 3] = [0.9505, 1.0, 1.0890];
        let palette: Vec<u8> = vec![
            0, 128, 128, // black
            128, 128, 128, // mid-gray
            255, 128, 128, // white
        ];
        let rgb = super::lab_palette_to_rgb(&palette, d65);
        assert_eq!(rgb.len(), 9, "3 Lab entries → 9 RGB bytes");
        // Black entry: all near 0
        assert!(rgb[0] <= 5 && rgb[1] <= 5 && rgb[2] <= 5);
        // White entry: all near 255
        assert!(rgb[6] >= 250 && rgb[7] >= 250 && rgb[8] >= 250);
    }

    #[test]
    fn extract_lab_whitepoint_d65() {
        use crate::object::Object;
        let cs = Object::Array(vec![
            Object::Name("Lab".to_string()),
            Object::Dictionary({
                let mut d = std::collections::HashMap::new();
                d.insert(
                    "WhitePoint".to_string(),
                    Object::Array(vec![
                        Object::Real(0.9505),
                        Object::Real(1.0),
                        Object::Real(1.0890),
                    ]),
                );
                d
            }),
        ]);
        let wp = super::extract_lab_whitepoint(&cs);
        assert!((wp[0] - 0.9505).abs() < 1e-6);
        assert!((wp[1] - 1.0).abs() < 1e-6);
        assert!((wp[2] - 1.0890).abs() < 1e-6);
    }

    #[test]
    fn extract_lab_whitepoint_missing_falls_back_to_d65() {
        use crate::object::Object;
        let cs = Object::Name("Lab".to_string());
        let wp = super::extract_lab_whitepoint(&cs);
        assert!((wp[0] - 0.9505).abs() < 1e-6);
    }
}

// ── Phase 1 / Phase 2 split: enumerate-then-materialize image API ─────────────

/// A PDF stream filter as stored in the `/Filter` key of an image XObject.
///
/// Knowing the filter chain lets callers decide whether to decode (e.g. skip
/// decompression for JPEG re-embed pipelines that only need `raw_compressed_bytes`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum PdfFilter {
    /// JPEG (DCTDecode) — compressed bytes are a valid JPEG file.
    DCTDecode,
    /// JPEG 2000 (JPXDecode).
    JPXDecode,
    /// Deflate/zlib (FlateDecode).
    FlateDecode,
    /// LZW compression (LZWDecode).
    LZWDecode,
    /// CCITT Group 3/4 fax (CCITTFaxDecode).
    CCITTFaxDecode,
    /// JBIG2 bi-level compression.
    JBIG2Decode,
    /// ASCII hex encoding (ASCIIHexDecode).
    ASCIIHexDecode,
    /// ASCII base-85 encoding (ASCII85Decode).
    ASCII85Decode,
    /// Run-length encoding (RunLengthDecode).
    RunLengthDecode,
    /// Crypt filter (used with encrypted streams).
    Crypt,
    /// Any filter not listed above; carries the raw PDF name.
    Other(String),
}

impl PdfFilter {
    /// Map a PDF filter name (or its abbreviated form) to a `PdfFilter` variant.
    pub fn from_name(name: &str) -> Self {
        match Self::try_from_name(name, |value| Ok::<_, std::convert::Infallible>(value.to_owned()))
        {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    fn try_from_name<E>(
        name: &str,
        copy: impl FnOnce(&str) -> std::result::Result<String, E>,
    ) -> std::result::Result<Self, E> {
        Ok(match name {
            "DCTDecode" | "DCT" => PdfFilter::DCTDecode,
            "JPXDecode" => PdfFilter::JPXDecode,
            "FlateDecode" | "Fl" => PdfFilter::FlateDecode,
            "LZWDecode" | "LZW" => PdfFilter::LZWDecode,
            "CCITTFaxDecode" | "CCF" => PdfFilter::CCITTFaxDecode,
            "JBIG2Decode" => PdfFilter::JBIG2Decode,
            "ASCIIHexDecode" | "AHx" => PdfFilter::ASCIIHexDecode,
            "ASCII85Decode" | "A85" => PdfFilter::ASCII85Decode,
            "RunLengthDecode" | "RL" => PdfFilter::RunLengthDecode,
            "Crypt" => PdfFilter::Crypt,
            other => PdfFilter::Other(copy(other)?),
        })
    }
}

/// Parses the `/Filter` entry of an image dictionary into a `Vec<PdfFilter>`.
///
/// The spec allows either a single name (`/DCTDecode`) or an array of names
/// (`[/ASCII85Decode /FlateDecode]`).
pub(crate) fn parse_filter_chain(
    dict: &std::collections::HashMap<String, crate::object::Object>,
    doc: Option<&crate::document::PdfDocument>,
) -> Result<Vec<PdfFilter>> {
    use crate::object::Object;
    let Some(source) = dict.get("Filter") else {
        return Ok(Vec::new());
    };
    let source = resolve_image_object(doc, source)?;
    let values = match &*source {
        Object::Null => return Ok(Vec::new()),
        Object::Array(values) => values.as_slice(),
        value => std::slice::from_ref(value),
    };
    let budget = doc.and_then(|d| d.import_budget());
    if let Some(budget) = budget {
        if values.len() > budget.limits.filters {
            return Err(budget.reject("image filter count"));
        }
        budget.charge_entries(values.len())?;
        budget.charge_decoded(
            values
                .len()
                .checked_mul(std::mem::size_of::<PdfFilter>())
                .ok_or_else(|| budget.reject("image filter storage"))?,
        )?;
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(|_| match budget {
            Some(b) => b.reject("image filter allocation"),
            None => Error::Image("image filter allocation".into()),
        })?;
    for value in values {
        let value = resolve_image_object(doc, value)?;
        let name = value
            .as_name()
            .ok_or_else(|| Error::Image("image Filter must contain names".into()))?;
        output.push(PdfFilter::try_from_name(name, |name| -> Result<String> {
            match budget {
                Some(b) => String::from_utf8(b.copy_bytes(name.as_bytes())?)
                    .map_err(|_| Error::Image("invalid filter name".into())),
                None => Ok(name.to_owned()),
            }
        })?);
    }
    Ok(output)
}

/// Internal image source stored inside a [`PdfImageHandle`].
#[derive(Clone)]
enum PdfImageSource {
    /// Indirect Image XObject reference; loaded on demand.
    XObject(ObjectRef),
    /// Inline image: pre-built `Object::Stream` plus the raw compressed bytes.
    Inline {
        /// Synthetic `Object::Stream` built from the inline dict + data —
        /// ready to pass directly to `extract_image_from_xobject`.
        stream_object: crate::object::Object,
        /// Raw compressed bytes as they appeared between `ID` and `EI`.
        /// Stored as `crate::import_limits::StreamData` (cheaply cloneable, refcounted) so that
        /// the same allocation can be shared with the Stream data field
        /// without duplicating a potentially large JPEG/JBIG2/etc payload.
        compressed_bytes: crate::import_limits::StreamData,
    },
}

/// A lightweight handle to a PDF image that has **not** been decoded yet.
///
/// Created by [`crate::PdfDocument::page_image_handles`], which walks the page content
/// stream and reads XObject dictionary metadata without decompressing any stream.
/// Callers can inspect the metadata fields to decide which images to materialise,
/// then call [`decode`](PdfImageHandle::decode) or
/// [`raw_compressed_bytes`](PdfImageHandle::raw_compressed_bytes) only on those
/// they actually need.
///
/// # Example
///
/// ```no_run
/// # use pdf_oxide::PdfDocument;
/// # let bytes = std::fs::read("page.pdf").unwrap();
/// let doc = PdfDocument::from_bytes(bytes).unwrap();
/// // Phase 1: enumerate without decompression
/// let handles = doc.page_image_handles(0).unwrap();
/// // Phase 2: decode only images larger than a thumbnail
/// let images: Vec<_> = handles
///     .into_iter()
///     .filter(|h| h.width >= 200 && h.height >= 200)
///     .map(|h| h.decode())
///     .collect::<Result<_, _>>()
///     .unwrap();
/// ```
#[derive(Clone)]
#[non_exhaustive]
pub struct PdfImageHandle<'doc> {
    /// Image width in pixels (from XObject `/Width`).
    pub width: u32,
    /// Image height in pixels (from XObject `/Height`).
    pub height: u32,
    /// Colour space (from XObject `/ColorSpace`).
    pub color_space: ColorSpace,
    /// Bits per component (from XObject `/BitsPerComponent`).
    pub bits_per_component: u8,
    /// Compressed stream length in bytes (from XObject `/Length`).
    ///
    /// For inline images this is `data.len()` as stored between `ID` and `EI`.
    pub byte_size_compressed: u64,
    /// Ordered list of filters applied to the stream (outermost first).
    pub filter_chain: Vec<PdfFilter>,
    /// `true` if the image is an inline image (embedded in the content stream).
    pub is_inline: bool,
    /// Zero-based index of this image among all images painted on the page,
    /// in content-stream paint order.
    pub paint_order: usize,
    /// Lexicographic operator invocation path for interleaving paths/images.
    pub source_order: Vec<usize>,
    /// Nonstroking alpha constant at this image invocation.
    pub opacity: f32,
    /// Axis-aligned bounding box of this image in PDF user space, computed
    /// during Phase 1 by applying the current transformation matrix to the
    /// unit rectangle `[0,0,1,1]`.
    pub bbox: crate::geometry::Rect,
    /// Active clipping region in the same PDF user-space coordinates.
    pub clip: super::image_clip::ImageClip,
    /// Rotation angle in degrees (0, 90, 180, or 270), derived from the CTM
    /// during Phase 1.
    pub rotation_degrees: f32,

    // Internal fields
    ctm: crate::content::Matrix,
    doc: &'doc crate::document::PdfDocument,
    source: PdfImageSource,
    /// Active resource `/ColorSpace` subdictionary (name → resolved Object) for
    /// this image's scope, so `decode()` can resolve a resource-name
    /// `/ColorSpace` (e.g. `/CS0`) the same way the renderer does. Empty when the
    /// image's colour space needs no resource lookup (the common case), which
    /// preserves the original `color_space_map=None` decode path.
    color_space_resources: std::collections::HashMap<String, crate::object::Object>,
    /// For an Indexed image (`[/Indexed base hival lookup]`, §8.6.6.3), the
    /// resolved de-indexed *base* colour space; `None` for non-Indexed images.
    indexed_base: Option<ColorSpace>,
    /// ImageMask can omit BitsPerComponent; it still consumes current paint colour.
    uses_current_colour: bool,
    /// Current nonstroking solid colour at this invocation; None is unsupported.
    pub(crate) stencil_colour: Option<(f32, f32, f32)>,
    /// Same pattern-cell paint invocation across tiles: decode/mask once, then
    /// apply each placement's opacity, clip and CTM, copying pixels only on mutation.
    pub(crate) pattern_pixels:
        Option<std::sync::Arc<std::sync::OnceLock<(PdfImage, Option<bool>)>>>,
}

impl<'doc> PdfImageHandle<'doc> {
    /// Decode this image into a [`PdfImage`].
    ///
    /// This is the expensive operation: it decompresses the image stream,
    /// decodes pixels, and applies colour-space conversions as needed.
    ///
    /// Takes `&self` so a single handle supports a two-phase inspect → raw →
    /// decode flow ([`raw_compressed_bytes`](Self::raw_compressed_bytes) then
    /// `decode`) without re-enumerating the page.
    pub fn decode(&self) -> Result<PdfImage> {
        self.doc.check_import_budget()?;
        let (mut image, interpolation) = if let Some(cache) = &self.pattern_pixels {
            if cache.get().is_none() {
                let decoded = self.decode_base()?;
                let _ = cache.set(decoded);
            }
            let (image, interpolation) = cache.get().expect("initialized pattern pixels");
            if let Some(budget) = self.doc.import_budget() {
                budget.charge_cache_copy(std::mem::size_of::<PdfImage>())?;
            }
            (image.clone(), *interpolation)
        } else if let Some(key) = self.shared_base_key() {
            let hit = self
                .doc
                .import_image_cache
                .lock_or_recover()
                .get(&key)
                .cloned();
            if let Some(decoded) = hit {
                self.doc
                    .import_budget()
                    .expect("bounded shared image")
                    .charge_cache_copy(std::mem::size_of::<PdfImage>())?;
                decoded
            } else {
                let (mut image, interpolation) = self.decode_base()?;
                image.png_cache = PngCache(Some(Arc::new(OnceLock::new())));
                self.doc
                    .import_image_cache
                    .lock_or_recover()
                    .insert(key, (image.clone(), interpolation));
                self.doc.check_import_budget()?;
                (image, interpolation)
            }
        } else {
            self.decode_base()?
        };
        if self.opacity < 1.0 {
            if let Some(budget) = self.doc.import_budget() {
                image.apply_opacity(self.opacity, budget)?;
            }
        }
        self.doc.check_import_budget()?;

        // Use pre-computed bbox and rotation from Phase 1 — no need to call
        // back into document.rs helpers here.
        let needs_affine = matches!(
            self.clip,
            super::image_clip::ImageClip::Polygon(_) | super::image_clip::ImageClip::Complex(_)
        ) || self.ctm.b != 0.0
            || self.ctm.c != 0.0
            || self.ctm.a <= 0.0
            || self.ctm.d <= 0.0;
        if let (true, Some(budget)) = (needs_affine, self.doc.import_budget()) {
            let frame = self
                .clip
                .visible_bounds(self.bbox)?
                .ok_or_else(|| Error::Image("empty image clip".into()))?;
            let interpolate = image.interpolate.or(interpolation).unwrap_or(false);
            image.apply_affine_clip(self.ctm, &self.clip, frame, interpolate, budget)?;
            self.doc.check_import_budget()?;
            return Ok(image);
        }
        image.set_bbox(self.bbox);
        image.set_matrix([
            self.ctm.a, self.ctm.b, self.ctm.c, self.ctm.d, self.ctm.e, self.ctm.f,
        ]);
        image.set_rotation_degrees(self.rotation_degrees as i32);

        Ok(image)
    }

    // Only immutable, self-contained XObjects: a named colour space and a
    // stencil's current paint colour depend on the invocation's resources.
    // Decode/mask/ICC belong to the source; opacity, CTM and clip are applied
    // after this request-local cache on every placement.
    fn shared_base_key(&self) -> Option<ObjectRef> {
        if self.doc.import_budget().is_none()
            || self.uses_current_colour
            || !self.color_space_resources.is_empty()
            || self.indexed_base.is_some()
        {
            return None;
        }
        match self.source {
            PdfImageSource::XObject(key) => Some(key),
            _ => None,
        }
    }

    fn decode_base(&self) -> Result<(PdfImage, Option<bool>)> {
        use crate::extractors::extract_image_from_xobject;

        self.doc.check_import_budget()?;

        let xobject_for_extract;
        let (obj, obj_ref) = match &self.source {
            PdfImageSource::XObject(obj_ref) => {
                xobject_for_extract = self.doc.load_object(*obj_ref)?;
                (&xobject_for_extract, Some(*obj_ref))
            },
            PdfImageSource::Inline { stream_object, .. } => (stream_object, None),
        };

        let cs_map = if self.color_space_resources.is_empty() {
            None
        } else {
            Some(&self.color_space_resources)
        };
        let interpolation = if let Some(budget) = self.doc.import_budget() {
            obj.as_dict()
                .and_then(|d| d.get("Interpolate"))
                .map(|o| image_mask_limited::resolve(self.doc, o, budget))
                .transpose()?
                .as_deref()
                .map(|o| match o {
                    crate::object::Object::Null => Ok(false),
                    crate::object::Object::Boolean(v) => Ok(*v),
                    _ => Err(Error::Image("invalid image Interpolate flag".into())),
                })
                .transpose()?
                .or(Some(false))
        } else {
            None
        };
        let mut stencil = false;
        let mask = if let Some(budget) = self.doc.import_budget() {
            // The limited handle path imports the image as an independent IR
            // picture. Compose image alpha, while standalone stencils still
            // require the graphics state's current nonstroking colour.
            // Never return the base pixels while silently dropping a mask.
            // The legacy extractor remains available to renderer callers that
            // apply these masks themselves.
            let dict = obj
                .as_dict()
                .ok_or_else(|| Error::Image("invalid image object".into()))?;
            if let Some(value) = dict.get("ImageMask") {
                let value = resolve_image_object(Some(self.doc), value)?;
                stencil = match &*value {
                    crate::object::Object::Null | crate::object::Object::Boolean(false) => false,
                    crate::object::Object::Boolean(true) => true,
                    _ => return Err(Error::Image("invalid ImageMask boolean".into())),
                };
            }
            if stencil {
                None
            } else {
                image_mask_limited::Mask::read(self.doc, obj, cs_map, budget)?
            }
        } else {
            None
        };

        // Pass the active resource ColorSpace map so a resource-name
        // `/ColorSpace` (e.g. `/CS0`) resolves the same way the renderer does.
        // `None` when empty preserves the original decode path for the common
        // case where the image's colour space needs no resource lookup.
        let mut image = if stencil {
            image_mask_limited_extra::stencil(
                self.doc,
                obj,
                self.stencil_colour,
                self.doc.import_budget().expect("bounded stencil"),
            )?
        } else {
            extract_image_from_xobject(Some(self.doc), obj, obj_ref, cs_map)?
        };
        // Validate codec output against the source declaration before a mask
        // compositor may increase the bitmap resolution in the same unit square.
        if self.doc.import_budget().is_some()
            && (image.width != self.width || image.height != self.height)
        {
            return Err(Error::Unsupported(
                "decoded image dimensions differ from declaration".into(),
            ));
        }
        if let Some(budget) = self.doc.import_budget() {
            image.convert_to_srgb_with_budget(budget)?;
        }
        let mask_interpolation = mask
            .as_ref()
            .and_then(image_mask_limited::Mask::interpolation);
        if let Some(mask) = mask {
            mask.apply(&mut image, self.doc.import_budget().expect("bounded mask"))?;
        }
        image.interpolate = interpolation;
        if let (Some(base), Some(mask)) = (interpolation, mask_interpolation) {
            if base != mask {
                // A single RGBA texture can represent different sampling only
                // when the other plane is constant. Do not smooth hard mask
                // edges merely to interpolate the colour plane, or vice versa.
                let ImageData::Raw {
                    pixels,
                    format: PixelFormat::RGBA,
                } = image.data.as_ref()
                else {
                    unreachable!("composed mask is RGBA")
                };
                let budget = self.doc.import_budget().expect("bounded mask");
                let mut first_rgb: Option<[u8; 3]> = None;
                let (mut constant_rgb, mut constant_alpha) = (true, true);
                for (i, p) in pixels.chunks_exact(4).enumerate() {
                    if i % 4096 == 0 {
                        budget.check()?;
                    }
                    constant_alpha &= p[3] == pixels[3];
                    if p[3] != 0 {
                        let rgb = [p[0], p[1], p[2]];
                        constant_rgb &= first_rgb.is_none_or(|first| first == rgb);
                        first_rgb = Some(rgb);
                    }
                }
                if (!base && mask && constant_rgb) || (base && !mask && constant_alpha) {
                    image.interpolate = Some(true);
                } else {
                    // A precomposed RGBA resource cannot carry two independent
                    // sampling planes. Keep the previously supported compositor
                    // and renderer profile for this case; do not claim that a
                    // single texture's filtering implements both PDF flags.
                    // Separate-plane replay remains a follow-up contract.
                    image.interpolate = None;
                }
            }
        }
        Ok((image, interpolation))
    }

    /// Return the raw compressed bytes exactly as stored in the PDF stream,
    /// **without** decompressing them.
    ///
    /// For JPEG images (`filter_chain == [DCTDecode]`) these bytes form a valid
    /// JPEG file and can be written directly to disk or forwarded to a downstream
    /// pipeline without recompression.
    ///
    /// Takes `&self` so it can be combined with [`decode`](Self::decode) on the
    /// same handle (inspect → raw → decode) without re-enumerating the page.
    pub fn raw_compressed_bytes(&self) -> Result<Vec<u8>> {
        self.doc.check_import_budget()?;
        let copy = |bytes: &[u8]| match self.doc.import_budget() {
            Some(budget) => budget.copy_bytes(bytes),
            None => Ok(bytes.to_vec()),
        };
        match &self.source {
            PdfImageSource::XObject(obj_ref) => {
                let obj = self.doc.load_object(*obj_ref)?;
                match obj {
                    crate::object::Object::Stream { data, .. } => copy(&data),
                    _ => Err(crate::error::Error::Image("XObject is not a stream".to_string())),
                }
            },
            PdfImageSource::Inline {
                compressed_bytes, ..
            } => copy(compressed_bytes),
        }
    }

    /// For an Indexed image, the de-indexed *base* colour space.
    ///
    /// When [`color_space`](Self::color_space) is [`ColorSpace::Indexed`] the
    /// image samples are single palette indices (`components() == 1`) into an
    /// `[/Indexed base hival lookup]` array (§8.6.6.3); this returns the resolved
    /// `base` colour space — the space in which the de-indexed output pixels are
    /// expressed. Returns `None` for every non-Indexed image (and only if an
    /// Indexed `base` could not be parsed at all).
    pub fn indexed_base(&self) -> Option<ColorSpace> {
        self.indexed_base
    }
}

impl std::fmt::Debug for PdfImageHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfImageHandle")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("color_space", &self.color_space)
            .field("bits_per_component", &self.bits_per_component)
            .field("byte_size_compressed", &self.byte_size_compressed)
            .field("filter_chain", &self.filter_chain)
            .field("is_inline", &self.is_inline)
            .field("paint_order", &self.paint_order)
            .field("bbox", &self.bbox)
            .field("rotation_degrees", &self.rotation_degrees)
            .field("color_space_resources", &self.color_space_resources)
            .field("indexed_base", &self.indexed_base)
            .finish_non_exhaustive()
    }
}

/// Derive a rotation angle in degrees from a transformation matrix.
///
/// Computes `atan2(b, a)` and rounds to the nearest integer degree.
fn matrix_to_rotation(m: crate::content::Matrix) -> f32 {
    let angle_rad = m.b.atan2(m.a);
    let angle_deg = angle_rad.to_degrees();
    let normalized = angle_deg % 360.0;
    if normalized < 0.0 {
        normalized + 360.0
    } else {
        normalized
    }
}

/// Transform an axis-aligned bounding rectangle by a CTM.
///
/// Transforms all four corners and returns the axis-aligned bounding box of the
/// result, which correctly handles rotation, shear, and negative scaling.
fn transform_bbox_with_ctm(
    rect: &crate::geometry::Rect,
    ctm: crate::content::Matrix,
) -> crate::geometry::Rect {
    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x + rect.width;
    let y1 = rect.y + rect.height;

    let tx0 = ctm.a * x0 + ctm.c * y0 + ctm.e;
    let ty0 = ctm.b * x0 + ctm.d * y0 + ctm.f;

    let tx1 = ctm.a * x1 + ctm.c * y0 + ctm.e;
    let ty1 = ctm.b * x1 + ctm.d * y0 + ctm.f;

    let tx2 = ctm.a * x0 + ctm.c * y1 + ctm.e;
    let ty2 = ctm.b * x0 + ctm.d * y1 + ctm.f;

    let tx3 = ctm.a * x1 + ctm.c * y1 + ctm.e;
    let ty3 = ctm.b * x1 + ctm.d * y1 + ctm.f;

    let min_x = tx0.min(tx1).min(tx2).min(tx3);
    let max_x = tx0.max(tx1).max(tx2).max(tx3);
    let min_y = ty0.min(ty1).min(ty2).min(ty3);
    let max_y = ty0.max(ty1).max(ty2).max(ty3);

    crate::geometry::Rect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

/// Resolve a handle's reported colour space from a raw `/ColorSpace` entry.
///
/// Shared by the XObject and inline handle builders so the two paths cannot
/// drift. Returns the `(color_space, indexed_base)` pair stored on the handle:
///
/// - **Resource-name resolution.** If `entry` is an `Object::Name` that is *not*
///   one of the standard device spaces (`DeviceGray`/`DeviceRGB`/`DeviceCMYK`/
///   `Pattern`, which "always identify the corresponding colour spaces directly"
///   and "never refer to resources", §8.6.3/§8.9.7), it is looked up in
///   `color_space_resources` (the active `/Resources/ColorSpace` subdictionary),
///   mirroring the renderer / `extract_image_from_xobject`.
/// - **Indirect-ref hop.** If the entry (or looked-up value) is a `Reference`,
///   one `load_object` hop is resolved via `doc`.
/// - **Indexed (§8.6.6.3).** When the resolved object is an
///   `[/Indexed base hival lookup]` array, returns `(ColorSpace::Indexed,
///   Some(base))` with the de-indexed `base` resolved (including an indirect
///   `base` ref and inner refs, e.g. `[/ICCBased <stream_ref>]`), reusing the
///   same base-resolution logic as `resolve_indexed_palette`. If the base cannot
///   be parsed, returns `(ColorSpace::Indexed, None)`.
/// - Otherwise returns `(parse_color_space(resolved)?, None)`, falling back to
///   `DeviceRGB` on parse failure.
fn resolve_color_space_for_handle(
    entry: &crate::object::Object,
    color_space_resources: &std::collections::HashMap<String, crate::object::Object>,
    doc: Option<&crate::document::PdfDocument>,
) -> (ColorSpace, Option<ColorSpace>) {
    use crate::object::Object;
    use std::borrow::Cow;

    // (a) Resource-name → resolved Object (skip standard device names, which
    // always identify their space directly and never refer to resources).
    let after_name = match entry {
        Object::Name(name)
            if !matches!(name.as_str(), "DeviceGray" | "DeviceRGB" | "DeviceCMYK" | "Pattern") =>
        {
            color_space_resources.get(name).unwrap_or(entry)
        },
        _ => entry,
    };

    // (b) Resolve one indirect-ref hop.
    let resolved = match (after_name.as_reference(), doc) {
        (Some(r), Some(d)) => d
            .load_object(r)
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(after_name)),
        _ => Cow::Borrowed(after_name),
    };

    // (c) `[/Indexed base hival lookup]` (§8.6.6.3): report Indexed + base.
    if let Object::Array(arr) = resolved.as_ref() {
        if arr.first().and_then(|o| o.as_name()) == Some("Indexed") && arr.len() >= 2 {
            // Resolve the base object, mirroring resolve_indexed_palette: an
            // indirect base ref, plus inner refs of an array base such as
            // [/ICCBased <stream_ref>] so parse_color_space can read /N.
            let base_obj = if let Some(d) = doc {
                let outer = if let Some(r) = arr[1].as_reference() {
                    match d.load_object(r) {
                        Ok(value) => value,
                        Err(_) => match crate::object_copy::clone_object(
                            &arr[1],
                            d.import_budget().map(AsRef::as_ref),
                        ) {
                            Ok(value) => value,
                            Err(_) => return (ColorSpace::Indexed, None),
                        },
                    }
                } else {
                    match crate::object_copy::clone_object(
                        &arr[1],
                        d.import_budget().map(AsRef::as_ref),
                    ) {
                        Ok(value) => value,
                        Err(_) => return (ColorSpace::Indexed, None),
                    }
                };
                if let Object::Array(mut inner) = outer {
                    for item in inner.iter_mut() {
                        if let Some(r) = item.as_reference() {
                            if let Ok(resolved) = d.load_object(r) {
                                *item = resolved;
                            }
                        }
                    }
                    Object::Array(inner)
                } else {
                    outer
                }
            } else {
                arr[1].clone()
            };
            let base = parse_color_space(&base_obj).ok();
            return (ColorSpace::Indexed, base);
        }
    }

    // (d) Non-Indexed: parse directly, fall back to DeviceRGB.
    match parse_color_space(&resolved) {
        Ok(cs) => (cs, None),
        Err(_) => (ColorSpace::DeviceRGB, None),
    }
}

/// Build a `PdfImageHandle` from an Image XObject dictionary entry.
///
/// Returns `None` if the XObject reference cannot be resolved or the dict lacks
/// required fields (`Width`, `Height`), or if those fields contain non-positive
/// values.
pub(crate) fn image_handle_from_xobject<'doc>(
    doc: &'doc crate::document::PdfDocument,
    obj_ref: ObjectRef,
    xobject_dict: &std::collections::HashMap<String, crate::object::Object>,
    ctm: crate::content::Matrix,
    paint_order: usize,
    color_space_resources: &std::collections::HashMap<String, crate::object::Object>,
) -> Option<PdfImageHandle<'doc>> {
    // /Width and /Height may be indirect references (ISO 32000-1 §7.3.10);
    // resolve them the same way `extract_image_from_xobject` does.
    let resolve_int = |o: &crate::object::Object| -> Option<i64> {
        match o.as_reference() {
            Some(r) => doc.load_object(r).ok().and_then(|v| v.as_integer()),
            None => o.as_integer(),
        }
    };
    let w = xobject_dict
        .get("Width")
        .and_then(resolve_int)
        .filter(|&n| n > 0)
        .map(|n| n as u32)?;
    let h = xobject_dict
        .get("Height")
        .and_then(resolve_int)
        .filter(|&n| n > 0)
        .map(|n| n as u32)?;
    let bpc = xobject_dict
        .get("BitsPerComponent")
        .and_then(|o| o.as_integer())
        .unwrap_or(8) as u8;
    let byte_size = xobject_dict
        .get("Length")
        .and_then(|o| o.as_integer())
        .filter(|&n| n >= 0)
        .map(|n| n as u64)
        .unwrap_or(0);
    let filter_chain = parse_filter_chain(xobject_dict, Some(doc)).ok()?;

    // Resolve the reported colour space via the shared helper: resource-name →
    // map entry, one indirect-ref hop, and `[/Indexed base ...]` (§8.6.6.3) →
    // `Indexed` + de-indexed base. Default to DeviceRGB when `/ColorSpace` is
    // absent.
    let (color_space, indexed_base) = match xobject_dict.get("ColorSpace") {
        Some(entry) => resolve_color_space_for_handle(entry, color_space_resources, Some(doc)),
        None => (ColorSpace::DeviceRGB, None),
    };

    // Compute bbox and rotation in Phase 1 while the CTM is in scope.
    let unit_rect = crate::geometry::Rect::new(0.0, 0.0, 1.0, 1.0);
    let bbox = transform_bbox_with_ctm(&unit_rect, ctm);
    let rotation_degrees = matrix_to_rotation(ctm);

    Some(PdfImageHandle {
        width: w,
        height: h,
        color_space,
        bits_per_component: bpc,
        byte_size_compressed: byte_size,
        filter_chain,
        is_inline: false,
        paint_order,
        source_order: Vec::new(),
        opacity: 1.0,
        bbox,
        clip: super::image_clip::ImageClip::Unbounded,
        rotation_degrees,
        ctm,
        doc,
        source: PdfImageSource::XObject(obj_ref),
        // A colour-space array, including an indirect one, is self-contained.
        // Match the decoder: resolve the object first, then consult page
        // resources only if it is a name. A resolution failure is not ignored
        // by decode; it simply cannot make this source cacheable.
        color_space_resources: if xobject_dict.get("ColorSpace").is_some_and(|value| {
            matches!(resolve_image_object(Some(doc), value).as_deref(), Ok(Object::Array(_)))
        }) {
            std::collections::HashMap::new()
        } else {
            crate::object_copy::clone_dictionary(
                color_space_resources,
                doc.import_budget().map(AsRef::as_ref),
            )
            .ok()?
        },
        indexed_base,
        uses_current_colour: !matches!(
            xobject_dict.get("ImageMask"),
            None | Some(Object::Boolean(false) | Object::Null)
        ),
        stencil_colour: Some((0., 0., 0.)),
        pattern_pixels: None,
    })
}

/// Build a `PdfImageHandle` from an inline image (`BI`/`ID`/`EI` sequence).
pub(crate) fn image_handle_from_inline<'doc>(
    doc: &'doc crate::document::PdfDocument,
    dict: &std::collections::HashMap<String, crate::object::Object>,
    data: Vec<u8>,
    ctm: crate::content::Matrix,
    paint_order: usize,
    color_space_resources: &std::collections::HashMap<String, crate::object::Object>,
) -> Option<PdfImageHandle<'doc>> {
    use crate::object::Object;

    // Inline image dicts use abbreviated keys; expand them.
    let expanded = crate::extractors::expand_inline_image_dict(
        crate::object_copy::clone_dictionary(dict, doc.import_budget().map(AsRef::as_ref)).ok()?,
    );

    let w = expanded
        .get("Width")
        .and_then(|o| o.as_integer())
        .filter(|&n| n > 0)
        .map(|n| n as u32)?;
    let h = expanded
        .get("Height")
        .and_then(|o| o.as_integer())
        .filter(|&n| n > 0)
        .map(|n| n as u32)?;
    let bpc = expanded
        .get("BitsPerComponent")
        .and_then(|o| o.as_integer())
        .unwrap_or(8) as u8;
    let byte_size = data.len() as u64;
    let filter_chain = parse_filter_chain(&expanded, Some(doc)).ok()?;

    // Resolve the reported colour space via the same shared helper as the
    // XObject path so the two agree. For inline images a resource-name
    // `/ColorSpace` into `/Resources/ColorSpace` is explicitly legal (§8.9.7),
    // and `[/Indexed base ...]` (§8.6.6.3) reports `Indexed` + de-indexed base.
    let (color_space, indexed_base) = match expanded.get("ColorSpace") {
        Some(entry) => resolve_color_space_for_handle(entry, color_space_resources, Some(doc)),
        None => (ColorSpace::DeviceRGB, None),
    };

    // Compute bbox and rotation in Phase 1 while the CTM is in scope.
    let unit_rect = crate::geometry::Rect::new(0.0, 0.0, 1.0, 1.0);
    let bbox = transform_bbox_with_ctm(&unit_rect, ctm);
    let rotation_degrees = matrix_to_rotation(ctm);

    // Build a synthetic Object::Stream so decode() can call extract_image_from_xobject.
    // Share a single Bytes allocation between the Stream (for decode) and the
    // handle (for raw_compressed_bytes). Bytes is refcounted, so this avoids
    // duplicating potentially large image payloads (e.g. 10 MB JPEG → 20 MB RSS).
    let mut stream_dict = expanded;
    stream_dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
    // Synthetic inline streams must retain the originating request context,
    // just like parsed XObject streams. Otherwise decode_stream_data() takes
    // the legacy unbounded filter path after the content parser returns.
    let compressed_bytes = crate::import_limits::StreamData::bounded(data, doc.import_budget());
    let stream_object = Object::Stream {
        dict: stream_dict,
        data: compressed_bytes.clone(),
    };

    Some(PdfImageHandle {
        width: w,
        height: h,
        color_space,
        bits_per_component: bpc,
        byte_size_compressed: byte_size,
        filter_chain,
        is_inline: true,
        paint_order,
        source_order: Vec::new(),
        opacity: 1.0,
        bbox,
        clip: super::image_clip::ImageClip::Unbounded,
        rotation_degrees,
        ctm,
        doc,
        source: PdfImageSource::Inline {
            stream_object,
            compressed_bytes,
        },
        color_space_resources: crate::object_copy::clone_dictionary(
            color_space_resources,
            doc.import_budget().map(AsRef::as_ref),
        )
        .ok()?,
        indexed_base,
        uses_current_colour: true, // Inline sources never enter the XObject cache.
        stencil_colour: Some((0., 0., 0.)),
        pattern_pixels: None,
    })
}

#[cfg(test)]
mod handle_color_space_tests {
    use super::*;
    use crate::object::{Object, ObjectRef};
    use std::collections::HashMap;

    fn empty_map() -> HashMap<String, Object> {
        HashMap::new()
    }

    /// `[/Indexed /DeviceRGB 1 <00 00 00 FF FF FF>]` as a direct array.
    fn direct_indexed_rgb() -> Object {
        Object::Array(vec![
            Object::Name("Indexed".to_string()),
            Object::Name("DeviceRGB".to_string()),
            Object::Integer(1),
            Object::String(vec![0, 0, 0, 255, 255, 255]),
        ])
    }

    #[test]
    fn helper_direct_indexed_reports_indexed_with_rgb_base() {
        let entry = direct_indexed_rgb();
        let (cs, base) = resolve_color_space_for_handle(&entry, &empty_map(), None);
        assert_eq!(cs, ColorSpace::Indexed);
        assert_eq!(base, Some(ColorSpace::DeviceRGB));
    }

    #[test]
    fn helper_resource_name_resolves_to_device_gray() {
        // `/CS0` → /DeviceGray via the resource map (not a standard device name,
        // so it is looked up).
        let mut map = empty_map();
        map.insert("CS0".to_string(), Object::Name("DeviceGray".to_string()));
        let entry = Object::Name("CS0".to_string());
        let (cs, base) = resolve_color_space_for_handle(&entry, &map, None);
        assert_eq!(cs, ColorSpace::DeviceGray);
        assert_eq!(base, None);
    }

    #[test]
    fn helper_resource_name_resolves_to_indexed() {
        // `/CS0` → an Indexed array via the resource map: Indexed + base.
        let mut map = empty_map();
        map.insert("CS0".to_string(), direct_indexed_rgb());
        let entry = Object::Name("CS0".to_string());
        let (cs, base) = resolve_color_space_for_handle(&entry, &map, None);
        assert_eq!(cs, ColorSpace::Indexed);
        assert_eq!(base, Some(ColorSpace::DeviceRGB));
    }

    #[test]
    fn helper_standard_device_name_not_resource_resolved() {
        // A standard device name must resolve directly and never be looked up,
        // even if the map (incorrectly) shadows it.
        let mut map = empty_map();
        map.insert("DeviceRGB".to_string(), Object::Name("DeviceGray".to_string()));
        let entry = Object::Name("DeviceRGB".to_string());
        let (cs, base) = resolve_color_space_for_handle(&entry, &map, None);
        assert_eq!(cs, ColorSpace::DeviceRGB);
        assert_eq!(base, None);
    }

    /// Build a PDF whose object `99 0 obj` is the given colour-space object, so
    /// `resolve_color_space_for_handle(99 0 R, .., Some(doc))` can resolve the
    /// indirect-ref hop. The catalog/pages are minimal placeholders.
    fn pdf_with_cs_object(cs_body: &str) -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets: Vec<(u32, usize)> = Vec::new();

        offsets.push((1, pdf.len()));
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        offsets.push((2, pdf.len()));
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        offsets.push((3, pdf.len()));
        pdf.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 1 1] >>\nendobj\n",
        );

        offsets.push((99, pdf.len()));
        pdf.extend_from_slice(format!("99 0 obj\n{}\nendobj\n", cs_body).as_bytes());

        let xref_off = pdf.len();
        // Emit a single contiguous xref subsection covering 0..=99 with most
        // entries free; only the objects we wrote are marked in-use.
        let max_id = 99u32;
        pdf.extend_from_slice(format!("xref\n0 {}\n", max_id + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for id in 1..=max_id {
            if let Some((_, off)) = offsets.iter().find(|(oid, _)| *oid == id) {
                pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
            } else {
                pdf.extend_from_slice(b"0000000000 65535 f \n");
            }
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                max_id + 1,
                xref_off
            )
            .as_bytes(),
        );
        pdf
    }

    #[test]
    fn helper_indirect_indexed_array_de_indexed() {
        // `/ColorSpace 99 0 R` where 99 0 obj is `[/Indexed /DeviceRGB 1 <...>]`.
        let doc = crate::document::PdfDocument::from_bytes(pdf_with_cs_object(
            "[/Indexed /DeviceRGB 1 <000000FFFFFF>]",
        ))
        .expect("synthetic pdf parses");
        let entry = Object::Reference(ObjectRef::new(99, 0));
        let (cs, base) = resolve_color_space_for_handle(&entry, &empty_map(), Some(&doc));
        assert_eq!(cs, ColorSpace::Indexed);
        assert_eq!(base, Some(ColorSpace::DeviceRGB));
    }

    #[test]
    fn helper_indirect_plain_device_gray() {
        // `/ColorSpace 99 0 R` where 99 0 obj is `/DeviceGray`.
        let doc =
            crate::document::PdfDocument::from_bytes(pdf_with_cs_object("/DeviceGray")).unwrap();
        let entry = Object::Reference(ObjectRef::new(99, 0));
        let (cs, base) = resolve_color_space_for_handle(&entry, &empty_map(), Some(&doc));
        assert_eq!(cs, ColorSpace::DeviceGray);
        assert_eq!(base, None);
    }

    #[test]
    fn inline_indexed_consistent_with_xobject() {
        // The inline path uses the same helper, so a direct Indexed array must
        // report `Indexed` + DeviceRGB base — identical to the XObject contract.
        let entry = direct_indexed_rgb();
        let (xobj_cs, xobj_base) = resolve_color_space_for_handle(&entry, &empty_map(), None);
        // Inline path: same helper, same inputs.
        let (inline_cs, inline_base) = resolve_color_space_for_handle(&entry, &empty_map(), None);
        assert_eq!(xobj_cs, inline_cs);
        assert_eq!(xobj_base, inline_base);
        assert_eq!(inline_cs, ColorSpace::Indexed);
        assert_eq!(inline_base, Some(ColorSpace::DeviceRGB));
    }

    /// Build a PDF with an image XObject (object 99) so a handle can be built
    /// against a real document and `decode()` exercised. The image is a 1×1
    /// uncompressed sample; `cs_name` is written verbatim as its `/ColorSpace`.
    fn pdf_with_image_xobject(cs_name: &str, sample: &[u8]) -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets: Vec<(u32, usize)> = Vec::new();

        offsets.push((1, pdf.len()));
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        offsets.push((2, pdf.len()));
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        offsets.push((3, pdf.len()));
        pdf.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 1 1] >>\nendobj\n",
        );

        offsets.push((99, pdf.len()));
        let header = format!(
            "99 0 obj\n<< /Type /XObject /Subtype /Image /Width 1 /Height 1 \
             /BitsPerComponent 8 /ColorSpace {} /Length {} >>\nstream\n",
            cs_name,
            sample.len()
        );
        pdf.extend_from_slice(header.as_bytes());
        pdf.extend_from_slice(sample);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        let xref_off = pdf.len();
        let max_id = 99u32;
        pdf.extend_from_slice(format!("xref\n0 {}\n", max_id + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for id in 1..=max_id {
            if let Some((_, off)) = offsets.iter().find(|(oid, _)| *oid == id) {
                pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
            } else {
                pdf.extend_from_slice(b"0000000000 65535 f \n");
            }
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                max_id + 1,
                xref_off
            )
            .as_bytes(),
        );
        pdf
    }

    #[test]
    fn decode_resolves_resource_name_cs() {
        // Image XObject with `/ColorSpace /CS0`; the page `/Resources/ColorSpace`
        // maps `/CS0 → /DeviceGray`. Before the fix, decode() failed with
        // "Unsupported color space: CS0"; now it succeeds.
        let doc = crate::document::PdfDocument::from_bytes(pdf_with_image_xobject("/CS0", &[128]))
            .unwrap();
        let obj_ref = ObjectRef::new(99, 0);
        let xobj = doc.load_object(obj_ref).unwrap();
        let dict = xobj.as_dict().expect("image xobject dict").clone();

        let mut map = empty_map();
        map.insert("CS0".to_string(), Object::Name("DeviceGray".to_string()));

        let handle = image_handle_from_xobject(
            &doc,
            obj_ref,
            &dict,
            crate::content::Matrix::identity(),
            0,
            &map,
        )
        .expect("handle builds");

        assert_eq!(handle.color_space, ColorSpace::DeviceGray);
        assert!(handle.indexed_base().is_none());
        // decode() must now resolve /CS0 via the stored resource map.
        let image = handle.decode().expect("decode resolves resource-name CS");
        assert_eq!(*image.color_space(), ColorSpace::DeviceGray);
    }

    #[test]
    fn xobject_direct_indexed_handle_reports_indexed_with_base() {
        let doc = crate::document::PdfDocument::from_bytes(pdf_with_image_xobject(
            "[/Indexed /DeviceRGB 1 <000000FFFFFF>]",
            &[0],
        ))
        .unwrap();
        let obj_ref = ObjectRef::new(99, 0);
        let dict = doc.load_object(obj_ref).unwrap().as_dict().unwrap().clone();
        let handle = image_handle_from_xobject(
            &doc,
            obj_ref,
            &dict,
            crate::content::Matrix::identity(),
            0,
            &empty_map(),
        )
        .unwrap();
        assert_eq!(handle.color_space, ColorSpace::Indexed);
        assert_eq!(handle.indexed_base(), Some(ColorSpace::DeviceRGB));
    }
}

#[cfg(test)]
mod png_bytes_panic_safety_tests {
    use super::*;

    /// A raw RGB buffer longer than width×height×3 (e.g. a 16-bit-per-component
    /// image whose samples were not collapsed to 8-bit) must NOT panic the PNG
    /// encoder — it must surface a recoverable `Error` that crosses the FFI
    /// boundary so Python/other callers can catch it. Regression for the
    /// `assertion left == right failed: Invalid buffer length` panic.
    #[test]
    fn to_png_bytes_rejects_oversized_rgb_buffer_without_panicking() {
        let (w, h) = (4u32, 2u32);
        // Twice the bytes a 4x2 RGB image needs (mimics undownsampled 16-bit).
        let pixels = vec![0u8; (w * h * 3 * 2) as usize];
        let img = PdfImage::new(
            w,
            h,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Raw {
                pixels,
                format: PixelFormat::RGB,
            },
        );
        let result = img.to_png_bytes();
        assert!(
            result.is_err(),
            "oversized RGB buffer must yield a recoverable Err, not a panic"
        );
    }

    /// A correctly sized 8-bit RGB buffer (the shape produced after 16-bit
    /// samples are downsampled at parse time) encodes to a valid PNG.
    #[test]
    fn to_png_bytes_encodes_correctly_sized_rgb() {
        let (w, h) = (4u32, 2u32);
        let pixels = vec![128u8; (w * h * 3) as usize];
        let img = PdfImage::new(
            w,
            h,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Raw {
                pixels,
                format: PixelFormat::RGB,
            },
        );
        let png = img.to_png_bytes().expect("correctly sized RGB must encode");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']), "valid PNG signature");
    }

    /// The 16→8 reduction rounds `v*255/65535` to nearest and hits the endpoints
    /// exactly, rather than flooring via a high-byte drop (WS1.8b). The floor
    /// form `v >> 8` would map 0xFF80 → 0xFF (255) here too, but biases the
    /// mid-range and interior values downward; these anchors pin the rounding.
    #[test]
    fn reduce_16_to_8_rounds_to_nearest() {
        assert_eq!(reduce_16_to_8(0x00, 0x00), 0, "black stays black");
        assert_eq!(reduce_16_to_8(0xFF, 0xFF), 255, "white stays white");
        // 0x0080 = 128: 128*255/65535 = 0.498 → rounds to 0; high-byte drop also 0.
        assert_eq!(reduce_16_to_8(0x00, 0x80), 0);
        // 0x0100 = 256: 256*255/65535 = 0.996 → rounds to 1 (floor >> 8 = 1 too).
        assert_eq!(reduce_16_to_8(0x01, 0x00), 1);
        // 0x8080 = 32896: 32896*255/65535 = 127.998 → 128; floor >>8 = 0x80 = 128.
        assert_eq!(reduce_16_to_8(0x80, 0x80), 128);
        // 0xFF7F = 65407: 65407*255/65535 = 254.5 → rounds up to 255; >>8 = 255.
        assert_eq!(reduce_16_to_8(0xFF, 0x7F), 255);
        // Monotonic and full-range: every high-byte value round-trips near itself.
        for hi in 0u8..=255 {
            let out = reduce_16_to_8(hi, hi);
            assert!(
                out >= hi.saturating_sub(1) && out <= hi.saturating_add(1),
                "reduction stays within 1 LSB of the high byte for hi={hi}"
            );
        }
    }
}

#[cfg(test)]
mod ccitt_decode_array_polarity_tests {
    use super::*;
    use crate::object::Object;
    use std::collections::HashMap;

    /// Pack an MSB-first "0"/"1" bit string into bytes, zero-padding the
    /// final byte (same convention as the CCITT decoder's own hand-built
    /// codestream tests in `src/decoders/ccitt.rs`).
    fn pack_bits(bits: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut acc = 0u8;
        let mut n = 0u8;
        for ch in bits.chars() {
            acc = (acc << 1) | (ch == '1') as u8;
            n += 1;
            if n == 8 {
                bytes.push(acc);
                acc = 0;
                n = 0;
            }
        }
        if n > 0 {
            bytes.push(acc << (8 - n));
        }
        bytes
    }

    /// Hand-built CCITT G4 (T.6) codestream for an 8x3 bilevel image whose
    /// *raw* decoded runs are black-majority: rows 0-1 solid black, row 2
    /// black with a 2-pixel-wide white notch at columns 3-4. Every run uses
    /// Horizontal mode ("001" + a Modified-Huffman white-run code + a
    /// black-run code from ITU-T T.4 - the same tables `decode_row_g4`
    /// reads), which is reference-line-independent, so each row's bits are
    /// self-contained and can be verified without tracing 2D prediction:
    ///   row 0/1: white-run 0 ("00110101"), black-run 8 ("000101")
    ///   row 2:   white-run 0, black-run 3 ("10"); white-run 2 ("0111"),
    ///            black-run 3
    /// This mirrors the real corpus defect (govdocs 00339_005342 page 0):
    /// some scanners emit a black-majority raw codestream and rely on the
    /// image's /Decode [1 0] to restore the true white-majority page.
    fn black_majority_g4_stream() -> Vec<u8> {
        let row_all_black = format!("001{}{}", "00110101", "000101");
        let row_notch = format!("001{}{}001{}{}", "00110101", "10", "0111", "10");
        pack_bits(&format!("{row_all_black}{row_all_black}{row_notch}"))
    }

    /// Build a minimal CCITTFaxDecode image XObject (8x3, K=-1) wrapping
    /// `black_majority_g4_stream`, with an optional `/Decode` override.
    fn ccitt_xobject(decode: Option<[i64; 2]>) -> Object {
        let mut decode_parms = HashMap::new();
        decode_parms.insert("K".to_string(), Object::Integer(-1));
        decode_parms.insert("Columns".to_string(), Object::Integer(8));
        decode_parms.insert("Rows".to_string(), Object::Integer(3));

        let mut dict = HashMap::new();
        dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
        dict.insert("Width".to_string(), Object::Integer(8));
        dict.insert("Height".to_string(), Object::Integer(3));
        dict.insert("BitsPerComponent".to_string(), Object::Integer(1));
        dict.insert("ColorSpace".to_string(), Object::Name("DeviceGray".to_string()));
        dict.insert("Filter".to_string(), Object::Name("CCITTFaxDecode".to_string()));
        dict.insert("DecodeParms".to_string(), Object::Dictionary(decode_parms));
        if let Some([lo, hi]) = decode {
            dict.insert(
                "Decode".to_string(),
                Object::Array(vec![Object::Integer(lo), Object::Integer(hi)]),
            );
        }

        Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from(black_majority_g4_stream()),
        }
    }

    /// Decode `xobject` to a Luma8 image and count white (0xFF) vs black
    /// (0x00) pixels.
    fn white_black_counts(xobject: &Object) -> (u32, u32) {
        let img = extract_image_from_xobject(None, xobject, None, None)
            .expect("decode hand-built CCITT test image");
        let luma = img
            .to_dynamic_image()
            .expect("decode CCITT test image pixels")
            .into_luma8();
        let white = luma.iter().filter(|&&v| v == 0xFF).count() as u32;
        let black = luma.iter().filter(|&&v| v == 0x00).count() as u32;
        (white, black)
    }

    /// Baseline: no `/Decode` override (implicit default `[0 1]`), no
    /// `/BlackIs1`. The raw codestream is black-majority (22/24 px), so the
    /// correctly-decoded image must stay black-majority - this un-inverted
    /// case must keep working after the `/Decode` fix below.
    #[test]
    fn default_decode_keeps_raw_black_majority_polarity() {
        let (white, black) = white_black_counts(&ccitt_xobject(None));
        assert_eq!(white + black, 24);
        assert!(
            black > white,
            "expected black-majority (raw, no /Decode override), got white={white} black={black}"
        );
    }

    /// The corpus bug (govdocs 00339_005342 page 0): `/Decode [1 0]` on the
    /// image XObject must invert CCITT polarity, turning this same
    /// black-majority raw codestream into a white-majority image - a
    /// mostly-white page with a small black mark - matching poppler.
    /// Before the fix, `extract_image_from_xobject` never read `/Decode`
    /// for the CCITT path, so this asserted black-majority (the bug).
    #[test]
    fn decode_1_0_inverts_to_white_majority() {
        let (white, black) = white_black_counts(&ccitt_xobject(Some([1, 0])));
        assert_eq!(white + black, 24);
        assert!(
            white > black,
            "expected white-majority under /Decode [1 0], got white={white} black={black}"
        );
        assert_eq!(black, 2, "exactly the 2-pixel notch should render black (the 'mark')");
    }

    /// `/Decode [0 1]` written out explicitly is the non-inverted default
    /// and must behave identically to an absent `/Decode` entry.
    #[test]
    fn decode_0_1_is_a_no_op() {
        let (white, black) = white_black_counts(&ccitt_xobject(Some([0, 1])));
        assert_eq!((white, black), white_black_counts(&ccitt_xobject(None)));
    }
}

/// Regression coverage for the non-CCITT 1-bit `/DeviceGray` image path:
/// a raw packed-bit image (no `/Filter`, or `/Filter /FlateDecode`) used
/// to be unconditionally force-fed through the CCITT decompressor
/// (`to_dynamic_image`'s single `bits_per_component == 1 &&
/// DeviceGray` branch had no filter check), which fails to decode
/// already-unpacked bits as if they were CCITT-compressed data and drops
/// the image entirely. `/ImageMask` variants of the same filters were
/// unaffected (they go through the separate `render_image_mask` bit
/// unpacker), which is why the bug only showed on plain (non-mask)
/// images.
#[cfg(test)]
mod non_ccitt_1bpc_devicegray_tests {
    use super::*;
    use crate::object::Object;
    use std::collections::HashMap;

    /// 8x3 raw packed-bit image: rows 0-1 all-black, row 2 black with a
    /// 2-pixel-wide white notch at columns 3-4 (mirrors the CCITT
    /// polarity tests' pattern for an easy visual cross-check). Under the
    /// default `/Decode [0 1]`, sample bit 0 -> black, bit 1 -> white.
    fn black_majority_rows() -> [u8; 3] {
        [0x00, 0x00, 0x18]
    }

    /// Build a minimal 1-bit `/DeviceGray` image XObject wrapping
    /// `black_majority_rows`, either uncompressed or FlateDecode-filtered,
    /// with an optional `/Decode` override.
    fn devicegray_1bpc_xobject(flate: bool, decode: Option<[i64; 2]>) -> Object {
        let raw: Vec<u8> = black_majority_rows().to_vec();
        let (filter, data) = if flate {
            use flate2::write::ZlibEncoder;
            use flate2::Compression;
            use std::io::Write;
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&raw).expect("flate-compress test bitmap");
            (Some("FlateDecode"), encoder.finish().expect("finish flate stream"))
        } else {
            (None, raw)
        };

        let mut dict = HashMap::new();
        dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
        dict.insert("Width".to_string(), Object::Integer(8));
        dict.insert("Height".to_string(), Object::Integer(3));
        dict.insert("BitsPerComponent".to_string(), Object::Integer(1));
        dict.insert("ColorSpace".to_string(), Object::Name("DeviceGray".to_string()));
        if let Some(f) = filter {
            dict.insert("Filter".to_string(), Object::Name(f.to_string()));
        }
        if let Some([lo, hi]) = decode {
            dict.insert(
                "Decode".to_string(),
                Object::Array(vec![Object::Integer(lo), Object::Integer(hi)]),
            );
        }

        Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from(data),
        }
    }

    fn white_black_counts(xobject: &Object) -> (u32, u32) {
        let img = extract_image_from_xobject(None, xobject, None, None)
            .expect("decode hand-built non-CCITT 1bpc test image");
        let luma = img
            .to_dynamic_image()
            .expect("decode non-CCITT 1bpc test image pixels")
            .into_luma8();
        let white = luma.iter().filter(|&&v| v == 0xFF).count() as u32;
        let black = luma.iter().filter(|&&v| v == 0x00).count() as u32;
        (white, black)
    }

    /// Before the fix this failed outright (the CCITT decompressor
    /// rejects raw packed bits as malformed input), not just mis-decoded.
    #[test]
    fn uncompressed_1bpc_devicegray_decodes_without_ccitt() {
        let (white, black) = white_black_counts(&devicegray_1bpc_xobject(false, None));
        assert_eq!(white + black, 24);
        assert_eq!(black, 22, "22 of 24 pixels are black per the hand-built bitmap");
        assert_eq!(white, 2, "the 2-pixel notch at row 2 cols 3-4 is white");
    }

    #[test]
    fn flate_decoded_1bpc_devicegray_decodes_without_ccitt() {
        let (white, black) = white_black_counts(&devicegray_1bpc_xobject(true, None));
        assert_eq!(white + black, 24);
        assert_eq!(black, 22);
        assert_eq!(white, 2);
    }

    /// `/Decode [1 0]` inverts polarity for the non-CCITT path exactly
    /// like it does for CCITT (ISO 32000-1 §8.9.5.2 Table 90) — the
    /// bug's fallback path always produced a fully dropped image, so
    /// there was no polarity to even get wrong before this fix.
    #[test]
    fn decode_1_0_inverts_polarity_on_non_ccitt_path() {
        let (white, black) = white_black_counts(&devicegray_1bpc_xobject(true, Some([1, 0])));
        assert_eq!(white + black, 24);
        assert_eq!(white, 22, "polarity inverted: majority is now white");
        assert_eq!(black, 2, "the notch is now the black pixels");
    }
}
