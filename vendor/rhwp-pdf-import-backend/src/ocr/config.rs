//! OCR configuration options.

use std::path::PathBuf;

/// Detection image resize strategy.
///
/// Controls how input images are resized before detection.
#[derive(Debug, Clone, PartialEq)]
pub enum DetResizeStrategy {
    /// Scale DOWN so the longest side fits within `max_side` (PP-OCRv3/v4 default).
    /// Good for mobile models that expect smaller inputs.
    MaxSide {
        /// Maximum side length (default: 960)
        max_side: u32,
    },
    /// Scale UP so the shortest side is at least `min_side`, but cap at `max_side_limit`
    /// (PP-OCRv5 default). Preserves high resolution for server models.
    MinSide {
        /// Minimum side length — images smaller than this are scaled up (default: 64)
        min_side: u32,
        /// Maximum allowed side length (default: 4000)
        max_side_limit: u32,
    },
}

impl Default for DetResizeStrategy {
    fn default() -> Self {
        // Default to v3/v4 strategy for backward compatibility
        DetResizeStrategy::MaxSide { max_side: 960 }
    }
}

/// Configuration for OCR processing.
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// Detection confidence threshold (0.0 - 1.0, default: 0.3)
    pub det_threshold: f32,

    /// Box confidence threshold for filtering (0.0 - 1.0, default: 0.6)
    pub box_threshold: f32,

    /// Recognition confidence threshold (0.0 - 1.0, default: 0.5)
    pub rec_threshold: f32,

    /// Maximum side length for detection input (default: 960).
    /// Ignored when `det_resize_strategy` is set; kept for backward compatibility.
    pub det_max_side: u32,

    /// Detection image resize strategy (default: MaxSide { max_side: 960 }).
    /// Use `MinSide` for PP-OCRv5 server models that benefit from high-resolution input.
    pub det_resize_strategy: DetResizeStrategy,

    /// Target height for recognition input (default: 48)
    pub rec_target_height: u32,

    /// Number of inference threads (default: 4)
    pub num_threads: usize,

    /// Unclip ratio for expanding detected boxes (default: 1.5)
    pub unclip_ratio: f32,

    /// Maximum number of text box candidates (default: 1000)
    pub max_candidates: usize,

    /// Enable style detection from OCR geometry (default: true)
    pub detect_styles: bool,

    /// Custom path to detection model (None = use bundled)
    pub det_model_path: Option<PathBuf>,

    /// Custom path to recognition model (None = use bundled)
    pub rec_model_path: Option<PathBuf>,

    /// Custom path to character dictionary (None = use bundled)
    pub dict_path: Option<PathBuf>,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            det_threshold: 0.3,
            box_threshold: 0.6,
            rec_threshold: 0.5,
            det_max_side: 960,
            det_resize_strategy: DetResizeStrategy::default(),
            rec_target_height: 48,
            num_threads: 4,
            unclip_ratio: 1.5,
            max_candidates: 1000,
            detect_styles: true,
            det_model_path: None,
            rec_model_path: None,
            dict_path: None,
        }
    }
}

impl OcrConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration tuned for PP-OCRv5 server models.
    ///
    /// Uses `MinSide` resize strategy to preserve high-resolution input,
    /// which is essential for PP-OCRv5 server detection models.
    pub fn v5() -> Self {
        Self {
            det_resize_strategy: DetResizeStrategy::MinSide {
                min_side: 64,
                max_side_limit: 4000,
            },
            box_threshold: 0.6,
            ..Default::default()
        }
    }

    /// Create a builder for custom configuration.
    pub fn builder() -> OcrConfigBuilder {
        OcrConfigBuilder::new()
    }
}

/// Builder for OcrConfig with fluent API.
#[derive(Debug, Clone, Default)]
pub struct OcrConfigBuilder {
    config: OcrConfig,
}

impl OcrConfigBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self {
            config: OcrConfig::default(),
        }
    }

    /// Set detection threshold.
    pub fn det_threshold(mut self, threshold: f32) -> Self {
        self.config.det_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set box confidence threshold.
    pub fn box_threshold(mut self, threshold: f32) -> Self {
        self.config.box_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set recognition threshold.
    pub fn rec_threshold(mut self, threshold: f32) -> Self {
        self.config.rec_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set maximum side length for detection (v3/v4 MaxSide strategy).
    ///
    /// This also sets `det_resize_strategy` to `MaxSide`.
    pub fn det_max_side(mut self, max_side: u32) -> Self {
        let max_side = max_side.max(32);
        self.config.det_max_side = max_side;
        self.config.det_resize_strategy = DetResizeStrategy::MaxSide { max_side };
        self
    }

    /// Set detection resize strategy.
    ///
    /// Use `DetResizeStrategy::MinSide` for PP-OCRv5 server models.
    pub fn det_resize_strategy(mut self, strategy: DetResizeStrategy) -> Self {
        self.config.det_resize_strategy = strategy;
        self
    }

    /// Set target height for recognition.
    pub fn rec_target_height(mut self, height: u32) -> Self {
        self.config.rec_target_height = height.max(16);
        self
    }

    /// Set number of inference threads.
    pub fn num_threads(mut self, threads: usize) -> Self {
        self.config.num_threads = threads.max(1);
        self
    }

    /// Set unclip ratio for box expansion.
    pub fn unclip_ratio(mut self, ratio: f32) -> Self {
        self.config.unclip_ratio = ratio.max(1.0);
        self
    }

    /// Set maximum number of text box candidates.
    pub fn max_candidates(mut self, max: usize) -> Self {
        self.config.max_candidates = max.max(1);
        self
    }

    /// Enable or disable style detection.
    pub fn detect_styles(mut self, detect: bool) -> Self {
        self.config.detect_styles = detect;
        self
    }

    /// Set custom detection model path.
    pub fn det_model_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.det_model_path = Some(path.into());
        self
    }

    /// Set custom recognition model path.
    pub fn rec_model_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.rec_model_path = Some(path.into());
        self
    }

    /// Set custom dictionary path.
    pub fn dict_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.dict_path = Some(path.into());
        self
    }

    /// Build the configuration.
    pub fn build(self) -> OcrConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OcrConfig::default();
        assert!((config.det_threshold - 0.3).abs() < f32::EPSILON);
        assert!((config.box_threshold - 0.6).abs() < f32::EPSILON);
        assert_eq!(config.num_threads, 4);
        assert!(config.detect_styles);
        assert_eq!(config.det_resize_strategy, DetResizeStrategy::MaxSide { max_side: 960 });
    }

    #[test]
    fn test_v5_config() {
        let config = OcrConfig::v5();
        assert!(matches!(
            config.det_resize_strategy,
            DetResizeStrategy::MinSide {
                min_side: 64,
                max_side_limit: 4000
            }
        ));
    }

    #[test]
    fn test_builder() {
        let config = OcrConfig::builder()
            .det_threshold(0.5)
            .num_threads(8)
            .detect_styles(false)
            .build();

        assert!((config.det_threshold - 0.5).abs() < f32::EPSILON);
        assert_eq!(config.num_threads, 8);
        assert!(!config.detect_styles);
    }

    #[test]
    fn test_builder_clamping() {
        let config = OcrConfig::builder()
            .det_threshold(1.5) // Should clamp to 1.0
            .num_threads(0) // Should clamp to 1
            .build();

        assert!((config.det_threshold - 1.0).abs() < f32::EPSILON);
        assert_eq!(config.num_threads, 1);
    }
}
