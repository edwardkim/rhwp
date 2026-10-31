//! Bounded HWP counted UTF-16 script sources shared by execution and cross-format preservation.
use crate::error::HwpError;
use crate::model::document::Document;
const MAX_SCRIPT_BYTES: usize = 1_048_576;
fn invalid() -> HwpError {
    HwpError::InvalidField("FORM_ACTION_INVALID: invalid or excessive form action".into())
}

fn utf16(bytes: &[u8]) -> Result<String, HwpError> {
    if bytes.len() > MAX_SCRIPT_BYTES || !bytes.len().is_multiple_of(2) {
        return Err(invalid());
    }
    String::from_utf16(
        &bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<_>>(),
    )
    .map(|s| s.trim_start_matches('\u{feff}').to_owned())
    .map_err(|_| invalid())
}

pub(crate) fn script_sources_from_streams(
    streams: &[(String, Vec<u8>)],
) -> Result<(String, String), HwpError> {
    use std::io::Read;
    let Some((_, bytes)) = streams
        .iter()
        .find(|(path, _)| path.trim_start_matches('/') == "Scripts/DefaultJScript")
    else {
        return Ok((String::new(), String::new()));
    };
    if bytes.len() > MAX_SCRIPT_BYTES {
        return Err(invalid());
    }
    // Distribution documents wrap Scripts too (HWP5 section 4.2.13). The
    // self-describing record remains after convert_to_editable clears the
    // document flag, so inspect the stream contract, not that mutable flag.
    // Bound the stored input above and the decrypted/decompressed result here.
    let distribution_header =
        (u32::from(crate::parser::tags::HWPTAG_DISTRIBUTE_DOC_DATA) | (256 << 20)).to_le_bytes();
    let raw = if bytes.starts_with(&distribution_header) {
        crate::parser::crypto::decrypt_viewtext_section_limited(bytes, true, MAX_SCRIPT_BYTES)
            .map_err(|_| invalid())?
    } else {
        let mut raw = Vec::new();
        flate2::read::DeflateDecoder::new(bytes.as_slice())
            .take((MAX_SCRIPT_BYTES + 1) as u64)
            .read_to_end(&mut raw)
            .map_err(|_| invalid())?;
        raw
    };
    if raw.len() > MAX_SCRIPT_BYTES {
        return Err(invalid());
    }
    let mut offset = 0usize;
    let mut part = || -> Result<String, HwpError> {
        let count_end = offset.checked_add(4).ok_or_else(invalid)?;
        let count = u32::from_le_bytes(
            raw.get(offset..count_end)
                .ok_or_else(invalid)?
                .try_into()
                .map_err(|_| invalid())?,
        ) as usize;
        offset = count_end;
        let end = offset
            .checked_add(count.checked_mul(2).ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        let decoded = utf16(raw.get(offset..end).ok_or_else(invalid)?)?;
        offset = end;
        Ok(decoded)
    };
    Ok((part()?, part()?))
}

pub(crate) fn scripts(document: &Document) -> Result<(String, String), HwpError> {
    match (
        document.hwpx_aux_entry("Scripts/headerScripts"),
        document.hwpx_aux_entry("Scripts/sourceScripts"),
    ) {
        (Some(header), Some(source)) => Ok((utf16(header)?, utf16(source)?)),
        _ => script_sources_from_streams(&document.extra_streams),
    }
}
