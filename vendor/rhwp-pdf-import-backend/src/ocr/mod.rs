//! OCR (Optical Character Recognition) module for scanned PDF text extraction.
//!
//! This module provides PaddleOCR-based text extraction for scanned PDFs using
//! ONNX Runtime for CPU-only inference. It integrates seamlessly with the existing
//! text extraction pipeline.
//!
//! # Features
//!
//! - **Auto-detect scanned pages**: Automatically identify pages that need OCR
//! - **Unified output**: OCR results match the format of native text extraction
//! - **Style detection**: Infer font sizes and heading styles from OCR geometry
//! - **Fast CPU inference**: Target < 1 second per A4 page on modern CPU
//!
//! # Architecture
//!
//! The OCR pipeline consists of:
//! 1. **Preprocessing**: Image resizing, normalization, tensor conversion
//! 2. **Detection**: DBNet++ model finds text regions (bounding boxes)
//! 3. **Recognition**: SVTR model reads text from cropped regions
//! 4. **Postprocessing**: Convert OCR results to TextSpan format
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::{PdfDocument, ocr::OcrEngine};
//!
//! let mut doc = PdfDocument::open("scanned.pdf")?;
//! let engine = OcrEngine::new()?;
//!
//! // Check if page needs OCR
//! if ocr::needs_ocr(&doc, 0)? {
//!     let result = engine.ocr_page(&doc, 0)?;
//!     for span in result.spans {
//!         println!("{} at {:?}", span.text, span.bbox);
//!     }
//! }
//! ```

// Sub-modules
mod backend;
mod config;
mod detector;
mod engine;
mod error;
mod postprocessor;
mod preprocessor;
mod recognizer;

// Re-exports
pub use config::{DetResizeStrategy, OcrConfig, OcrConfigBuilder};
pub use detector::TextDetector;
pub use engine::{OcrEngine, OcrOutput, OcrSpan};
pub use error::OcrError;
pub use postprocessor::DetectedBox;
pub use preprocessor::{crop_text_region, preprocess_for_detection, preprocess_for_recognition};
pub use recognizer::{RecognitionResult, TextRecognizer};

// High-level OCR functions and types exported at module level:
// PageType, detect_page_type, needs_ocr, ocr_page, ocr_page_spans, extract_text_with_ocr

use crate::{PdfDocument, Result};

/// Check if a PDF page needs OCR (is a scanned page).
///
/// A page is considered "scanned" if:
/// 1. It has no native text (or very little)
/// 2. It contains images (typically a full-page scan)
///
/// # Arguments
///
/// * `doc` - The PDF document
/// * `page` - Page number (0-indexed)
///
/// # Returns
///
/// `true` if the page likely needs OCR, `false` otherwise.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::{PdfDocument, ocr};
///
/// let mut doc = PdfDocument::open("document.pdf")?;
/// if ocr::needs_ocr(&doc, 0)? {
///     println!("Page 0 is scanned, needs OCR");
/// }
/// ```
/// Result of scanned page detection with granular classification.
#[derive(Debug, Clone, PartialEq)]
pub enum PageType {
    /// Page has native text — no OCR needed.
    NativeText,
    /// Page is fully scanned (large image, no/minimal text) — OCR the whole page.
    ScannedPage,
    /// Page has some native text but also large images that may contain text.
    /// Hybrid merge should be used: native text + OCR for image regions.
    HybridPage,
}

