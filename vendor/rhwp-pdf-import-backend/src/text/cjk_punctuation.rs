//! CJK Punctuation Detection for Word Boundary Analysis
//!
//! This module provides detection functions for CJK (Chinese, Japanese, Korean) punctuation
//! characters that affect word boundary determination. CJK punctuation includes both fullwidth
//! and halfwidth variants that have different boundary semantics than ASCII punctuation.
//!
//! # Punctuation Categories
//!
//! - **Sentence-ending**: Fullstop, question mark, exclamation - always create boundaries
//! - **Enumeration**: Ideographic comma, semicolon - create boundaries in most contexts
//! - **Brackets**: Parentheses, angle brackets, corner brackets - paired boundaries
//! - **Other**: Ellipsis, middle dot, wave dash - context-dependent
//!
//! # Unicode Ranges
//!
//! - CJK Symbols and Punctuation: U+3000-U+303F
//! - Halfwidth and Fullwidth Forms: U+FF00-U+FFEF

/// CJK text density classification for adaptive scoring.
///
/// Different CJK documents have different character densities:
/// - Academic papers, formal documents: High density (many chars per page)
/// - Children's books, sparse layouts: Low density (fewer chars per page)
/// - Most books: Medium density
///
/// Density affects how aggressively punctuation creates boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDensity {
    /// Low density: Sparse text with significant whitespace
    /// Example: Children's books, formatted documents with large margins
    /// Characteristic: < 500 characters per page equivalent
    Low,

    /// Medium density: Normal text with standard spacing
    /// Example: Most books, regular documents
    /// Characteristic: 500-2000 characters per page equivalent
    Medium,

    /// High density: Dense text with minimal whitespace
    /// Example: Academic papers, technical documents, legal text
    /// Characteristic: > 2000 characters per page equivalent
    High,
}

impl TextDensity {
    /// Classify text density based on character count in a sample.
    ///
    /// This uses a simple heuristic: count characters in a representative sample
    /// and estimate document-wide density.
    ///
    /// # Arguments
    /// * `char_count` - Total characters processed so far
    /// * `page_count` - Number of pages processed
    ///
    /// # Returns
    /// Classified density level
    pub fn classify(char_count: usize, page_count: usize) -> Self {
        if page_count == 0 {
            return Self::Medium; // Default if unknown
        }

        let chars_per_page = char_count / page_count;

        match chars_per_page {
            0..=500 => Self::Low,
            501..=2000 => Self::Medium,
            _ => Self::High,
        }
    }

    /// Get density multiplier for adaptive scoring.
    ///
    /// Returns a factor (0.5-1.5) to multiply base scores:
    /// - Low density (0.6): More conservative, require higher scores for boundaries
    /// - Medium density (1.0): No adjustment
    /// - High density (1.4): More aggressive, lower scores trigger boundaries
    pub fn score_multiplier(&self) -> f32 {
        match self {
            Self::Low => 0.6,    // More conservative: require 0.6× normal score
            Self::Medium => 1.0, // No adjustment
            Self::High => 1.4,   // More aggressive: 1.4× normal score
        }
    }
}

/// Check if a character code represents fullwidth CJK punctuation.
///
/// Fullwidth punctuation is typically used in CJK text and occupies the same
/// width as CJK ideographs (one em square). These characters create stronger
/// word boundaries than their ASCII equivalents.
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is fullwidth CJK punctuation
pub fn is_fullwidth_punctuation(code: u32) -> bool {
    is_sentence_ending_punctuation(code)
        || is_enumeration_punctuation(code)
        || is_bracket_punctuation(code)
        || is_other_cjk_punctuation(code)
}

/// Check if a character is sentence-ending punctuation.
///
/// These punctuation marks always create word boundaries and typically end
/// sentences or clauses in CJK text.
///
/// # Punctuation Characters
///
/// - U+3002: IDEOGRAPHIC FULL STOP (。)
/// - U+FF01: FULLWIDTH EXCLAMATION MARK (！)
/// - U+FF1F: FULLWIDTH QUESTION MARK (？)
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is sentence-ending punctuation
pub fn is_sentence_ending_punctuation(code: u32) -> bool {
    matches!(
        code,
        0x3002  // IDEOGRAPHIC FULL STOP (。)
        | 0xFF01 // FULLWIDTH EXCLAMATION MARK (！)
        | 0xFF1F // FULLWIDTH QUESTION MARK (？)
    )
}

