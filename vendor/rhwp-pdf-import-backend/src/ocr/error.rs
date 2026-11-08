//! OCR-specific error types.

use std::fmt;

/// Result type alias for OCR operations.
pub type OcrResult<T> = std::result::Result<T, OcrError>;

/// Errors that can occur during OCR operations.
#[derive(Debug)]
pub enum OcrError {
    /// Failed to load ONNX model
    ModelLoadError(String),

    /// Failed during model inference
    InferenceError(String),

    /// Invalid input image
    InvalidImage(String),

    /// Preprocessing failed
    PreprocessingError(String),

    /// Postprocessing failed (box extraction, NMS)
    PostprocessingError(String),

    /// Dictionary/character set error
    DictionaryError(String),

    /// No text detected in image
    NoTextDetected,

    /// Configuration error
    ConfigError(String),

    /// I/O error (file not found, etc.)
    IoError(std::io::Error),
}

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OcrError::ModelLoadError(msg) => write!(f, "Failed to load OCR model: {}", msg),
            OcrError::InferenceError(msg) => write!(f, "OCR inference failed: {}", msg),
            OcrError::InvalidImage(msg) => write!(f, "Invalid image for OCR: {}", msg),
            OcrError::PreprocessingError(msg) => write!(f, "Image preprocessing failed: {}", msg),
            OcrError::PostprocessingError(msg) => write!(f, "OCR postprocessing failed: {}", msg),
            OcrError::DictionaryError(msg) => write!(f, "Character dictionary error: {}", msg),
            OcrError::NoTextDetected => write!(f, "No text detected in image"),
            OcrError::ConfigError(msg) => write!(f, "OCR configuration error: {}", msg),
            OcrError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for OcrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OcrError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for OcrError {
    fn from(err: std::io::Error) -> Self {
        OcrError::IoError(err)
    }
}

// Available wherever the OCR module is (`ocr` or `ocr-tract` — #524),
// so the `?` operator works on the tract/wasm path too.
#[cfg(any(feature = "ocr", feature = "ocr-tract"))]
impl From<OcrError> for crate::Error {
    fn from(err: OcrError) -> Self {
        crate::Error::Ocr(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OcrError::ModelLoadError("model.onnx not found".to_string());
        assert!(err.to_string().contains("model.onnx"));
    }

    #[test]
    fn test_error_conversion() {
        let ocr_err = OcrError::NoTextDetected;
        let pdf_err: crate::Error = ocr_err.into();
        assert!(pdf_err.to_string().contains("No text detected"));
    }
}
