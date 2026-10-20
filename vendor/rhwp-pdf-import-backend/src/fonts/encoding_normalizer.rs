//! Encoding normalization for custom font encodings.
//!
//! This module provides lightweight encoding normalization to convert raw character
//! codes through font-specific encodings (standard or custom) before word boundary
//! detection. This ensures that word boundary detection works on actual Unicode
//! characters rather than raw byte codes.
//!
//! Per PDF Spec ISO 32000-1:2008, Section 9.6.6:
//! - Fonts can have standard encodings (WinAnsiEncoding, MacRomanEncoding, etc.)
//! - Fonts can have custom encodings with /Differences arrays
//! - /Differences override the base encoding for specific character codes
//!
//! # Example
//!
//! Given a custom encoding with /Differences [0x64 /rho], the byte 0x64 should
//! normalize to the Greek letter 'ρ' (U+03C1), not the ASCII 'd' (U+0064).

use super::Encoding;

/// Normalize character codes through font encoding.
///
/// This struct provides a lightweight wrapper around a font's encoding
/// to normalize raw character codes into Unicode code points.
#[derive(Debug, Clone)]
pub struct EncodingNormalizer {
    /// The font's encoding (Standard, Custom, or Identity)
    encoding: Encoding,

    /// Font name for debugging/logging
    font_name: String,
}

impl EncodingNormalizer {
    /// Create an encoding normalizer from a font's encoding.
    ///
    /// # Arguments
    ///
    /// * `encoding` - The font's encoding (from FontInfo)
    /// * `font_name` - The font's base name (for debugging)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::fonts::{Encoding, encoding_normalizer::EncodingNormalizer};
    ///
    /// let encoding = Encoding::Standard("WinAnsiEncoding".to_string());
    /// let normalizer = EncodingNormalizer::new(encoding, "Helvetica".to_string());
    /// ```
    pub fn new(encoding: Encoding, font_name: String) -> Self {
        Self {
            encoding,
            font_name,
        }
    }

    /// Normalize a raw character code through the font's encoding.
    ///
    /// Per PDF Spec ISO 32000-1:2008, Section 9.10:
    /// Character-to-Unicode mapping priority:
    /// 1. ToUnicode CMap (handled elsewhere, before this function is called)
    /// 2. Font encoding (this function)
    /// 3. Adobe Glyph List (fallback, handled elsewhere)
    ///
    /// # Arguments
    ///
    /// * `char_code` - The raw byte value from the PDF content stream
    ///
    /// # Returns
    ///
    /// The normalized Unicode code point, or the raw code if no mapping exists
    pub fn normalize(&self, char_code: u8) -> u32 {
        match &self.encoding {
            Encoding::Custom(mappings) => {
                // Custom encoding: use explicit character mappings
                if let Some(&mapped_char) = mappings.get(&char_code) {
                    mapped_char as u32
                } else {
                    // No mapping - return raw code
                    char_code as u32
                }
            },
            Encoding::Standard(encoding_name) => {
                // Standard encoding: apply standard encoding rules
                // For now, we pass through the code as-is, since standard encodings
                // are typically handled by ToUnicode CMap or character_mapper
                // This is a placeholder for future standard encoding normalization
                self.normalize_standard_encoding(char_code, encoding_name)
            },
            Encoding::Identity => {
                // Identity encoding: code == Unicode (for CID fonts)
                char_code as u32
            },
        }
    }

    /// Apply standard encoding normalization rules.
    ///
    /// Standard encodings like WinAnsiEncoding and MacRomanEncoding have
    /// well-defined mappings from character codes to Unicode.
    ///
    /// For now, this is a pass-through, as most standard encoding handling
    /// happens in the ToUnicode CMap or character_mapper. This can be
    /// enhanced in the future to provide explicit standard encoding tables.
    fn normalize_standard_encoding(&self, char_code: u8, _encoding_name: &str) -> u32 {
        // Per PDF 32000-1:2008 §9.10.2, ToUnicode CMap takes priority over font
        // encoding for text extraction. Standard encoding tables serve as fallback.
        char_code as u32
    }

    /// Get the encoding type as a string for debugging.
    ///
    /// # Returns
    ///
    /// - "Custom" for custom encodings with /Differences
    /// - `"Standard(<name>)"` for standard encodings
    /// - "Identity" for identity encodings
    pub fn encoding_type(&self) -> String {
        match &self.encoding {
            Encoding::Custom(_) => "Custom".to_string(),
            Encoding::Standard(name) => format!("Standard({})", name),
            Encoding::Identity => "Identity".to_string(),
        }
    }

    /// Get the font name associated with this normalizer.
    pub fn font_name(&self) -> &str {
        &self.font_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_custom_encoding_normalization() {
        // Create custom encoding with /Differences [0x64 /rho]
        let mut mappings = HashMap::new();
        mappings.insert(0x64, 'ρ'); // Greek rho at position 0x64

        let encoding = Encoding::Custom(mappings);
        let normalizer = EncodingNormalizer::new(encoding, "CustomFont".to_string());

        // Code 0x64 should normalize to Greek rho (U+03C1)
        let normalized = normalizer.normalize(0x64);
        assert_eq!(normalized, 0x03C1, "0x64 should normalize to Greek rho");
    }

    #[test]
    fn test_custom_encoding_no_mapping() {
        // Create custom encoding with only 0x64 mapped
        let mut mappings = HashMap::new();
        mappings.insert(0x64, 'ρ');

        let encoding = Encoding::Custom(mappings);
        let normalizer = EncodingNormalizer::new(encoding, "CustomFont".to_string());

        // Code 0x65 has no mapping - should return raw code
        let normalized = normalizer.normalize(0x65);
        assert_eq!(normalized, 0x65, "Unmapped code should return raw value");
    }

    #[test]
    fn test_standard_encoding_passthrough() {
        // Standard encoding should pass through (for now)
        let encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        let normalizer = EncodingNormalizer::new(encoding, "Helvetica".to_string());

        let normalized = normalizer.normalize(0x41); // 'A'
        assert_eq!(normalized, 0x41, "Standard encoding passes through");
    }

    #[test]
    fn test_identity_encoding() {
        // Identity encoding should pass through as-is
        let encoding = Encoding::Identity;
        let normalizer = EncodingNormalizer::new(encoding, "CIDFont".to_string());

        let normalized = normalizer.normalize(0x80);
        assert_eq!(normalized, 0x80, "Identity encoding passes through");
    }

    #[test]
    fn test_encoding_type_custom() {
        let encoding = Encoding::Custom(HashMap::new());
        let normalizer = EncodingNormalizer::new(encoding, "Test".to_string());

        assert_eq!(normalizer.encoding_type(), "Custom");
    }

    #[test]
    fn test_encoding_type_standard() {
        let encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        let normalizer = EncodingNormalizer::new(encoding, "Test".to_string());

        assert_eq!(normalizer.encoding_type(), "Standard(WinAnsiEncoding)");
    }

    #[test]
    fn test_encoding_type_identity() {
        let encoding = Encoding::Identity;
        let normalizer = EncodingNormalizer::new(encoding, "Test".to_string());

        assert_eq!(normalizer.encoding_type(), "Identity");
    }
}
