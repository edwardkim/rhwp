//! Barcode and QR code generation for PDF documents.
//!
//! This module provides functionality to generate barcodes and QR codes
//! that can be embedded in PDF documents.
//!
//! ## Supported Barcode Types
//!
//! ### 1D Barcodes (via `barcoders` crate)
//! - Code 128 (A, B, C)
//! - Code 39
//! - EAN-13
//! - EAN-8
//! - UPC-A
//! - ITF (Interleaved 2 of 5)
//! - Code 93
//! - Codabar
//!
//! ### 2D Barcodes (via `qrcode` crate)
//! - QR Code (with configurable error correction levels)
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::writer::barcode::{BarcodeGenerator, BarcodeType, QrCodeOptions};
//!
//! // Generate a Code 128 barcode
//! let barcode_png = BarcodeGenerator::generate_1d(
//!     BarcodeType::Code128,
//!     "12345678",
//!     200,  // width
//!     80,   // height
//! )?;
//!
//! // Generate a QR code
//! let qr_png = BarcodeGenerator::generate_qr(
//!     "https://example.com",
//!     QrCodeOptions::default().size(256),
//! )?;
//! ```

use crate::error::{Error, Result};

/// Types of 1D barcodes supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeType {
    /// Code 128 (auto-select optimal encoding)
    Code128,
    /// Code 39 (alphanumeric)
    Code39,
    /// EAN-13 (13 digits, European Article Number)
    Ean13,
    /// EAN-8 (8 digits, compact EAN)
    Ean8,
    /// UPC-A (12 digits, Universal Product Code)
    UpcA,
    /// ITF - Interleaved 2 of 5 (numeric pairs)
    Itf,
    /// Code 93 (alphanumeric, compact)
    Code93,
    /// Codabar (numeric with special characters)
    Codabar,
}

impl std::fmt::Display for BarcodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarcodeType::Code128 => write!(f, "Code 128"),
            BarcodeType::Code39 => write!(f, "Code 39"),
            BarcodeType::Ean13 => write!(f, "EAN-13"),
            BarcodeType::Ean8 => write!(f, "EAN-8"),
            BarcodeType::UpcA => write!(f, "UPC-A"),
            BarcodeType::Itf => write!(f, "ITF"),
            BarcodeType::Code93 => write!(f, "Code 93"),
            BarcodeType::Codabar => write!(f, "Codabar"),
        }
    }
}

/// QR code error correction level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QrErrorCorrection {
    /// Low (~7% correction capability)
    Low,
    /// Medium (~15% correction capability)
    #[default]
    Medium,
    /// Quartile (~25% correction capability)
    Quartile,
    /// High (~30% correction capability)
    High,
}

/// Options for QR code generation.
#[derive(Debug, Clone)]
pub struct QrCodeOptions {
    /// Size of the QR code in pixels (width = height)
    pub size: u32,
    /// Error correction level
    pub error_correction: QrErrorCorrection,
    /// Quiet zone (border) in modules
    pub quiet_zone: u32,
    /// Foreground color (RGBA)
    pub foreground: [u8; 4],
    /// Background color (RGBA)
    pub background: [u8; 4],
}

impl Default for QrCodeOptions {
    fn default() -> Self {
        Self {
            size: 200,
            error_correction: QrErrorCorrection::Medium,
            quiet_zone: 4,
            foreground: [0, 0, 0, 255],       // Black
            background: [255, 255, 255, 255], // White
        }
    }
}

impl QrCodeOptions {
    /// Create new QR code options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the size in pixels.
    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    /// Set the error correction level.
    pub fn error_correction(mut self, level: QrErrorCorrection) -> Self {
        self.error_correction = level;
        self
    }

    /// Set the quiet zone (border) in modules.
    pub fn quiet_zone(mut self, modules: u32) -> Self {
        self.quiet_zone = modules;
        self
    }

    /// Set the foreground color (RGBA).
    pub fn foreground(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.foreground = [r, g, b, a];
        self
    }

    /// Set the background color (RGBA).
    pub fn background(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.background = [r, g, b, a];
        self
    }
}

