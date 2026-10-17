//! Object stream parsing (PDF 1.5+).
//!
//! Object streams (/Type /ObjStm) allow multiple objects to be compressed together
//! in a single stream for better compression ratios. This module handles parsing
//! these streams and extracting individual objects.
//!
//! # Format
//!
//! An object stream has this structure:
//! ```text
//! N 0 obj
//! << /Type /ObjStm
//!    /N 5              % Number of objects in stream
//!    /First 30         % Byte offset to first object's data
//!    /Filter /FlateDecode
//! >>
//! stream
//! 10 0 11 15 12 28 13 42 14 55    % Pairs: (obj_num, offset)
//! <dict>                           % Object 10 at offset 0
//! <array>                          % Object 11 at offset 15
//! ...
//! endstream
//! endobj
//! ```
//!
//! The first part contains N pairs of integers (object number, byte offset relative
//! to /First). The second part contains the actual object data.

use crate::error::{Error, Result};
use crate::object::Object;
use std::collections::HashMap;

/// Parse an object stream and extract all objects.
///
/// This is a convenience method that calls `parse_object_stream_with_decryption`
/// with no encryption parameters.
///
/// # Arguments
///
/// * `stream_obj` - The object stream object (must be a Stream with /Type /ObjStm)
///
/// # Returns
///
/// A HashMap mapping object numbers to their parsed objects.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::objstm::parse_object_stream;
/// use pdf_oxide::object::Object;
///
/// // Assuming we have loaded an object stream
/// let objects = parse_object_stream(&stream_obj)?;
/// let obj_10 = objects.get(&10).unwrap();
/// # Ok::<(), pdf_oxide::error::Error>(())
/// ```
pub fn parse_object_stream(stream_obj: &Object) -> Result<HashMap<u32, Object>> {
    parse_object_stream_with_decryption(stream_obj, None, 0, 0)
}

/// Parse an object stream with optional decryption.
///
/// PDF Spec: Object streams (PDF 1.5+) can be encrypted like any other stream.
/// The encryption must be applied before decompression.
///
/// # Arguments
///
/// * `stream_obj` - The object stream object (must be a Stream with /Type /ObjStm)
/// * `decryption_fn` - Optional decryption function (from EncryptionHandler)
/// * `obj_num` - Object number (for encryption key derivation)
/// * `gen_num` - Generation number (for encryption key derivation)
///
/// # Returns
///
/// A HashMap mapping object numbers to their parsed objects.
///
/// # Errors
///
/// Returns an error if:
/// - The object is not a stream
/// - The stream is not a valid object stream (/Type /ObjStm)
/// - Required dictionary entries (/N, /First) are missing
/// - Stream decoding/decryption fails
/// - Object parsing fails
pub fn parse_object_stream_with_decryption(
    stream_obj: &Object,
    decryption_fn: Option<&dyn Fn(&[u8]) -> Result<Vec<u8>>>,
    obj_num: u32,
    gen_num: u32,
) -> Result<HashMap<u32, Object>> {
    let budget = match stream_obj {
        Object::Stream { data, .. } => data.budget(),
        _ => None,
    };
    if let Some(budget) = budget {
        budget.check()?;
    }
    // Extract stream dictionary and data
    let dict = match stream_obj {
        Object::Stream { dict, .. } => dict,
        _ => return Err(Error::InvalidPdf("object stream is not a Stream object".to_string())),
    };

    // Verify this is an object stream
    if let Some(type_obj) = dict.get("Type") {
        if let Some(type_name) = type_obj.as_name() {
            if type_name != "ObjStm" {
                return Err(Error::InvalidPdf(format!(
                    "expected /Type /ObjStm, got /Type /{}",
                    type_name
                )));
            }
        }
    }

    // Get required parameters
    let n = dict
        .get("N")
        .and_then(|o| o.as_integer())
        .ok_or_else(|| Error::InvalidPdf("object stream missing /N entry".to_string()))?;

    let first = dict
        .get("First")
        .and_then(|o| o.as_integer())
        .ok_or_else(|| Error::InvalidPdf("object stream missing /First entry".to_string()))?;

    // Validate parameters
    if !(0..=1_000_000).contains(&n) {
        return Err(Error::InvalidPdf(format!("invalid object stream /N value: {}", n)));
    }

    if !(0..=10_000_000).contains(&first) {
        return Err(Error::InvalidPdf(format!("invalid object stream /First value: {}", first)));
    }

    let n = n as usize;
    let first = first as usize;
    if let Some(budget) = budget {
        budget.charge_entries(n)?;
    }

    // Decode the stream data (with decryption if provided)
    let decoded_data =
        stream_obj.decode_stream_data_with_decryption(decryption_fn, obj_num, gen_num)?;

    // Validate decoded data size
    if decoded_data.len() < first {
        return Err(Error::InvalidPdf(format!(
            "object stream data too short: {} bytes, expected at least {}",
            decoded_data.len(),
            first
        )));
    }

    // Parse the pairs section (before /First)
    let pairs_data = &decoded_data[..first];
    let pairs = parse_object_number_pairs(pairs_data, n)?;

    // Parse the objects section (after /First)
    let objects_data = &decoded_data[first..];
    let mut result = HashMap::new();

    for (obj_num, offset_in_data) in pairs {
        // The offset is relative to the start of objects_data
        if offset_in_data >= objects_data.len() {
            log::warn!(
                "Object {} offset {} is beyond stream data length {}",
                obj_num,
                offset_in_data,
                objects_data.len()
            );
            continue;
        }

        // Parse the object starting at this offset
        let obj_data = &objects_data[offset_in_data..];
        match crate::parser::parse_object_with_budget(obj_data, budget) {
            Ok((_remaining, obj)) => {
                result.insert(obj_num, obj);
            },
            Err(e) => {
                if let Some(budget) = budget {
                    budget.check()?;
                }
                log::warn!(
                    "Failed to parse object {} from stream at offset {}: {:?}",
                    obj_num,
                    offset_in_data,
                    e
                );
                // Continue parsing other objects even if one fails
                continue;
            },
        }
    }

    Ok(result)
}

