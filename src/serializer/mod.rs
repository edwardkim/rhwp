//! HWP 문서 직렬화 모듈
//!
//! Document IR을 HWP 5.0 바이너리 파일로 변환하는 기능을 제공한다.
//! `parser` 모듈의 역방향으로 동작한다.

pub mod body_text;
pub mod byte_writer;
pub mod cfb_writer;
pub mod control;
pub mod doc_info;
pub mod header;
pub mod hml;
pub mod hwpx;
pub mod mini_cfb;
pub mod record_writer;

pub use cfb_writer::serialize_hwp;
pub use hml::serialize_hml;
pub use hwpx::serialize_hwpx;

/// 직렬화 에러 (HWP + HWPX 공용)
#[derive(Debug)]
pub enum SerializeError {
    /// CFB 생성/쓰기 실패 (HWP)
    CfbError(String),
    /// 압축 실패 (HWP/HWPX 공용)
    CompressError(String),
    /// ZIP 생성/쓰기 실패 (HWPX)
    ZipError(String),
    /// XML 생성 실패 (HWPX)
    XmlError(String),
    /// 지원하지 않는 입력 (예: HWP 소스를 HWPX 직렬화기에 넘긴 경우)
    UnsupportedInput(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::CfbError(e) => write!(f, "CFB 쓰기 실패: {}", e),
            SerializeError::CompressError(e) => write!(f, "압축 실패: {}", e),
            SerializeError::ZipError(e) => write!(f, "ZIP 쓰기 실패: {}", e),
            SerializeError::XmlError(e) => write!(f, "XML 쓰기 실패: {}", e),
            SerializeError::UnsupportedInput(e) => write!(f, "지원하지 않는 입력: {}", e),
        }
    }
}

impl std::error::Error for SerializeError {}

use crate::model::document::Document;

// ---------------------------------------------------------------------------
// Trait 추상화: DocumentSerializer
// ---------------------------------------------------------------------------

/// 문서 직렬화 trait — Document IR을 바이트로 변환
pub trait DocumentSerializer {
    fn serialize(&self, doc: &Document) -> Result<Vec<u8>, SerializeError>;
}

/// HWP 5.0 바이너리 직렬화
pub struct HwpSerializer;

impl DocumentSerializer for HwpSerializer {
    fn serialize(&self, doc: &Document) -> Result<Vec<u8>, SerializeError> {
        serialize_hwp(doc)
    }
}

/// HWPX(ZIP+XML) 직렬화
pub struct HwpxSerializer;

impl DocumentSerializer for HwpxSerializer {
    fn serialize(&self, doc: &Document) -> Result<Vec<u8>, SerializeError> {
        serialize_hwpx(doc)
    }
}

/// 현재 지원 포맷(HWP)으로 직렬화
pub fn serialize_document(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    HwpSerializer.serialize(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSerializer;
    impl DocumentSerializer for MockSerializer {
        fn serialize(&self, _doc: &Document) -> Result<Vec<u8>, SerializeError> {
            Ok(vec![0xDE, 0xAD])
        }
    }

    #[test]
    fn test_mock_serializer() {
        let doc = Document::default();
        assert_eq!(MockSerializer.serialize(&doc).unwrap(), vec![0xDE, 0xAD]);
    }
}
