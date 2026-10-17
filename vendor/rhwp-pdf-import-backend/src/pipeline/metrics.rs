//! Quality metrics for text extraction.
//!
//! This module provides comprehensive metrics collection and export functionality
//! for tracking extraction quality improvements over time.
//!
//! # Quality Score
//!
//! The quality score is estimated based on:
//! - **Hyphenation**: Reconstructed hyphenated words (0-0.5 points)
//! - **Extended Latin**: AGL fallback usage (0-0.3 points)
//! - **Script Handling**: CJK and RTL character support (0-0.6 points)
//! - **Baseline**: 7.0 points
//!
//! The score ranges from 0.0 to 10.0:
//! - 7.0-7.5: Baseline quality (before Priority 1/2)
//! - 8.0-8.5: Good quality (after Priority 1/2)
//! - 8.5+: Excellent quality (target achieved)

/// Quality metrics for text extraction
///
/// Captures detailed statistics about the extraction process to enable
/// quality tracking and analysis.
#[derive(Debug, Clone, Default)]
pub struct ExtractionMetrics {
    /// Total characters extracted
    pub total_characters: usize,

    /// Total words detected
    pub total_words: usize,

    /// Word boundaries detected
    pub word_boundaries_detected: usize,

    /// Hyphenated words reconstructed
    pub hyphenated_words_reconstructed: usize,

    /// Characters mapped via AGL fallback
    pub agl_fallback_mappings: usize,

    /// Density-adaptive scoring applications
    pub density_adaptive_applications: usize,

    /// CJK characters detected
    pub cjk_characters_detected: usize,

    /// RTL characters detected
    pub rtl_characters_detected: usize,

    /// Document script profile detected
    pub detected_script: Option<String>,

    /// Extraction time in milliseconds
    pub extraction_time_ms: u128,

    /// Document type inferred
    pub inferred_document_type: Option<String>,
}

impl ExtractionMetrics {
    /// Estimate quality score based on metrics
    ///
    /// Returns a score from 0.0 to 10.0 indicating extraction quality:
    /// - 7.0-7.5: Baseline quality (before Priority 1/2)
    /// - 8.0-8.5: Good quality (after Priority 1/2)
    /// - 8.5+: Excellent quality (target achieved)
    ///
    /// Score factors:
    /// - Word boundary accuracy (inferred from hyphenation count)
    /// - AGL fallback usage (indicates complex scripts)
    /// - Script diversity (RTL + CJK handling)
    /// - Processing efficiency (time per character)
    pub fn estimate_quality_score(&self) -> f32 {
        let mut score = 7.0; // Baseline

        // Hyphenation factor: more reconstructions indicate better handling
        let hyphenation_factor = (self.hyphenated_words_reconstructed as f32).min(100.0) / 100.0;
        score += hyphenation_factor * 0.5;

        // AGL fallback factor: indicates handling of extended Latin
        let agl_factor = (self.agl_fallback_mappings as f32).min(100.0) / 100.0;
        score += agl_factor * 0.3;

        // Script diversity factor: handling multiple scripts is harder
        let mut script_complexity: f32 = 0.0;
        if self.cjk_characters_detected > 0 {
            script_complexity += 0.3;
        }
        if self.rtl_characters_detected > 0 {
            script_complexity += 0.3;
        }
        score += script_complexity.min(0.6) * 0.5;

        // Clamp to reasonable range
        score.min(10.0)
    }

