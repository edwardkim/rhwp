//! Stream decoder implementations for PDF filters.
//!
//! This module provides decoders for various PDF compression and encoding filters:
//! - FlateDecode (zlib/deflate) - most common
//! - ASCIIHexDecode - hexadecimal encoding
//! - ASCII85Decode - base85 encoding
//! - LZWDecode - LZW compression
//! - RunLengthDecode - run-length encoding
//! - DCTDecode - JPEG (pass-through)
//! - CCITTFaxDecode - CCITT Fax compression (pass-through)
//! - JBIG2Decode - JBIG2 compression (pass-through)
//!
//! Decoders can be chained together in a filter pipeline.

use crate::error::{Error, Result};
use crate::parser_config::ParserOptions;

mod ascii85;
mod ascii_hex;
mod bounded;
mod brotli;
pub(crate) mod ccitt;
mod dct;
mod flate;
mod flate_limited;
mod jbig2;
pub(crate) mod jpx;
mod lzw;
mod predictor;
mod runlength;

pub use ascii85::Ascii85Decoder;
pub use ascii_hex::AsciiHexDecoder;
pub use brotli::BrotliDecoder;
pub use ccitt::{decode_with_budget as decode_ccitt_with_budget, CcittDecoded, CcittFaxDecoder};
pub use dct::DctDecoder;
pub use flate::FlateDecoder;
pub use jbig2::Jbig2Decoder;
pub use lzw::LzwDecoder;
pub use predictor::{decode_predictor, CcittParams, DecodeParams, PngPredictor};
pub use runlength::RunLengthDecoder;

/// Security limits for decompression (decompression bomb protection).
///
/// PDF Spec: ISO 32000-1:2008 does not specify decompression limits, but these
/// are necessary security measures to prevent memory exhaustion attacks.
///
/// Default values:
/// - Max decompression ratio: 100:1 (compressed:decompressed)
/// - Max decompressed size: 100 MB
const DEFAULT_MAX_DECOMPRESSION_RATIO: u32 = 100;
const DEFAULT_MAX_DECOMPRESSED_SIZE: usize = 100 * 1024 * 1024;

/// PDF stream filter types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// FlateDecode (deflate/zlib compression)
    FlateDecode,
    /// ASCIIHexDecode (hexadecimal encoding)
    ASCIIHexDecode,
    /// ASCII85Decode (base-85 encoding)
    ASCII85Decode,
    /// LZWDecode (Lempel-Ziv-Welch compression)
    LZWDecode,
    /// RunLengthDecode (run-length encoding)
    RunLengthDecode,
    /// DCTDecode (JPEG compression)
    DCTDecode,
    /// CCITTFaxDecode (CCITT Fax compression)
    CCITTFaxDecode,
    /// JBIG2Decode (JBIG2 compression)
    JBIG2Decode,
    /// JPXDecode (JPEG 2000 compression)
    JPXDecode,
    /// BrotliDecode (Brotli compression, PDF 2.0)
    BrotliDecode,
}

/// Trait for PDF stream decoders.
///
/// Each decoder implements a specific PDF filter algorithm and can decode
/// compressed or encoded stream data.
pub trait StreamDecoder {
    /// Decode the input data.
    ///
    /// # Arguments
    ///
    /// * `input` - The encoded/compressed data
    ///
    /// # Returns
    ///
    /// The decoded data or an error if decoding fails.
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Get the name of this decoder (e.g., "FlateDecode").
    fn name(&self) -> &str;
}

/// Normalize a PDF filter name, handling spec abbreviations and case variations.
///
/// PDF Spec: ISO 32000-1:2008, Table 6 — Standard filter abbreviations.
fn normalize_filter_name(name: &str) -> Result<&'static str> {
    // Fast path: exact match
    match name {
        "FlateDecode" => return Ok("FlateDecode"),
        "ASCIIHexDecode" => return Ok("ASCIIHexDecode"),
        "ASCII85Decode" => return Ok("ASCII85Decode"),
        "LZWDecode" => return Ok("LZWDecode"),
        "RunLengthDecode" => return Ok("RunLengthDecode"),
        "DCTDecode" => return Ok("DCTDecode"),
        "CCITTFaxDecode" => return Ok("CCITTFaxDecode"),
        "JBIG2Decode" => return Ok("JBIG2Decode"),
        "JPXDecode" => return Ok("JPXDecode"),
        "BrotliDecode" => return Ok("BrotliDecode"),
        _ => {},
    }

    // PDF spec abbreviations (Table 6)
    match name {
        "Fl" => return Ok("FlateDecode"),
        "AHx" => return Ok("ASCIIHexDecode"),
        "A85" => return Ok("ASCII85Decode"),
        "LZW" => return Ok("LZWDecode"),
        "RL" => return Ok("RunLengthDecode"),
        "DCT" => return Ok("DCTDecode"),
        "CCF" => return Ok("CCITTFaxDecode"),
        _ => {},
    }

    // Case-insensitive fallback
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "flatedecode" => Ok("FlateDecode"),
        "asciihexdecode" => Ok("ASCIIHexDecode"),
        "ascii85decode" => Ok("ASCII85Decode"),
        "lzwdecode" => Ok("LZWDecode"),
        "runlengthdecode" => Ok("RunLengthDecode"),
        "dctdecode" => Ok("DCTDecode"),
        "ccittfaxdecode" => Ok("CCITTFaxDecode"),
        "jbig2decode" => Ok("JBIG2Decode"),
        "jpxdecode" => Ok("JPXDecode"),
        "brotlidecode" => Ok("BrotliDecode"),
        _ => Err(Error::UnsupportedFilter(name.to_string())),
    }
}

