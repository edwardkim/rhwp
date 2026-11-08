//! ByteRange calculation for PDF signatures.
//!
//! PDF digital signatures use a ByteRange array to specify which portions
//! of the document are covered by the signature. The signature itself is
//! stored in a placeholder that is excluded from the signed bytes.
//!
//! ## ByteRange Format
//!
//! The ByteRange is an array of four integers:
//! `[offset1, length1, offset2, length2]`
//!
//! Where:
//! - `offset1` = 0 (start of file)
//! - `length1` = byte offset where the signature value begins
//! - `offset2` = byte offset where the signature value ends
//! - `length2` = remaining bytes to end of file
//!
//! The signature value is a hex-encoded string within `<` and `>` delimiters.

use crate::error::{Error, Result};

/// Calculator for PDF signature byte ranges.
#[derive(Debug)]
pub struct ByteRangeCalculator {
    /// Size of the placeholder for the signature value (hex digits + 2 for angle brackets)
    placeholder_size: usize,
}

impl ByteRangeCalculator {
    /// Create a new ByteRange calculator with the specified signature size.
    ///
    /// # Arguments
    ///
    /// * `estimated_signature_size` - Estimated size of the DER-encoded signature in bytes
    ///
    /// The placeholder size will be calculated as: (signature_size * 2) + 2
    /// because the signature is hex-encoded and enclosed in angle brackets.
    pub fn new(estimated_signature_size: usize) -> Self {
        // Each byte becomes 2 hex characters, plus 2 for < and >
        let placeholder_size = estimated_signature_size * 2 + 2;
        Self { placeholder_size }
    }

    /// Create a ByteRange calculator with a specific placeholder size.
    pub fn with_placeholder_size(placeholder_size: usize) -> Self {
        Self { placeholder_size }
    }

    /// Get the placeholder size (for the /Contents value).
    pub fn placeholder_size(&self) -> usize {
        self.placeholder_size
    }

    /// Generate a placeholder string for the signature contents.
    ///
    /// This returns a hex string of zeros that will be replaced with
    /// the actual signature after the document is prepared.
    pub fn generate_placeholder(&self) -> String {
        // Placeholder is: <00000...000>
        // Where the number of zeros is (placeholder_size - 2)
        format!("<{}>", "0".repeat(self.placeholder_size - 2))
    }

    /// Calculate the ByteRange array given the position of the /Contents value.
    ///
    /// # Arguments
    ///
    /// * `file_size` - Total size of the PDF file
    /// * `contents_offset` - Byte offset where the /Contents value starts (including '<')
    ///
    /// # Returns
    ///
    /// An array `[0, before_sig, after_sig_start, after_sig_len]`
    pub fn calculate_byte_range(&self, file_size: usize, contents_offset: usize) -> [i64; 4] {
        let before_sig = contents_offset as i64;
        let after_sig_start = (contents_offset + self.placeholder_size) as i64;
        let after_sig_len = file_size as i64 - after_sig_start;

        [0, before_sig, after_sig_start, after_sig_len]
    }

    /// Format a ByteRange array as a PDF array string.
    pub fn format_byte_range(byte_range: &[i64; 4]) -> String {
        format!("[{} {} {} {}]", byte_range[0], byte_range[1], byte_range[2], byte_range[3])
    }

