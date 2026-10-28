//! Document property analysis for adaptive layout detection.
//!
//! This module analyzes PDF page characteristics to compute adaptive parameters
//! for layout analysis algorithms (XY-Cut, clustering, column detection).
//!
//! ## Key Insight
//!
//! PDFs have vastly different characteristics:
//! - Font sizes: 6pt (footnotes) to 72pt (titles)
//! - Page sizes: A4, Letter, Legal, Custom
//! - Layouts: Single-column, multi-column, mixed
//!
//! Fixed parameters (e.g., "gap > 50pt") work poorly across this diversity.
//! Adaptive parameters based on document analysis work much better.
//!
//! ## Approach
//!
//! 1. Analyze page/document to extract properties (fonts, spacing, dimensions)
//! 2. Compute adaptive thresholds as ratios of measured properties
//! 3. Apply to layout algorithms (XY-Cut, clustering)
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::layout::{DocumentProperties, AdaptiveLayoutParams};
//!
//! // Analyze page characteristics
//! let props = DocumentProperties::analyze(&chars)?;
//!
//! // Compute adaptive parameters
//! let params = AdaptiveLayoutParams::from_properties(&props);
//!
//! // Use for layout analysis
//! let layout = xy_cut_adaptive(page_bbox, blocks, &params);
//! ```

use crate::geometry::Rect;
use crate::layout::TextChar;

/// Properties of a PDF page used for adaptive layout analysis.
///
/// These properties are measured from the actual text content and used
/// to compute adaptive thresholds for layout algorithms.
#[derive(Debug, Clone)]
pub struct DocumentProperties {
    /// Median font size across all characters (in points).
    ///
    /// Used as baseline for:
    /// - Word clustering threshold
    /// - Line spacing detection
    /// - Column gap detection
    pub median_font_size: f32,

    /// Median character width (in PDF units).
    ///
    /// Used for word clustering - gaps larger than this likely indicate word boundaries.
    pub median_char_width: f32,

    /// Median vertical spacing between lines (in PDF units).
    ///
    /// Used for line clustering - gaps significantly larger than this likely indicate
    /// paragraph breaks or section boundaries.
    pub median_line_spacing: f32,

    /// Page width (in PDF units, typically 612 for Letter, 595 for A4).
    pub page_width: f32,

    /// Page height (in PDF units, typically 792 for Letter, 842 for A4).
    pub page_height: f32,

    /// Detected number of columns (1 = single-column, 2+ = multi-column).
    ///
    /// Detected via gap analysis in horizontal projection profile.
    pub column_count: usize,

    /// Average characters per line.
    ///
    /// Used to detect abnormally short/long lines that may indicate
    /// special formatting (headers, footers, captions).
    pub avg_chars_per_line: f32,

    /// Standard deviation of line Y-coordinates.
    ///
    /// High variance suggests irregular layout (tables, figures).
    /// Low variance suggests regular paragraph text.
    pub line_y_variance: f32,
}

impl DocumentProperties {
    /// Analyze a page's text characters to extract properties.
    ///
    /// This performs statistical analysis of fonts, spacing, and layout
    /// to compute properties used for adaptive parameter selection.
    ///
    /// # Arguments
    ///
    /// * `chars` - All text characters on the page
    /// * `page_bbox` - Bounding box of the page
    ///
    /// # Returns
    ///
    /// Document properties, or error if analysis fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::layout::DocumentProperties;
    /// # use pdf_oxide::geometry::Rect;
    /// # fn example(chars: Vec<pdf_oxide::layout::TextChar>) -> Result<(), Box<dyn std::error::Error>> {
    /// let page_bbox = Rect::new(0.0, 0.0, 612.0, 792.0);
    /// let props = DocumentProperties::analyze(&chars, page_bbox)?;
    ///
    /// println!("Median font size: {:.1}pt", props.median_font_size);
    /// println!("Column count: {}", props.column_count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze(chars: &[TextChar], page_bbox: Rect) -> Result<Self, String> {
        if chars.is_empty() {
            return Err("Cannot analyze empty page".into());
        }

        // 1. Compute median font size
        let median_font_size = Self::compute_median_font_size(chars);

        // 2. Compute median character width
        let median_char_width = Self::compute_median_char_width(chars);

        // 3. Estimate line spacing (requires clustering into lines first)
        let (median_line_spacing, avg_chars_per_line, line_y_variance) =
            Self::estimate_line_properties(chars);

        // 4. Detect columns (rough heuristic based on horizontal gaps)
        let column_count = Self::detect_column_count(chars, page_bbox.width);