fn create_decoder(filter_name: &str) -> Result<Box<dyn StreamDecoder>> {
    let canonical = normalize_filter_name(filter_name)?;
    Ok(match canonical {
        "FlateDecode" => Box::new(FlateDecoder::default()),
        "ASCIIHexDecode" => Box::new(AsciiHexDecoder),
        "ASCII85Decode" => Box::new(Ascii85Decoder),
        "LZWDecode" => Box::new(LzwDecoder),
        "RunLengthDecode" => Box::new(RunLengthDecoder),
        "DCTDecode" => Box::new(DctDecoder),
        "CCITTFaxDecode" => Box::new(CcittFaxDecoder),
        "JBIG2Decode" => Box::new(Jbig2Decoder),
        "JPXDecode" => Box::new(jpx::JpxDecoder),
        "BrotliDecode" => Box::new(BrotliDecoder),
        // normalize_filter_name already returns Err for unknown filters
        _ => unreachable!(),
    })
}

/// Decode stream data using a filter pipeline.
///
/// PDF streams can have multiple filters applied in sequence. This function
/// applies each filter in order to decode the data.
///
/// # Arguments
///
/// * `data` - The raw stream data
/// * `filters` - List of filter names to apply in order
///
/// # Returns
///
/// The fully decoded data or an error if any filter fails.
///
/// # Examples
///
/// ```rust,no_run
/// use pdf_oxide::decoders::decode_stream;
///
/// let compressed_data = vec![/* compressed bytes */];
/// let filters = vec!["FlateDecode".to_string()];
/// let decoded = decode_stream(&compressed_data, &filters).unwrap();
/// ```
pub fn decode_stream(data: &[u8], filters: &[String]) -> Result<Vec<u8>> {
    decode_stream_with_params(data, filters, None)
}

/// Decode with a shared request budget, checking every output extension.
pub fn decode_stream_with_budget(
    data: &[u8],
    filters: &[String],
    params: Option<&DecodeParams>,
    budget: &crate::import_limits::ImportBudget,
) -> Result<Vec<u8>> {
    bounded::decode(data, filters, params, budget)
}

/// Decode stream data with parser options (includes decompression bomb protection).
///
/// This function extends `decode_stream` by supporting parser options for
/// security limits and strict mode behavior.
///
/// # Arguments
///
/// * `data` - The raw stream data
/// * `filters` - List of filter names to apply in order
/// * `params` - Optional decode parameters (for predictors, etc.)
/// * `options` - Parser options for security limits
///
/// # Returns
///
/// The fully decoded data or an error if any filter fails or security limits are exceeded.
///
/// # Security
///
/// This function includes decompression bomb protection:
/// - Checks decompression ratio before decompressing
/// - Checks output size limit after decompression
/// - Uses limits from `options` or defaults if None
pub fn decode_stream_with_options(
    data: &[u8],
    filters: &[String],
    params: Option<&DecodeParams>,
    options: Option<&ParserOptions>,
) -> Result<Vec<u8>> {
    // Get security limits from options or use defaults
    let max_ratio = options
        .map(|o| o.max_decompression_ratio)
        .unwrap_or(DEFAULT_MAX_DECOMPRESSION_RATIO);
    let max_size = options
        .map(|o| o.max_decompressed_size)
        .unwrap_or(DEFAULT_MAX_DECOMPRESSED_SIZE);

    let compressed_size = data.len();
    let mut current = data.to_vec();

    // Apply filters in order
    for filter_name in filters {
        let decoder = create_decoder(filter_name)?;

        current = decoder.decode(&current)?;

        // SECURITY: Check decompression ratio after each filter
        // PDF Spec: ISO 32000-1:2008 does not specify limits, but this is a
        // critical security measure to prevent decompression bomb attacks.
        if max_ratio > 0 && compressed_size > 0 {
            let ratio = current.len() as u64 / compressed_size.max(1) as u64;
            if ratio > max_ratio as u64 {
                return Err(Error::Decode(format!(
                    "Decompression bomb detected: ratio {}:1 exceeds limit {}:1 (compressed: {} bytes, decompressed: {} bytes)",
                    ratio,
                    max_ratio,
                    compressed_size,
                    current.len()
                )));
            }
        }

        // SECURITY: Check maximum decompressed size
        if max_size > 0 && current.len() > max_size {
            return Err(Error::Decode(format!(
                "Decompression bomb detected: decompressed size {} bytes exceeds limit {} bytes",
                current.len(),
                max_size
            )));
        }
    }

    // Apply predictor if specified
    if let Some(params) = params {
        if params.predictor != 1 {
            current = decode_predictor(&current, params)?;
        }
    }

    Ok(current)
}

