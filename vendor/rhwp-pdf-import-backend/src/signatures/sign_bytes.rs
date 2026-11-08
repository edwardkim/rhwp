//! Two-pass PDF signing: appends an incremental update with a CMS/PKCS#7
//! signature over the ByteRange-covered portions of the document.
//!
//! ## Protocol (ISO 32000-1:2008 §12.8.1)
//!
//! 1. Append a new `obj` containing the signature dictionary with:
//!    - A fixed-width `/ByteRange [AAAAAAAAAA BBBBBBBBBB CCCCCCCCCC DDDDDDDDDD]`
//!      placeholder (each number occupies exactly 10 characters so the total
//!      field width is constant across both passes).
//!    - A zero-padded `/Contents <000...000>` placeholder whose size is
//!      `estimated_size * 2 + 2` (hex encoding + angle brackets).
//!    - The standard `/Filter /SubFilter /M` entries.
//! 2. Append a minimal xref section and trailer pointing back to the
//!    existing catalog (`/Root`) and previous xref (`/Prev`).
//! 3. Locate the `/Contents` placeholder offset; calculate the actual
//!    ByteRange from the total output length and that offset.
//! 4. Patch the ByteRange placeholder in-place (same total width, trailing
//!    spaces absorb the difference in decimal-digit count).
//! 5. Extract the two signed byte ranges; call `PdfSigner::sign`; insert
//!    the hex-encoded signature into the `/Contents` placeholder.

use super::signer::PdfSigner;
use super::types::{SignOptions, SigningCredentials};
use crate::error::{Error, Result};
use crate::object::encode_pdf_text_string;

// ─── Width constants ────────────────────────────────────────────────────────
//
// Each of the four ByteRange numbers occupies exactly BR_FIELD_W characters
// (right-justified, space-padded on the left). Keeping the width fixed means
// the total text produced by pass 1 is identical in length to pass 2, so
// /Contents offsets don't shift between passes.

const BR_FIELD_W: usize = 10;
const BR_PLACEHOLDER: &str = "0000000000 0000000000 0000000000 0000000000";

// ─── Public entry point ──────────────────────────────────────────────────────

/// Append a digital signature to `pdf_data` as an incremental update and
/// return the signed PDF bytes.
///
/// `credentials` must carry a valid X.509 certificate **and** private key
/// (e.g. loaded via [`SigningCredentials::from_pem`] or
/// [`SigningCredentials::from_pkcs12`]).
///
/// # Errors
///
/// Returns an error if:
/// - the existing PDF bytes do not contain a parseable `startxref` or
///   `/Root` entry (corrupted / not a PDF),
/// - the private key cannot be used to sign (wrong format or corrupted), or
/// - the estimated signature size is too small for the produced CMS blob.
pub fn sign_pdf_bytes(
    pdf_data: &[u8],
    credentials: &SigningCredentials,
    opts: SignOptions,
) -> Result<Vec<u8>> {
    // Legacy adbe.pkcs7.detached path — CMS via PdfSigner::sign,
    // byte-for-byte unchanged from prior releases.
    let signer = PdfSigner::new(credentials.clone(), opts);
    sign_pdf_bytes_with(pdf_data, signer, &|s, sb| s.sign(sb))
}

/// Thin wrapper over [`sign_pdf_bytes_with_cms`] that discards the CMS
/// blob — the byte-for-byte-unchanged legacy entry point.
fn sign_pdf_bytes_with(
    pdf_data: &[u8],
    signer: PdfSigner,
    cms_fn: &dyn Fn(&PdfSigner, &[u8]) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    sign_pdf_bytes_with_cms(pdf_data, signer, cms_fn).map(|(out, _cms)| out)
}

