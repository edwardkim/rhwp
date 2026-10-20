//! Unified configuration for the text extraction pipeline.
//!
//! This module consolidates configuration that was previously scattered across:
//! - TextExtractionConfig
//! - SpanMergingConfig
//! - SpacingConfig
//! - ConversionOptions

/// Logging detail level for extraction pipeline.
///
/// Controls the verbosity of logging output during text extraction.
/// When the `logging` feature is enabled, logging is written to stderr.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// Only critical errors are logged
    Error,
    /// Warnings and errors are logged
    Warn,
    /// General information (default level)
    #[default]
    Info,
    /// Detailed debug information for troubleshooting
    Debug,
    /// Very detailed trace information (character-level details)
    Trace,
}

/// Document type classification for optimized extraction settings.
///
/// Different document types have different characteristics that benefit from
/// tuned extraction parameters:
/// - Academic: Dense text, special characters, equations
/// - Business: Tables, formal structure, headers/footers
/// - Novel: Long narrative, simple formatting, chapters
/// - CJK: Chinese/Japanese/Korean text with special boundary rules
/// - RTL: Arabic/Hebrew with right-to-left text flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    /// Academic papers, theses, technical documents
    /// Characteristics: Dense text, many special characters, equations, citations
    /// Optimizations: Aggressive hyphenation, strict spacing, preserve formatting
    Academic,

    /// Business documents, reports, contracts
    /// Characteristics: Formal structure, tables, headers/footers, multi-column
    /// Optimizations: Table detection, header preservation, justified text
    Business,

    /// Novels, books, creative content
    /// Characteristics: Long narrative, simple formatting, chapters
    /// Optimizations: Relaxed spacing, chapter detection, paragraph preservation
    Novel,

    /// CJK documents: Chinese, Japanese, Korean
    /// Characteristics: Different word boundaries, no spaces, special punctuation
    /// Optimizations: CJK-aware boundaries, density-adaptive scoring, custom punctuation
    Cjk,

    /// RTL documents: Arabic, Hebrew
    /// Characteristics: Right-to-left text flow, special diacritics, ligatures
    /// Optimizations: RTL detection, diacritic handling, ligature support
    Rtl,

    /// Generic/unknown documents - balanced defaults
    Generic,
}

impl DocumentType {
    /// Create optimized TextPipelineConfig for this document type
    pub fn create_config(&self) -> TextPipelineConfig {
        match self {
            Self::Academic => Self::academic_config(),
            Self::Business => Self::business_config(),
            Self::Novel => Self::novel_config(),
            Self::Cjk => Self::cjk_config(),
            Self::Rtl => Self::rtl_config(),
            Self::Generic => TextPipelineConfig::default(),
        }
    }

    fn academic_config() -> TextPipelineConfig {
        TextPipelineConfig {
            spacing: SpacingConfig { word_margin: 0.1 },
            tj_threshold: TjThresholdConfig {
                space_insertion_threshold: -120.0,
                use_adaptive: true, // Strict spacing
            },
            reading_order: ReadingOrderConfig {
                strategy: ReadingOrderStrategyType::StructureTreeFirst,
            },
            output: OutputConfig {
                detect_headings: false,
                include_images: false,
                bold_marker_behavior: BoldMarkerBehavior::Conservative,
                preserve_layout: true, // Preserve formatting
                extract_tables: true,  // Detect tables
                image_output_dir: None,
                embed_images: true,
                include_form_fields: true,
                include_artifacts: true,
            },
            word_boundary_mode: WordBoundaryMode::Primary,
            enable_hyphenation_reconstruction: true, // Aggressive hyphenation
            log_level: LogLevel::Info,
            collect_metrics: false,
        }
    }

    fn business_config() -> TextPipelineConfig {
        TextPipelineConfig {
            spacing: SpacingConfig { word_margin: 0.1 },
            tj_threshold: TjThresholdConfig {
                space_insertion_threshold: -120.0,
                use_adaptive: true, // Strict boundaries
            },
            reading_order: ReadingOrderConfig {
                strategy: ReadingOrderStrategyType::XYCut,
            },
            output: OutputConfig {
                detect_headings: false,
                include_images: false,
                bold_marker_behavior: BoldMarkerBehavior::Conservative,
                preserve_layout: true, // Headers/footers matter
                extract_tables: true,  // Essential for reports
                image_output_dir: None,
                embed_images: true,
                include_form_fields: true,
                include_artifacts: true,
            },
            word_boundary_mode: WordBoundaryMode::Primary,
            enable_hyphenation_reconstruction: true,
            log_level: LogLevel::Info,
            collect_metrics: false,
        }
    }