/// Parse the pairs section of an object stream.
///
/// The pairs section contains N pairs of integers: (object_number, offset).
/// The offset is relative to the start of the objects data section.
///
/// # Arguments
///
/// * `data` - The pairs section data (before /First offset)
/// * `count` - Expected number of pairs (from /N)
///
/// # Returns
///
/// A vector of (object_number, offset) tuples.
fn parse_object_number_pairs(data: &[u8], count: usize) -> Result<Vec<(u32, usize)>> {
    let mut pairs = Vec::with_capacity(count);
    let mut remaining = data;

    for i in 0..count {
        // Skip whitespace
        remaining = skip_whitespace(remaining);

        // Parse object number
        let (rest, obj_num_str) =
            read_integer_string(remaining).ok_or_else(|| Error::ParseError {
                offset: 0,
                reason: format!("failed to parse object number for pair {}", i),
            })?;

        let obj_num: u32 = obj_num_str.parse().map_err(|_| Error::ParseError {
            offset: 0,
            reason: format!("invalid object number: {}", obj_num_str),
        })?;

        remaining = skip_whitespace(rest);

        // Parse offset
        let (rest, offset_str) =
            read_integer_string(remaining).ok_or_else(|| Error::ParseError {
                offset: 0,
                reason: format!("failed to parse offset for pair {}", i),
            })?;

        let offset: usize = offset_str.parse().map_err(|_| Error::ParseError {
            offset: 0,
            reason: format!("invalid offset: {}", offset_str),
        })?;

        pairs.push((obj_num, offset));
        remaining = rest;
    }

    Ok(pairs)
}

/// Skip PDF whitespace characters.
///
/// PDF whitespace: null (0), tab (9), LF (10), FF (12), CR (13), space (32)
fn skip_whitespace(data: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            0 | 9 | 10 | 12 | 13 | 32 => i += 1,
            _ => break,
        }
    }
    &data[i..]
}

