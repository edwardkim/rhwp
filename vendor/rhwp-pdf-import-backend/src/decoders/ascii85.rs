//! ASCII85Decode (Base85) implementation.
//!
//! Decodes ASCII85/Base85 encoded data. This encoding represents 4 bytes
//! as 5 ASCII characters in the range '!' to 'u'.
//! Special case: 'z' represents 4 zero bytes (00000000).

use crate::decoders::StreamDecoder;
use crate::error::{Error, Result};

/// ASCII85Decode filter implementation.
///
/// Decodes data encoded using the ASCII85/Base85 algorithm.
pub struct Ascii85Decoder;

impl StreamDecoder for Ascii85Decoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut acc: u32 = 0;
        let mut count = 0;

        for &byte in input {
            match byte {
                b'~' => break, // End marker '~>'
                b'z' => {
                    // Special case: 'z' represents 4 zero bytes
                    if count != 0 {
                        return Err(Error::Decode(
                            "ASCII85Decode: 'z' must not appear in the middle of a group"
                                .to_string(),
                        ));
                    }
                    output.extend_from_slice(&[0, 0, 0, 0]);
                },
                b'!'..=b'u' => {
                    // Valid ASCII85 character (33-117)
                    acc = acc
                        .checked_mul(85)
                        .and_then(|v| v.checked_add((byte - b'!') as u32))
                        .ok_or_else(|| {
                            Error::Decode("ASCII85Decode: overflow in decoding".to_string())
                        })?;
                    count += 1;

                    if count == 5 {
                        // Got a complete group - output 4 bytes
                        output.extend_from_slice(&acc.to_be_bytes());
                        acc = 0;
                        count = 0;
                    }
                },
                _ if byte.is_ascii_whitespace() => {}, // Skip whitespace
                _ => {
                    return Err(Error::Decode(format!(
                        "ASCII85Decode: invalid character '{}'",
                        byte as char
                    )));
                },
            }
        }

        // Handle trailing bytes (incomplete group)
        if count > 0 {
            if count == 1 {
                return Err(Error::Decode(
                    "ASCII85Decode: incomplete group (need at least 2 characters)".to_string(),
                ));
            }

            // Pad with 'u' (84 = 117 - 33) to complete the group
            for _ in count..5 {
                acc = acc
                    .checked_mul(85)
                    .and_then(|v| v.checked_add(84))
                    .ok_or_else(|| {
                        Error::Decode("ASCII85Decode: overflow in padding".to_string())
                    })?;
            }

            // Output count-1 bytes (first N-1 bytes are valid)
            let bytes = acc.to_be_bytes();
            output.extend_from_slice(&bytes[..count - 1]);
        }

        Ok(output)
    }

    fn name(&self) -> &str {
        "ASCII85Decode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii85_decode_simple() {
        let decoder = Ascii85Decoder;
        // "Test" encoded in ASCII85 (4 bytes = 1 complete group)
        let input = b"<+U,m";
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"Test");
    }

    #[test]
    fn test_ascii85_decode_z_special_case() {
        let decoder = Ascii85Decoder;
        // 'z' should decode to 4 zero bytes
        let input = b"z";
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"\x00\x00\x00\x00");
    }

    #[test]
    fn test_ascii85_decode_multiple_z() {
        let decoder = Ascii85Decoder;
        let input = b"zz";
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"\x00\x00\x00\x00\x00\x00\x00\x00");
    }

    #[test]
    fn test_ascii85_decode_with_whitespace() {
        let decoder = Ascii85Decoder;
        let input = b"<+U ,m"; // "Test" with space
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"Test");
    }

    #[test]
    fn test_ascii85_decode_with_end_marker() {
        let decoder = Ascii85Decoder;
        let input = b"<+U,m~>"; // "Test" with EOD marker
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"Test");
    }

    #[test]
    fn test_ascii85_decode_empty() {
        let decoder = Ascii85Decoder;
        let input = b"";
        let output = decoder.decode(input).unwrap();
        assert_eq!(output, b"");
    }

    #[test]
    fn test_ascii85_decode_padding() {
        let decoder = Ascii85Decoder;
        // Incomplete group - should be padded
        let input = b"!!"; // Minimal valid incomplete group
        let output = decoder.decode(input).unwrap();
        // Should decode to at least 1 byte
        assert!(!output.is_empty());
    }

    #[test]
    fn test_ascii85_decode_invalid_character() {
        let decoder = Ascii85Decoder;
        let input = b"Hello\x00"; // Contains invalid character
        let result = decoder.decode(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_ascii85_decode_z_in_middle() {
        let decoder = Ascii85Decoder;
        // 'z' in the middle of a group is invalid
        let input = b"!z";
        let result = decoder.decode(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_ascii85_decode_single_char() {
        let decoder = Ascii85Decoder;
        // Single character (not 'z') is invalid
        let input = b"!";
        let result = decoder.decode(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_ascii85_decoder_name() {
        let decoder = Ascii85Decoder;
        assert_eq!(decoder.name(), "ASCII85Decode");
    }
}