        Ok(Self {
            median_font_size,
            median_char_width,
            median_line_spacing,
            page_width: page_bbox.width,
            page_height: page_bbox.height,
            column_count,
            avg_chars_per_line,
            line_y_variance,
        })
    }

    /// Compute median font size from characters.
    ///
    /// Uses `select_nth_unstable_by` (O(n) quickselect) rather than a full
    /// sort — only the middle order-statistic is needed, and the result is
    /// exact.
    fn compute_median_font_size(chars: &[TextChar]) -> f32 {
        let mut font_sizes: Vec<f32> = chars.iter().map(|c| c.font_size).collect();
        if font_sizes.is_empty() {
            return 12.0; // Default fallback
        }
        let mid = font_sizes.len() / 2;
        font_sizes.select_nth_unstable_by(mid, |a, b| crate::utils::safe_float_cmp(*a, *b));
        font_sizes[mid]
    }

    /// Compute median character width from characters.
    ///
    /// O(n) quickselect for the median (see `compute_median_font_size`).
    fn compute_median_char_width(chars: &[TextChar]) -> f32 {
        let mut widths: Vec<f32> = chars.iter().map(|c| c.bbox.width).collect();
        if widths.is_empty() {
            return 6.0; // Default fallback
        }
        let mid = widths.len() / 2;
        widths.select_nth_unstable_by(mid, |a, b| crate::utils::safe_float_cmp(*a, *b));
        widths[mid]
    }

    /// Estimate line spacing properties.
    ///
    /// Uses simple Y-coordinate clustering to identify lines,
    /// then measures vertical spacing between them.
    fn estimate_line_properties(chars: &[TextChar]) -> (f32, f32, f32) {
        if chars.is_empty() {
            return (12.0, 50.0, 0.0);
        }

        // Cluster characters by Y coordinate (simple binning)
        use std::collections::HashMap;
        let mut y_bins: HashMap<i32, Vec<&TextChar>> = HashMap::new();

        for ch in chars {
            // Bin Y coordinates by rounding to nearest 5 units
            let y_bin = (ch.bbox.y / 5.0).round() as i32;
            y_bins.entry(y_bin).or_default().push(ch);
        }

        // Extract line Y-coordinates (center of each bin)
        let mut line_ys: Vec<f32> = y_bins.keys().map(|&k| k as f32 * 5.0).collect();
        line_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a)); // Top to bottom

        // Compute line spacing (gaps between consecutive lines)
        let mut spacings = Vec::new();
        for i in 0..line_ys.len().saturating_sub(1) {
            let spacing = (line_ys[i] - line_ys[i + 1]).abs();
            if spacing > 0.1 {
                // Ignore zero/tiny spacings
                spacings.push(spacing);
            }
        }

        let median_line_spacing = if spacings.is_empty() {
            12.0
        } else {
            let mid = spacings.len() / 2;
            spacings.select_nth_unstable_by(mid, |a, b| crate::utils::safe_float_cmp(*a, *b));
            spacings[mid]
        };

        // Compute average characters per line
        let total_lines = y_bins.len() as f32;
        let avg_chars_per_line = if total_lines > 0.0 {
            chars.len() as f32 / total_lines
        } else {
            50.0
        };

        // Compute line Y variance
        let mean_y = line_ys.iter().sum::<f32>() / line_ys.len().max(1) as f32;
        let variance = line_ys.iter().map(|&y| (y - mean_y).powi(2)).sum::<f32>()
            / line_ys.len().max(1) as f32;

        (median_line_spacing, avg_chars_per_line, variance)
    }

    /// Detect number of columns via horizontal gap analysis.
    ///
    /// Creates a horizontal projection profile and counts significant gaps
    /// that likely represent column boundaries.
    fn detect_column_count(chars: &[TextChar], page_width: f32) -> usize {
        if chars.is_empty() {
            return 1;
        }

        // Create horizontal projection profile (character density per X-bin)
        const BIN_WIDTH: f32 = 10.0; // 10 PDF units per bin
        let bin_count = (page_width / BIN_WIDTH).ceil() as usize;
        let mut bins = vec![0usize; bin_count];

        for ch in chars {
            let bin = (ch.bbox.x / BIN_WIDTH).floor() as usize;
            if bin < bin_count {
                bins[bin] += 1;
            }
        }

        // Find significant gaps (bins with zero or very low density)
        let max_density = *bins.iter().max().unwrap_or(&1);
        let gap_threshold = (max_density as f32 * 0.1) as usize; // 10% of max

        let mut gap_count = 0;
        let mut in_gap = false;
        let mut gap_width = 0;
        let mut has_content = false; // Track if we've seen content yet (to ignore leading edge gaps)

        for &density in &bins {
            if density <= gap_threshold {
                if !in_gap {
                    in_gap = true;
                    gap_width = 1;
                } else {
                    gap_width += 1;
                }
            } else {
                if in_gap && gap_width >= 3 && has_content {
                    // Significant gap (at least 30 PDF units), but only count if we've seen content
                    // This ignores the leading edge gap before any text appears
                    gap_count += 1;
                }
                in_gap = false;
                gap_width = 0;

                // Mark that we've seen content (do this AFTER checking for gap to avoid counting leading edge)
                has_content = true;
            }
        }

        // Number of columns = number of significant gaps + 1
        // (but cap at 4 to avoid false positives)
        (gap_count + 1).min(4)
    }
}

