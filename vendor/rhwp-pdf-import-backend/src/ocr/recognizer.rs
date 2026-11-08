//! Text recognition using SVTR model.
//!
//! SVTR (Scene Text Recognition with Visual and Linguistic Transformation)
//! recognizes text from cropped text region images.

use std::path::Path;

use image::DynamicImage;
use ndarray::Array4;

use super::backend::{build_backend, InferenceBackend};
use super::config::OcrConfig;
use super::error::{OcrError, OcrResult};
use super::preprocessor::preprocess_for_recognition;

/// Result of text recognition for a single text region.
#[derive(Debug, Clone)]
pub struct RecognitionResult {
    /// Recognized text
    pub text: String,
    /// Overall confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Per-character confidence scores
    pub char_confidences: Vec<f32>,
}

/// Text recognizer using an SVTR ONNX model, run through a pluggable
/// [`InferenceBackend`] (`ort` natively, `tract` on `wasm32`).
pub struct TextRecognizer {
    backend: Box<dyn InferenceBackend>,
    dictionary: Vec<char>,
    config: OcrConfig,
}

impl TextRecognizer {
    /// Create a new text recognizer from model file and dictionary.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the SVTR ONNX model file
    /// * `dict_path` - Path to the character dictionary file
    /// * `config` - OCR configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::ocr::{TextRecognizer, OcrConfig};
    ///
    /// let recognizer = TextRecognizer::new(
    ///     "models/rec.onnx",
    ///     "models/en_dict.txt",
    ///     OcrConfig::default()
    /// )?;
    /// ```
    pub fn new(
        model_path: impl AsRef<Path>,
        dict_path: impl AsRef<Path>,
        config: OcrConfig,
    ) -> OcrResult<Self> {
        let model_bytes = std::fs::read(model_path.as_ref())
            .map_err(|e| OcrError::ModelLoadError(format!("Failed to read model file: {}", e)))?;

        let dict_content = std::fs::read_to_string(dict_path.as_ref())
            .map_err(|e| OcrError::DictionaryError(format!("Failed to read dictionary: {}", e)))?;

        Self::from_bytes(&model_bytes, &dict_content, config)
    }

    /// Create a new text recognizer from model bytes and dictionary string.
    ///
    /// # Arguments
    ///
    /// * `model_bytes` - ONNX model data as bytes
    /// * `dict_content` - Character dictionary as string (one char per line)
    /// * `config` - OCR configuration
    pub fn from_bytes(
        model_bytes: &[u8],
        dict_content: &str,
        config: OcrConfig,
    ) -> OcrResult<Self> {
        let dictionary = Self::parse_dictionary(dict_content)?;
        let backend = build_backend(model_bytes, config.num_threads)?;
        Ok(Self {
            backend,
            dictionary,
            config,
        })
    }

    /// Parse character dictionary from string.
    ///
    /// PaddleOCR convention: model output index 0 is the CTC blank token,
    /// and dictionary characters map to indices 1..N. We insert a blank
    /// placeholder at index 0 so that `dictionary[model_index]` gives
    /// the correct character.
    ///
    /// PaddleOCR also emits a **space** as its last class. Native
    /// provisioning (`AutoExtractor::prefetch_models`) appends a
    /// trailing-space line to the dict file so that class is decodable.
    /// The `wasm32` build, however, receives dict *bytes* directly from
    /// the host (no filesystem / no `prefetch_models` post-processing —
    /// #524), so guarantee the space class here instead: append a
    /// trailing space unless the dict already ends with one. Idempotent
    /// (native dicts already end with `" "` → no-op) and safe for models
    /// without a space class (the extra index is simply never the
    /// arg-max). Without this, every inter-word space is dropped and the
    /// text runs together (empirically confirmed, #524 task 5).
    fn parse_dictionary(content: &str) -> OcrResult<Vec<char>> {
        let chars: Vec<char> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| line.chars().next())
            .collect();

        if chars.is_empty() {
            return Err(OcrError::DictionaryError("Dictionary is empty".to_string()));
        }

        // Prepend blank character at index 0 (PaddleOCR CTC blank convention)
        let mut dict = Vec::with_capacity(chars.len() + 2);
        dict.push('\0'); // index 0 = CTC blank
        dict.extend(chars);
        if dict.last() != Some(&' ') {
            dict.push(' '); // PaddleOCR space class (last)
        }