/// Options for 1D barcode generation.
#[derive(Debug, Clone)]
pub struct BarcodeOptions {
    /// Width of the barcode in pixels
    pub width: u32,
    /// Height of the barcode in pixels
    pub height: u32,
    /// Foreground color (RGBA)
    pub foreground: [u8; 4],
    /// Background color (RGBA)
    pub background: [u8; 4],
    /// Include human-readable text below barcode
    pub show_text: bool,
}

impl Default for BarcodeOptions {
    fn default() -> Self {
        Self {
            width: 200,
            height: 80,
            foreground: [0, 0, 0, 255],       // Black
            background: [255, 255, 255, 255], // White
            show_text: false,                 // Text rendering not implemented yet
        }
    }
}

impl BarcodeOptions {
    /// Create new barcode options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Set the foreground color (RGBA).
    pub fn foreground(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.foreground = [r, g, b, a];
        self
    }

    /// Set the background color (RGBA).
    pub fn background(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.background = [r, g, b, a];
        self
    }

    /// Whether to show human-readable text below barcode.
    pub fn show_text(mut self, show: bool) -> Self {
        self.show_text = show;
        self
    }
}

/// Barcode generator for creating 1D and 2D barcodes as PNG images.
pub struct BarcodeGenerator;

#[cfg(feature = "barcodes")]
impl BarcodeGenerator {
    fn encode_1d(barcode_type: BarcodeType, data: &str) -> Result<Vec<u8>> {
        use barcoders::sym::codabar::Codabar;
        use barcoders::sym::code128::Code128;
        use barcoders::sym::code39::Code39;
        use barcoders::sym::code93::Code93;
        use barcoders::sym::ean13::EAN13;
        use barcoders::sym::ean8::EAN8;
        use barcoders::sym::tf::TF;

        match barcode_type {
            BarcodeType::Code128 => {
                // Auto-prepend charset B (\u{0181}) if no charset prefix present
                let data_with_prefix = if data.starts_with('\u{00C0}')
                    || data.starts_with('\u{0181}')
                    || data.starts_with('\u{0106}')
                {
                    data.to_string()
                } else {
                    format!("\u{0181}{}", data)
                };
                Code128::new(&data_with_prefix)
                    .map_err(|e| Error::Barcode(format!("Code128 encoding error: {}", e)))
                    .map(|b| b.encode())
            },
            BarcodeType::Code39 => Code39::new(data)
                .map_err(|e| Error::Barcode(format!("Code39 encoding error: {}", e)))
                .map(|b| b.encode()),
            BarcodeType::Ean13 => EAN13::new(data)
                .map_err(|e| Error::Barcode(format!("EAN-13 encoding error: {}", e)))
                .map(|b| b.encode()),
            BarcodeType::Ean8 => EAN8::new(data)
                .map_err(|e| Error::Barcode(format!("EAN-8 encoding error: {}", e)))
                .map(|b| b.encode()),
            BarcodeType::UpcA => {
                let upc_data = if data.len() == 11 {
                    format!("0{}", data)
                } else if data.len() == 12 {
                    format!("0{}", &data[..11])
                } else {
                    return Err(Error::Barcode("UPC-A requires 11 or 12 digits".to_string()));
                };
                EAN13::new(&upc_data)
                    .map_err(|e| Error::Barcode(format!("UPC-A encoding error: {}", e)))
                    .map(|b| b.encode())
            },
            BarcodeType::Itf => TF::interleaved(data)
                .map_err(|e| Error::Barcode(format!("ITF encoding error: {}", e)))
                .map(|b| b.encode()),
            BarcodeType::Code93 => Code93::new(data)
                .map_err(|e| Error::Barcode(format!("Code93 encoding error: {}", e)))
                .map(|b| b.encode()),
            BarcodeType::Codabar => Codabar::new(data)
                .map_err(|e| Error::Barcode(format!("Codabar encoding error: {}", e)))
                .map(|b| b.encode()),
        }
    }

