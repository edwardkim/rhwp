//! DCTDecode (JPEG) filter implementation.
//!
//! Pass-through decoder for JPEG data. JPEG images are already in compressed
//! format and are returned unchanged for later image extraction.

use crate::decoders::StreamDecoder;
use crate::error::Result;

/// DCTDecode filter implementation.
///
/// Pass-through for JPEG data - no actual decoding performed.
/// JPEG images are kept in their compressed format for later extraction.
pub struct DctDecoder;

impl StreamDecoder for DctDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        // JPEG data is already in final format.
        // Just return it unchanged for now.
        // Phase 5 will handle actual image extraction/decoding.
        Ok(input.to_vec())
    }

    fn name(&self) -> &str {
        "DCTDecode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dct_decode_passthrough() {
        let decoder = DctDecoder;
        let jpeg_data = b"\xFF\xD8\xFF\xE0\x00\x10JFIF"; // JPEG header
        let output = decoder.decode(jpeg_data).unwrap();
        assert_eq!(output, jpeg_data);
    }

    #[test]
    fn test_dct_decode_empty() {
        let decoder = DctDecoder;
        let input = b"";
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"");
    }

    #[test]
    fn test_dct_decoder_name() {
        let decoder = DctDecoder;
        assert_eq!(decoder.name(), "DCTDecode");
    }
}
