//! BrotliDecode filter (PDF 2.0, ISO 32000-2:2020).

use crate::error::{Error, Result};
use std::io::Read;

use super::StreamDecoder;

/// Decoder for BrotliDecode filter (PDF 2.0).
pub struct BrotliDecoder;

impl StreamDecoder for BrotliDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut reader = brotli::Decompressor::new(input, 4096);
        reader
            .read_to_end(&mut output)
            .map_err(|e| Error::Decode(format!("BrotliDecode: {}", e)))?;
        Ok(output)
    }

    fn name(&self) -> &str {
        "BrotliDecode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brotli_roundtrip() {
        let original = b"Hello, PDF 2.0 BrotliDecode filter! This is a test.";

        // Compress with brotli
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 6, 22);
            std::io::Write::write_all(&mut writer, original).unwrap();
        }

        // Decompress with our decoder
        let decoder = BrotliDecoder;
        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_brotli_empty_input() {
        // Brotli-compressed empty data
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 6, 22);
            std::io::Write::write_all(&mut writer, b"").unwrap();
        }
        let decoder = BrotliDecoder;
        let decoded = decoder.decode(&compressed).unwrap();
        assert!(decoded.is_empty());
    }
}
