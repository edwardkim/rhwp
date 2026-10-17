//! Ligature Expansion Enhancement
//!
//! This module implements intelligent ligature splitting at word boundaries.
//! When a ligature (fi, fl, ffi, ffl, ff) is followed by a word boundary,
//! it is split into component characters. When not followed by a boundary,
//! it is kept as a ligature.
//!
//! Per ISO 32000-1:2008 Section 9.10, ligatures are Unicode characters
//! (U+FB00-U+FB04) that represent multiple glyphs as a single character.
//! This module provides spec-compliant ligature expansion that integrates
//! with word boundary detection.

#![forbid(unsafe_code)]

use crate::text::{BoundaryContext, CharacterInfo};

/// Decision about whether to keep or split a ligature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LigatureDecision {
    /// Keep ligature as single unit (no boundary after)
    Keep,
    /// Split ligature into component characters (boundary detected)
    Split,
}

/// Lightweight decision maker for ligature splitting.
///
/// This struct determines whether a ligature should be split based on
/// word boundary signals:
/// - TJ offset values (negative offsets indicate spacing)
/// - Geometric gaps between characters
/// - Context from boundary detection
pub struct LigatureDecisionMaker;

impl LigatureDecisionMaker {
    /// Decide whether a ligature should be split or kept intact.
    ///
    /// Decision logic:
    /// 1. If next_char is None (ligature at end), return Keep
    /// 2. If next_char has significant TJ offset (< -100), return Split
    /// 3. If next_char has large geometric gap (>= 0.5 * font_size), return Split
    /// 4. Otherwise return Keep
    ///
    /// # Arguments
    ///
    /// * `char_info` - The ligature character information
    /// * `context` - Boundary detection context (font metrics)
    /// * `next_char` - Optional next character in the stream
    ///
    /// # Returns
    ///
    /// LigatureDecision indicating whether to Keep or Split the ligature
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ligature = CharacterInfo { code: 0xFB01, ... }; // fi ligature
    /// let next = Some(CharacterInfo { code: 0x63, tj_offset: Some(-150), ... });
    /// let context = BoundaryContext::new(12.0);
    ///
    /// let decision = LigatureDecisionMaker::decide(&ligature, &context, next.as_ref());
    /// assert_eq!(decision, LigatureDecision::Split); // Large TJ offset triggers split
    /// ```
    pub fn decide(
        char_info: &CharacterInfo,
        context: &BoundaryContext,
        next_char: Option<&CharacterInfo>,
    ) -> LigatureDecision {
        // Rule 1: No next character means ligature is at end of text
        // Keep it as-is (no boundary to split at)
        let next = match next_char {
            Some(c) => c,
            None => return LigatureDecision::Keep,
        };

        // Rule 2: Check TJ offset for explicit spacing signal
        // TJ offset threshold: -100 (values more negative indicate word boundaries)
        // Per PDF Spec Section 9.4.4, negative TJ values insert extra space
        if let Some(tj_offset) = next.tj_offset {
            if tj_offset < -100 {
                return LigatureDecision::Split;
            }
        }

        // Rule 3: Check geometric gap between ligature and next character
        // Gap threshold: 0.5 * font_size (conservative boundary detection)
        // Use strict comparison (gap > threshold) to avoid edge cases
        let ligature_end = char_info.x_position + char_info.width;
        let gap = next.x_position - ligature_end;
        let threshold = context.font_size * 0.5;

        if gap > threshold {
            return LigatureDecision::Split;
        }

        // Rule 4: No boundary signals detected, keep ligature intact
        LigatureDecision::Keep
    }
}

/// Get the component characters for a ligature.
///
/// Returns the string of characters that make up the ligature.
/// Supports standard Unicode ligatures U+FB00-U+FB04.
///
/// # Arguments
///
/// * `ligature` - The ligature character
///
/// # Returns
///
/// Some(component_string) if the character is a ligature, None otherwise
///
/// # Examples
///
/// ```
/// use pdf_oxide::text::ligature_processor::get_ligature_components;
///
/// assert_eq!(get_ligature_components('ﬁ'), Some("fi"));
/// assert_eq!(get_ligature_components('ﬂ'), Some("fl"));
/// assert_eq!(get_ligature_components('a'), None);
/// ```
pub fn get_ligature_components(ligature: char) -> Option<&'static str> {
    match ligature {
        'ﬀ' => Some("ff"),  // U+FB00 - LATIN SMALL LIGATURE FF
        'ﬁ' => Some("fi"),  // U+FB01 - LATIN SMALL LIGATURE FI
        'ﬂ' => Some("fl"),  // U+FB02 - LATIN SMALL LIGATURE FL
        'ﬃ' => Some("ffi"), // U+FB03 - LATIN SMALL LIGATURE FFI
        'ﬄ' => Some("ffl"), // U+FB04 - LATIN SMALL LIGATURE FFL
        'ﬅ' => Some("st"),  // U+FB05 - LATIN SMALL LIGATURE LONG S T
        'ﬆ' => Some("st"),  // U+FB06 - LATIN SMALL LIGATURE ST
        _ => None,
    }
}