/// The shared byte-range / incremental-update assembler. The CMS blob
/// is produced by `cms_fn` — the *only* variation point between the
/// legacy and PAdES paths; the byte-range math is identical (and must
/// stay so — #235 plan §2.3 "do not modify the byte-range logic").
///
/// Returns `(signed_pdf, contents_string)`. `contents_string` is the
/// exact byte string stored in `/Contents` (the CMS DER **plus the
/// zero-padding** that fills the fixed-width placeholder) — i.e. the
/// literal value a reader's PDF parser hex-decodes. The B-LT path keys
/// its DSS `/VRI` off `SHA-1(contents_string)`; computing it here from
/// the same bytes the reader will (`SignatureInfo::contents`, used by
/// `classify_pades_level`) makes the write-side and read-side keys
/// identical *by construction* — and avoids `enumerate_signatures`,
/// which only surfaces AcroForm-linked fields, not the bare
/// `/Type /Sig` object this minimal incremental update emits.
fn sign_pdf_bytes_with_cms(
    pdf_data: &[u8],
    signer: PdfSigner,
    cms_fn: &dyn Fn(&PdfSigner, &[u8]) -> Result<Vec<u8>>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // ── 1. Extract the minimum structural info from the existing PDF ──────
    let prev_startxref = scan_startxref(pdf_data)
        .ok_or_else(|| Error::InvalidPdf("cannot find startxref in existing PDF".into()))?;
    let root_ref = scan_root_ref(pdf_data)
        .ok_or_else(|| Error::InvalidPdf("cannot find /Root ref in existing PDF".into()))?;
    let next_obj_num = scan_next_obj_num(pdf_data).ok_or_else(|| {
        Error::InvalidPdf("cannot determine next object number from PDF trailer".into())
    })?;

    // ── 2. Build the signature dictionary text (fixed-width placeholders) ─
    let sig_dict_text = build_sig_dict_text(&signer, next_obj_num);

    // Locate /Contents '<' offset within the sig dict text (will become the
    // offset we patch after computing ByteRange).
    let contents_in_dict = find_contents_offset_in_text(sig_dict_text.as_bytes())
        .ok_or_else(|| Error::InvalidPdf("cannot find /Contents in built sig dict".into()))?;

    // ── 3. Pre-compute all section offsets ────────────────────────────────
    let sig_dict_start = pdf_data.len(); // offset of "N 0 obj\n"
    let xref_start = sig_dict_start + sig_dict_text.len();

    let xref_entry = format!("{:010} 00000 n \r\n", sig_dict_start);
    let xref_section = format!("xref\n{} 1\n{}", next_obj_num, xref_entry);

    let trailer_section = format!(
        "trailer\n<< /Size {} /Prev {} /Root {} >>\n",
        next_obj_num + 1,
        prev_startxref,
        root_ref,
    );

    let startxref_section = format!("startxref\n{}\n%%EOF\n", xref_start);

    let total_len = sig_dict_start
        + sig_dict_text.len()
        + xref_section.len()
        + trailer_section.len()
        + startxref_section.len();

    // ── 4. Compute actual ByteRange ───────────────────────────────────────
    let contents_abs = sig_dict_start + contents_in_dict; // offset of '<'
    let contents_size = signer.placeholder_size(); // '<' + hex + '>'
    let after_contents = contents_abs + contents_size;
    let byte_range: [i64; 4] = [
        0,
        contents_abs as i64,
        after_contents as i64,
        (total_len - after_contents) as i64,
    ];

    // ── 5. Patch ByteRange placeholder in sig dict ────────────────────────
    let patched_sig_dict = patch_byterange(sig_dict_text, &byte_range);

    // ── 6. Assemble the full output ───────────────────────────────────────
    let mut output = Vec::with_capacity(total_len);
    output.extend_from_slice(pdf_data);
    output.extend_from_slice(patched_sig_dict.as_bytes());
    output.extend_from_slice(xref_section.as_bytes());
    output.extend_from_slice(trailer_section.as_bytes());
    output.extend_from_slice(startxref_section.as_bytes());

    debug_assert_eq!(
        output.len(),
        total_len,
        "assembled output length must match pre-computed total_len"
    );

    // ── 7. Extract signed bytes and sign ─────────────────────────────────
    let signed_bytes =
        super::byterange::ByteRangeCalculator::extract_signed_bytes(&output, &byte_range)?;
    let cms_blob = cms_fn(&signer, &signed_bytes)?;

    // ── 8. Insert signature ───────────────────────────────────────────────
    signer.insert_signature(&mut output, contents_abs, &cms_blob)?;

    // The `/Contents` *value* a reader parses is the CMS DER followed by
    // the zero-padding that fills the fixed-width hex placeholder
    // (`insert_signature` pads with `'0'` hex chars ⇒ `0x00` bytes).
    // `(placeholder_size - 2) / 2` is that decoded length in bytes
    // (`< … >` minus the two angle brackets, two hex chars per byte).
    let contents_len = (signer.placeholder_size() - 2) / 2;
    let mut contents_string = cms_blob;
    contents_string.resize(contents_len, 0);

    Ok((output, contents_string))
}

/// Sign a PDF at a PAdES baseline level (#235 TODO #12) — the
/// level-driven public entry point that ties the pieces together
/// (ETSI EN 319 142-1 §5). Reuses the proven byte-range assembler
/// verbatim; B-B/B-T differ only in the CMS builder, B-LT adds the DSS
/// as a *second* incremental update so the B-T signature byte-range is
/// untouched (feature plan §4.1), and B-LTA adds a `/DocTimeStamp`
/// (ETSI.RFC3161) over the whole file incl. the DSS as a *third*
/// incremental update (ETSI EN 319 142-1 §5).
///
/// `timestamper` (required for `BT`/`BLt`/`BLta`) returns the RFC 3161
/// token over the supplied digest input — an offline pre-fetched token
/// or a live TSA call by the caller. `material` is the DSS validation
/// set for `BLt`/`BLta`.
///
/// # Errors
/// - [`Error::Unsupported`] — `BT`/`BLt`/`BLta` without a `timestamper`.
/// - [`Error::InvalidPdf`] — unparseable PDF / signature.
#[allow(clippy::too_many_arguments)]
pub fn sign_pdf_bytes_pades(
    pdf_data: &[u8],
    credentials: &SigningCredentials,
    opts: SignOptions,
    level: crate::signatures::PadesLevel,
    timestamper: Option<&dyn Fn(&[u8]) -> Result<Vec<u8>>>,
    material: &crate::signatures::RevocationMaterial,
) -> Result<Vec<u8>> {
    use crate::signatures::PadesLevel;

    // B-T, B-LT and B-LTA all need an RFC 3161 token source (B-LTA's
    // document timestamp included).
    if matches!(level, PadesLevel::BT | PadesLevel::BLt | PadesLevel::BLta) && timestamper.is_none()
    {
        return Err(Error::Unsupported(
            "PAdES-B-T/B-LT/B-LTA require a timestamper (RFC 3161 token source)".into(),
        ));
    }

    let dts_size = opts.estimated_size.max(8192);
    let signer = PdfSigner::new(credentials.clone(), opts);
    let (signed, contents_string) = match level {
        PadesLevel::BB => sign_pdf_bytes_with_cms(pdf_data, signer, &|s, sb| s.sign_pades(sb)),
        PadesLevel::BT | PadesLevel::BLt | PadesLevel::BLta => {
            let ts = timestamper.expect("checked above");
            sign_pdf_bytes_with_cms(pdf_data, signer, &|s, sb| s.sign_pades_t(sb, ts))
        },
    }?;

    if level == PadesLevel::BB || level == PadesLevel::BT {
        return Ok(signed);
    }

    // B-LT: append the DSS as a SECOND incremental update keyed by the
    // signature's uppercase-hex SHA-1(/Contents) (the B-T byte range is
    // a strict prefix and stays valid — feature plan §4.1 / I1/I2). The
    // VRI key is computed over the *padded* /Contents byte string the
    // assembler returned — byte-identical to what a reader parses into
    // `SignatureInfo::contents` and feeds to `classify_pades_level`, so
    // the write-side and read-side keys match by construction. (Using
    // `enumerate_signatures` here would yield an empty VRI set —
    // silently degrading B-LT — for the bare `/Type /Sig` object this
    // path emits, since it only surfaces AcroForm-linked fields.)
    let doc = crate::document::PdfDocument::from_bytes(signed.clone())?;
    let keys: Vec<String> = crate::signatures::pades::vri_key(&contents_string)
        .into_iter()
        .collect();
    let blt = crate::signatures::pades::append_dss(&signed, &doc, material, &keys)?;

    if level == PadesLevel::BLt {
        return Ok(blt);
    }

    // B-LTA = B-LT + a `/DocTimeStamp` (ETSI.RFC3161) over the whole
    // file *including* the DSS, as a third incremental update — so the
    // archival timestamp covers the signature and its validation
    // material (ETSI EN 319 142-1 §5; feature plan §1.2).
    let ts = timestamper.expect("checked above");
    append_doc_timestamp(&blt, ts, dts_size)
}

