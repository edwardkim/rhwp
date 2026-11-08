//! Main OCR engine combining detection and recognition.
//!
//! The OcrEngine provides a high-level interface for performing OCR on images,
//! coordinating the detection and recognition pipelines.

use std::path::Path;

use image::DynamicImage;

use super::config::OcrConfig;
use super::detector::TextDetector;
use super::error::OcrResult;
use super::preprocessor::crop_text_region;
use super::recognizer::TextRecognizer;

/// Recognized text span with position and confidence.
#[derive(Debug, Clone)]
pub struct OcrSpan {
    /// Recognized text
    pub text: String,
    /// Quadrilateral bounding box [top-left, top-right, bottom-right, bottom-left]
    pub polygon: [[f32; 2]; 4],
    /// Overall confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Per-character confidence scores
    pub char_confidences: Vec<f32>,
}

impl OcrSpan {
    /// Convert OCR span to a TextSpan for integration with existing text extraction.
    ///
    /// This creates a TextSpan with:
    /// - Bounding box converted from polygon
    /// - Font size estimated from text height
    /// - Default styling (font name "OCR", normal weight, black color)
    ///
    /// # Arguments
    ///
    /// * `sequence` - Sequence number for reading order
    /// * `scale` - Scale factor to convert from image coordinates to PDF coordinates
    ///            (typically image_dpi / 72.0 to convert to points)
    pub fn to_text_span(&self, sequence: usize, scale: f32) -> crate::layout::text_block::TextSpan {
        use crate::geometry::Rect;
        use crate::layout::text_block::{Color, FontWeight, TextSpan};

        // Convert polygon to axis-aligned bounding box
        let min_x = self.polygon.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
        let max_x = self.polygon.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
        let min_y = self.polygon.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let max_y = self.polygon.iter().map(|p| p[1]).fold(f32::MIN, f32::max);

        // Apply scale to convert image coordinates to PDF coordinates
        let bbox = Rect::new(min_x / scale, min_y / scale, max_x / scale, max_y / scale);

        // Estimate font size from text height
        let height_pixels = max_y - min_y;
        let font_size = self.estimate_font_size(height_pixels, scale);

        TextSpan {
            provenance: None,
            artifact_type: None,
            text: self.text.clone(),
            bbox,
            font_name: "OCR".to_string(),
            font_size,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence,
            split_boundary_before: false,
            offset_semantic: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            text_rise: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: Vec::new(),
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }
    }

    /// Estimate font size in points from text box height.
    ///
    /// Uses heuristic: font_size ≈ height * 0.75 (accounting for descenders/ascenders)
    fn estimate_font_size(&self, height_pixels: f32, scale: f32) -> f32 {
        // Convert pixel height to points and apply heuristic
        // Typical text boxes include space for ascenders/descenders
        // so actual font size is about 75% of the box height
        let height_points = height_pixels / scale;
        (height_points * 0.75).clamp(6.0, 72.0) // Clamp to reasonable font sizes
    }