    /// Generate a 1D barcode as PNG bytes.
    pub fn generate_1d(
        barcode_type: BarcodeType,
        data: &str,
        options: &BarcodeOptions,
    ) -> Result<Vec<u8>> {
        use barcoders::generators::image::*;

        let encoded = Self::encode_1d(barcode_type, data)?;

        let image_gen = Image::PNG {
            height: options.height,
            xdim: 1,
            rotation: Rotation::Zero,
            foreground: Color::new(options.foreground),
            background: Color::new(options.background),
        };

        let png_bytes = image_gen
            .generate(&encoded)
            .map_err(|e| Error::Barcode(format!("Image generation error: {}", e)))?;

        if let Ok(img) = image::load_from_memory(&png_bytes) {
            let scaled = img.resize_exact(
                options.width,
                options.height,
                image::imageops::FilterType::Nearest,
            );
            let mut buf = Vec::new();
            scaled
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|e| Error::Barcode(format!("PNG encoding error: {}", e)))?;
            Ok(buf)
        } else {
            Ok(png_bytes)
        }
    }

    /// Generate a 1D barcode as SVG string.
    pub fn generate_1d_svg(
        barcode_type: BarcodeType,
        data: &str,
        options: &BarcodeOptions,
    ) -> Result<String> {
        use barcoders::generators::svg::{Color, SVG};

        let encoded = Self::encode_1d(barcode_type, data)?;

        let svg_gen = SVG::new(options.height)
            .foreground(Color::new(options.foreground))
            .background(Color::new(options.background));

        svg_gen
            .generate(&encoded)
            .map_err(|e| Error::Barcode(format!("SVG generation error: {}", e)))
    }

    /// Generate a QR code as PNG bytes.
    ///
    /// # Arguments
    /// * `data` - The data to encode (URL, text, etc.)
    /// * `options` - QR code generation options
    ///
    /// # Returns
    /// PNG image bytes
    pub fn generate_qr(data: &str, options: &QrCodeOptions) -> Result<Vec<u8>> {
        use qrcode::{EcLevel, QrCode};

        let ec_level = match options.error_correction {
            QrErrorCorrection::Low => EcLevel::L,
            QrErrorCorrection::Medium => EcLevel::M,
            QrErrorCorrection::Quartile => EcLevel::Q,
            QrErrorCorrection::High => EcLevel::H,
        };

        let code = QrCode::with_error_correction_level(data, ec_level)
            .map_err(|e| Error::Barcode(format!("QR code encoding error: {}", e)))?;

        // Get the QR code dimensions
        let qr_width = code.width();
        let module_count = qr_width + (options.quiet_zone as usize * 2);

        // Calculate module size to fit desired output size
        let module_size = (options.size as usize / module_count).max(1);
        let actual_size = module_count * module_size;

        // Create image buffer
        let mut img = image::RgbaImage::new(actual_size as u32, actual_size as u32);

        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba(options.background);
        }

        // Draw QR code modules
        let quiet_px = options.quiet_zone as usize * module_size;
        for (y, row) in code.to_colors().chunks(qr_width).enumerate() {
            for (x, &module) in row.iter().enumerate() {
                if module == qrcode::Color::Dark {
                    // Fill this module with foreground color
                    let start_x = quiet_px + x * module_size;
                    let start_y = quiet_px + y * module_size;
                    for dy in 0..module_size {
                        for dx in 0..module_size {
                            let px = (start_x + dx) as u32;
                            let py = (start_y + dy) as u32;
                            if px < actual_size as u32 && py < actual_size as u32 {
                                img.put_pixel(px, py, image::Rgba(options.foreground));
                            }
                        }
                    }
                }
            }
        }

        // Resize to exact requested size if different
        let final_img = if actual_size != options.size as usize {
            image::DynamicImage::ImageRgba8(img).resize_exact(
                options.size,
                options.size,
                image::imageops::FilterType::Nearest,
            )
        } else {
            image::DynamicImage::ImageRgba8(img)
        };

        // Encode to PNG
        let mut buf = Vec::new();
        final_img
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| Error::Barcode(format!("PNG encoding error: {}", e)))?;

        Ok(buf)
    }

    /// Generate a QR code as SVG string.
    pub fn generate_qr_svg(data: &str, options: &QrCodeOptions) -> Result<String> {
        use qrcode::{EcLevel, QrCode};

        let ec_level = match options.error_correction {
            QrErrorCorrection::Low => EcLevel::L,
            QrErrorCorrection::Medium => EcLevel::M,
            QrErrorCorrection::Quartile => EcLevel::Q,
            QrErrorCorrection::High => EcLevel::H,
        };

        let code = QrCode::with_error_correction_level(data, ec_level)
            .map_err(|e| Error::Barcode(format!("QR code encoding error: {}", e)))?;

        let qr_width = code.width();
        let quiet = options.quiet_zone as usize;
        let module_count = qr_width + quiet * 2;
        let module_size = (options.size as usize / module_count).max(1);
        let svg_size = module_count * module_size;

        let fg = options.foreground;
        let bg = options.background;
        let fg_hex = format!("#{:02X}{:02X}{:02X}", fg[0], fg[1], fg[2]);
        let bg_hex = format!("#{:02X}{:02X}{:02X}", bg[0], bg[1], bg[2]);

        let mut rects = String::new();
        for (y, row) in code.to_colors().chunks(qr_width).enumerate() {
            for (x, &module) in row.iter().enumerate() {
                if module == qrcode::Color::Dark {
                    let rx = (quiet + x) * module_size;
                    let ry = (quiet + y) * module_size;
                    rects.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}"/>"#,
                        rx, ry, module_size, module_size
                    ));
                }
            }
        }

        Ok(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {s} {s}" width="{s}" height="{s}"><rect width="{s}" height="{s}" fill="{bg}"/><g fill="{fg}">{rects}</g></svg>"#,
            s = svg_size,
            bg = bg_hex,
            fg = fg_hex,
            rects = rects
        ))
    }

    /// Generate a QR code with default options.
    pub fn generate_qr_simple(data: &str, size: u32) -> Result<Vec<u8>> {
        Self::generate_qr(data, &QrCodeOptions::default().size(size))
    }

    /// Generate a Code 128 barcode with default options.
    pub fn generate_code128(data: &str, width: u32, height: u32) -> Result<Vec<u8>> {
        Self::generate_1d(
            BarcodeType::Code128,
            data,
            &BarcodeOptions::default().width(width).height(height),
        )
    }

    /// Generate an EAN-13 barcode with default options.
    pub fn generate_ean13(data: &str, width: u32, height: u32) -> Result<Vec<u8>> {
        Self::generate_1d(
            BarcodeType::Ean13,
            data,
            &BarcodeOptions::default().width(width).height(height),
        )
    }
}