    fn novel_config() -> TextPipelineConfig {
        TextPipelineConfig {
            spacing: SpacingConfig { word_margin: 0.15 }, // Slightly relaxed
            tj_threshold: TjThresholdConfig {
                space_insertion_threshold: -100.0, // Relaxed
                use_adaptive: false,
            },
            reading_order: ReadingOrderConfig {
                strategy: ReadingOrderStrategyType::Simple,
            },
            output: OutputConfig {
                detect_headings: false,
                include_images: false,
                bold_marker_behavior: BoldMarkerBehavior::Conservative,
                preserve_layout: false, // Minimal formatting
                extract_tables: false,
                image_output_dir: None,
                embed_images: true,
                include_form_fields: true,
                include_artifacts: true,
            },
            word_boundary_mode: WordBoundaryMode::Tiebreaker,
            enable_hyphenation_reconstruction: true, // Essential for novels
            log_level: LogLevel::Info,
            collect_metrics: false,
        }
    }

    fn cjk_config() -> TextPipelineConfig {
        TextPipelineConfig {
            spacing: SpacingConfig { word_margin: 0.05 }, // Tighter for CJK
            tj_threshold: TjThresholdConfig {
                space_insertion_threshold: -80.0, // CJK-aware
                use_adaptive: true,
            },
            reading_order: ReadingOrderConfig {
                strategy: ReadingOrderStrategyType::StructureTreeFirst,
            },
            output: OutputConfig {
                detect_headings: false,
                include_images: false,
                bold_marker_behavior: BoldMarkerBehavior::Conservative,
                preserve_layout: true,
                extract_tables: true,
                image_output_dir: None,
                embed_images: true,
                include_form_fields: true,
                include_artifacts: true,
            },
            word_boundary_mode: WordBoundaryMode::Primary,
            enable_hyphenation_reconstruction: false, // Not applicable to CJK
            log_level: LogLevel::Info,
            collect_metrics: false,
        }
    }

    fn rtl_config() -> TextPipelineConfig {
        TextPipelineConfig {
            spacing: SpacingConfig { word_margin: 0.1 },
            tj_threshold: TjThresholdConfig {
                space_insertion_threshold: -120.0,
                use_adaptive: true,
            },
            reading_order: ReadingOrderConfig {
                strategy: ReadingOrderStrategyType::StructureTreeFirst,
            },
            output: OutputConfig {
                detect_headings: false,
                include_images: false,
                bold_marker_behavior: BoldMarkerBehavior::Conservative,
                preserve_layout: true,
                extract_tables: true,
                image_output_dir: None,
                embed_images: true,
                include_form_fields: true,
                include_artifacts: true,
            },
            word_boundary_mode: WordBoundaryMode::Tiebreaker,
            enable_hyphenation_reconstruction: false, // Different rules
            log_level: LogLevel::Info,
            collect_metrics: false,
        }
    }

    /// Detect document type from text sample
    ///
    /// Examines first 1000 characters to classify document
    pub fn detect_from_sample(sample: &str) -> Self {
        if sample.is_empty() {
            return Self::Generic;
        }

        let cjk_ratio = Self::count_cjk_chars(sample) as f32 / sample.len() as f32;
        let rtl_ratio = Self::count_rtl_chars(sample) as f32 / sample.len() as f32;
        let special_ratio = Self::count_special_chars(sample) as f32 / sample.len() as f32;

        // CJK if >10% of text is CJK characters
        if cjk_ratio > 0.1 {
            return Self::Cjk;
        }

        // RTL if >20% of text is RTL characters
        if rtl_ratio > 0.2 {
            return Self::Rtl;
        }

        // Check business patterns first
        if Self::looks_like_business(sample) {
            return Self::Business;
        }

        // Academic if high special character count (equations, citations, etc.)
        if special_ratio >= 0.08 {
            return Self::Academic;
        }

        // Novel if mostly lowercase ASCII with few numbers
        if Self::looks_like_narrative(sample) {
            return Self::Novel;
        }

        Self::Generic
    }

