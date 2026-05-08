//! 외부 입력 변환 파이프라인 JSON 중간 표현 (Neumann 작업 1단계, Task #660).
//!
//! Claude Code Skill이 PDF/이미지/MD/DOCX를 분석하여 생성하는 `ingest_schema_v1.json` 을 읽어
//! Rust 측에서 [`Document`](crate::model::document::Document) IR로 변환하는 경로의 입력 단계다.
//!
//! 사용 예:
//! ```ignore
//! let bytes = std::fs::read("ingest.json").unwrap();
//! let ingest = rhwp::parser::ingest::parse_ingest_bytes(&bytes).unwrap();
//! ```

pub mod schema;

pub use schema::*;

use crate::error::HwpError;

/// JSON 바이트로부터 [`IngestDocument`]를 파싱한다.
pub fn parse_ingest_bytes(bytes: &[u8]) -> Result<IngestDocument, HwpError> {
    serde_json::from_slice::<IngestDocument>(bytes)
        .map_err(|e| HwpError::InvalidFile(format!("ingest JSON 파싱 실패: {e}")))
}

/// 문자열로부터 [`IngestDocument`]를 파싱한다.
pub fn parse_ingest_str(s: &str) -> Result<IngestDocument, HwpError> {
    serde_json::from_str::<IngestDocument>(s)
        .map_err(|e| HwpError::InvalidFile(format!("ingest JSON 파싱 실패: {e}")))
}
