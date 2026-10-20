//! Error types for the PDF library.
//!
//! This module defines all error types that can occur during PDF parsing and processing.

#![forbid(unsafe_code)]

/// Result type alias for PDF library operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur during PDF processing.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)] // "Invalid" prefix is intentional for clarity
pub enum Error {
    /// A request-scoped import budget was exhausted. Never recover as partial success.
    #[error("PDF import resource limit: {0}")]
    ImportResourceLimit(&'static str),
    /// Invalid PDF header (expected '%PDF-')
    #[error("Invalid PDF header: expected '%PDF-', found '{0}'")]
    InvalidHeader(String),

    /// Unsupported PDF version
    #[error("Unsupported PDF version: {0}")]
    UnsupportedVersion(String),

    /// Parse error at specific byte offset
    #[error("Failed to parse object at byte {offset}: {reason}")]
    ParseError {
        /// Byte offset where error occurred
        offset: usize,
        /// Reason for parse failure
        reason: String,
    },

    /// Parse warning (non-fatal)
    #[error("Parse warning at byte {offset}: {message}")]
    ParseWarning {
        /// Byte offset where warning occurred
        offset: usize,
        /// Warning message
        message: String,
    },

    /// Invalid cross-reference table
    #[error("Invalid cross-reference table")]
    InvalidXref,

    /// Referenced object not found in cross-reference table
    #[error("Object not found: {0} {1} R")]
    ObjectNotFound(u32, u16),

    /// Object has wrong type
    #[error("Invalid object type: expected {expected}, found {found}")]
    InvalidObjectType {
        /// Expected object type
        expected: String,
        /// Actual object type found
        found: String,
    },

    /// Unexpected end of file
    #[error("End of file reached unexpectedly")]
    UnexpectedEof,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 decoding error
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    // Additional error types for later phases
    /// Invalid PDF structure (generic)
    #[error("Invalid PDF: {0}")]
    InvalidPdf(String),

    /// Stream decoding error
    #[error("Stream decoding error: {0}")]
    Decode(String),

    /// Encoding error (e.g., image encoding)
    #[error("Encoding error: {0}")]
    Encode(String),

    /// Unsupported stream filter
    #[error("Unsupported filter: {0}")]
    UnsupportedFilter(String),

    /// Font error
    #[error("Font error: {0}")]
    Font(String),

    /// Image error
    #[error("Image error: {0}")]
    Image(String),

    /// ML model error
    #[cfg(feature = "ml")]
    #[error("ML error: {0}")]
    Ml(String),

    /// OCR error. Available whenever the OCR module is compiled —
    /// `ocr` (native ONNX Runtime) or `ocr-tract` (pure-Rust tract /
    /// wasm, which `ml` implies — issue #524).
    #[cfg(any(feature = "ocr", feature = "ocr-tract"))]
    #[error("OCR error: {0}")]
    Ocr(String),

    /// Circular reference detected in object graph
    #[error("Circular reference detected: object {0}")]
    CircularReference(crate::object::ObjectRef),

    /// Recursion depth limit exceeded
    #[error("Recursion depth limit exceeded (max: {0})")]
    RecursionLimitExceeded(u32),

    /// Invalid operation (e.g., calling methods on uninitialized document)
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Layout analysis error
    #[error("Layout analysis error: {0}")]
    LayoutAnalysis(String),

    /// Barcode/QR code generation error
    #[error("Barcode error: {0}")]
    Barcode(String),

    /// PDF is encrypted and has not been authenticated with the correct password.
    ///
    /// This error is returned when attempting to extract content from a PDF that
    /// requires a password. Call `authenticate(password)` or open with
    /// `PdfDocument::open_with_password()` before extracting content.
    #[error("PDF is encrypted and requires a password. Call authenticate(password) before extracting content.")]
    EncryptedPdf,

    /// OCR backend is unavailable. Additive variant for the
    /// OCR API cluster. Typed reason lets the
    /// binding map to the right language-native exception
    /// (`OcrUnavailable` in Python / Java / Ruby / PHP / Go / C# /
    /// Node / WASM per `research-typed-signals-cross-lang.md` §13).
    #[error("OCR unavailable: {reason:?}")]
    OcrUnavailable {
        /// Why OCR couldn't be initialised or invoked.
        reason: crate::extractors::status::OcrUnavailableReason,
    },

    /// Runtime error during PostScript Type 4 function evaluation
    /// (stack underflow, typecheck, division by zero, overflow, undefined
    /// operations).
    ///
    /// Distinct from [`Error::InvalidPdf`], which means a malformed parse;
    /// `Type4Runtime` means a syntactically valid program failed at execution
    /// time. Callers should typically fall back to a default gray rendering
    /// rather than failing the entire page.
    #[error("Type 4 runtime error: {0}")]
    Type4Runtime(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_header_error() {
        let err = Error::InvalidHeader("NotAPDF".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid PDF header"));
        assert!(msg.contains("NotAPDF"));
    }

    #[test]
    fn test_unsupported_version_error() {
        let err = Error::UnsupportedVersion("3.0".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Unsupported PDF version"));
        assert!(msg.contains("3.0"));
    }

    #[test]
    fn test_parse_error() {
        let err = Error::ParseError {
            offset: 1234,
            reason: "invalid token".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("1234"));
        assert!(msg.contains("invalid token"));
    }

    #[test]
    fn test_object_not_found_error() {
        let err = Error::ObjectNotFound(10, 0);
        let msg = format!("{}", err);
        assert!(msg.contains("10 0 R"));
    }

    #[test]
    fn test_invalid_object_type_error() {
        let err = Error::InvalidObjectType {
            expected: "Dictionary".to_string(),
            found: "Array".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Dictionary"));
        assert!(msg.contains("Array"));
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_encrypted_pdf_error() {
        let err = Error::EncryptedPdf;
        let msg = format!("{}", err);
        assert!(msg.contains("encrypted"));
        assert!(msg.contains("password"));
    }
}