    fn count_cjk_chars(text: &str) -> usize {
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                matches!(
                    code,
                    0x3040..=0x309F   // Hiragana
                    | 0x30A0..=0x30FF // Katakana
                    | 0x3400..=0x4DBF // CJK Ext A
                    | 0x4E00..=0x9FFF // CJK
                    | 0xAC00..=0xD7AF // Hangul (Korean)
                )
            })
            .count()
    }

    fn count_rtl_chars(text: &str) -> usize {
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                matches!(
                    code,
                    0x0590..=0x05FF   // Hebrew
                    | 0x0600..=0x06FF // Arabic
                    | 0x0750..=0x077F // Arabic Supplement
                )
            })
            .count()
    }

    fn count_special_chars(text: &str) -> usize {
        text.chars()
            .filter(|c| {
                matches!(
                    *c,
                    '©' | '®'
                        | '™'
                        | '§'
                        | '¶'
                        | '†'
                        | '‡'
                        | '€'
                        | '£'
                        | '¥'
                        | '¢'
                        | '±'
                        | '×'
                        | '÷'
                        | '√'
                        | '∞'
                        | '∫'
                        | '←'
                        | '→'
                        | '↑'
                        | '↓'
                        | '°'
                        | '′'
                        | '″'
                )
            })
            .count()
    }

    fn looks_like_narrative(text: &str) -> bool {
        // Narrative has more lowercase than uppercase, many common words,
        // and contains typical narrative punctuation and sentence structures
        let lower_count = text.chars().filter(|c| c.is_lowercase()).count();
        let upper_count = text.chars().filter(|c| c.is_uppercase()).count();
        let digit_count = text.chars().filter(|c| c.is_ascii_digit()).count();

        // Count sentence-like patterns
        let period_count = text.matches('.').count();
        let has_narrative_words = text.contains("was ")
            || text.contains("were ")
            || text.contains("walked ")
            || text.contains("said ")
            || text.contains("went ");

        lower_count > upper_count * 5
            && digit_count < text.len() / 20
            && (has_narrative_words || period_count > 2)
    }

    fn looks_like_business(text: &str) -> bool {
        // Business has structured patterns, key terms, tables
        text.contains("Table")
            || text.contains("Figure")
            || text.contains("report")
            || text.contains("document")
            || text.contains("agreement")
    }
}

impl LogLevel {
    /// Check if a message at the given level should be logged.
    ///
    /// This is used internally by logging macros to determine whether
    /// to actually emit log output.
    pub fn should_log(&self, level: LogLevel) -> bool {
        match (*self, level) {
            (Self::Error, Self::Error) => true,
            (Self::Warn, Self::Error | Self::Warn) => true,
            (Self::Info, Self::Error | Self::Warn | Self::Info) => true,
            (Self::Debug, Self::Error | Self::Warn | Self::Info | Self::Debug) => true,
            (Self::Trace, _) => true,
            _ => false,
        }
    }
}

/// Word boundary detection mode for TJ array processing.
///
/// Per ISO 32000-1:2008 Section 9.4.4, word boundaries can be detected
/// using multiple signals (TJ offsets, geometric gaps, character properties).
///
/// Tiebreaker mode (default): Uses WordBoundaryDetector only when TJ offset
/// and geometric signals contradict each other (backward compatible).
///
/// Primary mode: Uses WordBoundaryDetector to detect boundaries BEFORE creating
/// TextSpans, partitioning the tj_character_array into word-level clusters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordBoundaryMode {
    /// Use WordBoundaryDetector only as tiebreaker (backward compatible, default)
    #[default]
    Tiebreaker,
    /// Use WordBoundaryDetector as primary detector before span creation
    Primary,
}

/// Unified configuration for the text extraction pipeline.
///
/// This replaces the scattered configuration across multiple modules
/// and provides a single configuration point for the entire pipeline.
#[derive(Debug, Clone)]
pub struct TextPipelineConfig {
    /// Spacing configuration for span merging
    pub spacing: SpacingConfig,

    /// TJ offset threshold configuration
    pub tj_threshold: TjThresholdConfig,

    /// Reading order strategy to use
    pub reading_order: ReadingOrderConfig,

    /// Output formatting options
    pub output: OutputConfig,

    /// Word boundary detection mode for TJ array processing (Phase 9.2)
    pub word_boundary_mode: WordBoundaryMode,

    /// Enable hyphenation reconstruction
    /// When true, hyphenated words at line breaks are reconstructed
    pub enable_hyphenation_reconstruction: bool,

    /// Logging detail level for the extraction pipeline
    pub log_level: LogLevel,

    /// Enable metrics collection during extraction
    pub collect_metrics: bool,
}

impl Default for TextPipelineConfig {
    fn default() -> Self {
        Self {
            spacing: SpacingConfig::default(),
            tj_threshold: TjThresholdConfig::default(),
            reading_order: ReadingOrderConfig::default(),
            output: OutputConfig::default(),
            word_boundary_mode: WordBoundaryMode::default(),
            enable_hyphenation_reconstruction: true,
            log_level: LogLevel::default(),
            collect_metrics: false,
        }
    }
}

