//! FlateDecode (zlib/deflate) implementation.
//!
//! This is the most common PDF compression filter, used in ~90% of PDFs.
//! Uses the flate2 crate for zlib decompression.

#![forbid(unsafe_code)]

use crate::decoders::StreamDecoder;
use crate::error::{Error, Result};
use flate2::read::{DeflateDecoder, ZlibDecoder};
use std::io::Read;

/// Default cap for [`FlateDecoder`]: 256 MB per stream.
///
/// Prevents zip-bomb / flate-bomb attacks where a tiny compressed stream
/// expands to an arbitrarily large output, exhausting virtual memory and
/// triggering an allocator abort (SIGABRT / exit 134).
///
/// 256 MB accommodates A4 @ 600 DPI RGB (~99 MB) with headroom.
///
/// Override via:
/// - `PDF_OXIDE_MAX_DECOMPRESS_MB` environment variable (e.g. `64` for 64 MB)
/// - [`FlateDecoder::with_limit`] for programmatic control
pub const DEFAULT_MAX_DECOMPRESSED_BYTES: u64 = 256 * 1024 * 1024;

fn effective_limit_from_str(val: Option<&str>) -> u64 {
    val.and_then(|v| v.parse::<u64>().ok())
        .map(|mb| mb * 1024 * 1024)
        .unwrap_or(DEFAULT_MAX_DECOMPRESSED_BYTES)
}

/// Read the decompression limit from the environment, falling back to the
/// compile-time default.
fn effective_limit() -> u64 {
    let val = std::env::var("PDF_OXIDE_MAX_DECOMPRESS_MB");
    effective_limit_from_str(val.as_deref().ok())
}

/// Heuristic validator for partial-recovery output from a failing decompress.
///
/// Returns `true` if the decoded bytes look like a plausible PDF stream — any
/// of: a content-stream operator (BT/ET/Tj/TJ/Tm/Td), a common PDF object
/// marker (<</>>/stream/obj/endobj), or a `%PDF-` prefix. The set is kept
/// intentionally broad because FlateDecode also wraps object streams, xref
/// streams, font programs, and image data, none of which carry text operators.
///
/// The guard exists to distinguish genuine partially-decoded output from
/// `deflate` "success" on a misaligned input — the latter produces short runs
/// of pseudo-random bytes (#364 symptom: 128 bytes of `P\xffj!}` repeating)
/// that contain none of these markers.
///
/// Conservative fallback: if the decoded bytes are mostly ASCII (printable
/// plus whitespace), the output is also treated as plausible, because stream
/// contents in the wild include ASCII-only data (hex-encoded images, small
/// object streams) that do not hit any specific marker.
/// A deflate stream that simply *stops* — no final block marker — rather than
/// one whose bits are corrupt.
///
/// flate2 1.1.10 began rejecting these outright ("Reject incomplete deflate
/// streams at EOF", flate2-rs#556); 1.1.9 returned whatever it had decoded.
/// Real PDFs are full of them: an incremental-update xref stream in the pdf.js
/// corpus is 51 compressed bytes that decode to 70 and never terminate, and
/// zlib, poppler and pdf.js all hand back those 70 bytes. Truncation cannot
/// corrupt the prefix — it only cuts it short — so the bytes decoded before the
/// end are exactly what a tolerant reader produces.
fn is_truncation(e: &std::io::Error) -> bool {
    if e.kind() == std::io::ErrorKind::UnexpectedEof {
        return true;
    }
    // zlib-rs / miniz_oxide report it in the message rather than the kind.
    let msg = e.to_string();
    msg.contains("incomplete deflate stream") || msg.contains("unexpected end")
}

fn looks_like_real_stream(output: &[u8]) -> bool {
    if output.is_empty() {
        return false;
    }
    // Cheap, content-stream-oriented markers first.
    const MARKERS: &[&[u8]] = &[
        b"BT", b"ET", b"Tj", b"TJ", b"Tm", b"Td", b"stream", b"endobj", b"%PDF-",
    ];
    for m in MARKERS {
        if output.windows(m.len()).any(|w| w == *m) {
            return true;
        }
    }
    // Fallback: accept outputs that are ≥ 80% printable/whitespace ASCII.
    // This catches legitimate but marker-less content (hex palettes, short
    // object streams) while still rejecting the high-bit-heavy garbage the
    // partial-recovery path produces on misaligned deflate input.
    let printable = output
        .iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b) || b == b'\t' || b == b'\n' || b == b'\r')
        .count();
    printable * 5 >= output.len() * 4
}

