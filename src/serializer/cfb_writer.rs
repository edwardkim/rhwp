//! CFB 컨테이너 조립 + 스트림 압축
//!
//! `parser::cfb_reader`의 역방향으로, 직렬화된 스트림을 CFB 컨테이너로 조립한다.
//!
//! 구조:
//! - /FileHeader (256바이트, 비압축)
//! - /DocInfo (레코드 바이트, 조건부 deflate)
//! - /BodyText/Section{N} (레코드 바이트, 조건부 deflate)
//! - /BinData/BIN{XXXX}.{ext} (바이너리 데이터)

use std::io::Write;

use crate::model::bin_data::BinDataContent;
use crate::model::bin_data::{BinData, BinDataType, MAX_BIN_DATA_BYTES};
use crate::model::document::{Document, Preview};
use crate::password_crypto::{encrypt_hwp5_stream, HWP5_ENCRYPT_VERSION};

use super::body_text::serialize_section;
use super::content_loss::{
    ContentLoss, ContentLossReason, ContentLossReport, SerializedDocument, SerializedFormat,
};
use super::doc_info::serialize_doc_info;
use super::header::serialize_file_header;
use super::mini_cfb;
use super::SerializeError;

#[derive(Clone, Copy)]
enum ContentLossWarningMode {
    ReportOnly,
    LegacyStderr,
}

/// Document IR을 HWP 5.0 CFB 바이너리로 직렬화
pub fn serialize_hwp(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let serialized = serialize_hwp_inner(doc, None, ContentLossWarningMode::LegacyStderr)?;
    Ok(serialized.into_bytes())
}

/// HWP 직렬화 바이트와 바로 그 산출물의 내용 손실을 함께 반환한다 (#4430).
pub fn serialize_hwp_with_report(doc: &Document) -> Result<SerializedDocument, SerializeError> {
    serialize_hwp_inner(doc, None, ContentLossWarningMode::ReportOnly)
}

/// Document IR을 HWP5 EncryptVersion 4 비밀번호 문서로 직렬화한다.
///
/// 암호 적용 범위는 한글의 HWP5 contract에 맞춰 DocInfo, BodyText, BinData,
/// Scripts 및 고아 BinData stream이다. FileHeader와 PrvImage/PrvText는 평문이다.
pub fn serialize_hwp_with_password(
    doc: &Document,
    password: &[u8],
) -> Result<Vec<u8>, SerializeError> {
    let serialized =
        serialize_hwp_inner(doc, Some(password), ContentLossWarningMode::LegacyStderr)?;
    Ok(serialized.into_bytes())
}

/// 비밀번호 HWP 바이트와 바로 그 산출물의 내용 손실을 함께 반환한다 (#4430).
pub fn serialize_hwp_with_password_and_report(
    doc: &Document,
    password: &[u8],
) -> Result<SerializedDocument, SerializeError> {
    serialize_hwp_inner(doc, Some(password), ContentLossWarningMode::ReportOnly)
}