impl TextPipelineConfig {
    /// Create config for a specific document type
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::pipeline::config::{TextPipelineConfig, DocumentType};
    ///
    /// let config = TextPipelineConfig::for_document_type(DocumentType::Academic);
    /// ```
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        doc_type.create_config()
    }

    /// Detect document type from text sample and create optimized config
    ///
    /// Analyzes the first portion of text to classify the document type,
    /// then returns a configuration optimized for that type.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::pipeline::config::TextPipelineConfig;
    ///
    /// let sample = "これは日本語です。";
    /// let config = TextPipelineConfig::detect_and_optimize(sample);
    /// ```
    pub fn detect_and_optimize(sample: &str) -> Self {
        let doc_type = DocumentType::detect_from_sample(sample);
        doc_type.create_config()
    }

    /// Create config with pdfplumber-compatible defaults.
    ///
    /// Uses word_margin = 0.1 and simple reading order.
    pub fn pdfplumber_compatible() -> Self {
        Self {
            spacing: SpacingConfig { word_margin: 0.1 },
            tj_threshold: TjThresholdConfig {
                space_insertion_threshold: -120.0,
                use_adaptive: false,
            },
            reading_order: ReadingOrderConfig {
                strategy: ReadingOrderStrategyType::Simple,
            },
            output: OutputConfig::default(),
            word_boundary_mode: WordBoundaryMode::Tiebreaker,
            enable_hyphenation_reconstruction: true,
            log_level: LogLevel::default(),
            collect_metrics: false,
        }
    }

    /// Create from legacy ConversionOptions for backwards compatibility.
    ///
    /// Maps old `ConversionOptions` to new `TextPipelineConfig` to maintain
    /// backwards compatibility while migrating to the new pipeline architecture.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::converters::ConversionOptions;
    /// use pdf_oxide::pipeline::config::TextPipelineConfig;
    ///
    /// let options = ConversionOptions::default();
    /// let config = TextPipelineConfig::from_conversion_options(&options);
    /// ```
    pub fn from_conversion_options(opts: &crate::converters::ConversionOptions) -> Self {
        use crate::converters::BoldMarkerBehavior as OldBMB;
        use crate::converters::ReadingOrderMode;

        // Map reading order mode to strategy type
        let strategy = match &opts.reading_order_mode {
            ReadingOrderMode::TopToBottomLeftToRight => ReadingOrderStrategyType::Simple,
            ReadingOrderMode::ColumnAware => ReadingOrderStrategyType::XYCut,
            ReadingOrderMode::StructureTreeFirst { .. } => {
                ReadingOrderStrategyType::StructureTreeFirst
            },
        };

        // Map BoldMarkerBehavior enum (both have same variants)
        let bold_marker_behavior = match opts.bold_marker_behavior {
            OldBMB::Aggressive => BoldMarkerBehavior::Aggressive,
            OldBMB::Conservative => BoldMarkerBehavior::Conservative,
        };

        Self {
            spacing: SpacingConfig::default(),
            tj_threshold: TjThresholdConfig::default(),
            reading_order: ReadingOrderConfig { strategy },
            output: OutputConfig {
                detect_headings: opts.detect_headings,
                include_images: opts.include_images,
                bold_marker_behavior,
                preserve_layout: opts.preserve_layout,
                extract_tables: opts.extract_tables,
                image_output_dir: opts.image_output_dir.clone(),
                embed_images: opts.embed_images,
                include_form_fields: opts.include_form_fields,
                include_artifacts: opts.include_artifacts,
            },
            word_boundary_mode: WordBoundaryMode::Tiebreaker, // Keep old behavior compatible
            enable_hyphenation_reconstruction: true,
            log_level: LogLevel::default(),
            collect_metrics: false,
        }
    }

    /// Set the word boundary detection mode (Phase 9.2)
    pub fn with_word_boundary_mode(mut self, mode: WordBoundaryMode) -> Self {
        self.word_boundary_mode = mode;
        self
    }

    /// Set whether to enable hyphenation reconstruction.
    ///
    /// When enabled, hyphenated words at line breaks are reconstructed
    /// (e.g., "govern-" + "ment" becomes "government").
    ///
    /// # Arguments
    ///
    /// * `enabled` - true to enable hyphenation reconstruction (default)
    pub fn with_hyphenation_reconstruction(mut self, enabled: bool) -> Self {
        self.enable_hyphenation_reconstruction = enabled;
        self
    }

    /// Set the logging detail level for the extraction pipeline.
    ///
    /// # Arguments
    ///
    /// * `level` - The logging level to use
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::pipeline::config::{TextPipelineConfig, LogLevel};
    ///
    /// let config = TextPipelineConfig::default()
    ///     .with_log_level(LogLevel::Debug);
    /// ```
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Enable metrics collection during extraction.
    ///
    /// When enabled, the extraction pipeline will collect detailed metrics
    /// about the extraction process for quality tracking and analysis.
    ///
    /// # Arguments
    ///
    /// * `enabled` - true to enable metrics collection
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::pipeline::config::TextPipelineConfig;
    ///
    /// let config = TextPipelineConfig::default()
    ///     .with_metrics_collection(true);
    /// ```
    pub fn with_metrics_collection(mut self, enabled: bool) -> Self {
        self.collect_metrics = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    #[test]
    fn test_log_level_should_log() {
        let info_level = LogLevel::Info;
        assert!(info_level.should_log(LogLevel::Error));
        assert!(info_level.should_log(LogLevel::Warn));
        assert!(info_level.should_log(LogLevel::Info));
        assert!(!info_level.should_log(LogLevel::Debug));
        assert!(!info_level.should_log(LogLevel::Trace));

        let debug_level = LogLevel::Debug;
        assert!(debug_level.should_log(LogLevel::Error));
        assert!(debug_level.should_log(LogLevel::Warn));
        assert!(debug_level.should_log(LogLevel::Info));
        assert!(debug_level.should_log(LogLevel::Debug));
        assert!(!debug_level.should_log(LogLevel::Trace));

        let trace_level = LogLevel::Trace;
        assert!(trace_level.should_log(LogLevel::Error));
        assert!(trace_level.should_log(LogLevel::Warn));
        assert!(trace_level.should_log(LogLevel::Info));
        assert!(trace_level.should_log(LogLevel::Debug));
        assert!(trace_level.should_log(LogLevel::Trace));
    }

    #[test]
    fn test_config_log_level_default() {
        let config = TextPipelineConfig::default();
        assert_eq!(config.log_level, LogLevel::Info);
    }

    #[test]
    fn test_config_with_log_level() {
        let config = TextPipelineConfig::default().with_log_level(LogLevel::Debug);
        assert_eq!(config.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_config_with_log_level_trace() {
        let config = TextPipelineConfig::default().with_log_level(LogLevel::Trace);
        assert_eq!(config.log_level, LogLevel::Trace);
    }

    #[test]
    fn test_config_with_log_level_error() {
        let config = TextPipelineConfig::default().with_log_level(LogLevel::Error);
        assert_eq!(config.log_level, LogLevel::Error);
    }

    #[test]
    fn test_pdfplumber_compatible_has_log_level() {
        let config = TextPipelineConfig::pdfplumber_compatible();
        assert_eq!(config.log_level, LogLevel::Info);
    }

    // Document type preset tests

    #[test]
    fn test_document_type_academic_config() {
        let config = DocumentType::Academic.create_config();
        assert!(config.enable_hyphenation_reconstruction);
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(config.output.preserve_layout);
        assert!(config.output.extract_tables);
        assert!(config.tj_threshold.use_adaptive);
    }

    #[test]
    fn test_document_type_business_config() {
        let config = DocumentType::Business.create_config();
        assert!(config.enable_hyphenation_reconstruction);
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(config.output.preserve_layout);
        assert!(config.output.extract_tables);
        assert_eq!(config.reading_order.strategy, ReadingOrderStrategyType::XYCut);
    }

    #[test]
    fn test_document_type_novel_config() {
        let config = DocumentType::Novel.create_config();
        assert!(config.enable_hyphenation_reconstruction);
        assert!(!config.output.preserve_layout);
        assert!(!config.output.extract_tables);
        assert_eq!(config.reading_order.strategy, ReadingOrderStrategyType::Simple);
        assert!(!config.tj_threshold.use_adaptive);
    }

    #[test]
    fn test_document_type_cjk_config() {
        let config = DocumentType::Cjk.create_config();
        assert!(!config.enable_hyphenation_reconstruction);
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(config.output.preserve_layout);
        assert!(config.output.extract_tables);
        assert!(config.tj_threshold.use_adaptive);
        assert_eq!(config.word_boundary_mode, WordBoundaryMode::Primary);
    }

    #[test]
    fn test_document_type_rtl_config() {
        let config = DocumentType::Rtl.create_config();
        assert!(!config.enable_hyphenation_reconstruction);
        assert!(config.output.preserve_layout);
        assert!(config.output.extract_tables);
        assert_eq!(config.word_boundary_mode, WordBoundaryMode::Tiebreaker);
    }

    #[test]
    fn test_document_type_generic_config() {
        let config = DocumentType::Generic.create_config();
        // Generic should match the default config structure
        assert_eq!(config.log_level, LogLevel::default());
        assert_eq!(config.word_boundary_mode, WordBoundaryMode::default());
        assert!(config.enable_hyphenation_reconstruction);
    }

    // Document type detection tests

    #[test]
    fn test_detect_empty_sample() {
        let doc_type = DocumentType::detect_from_sample("");
        assert_eq!(doc_type, DocumentType::Generic);
    }

    #[test]
    fn test_detect_cjk_sample() {
        let sample = "これは日本語です。This is bilingual text.";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Cjk);
    }

    #[test]
    fn test_detect_cjk_chinese() {
        let sample = "这是中文文本。";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Cjk);
    }

    #[test]
    fn test_detect_cjk_korean() {
        let sample = "이것은 한국어 텍스트입니다.";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Cjk);
    }

    #[test]
    fn test_detect_rtl_sample() {
        let sample = "مرحبا بك في النص العربي";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Rtl);
    }

    #[test]
    fn test_detect_rtl_hebrew() {
        let sample = "זה טקסט בעברית";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Rtl);
    }

    #[test]
    fn test_detect_academic_sample() {
        // Sample has enough special chars for academic detection (>= 0.08 ratio)
        let sample =
            "The ∫∞√∑ equations © research shows ± evidence × mathematical ÷ concepts ® article";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Academic);
    }

    #[test]
    fn test_detect_academic_with_symbols() {
        let sample = "Consider the integral ∫ from a to b and the summation ∑ with limit n → ∞ © 2024 ® ± × ÷ √ Author";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Academic);
    }

    #[test]
    fn test_detect_novel_sample() {
        let sample = "The quick brown fox jumps over the lazy dog. She walked through the forest, listening to the birds singing their morning songs.";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Novel);
    }

    #[test]
    fn test_detect_novel_narrative() {
        let sample = "Once upon a time, there was a kingdom far away. The princess walked through the castle gardens every morning, admiring the flowers and trees.";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Novel);
    }

    #[test]
    fn test_detect_business_sample() {
        let sample = "Table 1 shows the results. Figure 2 displays the report findings. The document contains the agreement terms with key provisions.";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Business);
    }

    #[test]
    fn test_detect_generic_mixed_text() {
        let sample = "ABC DEF GHI JKL MNO PQR STU VWX YZ are letters. Numbers like 1234567890 appear here too.";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Generic);
    }

    // Builder methods tests

    #[test]
    fn test_for_document_type_builder() {
        let config = TextPipelineConfig::for_document_type(DocumentType::Business);
        assert!(config.enable_hyphenation_reconstruction);
        assert!(config.output.extract_tables);
    }

    #[test]
    fn test_detect_and_optimize() {
        let sample = "これは日本語です。";
        let config = TextPipelineConfig::detect_and_optimize(sample);
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(!config.enable_hyphenation_reconstruction); // CJK config
    }

    #[test]
    fn test_for_document_type_academic() {
        let config = TextPipelineConfig::for_document_type(DocumentType::Academic);
        assert!(config.tj_threshold.use_adaptive);
        assert_eq!(config.word_boundary_mode, WordBoundaryMode::Primary);
    }

    #[test]
    fn test_for_document_type_cjk_spacing() {
        let config = TextPipelineConfig::for_document_type(DocumentType::Cjk);
        // CJK should have tighter spacing
        assert!(config.spacing.word_margin < 0.1);
    }

    #[test]
    fn test_for_document_type_novel_spacing() {
        let config = TextPipelineConfig::for_document_type(DocumentType::Novel);
        // Novel should have relaxed spacing
        assert!(config.spacing.word_margin > 0.1);
    }

    #[test]
    fn test_detect_sample_with_high_cjk_ratio() {
        let sample = "これはひらがなですあ。カタカナです。テスト。";
        let doc_type = DocumentType::detect_from_sample(sample);
        assert_eq!(doc_type, DocumentType::Cjk);
    }

    #[test]
    fn test_detect_sample_with_low_cjk_ratio() {
        let sample = "This is mostly English text with some 日本語 mixed in.";
        let doc_type = DocumentType::detect_from_sample(sample);
        // Should be Generic since CJK ratio is too low
        assert_ne!(doc_type, DocumentType::Cjk);
    }

    #[test]
    fn test_count_cjk_chars() {
        let text = "これは日本語です";
        let count = DocumentType::count_cjk_chars(text);
        assert!(count > 0);
    }

    #[test]
    fn test_count_rtl_chars() {
        let text = "مرحبا بك";
        let count = DocumentType::count_rtl_chars(text);
        assert!(count > 0);
    }

    #[test]
    fn test_count_special_chars() {
        let text = "Equation: ∫√∞ with © symbol";
        let count = DocumentType::count_special_chars(text);
        assert!(count > 0);
    }

    #[test]
    fn test_looks_like_narrative() {
        let text = "she was running through the forest. she walked past the trees. they said hello to her.";
        assert!(DocumentType::looks_like_narrative(text));
    }

    #[test]
    fn test_looks_not_like_narrative_high_digits() {
        let text = "1234567890 ABC DEF GHIJ 1234567890 KLMN";
        assert!(!DocumentType::looks_like_narrative(text));
    }

    #[test]
    fn test_looks_like_business() {
        let text = "This Table shows the Figure in our report and document with agreement details";
        assert!(DocumentType::looks_like_business(text));
    }

    #[test]
    fn test_looks_not_like_business() {
        let text = "This is a simple story about a dog and a cat in the forest";
        assert!(!DocumentType::looks_like_business(text));
    }

    // Metrics collection tests

    #[test]
    fn test_collect_metrics_default_disabled() {
        let config = TextPipelineConfig::default();
        assert!(!config.collect_metrics);
    }

    #[test]
    fn test_collect_metrics_enabled() {
        let config = TextPipelineConfig::default().with_metrics_collection(true);
        assert!(config.collect_metrics);
    }

    #[test]
    fn test_collect_metrics_disabled_explicitly() {
        let config = TextPipelineConfig::default().with_metrics_collection(false);
        assert!(!config.collect_metrics);
    }

    #[test]
    fn test_collect_metrics_builder_chain() {
        let config = TextPipelineConfig::default()
            .with_log_level(LogLevel::Debug)
            .with_metrics_collection(true);
        assert!(config.collect_metrics);
        assert_eq!(config.log_level, LogLevel::Debug);
    }
}