/// Detect the type of a PDF page for OCR purposes.
///
/// Since #460 this delegates to the unified v0.3.51 classifier
/// [`PdfDocument::classify_page`] (render-mode-3, union
/// CTM-transformed image coverage, the enriched T0.5 garbled gate,
/// structure-tree + producer priors) and maps its [`PageKind`] to a
/// [`PageType`]: `TextLayer`/`Empty` → `NativeText`, `Scanned` →
/// `ScannedPage`, `ImageText`/`Mixed` → `HybridPage`. This keeps
/// `detect_page_type` / `needs_ocr` / `extract_text_with_ocr` and the
/// `AutoExtractor` making the *same* per-page decision (the headline
/// #460 win); the old bespoke text-length / largest-image / U+FFFD
/// heuristics were replaced — the `PageType` contract is unchanged.
///
/// # Arguments
///
/// * `doc` - The PDF document
/// * `page` - Page number (0-indexed)
///
/// # Returns
///
/// The detected [`PageType`].
pub fn detect_page_type(doc: &PdfDocument, page: usize) -> Result<PageType> {
    // #460: route OCR detection through the unified v0.3.51 classifier
    // (`PdfDocument::classify_page`) instead of a separate heuristic, so
    // `detect_page_type` / `needs_ocr` / `extract_text_with_ocr` and the
    // new `AutoExtractor`/`extract_page_auto` make the *same* per-page
    // decision (the headline #460 win — no divergent reading order /
    // sparse-text-over-scan misfire). The classifier is signal-richer
    // (render-mode-3, union transformed-image coverage, enriched T0.5
    // garbled gate) than the old text-len/largest-image heuristic, so
    // results are equal-or-better; the `PageType` contract is unchanged.
    // (No recursion: `classify_page` uses `extract_spans`, never
    // `extract_text`.)
    use crate::extractors::auto::PageKind;
    let cls = doc.classify_page(page)?;
    Ok(match cls.kind {
        // Usable native text (incl. a good invisible OCR sidecar) or a
        // blank page → no OCR needed.
        PageKind::TextLayer | PageKind::Empty => PageType::NativeText,
        // Image-dominated / garbled-or-no usable text → OCR the page.
        PageKind::Scanned => PageType::ScannedPage,
        // Real native text *and* image regions that may carry text →
        // hybrid merge (native + region OCR).
        PageKind::ImageText | PageKind::Mixed => PageType::HybridPage,
    })
}

/// Check if a PDF page needs OCR (is a scanned page).
///
/// This is a simplified wrapper around [`detect_page_type`] that returns
/// `true` for both `ScannedPage` and `HybridPage` types.
pub fn needs_ocr(doc: &PdfDocument, page: usize) -> Result<bool> {
    let page_type = detect_page_type(doc, page)?;
    Ok(matches!(page_type, PageType::ScannedPage | PageType::HybridPage))
}

/// OCR text extraction options.
#[derive(Debug, Clone)]
pub struct OcrExtractOptions {
    /// OCR configuration
    pub config: OcrConfig,
    /// Scale factor for coordinate conversion (image DPI / 72.0)
    /// Default: 300.0 / 72.0 ≈ 4.17 (assumes 300 DPI scan)
    pub scale: f32,
    /// Whether to fall back to native text if OCR fails
    pub fallback_to_native: bool,
}

impl Default for OcrExtractOptions {
    fn default() -> Self {
        Self {
            config: OcrConfig::default(),
            scale: 300.0 / 72.0, // Assume 300 DPI scanned document
            fallback_to_native: true,
        }
    }
}

impl OcrExtractOptions {
    /// Create options with a custom DPI.
    pub fn with_dpi(dpi: f32) -> Self {
        Self {
            scale: dpi / 72.0,
            ..Default::default()
        }
    }
}

/// OCR a single page of a PDF document.
///
/// This function:
/// 1. Extracts the largest image from the page (assumed to be the scan)
/// 2. Converts it to a DynamicImage
/// 3. Runs OCR on the image
/// 4. Returns the recognized text
///
/// # Arguments
///
/// * `doc` - The PDF document
/// * `page` - Page number (0-indexed)
/// * `engine` - The OCR engine to use
/// * `options` - OCR extraction options
///
/// # Returns
///
/// The recognized text from the page.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::{PdfDocument, ocr::{self, OcrEngine, OcrConfig}};
///
/// let mut doc = PdfDocument::open("scanned.pdf")?;
/// let engine = OcrEngine::new("det.onnx", "rec.onnx", "dict.txt", OcrConfig::default())?;
///
/// let text = ocr::ocr_page(&doc, 0, &engine, OcrExtractOptions::default())?;
/// println!("OCR text: {}", text);
/// ```
pub fn ocr_page(
    doc: &PdfDocument,
    page: usize,
    engine: &OcrEngine,
    options: &OcrExtractOptions,
) -> Result<String> {
    // Extract images from the page
    let images = doc.extract_images(page)?;

    if images.is_empty() {
        if options.fallback_to_native {
            return doc.extract_text(page);
        }
        return Ok(String::new());
    }

    // Find the largest image (assumed to be the page scan)
    let largest_image = images
        .iter()
        .max_by_key(|img| (img.width() as u64) * (img.height() as u64))
        .unwrap();

    // Convert to DynamicImage
    let dynamic_image = largest_image.to_dynamic_image()?;

    // Run OCR
    let ocr_result = engine
        .ocr_image(&dynamic_image)
        .map_err(|e| crate::error::Error::Image(format!("OCR failed: {}", e)))?;

    // Return the text in reading order
    Ok(ocr_result.text_in_reading_order())
}