fn serialize_hwp_inner(
    doc: &Document,
    password: Option<&[u8]>,
    warning_mode: ContentLossWarningMode,
) -> Result<SerializedDocument, SerializeError> {
    // 1. FileHeader 직렬화
    // [Task #1768] 배포용/암호화 문서 강하: IR 은 이미 복호화된 평문이고 본 직렬화는
    // ViewText/DISTRIBUTE_DOC_DATA 를 생성하지 않으므로, 플래그를 유지하면 산출물
    // 재로드가 "암호 오류: DISTRIBUTE_DOC_DATA 레코드 없음" 으로 실패한다. 일반
    // 문서로 강하(배포용 0x04 · 암호화 0x02 클리어, raw_data 의 플래그와
    // EncryptVersion도 패치).
    let header_bytes = if let Some(_) = password {
        let mut header = doc.header.clone();
        header.distribution = false;
        header.encrypted = true;
        header.flags = (header.flags | 0x02) & !0x04;
        if let Some(raw) = header.raw_data.as_mut() {
            if raw.len() >= 48 {
                raw[36..40].copy_from_slice(&header.flags.to_le_bytes());
                raw[44..48].copy_from_slice(&HWP5_ENCRYPT_VERSION.to_le_bytes());
            }
        }
        let mut bytes = serialize_file_header(&header);
        bytes[36..40].copy_from_slice(&header.flags.to_le_bytes());
        bytes[44..48].copy_from_slice(&HWP5_ENCRYPT_VERSION.to_le_bytes());
        bytes
    } else if doc.header.distribution || doc.header.encrypted {
        let mut header = doc.header.clone();
        header.distribution = false;
        header.encrypted = false;
        header.flags &= !0x06;
        if let Some(raw) = header.raw_data.as_mut() {
            if raw.len() >= 40 {
                let flags = u32::from_le_bytes([raw[36], raw[37], raw[38], raw[39]]) & !0x06u32;
                raw[36..40].copy_from_slice(&flags.to_le_bytes());
            }
            if doc.header.encrypted && raw.len() >= 48 {
                raw[44..48].fill(0);
            }
        }
        serialize_file_header(&header)
    } else {
        serialize_file_header(&doc.header)
    };

    // 2. DocInfo 직렬화
    let doc_info_bytes = serialize_doc_info(&doc.doc_info, &doc.doc_properties);

    // 3. BodyText 섹션별 직렬화
    let mut section_bytes_list = Vec::new();
    for section in &doc.sections {
        let section_bytes = serialize_section(section);
        section_bytes_list.push(section_bytes);
    }

    // 4. 압축 여부 결정
    let compressed = doc.header.compressed;

    // 5. 미리보기 텍스트 보정
    //
    // preview 는 파싱 원본에서 그대로 실려 온다 (원본 보존 우선). 그러나 내장
    // 템플릿에서 만든 새 문서는 템플릿의 placeholder 미리보기("\r\n")를 물고
    // 나가 탐색기·한컴 미리보기에 빈 문서로 보인다. 원본 미리보기가 없거나
    // placeholder 일 때만 본문에서 새로 만든다 — 실문서의 원본 미리보기는 건드리지 않는다.
    let preview = supplement_preview(doc);

    // 6. CFB 컨테이너 조립
    let mut content_loss = ContentLossReport::new(SerializedFormat::Hwp);
    let bytes = write_hwp_cfb(
        &header_bytes,
        &doc_info_bytes,
        &section_bytes_list,
        &doc.doc_info.bin_data_list,
        &doc.bin_data_content,
        &preview,
        &doc.extra_streams,
        compressed,
        password,
        &mut content_loss,
    )?;
    if matches!(warning_mode, ContentLossWarningMode::LegacyStderr) {
        content_loss.write_warnings_to_stderr();
    }
    Ok(SerializedDocument::new(bytes, content_loss))
}

/// PrvText 가 비었거나 placeholder 면 본문 텍스트로 채운다.
///
/// 원본 미리보기가 실재하면 그대로 둔다 (라운드트립 보존).
fn supplement_preview(doc: &Document) -> Option<Preview> {
    let has_real_text = doc
        .preview
        .as_ref()
        .and_then(|p| p.text.as_ref())
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);

    if has_real_text {
        return doc.preview.clone();
    }

    let text = build_preview_text(doc);
    if text.trim().is_empty() {
        return doc.preview.clone();
    }

    Some(Preview {
        image: doc.preview.as_ref().and_then(|p| p.image.clone()),
        text: Some(text),
    })
}

/// 본문 문단에서 미리보기 텍스트를 만든다.
///
/// 한컴은 앞부분 일부만 담는다 (shortcut.hwp 실측 2044B ≈ 1022자).
/// 표/글상자 안 텍스트는 제외하고 본문 문단만 이어 붙인다.
fn build_preview_text(doc: &Document) -> String {
    const MAX_CHARS: usize = 1000;

    let mut out = String::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            let line = para.text.trim_end_matches('\u{0}');
            if line.is_empty() {
                continue;
            }
            out.push_str(line);
            out.push_str("\r\n");
            if out.chars().count() >= MAX_CHARS {
                return out.chars().take(MAX_CHARS).collect();
            }
        }
    }
    out
}

/// raw deflate 압축 (wbits=-15)
fn compress_stream(data: &[u8]) -> Result<Vec<u8>, SerializeError> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| SerializeError::CompressError(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| SerializeError::CompressError(e.to_string()))
}

