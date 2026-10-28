//! Image handling for PDF generation.
//!
//! This module provides support for embedding images in PDF documents.
//! Per PDF spec Section 8.9, images are represented as XObjects.
//!
//! # Supported Formats
//!
//! - **JPEG**: Pass-through embedding using DCTDecode filter
//! - **PNG**: Deflate compression with predictor support
//!
//! # Color Spaces
//!
//! - DeviceRGB (3 components)
//! - DeviceGray (1 component)
//! - DeviceCMYK (4 components)

use std::collections::HashMap;
use std::io::Write;

use crate::object::Object;

/// Image format for PDF embedding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG image (DCTDecode filter)
    Jpeg,
    /// PNG image (FlateDecode filter)
    Png,
    /// Raw uncompressed image data
    Raw,
}

/// Color space for image data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// Grayscale (1 component per pixel)
    DeviceGray,
    /// RGB color (3 components per pixel)
    DeviceRGB,
    /// CMYK color (4 components per pixel)
    DeviceCMYK,
}

impl ColorSpace {
    /// Get the number of color components.
    pub fn components(&self) -> u8 {
        match self {
            ColorSpace::DeviceGray => 1,
            ColorSpace::DeviceRGB => 3,
            ColorSpace::DeviceCMYK => 4,
        }
    }

    /// Get the PDF name for this color space.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ColorSpace::DeviceGray => "DeviceGray",
            ColorSpace::DeviceRGB => "DeviceRGB",
            ColorSpace::DeviceCMYK => "DeviceCMYK",
        }
    }
}

/// Image data for PDF embedding.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Bits per component (usually 8)
    pub bits_per_component: u8,
    /// Color space
    pub color_space: ColorSpace,
    /// Image format
    pub format: ImageFormat,
    /// Raw or encoded image data
    pub data: Vec<u8>,
    /// Optional soft mask (alpha channel) data
    pub soft_mask: Option<Vec<u8>>,
}