/// Configuration for geometric spacing decisions.
///
/// Controls how gaps between text spans are interpreted.
#[derive(Debug, Clone, Copy)]
pub struct SpacingConfig {
    /// Word margin as ratio of character size.
    ///
    /// If the gap between spans is less than this ratio times the average
    /// character width, they are considered part of the same word.
    ///
    /// Default: 0.1 (pdfplumber default)
    pub word_margin: f32,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self { word_margin: 0.1 }
    }
}

/// Configuration for TJ offset threshold.
///
/// Controls how TJ array offsets are interpreted for word boundary detection.
#[derive(Debug, Clone)]
pub struct TjThresholdConfig {
    /// Static threshold for TJ offsets (in text space units).
    ///
    /// Offsets more negative than this value indicate a word boundary.
    /// Default: -120.0 (pdfplumber default)
    pub space_insertion_threshold: f32,

    /// Whether to use adaptive threshold based on font metrics.
    ///
    /// When true, the threshold is computed dynamically based on
    /// the statistical distribution of gaps in the document.
    pub use_adaptive: bool,
}

impl Default for TjThresholdConfig {
    fn default() -> Self {
        Self {
            space_insertion_threshold: -120.0,
            use_adaptive: false,
        }
    }
}

/// Configuration for reading order strategy.
#[derive(Debug, Clone)]
pub struct ReadingOrderConfig {
    /// The reading order strategy to use for horizontal (left-to-right,
    /// top-to-bottom) pages.
    ///
    /// **Vertical-writing override.** When a page is detected as
    /// vertical-majority — at least half of the page's spans carry
    /// `wmode == 1` from a `/WMode 1` CMap or a `-V`-suffixed encoding
    /// (Identity-V, UniJIS-UTF16-V, …) — the configured strategy is
    /// bypassed and the page is routed through a tategaki-aware
    /// right-to-left column ordering instead. None of the four
    /// horizontal strategies (`Simple`, `Geometric`, `XYCut`,
    /// `StructureTreeFirst`) can produce correct right-to-left column
    /// order for vertical writing, so the override is unconditional.
    /// Per-span `wmode` is preserved on every output span regardless of
    /// which path the page took, so downstream consumers can still
    /// distinguish the two modes.
    pub strategy: ReadingOrderStrategyType,
}