/// CFB 컨테이너를 인메모리로 생성하여 바이트 배열 반환
fn write_hwp_cfb(
    header_bytes: &[u8],
    doc_info_bytes: &[u8],
    section_bytes_list: &[Vec<u8>],
    bin_data_list: &[BinData],
    bin_data_content: &[BinDataContent],
    preview: &Option<Preview>,
    extra_streams: &[(String, Vec<u8>)],
    compressed: bool,
    password: Option<&[u8]>,
    content_loss: &mut ContentLossReport,
) -> Result<Vec<u8>, SerializeError> {
    // 스트림 목록 수집
    let mut streams: Vec<(String, Vec<u8>)> = Vec::new();

    // 1. /FileHeader (항상 비압축)
    streams.push(("/FileHeader".to_string(), header_bytes.to_vec()));

    // 2. /DocInfo (조건부 압축)
    let doc_info_data = if compressed {
        compress_stream(doc_info_bytes)?
    } else {
        doc_info_bytes.to_vec()
    };
    streams.push((
        "/DocInfo".to_string(),
        encrypt_if_password(doc_info_data, password),
    ));

    // 3. /BodyText/Section{N} (조건부 압축)
    for (i, section_bytes) in section_bytes_list.iter().enumerate() {
        let path = format!("/BodyText/Section{}", i);
        let data = if compressed {
            compress_stream(section_bytes)?
        } else {
            section_bytes.clone()
        };
        streams.push((path, encrypt_if_password(data, password)));
    }

    // 4. /BinData/BIN{XXXX}.{ext}
    // BinData는 개별 압축 속성에 따라 재압축
    const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    for content in bin_data_content {
        let (storage_id, ext, mut should_compress) =
            find_bin_data_info_with_compress(bin_data_list, content, compressed);
        if password.is_some() {
            should_compress = compressed;
        }
        let storage_name = format!("BIN{:04X}.{}", storage_id, ext);
        let path = format!("/BinData/{}", storage_name);

        // [#2550] 압축 해제 상한. 초과 항목(deflate bomb 포함)은 해제 없이 원본
        // 저장 바이트를 그대로 통과시킨다 — 재압축·OLE prefix 복원·매직 판정은
        // 해제된 바이트에만 의미가 있고, 원본 스트림은 이미 그 처리가 끝난
        // 형태이므로 전부 건너뛴다. 정상 대용량 개체는 무손실, 폭탄은 애초에
        // 해제하지 않으므로 OOM 이 없다.
        let bytes = match content.data.load_limited(MAX_BIN_DATA_BYTES) {
            Some(bytes) => bytes,
            None => match content.data.load_raw() {
                // 저장 형태를 그대로 쓰려면 그 압축 상태가 이 문서에서 기대되는
                // 상태와 같아야 한다. 다르면(암호 저장의 압축 강제 등) 읽는 쪽이
                // 압축 바이트를 원본으로 오해해 조용히 깨지므로 통과시키지 않는다.
                Some(stored) if stored.compressed == should_compress => {
                    streams.push((path, encrypt_if_password(stored.bytes, password)));
                    continue;
                }
                Some(_) => {
                    // 상한 초과 + 압축 상태 불일치 — 해제(OOM 위험)도, 그대로 쓰기
                    // (오독)도 안 된다. 렌더·클립보드와 같은 placeholder 로 접는다.
                    content_loss.record(ContentLoss::binary_content_emptied(
                        storage_id,
                        path.clone(),
                        ContentLossReason::StoredCompressionMismatch,
                    ));
                    streams.push((path, encrypt_if_password(Vec::new(), password)));
                    continue;
                }
                // HWPX ZIP·인메모리 항목처럼 원본 HWP5 저장 형태가 없는 경우에는
                // 안전한 raw passthrough가 불가능하다. 여기서 `load()`로 되돌아가면
                // HWPX deflate bomb가 다시 무제한 materialize되므로 placeholder로
                // 접는다. 이미 메모리에 있는 `Loaded` 값도 `load_limited()`에서
                // 길이를 확인했으므로 같은 경로를 탄다.
                None => {
                    content_loss.record(ContentLoss::binary_content_emptied(
                        storage_id,
                        path.clone(),
                        ContentLossReason::RawPassthroughUnavailable,
                    ));
                    streams.push((path, encrypt_if_password(Vec::new(), password)));
                    continue;
                }
            },
        };

        // OLE Storage 복원: 파서(`load_bin_data_content`)는 내부 CFB 를 바로 노출하기 위해
        // 선두 4-byte LE size prefix 를 제거(`drain(..4)`)한다. 직렬화 시 이를 다시 붙이지
        // 않으면 한컴이 CFB 매직(D0CF11E0)을 OLE 개체 크기(~3.75GB)로 오인하여
        // "메모리 부족" 오류가 발생한다. 파서의 strip 조건을 그대로 미러링한다.
        let is_ole_storage = bytes.len() >= 8
            && bytes[..8] == CFB_MAGIC
            && bin_data_list
                .iter()
                .any(|bd| bd.data_type == BinDataType::Storage && bd.storage_id == content.id);
        let payload: Vec<u8> = if is_ole_storage {
            let mut v = Vec::with_capacity(bytes.len() + 4);
            v.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            v.extend_from_slice(&bytes);
            v
        } else {
            bytes
        };

        let data = if should_compress {
            compress_stream(&payload).unwrap_or_else(|_| payload.clone())
        } else {
            payload
        };
        streams.push((path, encrypt_if_password(data, password)));
    }

    // 5. 미리보기 데이터 (PrvImage, PrvText)
    if let Some(ref prv) = preview {
        if let Some(ref image) = prv.image {
            streams.push(("/PrvImage".to_string(), image.data.clone()));
        }
        if let Some(ref text) = prv.text {
            // UTF-16LE로 인코딩
            let utf16: Vec<u16> = text.encode_utf16().collect();
            let mut bytes = Vec::with_capacity(utf16.len() * 2);
            for ch in &utf16 {
                bytes.extend_from_slice(&ch.to_le_bytes());
            }
            streams.push(("/PrvText".to_string(), bytes));
        }
    }

    // 6. 추가 스트림 (Scripts, DocOptions 등 — 라운드트립 보존)
    for (path, data) in extra_streams {
        let payload = if password.is_some()
            && (path.starts_with("/Scripts/") || path.starts_with("/BinData/"))
        {
            encrypt_if_password(data.clone(), password)
        } else {
            data.clone()
        };
        streams.push((path.clone(), payload));
    }

    // mini_cfb로 CFB 컨테이너 조립 (WASM 호환)
    let named_streams: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(path, data)| (path.as_str(), data.as_slice()))
        .collect();

    mini_cfb::build_cfb(&named_streams).map_err(|e| SerializeError::CfbError(e))
}