impl ImageData {
    /// Create new image data.
    pub fn new(width: u32, height: u32, color_space: ColorSpace, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            bits_per_component: 8,
            color_space,
            format: ImageFormat::Raw,
            data,
            soft_mask: None,
        }
    }

    /// Load a JPEG image from raw JPEG data.
    ///
    /// JPEG images can be embedded directly without transcoding.
    pub fn from_jpeg(data: Vec<u8>) -> Result<Self, ImageError> {
        // Parse JPEG header to get dimensions and color info
        let (width, height, color_space) = parse_jpeg_header(&data)?;

        Ok(Self {
            width,
            height,
            bits_per_component: 8,
            color_space,
            format: ImageFormat::Jpeg,
            data,
            soft_mask: None,
        })
    }

    /// Load a PNG image from raw PNG data.
    pub fn from_png(data: &[u8]) -> Result<Self, ImageError> {
        use image::GenericImageView;

        let img = image::load_from_memory_with_format(data, image::ImageFormat::Png)
            .map_err(|e| ImageError::DecodeError(e.to_string()))?;

        let (width, height) = img.dimensions();

        // Determine color space and extract pixel data
        let (color_space, pixels, alpha) = match img.color() {
            image::ColorType::L8 | image::ColorType::L16 => {
                let gray = img.to_luma8();
                (ColorSpace::DeviceGray, gray.into_raw(), None)
            },
            image::ColorType::La8 | image::ColorType::La16 => {
                let rgba = img.to_luma_alpha8();
                let mut gray = Vec::with_capacity((width * height) as usize);
                let mut alpha_channel = Vec::with_capacity((width * height) as usize);
                for pixel in rgba.pixels() {
                    gray.push(pixel.0[0]);
                    alpha_channel.push(pixel.0[1]);
                }
                (ColorSpace::DeviceGray, gray, Some(alpha_channel))
            },
            image::ColorType::Rgb8 | image::ColorType::Rgb16 => {
                let rgb = img.to_rgb8();
                (ColorSpace::DeviceRGB, rgb.into_raw(), None)
            },
            image::ColorType::Rgba8 | image::ColorType::Rgba16 => {
                let rgba = img.to_rgba8();
                let mut rgb = Vec::with_capacity((width * height * 3) as usize);
                let mut alpha_channel = Vec::with_capacity((width * height) as usize);
                for pixel in rgba.pixels() {
                    rgb.push(pixel.0[0]);
                    rgb.push(pixel.0[1]);
                    rgb.push(pixel.0[2]);
                    alpha_channel.push(pixel.0[3]);
                }
                (ColorSpace::DeviceRGB, rgb, Some(alpha_channel))
            },
            _ => {
                // Fall back to RGB
                let rgb = img.to_rgb8();
                (ColorSpace::DeviceRGB, rgb.into_raw(), None)
            },
        };

        // Compress pixel data: add PNG None-filter byte (0x00) per scanline
        // then Flate. This produces data valid for DecodeParms Predictor=15.
        let components = color_space.components();
        let compressed = compress_image_data(&pixels, width, components)?;

        Ok(Self {
            width,
            height,
            bits_per_component: 8,
            color_space,
            format: ImageFormat::Png,
            data: compressed,
            // Alpha is 1 component (gray) per pixel.
            soft_mask: alpha
                .map(|a| compress_image_data(&a, width, 1))
                .transpose()?,
        })
    }

    /// Load an image from raw bytes, auto-detecting format.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ImageError> {
        // Check for JPEG magic bytes
        if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
            return Self::from_jpeg(data.to_vec());
        }

        // Check for PNG magic bytes
        if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
            return Self::from_png(data);
        }

        Err(ImageError::UnsupportedFormat)
    }

    /// Load an image from a file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ImageError> {
        let data = std::fs::read(path.as_ref()).map_err(|e| ImageError::IoError(e.to_string()))?;
        Self::from_bytes(&data)
    }

    /// Build the PDF Image XObject dictionary.
    pub fn build_xobject_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
        dict.insert("Width".to_string(), Object::Integer(self.width as i64));
        dict.insert("Height".to_string(), Object::Integer(self.height as i64));
        dict.insert(
            "ColorSpace".to_string(),
            Object::Name(self.color_space.pdf_name().to_string()),
        );
        dict.insert(
            "BitsPerComponent".to_string(),
            Object::Integer(self.bits_per_component as i64),
        );

        // Add filter based on format
        match self.format {
            ImageFormat::Jpeg => {
                dict.insert("Filter".to_string(), Object::Name("DCTDecode".to_string()));
            },
            ImageFormat::Png => {
                dict.insert("Filter".to_string(), Object::Name("FlateDecode".to_string()));
                // Add predictor for PNG-style row filtering
                let mut decode_parms = HashMap::new();
                decode_parms.insert("Predictor".to_string(), Object::Integer(15));
                decode_parms.insert(
                    "Colors".to_string(),
                    Object::Integer(self.color_space.components() as i64),
                );
                decode_parms.insert(
                    "BitsPerComponent".to_string(),
                    Object::Integer(self.bits_per_component as i64),
                );
                decode_parms.insert("Columns".to_string(), Object::Integer(self.width as i64));
                dict.insert("DecodeParms".to_string(), Object::Dictionary(decode_parms));
            },
            ImageFormat::Raw => {
                // No filter for raw data
            },
        }

        dict.insert("Length".to_string(), Object::Integer(self.data.len() as i64));

        dict
    }

    /// Build a soft mask (alpha channel) XObject dictionary.
    pub fn build_soft_mask_dict(&self) -> Option<HashMap<String, Object>> {
        self.soft_mask.as_ref().map(|mask_data| {
            let mut dict = HashMap::new();
            dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
            dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
            dict.insert("Width".to_string(), Object::Integer(self.width as i64));
            dict.insert("Height".to_string(), Object::Integer(self.height as i64));
            dict.insert("ColorSpace".to_string(), Object::Name("DeviceGray".to_string()));
            dict.insert("BitsPerComponent".to_string(), Object::Integer(8));
            dict.insert("Filter".to_string(), Object::Name("FlateDecode".to_string()));
            // The alpha data is compressed by compress_image_data(), which
            // prepends a PNG None-filter byte (0x00) to every scanline.
            // Without DecodeParms/Predictor=15 the viewer would interpret
            // those filter bytes as pixel data, shifting every row by one
            // byte and producing a diagonal-line artifact (#450).
            let mut decode_parms = HashMap::new();
            decode_parms.insert("Predictor".to_string(), Object::Integer(15));
            decode_parms.insert("Colors".to_string(), Object::Integer(1));
            decode_parms.insert("BitsPerComponent".to_string(), Object::Integer(8));
            decode_parms.insert("Columns".to_string(), Object::Integer(self.width as i64));
            dict.insert("DecodeParms".to_string(), Object::Dictionary(decode_parms));
            dict.insert("Length".to_string(), Object::Integer(mask_data.len() as i64));
            dict
        })
    }

    /// Get the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Calculate dimensions to fit within a bounding box while maintaining aspect ratio.
    pub fn fit_to_box(&self, max_width: f32, max_height: f32) -> (f32, f32) {
        let aspect = self.aspect_ratio();
        let box_aspect = max_width / max_height;

        if aspect > box_aspect {
            // Image is wider than box, constrain by width
            (max_width, max_width / aspect)
        } else {
            // Image is taller than box, constrain by height
            (max_height * aspect, max_height)
        }
    }
}

