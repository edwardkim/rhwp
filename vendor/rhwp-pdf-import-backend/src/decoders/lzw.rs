//! LZWDecode implementation for PDF.
//!
//! Decompresses data using the Lempel-Ziv-Welch (LZW) algorithm as specified
//! in the PDF Reference (Section 7.4.4).
//!
//! PDF's LZW implementation:
//! - Uses MSB-first bit ordering
//! - Starts with 9-bit codes
//! - Increases code size when table fills up
//! - Uses EarlyChange=1 (change code size one code earlier than GIF/TIFF)
//! - Clear code is 256, EOD code is 257
//! - First available code is 258

use crate::decoders::StreamDecoder;
use crate::error::{Error, Result};

/// LZWDecode filter implementation.
///
/// Decompresses data using the LZW algorithm.
pub struct LzwDecoder;

impl StreamDecoder for LzwDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        // Try weezl first with PDF-compatible settings
        match decode_lzw_weezl(input) {
            Ok(data) => Ok(data),
            Err(_) => {
                // Fall back to custom implementation for edge cases
                decode_lzw_custom(input)
            },
        }
    }

    fn name(&self) -> &str {
        "LZWDecode"
    }
}

/// Decode using weezl crate (well-tested LZW implementation).
fn decode_lzw_weezl(input: &[u8]) -> Result<Vec<u8>> {
    use weezl::{decode::Decoder as WeezlDecoder, BitOrder};

    // PDF uses MSB bit order, 8-bit minimum code size
    let mut decoder = WeezlDecoder::new(BitOrder::Msb, 8);

    match decoder.decode(input) {
        Ok(output) => Ok(output),
        Err(e) => {
            log::warn!("Weezl LZW decode failed: {:?}, falling back to custom", e);
            Err(Error::Decode(format!("LZWDecode error: {:?}", e)))
        },
    }
}

/// Custom LZW decoder for PDF (handles edge cases).
///
/// This implementation follows the PDF spec exactly, including EarlyChange behavior.
fn decode_lzw_custom(input: &[u8]) -> Result<Vec<u8>> {
    const CLEAR_CODE: u16 = 256;
    const EOD_CODE: u16 = 257;
    const FIRST_CODE: u16 = 258;
    const MAX_CODE_BITS: u8 = 12;

    let mut output = Vec::new();
    let mut table = init_lzw_table();
    let mut code_bits = 9;
    let mut next_code = FIRST_CODE;
    let mut bit_reader = BitReader::new(input);
    let mut prev_code: Option<u16> = None;

    loop {
        // EarlyChange=1: Check if we need to increase code size BEFORE reading
        // PDF's EarlyChange=1 means: increase code size when next_code == 2^code_bits - 1
        // This is "one code early" compared to standard LZW (which waits until 2^code_bits)
        if code_bits < MAX_CODE_BITS && next_code > 0 {
            let increase_at = (1 << code_bits) - 1; // 2^code_bits - 1 (511, 1023, 2047)
            if next_code == increase_at {
                code_bits += 1;
            }
        }

        let code = match bit_reader.read_bits(code_bits) {
            Some(c) => c as u16,
            None => break, // End of data
        };

        if code == EOD_CODE {
            break;
        }

        if code == CLEAR_CODE {
            // Reset table
            table = init_lzw_table();
            code_bits = 9;
            next_code = FIRST_CODE;
            prev_code = None;
            continue;
        }

        // Get the string for this code
        let string = if code < next_code {
            // Code is in table
            table
                .get(&code)
                .ok_or_else(|| {
                    Error::Decode(format!(
                        "Invalid LZW code: {} (table size: {})",
                        code,
                        table.len()
                    ))
                })?
                .clone()
        } else if code == next_code && prev_code.is_some() {
            // Special case: code == next_code
            // String is prev_string + prev_string[0]
            let prev_string = table.get(&prev_code.unwrap()).unwrap();
            let mut s = prev_string.clone();
            s.push(prev_string[0]);
            s
        } else {
            return Err(Error::Decode(format!(
                "Invalid LZW code: {} (next_code={}, code_bits={})",
                code, next_code, code_bits
            )));
        };

        // Output the string
        output.extend_from_slice(&string);

        // Add new entry to table
        if let Some(prev) = prev_code {
            if next_code < 4096 {
                let prev_string = table.get(&prev).unwrap();
                let mut new_string = prev_string.clone();
                new_string.push(string[0]);
                table.insert(next_code, new_string);
                next_code += 1;
            }
        }

        prev_code = Some(code);
    }

    Ok(output)
}

/// Initialize the LZW string table with single-byte strings.
fn init_lzw_table() -> std::collections::HashMap<u16, Vec<u8>> {
    let mut table = std::collections::HashMap::new();
    for i in 0..=255u16 {
        table.insert(i, vec![i as u8]);
    }
    table
}

/// Bit reader for MSB-first bit ordering.
struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0-7, position within current byte (0 = MSB)
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    fn read_bits(&mut self, n: u8) -> Option<u32> {
        if n == 0 || n > 16 {
            return None;
        }

        let mut result = 0u32;
        let mut remaining = n;

        while remaining > 0 {
            if self.byte_pos >= self.data.len() {
                return None;
            }

            let bits_in_current_byte = 8 - self.bit_pos;
            let bits_to_read = remaining.min(bits_in_current_byte);

            // Extract bits from current byte
            let byte = self.data[self.byte_pos];
            let shift_amount = bits_in_current_byte - bits_to_read;
            let mask = if bits_to_read == 8 {
                0xFF
            } else {
                ((1u8 << bits_to_read) - 1) << shift_amount
            };
            let bits = (byte & mask) >> shift_amount;

            result = (result << bits_to_read) | (bits as u32);

            self.bit_pos += bits_to_read;
            if self.bit_pos >= 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }

            remaining -= bits_to_read;
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weezl::{encode::Encoder as LzwEncoder, BitOrder};

    #[test]
    fn test_lzw_decode_simple() {
        let decoder = LzwDecoder;

        // Compress some data
        let original = b"ABCABCABCABC";
        let mut encoder = LzwEncoder::new(BitOrder::Msb, 8);
        let compressed = encoder.encode(original).unwrap();

        // Decompress
        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_lzw_decode_empty() {
        let decoder = LzwDecoder;

        let original = b"";
        let mut encoder = LzwEncoder::new(BitOrder::Msb, 8);
        let compressed = encoder.encode(original).unwrap();

        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_lzw_decode_repeated_pattern() {
        let decoder = LzwDecoder;

        // LZW is efficient with repeated patterns
        let original = b"The quick brown fox jumps over the lazy dog. ".repeat(10);
        let mut encoder = LzwEncoder::new(BitOrder::Msb, 8);
        let compressed = encoder.encode(&original).unwrap();

        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_lzw_decode_invalid_data() {
        let decoder = LzwDecoder;

        // Invalid LZW data
        let invalid = b"This is not LZW compressed data";
        let result = decoder.decode(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_lzw_decoder_name() {
        let decoder = LzwDecoder;
        assert_eq!(decoder.name(), "LZWDecode");
    }
}
