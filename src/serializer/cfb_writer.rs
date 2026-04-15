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
use crate::model::bin_data::{BinData, BinDataType};
use crate::model::document::{Document, Preview};

use super::body_text::serialize_section;
use super::doc_info::serialize_doc_info;
use super::header::serialize_file_header;
use super::mini_cfb;

/// 직렬화 에러
#[derive(Debug)]
pub enum SerializeError {
    /// CFB 생성/쓰기 실패
    CfbError(String),
    /// 압축 실패
    CompressError(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::CfbError(e) => write!(f, "CFB 쓰기 실패: {}", e),
            SerializeError::CompressError(e) => write!(f, "압축 실패: {}", e),
        }
    }
}

impl std::error::Error for SerializeError {}

/// Document IR을 HWP 5.0 CFB 바이너리로 직렬화
pub fn serialize_hwp(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    // 1. FileHeader 직렬화
    let header_bytes = serialize_file_header(&doc.header);

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

    // 5. CFB 컨테이너 조립
    write_hwp_cfb(
        &header_bytes,
        &doc_info_bytes,
        &section_bytes_list,
        &doc.doc_info.bin_data_list,
        &doc.bin_data_content,
        &doc.preview,
        &doc.extra_streams,
        compressed,
    )
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
    streams.push(("/DocInfo".to_string(), doc_info_data));

    // 3. /BodyText/Section{N} (조건부 압축)
    for (i, section_bytes) in section_bytes_list.iter().enumerate() {
        let path = format!("/BodyText/Section{}", i);
        let data = if compressed {
            compress_stream(section_bytes)?
        } else {
            section_bytes.clone()
        };
        streams.push((path, data));
    }

    // 4. /BinData/BIN{XXXX}.{ext}
    // BinData는 개별 압축 속성에 따라 재압축
    for content in bin_data_content {
        let (storage_id, ext, should_compress) =
            find_bin_data_info_with_compress(bin_data_list, content, compressed);
        let storage_name = format!("BIN{:04X}.{}", storage_id, ext);
        let path = format!("/BinData/{}", storage_name);
        let data = if should_compress {
            compress_stream(&content.data).unwrap_or_else(|_| content.data.clone())
        } else {
            content.data.clone()
        };
        streams.push((path, data));
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
        streams.push((path.clone(), data.clone()));
    }

    // mini_cfb로 CFB 컨테이너 조립 (WASM 호환)
    let named_streams: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(path, data)| (path.as_str(), data.as_slice()))
        .collect();

    mini_cfb::build_cfb(&named_streams).map_err(|e| SerializeError::CfbError(e))
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
        if bd.data_type == BinDataType::Embedding && bd.storage_id == content.id {
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
