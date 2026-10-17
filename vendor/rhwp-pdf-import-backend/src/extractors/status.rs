//! OCR-availability reason — surfaced by the OCR API surface when the
//! engine fails to initialise (missing dylib, model load error, etc.).
//!
//! Re-exported from `crate::extractors::OcrUnavailableReason`.
//!
//! `OcrUnavailableReason` is the only production-consumed type in
//! this module (used as the payload of `Error::OcrUnavailable`).
//! A broader `ExtractionSignal` enum was prototyped to back per-call
//! `*_status` companion accessors; those accessors never shipped and
//! the speculative enum was removed.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Reason that OCR is unavailable on the current call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OcrUnavailableReason {
    /// `libonnxruntime.so` / `.dylib` / `.dll` failed to load via
    /// `dlopen` / `LoadLibrary`.
    DylibMissing,
    /// OCR feature is compile-time disabled in this build.
    FeatureDisabled,
    /// No `OcrEngine` was supplied and the caller invoked
    /// `extract_text_ocr_only` (which requires an explicit engine).
    /// `extract_text_auto` does NOT raise this — it silently degrades.
    EngineNotProvided,
    /// `ort::Session::run` or `Session::builder().commit()` returned
    /// an error.
    ModelLoadFailed {
        /// Underlying error string from the ORT crate.
        detail: String,
    },
    /// ORT init panicked (e.g. corrupted Mutex from a prior failed init).
    /// Captured by `std::panic::catch_unwind`.
    InitPanicked {
        /// Panic payload as a string.
        detail: String,
    },
}

impl OcrUnavailableReason {
    /// Stable string identifier (matches the Python-binding string form).
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::DylibMissing => "dylib_missing",
            Self::FeatureDisabled => "feature_disabled",
            Self::EngineNotProvided => "engine_not_provided",
            Self::ModelLoadFailed { .. } => "model_load_failed",
            Self::InitPanicked { .. } => "init_panicked",
        }
    }

    /// Detail string (empty for variants without detail).
    pub fn detail(&self) -> String {
        match self {
            Self::ModelLoadFailed { detail } | Self::InitPanicked { detail } => detail.clone(),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_unavailable_kind_str_stable() {
        assert_eq!(OcrUnavailableReason::DylibMissing.kind_str(), "dylib_missing");
        assert_eq!(OcrUnavailableReason::EngineNotProvided.kind_str(), "engine_not_provided");
        assert_eq!(
            OcrUnavailableReason::ModelLoadFailed { detail: "x".into() }.kind_str(),
            "model_load_failed"
        );
    }

    #[test]
    fn ocr_unavailable_detail_passthrough() {
        let r = OcrUnavailableReason::ModelLoadFailed {
            detail: "missing.onnx".into(),
        };
        assert_eq!(r.detail(), "missing.onnx");
        assert_eq!(OcrUnavailableReason::DylibMissing.detail(), "");
    }
}