/// Image embedding error.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    /// Unsupported image format
    #[error("Unsupported image format")]
    UnsupportedFormat,

    /// Failed to decode image
    #[error("Failed to decode image: {0}")]
    DecodeError(String),

    /// Failed to compress image data
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Invalid image data
    #[error("Invalid image data: {0}")]
    InvalidData(String),
}

/// Parse JPEG header to extract dimensions and color space.
fn parse_jpeg_header(data: &[u8]) -> Result<(u32, u32, ColorSpace), ImageError> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(ImageError::InvalidData("Not a valid JPEG".to_string()));
    }

    let mut pos = 2;
    while pos < data.len() - 1 {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }

        let marker = data[pos + 1];
        pos += 2;

        // Skip padding
        if marker == 0xFF || marker == 0x00 {
            continue;
        }

        // SOF markers (Start of Frame)
        if matches!(
            marker,
            0xC0 | 0xC1
                | 0xC2
                | 0xC3
                | 0xC5
                | 0xC6
                | 0xC7
                | 0xC9
                | 0xCA
                | 0xCB
                | 0xCD
                | 0xCE
                | 0xCF
        ) {
            if pos + 7 > data.len() {
                return Err(ImageError::InvalidData("Truncated JPEG header".to_string()));
            }

            let _precision = data[pos + 2];
            let height = u16::from_be_bytes([data[pos + 3], data[pos + 4]]) as u32;
            let width = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
            let components = data[pos + 7];

            let color_space = match components {
                1 => ColorSpace::DeviceGray,
                3 => ColorSpace::DeviceRGB,
                4 => ColorSpace::DeviceCMYK,
                _ => ColorSpace::DeviceRGB,
            };

            return Ok((width, height, color_space));
        }

        // Skip other markers
        if pos + 2 > data.len() {
            break;
        }
        let length = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += length;
    }

    Err(ImageError::InvalidData("Could not find JPEG dimensions".to_string()))
}

/// Compress raw pixel data using Flate with a PNG None-filter byte (0x00)
/// prepended to each scanline.
///
/// The PDF spec requires that when `FlateDecode` is used with
/// `DecodeParms/Predictor=15` (PNG adaptive predictor), the decompressed
/// byte stream must begin each row with a filter-type byte followed by the
/// (possibly filtered) row samples.  Filter type 0 = None means the samples
/// are stored verbatim.  Flate still compresses the whole stream, including
/// the filter bytes, so LZ compression quality is unaffected.
fn compress_image_data(pixels: &[u8], width: u32, components: u8) -> Result<Vec<u8>, ImageError> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let row_bytes = width as usize * components as usize;

    if row_bytes == 0 || pixels.is_empty() {
        let encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        return encoder
            .finish()
            .map_err(|e| ImageError::CompressionError(e.to_string()));
    }

    // Pre-allocate: 1 filter byte per row + all pixel bytes.
    let num_rows = pixels.len().div_ceil(row_bytes);
    let mut filtered = Vec::with_capacity(pixels.len() + num_rows);
    for row in pixels.chunks(row_bytes) {
        filtered.push(0u8); // filter type 0 = None
        filtered.extend_from_slice(row);
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&filtered)
        .map_err(|e| ImageError::CompressionError(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| ImageError::CompressionError(e.to_string()))
}

/// Image placement on a PDF page.
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// X position (left edge)
    pub x: f32,
    /// Y position (bottom edge)
    pub y: f32,
    /// Display width
    pub width: f32,
    /// Display height
    pub height: f32,
}

impl ImagePlacement {
    /// Create a new image placement.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create placement at origin with given dimensions.
    pub fn at_origin(width: f32, height: f32) -> Self {
        Self::new(0.0, 0.0, width, height)
    }

    /// Generate the transformation matrix for this placement.
    ///
    /// Returns the six values for the `cm` operator: a, b, c, d, e, f
    /// where the matrix is:
    /// ```text
    /// [ a  b  0 ]
    /// [ c  d  0 ]
    /// [ e  f  1 ]
    /// ```
    pub fn transform_matrix(&self) -> (f32, f32, f32, f32, f32, f32) {
        // Scale and translate: width, 0, 0, height, x, y
        (self.width, 0.0, 0.0, self.height, self.x, self.y)
    }
}