/// Decode stream data using a filter pipeline with optional decode parameters.
///
/// This function extends `decode_stream` by supporting decode parameters
/// (e.g., PNG predictors) that are applied after the main filters.
///
/// # Arguments
///
/// * `data` - The raw stream data
/// * `filters` - List of filter names to apply in order
/// * `params` - Optional decode parameters (for predictors, etc.)
///
/// # Returns
///
/// The fully decoded data or an error if any filter fails.
pub fn decode_stream_with_params(
    data: &[u8],
    filters: &[String],
    params: Option<&DecodeParams>,
) -> Result<Vec<u8>> {
    let mut current = data.to_vec();

    // Apply filters in order
    for filter_name in filters {
        let decoder = create_decoder(filter_name)?;

        current = decoder.decode(&current)?;
    }

    // Apply predictor if specified
    if let Some(params) = params {
        if params.predictor != 1 {
            current = decode_predictor(&current, params)?;
        }
    }

    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_stream_no_filters() {
        let data = b"Hello, World!";
        let result = decode_stream(data, &[]).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_decode_stream_unsupported_filter() {
        let data = b"test";
        let filters = vec!["UnsupportedFilter".to_string()];
        let result = decode_stream(data, &filters);
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::UnsupportedFilter(name)) => {
                assert_eq!(name, "UnsupportedFilter");
            },
            _ => panic!("Expected UnsupportedFilter error"),
        }
    }

    #[test]
    fn test_decode_stream_pipeline() {
        // Test with ASCIIHexDecode
        let data = b"48656C6C6F"; // "Hello" in hex
        let filters = vec!["ASCIIHexDecode".to_string()];
        let result = decode_stream(data, &filters).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_normalize_filter_abbreviations() {
        assert_eq!(normalize_filter_name("A85").unwrap(), "ASCII85Decode");
        assert_eq!(normalize_filter_name("AHx").unwrap(), "ASCIIHexDecode");
        assert_eq!(normalize_filter_name("LZW").unwrap(), "LZWDecode");
        assert_eq!(normalize_filter_name("Fl").unwrap(), "FlateDecode");
        assert_eq!(normalize_filter_name("RL").unwrap(), "RunLengthDecode");
        assert_eq!(normalize_filter_name("CCF").unwrap(), "CCITTFaxDecode");
        assert_eq!(normalize_filter_name("DCT").unwrap(), "DCTDecode");
    }

    #[test]
    fn test_normalize_filter_case_insensitive() {
        assert_eq!(normalize_filter_name("Flatedecode").unwrap(), "FlateDecode");
        assert_eq!(normalize_filter_name("FLATEDECODE").unwrap(), "FlateDecode");
        assert_eq!(normalize_filter_name("flatedecode").unwrap(), "FlateDecode");
        assert_eq!(normalize_filter_name("ascii85decode").unwrap(), "ASCII85Decode");
        assert_eq!(normalize_filter_name("ASCIIHEXDECODE").unwrap(), "ASCIIHexDecode");
    }

    #[test]
    fn test_normalize_filter_unknown() {
        let result = normalize_filter_name("BogusFilter");
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::UnsupportedFilter(name)) => {
                assert_eq!(name, "BogusFilter");
            },
            _ => panic!("Expected UnsupportedFilter error"),
        }
    }

    #[test]
    fn test_decode_stream_with_abbreviation() {
        let data = b"48656C6C6F"; // "Hello" in hex
        let filters = vec!["AHx".to_string()];
        let result = decode_stream(data, &filters).unwrap();
        assert_eq!(result, b"Hello");
    }
}