    /// Extract the bytes to be signed from a PDF file.
    ///
    /// This returns the concatenation of the two ranges specified by ByteRange.
    ///
    /// # Arguments
    ///
    /// * `pdf_data` - Complete PDF file data
    /// * `byte_range` - The ByteRange array
    ///
    /// # Returns
    ///
    /// The bytes that should be signed
    pub fn extract_signed_bytes(pdf_data: &[u8], byte_range: &[i64; 4]) -> Result<Vec<u8>> {
        let offset1 = byte_range[0] as usize;
        let length1 = byte_range[1] as usize;
        let offset2 = byte_range[2] as usize;
        let length2 = byte_range[3] as usize;

        // Validate ranges
        if offset1 + length1 > pdf_data.len() {
            return Err(Error::InvalidPdf(format!(
                "ByteRange first range exceeds file size: {} + {} > {}",
                offset1,
                length1,
                pdf_data.len()
            )));
        }
        if offset2 + length2 > pdf_data.len() {
            return Err(Error::InvalidPdf(format!(
                "ByteRange second range exceeds file size: {} + {} > {}",
                offset2,
                length2,
                pdf_data.len()
            )));
        }

        // Extract and concatenate the two ranges
        let mut signed_bytes = Vec::with_capacity(length1 + length2);
        signed_bytes.extend_from_slice(&pdf_data[offset1..offset1 + length1]);
        signed_bytes.extend_from_slice(&pdf_data[offset2..offset2 + length2]);

        Ok(signed_bytes)
    }

    /// Check if a ByteRange covers the entire document except the signature.
    ///
    /// A valid ByteRange should:
    /// - Start at offset 0
    /// - End at the file size
    /// - Have no gaps except for the signature placeholder
    pub fn validate_byte_range(byte_range: &[i64; 4], file_size: usize) -> Result<()> {
        let offset1 = byte_range[0];
        let length1 = byte_range[1];
        let offset2 = byte_range[2];
        let length2 = byte_range[3];

        // First range must start at 0
        if offset1 != 0 {
            return Err(Error::InvalidPdf(format!("ByteRange must start at 0, got {}", offset1)));
        }

        // Second range must end at file size
        let expected_end = file_size as i64;
        let actual_end = offset2 + length2;
        if actual_end != expected_end {
            return Err(Error::InvalidPdf(format!(
                "ByteRange must end at file size {}, got {}",
                expected_end, actual_end
            )));
        }

        // First range must end before second range starts
        if length1 > offset2 {
            return Err(Error::InvalidPdf(format!(
                "ByteRange first range ({}) overlaps with second range start ({})",
                length1, offset2
            )));
        }

        Ok(())
    }

    /// Find the /Contents value position in a signature dictionary.
    ///
    /// This searches for the pattern `/Contents <` and returns the offset
    /// of the opening angle bracket.
    pub fn find_contents_offset(pdf_data: &[u8], sig_dict_offset: usize) -> Option<usize> {
        // Search for /Contents followed by whitespace and <
        let search_start = sig_dict_offset;
        let search_end = (sig_dict_offset + 4096).min(pdf_data.len());
        let search_window = &pdf_data[search_start..search_end];

        // Look for "/Contents" pattern
        let contents_pattern = b"/Contents";
        let mut pos = 0;
        while pos + contents_pattern.len() < search_window.len() {
            if search_window[pos..].starts_with(contents_pattern) {
                // Found /Contents, now find the next '<'
                let after_contents = pos + contents_pattern.len();
                for i in after_contents..search_window.len() {
                    let byte = search_window[i];
                    if byte == b'<' {
                        return Some(search_start + i);
                    }
                    // Skip whitespace
                    if !matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                        break;
                    }
                }
            }
            pos += 1;
        }

        None
    }

    /// Replace the placeholder in the PDF with the actual signature.
    ///
    /// # Arguments
    ///
    /// * `pdf_data` - Mutable PDF file data
    /// * `contents_offset` - Byte offset where the /Contents value starts
    /// * `signature_hex` - Hex-encoded signature to insert
    ///
    /// # Returns
    ///
    /// The modified PDF data with the signature inserted
    pub fn insert_signature(
        &self,
        pdf_data: &mut [u8],
        contents_offset: usize,
        signature_hex: &str,
    ) -> Result<()> {
        // Verify the signature fits in the placeholder
        let sig_len = signature_hex.len() + 2; // +2 for angle brackets
        if sig_len > self.placeholder_size {
            return Err(Error::InvalidPdf(format!(
                "Signature ({} bytes) exceeds placeholder size ({} bytes)",
                sig_len, self.placeholder_size
            )));
        }

        // Build the padded signature value
        let mut sig_value = String::with_capacity(self.placeholder_size);
        sig_value.push('<');
        sig_value.push_str(signature_hex);
        // Pad with zeros to fill the placeholder
        let padding_needed = (self.placeholder_size - 2) - signature_hex.len();
        for _ in 0..padding_needed {
            sig_value.push('0');
        }
        sig_value.push('>');

        // Verify replacement region
        if contents_offset + self.placeholder_size > pdf_data.len() {
            return Err(Error::InvalidPdf(
                "Signature insertion would exceed file bounds".to_string(),
            ));
        }

        // Replace the placeholder
        pdf_data[contents_offset..contents_offset + self.placeholder_size]
            .copy_from_slice(sig_value.as_bytes());

        Ok(())
    }
}