/// Image XObject manager for PDF generation.
#[derive(Debug, Default)]
pub struct ImageManager {
    /// Registered images (name -> image data)
    images: HashMap<String, ImageData>,
    /// Image resource IDs (name -> resource ID like "Im1")
    resource_ids: HashMap<String, String>,
    /// Next image resource ID number
    next_id: u32,
}

impl ImageManager {
    /// Create a new image manager.
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            resource_ids: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register an image.
    ///
    /// # Arguments
    /// * `name` - Name to register the image under
    /// * `image` - The image data
    ///
    /// # Returns
    /// The image resource ID (e.g., "Im1")
    pub fn register(&mut self, name: impl Into<String>, image: ImageData) -> String {
        let name = name.into();
        let resource_id = format!("Im{}", self.next_id);
        self.next_id += 1;

        self.resource_ids.insert(name.clone(), resource_id.clone());
        self.images.insert(name, image);

        resource_id
    }

    /// Load and register an image from file.
    pub fn register_from_file(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<String, ImageError> {
        let image = ImageData::from_file(path)?;
        Ok(self.register(name, image))
    }

    /// Get an image by name.
    pub fn get(&self, name: &str) -> Option<&ImageData> {
        self.images.get(name)
    }

    /// Get the resource ID for an image.
    pub fn resource_id(&self, name: &str) -> Option<&str> {
        self.resource_ids.get(name).map(|s| s.as_str())
    }

    /// Iterate over all registered images.
    pub fn images(&self) -> impl Iterator<Item = (&str, &ImageData)> {
        self.images.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over all images with resource IDs.
    pub fn images_with_ids(&self) -> impl Iterator<Item = (&str, &str, &ImageData)> {
        self.images.iter().filter_map(|(name, image)| {
            self.resource_ids
                .get(name)
                .map(|id| (name.as_str(), id.as_str(), image))
        })
    }

    /// Get the number of registered images.
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Check if any images are registered.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_space_components() {
        assert_eq!(ColorSpace::DeviceGray.components(), 1);
        assert_eq!(ColorSpace::DeviceRGB.components(), 3);
        assert_eq!(ColorSpace::DeviceCMYK.components(), 4);
    }

    #[test]
    fn test_color_space_pdf_name() {
        assert_eq!(ColorSpace::DeviceGray.pdf_name(), "DeviceGray");
        assert_eq!(ColorSpace::DeviceRGB.pdf_name(), "DeviceRGB");
        assert_eq!(ColorSpace::DeviceCMYK.pdf_name(), "DeviceCMYK");
    }

    #[test]
    fn test_image_aspect_ratio() {
        let image = ImageData::new(200, 100, ColorSpace::DeviceRGB, vec![]);
        assert!((image.aspect_ratio() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_image_fit_to_box() {
        let image = ImageData::new(200, 100, ColorSpace::DeviceRGB, vec![]);

        // Fit wide image to square box
        let (w, h) = image.fit_to_box(100.0, 100.0);
        assert!((w - 100.0).abs() < 0.001);
        assert!((h - 50.0).abs() < 0.001);

        // Fit wide image to tall box
        let (w, h) = image.fit_to_box(100.0, 200.0);
        assert!((w - 100.0).abs() < 0.001);
        assert!((h - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_image_placement_transform() {
        let placement = ImagePlacement::new(100.0, 200.0, 50.0, 75.0);
        let (a, b, c, d, e, f) = placement.transform_matrix();

        assert!((a - 50.0).abs() < 0.001);
        assert!((d - 75.0).abs() < 0.001);
        assert!((e - 100.0).abs() < 0.001);
        assert!((f - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_image_manager() {
        let mut manager = ImageManager::new();
        assert!(manager.is_empty());

        let image = ImageData::new(100, 100, ColorSpace::DeviceRGB, vec![0; 30000]);
        let id = manager.register("test", image);

        assert!(!manager.is_empty());
        assert_eq!(manager.len(), 1);
        assert!(manager.get("test").is_some());
        assert_eq!(manager.resource_id("test"), Some(id.as_str()));
    }

    #[test]
    fn test_xobject_dict_jpeg() {
        let mut image = ImageData::new(100, 50, ColorSpace::DeviceRGB, vec![0xFF, 0xD8]);
        image.format = ImageFormat::Jpeg;

        let dict = image.build_xobject_dict();

        assert_eq!(dict.get("Type"), Some(&Object::Name("XObject".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Image".to_string())));
        assert_eq!(dict.get("Width"), Some(&Object::Integer(100)));
        assert_eq!(dict.get("Height"), Some(&Object::Integer(50)));
        assert_eq!(dict.get("Filter"), Some(&Object::Name("DCTDecode".to_string())));
    }

    #[test]
    fn test_invalid_jpeg_header() {
        let result = parse_jpeg_header(&[0x00, 0x00]);
        assert!(matches!(result, Err(ImageError::InvalidData(_))));
    }

    // ── issue #425 regression tests ────────────────────────────────────────

    /// Build a minimal PNG in memory using the `image` crate so we can round-
    /// trip through `from_png()` without touching the filesystem.
    fn make_png_bytes(width: u32, height: u32, pixels_rgb: &[u8]) -> Vec<u8> {
        use image::RgbImage;
        let img = RgbImage::from_raw(width, height, pixels_rgb.to_vec())
            .expect("pixel buffer size mismatch");
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("failed to encode PNG");
        buf
    }

    #[test]
    fn test_from_png_compressed_stream_has_filter_bytes() {
        // 2×2 RGB PNG: red, green, blue, yellow.
        let pixels: &[u8] = &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0];
        let png = make_png_bytes(2, 2, pixels);

        let image_data = ImageData::from_png(&png).expect("from_png failed");
        assert_eq!(image_data.width, 2);
        assert_eq!(image_data.height, 2);
        assert_eq!(image_data.color_space, ColorSpace::DeviceRGB);

        // Decompress the stored stream and check per-row filter bytes.
        use flate2::read::ZlibDecoder;
        use std::io::Read;
        let mut dec = ZlibDecoder::new(&image_data.data[..]);
        let mut raw = Vec::new();
        dec.read_to_end(&mut raw).expect("decompression failed");

        // Layout: 2 rows × (1 filter byte + 2 pixels × 3 channels) = 14 bytes.
        assert_eq!(raw.len(), 2 * (1 + 2 * 3), "decompressed length mismatch");

        assert_eq!(raw[0], 0x00, "row 0 filter byte must be 0 (None)");
        assert_eq!(raw[7], 0x00, "row 1 filter byte must be 0 (None)");

        // Pixel values must be preserved after stripping filter bytes.
        assert_eq!(&raw[1..7], &[255, 0, 0, 0, 255, 0], "row 0 RGB pixels");
        assert_eq!(&raw[8..14], &[0, 0, 255, 255, 255, 0], "row 1 RGB pixels");
    }

    #[test]
    fn test_from_png_xobject_dict_predictor_matches_filter_bytes() {
        let png = make_png_bytes(3, 2, &[0u8; 3 * 2 * 3]);
        let image_data = ImageData::from_png(&png).expect("from_png failed");
        let dict = image_data.build_xobject_dict();

        // Must use FlateDecode (not DCT).
        assert_eq!(dict["Filter"], Object::Name("FlateDecode".to_string()));

        // DecodeParms must have Predictor=15 (PNG adaptive), matching the
        // per-row filter bytes we now inject in compress_image_data().
        let parms = match dict.get("DecodeParms") {
            Some(Object::Dictionary(d)) => d,
            _ => panic!("DecodeParms missing or wrong type"),
        };
        assert_eq!(parms["Predictor"], Object::Integer(15));
        assert_eq!(parms["Columns"], Object::Integer(3));
        assert_eq!(parms["Colors"], Object::Integer(3)); // DeviceRGB
    }

    #[test]
    fn test_from_png_grayscale_filter_bytes() {
        // Grayscale PNG: 2 pixels wide, 1 row.
        use image::GrayImage;
        let img = GrayImage::from_raw(2, 1, vec![100u8, 200]).unwrap();
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();

        let image_data = ImageData::from_png(&buf).expect("grayscale from_png failed");
        assert_eq!(image_data.color_space, ColorSpace::DeviceGray);

        use flate2::read::ZlibDecoder;
        use std::io::Read;
        let mut dec = ZlibDecoder::new(&image_data.data[..]);
        let mut raw = Vec::new();
        dec.read_to_end(&mut raw).unwrap();

        // 1 row × (1 filter byte + 2 gray pixels) = 3 bytes.
        assert_eq!(raw.len(), 3);
        assert_eq!(raw[0], 0x00, "filter byte must be None");
        assert_eq!(&raw[1..], &[100, 200], "gray pixel values");
    }
}
