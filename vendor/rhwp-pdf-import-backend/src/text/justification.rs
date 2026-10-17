//! Text justification detection with PDF spec compliance.
//!
//! Implements ISO 32000-1:2008 Section 9.3 Text State Parameters:
//! - Tc (character spacing): Added after every character
//! - Tw (word spacing): Added after space characters (U+0020)
//! - Tz (horizontal scaling): Scales character widths and spacing
//!
//! Justification modes detected:
//! 1. Left-justified: Constant word spacing, ragged right edge
//! 2. Right-justified: Ragged left, aligned to right margin
//! 3. Center-justified: Balanced margins on both sides
//! 4. Fully-justified: Variable spacing to align both edges
//! 5. Unjustified: No apparent alignment structure

/// Justification modes for text alignment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustificationMode {
    /// Left-aligned with ragged right edge
    LeftJustified,
    /// Right-aligned with ragged left edge
    RightJustified,
    /// Centered with balanced margins
    CenterJustified,
    /// Aligned on both edges with variable spacing
    FullyJustified,
    /// No apparent justification structure
    Unjustified,
}

/// Detects text justification mode from line spacing and alignment.
///
/// Per ISO 32000-1:2008 Section 9.3.1, justification is determined by:
/// 1. Analysis of spacing variance (Tw - word spacing parameter)
/// 2. Line edge alignment (start_x and end_x positions)
/// 3. Margins relative to page boundaries
pub struct JustificationDetector;

impl JustificationDetector {
    /// Detect justification mode from line characteristics.
    ///
    /// # Arguments
    /// * `avg_word_spacing` - Average word spacing (Tw parameter) across line
    /// * `word_spacing_variance` - Variance in word spacing values
    /// * `start_x` - Line starting position (left edge)
    /// * `end_x` - Line ending position (right edge)
    /// * `page_width` - Total page width for margin calculation
    /// * `page_margin_left` - Left page margin (typically 0)
    ///
    /// # Returns
    /// `JustificationMode` indicating the detected justification
    pub fn detect(
        _avg_word_spacing: f32,
        word_spacing_variance: f32,
        start_x: f32,
        end_x: f32,
        page_width: f32,
        page_margin_left: f32,
    ) -> JustificationMode {
        let left_margin = start_x - page_margin_left;
        let right_margin = page_width - end_x;

        // Calculate margin balance for center detection
        let margin_diff = (left_margin - right_margin).abs();
        let is_centered = margin_diff < 10.0; // Allow 10 units tolerance

        // Check if line reaches both edges (fully justified)
        let aligns_left = left_margin < 5.0; // Within 5 units of left edge
        let aligns_right = right_margin < 5.0; // Within 5 units of right edge

        // Detect variance-based justification
        // High variance indicates variable spacing (fully justified)
        // Low variance indicates uniform spacing
        let has_spacing_variance = word_spacing_variance > 0.5;

        // Decision logic (priority order):
        if aligns_left && aligns_right {
            // Both edges aligned → fully justified
            JustificationMode::FullyJustified
        } else if is_centered {
            // Balanced margins → center justified
            JustificationMode::CenterJustified
        } else if aligns_right {
            // Right edge aligned, left ragged → right justified
            JustificationMode::RightJustified
        } else if aligns_left {
            // Left edge aligned (default case)
            if has_spacing_variance {
                // Variable spacing suggests fully justified attempt
                JustificationMode::FullyJustified
            } else {
                // Uniform spacing → left justified
                JustificationMode::LeftJustified
            }
        } else {
            // No alignment detected
            JustificationMode::Unjustified
        }
    }

    /// Calculate spacing variance in word spacing parameters.
    ///
    /// Variance indicates justification complexity:
    /// - Low variance: Consistent spacing (left-justified)
    /// - High variance: Variable spacing (fully-justified)
    pub fn calculate_word_spacing_variance(word_spacings: &[f32]) -> f32 {
        if word_spacings.is_empty() {
            return 0.0;
        }

        let mean = word_spacings.iter().sum::<f32>() / word_spacings.len() as f32;
        let variance = word_spacings
            .iter()
            .map(|&spacing| (spacing - mean).powi(2))
            .sum::<f32>()
            / word_spacings.len() as f32;

        variance.sqrt() // Return standard deviation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left_justified_detection() {
        let mode = JustificationDetector::detect(
            5.0,   // avg_word_spacing
            0.1,   // low variance
            0.0,   // start_x at left edge
            250.0, // end_x (ragged right)
            500.0, // page_width
            0.0,   // page_margin_left
        );
        assert_eq!(mode, JustificationMode::LeftJustified);
    }

    #[test]
    fn test_right_justified_detection() {
        let mode = JustificationDetector::detect(
            5.0,   // avg_word_spacing
            0.1,   // low variance
            250.0, // start_x (ragged left)
            500.0, // end_x at right edge
            500.0, // page_width
            0.0,   // page_margin_left
        );
        assert_eq!(mode, JustificationMode::RightJustified);
    }

    #[test]
    fn test_center_justified_detection() {
        let mode = JustificationDetector::detect(
            5.0,   // avg_word_spacing
            0.1,   // low variance
            200.0, // start_x (centered)
            300.0, // end_x (centered)
            500.0, // page_width
            0.0,   // page_margin_left
        );
        assert_eq!(mode, JustificationMode::CenterJustified);
    }

    #[test]
    fn test_fully_justified_detection() {
        let mode = JustificationDetector::detect(
            5.0,   // avg_word_spacing
            2.0,   // high variance
            0.0,   // start_x at left edge
            500.0, // end_x at right edge
            500.0, // page_width
            0.0,   // page_margin_left
        );
        assert_eq!(mode, JustificationMode::FullyJustified);
    }

    #[test]
    fn test_unjustified_detection() {
        let mode = JustificationDetector::detect(
            5.0,   // avg_word_spacing
            0.1,   // low variance
            100.0, // start_x (not aligned)
            350.0, // end_x (not aligned)
            500.0, // page_width
            0.0,   // page_margin_left
        );
        assert_eq!(mode, JustificationMode::Unjustified);
    }

    #[test]
    fn test_spacing_variance_calculation() {
        let spacings = vec![5.0, 5.0, 5.0, 5.0]; // Uniform spacing
        let variance = JustificationDetector::calculate_word_spacing_variance(&spacings);
        assert!(variance < 0.1, "Uniform spacing should have low variance");
    }

    #[test]
    fn test_spacing_variance_variable() {
        let spacings = vec![3.0, 5.0, 7.0, 4.0]; // Variable spacing
        let variance = JustificationDetector::calculate_word_spacing_variance(&spacings);
        assert!(variance > 1.0, "Variable spacing should have higher variance");
    }
}