    /// Get the axis-aligned bounding box of the polygon.
    pub fn bounding_rect(&self) -> crate::geometry::Rect {
        use crate::geometry::Rect;

        let min_x = self.polygon.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
        let max_x = self.polygon.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
        let min_y = self.polygon.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let max_y = self.polygon.iter().map(|p| p[1]).fold(f32::MIN, f32::max);

        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

/// Result of OCR processing on an image.
#[derive(Debug, Clone)]
pub struct OcrOutput {
    /// All recognized text spans
    pub spans: Vec<OcrSpan>,
    /// Average confidence across all spans
    pub total_confidence: f32,
}

impl OcrOutput {
    /// Get all text concatenated with spaces.
    pub fn text(&self) -> String {
        self.spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Total-order reading-order comparison for two detection boxes: group by a
    /// fixed 10-px Y band (top→bottom), then left→right by X, then raw Y as a
    /// stable tiebreaker.
    ///
    /// The band must be quantised from each box's own Y. The earlier rule
    /// ("compare X when |Δy| < 10, else compare Y") is **not transitive** — for
    /// near-aligned boxes A≈B and B≈C share a band and sort by X, but A vs C can
    /// fall outside the band and sort by Y, yielding a cycle. Rust's sort detects
    /// that and panics ("comparison function does not correctly implement a total
    /// order"), aborting the host process on image_text slides with a few labels
    /// on near-identical baselines. Quantising removes the relativity, so
    /// this is a genuine total order (integer band cmp, then the total-order
    /// `safe_float_cmp` on X then Y).
    fn reading_order_cmp(a: &[[f32; 2]; 4], b: &[[f32; 2]; 4]) -> std::cmp::Ordering {
        const Y_BAND: f32 = 10.0;
        let band = |y: f32| (y / Y_BAND).round() as i64;
        band(a[0][1])
            .cmp(&band(b[0][1]))
            .then_with(|| crate::utils::safe_float_cmp(a[0][0], b[0][0]))
            .then_with(|| crate::utils::safe_float_cmp(a[0][1], b[0][1]))
    }

    /// Get text spans sorted by reading order (top-to-bottom, left-to-right).
    pub fn text_in_reading_order(&self) -> String {
        let mut spans: Vec<_> = self.spans.iter().collect();

        spans.sort_by(|a, b| Self::reading_order_cmp(&a.polygon, &b.polygon));

        spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Convert all OCR spans to TextSpans for integration with layout analysis.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale factor to convert from image coordinates to PDF coordinates
    ///            (typically image_dpi / 72.0 to convert to points)
    ///
    /// # Returns
    ///
    /// Vector of TextSpans sorted in reading order.
    pub fn to_text_spans(&self, scale: f32) -> Vec<crate::layout::text_block::TextSpan> {
        let mut spans_with_pos: Vec<_> = self.spans.iter().enumerate().collect();

        // Sort by reading order (top to bottom, left to right) — total order.
        spans_with_pos.sort_by(|(_, a), (_, b)| Self::reading_order_cmp(&a.polygon, &b.polygon));

        // Convert to TextSpans with sequence numbers
        spans_with_pos
            .iter()
            .enumerate()
            .map(|(seq, (_, ocr_span))| ocr_span.to_text_span(seq, scale))
            .collect()
    }
}

/// Main OCR engine for text extraction from images.
///
/// Combines text detection (DBNet++) and recognition (SVTR) models
/// for end-to-end OCR.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::ocr::{OcrEngine, OcrConfig};
/// use image::open;
///
/// let engine = OcrEngine::new(
///     "models/det.onnx",
///     "models/rec.onnx",
///     "models/en_dict.txt",
///     OcrConfig::default()
/// )?;
///
/// let image = open("document.png")?;
/// let result = engine.ocr_image(&image)?;
///
/// println!("Extracted text: {}", result.text());
/// ```
pub struct OcrEngine {
    detector: TextDetector,
    recognizer: TextRecognizer,
    config: OcrConfig,
}

impl OcrEngine {
    /// Create a new OCR engine from model file paths.
    ///
    /// # Arguments
    ///
    /// * `det_model_path` - Path to DBNet++ detection model
    /// * `rec_model_path` - Path to SVTR recognition model
    /// * `dict_path` - Path to character dictionary
    /// * `config` - OCR configuration
    pub fn new(
        det_model_path: impl AsRef<Path>,
        rec_model_path: impl AsRef<Path>,
        dict_path: impl AsRef<Path>,
        config: OcrConfig,
    ) -> OcrResult<Self> {
        let detector = TextDetector::new(det_model_path, config.clone())?;
        let recognizer = TextRecognizer::new(rec_model_path, dict_path, config.clone())?;

        Ok(Self {
            detector,
            recognizer,
            config,
        })
    }

    /// Create a new OCR engine from model bytes (for bundled models).
    ///
    /// # Arguments
    ///
    /// * `det_model_bytes` - Detection model ONNX bytes
    /// * `rec_model_bytes` - Recognition model ONNX bytes
    /// * `dict_content` - Character dictionary content
    /// * `config` - OCR configuration
    pub fn from_bytes(
        det_model_bytes: &[u8],
        rec_model_bytes: &[u8],
        dict_content: &str,
        config: OcrConfig,
    ) -> OcrResult<Self> {
        let detector = TextDetector::from_bytes(det_model_bytes, config.clone())?;
        let recognizer = TextRecognizer::from_bytes(rec_model_bytes, dict_content, config.clone())?;

        Ok(Self {
            detector,
            recognizer,
            config,
        })
    }

    /// Perform OCR on an image.
    ///
    /// # Arguments
    ///
    /// * `image` - Input image
    ///
    /// # Returns
    ///
    /// OCR result containing all recognized text spans with positions.
    pub fn ocr_image(&self, image: &DynamicImage) -> OcrResult<OcrOutput> {
        // Step 1: Detect text regions
        let boxes = self.detector.detect(image)?;

        if boxes.is_empty() {
            return Ok(OcrOutput {
                spans: Vec::new(),
                total_confidence: 0.0,
            });
        }

        // Step 2: Recognize text in each region
        let mut spans = Vec::new();
        let mut total_confidence = 0.0;

        for detected_box in &boxes {
            // Crop the text region
            let crop = crop_text_region(image, &detected_box.polygon)?;

            // Recognize text in the crop
            let recognition = self.recognizer.recognize(&crop)?;

            // Filter out low-confidence results
            if recognition.confidence >= self.config.rec_threshold
                && !recognition.text.trim().is_empty()
            {
                total_confidence += recognition.confidence;

                spans.push(OcrSpan {
                    text: recognition.text,
                    polygon: detected_box.polygon,
                    confidence: recognition.confidence,
                    char_confidences: recognition.char_confidences,
                });
            }
        }

        // Calculate average confidence
        let avg_confidence = if spans.is_empty() {
            0.0
        } else {
            total_confidence / spans.len() as f32
        };

        Ok(OcrOutput {
            spans,
            total_confidence: avg_confidence,
        })
    }

    /// Get reference to the detector.
    pub fn detector(&self) -> &TextDetector {
        &self.detector
    }

    /// Get reference to the recognizer.
    pub fn recognizer(&self) -> &TextRecognizer {
        &self.recognizer
    }

    /// Get the configuration.
    pub fn config(&self) -> &OcrConfig {
        &self.config
    }
}

// OcrEngine is Send + Sync because its components use Mutex

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_output_text() {
        let result = OcrOutput {
            spans: vec![
                OcrSpan {
                    text: "Hello".to_string(),
                    polygon: [[0.0, 0.0], [50.0, 0.0], [50.0, 20.0], [0.0, 20.0]],
                    confidence: 0.95,
                    char_confidences: vec![],
                },
                OcrSpan {
                    text: "World".to_string(),
                    polygon: [[60.0, 0.0], [110.0, 0.0], [110.0, 20.0], [60.0, 20.0]],
                    confidence: 0.92,
                    char_confidences: vec![],
                },
            ],
            total_confidence: 0.935,
        };

        assert_eq!(result.text(), "Hello World");
    }

    // The reading-order comparator must be a TOTAL ORDER. The old
    // "compare X when |Δy| < 10, else compare Y" rule was intransitive on
    // near-aligned boxes and made Rust's sort panic (aborting the host
    // process). Brute-force antisymmetry + transitivity over a set that
    // includes the cyclic triple, and confirm a real sort does not panic.
    #[test]
    fn test_reading_order_cmp_is_total_order() {
        use std::cmp::Ordering;
        let poly = |x: f32, y: f32| [[x, y], [x + 5.0, y], [x + 5.0, y + 2.0], [x, y + 2.0]];
        // The cyclic triple under the old rule: A≈B and B≈C by X within 10px,
        // but A vs C compared by Y → cycle. Plus assorted near/far boxes.
        let pts = [
            poly(10.0, 0.0),
            poly(5.0, 8.0),
            poly(0.0, 16.0),
            poly(0.0, 0.0),
            poly(100.0, 1.0),
            poly(50.0, 9.0),
            poly(7.0, 23.0),
            poly(7.0, 24.0),
            poly(7.0, 25.0),
        ];
        for a in &pts {
            for b in &pts {
                assert_eq!(
                    OcrOutput::reading_order_cmp(a, b),
                    OcrOutput::reading_order_cmp(b, a).reverse(),
                    "antisymmetry"
                );
            }
        }
        let le = |x, y| OcrOutput::reading_order_cmp(x, y) != Ordering::Greater;
        for a in &pts {
            for b in &pts {
                for c in &pts {
                    if le(a, b) && le(b, c) {
                        assert!(le(a, c), "transitivity violated");
                    }
                }
            }
        }
        // The original symptom: sorting must not panic.
        let mut v = pts.to_vec();
        v.sort_by(OcrOutput::reading_order_cmp);
        assert_eq!(v.len(), pts.len());
    }

    #[test]
    fn test_ocr_output_reading_order() {
        let result = OcrOutput {
            spans: vec![
                // Second line
                OcrSpan {
                    text: "Line2".to_string(),
                    polygon: [[0.0, 50.0], [50.0, 50.0], [50.0, 70.0], [0.0, 70.0]],
                    confidence: 0.9,
                    char_confidences: vec![],
                },
                // First line
                OcrSpan {
                    text: "Line1".to_string(),
                    polygon: [[0.0, 0.0], [50.0, 0.0], [50.0, 20.0], [0.0, 20.0]],
                    confidence: 0.9,
                    char_confidences: vec![],
                },
            ],
            total_confidence: 0.9,
        };

        // Should sort by Y position (top to bottom)
        assert_eq!(result.text_in_reading_order(), "Line1 Line2");
    }

    #[test]
    fn test_ocr_span() {
        let span = OcrSpan {
            text: "Test".to_string(),
            polygon: [[10.0, 20.0], [110.0, 20.0], [110.0, 60.0], [10.0, 60.0]],
            confidence: 0.98,
            char_confidences: vec![0.99, 0.97, 0.98, 0.99],
        };

        assert_eq!(span.text, "Test");
        assert!(span.confidence > 0.9);
    }
}
