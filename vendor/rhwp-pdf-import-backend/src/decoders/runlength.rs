//! RunLengthDecode implementation.
//!
//! Decodes run-length encoded data according to PDF specification:
//! - Length byte 0-127: Copy next N+1 bytes literally
//! - Length byte 128: No-op (EOD marker)
//! - Length byte 129-255: Repeat next byte 257-N times

use crate::decoders::StreamDecoder;
use crate::error::{Error, Result};

/// RunLengthDecode filter implementation.
///
/// Decompresses run-length encoded data.
pub struct RunLengthDecoder;

impl StreamDecoder for RunLengthDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut i = 0;

        while i < input.len() {
            let length = input[i];
            i += 1;

            match length {
                0..=127 => {
                    // Copy next length+1 bytes literally
                    let count = length as usize + 1;

                    if i + count > input.len() {
                        return Err(Error::Decode(format!(
                            "RunLengthDecode: not enough data for literal run (need {}, have {})",
                            count,
                            input.len() - i
                        )));
                    }

                    output.extend_from_slice(&input[i..i + count]);
                    i += count;
                },
                128 => {
                    // EOD marker - no-op, but we'll break to end decoding
                    break;
                },
                129..=255 => {
                    // Repeat next byte 257-length times
                    let count = 257 - length as usize;

                    if i >= input.len() {
                        return Err(Error::Decode(
                            "RunLengthDecode: missing byte for run".to_string(),
                        ));
                    }

                    let byte = input[i];
                    i += 1;
                    output.resize(output.len() + count, byte);
                },
            }
        }

        Ok(output)
    }

    fn name(&self) -> &str {
        "RunLengthDecode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runlength_decode_literal() {
        let decoder = RunLengthDecoder;
        // Length 4 (copy 5 bytes), then "Hello"
        let input = vec![4, b'H', b'e', b'l', b'l', b'o'];
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output, b"Hello");
    }

    #[test]
    fn test_runlength_decode_run() {
        let decoder = RunLengthDecoder;
        // Repeat 'A' 5 times (257-252=5)
        let input = vec![252, b'A'];
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output, b"AAAAA");
    }

    #[test]
    fn test_runlength_decode_mixed() {
        let decoder = RunLengthDecoder;
        // Literal "Hi" (length 1 = 2 bytes), then repeat 'X' 3 times (257-254=3)
        let input = vec![1, b'H', b'i', 254, b'X'];
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output, b"HiXXX");
    }

    #[test]
    fn test_runlength_decode_eod_marker() {
        let decoder = RunLengthDecoder;
        // Literal "Hi", EOD marker (128), garbage after
        let input = vec![1, b'H', b'i', 128, 99, 99, 99];
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output, b"Hi");
    }

    #[test]
    fn test_runlength_decode_max_literal() {
        let decoder = RunLengthDecoder;
        // Max literal run: 127 -> copy 128 bytes
        let mut input = vec![127];
        input.extend_from_slice(&[b'A'; 128]);
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output.len(), 128);
        assert_eq!(output, vec![b'A'; 128]);
    }

    #[test]
    fn test_runlength_decode_max_run() {
        let decoder = RunLengthDecoder;
        // Max run: 129 -> repeat 128 times (257-129=128)
        let input = vec![129, b'B'];
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output.len(), 128);
        assert_eq!(output, vec![b'B'; 128]);
    }

    #[test]
    fn test_runlength_decode_empty() {
        let decoder = RunLengthDecoder;
        let input = vec![];
        let output = decoder.decode(&input).unwrap();
        assert_eq!(output, b"");
    }

    #[test]
    fn test_runlength_decode_insufficient_data_literal() {
        let decoder = RunLengthDecoder;
        // Says copy 5 bytes but only provides 3
        let input = vec![4, b'A', b'B', b'C'];
        let result = decoder.decode(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_runlength_decode_missing_run_byte() {
        let decoder = RunLengthDecoder;
        // Says repeat but doesn't provide the byte to repeat
        let input = vec![252];
        let result = decoder.decode(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_runlength_decoder_name() {
        let decoder = RunLengthDecoder;
        assert_eq!(decoder.name(), "RunLengthDecode");
    }
}