// ─── Text builders ───────────────────────────────────────────────────────────

fn build_sig_dict_text(signer: &PdfSigner, obj_num: u64) -> String {
    let opts = signer.options();
    let contents_placeholder = signer.generate_contents_placeholder();

    let mut dict = format!(
        "{} 0 obj\n<< /Type /Sig\n/Filter /Adobe.PPKLite\n/SubFilter /{}\n",
        obj_num,
        opts.sub_filter.as_pdf_name(),
    );

    // Fixed-width ByteRange placeholder — patched in step 5.
    // The string inside [...] must exactly match BR_PLACEHOLDER so that
    // patch_byterange can find and replace it.
    dict.push_str(&format!("/ByteRange [{}]\n", BR_PLACEHOLDER));

    // /Contents: the '<' here is exactly what find_contents_offset_in_text looks for
    dict.push_str(&format!("/Contents {}\n", contents_placeholder));

    if let Some(ref r) = opts.reason {
        dict.push_str(&format!("/Reason {}\n", pdf_text_hex(r)));
    }
    if let Some(ref l) = opts.location {
        dict.push_str(&format!("/Location {}\n", pdf_text_hex(l)));
    }
    if let Some(ref n) = opts.name {
        dict.push_str(&format!("/Name {}\n", pdf_text_hex(n)));
    }
    if let Some(ref c) = opts.contact_info {
        dict.push_str(&format!("/ContactInfo {}\n", pdf_text_hex(c)));
    }

    dict.push_str(&format!("/M ({})\n", format_pdf_date()));
    dict.push_str(">>\nendobj\n");
    dict
}

/// Build a `/Type /DocTimeStamp` dictionary (PAdES-B-LTA, ETSI EN
/// 319 142-1 §5 / ISO 32000-2 §12.8.5): a `SubFilter /ETSI.RFC3161`
/// object whose `/Contents` is a bare RFC 3161 timestamp token over the
/// `/ByteRange`-covered bytes — *no* CMS SignerInfo, *no* signer
/// certificate. Same fixed-width placeholders as `build_sig_dict_text`.
fn build_doctimestamp_dict_text(obj_num: u64, contents_placeholder: &str) -> String {
    let mut dict = format!(
        "{} 0 obj\n<< /Type /DocTimeStamp\n/Filter /Adobe.PPKLite\n/SubFilter /ETSI.RFC3161\n",
        obj_num,
    );
    dict.push_str(&format!("/ByteRange [{}]\n", BR_PLACEHOLDER));
    dict.push_str(&format!("/Contents {}\n", contents_placeholder));
    dict.push_str(&format!("/M ({})\n", format_pdf_date()));
    dict.push_str(">>\nendobj\n");
    dict
}

