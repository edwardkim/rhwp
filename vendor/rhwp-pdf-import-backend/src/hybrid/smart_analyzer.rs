//! Hybrid smart layout analyzer.
//!
//! This module orchestrates between classical and ML-based approaches
//! based on document complexity, providing the best balance of speed
//! and accuracy.
//!
//! NOTE: Heading detection has been removed (non-PDF-spec-compliant).
//! This module now focuses on reading order optimization only.

use crate::error::Result;
use crate::hybrid::complexity_estimator::{Complexity, ComplexityEstimator};
use crate::layout::text_block::TextBlock;

/// Heading classification levels (deprecated - heading detection removed).
///
/// This enum is kept for API compatibility but heading detection
/// is no longer performed as it is not PDF-spec-compliant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    /// Top-level heading
    H1,
    /// Second-level heading
    H2,
    /// Third-level heading
    H3,
    /// Fourth-level heading
    H4,
    /// Body text (default)
    Body,
}

/// Smart layout analyzer that chooses between classical and ML approaches.
///
/// # Strategy
///
/// - **Simple documents**: Always use fast classical algorithms
/// - **Moderate documents**: Use classical for speed (both work well)
/// - **Complex documents**: Try ML first, fall back to classical if unavailable
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::hybrid::SmartLayoutAnalyzer;
///
/// let analyzer = SmartLayoutAnalyzer::new();
/// let order = analyzer.determine_reading_order(&blocks, 612.0, 792.0)?;
/// let headings = analyzer.detect_headings(&blocks)?;
/// ```
pub struct SmartLayoutAnalyzer {
    /// Complexity threshold for using ML (default: Moderate)
    complexity_threshold: Complexity,
}

impl SmartLayoutAnalyzer {
    /// Create a new smart analyzer.
    ///
    /// This attempts to load ML models if the `ml` feature is enabled.
    /// If models can't be loaded, the analyzer will fall back to classical methods.
    pub fn new() -> Self {
        Self {
            complexity_threshold: Complexity::Moderate,
        }
    }

    /// Create a new analyzer with custom complexity threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum complexity to use ML models
    ///
    /// # Example
    ///
    /// ```
    /// use pdf_oxide::hybrid::{SmartLayoutAnalyzer, Complexity};
    ///
    /// // Always use ML if available
    /// let analyzer = SmartLayoutAnalyzer::with_threshold(Complexity::Simple);
    ///
    /// // Only use ML for very complex documents
    /// let analyzer = SmartLayoutAnalyzer::with_threshold(Complexity::Complex);
    /// ```
    pub fn with_threshold(threshold: Complexity) -> Self {
        let mut analyzer = Self::new();
        analyzer.complexity_threshold = threshold;
        analyzer
    }

    /// Determine reading order for text blocks using best available method.
    ///
    /// # Arguments
    ///
    /// * `blocks` - Text blocks to order
    /// * `page_width` - Width of the page in points
    /// * `page_height` - Height of the page in points
    ///
    /// # Returns
    ///
    /// Returns a vector of indices indicating the reading order.
    ///
    /// # Algorithm
    ///
    /// 1. Estimate page complexity
    /// 2. If complexity >= threshold and ML available: use ML
    /// 3. Otherwise: use classical top-to-bottom, left-to-right
    pub fn determine_reading_order(
        &self,
        blocks: &[TextBlock],
        page_width: f32,
        page_height: f32,
    ) -> Result<Vec<usize>> {
        if blocks.is_empty() {
            return Ok(vec![]);
        }

        // Estimate complexity
        let complexity =
            ComplexityEstimator::estimate_page_complexity(blocks, page_width, page_height);

        log::debug!(
            "Page complexity: {:?} (threshold: {:?})",
            complexity,
            self.complexity_threshold
        );

        // ML reading order removed - using only classical approach
        log::info!("Using classical reading order (complexity: {:?})", complexity);
        Ok(self.classical_reading_order(blocks))
    }

    /// Detect headings (DEPRECATED - heading detection has been removed).
    ///
    /// # Arguments
    ///
    /// * `blocks` - Text blocks to classify
    ///
    /// # Returns
    ///
    /// Returns a vector of HeadingLevel::Body for all blocks (constant result).
    ///
    /// # Note
    ///
    /// Heading detection is no longer supported as it is not PDF-spec-compliant.
    /// This method is kept for API compatibility only and always returns Body level.
    pub fn detect_headings(&self, blocks: &[TextBlock]) -> Result<Vec<HeadingLevel>> {
        if blocks.is_empty() {
            return Ok(vec![]);
        }

        // Heading detection removed (non-PDF-spec-compliant)
        // Return Body level for all blocks
        log::warn!(
            "Heading detection has been removed (non-PDF-spec-compliant). All blocks will be treated as body text."
        );
        Ok(vec![HeadingLevel::Body; blocks.len()])
    }

    /// Get analyzer capabilities.
    ///
    /// # Returns
    ///
    /// Returns an AnalyzerCapabilities struct describing what features are available.
    pub fn capabilities(&self) -> AnalyzerCapabilities {
        AnalyzerCapabilities {
            has_ml_reading_order: false,     // ML module removed
            has_ml_heading_detection: false, // Heading detection removed (non-spec-compliant)
            ml_models_loaded: false,         // ML module removed
            complexity_threshold: self.complexity_threshold,
        }
    }