/// Check if a character is enumeration punctuation.
///
/// These punctuation marks separate items in lists or clauses and create
/// boundaries when preceded by significant spacing signals (TJ offset or
/// geometric gap).
///
/// # Punctuation Characters
///
/// - U+3001: IDEOGRAPHIC COMMA (、)
/// - U+FF0C: FULLWIDTH COMMA (，)
/// - U+FF1B: FULLWIDTH SEMICOLON (；)
/// - U+FF1A: FULLWIDTH COLON (：)
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is enumeration punctuation
pub fn is_enumeration_punctuation(code: u32) -> bool {
    matches!(
        code,
        0x3001  // IDEOGRAPHIC COMMA (、)
        | 0xFF0C // FULLWIDTH COMMA (，)
        | 0xFF1B // FULLWIDTH SEMICOLON (；)
        | 0xFF1A // FULLWIDTH COLON (：)
    )
}

/// Check if a character is bracket/parenthesis punctuation.
///
/// These paired punctuation marks typically enclose content and create
/// boundaries at their opening and closing positions.
///
/// # Punctuation Characters
///
/// - U+3008-U+3011: Various angle and corner brackets
/// - U+3014-U+3015: Tortoise shell brackets
/// - U+FF08-U+FF09: Fullwidth parentheses
/// - U+FF3B-U+FF3D: Fullwidth square brackets
/// - U+FF5B-U+FF5D: Fullwidth curly brackets
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is bracket punctuation
pub fn is_bracket_punctuation(code: u32) -> bool {
    matches!(
        code,
        0x3008..=0x3011  // Angle and corner brackets
        | 0x3014..=0x3015 // Tortoise shell brackets
        | 0xFF08..=0xFF09 // Fullwidth parentheses (（）)
        | 0xFF3B..=0xFF3D // Fullwidth square brackets (［］)
        | 0xFF5B..=0xFF5D // Fullwidth curly brackets (｛｝)
    )
}

/// Check if a character is other CJK punctuation.
///
/// This includes miscellaneous CJK punctuation that may create boundaries
/// depending on context.
///
/// # Punctuation Characters
///
/// - U+3000: IDEOGRAPHIC SPACE
/// - U+3003: DITTO MARK
/// - U+30FB: KATAKANA MIDDLE DOT
/// - U+FF0E: FULLWIDTH FULL STOP (．)
/// - U+FF5E: FULLWIDTH TILDE (～)
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is other CJK punctuation
pub fn is_other_cjk_punctuation(code: u32) -> bool {
    matches!(
        code,
        0x3000  // IDEOGRAPHIC SPACE
        | 0x3003 // DITTO MARK
        | 0x30FB // KATAKANA MIDDLE DOT
        | 0xFF0E // FULLWIDTH FULL STOP (．)
        | 0xFF0D // FULLWIDTH HYPHEN-MINUS
        | 0xFF5E // FULLWIDTH TILDE (～)
    )
}

/// Get the boundary confidence score for CJK punctuation, optionally adjusted for text density.
///
/// This function returns a confidence score (0.0-1.0) indicating how strongly
/// a punctuation character signals a word boundary. Higher scores mean stronger
/// boundary indicators.
///
/// Base confidence levels:
/// - **1.0**: Sentence-ending punctuation (。！？) - always creates boundary
/// - **0.9**: Enumeration punctuation (、，；：) - strong boundary signal
/// - **0.8**: Bracket punctuation - paired boundaries
/// - **0.7**: Other CJK punctuation - context-dependent
/// - **0.0**: Not CJK punctuation
///
/// When density is provided, scores are adjusted:
/// - Low density: Multiply by 0.6 (be more conservative)
/// - Medium density: No adjustment (default behavior)
/// - High density: Multiply by 1.4 (be more aggressive)
///
/// # Arguments
///
/// * `code` - Unicode code point to evaluate
/// * `density` - Optional text density classification for adaptive scoring
///
/// # Returns
///
/// Confidence score from 0.0 (no boundary) to 1.0 (definite boundary), possibly adjusted
pub fn get_cjk_punctuation_boundary_score(code: u32, density: Option<TextDensity>) -> f32 {
    let base_score = get_base_punctuation_score(code);

    // Apply density adjustment if provided
    if let Some(d) = density {
        base_score * d.score_multiplier()
    } else {
        base_score
    }
}