        Ok(dict)
    }

    /// Recognize text from a single cropped text region.
    ///
    /// # Arguments
    ///
    /// * `crop` - Cropped image of a text region
    ///
    /// # Returns
    ///
    /// Recognition result with text, confidence, and per-character scores.
    pub fn recognize(&self, crop: &DynamicImage) -> OcrResult<RecognitionResult> {
        // Preprocess the crop
        let input_tensor = preprocess_for_recognition(crop, self.config.rec_target_height)?;

        // Run inference
        self.run_inference(&input_tensor)
    }

    /// Recognize text from multiple cropped regions (batched).
    ///
    /// # Arguments
    ///
    /// * `crops` - Vector of cropped text region images
    ///
    /// # Returns
    ///
    /// Vector of recognition results.
    pub fn recognize_batch(&self, crops: &[DynamicImage]) -> OcrResult<Vec<RecognitionResult>> {
        // For now, process sequentially
        // TODO: Implement true batch processing for better performance
        crops.iter().map(|crop| self.recognize(crop)).collect()
    }

    /// Run model inference through the configured backend.
    fn run_inference(&self, input: &Array4<f32>) -> OcrResult<RecognitionResult> {
        // SVTR outputs `[N, T, C]` (or `[T, C]`) softmax scores.
        let output_array = self.backend.run(input)?;
        // Decode using CTC greedy decoding.
        self.ctc_greedy_decode(&output_array.view())
    }

    /// CTC greedy decoding.
    ///
    /// Takes softmax output [N, T, C] and produces text by:
    /// 1. Taking argmax at each timestep
    /// 2. Removing consecutive duplicates
    /// 3. Removing blank tokens
    fn ctc_greedy_decode(&self, output: &ndarray::ArrayViewD<f32>) -> OcrResult<RecognitionResult> {
        let shape = output.shape();

        // Handle different output shapes
        let (seq_len, num_classes) = match shape.len() {
            2 => (shape[0], shape[1]),
            3 => (shape[1], shape[2]),
            _ => {
                return Err(OcrError::InferenceError(format!(
                    "Unexpected output shape: {:?}, expected 2D or 3D tensor",
                    shape
                )));
            },
        };

        let blank_idx = 0; // PaddleOCR CTC blank is always index 0
        let mut text = String::new();
        let mut char_confidences = Vec::new();
        let mut prev_idx = blank_idx;

        for t in 0..seq_len {
            // Find argmax and max confidence for this timestep
            let mut max_idx = 0;
            let mut max_conf = f32::MIN;

            for c in 0..num_classes {
                let prob = if shape.len() == 3 {
                    output[[0, t, c]]
                } else {
                    output[[t, c]]
                };

                if prob > max_conf {
                    max_conf = prob;
                    max_idx = c;
                }
            }

            // Skip if same as previous or blank
            if max_idx != prev_idx && max_idx != blank_idx && max_idx < self.dictionary.len() {
                let ch = self.dictionary[max_idx];
                if ch != '\0' {
                    text.push(ch);
                    char_confidences.push(max_conf);
                }
            }

            prev_idx = max_idx;
        }

        // Calculate overall confidence as geometric mean of character confidences
        let confidence = if char_confidences.is_empty() {
            0.0
        } else {
            let log_sum: f32 = char_confidences.iter().map(|c| c.ln()).sum();
            (log_sum / char_confidences.len() as f32).exp()
        };

        Ok(RecognitionResult {
            text,
            confidence,
            char_confidences,
        })
    }

    /// Get the character dictionary.
    pub fn dictionary(&self) -> &[char] {
        &self.dictionary
    }

    /// Check if a model is loaded. A `TextRecognizer` cannot be
    /// constructed without a successfully built backend, so this is
    /// always `true` once the value exists (kept for API stability).
    pub fn is_loaded(&self) -> bool {
        true
    }
}

// `TextRecognizer` is `Send + Sync`: the backend trait object is bound
// `Send + Sync` and all other fields are.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dictionary() {
        let dict_content = "a\nb\nc\n1\n2\n3";
        let dict = TextRecognizer::parse_dictionary(dict_content).unwrap();

        // 1 blank + 6 chars + appended PaddleOCR space class (#524).
        assert_eq!(dict.len(), 8);
        assert_eq!(dict[0], '\0'); // Blank at index 0 (PaddleOCR CTC convention)
        assert_eq!(dict[1], 'a');
        assert_eq!(dict[6], '3');
        assert_eq!(dict[7], ' '); // space is the last class
    }

    #[test]
    fn test_parse_dictionary_space_class_is_idempotent() {
        // A dict that already ends with a lone-space line (how native
        // `prefetch_models` writes it) must NOT get a second space —
        // otherwise the dict is one class too long and every output
        // index is shifted, garbling all text (#524 task 5).
        let with_space = TextRecognizer::parse_dictionary("a\nb\n ").unwrap();
        assert_eq!(with_space, vec!['\0', 'a', 'b', ' ']);

        // A raw dict with no space line (how the wasm host supplies
        // bytes) gets exactly one space appended so the space class is
        // decodable and inter-word spaces survive.
        let no_space = TextRecognizer::parse_dictionary("a\nb").unwrap();
        assert_eq!(no_space, vec!['\0', 'a', 'b', ' ']);
    }

    #[test]
    fn test_parse_dictionary_empty() {
        let result = TextRecognizer::parse_dictionary("");
        assert!(result.is_err());
    }

    #[test]
    fn test_recognition_result() {
        let result = RecognitionResult {
            text: "Hello".to_string(),
            confidence: 0.95,
            char_confidences: vec![0.99, 0.98, 0.92, 0.93, 0.95],
        };

        assert_eq!(result.text, "Hello");
        assert!(result.confidence > 0.9);
        assert_eq!(result.char_confidences.len(), 5);
    }
}