/// Expand a ligature character to component characters with proportional widths.
///
/// Distributes the ligature's width equally among component characters.
/// For example, "fi" with width 500.0 becomes [('f', 250.0), ('i', 250.0)].
///
/// # Arguments
///
/// * `ligature` - The ligature character to expand
/// * `original_width` - The width of the ligature in text space units
///
/// # Returns
///
/// Vector of (char, width) tuples representing the component characters.
/// Returns empty vector if the character is not a ligature.
///
/// # Examples
///
/// ```
/// use pdf_oxide::text::ligature_processor::expand_ligature_to_chars;
///
/// let components = expand_ligature_to_chars('ﬁ', 500.0);
/// assert_eq!(components.len(), 2);
/// assert_eq!(components[0], ('f', 250.0));
/// assert_eq!(components[1], ('i', 250.0));
/// ```
pub fn expand_ligature_to_chars(ligature: char, original_width: f32) -> Vec<(char, f32)> {
    // Get component string for this ligature
    let components_str = match get_ligature_components(ligature) {
        Some(s) => s,
        None => return Vec::new(), // Not a ligature
    };

    // Calculate proportional width for each component
    let num_components = components_str.chars().count();
    if num_components == 0 {
        return Vec::new();
    }

    let width_per_component = original_width / num_components as f32;

    // Create (char, width) tuples for each component
    components_str
        .chars()
        .map(|c| (c, width_per_component))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ligature_components_all_ligatures() {
        assert_eq!(get_ligature_components('ﬀ'), Some("ff"));
        assert_eq!(get_ligature_components('ﬁ'), Some("fi"));
        assert_eq!(get_ligature_components('ﬂ'), Some("fl"));
        assert_eq!(get_ligature_components('ﬃ'), Some("ffi"));
        assert_eq!(get_ligature_components('ﬄ'), Some("ffl"));
    }

    #[test]
    fn test_get_ligature_components_non_ligatures() {
        assert_eq!(get_ligature_components('a'), None);
        assert_eq!(get_ligature_components('A'), None);
        assert_eq!(get_ligature_components('1'), None);
        assert_eq!(get_ligature_components(' '), None);
    }

    #[test]
    fn test_expand_ligature_fi() {
        let result = expand_ligature_to_chars('ﬁ', 500.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ('f', 250.0));
        assert_eq!(result[1], ('i', 250.0));
    }

    #[test]
    fn test_expand_ligature_ffl() {
        let result = expand_ligature_to_chars('ﬄ', 600.0);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ('f', 200.0));
        assert_eq!(result[1], ('f', 200.0));
        assert_eq!(result[2], ('l', 200.0));
    }

    #[test]
    fn test_expand_non_ligature() {
        let result = expand_ligature_to_chars('a', 400.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_ligature_decision_no_next_char() {
        let char_info = CharacterInfo {
            code: 0xFB01,
            glyph_id: Some(1),
            width: 500.0,
            x_position: 0.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: None,
            protected_from_split: false,
        };

        let context = BoundaryContext::new(12.0);
        let decision = LigatureDecisionMaker::decide(&char_info, &context, None);

        assert_eq!(decision, LigatureDecision::Keep);
    }

    #[test]
    fn test_ligature_decision_large_tj_offset() {
        let ligature = CharacterInfo {
            code: 0xFB01,
            glyph_id: Some(1),
            width: 500.0,
            x_position: 0.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: None,
            protected_from_split: false,
        };

        let next = CharacterInfo {
            code: 0x63,
            glyph_id: Some(2),
            width: 400.0,
            x_position: 500.0,
            tj_offset: Some(-150),
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let context = BoundaryContext::new(12.0);
        let decision = LigatureDecisionMaker::decide(&ligature, &context, Some(&next));

        assert_eq!(decision, LigatureDecision::Split);
    }

    #[test]
    fn test_ligature_decision_large_gap() {
        let ligature = CharacterInfo {
            code: 0xFB01,
            glyph_id: Some(1),
            width: 500.0,
            x_position: 0.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: None,
            protected_from_split: false,
        };

        let next = CharacterInfo {
            code: 0x61,
            glyph_id: Some(2),
            width: 400.0,
            x_position: 510.0, // Gap of 10.0 (>= 6.0 threshold)
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let context = BoundaryContext::new(12.0);
        let decision = LigatureDecisionMaker::decide(&ligature, &context, Some(&next));

        assert_eq!(decision, LigatureDecision::Split);
    }

    #[test]
    fn test_ligature_decision_keep() {
        let ligature = CharacterInfo {
            code: 0xFB01,
            glyph_id: Some(1),
            width: 500.0,
            x_position: 0.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: None,
            protected_from_split: false,
        };

        let next = CharacterInfo {
            code: 0x6E,
            glyph_id: Some(2),
            width: 400.0,
            x_position: 500.0, // No gap, no TJ offset
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let context = BoundaryContext::new(12.0);
        let decision = LigatureDecisionMaker::decide(&ligature, &context, Some(&next));

        assert_eq!(decision, LigatureDecision::Keep);
    }
}