    // Private helper methods

    /// Classical reading order: top-to-bottom, left-to-right.
    fn classical_reading_order(&self, blocks: &[TextBlock]) -> Vec<usize> {
        let mut order: Vec<usize> = (0..blocks.len()).collect();

        order.sort_by(|&a, &b| {
            let block_a = &blocks[a];
            let block_b = &blocks[b];

            // Sort by Y position (top to bottom), then X position (left to right)
            crate::utils::safe_float_cmp(block_a.bbox.y, block_b.bbox.y)
                .then(crate::utils::safe_float_cmp(block_a.bbox.x, block_b.bbox.x))
        });

        order
    }
}

impl Default for SmartLayoutAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Capabilities of the smart analyzer.
#[derive(Debug, Clone)]
pub struct AnalyzerCapabilities {
    /// Whether ML reading order is compiled in (feature flag)
    pub has_ml_reading_order: bool,

    /// Whether ML heading detection is compiled in (feature flag)
    pub has_ml_heading_detection: bool,

    /// Whether ML models are actually loaded and ready
    pub ml_models_loaded: bool,

    /// Complexity threshold for using ML
    pub complexity_threshold: Complexity,
}

impl AnalyzerCapabilities {
    /// Check if any ML capabilities are available.
    pub fn has_any_ml(&self) -> bool {
        self.ml_models_loaded
    }

    /// Get a human-readable description of capabilities.
    pub fn description(&self) -> String {
        if self.ml_models_loaded {
            format!("ML-enhanced (threshold: {:?})", self.complexity_threshold)
        } else if self.has_ml_reading_order || self.has_ml_heading_detection {
            "ML compiled but models not loaded (using classical)".to_string()
        } else {
            "Classical only (ML feature not enabled)".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::text_block::{Color, FontWeight, TextBlock, TextChar};

    fn create_test_block(x: f32, y: f32, text: &str) -> TextBlock {
        let bbox = Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
        };
        let char_data = TextChar {
            char: 'A',
            bbox,
            font_name: "Arial".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * 12.0,
            descent: -0.35 * 12.0,
            matrix: None,
        };

        TextBlock {
            chars: vec![char_data],
            bbox: Rect {
                x,
                y,
                width: 100.0,
                height: 20.0,
            },
            text: text.to_string(),
            avg_font_size: 12.0,
            dominant_font: "Arial".to_string(),
            is_bold: false,
            is_italic: false,
            mcid: None,
            sequence: 0,
            rotation_degrees: 0.0,
        }
    }

    #[test]
    fn test_create_analyzer() {
        let analyzer = SmartLayoutAnalyzer::new();
        let caps = analyzer.capabilities();

        // Should always have classical capabilities
        assert!(!caps.description().is_empty());
    }

    #[test]
    fn test_reading_order() {
        let analyzer = SmartLayoutAnalyzer::new();

        let blocks = vec![
            create_test_block(100.0, 200.0, "third"),
            create_test_block(100.0, 100.0, "first"),
            create_test_block(100.0, 150.0, "second"),
        ];

        let order = analyzer
            .determine_reading_order(&blocks, 612.0, 792.0)
            .unwrap();

        // Should be sorted by Y position
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn test_heading_detection() {
        let analyzer = SmartLayoutAnalyzer::new();

        let blocks = vec![
            create_test_block(100.0, 100.0, "Test"),
            create_test_block(100.0, 130.0, "More text"),
        ];

        let headings = analyzer.detect_headings(&blocks).unwrap();

        // Should return a heading level for each block
        assert_eq!(headings.len(), 2);
    }

    #[test]
    fn test_empty_blocks() {
        let analyzer = SmartLayoutAnalyzer::new();

        let order = analyzer.determine_reading_order(&[], 612.0, 792.0).unwrap();
        assert_eq!(order.len(), 0);

        let headings = analyzer.detect_headings(&[]).unwrap();
        assert_eq!(headings.len(), 0);
    }

    #[test]
    fn test_with_threshold() {
        let analyzer = SmartLayoutAnalyzer::with_threshold(Complexity::Complex);
        let caps = analyzer.capabilities();

        assert_eq!(caps.complexity_threshold, Complexity::Complex);
    }

    #[test]
    fn test_capabilities() {
        let analyzer = SmartLayoutAnalyzer::new();
        let caps = analyzer.capabilities();

        // ML module removed - should always be false
        assert!(!caps.has_ml_reading_order);
        assert!(!caps.has_ml_heading_detection);
        assert!(!caps.ml_models_loaded);
    }

    #[test]
    fn test_classical_reading_order() {
        let analyzer = SmartLayoutAnalyzer::new();

        // Multi-column-like layout
        let blocks = vec![
            create_test_block(50.0, 100.0, "top-left"),
            create_test_block(400.0, 100.0, "top-right"),
            create_test_block(50.0, 200.0, "bottom-left"),
            create_test_block(400.0, 200.0, "bottom-right"),
        ];

        let order = analyzer.classical_reading_order(&blocks);

        // Classical: top-to-bottom, left-to-right
        // Should read: top-left, top-right, bottom-left, bottom-right
        assert_eq!(order[0], 0); // top-left
        assert_eq!(order[1], 1); // top-right
        assert_eq!(order[2], 2); // bottom-left
        assert_eq!(order[3], 3); // bottom-right
    }
}