impl Default for ReadingOrderConfig {
    fn default() -> Self {
        Self {
            strategy: ReadingOrderStrategyType::StructureTreeFirst,
        }
    }
}

/// Available reading order strategy types.
///
/// All four variants are horizontal (left-to-right, top-to-bottom)
/// strategies. Vertical-majority (tategaki) pages bypass the configured
/// variant entirely — see [`ReadingOrderConfig::strategy`] for the
/// override rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadingOrderStrategyType {
    /// Use structure tree MCIDs for reading order, fallback to geometric.
    ///
    /// This is the PDF-spec-compliant approach for Tagged PDFs
    /// (ISO 32000-1:2008 Section 14.7).
    StructureTreeFirst,

    /// Column-aware geometric analysis.
    ///
    /// Uses horizontal gap detection to identify columns and
    /// processes each column top-to-bottom.
    Geometric,

    /// Recursive XY-Cut spatial partitioning.
    ///
    /// Uses projection profiles to detect columns in complex layouts
    /// like newspapers and academic papers. More sophisticated than
    /// simple gap detection (ISO 32000-1:2008 Section 9.4).
    XYCut,

    /// Simple top-to-bottom, left-to-right ordering.
    ///
    /// Sorts spans by Y coordinate (descending) then X coordinate (ascending).
    Simple,
}