/// Get base (unadjusted) punctuation boundary score.
///
/// This is the foundational scoring used before any density adjustments.
/// Separated from the main function for clarity and testability.
///
/// # Arguments
///
/// * `code` - Unicode code point to evaluate
///
/// # Returns
///
/// Base confidence score from 0.0 to 1.0
fn get_base_punctuation_score(code: u32) -> f32 {
    if is_sentence_ending_punctuation(code) {
        1.0 // Definite boundary
    } else if is_enumeration_punctuation(code) {
        0.9 // Strong boundary signal
    } else if is_bracket_punctuation(code) {
        0.8 // Paired boundary
    } else if is_other_cjk_punctuation(code) {
        0.7 // Context-dependent
    } else {
        0.0 // Not CJK punctuation
    }
}

/// Check if a character is opening bracket punctuation.
///
/// Opening brackets typically create a boundary before the enclosed content.
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is an opening bracket
pub fn is_opening_bracket(code: u32) -> bool {
    matches!(
        code,
        0x3008  // LEFT ANGLE BRACKET
        | 0x300A // LEFT DOUBLE ANGLE BRACKET
        | 0x300C // LEFT CORNER BRACKET
        | 0x300E // LEFT WHITE CORNER BRACKET
        | 0x3010 // LEFT BLACK LENTICULAR BRACKET
        | 0x3014 // LEFT TORTOISE SHELL BRACKET
        | 0xFF08 // FULLWIDTH LEFT PARENTHESIS
        | 0xFF3B // FULLWIDTH LEFT SQUARE BRACKET
        | 0xFF5B // FULLWIDTH LEFT CURLY BRACKET
    )
}