/// Append a PAdES-B-LTA document timestamp as a further incremental
/// update over `pdf_data` (which already carries the B-T signature and
/// the B-LT DSS). The whole current file is the `/ByteRange`-covered
/// region, so the timestamp protects the signature *and* its DSS
/// (archival LTV — feature plan §1.2). `timestamper` returns the
/// RFC 3161 token over the supplied digest input (same closure contract
/// as B-T). The byte-range math is the proven protocol from
/// [`sign_pdf_bytes_with_cms`], replicated standalone so that function
/// stays byte-for-byte unmodified (#235 plan §2.3).
fn append_doc_timestamp(
    pdf_data: &[u8],
    timestamper: &dyn Fn(&[u8]) -> Result<Vec<u8>>,
    est_size: usize,
) -> Result<Vec<u8>> {
    let prev_startxref = scan_startxref(pdf_data)
        .ok_or_else(|| Error::InvalidPdf("B-LTA: cannot find startxref".into()))?;
    let root_ref = scan_root_ref(pdf_data)
        .ok_or_else(|| Error::InvalidPdf("B-LTA: cannot find /Root ref".into()))?;
    let next_obj_num = scan_next_obj_num(pdf_data)
        .ok_or_else(|| Error::InvalidPdf("B-LTA: cannot determine next object number".into()))?;

    let calc = super::byterange::ByteRangeCalculator::with_placeholder_size(est_size * 2 + 2);
    let placeholder = calc.generate_placeholder();
    let dict_text = build_doctimestamp_dict_text(next_obj_num, &placeholder);
    let contents_in_dict = find_contents_offset_in_text(dict_text.as_bytes())
        .ok_or_else(|| Error::InvalidPdf("B-LTA: cannot find /Contents in DocTimeStamp".into()))?;

    let sig_dict_start = pdf_data.len();
    let xref_start = sig_dict_start + dict_text.len();
    let xref_entry = format!("{:010} 00000 n \r\n", sig_dict_start);
    let xref_section = format!("xref\n{} 1\n{}", next_obj_num, xref_entry);
    let trailer_section = format!(
        "trailer\n<< /Size {} /Prev {} /Root {} >>\n",
        next_obj_num + 1,
        prev_startxref,
        root_ref,
    );
    let startxref_section = format!("startxref\n{}\n%%EOF\n", xref_start);
    let total_len = sig_dict_start
        + dict_text.len()
        + xref_section.len()
        + trailer_section.len()
        + startxref_section.len();

    let contents_abs = sig_dict_start + contents_in_dict;
    let contents_size = calc.placeholder_size();
    let after_contents = contents_abs + contents_size;
    let byte_range: [i64; 4] = [
        0,
        contents_abs as i64,
        after_contents as i64,
        (total_len - after_contents) as i64,
    ];

    let patched = patch_byterange(dict_text, &byte_range);
    let mut output = Vec::with_capacity(total_len);
    output.extend_from_slice(pdf_data);
    output.extend_from_slice(patched.as_bytes());
    output.extend_from_slice(xref_section.as_bytes());
    output.extend_from_slice(trailer_section.as_bytes());
    output.extend_from_slice(startxref_section.as_bytes());
    debug_assert_eq!(output.len(), total_len, "B-LTA assembled length mismatch");

    let signed_bytes =
        super::byterange::ByteRangeCalculator::extract_signed_bytes(&output, &byte_range)?;
    let token = timestamper(&signed_bytes)?;
    let token_hex: String = token.iter().map(|b| format!("{b:02X}")).collect();
    calc.insert_signature(&mut output, contents_abs, &token_hex)?;
    Ok(output)
}

/// Patch the `0000000000 0000000000 0000000000 0000000000` placeholder in the
/// signature dict text with the actual ByteRange values. The output is always
/// the same length as the input because each field is right-justified in a
/// `BR_FIELD_W`-wide space (trailing spaces absorb the freed digits).
fn patch_byterange(mut text: String, br: &[i64; 4]) -> String {
    // Re-format each number right-justified in BR_FIELD_W characters.
    // The placeholder "0000000000" is replaced by e.g. "         0" or "1234567890".
    let replacement = format!(
        "{:>BR_FIELD_W$} {:>BR_FIELD_W$} {:>BR_FIELD_W$} {:>BR_FIELD_W$}",
        br[0], br[1], br[2], br[3],
    );
    assert_eq!(
        replacement.len(),
        BR_PLACEHOLDER.len(),
        "replacement must have the same length as the placeholder"
    );
    if let Some(pos) = text.find(BR_PLACEHOLDER) {
        text.replace_range(pos..pos + BR_PLACEHOLDER.len(), &replacement);
    }
    text
}

/// Find the byte offset of the `<` that opens `/Contents <...>` within `data`.
/// Matches the first `<` that follows a `/Contents` keyword (skipping optional
/// whitespace).
fn find_contents_offset_in_text(data: &[u8]) -> Option<usize> {
    let pattern = b"/Contents ";
    let pos = data.windows(pattern.len()).position(|w| w == pattern)?;
    let after = pos + pattern.len();
    // Skip additional whitespace before '<'
    for (i, &b) in data[after..].iter().enumerate() {
        if b == b'<' {
            return Some(after + i);
        }
        if b != b' ' && b != b'\t' && b != b'\r' && b != b'\n' {
            break;
        }
    }
    None
}

// ─── PDF metadata scanners ───────────────────────────────────────────────────

/// Find the last `startxref` offset value in the file (scans the last 4 KB).
fn scan_startxref(data: &[u8]) -> Option<u64> {
    let window = &data[data.len().saturating_sub(4096)..];
    // rfind so we pick up the LAST startxref (most-recent incremental update)
    let pos = window.windows(9).rposition(|w| w == b"startxref")?;
    let after = &window[pos + 9..];
    let s = std::str::from_utf8(after).ok()?;
    let trimmed = s.trim_start_matches([' ', '\r', '\n']);
    let end = trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(trimmed.len());
    trimmed[..end].parse().ok()
}