/// Output formatting configuration.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Whether to detect and format headings.
    ///
    /// When true, larger text is formatted as markdown headings.
    /// Default: false (disabled for PDF spec compliance)
    pub detect_headings: bool,

    /// Whether to include images in output.
    pub include_images: bool,

    /// Bold marker behavior for whitespace-only spans.
    pub bold_marker_behavior: BoldMarkerBehavior,

    /// Preserve document layout using whitespace (Markdown) or CSS positioning (HTML).
    ///
    /// When enabled:
    /// - Markdown converter preserves column alignment via whitespace
    /// - HTML converter uses CSS `position:absolute` for spatial preservation
    /// - Plain text converter preserves spacing
    ///
    /// Default: false
    pub preserve_layout: bool,

    /// Extract table structures and format as markdown tables or HTML tables.
    ///
    /// When enabled:
    /// - Grid-aligned text is detected and formatted as tables
    /// - Markdown converter outputs markdown table syntax
    /// - HTML converter outputs HTML table elements
    ///
    /// Default: false
    pub extract_tables: bool,

    /// Directory to save extracted images.
    ///
    /// When Some(path):
    /// - Images are extracted to the specified directory
    /// - Image references use the provided path
    ///
    /// When None:
    /// - Images are referenced but not extracted to disk
    ///
    /// Default: None
    pub image_output_dir: Option<String>,

    /// Embed images as base64 data URIs.
    ///
    /// When true (default):
    /// - HTML output embeds images as base64 data URIs
    /// - Creates self-contained HTML files
    ///
    /// When false:
    /// - Images are saved to `image_output_dir` and referenced by path
    /// - Markdown always uses file references (base64 not well supported)
    ///
    /// Default: true
    pub embed_images: bool,

    /// Include form field values inline in output.
    ///
    /// When true (default), form field values are merged with page content at their
    /// spatial positions. When false, form field values are omitted.
    ///
    /// Default: true
    pub include_form_fields: bool,

    /// Include spans tagged `/Artifact` (running headers/footers, page
    /// numbers, watermarks; ISO 32000-1:2008 §14.8.2.2.1) in the output.
    ///
    /// Default: true (backward compatibility; see
    /// [`crate::converters::ConversionOptions::include_artifacts`]).
    pub include_artifacts: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            detect_headings: false, // Disabled for spec compliance
            include_images: false,  // Off by default to avoid base64 bloat
            bold_marker_behavior: BoldMarkerBehavior::Conservative,
            preserve_layout: false,
            extract_tables: false,
            image_output_dir: None,
            embed_images: true,
            include_form_fields: true,
            include_artifacts: true,
        }
    }
}

/// Behavior for bold markers around whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoldMarkerBehavior {
    /// Don't apply bold markers to whitespace-only spans.
    ///
    /// This prevents patterns like "** **" in output.
    Conservative,

    /// Apply bold markers to any span with bold font weight.
    Aggressive,
}

impl Default for BoldMarkerBehavior {
    /// Default behavior is Conservative mode.
    fn default() -> Self {
        Self::Conservative
    }
}