/// OCR a page and return TextSpans for layout integration.
///
/// This function is similar to `ocr_page` but returns structured TextSpans
/// that can be used with the existing layout analysis pipeline.
///
/// # Arguments
///
/// * `doc` - The PDF document
/// * `page` - Page number (0-indexed)
/// * `engine` - The OCR engine to use
/// * `options` - OCR extraction options
///
/// # Returns
///
/// Vector of TextSpans from the OCR result.
pub fn ocr_page_spans(
    doc: &PdfDocument,
    page: usize,
    engine: &OcrEngine,
    options: &OcrExtractOptions,
) -> Result<Vec<crate::layout::text_block::TextSpan>> {
    // Extract images from the page
    let images = doc.extract_images(page)?;

    if images.is_empty() {
        return Ok(Vec::new());
    }

    // Find the largest image (assumed to be the page scan)
    let largest_image = images
        .iter()
        .max_by_key(|img| (img.width() as u64) * (img.height() as u64))
        .unwrap();

    // Convert to DynamicImage
    let dynamic_image = largest_image.to_dynamic_image()?;

    // Run OCR
    let ocr_result = engine
        .ocr_image(&dynamic_image)
        .map_err(|e| crate::error::Error::Image(format!("OCR failed: {}", e)))?;

    // Convert to TextSpans
    Ok(ocr_result.to_text_spans(options.scale))
}