/// Read an integer string from the input.
///
/// Reads consecutive digit characters (with optional leading sign).
/// Returns the remaining input and the integer string.
fn read_integer_string(data: &[u8]) -> Option<(&[u8], String)> {
    if data.is_empty() {
        return None;
    }

    let mut i = 0;

    // Optional sign
    if data[i] == b'+' || data[i] == b'-' {
        i += 1;
    }

    // Must have at least one digit
    let start = i;
    while i < data.len() && data[i].is_ascii_digit() {
        i += 1;
    }

    if i == start {
        return None; // No digits found
    }

    let int_str = String::from_utf8_lossy(&data[..i]).to_string();
    Some((&data[i..], int_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import_limits::StreamData;
    use std::collections::HashMap;

    #[test]
    fn test_skip_whitespace() {
        assert_eq!(skip_whitespace(b"   hello"), b"hello");
        assert_eq!(skip_whitespace(b"\t\n\r hello"), b"hello");
        assert_eq!(skip_whitespace(b"hello"), b"hello");
        assert_eq!(skip_whitespace(b""), b"");
    }

    #[test]
    fn test_read_integer_string() {
        assert_eq!(read_integer_string(b"123 rest"), Some((&b" rest"[..], "123".to_string())));
        assert_eq!(read_integer_string(b"-456 rest"), Some((&b" rest"[..], "-456".to_string())));
        assert_eq!(read_integer_string(b"+789"), Some((&b""[..], "+789".to_string())));
        assert_eq!(read_integer_string(b"notanumber"), None);
        assert_eq!(read_integer_string(b""), None);
    }

    #[test]
    fn test_parse_object_number_pairs() {
        let data = b"10 0 11 15 12 28";
        let pairs = parse_object_number_pairs(data, 3).unwrap();

        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], (10, 0));
        assert_eq!(pairs[1], (11, 15));
        assert_eq!(pairs[2], (12, 28));
    }

    #[test]
    fn test_parse_object_number_pairs_with_whitespace() {
        let data = b"  10   0   11  15  12   28  ";
        let pairs = parse_object_number_pairs(data, 3).unwrap();

        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], (10, 0));
        assert_eq!(pairs[1], (11, 15));
        assert_eq!(pairs[2], (12, 28));
    }

    #[test]
    fn test_parse_object_stream_basic() {
        // Create a simple object stream with two objects
        // Object 10: integer 42
        // Object 11: name /Test

        let pairs_data = b"10 0 11 3"; // obj 10 at offset 0, obj 11 at offset 3
        let objects_data = b"42 /Test"; // "42 " is 3 bytes, then "/Test"

        let mut combined = Vec::new();
        combined.extend_from_slice(pairs_data);
        combined.push(b' '); // separator
        combined.extend_from_slice(objects_data);

        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("ObjStm".to_string()));
        dict.insert("N".to_string(), Object::Integer(2));
        dict.insert("First".to_string(), Object::Integer(9)); // Length of pairs section
        dict.insert("Length".to_string(), Object::Integer(combined.len() as i64));

        let stream = Object::Stream {
            dict,
            data: Bytes::from(combined),
        };

        let objects = parse_object_stream(&stream).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects.get(&10).unwrap().as_integer(), Some(42));
        assert_eq!(objects.get(&11).unwrap().as_name(), Some("Test"));
    }

    #[test]
    fn test_parse_object_stream_not_stream() {
        let obj = Object::Integer(42);
        let result = parse_object_stream(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_stream_missing_type() {
        let mut dict = HashMap::new();
        dict.insert("N".to_string(), Object::Integer(1));
        dict.insert("First".to_string(), Object::Integer(5));

        let stream = Object::Stream {
            dict,
            data: Bytes::from(b"1 0 42".to_vec()),
        };

        // Should still work - Type is optional if we trust the caller
        let result = parse_object_stream(&stream);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_object_stream_missing_n() {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("ObjStm".to_string()));
        dict.insert("First".to_string(), Object::Integer(5));

        let stream = Object::Stream {
            dict,
            data: Bytes::from(b"1 0 42".to_vec()),
        };

        let result = parse_object_stream(&stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_stream_missing_first() {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("ObjStm".to_string()));
        dict.insert("N".to_string(), Object::Integer(1));

        let stream = Object::Stream {
            dict,
            data: Bytes::from(b"1 0 42".to_vec()),
        };

        let result = parse_object_stream(&stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_stream_invalid_n() {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("ObjStm".to_string()));
        dict.insert("N".to_string(), Object::Integer(-1));
        dict.insert("First".to_string(), Object::Integer(5));

        let stream = Object::Stream {
            dict,
            data: Bytes::from(b"1 0 42".to_vec()),
        };

        let result = parse_object_stream(&stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_stream_data_too_short() {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("ObjStm".to_string()));
        dict.insert("N".to_string(), Object::Integer(1));
        dict.insert("First".to_string(), Object::Integer(100)); // Too large

        let stream = Object::Stream {
            dict,
            data: Bytes::from(b"1 0 42".to_vec()),
        };

        let result = parse_object_stream(&stream);
        assert!(result.is_err());
    }
}
