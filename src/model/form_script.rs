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

/// `Scripts/DefaultJScript` 가 deflate 로 저장돼 있는지.
///
/// HWP5 원본은 FileHeader 압축 비트가 0 이면 이 스트림도 압축하지 않고 저장한다.
/// 다른 형식에서 합성한 스트림(HWPX 파서의 contract stream 등)은 늘 압축한다.
pub(crate) fn streams_are_compressed(document: &Document) -> bool {
    document.header.compressed
        || document.provenance.format != crate::model::provenance::SourceFormat::Hwp5
}

fn is_distribution_wrapped(bytes: &[u8]) -> bool {
    bytes.starts_with(
        &(u32::from(crate::parser::tags::HWPTAG_DISTRIBUTE_DOC_DATA) | (256 << 20)).to_le_bytes(),
    )
}

/// HWP 로 저장할 `Scripts/*` 스트림을 출력 FileHeader 의 압축 상태로 다시 인코딩한다.
///
/// HWP5 는 Scripts 저장소의 스트림도 문서 압축 비트를 따른다. 메모리의 스트림 상태
/// ([`streams_are_compressed`])와 출력 압축 비트가 다르면 deflate 를 풀거나 다시 건다.
/// 배포용 래핑, 다른 경로, 해제에 실패한 바이트는 `None` 으로 원본을 그대로 둔다.
pub(crate) fn recode_script_stream(path: &str, bytes: &[u8], compressed: bool) -> Option<Vec<u8>> {
    use std::io::{Read, Write};
    if !path.trim_start_matches('/').starts_with("Scripts/")
        || bytes.len() > MAX_SCRIPT_BYTES
        || is_distribution_wrapped(bytes)
    {
        return None;
    }
    if compressed {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(bytes).ok()?;
        encoder.finish().ok()
    } else {
        let mut raw = Vec::new();
        flate2::read::DeflateDecoder::new(bytes)
            .take((MAX_SCRIPT_BYTES + 1) as u64)
            .read_to_end(&mut raw)
            .ok()?;
        (raw.len() <= MAX_SCRIPT_BYTES).then_some(raw)
    }
}

/// `compressed` 는 [`streams_are_compressed`] 의 판정이다.
pub(crate) fn script_sources_from_streams(
    streams: &[(String, Vec<u8>)],
    compressed: bool,
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
    let raw = if is_distribution_wrapped(bytes) {
        crate::parser::crypto::decrypt_viewtext_section_limited(bytes, true, MAX_SCRIPT_BYTES)
            .map_err(|_| invalid())?
    } else if compressed {
        let mut raw = Vec::new();
        flate2::read::DeflateDecoder::new(bytes.as_slice())
            .take((MAX_SCRIPT_BYTES + 1) as u64)
            .read_to_end(&mut raw)
            .map_err(|_| invalid())?;
        raw
    } else {
        bytes.clone()
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
        _ => script_sources_from_streams(&document.extra_streams, streams_are_compressed(document)),
    }
}