/// Returns `Err` if `output` reached the decompression cap, indicating that the
/// stream was truncated rather than fully decoded.
#[inline]
fn check_limit(output: &[u8], limit: u64) -> Result<()> {
    if output.len() as u64 >= limit {
        return Err(Error::Decode(format!(
            "FlateDecode output reached the {} MB safety limit; \
             stream may be a flate bomb or an unusually large image",
            limit / (1024 * 1024)
        )));
    }
    Ok(())
}

/// FlateDecode filter implementation.
///
/// Decompresses data using the zlib/deflate algorithm. The decompression cap
/// defaults to `DEFAULT_MAX_DECOMPRESSED_BYTES` and can be overridden with
/// [`FlateDecoder::with_limit`].
pub struct FlateDecoder {
    /// Maximum number of decompressed bytes accepted per stream.
    pub max_decompressed_bytes: u64,
}

impl Default for FlateDecoder {
    fn default() -> Self {
        Self {
            max_decompressed_bytes: effective_limit(),
        }
    }
}

impl FlateDecoder {
    /// Creates a decoder that rejects any stream decompressing to more than
    /// `limit` bytes. Use this to tighten or relax the default 256 MB cap.
    pub fn with_limit(limit: u64) -> Self {
        Self {
            max_decompressed_bytes: limit,
        }
    }
}