impl Default for ByteRangeCalculator {
    fn default() -> Self {
        // Default to 8KB signature (should be enough for most cases)
        Self::new(8192)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_size() {
        let calc = ByteRangeCalculator::new(1024);
        // 1024 bytes * 2 (hex) + 2 (brackets) = 2050
        assert_eq!(calc.placeholder_size(), 2050);
    }

    #[test]
    fn test_generate_placeholder() {
        let calc = ByteRangeCalculator::with_placeholder_size(10);
        let placeholder = calc.generate_placeholder();
        assert_eq!(placeholder, "<00000000>");
        assert_eq!(placeholder.len(), 10);
    }

    #[test]
    fn test_calculate_byte_range() {
        let calc = ByteRangeCalculator::with_placeholder_size(100);
        let byte_range = calc.calculate_byte_range(1000, 400);

        assert_eq!(byte_range[0], 0); // Start of file
        assert_eq!(byte_range[1], 400); // Before signature
        assert_eq!(byte_range[2], 500); // After signature (400 + 100)
        assert_eq!(byte_range[3], 500); // Remaining (1000 - 500)
    }

    #[test]
    fn test_format_byte_range() {
        let byte_range = [0, 100, 200, 300];
        let formatted = ByteRangeCalculator::format_byte_range(&byte_range);
        assert_eq!(formatted, "[0 100 200 300]");
    }

    #[test]
    fn test_extract_signed_bytes() {
        let pdf_data = b"AAABBBCCC"; // 9 bytes
        let byte_range = [0, 3, 6, 3]; // "AAA" + "CCC"

        let signed = ByteRangeCalculator::extract_signed_bytes(pdf_data, &byte_range).unwrap();
        assert_eq!(signed, b"AAACCC");
    }

    #[test]
    fn test_validate_byte_range_valid() {
        let byte_range = [0, 100, 150, 50];
        let result = ByteRangeCalculator::validate_byte_range(&byte_range, 200);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_byte_range_invalid_start() {
        let byte_range = [10, 100, 150, 50];
        let result = ByteRangeCalculator::validate_byte_range(&byte_range, 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_byte_range_invalid_end() {
        let byte_range = [0, 100, 150, 100];
        let result = ByteRangeCalculator::validate_byte_range(&byte_range, 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_signature() {
        let calc = ByteRangeCalculator::with_placeholder_size(10);
        let mut pdf_data = b"XX<00000000>YY".to_vec();
        let contents_offset = 2;
        let signature_hex = "ABCD";

        calc.insert_signature(&mut pdf_data, contents_offset, signature_hex)
            .unwrap();

        // Placeholder is 10 chars total: < + 8 hex chars + >
        // Signature is "ABCD" (4 chars), so padded with 4 zeros: "ABCD0000"
        // Result: <ABCD0000>
        assert_eq!(&pdf_data, b"XX<ABCD0000>YY");
    }

    #[test]
    fn test_insert_signature_too_large() {
        let calc = ByteRangeCalculator::with_placeholder_size(10);
        let mut pdf_data = b"XX<00000000>YY".to_vec();
        let signature_hex = "AABBCCDDEE"; // 10 chars + 2 brackets = 12 > 10

        let result = calc.insert_signature(&mut pdf_data, 2, signature_hex);
        assert!(result.is_err());
    }
}
