//! Document complexity estimation for hybrid ML/classical routing.
//!
//! This module analyzes PDF pages to determine their layout complexity,
//! which is used to decide whether to use classical algorithms (fast)
//! or ML models (accurate but slower).

use crate::layout::text_block::TextBlock;

/// Page complexity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Complexity {
    /// Simple single-column layout with uniform formatting.
    /// Use classical algorithms (fastest).
    Simple,

    /// Moderate complexity with some multi-column or varied formatting.
    /// Either approach works well.
    Moderate,

    /// Complex multi-column, irregular, or heavily formatted layout.
    /// ML models recommended for best accuracy.
    Complex,
}

/// Estimates document complexity to route between classical and ML approaches.
///
/// # Algorithm
///
/// Analyzes multiple factors weighted by importance:
/// - Column count (30%): More columns = more complex
/// - Font diversity (20%): More unique fonts = more complex
/// - Y-position variance (20%): Higher variance = irregular layout
/// - Block size variance (15%): Varied sizes = complex formatting
/// - Density (15%): Very sparse or dense = complex
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::hybrid::ComplexityEstimator;
/// # use pdf_oxide::layout::text_block::TextBlock;
/// # use pdf_oxide::geometry::Rect;
/// # use pdf_oxide::layout::text_block::{TextChar, FontWeight, Color};
/// #
/// # fn create_block(x: f32, y: f32) -> TextBlock {
/// #     let char_data = TextChar {
/// #         char: 'A',
/// #         bbox: Rect { x, y, width: 10.0, height: 10.0 },
/// #         font_name: "Arial".to_string(),
/// #         font_size: 12.0,
/// #         font_weight: FontWeight::Normal,
/// #         color: Color::black(),
/// #     };
/// #     TextBlock {
/// #         chars: vec![char_data],
/// #         bbox: Rect { x, y, width: 100.0, height: 20.0 },
/// #         text: "Test".to_string(),
/// #         avg_font_size: 12.0,
/// #         dominant_font: "Arial".to_string(),
/// #         is_bold: false,
/// #     }
/// # }
///
/// let blocks = vec![create_block(0.0, 0.0), create_block(0.0, 30.0)];
/// let complexity = ComplexityEstimator::estimate_page_complexity(&blocks, 612.0, 792.0);
/// ```
pub struct ComplexityEstimator;

impl ComplexityEstimator {
    /// Estimate page complexity based on text block analysis.
    ///
    /// # Arguments
    ///
    /// * `blocks` - Text blocks on the page
    /// * `page_width` - Width of the page in points
    /// * `page_height` - Height of the page in points
    ///
    /// # Returns
    ///
    /// Returns a Complexity classification:
    /// - Simple: Score < 0.3
    /// - Moderate: Score 0.3 - 0.6
    /// - Complex: Score > 0.6
    pub fn estimate_page_complexity(
        blocks: &[TextBlock],
        page_width: f32,
        page_height: f32,
    ) -> Complexity {
        let score = Self::calculate_complexity_score(blocks, page_width, page_height);

        if score < 0.3 {
            Complexity::Simple
        } else if score < 0.6 {
            Complexity::Moderate
        } else {
            Complexity::Complex
        }
    }

    /// Calculate numeric complexity score in [0, 1].
    ///
    /// # Arguments
    ///
    /// * `blocks` - Text blocks on the page
    /// * `page_width` - Width of the page in points
    /// * `page_height` - Height of the page in points
    ///
    /// # Returns
    ///
    /// Returns a score in [0, 1] where higher values indicate more complexity.
    pub fn calculate_complexity_score(
        blocks: &[TextBlock],
        page_width: f32,
        page_height: f32,
    ) -> f32 {
        if blocks.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;

        // Factor 1: Column detection (30% weight)
        // More columns = more complex
        let columns = Self::estimate_columns(blocks, page_width);
        score += (columns.saturating_sub(1) as f32 * 0.15).min(0.3);

        // Factor 2: Font diversity (20% weight)
        // More unique fonts = more complex typography
        let unique_fonts = Self::count_unique_fonts(blocks);
        score += (unique_fonts.saturating_sub(2) as f32 * 0.05).min(0.2);

        // Factor 3: Y-position variance (20% weight)
        // Higher variance = irregular layout
        let y_variance = Self::calculate_y_variance(blocks, page_height);
        score += y_variance.min(0.2);

        // Factor 4: Block size variance (15% weight)
        // Varied block sizes = complex formatting
        let size_variance = Self::calculate_size_variance(blocks);
        score += size_variance.min(0.15);

        // Factor 5: Density (15% weight)
        // Very sparse or very dense layouts are complex
        let density = Self::calculate_density(blocks, page_width, page_height);
        if !(0.2..=0.8).contains(&density) {
            score += 0.15; // Extreme densities add complexity
        }

        score.min(1.0)
    }