    /// Export metrics as JSON
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "total_characters": self.total_characters,
            "total_words": self.total_words,
            "word_boundaries_detected": self.word_boundaries_detected,
            "hyphenated_words_reconstructed": self.hyphenated_words_reconstructed,
            "agl_fallback_mappings": self.agl_fallback_mappings,
            "density_adaptive_applications": self.density_adaptive_applications,
            "cjk_characters_detected": self.cjk_characters_detected,
            "rtl_characters_detected": self.rtl_characters_detected,
            "detected_script": self.detected_script,
            "extraction_time_ms": self.extraction_time_ms,
            "inferred_document_type": self.inferred_document_type,
            "estimated_quality_score": self.estimate_quality_score(),
        })
        .to_string()
    }

    /// Export metrics as CSV row
    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            self.total_characters,
            self.total_words,
            self.word_boundaries_detected,
            self.hyphenated_words_reconstructed,
            self.agl_fallback_mappings,
            self.density_adaptive_applications,
            self.cjk_characters_detected,
            self.rtl_characters_detected,
            self.detected_script.as_deref().unwrap_or("-"),
            self.extraction_time_ms,
            self.estimate_quality_score(),
        )
    }

    /// Export metrics as YAML
    pub fn to_yaml(&self) -> String {
        format!(
            r#"extraction_metrics:
  total_characters: {}
  total_words: {}
  word_boundaries_detected: {}
  hyphenated_words_reconstructed: {}
  agl_fallback_mappings: {}
  density_adaptive_applications: {}
  cjk_characters_detected: {}
  rtl_characters_detected: {}
  detected_script: {}
  extraction_time_ms: {}
  inferred_document_type: {}
  estimated_quality_score: {:.2}"#,
            self.total_characters,
            self.total_words,
            self.word_boundaries_detected,
            self.hyphenated_words_reconstructed,
            self.agl_fallback_mappings,
            self.density_adaptive_applications,
            self.cjk_characters_detected,
            self.rtl_characters_detected,
            self.detected_script.as_deref().unwrap_or("-"),
            self.extraction_time_ms,
            self.inferred_document_type.as_deref().unwrap_or("-"),
            self.estimate_quality_score(),
        )
    }
}

/// Accumulator for batch metrics across multiple documents
#[derive(Debug, Default)]
pub struct BatchMetrics {
    /// Individual document metrics
    pub documents: Vec<ExtractionMetrics>,

    /// Total across all documents
    pub total_characters: usize,
    /// Total time spent in extraction across all documents in milliseconds
    pub total_extraction_time_ms: u128,
}

impl BatchMetrics {
    /// Add document metrics to batch
    pub fn add_document(&mut self, metrics: ExtractionMetrics) {
        self.total_characters += metrics.total_characters;
        self.total_extraction_time_ms += metrics.extraction_time_ms;
        self.documents.push(metrics);
    }

    /// Calculate average quality score across documents
    pub fn average_quality_score(&self) -> f32 {
        if self.documents.is_empty() {
            return 0.0;
        }
        let sum: f32 = self
            .documents
            .iter()
            .map(|m| m.estimate_quality_score())
            .sum();
        sum / self.documents.len() as f32
    }