// Stub implementations when feature is not enabled
#[cfg(not(feature = "barcodes"))]
impl BarcodeGenerator {
    /// Generate a 1D barcode (requires `barcodes` feature).
    pub fn generate_1d(
        _barcode_type: BarcodeType,
        _data: &str,
        _options: &BarcodeOptions,
    ) -> Result<Vec<u8>> {
        Err(Error::Barcode("Barcode generation requires the 'barcodes' feature".to_string()))
    }

    /// Generate a QR code (requires `barcodes` feature).
    pub fn generate_qr(_data: &str, _options: &QrCodeOptions) -> Result<Vec<u8>> {
        Err(Error::Barcode("QR code generation requires the 'barcodes' feature".to_string()))
    }

    /// Generate a QR code with default options (requires `barcodes` feature).
    pub fn generate_qr_simple(_data: &str, _size: u32) -> Result<Vec<u8>> {
        Err(Error::Barcode("QR code generation requires the 'barcodes' feature".to_string()))
    }

    /// Generate a Code 128 barcode (requires `barcodes` feature).
    pub fn generate_code128(_data: &str, _width: u32, _height: u32) -> Result<Vec<u8>> {
        Err(Error::Barcode("Barcode generation requires the 'barcodes' feature".to_string()))
    }

    /// Generate an EAN-13 barcode (requires `barcodes` feature).
    pub fn generate_ean13(_data: &str, _width: u32, _height: u32) -> Result<Vec<u8>> {
        Err(Error::Barcode("Barcode generation requires the 'barcodes' feature".to_string()))
    }
}