/// Adaptive layout parameters computed from document properties.
///
/// These parameters are used by layout algorithms (XY-Cut, clustering, etc.)
/// and are computed as ratios/multiples of measured document properties
/// rather than fixed absolute values.
#[derive(Debug, Clone)]
pub struct AdaptiveLayoutParams {
    /// Minimum gap for XY-Cut split (as ratio of page dimension).
    ///
    /// Typical: 0.05 (5% of page width/height)
    pub xy_cut_min_gap_ratio: f32,

    /// Word clustering threshold (horizontal gap between characters).
    ///
    /// Gaps larger than this likely indicate word boundaries.
    /// Computed as multiple of median character width.
    pub word_gap_threshold: f32,

    /// Line clustering threshold (vertical gap between characters).
    ///
    /// Gaps larger than this likely indicate line boundaries.
    /// Computed as multiple of median line spacing.
    pub line_gap_threshold: f32,

    /// Column gap threshold (minimum gap between columns).
    ///
    /// Computed as multiple of median font size.
    pub column_gap_threshold: f32,

    /// Maximum recursion depth for XY-Cut.
    pub xy_cut_max_depth: u32,

    /// Minimum region size for XY-Cut split (in PDF units).
    ///
    /// Regions smaller than this won't be split further.
    pub xy_cut_min_region_size: f32,

    /// Gaussian smoothing sigma for projection profiles.
    ///
    /// Controls noise reduction in XY-Cut valley detection.
    /// Adaptive based on layout density:
    /// - Dense layouts (short lines): Low sigma (0.5) for sharp peaks
    /// - Medium layouts: Medium sigma (1.5) balances noise vs detail
    /// - Sparse layouts (long lines): High sigma (2.5) for broad valleys
    ///
    /// Meunier (ICDAR 2005) recommends σ=2.0 as baseline.
    pub gaussian_sigma: f32,
}