/// Check if a character is closing bracket punctuation.
///
/// Closing brackets typically create a boundary after the enclosed content.
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is a closing bracket
pub fn is_closing_bracket(code: u32) -> bool {
    matches!(
        code,
        0x3009  // RIGHT ANGLE BRACKET
        | 0x300B // RIGHT DOUBLE ANGLE BRACKET
        | 0x300D // RIGHT CORNER BRACKET
        | 0x300F // RIGHT WHITE CORNER BRACKET
        | 0x3011 // RIGHT BLACK LENTICULAR BRACKET
        | 0x3015 // RIGHT TORTOISE SHELL BRACKET
        | 0xFF09 // FULLWIDTH RIGHT PARENTHESIS
        | 0xFF3D // FULLWIDTH RIGHT SQUARE BRACKET
        | 0xFF5D // FULLWIDTH RIGHT CURLY BRACKET
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ideographic_full_stop() {
        assert!(is_sentence_ending_punctuation(0x3002));
        assert!(is_fullwidth_punctuation(0x3002));
        assert_eq!(get_cjk_punctuation_boundary_score(0x3002, None), 1.0);
    }

    #[test]
    fn test_fullwidth_question_mark() {
        assert!(is_sentence_ending_punctuation(0xFF1F));
        assert!(is_fullwidth_punctuation(0xFF1F));
        assert_eq!(get_cjk_punctuation_boundary_score(0xFF1F, None), 1.0);
    }

    #[test]
    fn test_fullwidth_exclamation() {
        assert!(is_sentence_ending_punctuation(0xFF01));
        assert!(is_fullwidth_punctuation(0xFF01));
        assert_eq!(get_cjk_punctuation_boundary_score(0xFF01, None), 1.0);
    }

    #[test]
    fn test_ideographic_comma() {
        assert!(is_enumeration_punctuation(0x3001));
        assert!(is_fullwidth_punctuation(0x3001));
        assert_eq!(get_cjk_punctuation_boundary_score(0x3001, None), 0.9);
    }

    #[test]
    fn test_fullwidth_comma() {
        assert!(is_enumeration_punctuation(0xFF0C));
        assert!(is_fullwidth_punctuation(0xFF0C));
        assert_eq!(get_cjk_punctuation_boundary_score(0xFF0C, None), 0.9);
    }

    #[test]
    fn test_fullwidth_semicolon() {
        assert!(is_enumeration_punctuation(0xFF1B));
        assert!(is_fullwidth_punctuation(0xFF1B));
        assert_eq!(get_cjk_punctuation_boundary_score(0xFF1B, None), 0.9);
    }

    #[test]
    fn test_fullwidth_colon() {
        assert!(is_enumeration_punctuation(0xFF1A));
        assert!(is_fullwidth_punctuation(0xFF1A));
        assert_eq!(get_cjk_punctuation_boundary_score(0xFF1A, None), 0.9);
    }

    #[test]
    fn test_fullwidth_parentheses() {
        assert!(is_bracket_punctuation(0xFF08));
        assert!(is_opening_bracket(0xFF08));
        assert!(is_bracket_punctuation(0xFF09));
        assert!(is_closing_bracket(0xFF09));
        assert_eq!(get_cjk_punctuation_boundary_score(0xFF08, None), 0.8);
    }

    #[test]
    fn test_angle_brackets() {
        assert!(is_bracket_punctuation(0x3008));
        assert!(is_opening_bracket(0x3008));
        assert!(is_bracket_punctuation(0x3009));
        assert!(is_closing_bracket(0x3009));
    }

    #[test]
    fn test_ideographic_space() {
        assert!(is_other_cjk_punctuation(0x3000));
        assert!(is_fullwidth_punctuation(0x3000));
        assert_eq!(get_cjk_punctuation_boundary_score(0x3000, None), 0.7);
    }

    #[test]
    fn test_katakana_middle_dot() {
        assert!(is_other_cjk_punctuation(0x30FB));
        assert!(is_fullwidth_punctuation(0x30FB));
    }

    #[test]
    fn test_non_cjk_punctuation() {
        // ASCII period should not be CJK punctuation
        assert!(!is_fullwidth_punctuation(0x002E));
        assert_eq!(get_cjk_punctuation_boundary_score(0x002E, None), 0.0);

        // ASCII comma should not be CJK punctuation
        assert!(!is_fullwidth_punctuation(0x002C));
        assert_eq!(get_cjk_punctuation_boundary_score(0x002C, None), 0.0);
    }

    #[test]
    fn test_boundary_score_ordering() {
        // Sentence-ending > Enumeration > Bracket > Other
        assert!(
            get_cjk_punctuation_boundary_score(0x3002, None)
                > get_cjk_punctuation_boundary_score(0x3001, None)
        );
        assert!(
            get_cjk_punctuation_boundary_score(0x3001, None)
                > get_cjk_punctuation_boundary_score(0xFF08, None)
        );
        assert!(
            get_cjk_punctuation_boundary_score(0xFF08, None)
                > get_cjk_punctuation_boundary_score(0x30FB, None)
        );
    }

    // Text density tests
    #[test]
    fn test_text_density_classify_low() {
        let density = TextDensity::classify(400, 1); // 400 chars per page
        assert_eq!(density, TextDensity::Low);
    }

    #[test]
    fn test_text_density_classify_medium() {
        let density = TextDensity::classify(1200, 1); // 1200 chars per page
        assert_eq!(density, TextDensity::Medium);
    }

    #[test]
    fn test_text_density_classify_high() {
        let density = TextDensity::classify(3000, 1); // 3000 chars per page
        assert_eq!(density, TextDensity::High);
    }

    #[test]
    fn test_text_density_classify_boundary_low_medium() {
        let density_500 = TextDensity::classify(500, 1); // Boundary at 500
        let density_501 = TextDensity::classify(501, 1); // Just above boundary
        assert_eq!(density_500, TextDensity::Low);
        assert_eq!(density_501, TextDensity::Medium);
    }

    #[test]
    fn test_text_density_classify_boundary_medium_high() {
        let density_2000 = TextDensity::classify(2000, 1); // Boundary at 2000
        let density_2001 = TextDensity::classify(2001, 1); // Just above boundary
        assert_eq!(density_2000, TextDensity::Medium);
        assert_eq!(density_2001, TextDensity::High);
    }

    #[test]
    fn test_text_density_score_multiplier_low() {
        assert_eq!(TextDensity::Low.score_multiplier(), 0.6);
    }

    #[test]
    fn test_text_density_score_multiplier_medium() {
        assert_eq!(TextDensity::Medium.score_multiplier(), 1.0);
    }

    #[test]
    fn test_text_density_score_multiplier_high() {
        assert_eq!(TextDensity::High.score_multiplier(), 1.4);
    }

    #[test]
    fn test_punctuation_score_with_density_low() {
        // Ideographic full stop normally scores 1.0
        // With low density (0.6×), becomes 0.6
        let base_score = get_base_punctuation_score(0x3002);
        assert_eq!(base_score, 1.0);

        let adjusted = get_cjk_punctuation_boundary_score(0x3002, Some(TextDensity::Low));
        assert!((adjusted - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_punctuation_score_with_density_medium() {
        // Medium density should not change the score
        let base_score = get_base_punctuation_score(0x3002);
        let adjusted = get_cjk_punctuation_boundary_score(0x3002, Some(TextDensity::Medium));
        assert_eq!(adjusted, base_score);
    }

    #[test]
    fn test_punctuation_score_with_density_high() {
        // Angle bracket normally scores 0.8
        // With high density (1.4×), becomes 1.12
        let adjusted = get_cjk_punctuation_boundary_score(0x3008, Some(TextDensity::High));
        assert!((adjusted - 1.12).abs() < 0.01);
    }

    #[test]
    fn test_punctuation_score_enumeration_with_density_low() {
        // Ideographic comma normally scores 0.9
        // With low density (0.6×), becomes 0.54
        let adjusted = get_cjk_punctuation_boundary_score(0x3001, Some(TextDensity::Low));
        assert!((adjusted - 0.54).abs() < 0.01);
    }

    #[test]
    fn test_punctuation_score_enumeration_with_density_high() {
        // Ideographic comma normally scores 0.9
        // With high density (1.4×), becomes 1.26
        let adjusted = get_cjk_punctuation_boundary_score(0x3001, Some(TextDensity::High));
        assert!((adjusted - 1.26).abs() < 0.01);
    }

    #[test]
    fn test_punctuation_score_without_density() {
        // Should use base score when density is None
        let score_with_none = get_cjk_punctuation_boundary_score(0x3002, None);
        let base_score = get_base_punctuation_score(0x3002);
        assert_eq!(score_with_none, base_score);
    }

    #[test]
    fn test_density_classify_zero_pages() {
        // Should default to Medium when pages = 0
        let density = TextDensity::classify(1000, 0);
        assert_eq!(density, TextDensity::Medium);
    }

    #[test]
    fn test_density_classify_multi_page() {
        // Test averaging across multiple pages
        let density_per_page_1000 = TextDensity::classify(3000, 3); // 1000 per page
        assert_eq!(density_per_page_1000, TextDensity::Medium);

        let density_per_page_300 = TextDensity::classify(900, 3); // 300 per page
        assert_eq!(density_per_page_300, TextDensity::Low);

        let density_per_page_1500 = TextDensity::classify(9000, 6); // 1500 per page
        assert_eq!(density_per_page_1500, TextDensity::Medium);
    }

    #[test]
    fn test_bracket_scores_all_densities() {
        // Bracket punctuation (0.8) should scale correctly with all densities
        let low = get_cjk_punctuation_boundary_score(0xFF08, Some(TextDensity::Low));
        let medium = get_cjk_punctuation_boundary_score(0xFF08, Some(TextDensity::Medium));
        let high = get_cjk_punctuation_boundary_score(0xFF08, Some(TextDensity::High));

        assert!((low - 0.48).abs() < 0.01); // 0.8 * 0.6
        assert!((medium - 0.8).abs() < 0.01); // 0.8 * 1.0
        assert!((high - 1.12).abs() < 0.01); // 0.8 * 1.4
    }
}