#[cfg(all(test, feature = "barcodes"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_code() {
        let png = BarcodeGenerator::generate_qr_simple("https://example.com", 200).unwrap();
        assert!(!png.is_empty());
        // Verify PNG header
        assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn test_generate_code128() {
        let png = BarcodeGenerator::generate_code128("ABC123", 200, 80).unwrap();
        assert!(!png.is_empty());
        // Verify PNG header
        assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn test_generate_ean13() {
        let png = BarcodeGenerator::generate_ean13("5901234123457", 200, 80).unwrap();
        assert!(!png.is_empty());
        // Verify PNG header
        assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn test_qr_options() {
        let options = QrCodeOptions::new()
            .size(300)
            .error_correction(QrErrorCorrection::High)
            .quiet_zone(6)
            .foreground(0, 0, 128, 255)
            .background(255, 255, 255, 255);

        let png = BarcodeGenerator::generate_qr("Test data", &options).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_barcode_options() {
        let options = BarcodeOptions::new()
            .width(300)
            .height(100)
            .foreground(0, 0, 0, 255)
            .background(255, 255, 255, 255);

        let png = BarcodeGenerator::generate_1d(BarcodeType::Code128, "TEST", &options).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_barcode_type_display() {
        assert_eq!(BarcodeType::Code128.to_string(), "Code 128");
        assert_eq!(BarcodeType::Ean13.to_string(), "EAN-13");
    }
}

#[cfg(all(test, not(feature = "barcodes")))]
mod tests_no_feature {
    use super::*;

    #[test]
    fn test_feature_not_enabled() {
        let result = BarcodeGenerator::generate_qr_simple("test", 200);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires the 'barcodes' feature"));
    }

    #[test]
    fn test_generate_1d_not_enabled() {
        let result =
            BarcodeGenerator::generate_1d(BarcodeType::Code128, "test", &BarcodeOptions::new());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires the 'barcodes' feature"));
    }

    #[test]
    fn test_generate_qr_not_enabled() {
        let result = BarcodeGenerator::generate_qr("test", &QrCodeOptions::new());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires the 'barcodes' feature"));
    }

    #[test]
    fn test_generate_code128_not_enabled() {
        let result = BarcodeGenerator::generate_code128("test", 200, 80);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires the 'barcodes' feature"));
    }

    #[test]
    fn test_generate_ean13_not_enabled() {
        let result = BarcodeGenerator::generate_ean13("1234567890128", 200, 80);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires the 'barcodes' feature"));
    }
}

/// Tests that don't depend on the barcodes feature (always run)
#[cfg(test)]
mod tests_common {
    use super::*;

    // ---- Tests for BarcodeType Display ----

    #[test]
    fn test_barcode_type_display_all() {
        assert_eq!(BarcodeType::Code128.to_string(), "Code 128");
        assert_eq!(BarcodeType::Code39.to_string(), "Code 39");
        assert_eq!(BarcodeType::Ean13.to_string(), "EAN-13");
        assert_eq!(BarcodeType::Ean8.to_string(), "EAN-8");
        assert_eq!(BarcodeType::UpcA.to_string(), "UPC-A");
        assert_eq!(BarcodeType::Itf.to_string(), "ITF");
        assert_eq!(BarcodeType::Code93.to_string(), "Code 93");
        assert_eq!(BarcodeType::Codabar.to_string(), "Codabar");
    }

    // ---- Tests for BarcodeType equality ----

    #[test]
    fn test_barcode_type_equality() {
        assert_eq!(BarcodeType::Code128, BarcodeType::Code128);
        assert_ne!(BarcodeType::Code128, BarcodeType::Code39);
        assert_ne!(BarcodeType::Ean13, BarcodeType::Ean8);
    }

    #[test]
    fn test_barcode_type_copy() {
        let bt = BarcodeType::Code128;
        let bt2 = bt; // Copy
        assert_eq!(bt, bt2);
    }

    // ---- Tests for QrErrorCorrection ----

    #[test]
    fn test_qr_error_correction_default() {
        assert_eq!(QrErrorCorrection::default(), QrErrorCorrection::Medium);
    }

    #[test]
    fn test_qr_error_correction_equality() {
        assert_eq!(QrErrorCorrection::Low, QrErrorCorrection::Low);
        assert_eq!(QrErrorCorrection::Medium, QrErrorCorrection::Medium);
        assert_eq!(QrErrorCorrection::Quartile, QrErrorCorrection::Quartile);
        assert_eq!(QrErrorCorrection::High, QrErrorCorrection::High);
        assert_ne!(QrErrorCorrection::Low, QrErrorCorrection::High);
    }

    #[test]
    fn test_qr_error_correction_copy() {
        let ec = QrErrorCorrection::High;
        let ec2 = ec;
        assert_eq!(ec, ec2);
    }

    // ---- Tests for QrCodeOptions ----

    #[test]
    fn test_qr_code_options_default() {
        let opts = QrCodeOptions::default();
        assert_eq!(opts.size, 200);
        assert_eq!(opts.error_correction, QrErrorCorrection::Medium);
        assert_eq!(opts.quiet_zone, 4);
        assert_eq!(opts.foreground, [0, 0, 0, 255]);
        assert_eq!(opts.background, [255, 255, 255, 255]);
    }

    #[test]
    fn test_qr_code_options_new() {
        let opts = QrCodeOptions::new();
        assert_eq!(opts.size, 200);
    }

    #[test]
    fn test_qr_code_options_size() {
        let opts = QrCodeOptions::new().size(500);
        assert_eq!(opts.size, 500);
    }

    #[test]
    fn test_qr_code_options_error_correction() {
        let opts = QrCodeOptions::new().error_correction(QrErrorCorrection::High);
        assert_eq!(opts.error_correction, QrErrorCorrection::High);
    }

    #[test]
    fn test_qr_code_options_quiet_zone() {
        let opts = QrCodeOptions::new().quiet_zone(8);
        assert_eq!(opts.quiet_zone, 8);
    }

    #[test]
    fn test_qr_code_options_foreground() {
        let opts = QrCodeOptions::new().foreground(255, 0, 0, 128);
        assert_eq!(opts.foreground, [255, 0, 0, 128]);
    }

    #[test]
    fn test_qr_code_options_background() {
        let opts = QrCodeOptions::new().background(0, 0, 255, 200);
        assert_eq!(opts.background, [0, 0, 255, 200]);
    }

    #[test]
    fn test_qr_code_options_chaining() {
        let opts = QrCodeOptions::new()
            .size(300)
            .error_correction(QrErrorCorrection::Low)
            .quiet_zone(2)
            .foreground(128, 128, 128, 255)
            .background(200, 200, 200, 255);
        assert_eq!(opts.size, 300);
        assert_eq!(opts.error_correction, QrErrorCorrection::Low);
        assert_eq!(opts.quiet_zone, 2);
        assert_eq!(opts.foreground, [128, 128, 128, 255]);
        assert_eq!(opts.background, [200, 200, 200, 255]);
    }

    // ---- Tests for BarcodeOptions ----

    #[test]
    fn test_barcode_options_default() {
        let opts = BarcodeOptions::default();
        assert_eq!(opts.width, 200);
        assert_eq!(opts.height, 80);
        assert_eq!(opts.foreground, [0, 0, 0, 255]);
        assert_eq!(opts.background, [255, 255, 255, 255]);
        assert!(!opts.show_text);
    }

    #[test]
    fn test_barcode_options_new() {
        let opts = BarcodeOptions::new();
        assert_eq!(opts.width, 200);
        assert_eq!(opts.height, 80);
    }

    #[test]
    fn test_barcode_options_width() {
        let opts = BarcodeOptions::new().width(400);
        assert_eq!(opts.width, 400);
    }

    #[test]
    fn test_barcode_options_height() {
        let opts = BarcodeOptions::new().height(120);
        assert_eq!(opts.height, 120);
    }

    #[test]
    fn test_barcode_options_foreground() {
        let opts = BarcodeOptions::new().foreground(0, 0, 128, 255);
        assert_eq!(opts.foreground, [0, 0, 128, 255]);
    }

    #[test]
    fn test_barcode_options_background() {
        let opts = BarcodeOptions::new().background(255, 255, 0, 255);
        assert_eq!(opts.background, [255, 255, 0, 255]);
    }

    #[test]
    fn test_barcode_options_show_text() {
        let opts = BarcodeOptions::new().show_text(true);
        assert!(opts.show_text);
    }

    #[test]
    fn test_barcode_options_chaining() {
        let opts = BarcodeOptions::new()
            .width(300)
            .height(100)
            .foreground(10, 20, 30, 255)
            .background(240, 250, 255, 255)
            .show_text(true);
        assert_eq!(opts.width, 300);
        assert_eq!(opts.height, 100);
        assert_eq!(opts.foreground, [10, 20, 30, 255]);
        assert_eq!(opts.background, [240, 250, 255, 255]);
        assert!(opts.show_text);
    }
}