impl StreamDecoder for FlateDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        // A zero-length stream decodes to zero bytes. Every deflate
        // implementation this crate has used said so by returning success;
        // rejecting incomplete streams at EOF turned the empty stream into an
        // error, and the partial-recovery path below cannot rescue it because
        // it requires a non-empty buffer to inspect. Real files carry these:
        // a zero-area transparency group is written as an empty Form XObject,
        // and failing one aborts the parent content stream part-way through,
        // dropping every mark that would have been painted after it.
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut decoder = ZlibDecoder::new(input).take(self.max_decompressed_bytes);
        let mut output = Vec::new();

        // Try to read all data with standard zlib
        match decoder.read_to_end(&mut output) {
            Ok(_) => {
                check_limit(&output, self.max_decompressed_bytes)?;
                Ok(output)
            },
            Err(e) => {
                // Partial recovery: return only if output *looks like* a
                // plausible stream. The pre-fix behaviour accepted any
                // non-empty buffer, which let strategies 2 and 3 return
                // misaligned-deflate garbage (`P\xffj!}` × 16 on
                // nougat_026.pdf pages 1/2/5) that the text extractor then
                // emitted as zero bytes of output.
                // `looks_like_real_stream` is a *content-stream* heuristic — it
                // wants BT/Tj markers or mostly-printable ASCII. A truncated
                // xref or object stream is binary, so it fails that test and the
                // whole file then falls back to xref reconstruction, losing
                // objects. Gate on the failure mode instead: a truncated stream's
                // prefix is valid, a corrupt one's is not.
                if !output.is_empty() && (is_truncation(&e) || looks_like_real_stream(&output)) {
                    check_limit(&output, self.max_decompressed_bytes)?;
                    log::warn!(
                        "FlateDecode partial recovery: extracted {} bytes before corruption: {}",
                        output.len(),
                        e
                    );
                    return Ok(output);
                }

                // Strategy 2: Try raw deflate (no zlib wrapper)
                // Some PDFs have corrupt zlib headers but valid deflate data
                log::info!("Zlib decode failed, trying raw deflate");
                output.clear();
                let mut deflate_decoder =
                    DeflateDecoder::new(input).take(self.max_decompressed_bytes);

                match deflate_decoder.read_to_end(&mut output) {
                    Ok(_) => {
                        check_limit(&output, self.max_decompressed_bytes)?;
                        log::info!("Raw deflate recovery succeeded: {} bytes", output.len());
                        Ok(output)
                    },
                    Err(deflate_err) => {
                        if !output.is_empty()
                            && (is_truncation(&deflate_err) || looks_like_real_stream(&output))
                        {
                            check_limit(&output, self.max_decompressed_bytes)?;
                            log::warn!(
                                "Raw deflate partial recovery: extracted {} bytes before error",
                                output.len()
                            );
                            return Ok(output);
                        }

                        // Strategy 3: Try skipping zlib header (2 bytes) and reading deflate
                        if input.len() > 2 {
                            log::info!(
                                "Trying deflate after skipping potential corrupt zlib header"
                            );
                            output.clear();
                            let mut deflate_decoder =
                                DeflateDecoder::new(&input[2..]).take(self.max_decompressed_bytes);

                            match deflate_decoder.read_to_end(&mut output) {
                                Ok(_) => {
                                    check_limit(&output, self.max_decompressed_bytes)?;
                                    log::info!(
                                        "Deflate with header skip succeeded: {} bytes",
                                        output.len()
                                    );
                                    return Ok(output);
                                },
                                Err(_) => {
                                    if !output.is_empty() && looks_like_real_stream(&output) {
                                        check_limit(&output, self.max_decompressed_bytes)?;
                                        log::warn!(
                                            "Deflate with header skip partial recovery: {} bytes",
                                            output.len()
                                        );
                                        return Ok(output);
                                    }
                                },
                            }
                        }

                        // Strategy 4: Try fixing corrupt zlib header byte
                        // If first byte has invalid compression method, replace with 0x78 (standard deflate)
                        if input.len() >= 2 {
                            let first_byte = input[0];
                            let compression_method = first_byte & 0x0F;
                            if compression_method != 8 {
                                log::info!(
                                    "Detected invalid compression method {} in header byte 0x{:02x}, trying with corrected header",
                                    compression_method,
                                    first_byte
                                );
                                // Create new buffer with corrected header
                                let mut corrected = input.to_vec();
                                // Replace CM bits (0-3) with 8 (deflate), keep CINFO bits (4-7)
                                corrected[0] = (first_byte & 0xF0) | 0x08;

                                output.clear();
                                let mut decoder = ZlibDecoder::new(&corrected[..])
                                    .take(self.max_decompressed_bytes);
                                match decoder.read_to_end(&mut output) {
                                    Ok(_) if !output.is_empty() => {
                                        check_limit(&output, self.max_decompressed_bytes)?;
                                        log::info!(
                                            "Header correction recovery succeeded: {} bytes",
                                            output.len()
                                        );
                                        return Ok(output);
                                    },
                                    Err(_)
                                        if !output.is_empty()
                                            && looks_like_real_stream(&output) =>
                                    {
                                        check_limit(&output, self.max_decompressed_bytes)?;
                                        log::warn!(
                                            "Header correction partial recovery: {} bytes",
                                            output.len()
                                        );
                                        return Ok(output);
                                    },
                                    _ => {
                                        log::info!("Header correction failed");
                                    },
                                }
                            }
                        }

                        // Strategy 5: Brute-force scan for valid deflate data
                        // Try starting deflate decompression from offsets 0-20
                        // BUT validate the output contains valid PDF operators
                        log::info!("Trying brute-force scan for valid deflate data");
                        let max_offset = std::cmp::min(20, input.len());
                        for offset in 0..max_offset {
                            if offset == 0 || offset == 2 {
                                continue; // Already tried these
                            }

                            output.clear();
                            let mut deflate_decoder = DeflateDecoder::new(&input[offset..])
                                .take(self.max_decompressed_bytes);

                            match deflate_decoder.read_to_end(&mut output) {
                                Ok(_) if !output.is_empty() => {
                                    check_limit(&output, self.max_decompressed_bytes)?;
                                    // Validate output quality - check for PDF operators
                                    let decoded_str = String::from_utf8_lossy(&output);
                                    let has_pdf_operators = decoded_str.contains("BT")
                                        || decoded_str.contains("ET")
                                        || decoded_str.contains("Tj")
                                        || decoded_str.contains("TJ")
                                        || decoded_str.contains("Tm")
                                        || decoded_str.contains("Td");

                                    if has_pdf_operators {
                                        log::info!(
                                            "Brute-force deflate recovery succeeded at offset {}: {} bytes (validated PDF content)",
                                            offset,
                                            output.len()
                                        );
                                        return Ok(output);
                                    } else {
                                        log::info!(
                                            "Brute-force at offset {} produced {} bytes but no valid PDF operators - trying next offset",
                                            offset,
                                            output.len()
                                        );
                                        continue;
                                    }
                                },
                                Err(_) if !output.is_empty() => {
                                    check_limit(&output, self.max_decompressed_bytes)?;
                                    // Validate partial recovery too
                                    let decoded_str = String::from_utf8_lossy(&output);
                                    let has_pdf_operators = decoded_str.contains("BT")
                                        || decoded_str.contains("ET")
                                        || decoded_str.contains("Tj")
                                        || decoded_str.contains("TJ")
                                        || decoded_str.contains("Tm")
                                        || decoded_str.contains("Td");

                                    if has_pdf_operators {
                                        log::warn!(
                                            "Brute-force partial recovery at offset {}: {} bytes (validated PDF content)",
                                            offset,
                                            output.len()
                                        );
                                        return Ok(output);
                                    } else {
                                        log::info!(
                                            "Partial recovery at offset {} but no valid PDF operators - trying next offset",
                                            offset
                                        );
                                        continue;
                                    }
                                },
                                _ => continue,
                            }
                        }

                        // SPEC COMPLIANCE FIX: Removed strategies 8-9 that violated PDF spec
                        //
                        // Previous strategies 8-9 would return raw uncompressed data for streams
                        // labeled as /FlateDecode. This violates PDF Spec ISO 32000-1:2008,
                        // Section 7.3.8.2 which states that if a stream has /Filter /FlateDecode,
                        // it MUST be compressed with the FlateDecode algorithm.
                        //
                        // Returning raw data creates security risks:
                        // 1. Malicious PDFs could bypass compression validation
                        // 2. Type confusion attacks (treating compressed data as raw)
                        // 3. Inconsistent behavior across PDF processors
                        //
                        // Correct behavior: If all decompression strategies fail, return an error.
                        // The stream is either corrupted or malicious, and should not be processed.

                        log::error!(
                            "All FlateDecode recovery strategies failed. Zlib: {}, Deflate: {}",
                            e,
                            deflate_err
                        );
                        log::error!(
                            "Stream labeled as FlateDecode but cannot be decompressed - this violates PDF spec"
                        );

                        Err(Error::Decode(format!(
                            "FlateDecode decompression failed: stream is labeled as compressed but all decompression attempts failed. \
                            This violates PDF Spec ISO 32000-1:2008, Section 7.3.8.2. \
                            Zlib error: {}, Deflate error: {}. Compressed size: {} bytes.",
                            e,
                            deflate_err,
                            input.len()
                        )))
                    },
                }
            },
        }
    }

    fn name(&self) -> &str {
        "FlateDecode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    // #364 — when a strategy's partial recovery emits garbage (valid-looking
    // deflate bits that decode to pseudo-random bytes on a misaligned input),
    // the decoder must not accept it. Strategy 5 already validates via PDF
    // content-stream operators; strategies 1–4 now validate via
    // `looks_like_real_stream`. This test pins that guard.
    #[test]
    fn looks_like_real_stream_rejects_repeating_garbage() {
        // Actual symptom from nougat_026.pdf page 1 before the fix: 128 bytes
        // of `P\xffj!}\xef\xbd\xbd\xef\xbd\xbd...` high-bit-heavy repetition.
        let garbage = b"P\xffj!}\xef\xbd\xbd\xef\xbd\xbd\xef\xbd\xbd\xef\xbd\xbd\xef\xbd\xbd\xef\xbd\xbd\xef\xbd\xbd\xef\xbd\xbd".repeat(4);
        assert!(
            !looks_like_real_stream(&garbage),
            "misaligned-deflate garbage must be rejected as a partial recovery"
        );
    }

    #[test]
    fn looks_like_real_stream_accepts_content_stream_operators() {
        let real = b"BT /F1 12 Tf 100 700 Td (hello) Tj ET";
        assert!(looks_like_real_stream(real));
    }

    #[test]
    fn looks_like_real_stream_accepts_ascii_only_object_stream() {
        // Object-stream-like payload: ASCII, no content-stream operators.
        let object_stream = b"1 0 obj\n<< /Length 42 >>\nstream\nhello world\nendstream\nendobj\n";
        assert!(looks_like_real_stream(object_stream));
    }

    #[test]
    fn looks_like_real_stream_rejects_empty() {
        assert!(!looks_like_real_stream(&[]));
    }

    /// A deflate stream that ends without its final block marker still yields
    /// the bytes it did encode.
    ///
    /// flate2 1.1.10 rejects such a stream where 1.1.9 returned the prefix
    /// (flate2-rs#556). PDFs in the wild rely on the tolerant behaviour: an
    /// incremental-update xref stream of 51 compressed bytes decodes to 70 and
    /// simply stops, and zlib, poppler and pdf.js all accept it. When this
    /// decoder refused it, xref parsing failed, the reader fell back to
    /// reconstruction, and a page tree rebuilt from an incomplete object table
    /// dropped a whole page's worth of text.
    ///
    /// The payload here is deliberately BINARY — an xref stream is not text —
    /// because the partial-recovery path used to be gated on a content-stream
    /// heuristic that only accepts `BT`/`Tj` markers or printable ASCII, and so
    /// rejected exactly this shape.
    #[test]
    fn test_empty_stream_decodes_to_no_bytes() {
        // An empty Form XObject — what a zero-area transparency group is
        // written as — reaches the decoder as a zero-length stream. It must
        // decode to nothing rather than fail: the caller renders the form, and
        // an error there abandons the rest of the parent content stream.
        let decoded = FlateDecoder::default()
            .decode(&[])
            .expect("an empty stream must decode to zero bytes, not fail");

        assert!(
            decoded.is_empty(),
            "an empty stream must not invent bytes, got {}",
            decoded.len()
        );
    }

    #[test]
    fn test_truncated_deflate_stream_yields_its_decoded_prefix() {
        // Binary payload, compressible, with no content-stream operators.
        let original: Vec<u8> = (0u16..600).map(|i| (i % 251) as u8).collect();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&original).unwrap();
        let complete = encoder.finish().unwrap();

        // Cut the tail off: the final block marker and checksum never arrive.
        let truncated = &complete[..complete.len() * 3 / 4];
        assert!(truncated.len() < complete.len(), "test needs a genuinely shortened stream");

        let decoded = FlateDecoder::default()
            .decode(truncated)
            .expect("a truncated deflate stream must still yield its decoded prefix");

        assert!(!decoded.is_empty(), "recovery returned nothing for a truncated stream");
        assert!(
            original.starts_with(&decoded),
            "recovered bytes must be a prefix of the original, not garbage"
        );
    }

    #[test]
    fn test_flate_decode_simple() {
        let decoder = FlateDecoder::default();

        // Compress some data
        let original = b"Hello, FlateDecode!";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // Decompress
        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_flate_decode_empty() {
        let decoder = FlateDecoder::default();

        // Compress empty data
        let original = b"";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_flate_decode_large_data() {
        let decoder = FlateDecoder::default();

        // Create large repeated data
        let original = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(1000);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_flate_decode_invalid_data() {
        let decoder = FlateDecoder::default();

        // Invalid zlib data - should fail decompression
        // SPEC COMPLIANCE: We now correctly reject invalid compressed data
        // instead of returning it as raw data (which violated PDF spec)
        let invalid = b"This is not zlib compressed data";
        let result = decoder.decode(invalid);
        assert!(result.is_err());

        // Verify error message mentions spec compliance
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("FlateDecode decompression failed"));
        }
    }

    #[test]
    fn test_flate_decoder_name() {
        let decoder = FlateDecoder::default();
        assert_eq!(decoder.name(), "FlateDecode");
    }

    #[test]
    fn test_flate_bomb_rejected() {
        // Verify that check_limit rejects output at or above the cap.
        let large = vec![0u8; DEFAULT_MAX_DECOMPRESSED_BYTES as usize];
        let result = check_limit(&large, DEFAULT_MAX_DECOMPRESSED_BYTES);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("safety limit"));
    }

    #[test]
    fn test_check_limit_below_threshold() {
        let small = vec![0u8; 1024];
        assert!(check_limit(&small, DEFAULT_MAX_DECOMPRESSED_BYTES).is_ok());
    }

    #[test]
    fn test_custom_limit_accepts_data_within_limit() {
        // A decoder with a small cap should accept data below that cap.
        let original = b"x".repeat(512);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decoder = FlateDecoder::with_limit(1024);
        let decoded = decoder.decode(&compressed).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_custom_limit_rejects_data_over_limit() {
        // A decoder with a tiny cap should reject data that exceeds it.
        let original = b"x".repeat(100);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decoder = FlateDecoder::with_limit(10);
        let result = decoder.decode(&compressed);
        assert!(result.is_err(), "expected rejection when output exceeds custom limit");
    }

    #[test]
    fn test_bomb_error_does_not_expose_internal_symbol_name() {
        // The user-facing error message must not reference internal symbol names.
        let large = vec![0u8; DEFAULT_MAX_DECOMPRESSED_BYTES as usize];
        let result = check_limit(&large, DEFAULT_MAX_DECOMPRESSED_BYTES);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            !msg.contains("MAX_DECOMPRESSED_BYTES"),
            "error message must not reference internal symbol names: {msg}"
        );
    }

    #[test]
    fn test_effective_limit_env_variable() {
        assert_eq!(effective_limit_from_str(None), DEFAULT_MAX_DECOMPRESSED_BYTES);
        assert_eq!(effective_limit_from_str(Some("64")), 64 * 1024 * 1024);
        assert_eq!(effective_limit_from_str(Some("not_a_number")), DEFAULT_MAX_DECOMPRESSED_BYTES);
    }
}