    /// Estimate number of columns using X-position clustering.
    fn estimate_columns(blocks: &[TextBlock], page_width: f32) -> usize {
        if blocks.is_empty() {
            return 0;
        }

        // Collect and sort X positions
        let mut x_positions: Vec<f32> = blocks.iter().map(|b| b.bbox.x).collect();
        x_positions.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));

        // Count gaps larger than 20% of page width as column separators
        let mut columns = 1;
        let threshold = page_width * 0.2;

        for window in x_positions.windows(2) {
            if (window[1] - window[0]) > threshold {
                columns += 1;
            }
        }

        columns.min(4) // Cap at 4 columns
    }

    /// Count unique fonts in blocks.
    fn count_unique_fonts(blocks: &[TextBlock]) -> usize {
        let mut fonts: Vec<&str> = blocks.iter().map(|b| b.dominant_font.as_str()).collect();
        fonts.sort_unstable();
        fonts.dedup();
        fonts.len()
    }

    /// Calculate normalized Y-position variance.
    ///
    /// Higher variance indicates blocks are spread irregularly across the page.
    fn calculate_y_variance(blocks: &[TextBlock], page_height: f32) -> f32 {
        if blocks.is_empty() {
            return 0.0;
        }

        let mean_y: f32 = blocks.iter().map(|b| b.bbox.y).sum::<f32>() / blocks.len() as f32;
        let variance: f32 = blocks
            .iter()
            .map(|b| (b.bbox.y - mean_y).powi(2))
            .sum::<f32>()
            / blocks.len() as f32;

        // Normalize by page height
        (variance.sqrt() / page_height).min(1.0)
    }

    /// Calculate coefficient of variation for block sizes.
    ///
    /// Higher values indicate varied block sizes (complex formatting).
    fn calculate_size_variance(blocks: &[TextBlock]) -> f32 {
        if blocks.is_empty() {
            return 0.0;
        }

        let mean_size: f32 =
            blocks.iter().map(|b| b.avg_font_size).sum::<f32>() / blocks.len() as f32;

        if mean_size == 0.0 {
            return 0.0;
        }

        let variance: f32 = blocks
            .iter()
            .map(|b| (b.avg_font_size - mean_size).powi(2))
            .sum::<f32>()
            / blocks.len() as f32;

        // Coefficient of variation (normalized standard deviation)
        (variance.sqrt() / mean_size).min(1.0)
    }

    /// Calculate text density (coverage of page).
    ///
    /// Extreme densities (very sparse or very dense) indicate complex layouts.
    fn calculate_density(blocks: &[TextBlock], page_width: f32, page_height: f32) -> f32 {
        if page_width == 0.0 || page_height == 0.0 {
            return 0.0;
        }

        let total_area: f32 = blocks.iter().map(|b| b.bbox.width * b.bbox.height).sum();

        let page_area = page_width * page_height;
        (total_area / page_area).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::text_block::{Color, FontWeight, TextBlock, TextChar};

    fn create_test_block(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        font_size: f32,
        font: &str,
    ) -> TextBlock {
        let bbox = Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
        };
        let char_data = TextChar {
            char: 'A',
            bbox,
            font_name: font.to_string(),
            font_size,
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
            ascent: 0.95 * font_size,
            descent: -0.35 * font_size,
            matrix: None,
        };

        TextBlock {
            chars: vec![char_data],
            bbox: Rect {
                x,
                y,
                width,
                height,
            },
            text: "Test".to_string(),
            avg_font_size: font_size,
            dominant_font: font.to_string(),
            is_bold: false,
            is_italic: false,
            mcid: None,
            sequence: 0,
            rotation_degrees: 0.0,
        }
    }

    #[test]
    fn test_simple_layout() {
        // Single column, uniform font, regular spacing
        let blocks = vec![
            create_test_block(50.0, 100.0, 500.0, 20.0, 12.0, "Arial"),
            create_test_block(50.0, 130.0, 500.0, 20.0, 12.0, "Arial"),
            create_test_block(50.0, 160.0, 500.0, 20.0, 12.0, "Arial"),
        ];

        let complexity = ComplexityEstimator::estimate_page_complexity(&blocks, 612.0, 792.0);
        assert_eq!(complexity, Complexity::Simple);
    }

    #[test]
    fn test_multi_column_layout() {
        // Two columns
        let blocks = vec![
            create_test_block(50.0, 100.0, 200.0, 20.0, 12.0, "Arial"),
            create_test_block(350.0, 100.0, 200.0, 20.0, 12.0, "Arial"),
            create_test_block(50.0, 130.0, 200.0, 20.0, 12.0, "Arial"),
            create_test_block(350.0, 130.0, 200.0, 20.0, 12.0, "Arial"),
        ];

        let complexity = ComplexityEstimator::estimate_page_complexity(&blocks, 612.0, 792.0);
        assert!(complexity >= Complexity::Moderate);
    }

    #[test]
    fn test_mixed_fonts() {
        // Multiple fonts
        let blocks = vec![
            create_test_block(50.0, 100.0, 500.0, 20.0, 12.0, "Arial"),
            create_test_block(50.0, 130.0, 500.0, 20.0, 14.0, "Times"),
            create_test_block(50.0, 160.0, 500.0, 20.0, 10.0, "Courier"),
            create_test_block(50.0, 190.0, 500.0, 20.0, 16.0, "Helvetica"),
        ];

        let complexity = ComplexityEstimator::estimate_page_complexity(&blocks, 612.0, 792.0);
        assert!(complexity >= Complexity::Moderate);
    }

    #[test]
    fn test_irregular_layout() {
        // Irregular Y positions and varied sizes
        let blocks = vec![
            create_test_block(50.0, 100.0, 500.0, 20.0, 24.0, "Arial"),
            create_test_block(100.0, 300.0, 400.0, 15.0, 12.0, "Times"), // Different font
            create_test_block(50.0, 600.0, 300.0, 10.0, 8.0, "Courier"), // Different font
        ];

        let complexity = ComplexityEstimator::estimate_page_complexity(&blocks, 612.0, 792.0);
        // Should be at least Moderate due to irregular layout and font diversity
        assert!(complexity >= Complexity::Moderate);
    }

    #[test]
    fn test_empty_page() {
        let blocks: Vec<TextBlock> = vec![];
        let complexity = ComplexityEstimator::estimate_page_complexity(&blocks, 612.0, 792.0);
        assert_eq!(complexity, Complexity::Simple);
    }

    #[test]
    fn test_estimate_columns() {
        // Single column
        let single_col = vec![
            create_test_block(50.0, 100.0, 500.0, 20.0, 12.0, "Arial"),
            create_test_block(50.0, 130.0, 500.0, 20.0, 12.0, "Arial"),
        ];
        assert_eq!(ComplexityEstimator::estimate_columns(&single_col, 612.0), 1);

        // Two columns (gap > 20% of page width = 122.4pt)
        let two_col = vec![
            create_test_block(50.0, 100.0, 200.0, 20.0, 12.0, "Arial"),
            create_test_block(350.0, 100.0, 200.0, 20.0, 12.0, "Arial"),
        ];
        assert_eq!(ComplexityEstimator::estimate_columns(&two_col, 612.0), 2);
    }

    #[test]
    fn test_count_unique_fonts() {
        let blocks = vec![
            create_test_block(0.0, 0.0, 100.0, 20.0, 12.0, "Arial"),
            create_test_block(0.0, 0.0, 100.0, 20.0, 12.0, "Arial"),
            create_test_block(0.0, 0.0, 100.0, 20.0, 12.0, "Times"),
            create_test_block(0.0, 0.0, 100.0, 20.0, 12.0, "Courier"),
        ];

        assert_eq!(ComplexityEstimator::count_unique_fonts(&blocks), 3);
    }

    #[test]
    fn test_density_calculation() {
        // Sparse layout
        let sparse = vec![create_test_block(0.0, 0.0, 50.0, 20.0, 12.0, "Arial")];
        let density = ComplexityEstimator::calculate_density(&sparse, 612.0, 792.0);
        assert!(density < 0.01); // Very low density

        // Dense layout
        let dense = vec![create_test_block(0.0, 0.0, 600.0, 700.0, 12.0, "Arial")];
        let density = ComplexityEstimator::calculate_density(&dense, 612.0, 792.0);
        assert!(density > 0.8); // High density
    }

    #[test]
    fn test_complexity_ordering() {
        assert!(Complexity::Simple < Complexity::Moderate);
        assert!(Complexity::Moderate < Complexity::Complex);
        assert!(Complexity::Simple < Complexity::Complex);
    }

    #[test]
    fn test_complexity_score_range() {
        let blocks = vec![
            create_test_block(0.0, 0.0, 100.0, 20.0, 12.0, "Arial"),
            create_test_block(0.0, 30.0, 100.0, 20.0, 12.0, "Arial"),
        ];

        let score = ComplexityEstimator::calculate_complexity_score(&blocks, 612.0, 792.0);

        // Score should always be in [0, 1]
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }
}
