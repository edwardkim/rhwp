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

/// [#8374] `extra_streams` 의 `Scripts/DefaultJScript` 가 deflate 형태인가.
///
/// HWP5 파서는 스트림을 파일에 저장된 그대로 담으므로 FileHeader 압축 비트가 권위다 — 비트가
/// 꺼진 문서는 스크립트도 압축하지 않고 저장한다. HWPX 파서 등 다른 생산자는 이 스트림을
/// HWP5 압축형으로 합성하므로(`parser::hwpx::contract_streams`, 문서 헤더는 비압축) 헤더
/// 비트와 무관하게 압축형이다.
pub(crate) fn script_stream_compressed(document: &Document) -> bool {
    document.header.compressed
        || document.provenance.format != crate::model::provenance::SourceFormat::Hwp5
}

/// 배포용 문서는 Scripts 도 DISTRIBUTE_DOC_DATA 봉투로 싼다(HWP5 4.2.13).
fn is_distribution_envelope(bytes: &[u8]) -> bool {
    bytes.starts_with(
        &(u32::from(crate::parser::tags::HWPTAG_DISTRIBUTE_DOC_DATA) | (256 << 20)).to_le_bytes(),
    )
}

/// 압축형 `DefaultJScript` 를 원문으로 푼다. 해제 결과도 스크립트 상한으로 묶는다.
fn inflate_script_stream(bytes: &[u8]) -> Result<Vec<u8>, HwpError> {
    use std::io::Read;
    let mut raw = Vec::new();
    flate2::read::DeflateDecoder::new(bytes)
        .take((MAX_SCRIPT_BYTES + 1) as u64)
        .read_to_end(&mut raw)
        .map_err(|_| invalid())?;
    Ok(raw)
}

/// [#8374] HWP5 로 저장할 때 `DefaultJScript` 의 형태를 대상 파일의 압축 비트에 맞춘다.
///
/// 원본 형태(`script_stream_compressed`)와 대상 비트가 같거나 배포용 봉투면 `None`(그대로 쓴다).
/// 다르면 풀거나 압축한 바이트를 돌려준다 — HWPX 출처 문서를 비압축 HWP 로 쓰면 합성 압축형
/// 스크립트가 비압축 파일 안에 그대로 실려 다시 읽을 때 원문으로 해석된다. 풀 수 없는 원본은
/// 바이트를 보존하도록 `None` 이다.
pub(crate) fn script_stream_for_hwp(
    document: &Document,
    bytes: &[u8],
    target_compressed: bool,
) -> Option<Vec<u8>> {
    if bytes.is_empty()
        || script_stream_compressed(document) == target_compressed
        || is_distribution_envelope(bytes)
    {
        return None;
    }
    if target_compressed {
        use std::io::Write;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(bytes).ok()?;
        encoder.finish().ok()
    } else {
        inflate_script_stream(bytes).ok()
    }
}

pub(crate) fn script_sources_from_streams(
    document: &Document,
) -> Result<(String, String), HwpError> {
    let compressed = script_stream_compressed(document);
    let Some((_, bytes)) = document
        .extra_streams
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
    let raw = if is_distribution_envelope(bytes) {
        crate::parser::crypto::decrypt_viewtext_section_limited(bytes, compressed, MAX_SCRIPT_BYTES)
            .map_err(|_| invalid())?
    } else if !compressed {
        bytes.clone()
    } else {
        inflate_script_stream(bytes)?
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
        _ => script_sources_from_streams(document),
    }
}