/// Merge a page's native text layer with text OCR'd from its image
/// region(s) for a [`PageType::HybridPage`].
///
/// A hybrid page has a genuine native text layer **and** a raster
/// image that may carry its own text. The two are disjoint sources, so
/// the correct combined result is their union (native first — it is
/// the higher-fidelity, reading-ordered layer — then any OCR fragment
/// not already represented natively). OCR lines whose
/// whitespace-normalised, lower-cased form is already a substring of
/// the native layer are skipped, so a sparse invisible-OCR sidecar is
/// not double-emitted. Empty inputs degrade gracefully.
pub(crate) fn merge_native_and_ocr(native: &str, ocr: &str) -> String {
    let native_trimmed = native.trim_end();
    if ocr.trim().is_empty() {
        return native.to_string();
    }
    if native_trimmed.trim().is_empty() {
        return ocr.to_string();
    }
    let norm = |s: &str| {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let native_norm = norm(native_trimmed);
    let mut extra: Vec<&str> = Vec::new();
    for line in ocr.lines() {
        let lt = line.trim();
        if lt.is_empty() {
            continue;
        }
        let ln = norm(lt);
        // Skip blank-after-norm or already-present-in-native lines, and
        // de-dup within the OCR block itself.
        if ln.is_empty() || native_norm.contains(&ln) || extra.iter().any(|e| norm(e) == ln) {
            continue;
        }
        extra.push(lt);
    }
    if extra.is_empty() {
        return native.to_string();
    }
    format!("{native_trimmed}\n{}", extra.join("\n"))
}

/// Extract text from a page, automatically using OCR if needed.
///
/// This is the main entry point for text extraction that handles both
/// native PDF text and scanned pages transparently.
///
/// # Arguments
///
/// * `doc` - The PDF document
/// * `page` - Page number (0-indexed)
/// * `engine` - The OCR engine to use (optional, only needed for scanned pages)
/// * `options` - OCR extraction options
///
/// # Returns
///
/// The extracted text, either from native PDF text or OCR.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::{PdfDocument, ocr::{self, OcrEngine, OcrConfig, OcrExtractOptions}};
///
/// let mut doc = PdfDocument::open("mixed.pdf")?;
/// let engine = OcrEngine::new("det.onnx", "rec.onnx", "dict.txt", OcrConfig::default())?;
///
/// // Automatically uses native text or OCR as needed
/// let text = ocr::extract_text_with_ocr(&doc, 0, Some(&engine), OcrExtractOptions::default())?;
/// ```
pub fn extract_text_with_ocr(
    doc: &PdfDocument,
    page: usize,
    engine: Option<&OcrEngine>,
    options: OcrExtractOptions,
) -> Result<String> {
    let page_type = detect_page_type(doc, page)?;

    match page_type {
        PageType::NativeText => {
            // Native text is sufficient
            doc.extract_text(page)
        },
        PageType::ScannedPage => {
            // Full OCR needed
            if let Some(ocr_engine) = engine {
                match ocr_page(doc, page, ocr_engine, &options) {
                    Ok(ocr_text) => Ok(ocr_text),
                    Err(e) => {
                        log::warn!("OCR failed for scanned page {}: {}", page, e);
                        if options.fallback_to_native {
                            doc.extract_text(page)
                        } else {
                            Err(e)
                        }
                    },
                }
            } else {
                // No OCR engine, return whatever native text exists
                doc.extract_text(page)
            }
        },
        PageType::HybridPage => {
            // Real native text layer AND an image that may carry its own
            // text (screenshot/figure/caption). The two sources are
            // disjoint, so the correct result is their UNION — not
            // whichever string is longer. The old `ocr_len >
            // native_len*2 ? ocr : native` either/or silently DROPPED
            // the in-image text whenever the native layer was longer
            // (you got the paragraph, lost the caption — could not
            // "extract both"). #517's `PageKind::ImageText` is
            // specified as "native + region OCR"; honour it by merging.
            let native_text = doc.extract_text(page).unwrap_or_default();

            if let Some(ocr_engine) = engine {
                match ocr_page(doc, page, ocr_engine, &options) {
                    Ok(ocr_text) => Ok(merge_native_and_ocr(&native_text, &ocr_text)),
                    Err(e) => {
                        log::warn!("OCR failed for hybrid page {}: {}, using native text", page, e);
                        Ok(native_text)
                    },
                }
            } else {
                Ok(native_text)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_module_compiles() {
        let _ = OcrConfig::default();
    }

    #[test]
    fn test_ocr_extract_options_default() {
        let options = OcrExtractOptions::default();
        assert!((options.scale - 300.0 / 72.0).abs() < 0.01);
        assert!(options.fallback_to_native);
    }

    #[test]
    fn test_ocr_extract_options_with_dpi() {
        let options = OcrExtractOptions::with_dpi(200.0);
        assert!((options.scale - 200.0 / 72.0).abs() < 0.01);
    }

    #[test]
    fn test_page_type_enum() {
        assert_eq!(PageType::NativeText, PageType::NativeText);
        assert_ne!(PageType::NativeText, PageType::ScannedPage);
        assert_ne!(PageType::ScannedPage, PageType::HybridPage);
    }

    // Deterministic, model-free pins for the HybridPage merge — the
    // regression that an external consumer hit: a native text layer +
    // an image-with-text must yield the UNION, never one-or-the-other.
    #[test]
    fn merge_unions_disjoint_native_and_image_text() {
        let m = merge_native_and_ocr(
            "Native paragraph stays.\nSecond native line.",
            "Caption text from the figure",
        );
        assert!(m.contains("Native paragraph stays."), "{m:?}");
        assert!(m.contains("Second native line."), "{m:?}");
        assert!(m.contains("Caption text from the figure"), "{m:?}");
    }

    #[test]
    fn merge_is_not_either_or_when_native_is_longer() {
        // The exact shape of the old bug: native much longer than OCR.
        // Old code returned native and dropped the image text.
        let native = "A very long native paragraph ".repeat(8);
        let m = merge_native_and_ocr(&native, "INVOICE 42");
        assert!(m.contains("INVOICE 42"), "in-image text must survive: {m:?}");
        assert!(m.contains("A very long native paragraph"), "{m:?}");
    }

    #[test]
    fn merge_dedups_sidecar_and_handles_empties() {
        // An OCR line already present in the native layer is not
        // double-emitted (whitespace/case-insensitive).
        let m = merge_native_and_ocr("Hello World\nKeep me", "hello   world\nNEW LINE");
        assert_eq!(m.matches("Hello World").count(), 1, "{m:?}");
        assert!(m.contains("NEW LINE"), "{m:?}");
        assert_eq!(merge_native_and_ocr("only native", "   "), "only native");
        assert_eq!(merge_native_and_ocr("   ", "only ocr"), "only ocr");
    }
}