impl AdaptiveLayoutParams {
    /// Compute adaptive parameters from document properties.
    ///
    /// # Arguments
    ///
    /// * `props` - Analyzed document properties
    ///
    /// # Returns
    ///
    /// Adaptive parameters tuned for the specific document.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::layout::{DocumentProperties, AdaptiveLayoutParams};
    /// # use pdf_oxide::geometry::Rect;
    /// # fn example(chars: Vec<pdf_oxide::layout::TextChar>) -> Result<(), Box<dyn std::error::Error>> {
    /// let page_bbox = Rect::new(0.0, 0.0, 612.0, 792.0);
    /// let props = DocumentProperties::analyze(&chars, page_bbox)?;
    /// let params = AdaptiveLayoutParams::from_properties(&props);
    ///
    /// println!("Word gap threshold: {:.1}", params.word_gap_threshold);
    /// println!("Column gap threshold: {:.1}", params.column_gap_threshold);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_properties(props: &DocumentProperties) -> Self {
        Self {
            // XY-Cut split threshold: 5% of page dimension
            xy_cut_min_gap_ratio: 0.05,

            // Word gap: 30% of median character width
            // (spaces are typically 25-35% of character width)
            word_gap_threshold: props.median_char_width * 0.3,

            // Line gap: 130% of median line spacing
            // (allows for slight variation in line spacing)
            // Capped at 80% of median font size to prevent merging separate lines (Issue 211)
            line_gap_threshold: (props.median_line_spacing * 1.3).min(props.median_font_size * 0.8),

            // Column gap: 2× median font size
            // (columns typically separated by at least 2 characters worth of space)
            column_gap_threshold: props.median_font_size * 2.0,

            // Maximum recursion depth for XY-Cut
            xy_cut_max_depth: 10,

            // Minimum region size: 5% of page area
            xy_cut_min_region_size: (props.page_width * props.page_height * 0.05).sqrt(),

            // Adaptive Gaussian sigma based on layout density
            // FIX #2: Adaptive smoothing for dense vs sparse layouts
            //
            // Density metric: avg_chars_per_line
            // - Dense layouts (author grids): ~10-20 chars/line → Low sigma (0.5)
            // - Medium layouts (2-column): ~40-60 chars/line → Medium sigma (1.5)
            // - Sparse layouts (1-column): ~80+ chars/line → High sigma (2.5)
            //
            // Why adaptive sigma helps:
            // - Dense layouts: Sharp peaks, need less smoothing to preserve detail
            // - Sparse layouts: Noisy profiles, need more smoothing to find valleys
            //
            // Baseline: Meunier (ICDAR 2005) recommends σ=2.0 for standard documents
            gaussian_sigma: {
                let density = props.avg_chars_per_line;
                if density < 30.0 {
                    0.5 // Dense layout: minimal smoothing
                } else if density < 60.0 {
                    1.5 // Medium layout: moderate smoothing
                } else {
                    2.5 // Sparse layout: heavy smoothing
                }
            },
        }
    }

    /// Create default parameters for when document analysis is unavailable.
    ///
    /// These are reasonable defaults for typical Letter-sized PDFs
    /// with 10-12pt text.
    pub fn default_for_letter_pdf() -> Self {
        Self {
            xy_cut_min_gap_ratio: 0.05,
            word_gap_threshold: 3.0,    // ~0.5 character width for 12pt text
            line_gap_threshold: 15.0,   // ~1.3× line spacing for 12pt text
            column_gap_threshold: 24.0, // ~2× font size for 12pt text
            xy_cut_max_depth: 10,
            xy_cut_min_region_size: 50.0,
            gaussian_sigma: 2.0, // Meunier (ICDAR 2005) baseline
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Color, FontWeight};

    fn mock_char(x: f32, y: f32, font_size: f32) -> TextChar {
        let bbox = Rect::new(x, y, 6.0, font_size);
        TextChar {
            char: 'x',
            bbox,
            font_name: "Times".to_string(),
            font_size,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            is_italic: false,
            is_monospace: false,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * font_size,
            descent: -0.35 * font_size,
            matrix: None,
        }
    }

    #[test]
    fn test_median_font_size() {
        let chars = vec![
            mock_char(0.0, 100.0, 10.0),
            mock_char(10.0, 100.0, 12.0),
            mock_char(20.0, 100.0, 12.0),
            mock_char(30.0, 100.0, 14.0),
            mock_char(40.0, 100.0, 16.0),
        ];

        let median = DocumentProperties::compute_median_font_size(&chars);
        assert_eq!(median, 12.0);
    }

    #[test]
    fn test_column_detection_single() {
        // Single column: characters evenly distributed
        let mut chars = Vec::new();
        for i in 0..100 {
            chars.push(mock_char(100.0 + (i % 10) as f32 * 10.0, 100.0, 12.0));
        }

        let columns = DocumentProperties::detect_column_count(&chars, 612.0);
        assert_eq!(columns, 1);
    }

    #[test]
    fn test_column_detection_double() {
        // Two columns: characters in two groups with gap
        let mut chars = Vec::new();

        // Left column (x: 50-200)
        for i in 0..50 {
            chars.push(mock_char(50.0 + (i % 15) as f32 * 10.0, 100.0, 12.0));
        }

        // Right column (x: 350-500)
        for i in 0..50 {
            chars.push(mock_char(350.0 + (i % 15) as f32 * 10.0, 100.0, 12.0));
        }

        let columns = DocumentProperties::detect_column_count(&chars, 612.0);
        assert_eq!(columns, 2);
    }

    #[test]
    fn test_adaptive_params_from_properties() {
        let chars = vec![
            mock_char(0.0, 100.0, 12.0),
            mock_char(10.0, 100.0, 12.0),
            mock_char(20.0, 85.0, 12.0),
            mock_char(30.0, 85.0, 12.0),
        ];

        let page_bbox = Rect::new(0.0, 0.0, 612.0, 792.0);
        let props = DocumentProperties::analyze(&chars, page_bbox).unwrap();
        let params = AdaptiveLayoutParams::from_properties(&props);

        // Verify parameters are computed as ratios
        assert!(params.word_gap_threshold > 0.0);
        assert!(params.line_gap_threshold > 0.0);
        assert!(params.column_gap_threshold > 0.0);

        // Word gap should be smaller than column gap
        assert!(params.word_gap_threshold < params.column_gap_threshold);
    }
}
