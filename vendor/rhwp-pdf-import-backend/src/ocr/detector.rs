//! Text detection using DBNet++ model.
//!
//! DBNet++ (Differentiable Binarization Network) is a text detection model
//! that produces a probability map indicating text regions.

use std::path::Path;

use image::DynamicImage;
use ndarray::Array2;

use super::backend::{build_backend, InferenceBackend};
use super::config::OcrConfig;
use super::error::{OcrError, OcrResult};
use super::postprocessor::{extract_boxes, DetectedBox};
use super::preprocessor::preprocess_for_detection;

/// Text detector using a DBNet++ ONNX model, run through a pluggable
/// [`InferenceBackend`] (`ort` natively, `tract` on `wasm32`).
pub struct TextDetector {
    backend: Box<dyn InferenceBackend>,
    config: OcrConfig,
}

impl TextDetector {
    /// Create a new text detector from model file path.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the DBNet++ ONNX model file
    /// * `config` - OCR configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::ocr::{TextDetector, OcrConfig};
    ///
    /// let detector = TextDetector::new("models/det.onnx", OcrConfig::default())?;
    /// ```
    pub fn new(model_path: impl AsRef<Path>, config: OcrConfig) -> OcrResult<Self> {
        let model_bytes = std::fs::read(model_path.as_ref())
            .map_err(|e| OcrError::ModelLoadError(format!("Failed to read model file: {}", e)))?;

        Self::from_bytes(&model_bytes, config)
    }

    /// Create a new text detector from model bytes (for bundled models).
    ///
    /// # Arguments
    ///
    /// * `model_bytes` - ONNX model data as bytes
    /// * `config` - OCR configuration
    pub fn from_bytes(model_bytes: &[u8], config: OcrConfig) -> OcrResult<Self> {
        let backend = build_backend(model_bytes, config.num_threads)?;
        Ok(Self { backend, config })
    }

    /// Detect text regions in an image.
    ///
    /// # Arguments
    ///
    /// * `image` - Input image
    ///
    /// # Returns
    ///
    /// Vector of detected text boxes with coordinates and confidence scores.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let boxes = detector.detect(&image)?;
    /// for box in boxes {
    ///     println!("Found text at {:?} (confidence: {})", box.polygon, box.confidence);
    /// }
    /// ```
    pub fn detect(&self, image: &DynamicImage) -> OcrResult<Vec<DetectedBox>> {
        // Preprocess image
        let (input_tensor, scale) =
            preprocess_for_detection(image, &self.config.det_resize_strategy)?;

        // Run inference
        let prob_map = self.run_inference(&input_tensor)?;

        // Extract boxes from probability map
        let boxes = extract_boxes(
            prob_map.view(),
            self.config.det_threshold,
            self.config.box_threshold,
            self.config.max_candidates,
            self.config.unclip_ratio,
            scale,
        )?;

        Ok(boxes)
    }

    /// Run model inference through the configured backend.
    fn run_inference(&self, input: &ndarray::Array4<f32>) -> OcrResult<Array2<f32>> {
        // DBNet++ outputs an `[N, 1, H, W]` probability map.
        let output_array = self.backend.run(input)?;

        // Convert from [N, 1, H, W] to [H, W]
        let shape = output_array.shape();
        if shape.len() != 4 {
            return Err(OcrError::InferenceError(format!(
                "Unexpected output shape: {:?}, expected 4D tensor",
                shape
            )));
        }

        let height = shape[2];
        let width = shape[3];

        // Extract the probability map (first batch, first channel)
        let mut prob_map = Array2::zeros((height, width));
        for y in 0..height {
            for x in 0..width {
                prob_map[[y, x]] = output_array[[0, 0, y, x]];
            }
        }

        Ok(prob_map)
    }

    /// Check if a model is loaded. A `TextDetector` cannot be
    /// constructed without a successfully built backend, so this is
    /// always `true` once the value exists (kept for API stability).
    pub fn is_loaded(&self) -> bool {
        true
    }
}

// `TextDetector` is `Send + Sync`: the backend trait object is bound
// `Send + Sync` and all other fields are.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_config() {
        // Test that config is properly stored
        let config = OcrConfig::builder()
            .det_threshold(0.4)
            .box_threshold(0.6)
            .build();

        assert!((config.det_threshold - 0.4).abs() < f32::EPSILON);
        assert!((config.box_threshold - 0.6).abs() < f32::EPSILON);
    }

    // Note: Integration tests with actual models will be in tests/ocr/
    // These require model files to be present
}
