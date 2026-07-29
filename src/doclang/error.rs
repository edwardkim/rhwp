//! HWP → DocLang 변환 파이프라인의 에러 타입.
//!
//! rhwp 본체의 [`crate::error`](crate::error) 와 같은 관례를 따라 매크로 없이
//! `Display` 와 `std::error::Error` 를 직접 구현한다.

/// Errors produced by the HWP → DocLang conversion pipeline.
#[derive(Debug)]
pub enum ConvertError {
    /// rhwp failed to parse the input bytes.
    Parse(String),

    /// Encrypted documents are rejected by rhwp itself (ParseError::EncryptedDocument).
    EncryptedDocument,

    /// Distribution (배포용) documents parse successfully in rhwp, but converting them
    /// is out of scope for v1 by policy — rejected explicitly at the adapter boundary.
    DistributionDocumentUnsupported,

    /// HWP 3.x and legacy HWPML inputs are out of scope for v1.
    UnsupportedFormat(&'static str),

    /// XML serialization failure.
    Xml(String),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::Parse(msg) => write!(f, "failed to parse HWP document: {msg}"),
            ConvertError::EncryptedDocument => {
                write!(f, "encrypted HWP documents are not supported")
            }
            ConvertError::DistributionDocumentUnsupported => {
                write!(
                    f,
                    "distribution (배포용) HWP documents are not supported in v1"
                )
            }
            ConvertError::UnsupportedFormat(fmt) => write!(f, "unsupported input format: {fmt}"),
            ConvertError::Xml(msg) => write!(f, "failed to serialize DocLang XML: {msg}"),
        }
    }
}

impl std::error::Error for ConvertError {}