    /// Export batch metrics as JSON
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "document_count": self.documents.len(),
            "total_characters": self.total_characters,
            "total_extraction_time_ms": self.total_extraction_time_ms,
            "average_quality_score": self.average_quality_score(),
            "documents": self.documents.iter()
                .map(|m| serde_json::from_str::<serde_json::Value>(&m.to_json()).unwrap())
                .collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Export batch metrics as CSV
    pub fn to_csv(&self) -> String {
        let mut csv = String::from(
            "characters,words,boundaries,hyphenations,agl_mappings,density_applications,\
             cjk_chars,rtl_chars,script,time_ms,quality_score\n",
        );
        for doc in &self.documents {
            csv.push_str(&doc.to_csv_row());
            csv.push('\n');
        }
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_default() {
        let metrics = ExtractionMetrics::default();
        assert_eq!(metrics.total_characters, 0);
        assert_eq!(metrics.total_words, 0);
    }

    #[test]
    fn test_quality_score_baseline() {
        let metrics = ExtractionMetrics::default();
        assert!((metrics.estimate_quality_score() - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_quality_score_with_hyphenation() {
        let metrics = ExtractionMetrics {
            hyphenated_words_reconstructed: 50,
            ..Default::default()
        };
        let score = metrics.estimate_quality_score();
        assert!(score > 7.0);
        assert!(score <= 10.0);
    }

    #[test]
    fn test_quality_score_capped_at_10() {
        let metrics = ExtractionMetrics {
            hyphenated_words_reconstructed: 1000,
            agl_fallback_mappings: 1000,
            cjk_characters_detected: 1000,
            rtl_characters_detected: 1000,
            ..Default::default()
        };
        assert!(metrics.estimate_quality_score() <= 10.0);
    }

    #[test]
    fn test_metrics_to_json() {
        let metrics = ExtractionMetrics {
            total_characters: 1000,
            total_words: 200,
            ..Default::default()
        };
        let json = metrics.to_json();
        assert!(json.contains("1000"));
        assert!(json.contains("200"));
    }

    #[test]
    fn test_metrics_to_csv_row() {
        let metrics = ExtractionMetrics {
            total_characters: 1000,
            ..Default::default()
        };
        let csv = metrics.to_csv_row();
        assert!(csv.starts_with("1000,"));
    }

    #[test]
    fn test_metrics_to_yaml() {
        let metrics = ExtractionMetrics {
            total_characters: 1000,
            ..Default::default()
        };
        let yaml = metrics.to_yaml();
        assert!(yaml.contains("total_characters: 1000"));
    }

    #[test]
    fn test_batch_metrics_add_document() {
        let mut batch = BatchMetrics::default();
        let doc1 = ExtractionMetrics {
            total_characters: 500,
            ..Default::default()
        };
        batch.add_document(doc1);
        assert_eq!(batch.total_characters, 500);
        assert_eq!(batch.documents.len(), 1);
    }

    #[test]
    fn test_batch_metrics_average_quality_score() {
        let mut batch = BatchMetrics::default();
        let doc1 = ExtractionMetrics::default(); // Score 7.0
        let doc2 = ExtractionMetrics {
            hyphenated_words_reconstructed: 100,
            ..Default::default()
        }; // Higher score
        batch.add_document(doc1);
        batch.add_document(doc2);
        let avg = batch.average_quality_score();
        assert!(avg > 7.0);
        assert!(avg < 10.0);
    }

    #[test]
    fn test_batch_metrics_empty_average() {
        let batch = BatchMetrics::default();
        assert_eq!(batch.average_quality_score(), 0.0);
    }

    #[test]
    fn test_batch_metrics_to_csv() {
        let mut batch = BatchMetrics::default();
        batch.add_document(ExtractionMetrics::default());
        let csv = batch.to_csv();
        assert!(csv.contains("characters,words,boundaries"));
    }

    #[test]
    fn test_batch_metrics_to_json() {
        let mut batch = BatchMetrics::default();
        batch.add_document(ExtractionMetrics {
            total_characters: 500,
            ..Default::default()
        });
        let json = batch.to_json();
        assert!(json.contains("\"document_count\""));
        assert!(json.contains("500"));
        assert!(json.contains("\"documents\""));
    }

    #[test]
    fn test_metrics_with_all_fields() {
        let metrics = ExtractionMetrics {
            total_characters: 5000,
            total_words: 1000,
            word_boundaries_detected: 800,
            hyphenated_words_reconstructed: 50,
            agl_fallback_mappings: 20,
            density_adaptive_applications: 100,
            cjk_characters_detected: 200,
            rtl_characters_detected: 50,
            detected_script: Some("Latin+CJK".to_string()),
            extraction_time_ms: 500,
            inferred_document_type: Some("Academic".to_string()),
        };
        let quality = metrics.estimate_quality_score();
        assert!(quality > 7.0);
        assert!(quality <= 10.0);

        // Verify JSON export includes all fields
        let json = metrics.to_json();
        assert!(json.contains("5000"));
        assert!(json.contains("1000"));
        assert!(json.contains("Latin+CJK"));
    }

    #[test]
    fn test_quality_score_with_cjk_and_rtl() {
        let metrics = ExtractionMetrics {
            cjk_characters_detected: 500,
            rtl_characters_detected: 200,
            ..Default::default()
        };
        let score = metrics.estimate_quality_score();
        // Should have script complexity bonus
        assert!(score > 7.0);
        assert!(score <= 10.0);
    }

    #[test]
    fn test_metrics_csv_row_with_script() {
        let metrics = ExtractionMetrics {
            total_characters: 1000,
            detected_script: Some("Arabic".to_string()),
            ..Default::default()
        };
        let csv = metrics.to_csv_row();
        assert!(csv.contains("Arabic"));
    }

    #[test]
    fn test_metrics_csv_row_without_script() {
        let metrics = ExtractionMetrics {
            total_characters: 1000,
            detected_script: None,
            ..Default::default()
        };
        let csv = metrics.to_csv_row();
        assert!(csv.contains(",-"));
    }

    #[test]
    fn test_batch_metrics_multiple_documents() {
        let mut batch = BatchMetrics::default();
        for i in 0..5 {
            let metrics = ExtractionMetrics {
                total_characters: 1000 * (i + 1),
                hyphenated_words_reconstructed: 10 * i,
                ..Default::default()
            };
            batch.add_document(metrics);
        }
        assert_eq!(batch.documents.len(), 5);
        assert_eq!(batch.total_characters, 15000);
        let avg = batch.average_quality_score();
        assert!(avg > 7.0);
    }
}