/// Find the last `/Root X Y R` reference string in the file (scans the last
/// 4 KB only, same as `scan_startxref`, to avoid false-positive matches in
/// uncompressed content streams or metadata that contain the literal `/Root`).
fn scan_root_ref(data: &[u8]) -> Option<String> {
    let window = &data[data.len().saturating_sub(4096)..];
    let pattern = b"/Root ";
    let pos = window.windows(pattern.len()).rposition(|w| w == pattern)?;
    let after = &window[pos + pattern.len()..];
    // Collect up to 40 bytes as ASCII; stop at '/' or '>>'
    let end = after
        .iter()
        .position(|&b| b == b'/' || b == b'>' || b == b'\n')
        .unwrap_or(after.len().min(40));
    let s = std::str::from_utf8(&after[..end]).ok()?.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Find the highest object number in the latest trailer's `/Size` entry.
/// Returns the number itself — the next available object ID is this value.
fn scan_next_obj_num(data: &[u8]) -> Option<u64> {
    let window = &data[data.len().saturating_sub(4096)..];
    // Find LAST /Size entry in the tail (covers incremental updates)
    let pattern = b"/Size ";
    let pos = window.windows(pattern.len()).rposition(|w| w == pattern)?;
    let after = &window[pos + pattern.len()..];
    let s = std::str::from_utf8(after).ok()?;
    let trimmed = s.trim_start_matches(' ');
    let end = trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(trimmed.len());
    trimmed[..end].parse().ok()
}

// ─── PDF string helper ───────────────────────────────────────────────────────

/// Encode `s` as a PDF hex string `<AABB...>` using the same
/// PDFDocEncoding / UTF-16BE-with-BOM logic as the rest of the library
/// (`encode_pdf_text_string`).  Hex syntax requires no further escaping and
/// handles arbitrary byte sequences safely.
fn pdf_text_hex(s: &str) -> String {
    let bytes = encode_pdf_text_string(s);
    let mut out = String::with_capacity(bytes.len() * 2 + 2);
    out.push('<');
    for b in &bytes {
        out.push_str(&format!("{:02X}", b));
    }
    out.push('>');
    out
}

fn format_pdf_date() -> String {
    // Delegates to the single leap-year-correct implementation. The
    // prior local copy hard-coded month/day to "0101" and approximated
    // the year as 1970 + days/365 — corrupting every signature /M date
    // (README latent bug). WASM note: SystemTime::now() in the shared
    // helper still needs cfg-gating if signatures are ever enabled for
    // wasm32 (currently masked — `signatures` is off in the wasm build).
    super::pdf_date::format_pdf_date_utc()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// Parse the total byte length of a DER SEQUENCE from its hex encoding.
/// Handles the definite long form (0x82 two-byte length) used by CMS blobs.
#[cfg(test)]
fn der_sequence_len_from_hex(hex: &str) -> usize {
    let lb = u8::from_str_radix(&hex[2..4], 16).expect("DER len byte");
    if lb < 0x80 {
        (lb as usize) + 2
    } else {
        let n = (lb & 0x7f) as usize;
        let mut len = 0usize;
        for i in 0..n {
            let b = u8::from_str_radix(&hex[(4 + i * 2)..(6 + i * 2)], 16).expect("DER len");
            len = (len << 8) | (b as usize);
        }
        len + 2 + n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signatures::cms_verify::SignerVerify;
    use crate::signatures::verify_signer_detached;
    use crate::signatures::ByteRangeCalculator;

    fn load_test_creds() -> SigningCredentials {
        let cert =
            std::fs::read_to_string("tests/fixtures/test_signing_cert.pem").expect("cert fixture");
        let key =
            std::fs::read_to_string("tests/fixtures/test_signing_key.pem").expect("key fixture");
        SigningCredentials::from_pem(&cert, &key).expect("creds load")
    }

    fn minimal_pdf() -> Vec<u8> {
        // A valid single-page PDF with AcroForm stripped to the bare minimum
        // so the scanner tests have something real to work with.
        b"%PDF-1.4\n\
          1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
          2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
          3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
          xref\n0 4\n0000000000 65535 f \r\n0000000009 00000 n \r\n\
          0000000058 00000 n \r\n0000000115 00000 n \r\n\
          trailer\n<< /Size 4 /Root 1 0 R >>\n\
          startxref\n187\n%%EOF\n"
            .to_vec()
    }

    #[test]
    fn test_scan_startxref() {
        let pdf = minimal_pdf();
        let xref = scan_startxref(&pdf).expect("must find startxref");
        assert!(xref > 0, "startxref must be positive");
    }

    #[test]
    fn test_scan_root_ref() {
        let pdf = minimal_pdf();
        let root = scan_root_ref(&pdf).expect("must find root");
        assert!(root.contains("1 0 R"), "root ref must be '1 0 R': got {root}");
    }

    #[test]
    fn test_scan_next_obj_num() {
        let pdf = minimal_pdf();
        let n = scan_next_obj_num(&pdf).expect("must find /Size");
        assert_eq!(n, 4, "/Size must be 4 for this minimal PDF");
    }

    #[test]
    fn test_patch_byterange_same_length() {
        let text = format!("pre {} post", BR_PLACEHOLDER);
        let original_len = text.len();
        let br = [0i64, 12345, 99999, 200];
        let patched = patch_byterange(text, &br);
        assert_eq!(patched.len(), original_len, "patch must not change text length");
        assert!(!patched.contains(BR_PLACEHOLDER), "placeholder must be replaced");
    }

    fn hex_decode(s: &str) -> Vec<u8> {
        s.as_bytes()
            .chunks(2)
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn test_sign_pdf_bytes_roundtrip() {
        let pdf = minimal_pdf();
        let creds = load_test_creds();
        let opts = SignOptions {
            estimated_size: 4096,
            ..Default::default()
        };

        let signed = sign_pdf_bytes(&pdf, &creds, opts).expect("sign_pdf_bytes must succeed");

        // ── Parse the appended incremental update ─────────────────────
        // The incremental update is appended after the original PDF bytes.
        let tail = &signed[pdf.len()..];
        let tail_str = std::str::from_utf8(tail).unwrap();

        // Parse /ByteRange [...] from the sig dict text
        let br_pos = tail_str
            .find("/ByteRange [")
            .expect("/ByteRange must exist");
        let after_br = &tail_str[br_pos + 12..];
        let end = after_br.find(']').expect("] must follow /ByteRange");
        let nums: Vec<i64> = after_br[..end]
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(nums.len(), 4);
        let byte_range: [i64; 4] = [nums[0], nums[1], nums[2], nums[3]];

        // Validate ByteRange is sane
        assert_eq!(byte_range[0], 0);
        assert!(byte_range[1] > 0);
        assert!(byte_range[2] > byte_range[1]);
        assert!(byte_range[3] > 0);
        assert_eq!(
            byte_range[2] + byte_range[3],
            signed.len() as i64,
            "ByteRange must cover the whole file"
        );

        // ── Extract /Contents hex and decode the CMS blob ─────────────
        let ct_pos = tail_str.find("/Contents <").expect("/Contents must exist");
        let after_ct = &tail_str[ct_pos + 11..]; // skip "/Contents <"
        let close = after_ct.find('>').expect("> must follow /Contents <");
        let hex_str = &after_ct[..close];
        // Use the DER length field to find the exact CMS byte count rather than
        // trimming trailing '0' characters — a CMS whose last real byte is 0x00
        // would be silently truncated by the naive trim approach.
        let cms_len = der_sequence_len_from_hex(hex_str);
        let cms_blob = hex_decode(&hex_str[..cms_len * 2]);

        // ── Extract the signed bytes and verify ───────────────────────
        let signed_content = ByteRangeCalculator::extract_signed_bytes(&signed, &byte_range)
            .expect("extract_signed_bytes must succeed");

        let result =
            verify_signer_detached(&cms_blob, &signed_content).expect("verify must not error");
        assert_eq!(result, SignerVerify::Valid, "end-to-end PDF signature must verify as Valid");
    }

    // ── Finding 3 regression: scan_root_ref must ignore /Root in body ────────

    /// Parse + verify the appended signature exactly as a reader would
    /// (mirrors `test_sign_pdf_bytes_roundtrip`'s extraction). Returns
    /// `(verification, decoded_cms, contents_string)`:
    /// - `decoded_cms` — the CMS trimmed to its DER length, for
    ///   inspecting signed/unsigned attributes (the OID bytes are
    ///   hex-encoded in `/Contents`, never raw in the file).
    /// - `contents_string` — the **full** `/Contents` value incl. the
    ///   zero-padding (every hex pair decoded), i.e. byte-identical to
    ///   what a PDF parser stores in `SignatureInfo::contents` and what
    ///   `classify_pades_level` / `vri_key` hash. Lets the VRI key be
    ///   checked for write/read parity without an AcroForm.
    fn verify_appended_signature(
        orig_len: usize,
        signed: &[u8],
    ) -> (SignerVerify, Vec<u8>, Vec<u8>) {
        // Byte-oriented scan: a B-LT file appends *binary* DSS streams
        // after the signature, so the tail is not valid UTF-8 — only
        // the small ASCII `/ByteRange [...]` and `/Contents <...>` runs
        // are. `windows().position()` finds the first match, i.e. the
        // signature appended at `orig_len` (before any DSS increment).
        let tail = &signed[orig_len..];
        let br = tail
            .windows(12)
            .position(|w| w == b"/ByteRange [")
            .expect("/ByteRange");
        let after = &tail[br + 12..];
        let end = after.iter().position(|&b| b == b']').unwrap();
        let n: Vec<i64> = std::str::from_utf8(&after[..end])
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let byte_range = [n[0], n[1], n[2], n[3]];
        let ct = tail
            .windows(11)
            .position(|w| w == b"/Contents <")
            .expect("/Contents");
        let after_ct = &tail[ct + 11..];
        let close = after_ct.iter().position(|&b| b == b'>').unwrap();
        let hex_str = std::str::from_utf8(&after_ct[..close]).unwrap();
        let cms_len = der_sequence_len_from_hex(hex_str);
        let cms = hex_decode(&hex_str[..cms_len * 2]);
        let contents_string = hex_decode(hex_str);
        let content = ByteRangeCalculator::extract_signed_bytes(signed, &byte_range).unwrap();
        let v = verify_signer_detached(&cms, &content).expect("verify must not error");
        (v, cms, contents_string)
    }

    #[test]
    #[cfg(feature = "signatures")]
    fn test_sign_pdf_bytes_pades_levels() {
        use crate::signatures::pades::vri_key;
        use crate::signatures::{
            classify_pades_level, read_dss, sign_pdf_bytes_pades, PadesLevel, RevocationMaterial,
            SignatureInfo,
        };

        // Build a `SignatureInfo` carrying just the parsed `/Contents`
        // (all `classify_pades_level` needs) — AcroForm-independent, so
        // it works on the bare `/Type /Sig` object `sign_pdf_bytes`
        // emits (which `enumerate_signatures` deliberately won't surface).
        let info_with = |contents: Vec<u8>| SignatureInfo {
            contents: Some(contents),
            ..Default::default()
        };

        let pdf = minimal_pdf();
        let creds = load_test_creds();
        let mk_opts = || SignOptions {
            estimated_size: 4096,
            ..Default::default()
        };
        let ts: &dyn Fn(&[u8]) -> Result<Vec<u8>> =
            &|_sig| Ok(vec![0x30, 0x07, 0x02, 0x01, 0x01, 0x04, 0x02, b't', b's']);

        // B-B: signs, verifies, and carries the ESS attribute.
        let bb = sign_pdf_bytes_pades(
            &pdf,
            &creds,
            mk_opts(),
            PadesLevel::BB,
            None,
            &RevocationMaterial::default(),
        )
        .expect("B-B sign");
        let (v_bb, cms_bb, contents_bb) = verify_appended_signature(pdf.len(), &bb);
        assert_eq!(v_bb, SignerVerify::Valid);
        // id-aa-signingCertificateV2 = 1.2.840.113549.1.9.16.2.47.
        const ESS_OID: &[u8] = &[
            0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x2F,
        ];
        // The ESS OID lives in the *decoded* CMS (hex-encoded in the PDF).
        assert!(
            cms_bb.windows(ESS_OID.len()).any(|w| w == ESS_OID),
            "B-B CMS carries the ESS signing-certificate-v2 attribute"
        );
        // No timestamp attr ⇒ classifies as plain B-B.
        assert_eq!(classify_pades_level(&info_with(contents_bb), None), PadesLevel::BB);

        // B-T without a timestamper → fail-closed Unsupported.
        assert!(matches!(
            sign_pdf_bytes_pades(
                &pdf,
                &creds,
                mk_opts(),
                PadesLevel::BT,
                None,
                &RevocationMaterial::default()
            ),
            Err(Error::Unsupported(_))
        ));

        // B-T: signs with the timestamp attr, still verifies, classifies BT.
        let bt = sign_pdf_bytes_pades(
            &pdf,
            &creds,
            mk_opts(),
            PadesLevel::BT,
            Some(ts),
            &RevocationMaterial::default(),
        )
        .expect("B-T sign");
        let (v_bt, cms_bt, contents_bt) = verify_appended_signature(pdf.len(), &bt);
        assert_eq!(v_bt, SignerVerify::Valid);
        // id-aa-signatureTimeStampToken = 1.2.840.113549.1.9.16.2.14 —
        // the B-T unsigned attr must be spliced into the SignerInfo.
        const TS_OID: &[u8] = &[
            0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x0E,
        ];
        assert!(
            cms_bt.windows(TS_OID.len()).any(|w| w == TS_OID),
            "B-T CMS carries the signature-time-stamp unsigned attribute"
        );
        // The timestamp attr present (no DSS) ⇒ classifies as B-T.
        assert_eq!(classify_pades_level(&info_with(contents_bt), None), PadesLevel::BT);

        // B-LT: B-T + a DSS appended as a 2nd incremental update.
        let material = RevocationMaterial {
            certificates: vec![creds.certificate.clone()],
            ..RevocationMaterial::default()
        };
        let blt =
            sign_pdf_bytes_pades(&pdf, &creds, mk_opts(), PadesLevel::BLt, Some(ts), &material)
                .expect("B-LT sign");
        // The DSS is an append-only 2nd incremental update (I2): the
        // B-T signature's own /ByteRange self-delimits the bytes it
        // signed (the DSS append falls outside it), so the signature
        // still verifies within the full B-LT file — I1. (Slicing to
        // `bt.len()` would be wrong: `bt` and `blt` are separate sign
        // calls whose `/M` signing time differs byte-for-byte.)
        assert!(blt.len() > bt.len());
        let (v_blt, _cms_blt, contents_blt) = verify_appended_signature(pdf.len(), &blt);
        assert_eq!(
            v_blt,
            SignerVerify::Valid,
            "I1: the B-T signature still verifies in the full B-LT file"
        );
        let doc_blt = crate::document::PdfDocument::from_bytes(blt).unwrap();
        let dss = read_dss(&doc_blt)
            .expect("read_dss ok")
            .expect("DSS present after B-LT");
        assert_eq!(dss.certificates, vec![creds.certificate.clone()]);
        // Write/read VRI-key parity: the DSS must carry a /VRI entry
        // keyed by SHA-1 of the *exact* /Contents bytes a reader parses
        // — proving the write-side key (computed in `sign_pdf_bytes_pades`
        // over the padded contents) equals the read-side key.
        let key = vri_key(&contents_blt).expect("provider supports SHA-1");
        assert!(
            dss.vri_for(&key).is_some(),
            "DSS /VRI is keyed by SHA-1(/Contents) — write/read parity"
        );
        // …and the public read API agrees: timestamp attr + matching
        // VRI ⇒ B-LT.
        assert_eq!(
            classify_pades_level(&info_with(contents_blt), Some(&dss)),
            PadesLevel::BLt,
            "signature classifies as B-LT with the DSS+VRI present"
        );

        // B-LTA: B-LT + a /DocTimeStamp (ETSI.RFC3161) over the whole
        // file *including* the DSS, as a 3rd incremental update.
        let blta =
            sign_pdf_bytes_pades(&pdf, &creds, mk_opts(), PadesLevel::BLta, Some(ts), &material)
                .expect("B-LTA sign");
        // The B-T signature still verifies — its ByteRange self-delimits;
        // the DSS and DocTimeStamp appends fall outside it.
        let (v_blta, _, _) = verify_appended_signature(pdf.len(), &blta);
        assert_eq!(v_blta, SignerVerify::Valid, "B-LTA: original sig still valid");
        // The archival document timestamp object is present…
        assert!(
            crate::signatures::has_document_timestamp(&blta),
            "B-LTA carries a /DocTimeStamp ETSI.RFC3161 object"
        );
        // …and it is appended *after* the DSS (so it covers it).
        let dts_pos = blta
            .windows(13)
            .position(|w| w == b"/DocTimeStamp")
            .expect("/DocTimeStamp present");
        let dss_pos = blta
            .windows(4)
            .position(|w| w == b"/DSS")
            .expect("/DSS present");
        assert!(dss_pos < dts_pos, "DocTimeStamp is appended after the DSS");

        // Fail-closed: B-LTA without a timestamper is Unsupported.
        assert!(matches!(
            sign_pdf_bytes_pades(&pdf, &creds, mk_opts(), PadesLevel::BLta, None, &material),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn test_scan_root_ref_ignores_body_occurrence() {
        // Embed a fake "/Root " string deep in the body far from the trailer.
        // The scanner must still return the real trailer reference.
        let mut pdf = minimal_pdf();
        // Prepend >4 KB of content containing a misleading /Root occurrence.
        let filler = b"% /Root 99 0 R this is inside a comment not a trailer\n";
        let padding = filler.repeat(100); // ~5.4 KB
        let mut data = padding;
        data.extend_from_slice(&pdf);
        // The real /Root is in the last 4 KB (the trailer of minimal_pdf is tiny).
        let root = scan_root_ref(&data).expect("must find root in last 4 KB");
        assert!(
            root.contains("1 0 R"),
            "must return trailer /Root, not body occurrence; got: {root}"
        );
        // Confirm that there really IS a misleading /Root earlier in the data.
        let first = data.windows(b"/Root ".len()).position(|w| w == b"/Root ");
        assert!(first.unwrap() < data.len() - 4096, "misleading /Root is before the 4 KB window");

        // Drop pdf from outer scope warning
        let _ = pdf.drain(..);
    }

    // ── Finding 7 regression: non-ASCII metadata must not be raw UTF-8 ───────

    #[test]
    fn test_pdf_text_hex_ascii_roundtrip() {
        // ASCII stays as PDFDocEncoding bytes (no BOM).
        let h = pdf_text_hex("Hello");
        assert!(h.starts_with('<') && h.ends_with('>'));
        let bytes = hex_decode(&h[1..h.len() - 1]);
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_pdf_text_hex_latin1_no_bom() {
        // "é" is U+00E9 — within PDFDocEncoding range → single byte 0xE9, no BOM.
        let h = pdf_text_hex("é");
        let bytes = hex_decode(&h[1..h.len() - 1]);
        assert_eq!(bytes, &[0xE9], "PDFDocEncoding for é must be 0xE9, not multi-byte UTF-8");
    }

    #[test]
    fn test_pdf_text_hex_portuguese_reason() {
        // Regression guard: "Aprovado Lógico" — contains ó (U+00F3).
        // Must NOT emit the raw UTF-8 bytes 0xC3 0xB3 for ó.
        let h = pdf_text_hex("Aprovado Lógico");
        let bytes = hex_decode(&h[1..h.len() - 1]);
        // PDFDocEncoding: ó → 0xF3 (single byte), not 0xC3 0xB3 (UTF-8).
        assert!(
            !bytes.windows(2).any(|w| w == [0xC3, 0xB3]),
            "raw UTF-8 bytes for ó must not appear; got {:X?}",
            bytes
        );
        // ó must appear as its PDFDocEncoding byte 0xF3.
        assert!(bytes.contains(&0xF3), "PDFDocEncoding 0xF3 for ó must be present");
    }

    #[test]
    fn test_pdf_text_hex_cjk_uses_utf16be_bom() {
        // CJK characters trigger UTF-16BE with leading BOM 0xFE 0xFF.
        let h = pdf_text_hex("中文");
        let bytes = hex_decode(&h[1..h.len() - 1]);
        assert_eq!(&bytes[..2], &[0xFE, 0xFF], "UTF-16BE BOM must be present for CJK");
    }

    #[test]
    fn test_sign_metadata_non_ascii_encoded_in_sig_dict() {
        // End-to-end: signature dict text must contain hex-encoded metadata,
        // never raw multi-byte UTF-8 sequences for non-ASCII characters.
        let pdf = minimal_pdf();
        let creds = load_test_creds();
        let opts = SignOptions {
            reason: Some("Aprovado Lógico".to_string()), // ó = U+00F3
            location: Some("São Paulo".to_string()),     // ã = U+00E3, ~ ã
            name: Some("中文签名人".to_string()),
            estimated_size: 8192,
            ..Default::default()
        };

        let signed = sign_pdf_bytes(&pdf, &creds, opts).expect("sign must succeed");
        let tail = &signed[pdf.len()..];

        // The sig dict is written as UTF-8 text around hex strings, so we can
        // search the raw bytes for /Reason <...> etc.
        let tail_str = std::str::from_utf8(tail).unwrap();

        // Must contain hex string syntax (angle brackets) for /Reason.
        let reason_hex = tail_str.contains("/Reason <");
        let location_hex = tail_str.contains("/Location <");
        let name_hex = tail_str.contains("/Name <");
        assert!(reason_hex, "/Reason must use hex string syntax");
        assert!(location_hex, "/Location must use hex string syntax");
        assert!(name_hex, "/Name must use hex string syntax");

        // Raw UTF-8 encoding of ó is 0xC3 0xB3 — must NOT appear in the dict.
        let c3b3 = tail.windows(2).any(|w| w == [0xC3, 0xB3]);
        assert!(!c3b3, "raw UTF-8 bytes for ó must not appear in signed output");
    }
}