fn encrypt_if_password(data: Vec<u8>, password: Option<&[u8]>) -> Vec<u8> {
    match password {
        Some(value) => encrypt_hwp5_stream(&data, value),
        None => data,
    }
}

/// BinDataContent에 대응하는 BinData 정보(storage_id, extension, should_compress) 찾기
///
/// should_compress: BinData의 압축 속성에 따라 재압축 여부 결정
/// - Default: 문서 전체 compressed 플래그 따름
/// - Compress: 항상 압축
/// - NoCompress: 비압축
fn find_bin_data_info_with_compress<'a>(
    bin_data_list: &'a [BinData],
    content: &'a BinDataContent,
    doc_compressed: bool,
) -> (u16, &'a str, bool) {
    use crate::model::bin_data::BinDataCompression;
    for bd in bin_data_list {
        if matches!(bd.data_type, BinDataType::Embedding | BinDataType::Storage)
            && bd.storage_id == content.id
        {
            let ext = bd.extension.as_deref().unwrap_or("dat");
            let should_compress = match bd.compression {
                BinDataCompression::Default => doc_compressed,
                BinDataCompression::Compress => true,
                BinDataCompression::NoCompress => false,
            };
            return (bd.storage_id, ext, should_compress);
        }
    }
    // 못 찾으면 content에서 직접 추출 (문서 압축 플래그 따름)
    (content.id, &content.extension, doc_compressed)
}

#[cfg(test)]
mod tests;
