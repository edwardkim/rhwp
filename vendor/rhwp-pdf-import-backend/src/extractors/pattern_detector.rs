//! Email and URL pattern detection for text extraction.
//!
//! This module provides pattern detection to identify email addresses and URLs
//! in character sequences, marking them as protected from word boundary splitting.
//! This ensures patterns like `user@example.com` and `http://example.com` are
//! preserved as single tokens during text extraction.
//!
//! # Pattern Detection Strategy
//!
//! The detector uses conservative heuristics:
//! - Email: Look for '@' character with domain pattern (contains '.')
//! - URL: Look for scheme patterns (http://, https://, ftp://, mailto:)
//!
//! These patterns are non-breaking - they only mark characters as protected,
//! they don't change the text extraction flow.

use crate::error::Result;
use crate::text::CharacterInfo;

/// Configuration for pattern preservation behavior.
///
/// Controls which patterns are detected and preserved during text extraction.
#[derive(Debug, Clone)]
pub struct PatternPreservationConfig {
    /// Enable pattern preservation (master switch)
    pub preserve_patterns: bool,

    /// Detect and preserve email addresses
    pub detect_emails: bool,

    /// Detect and preserve URLs
    pub detect_urls: bool,
}

impl Default for PatternPreservationConfig {
    fn default() -> Self {
        Self {
            preserve_patterns: true,
            detect_emails: true,
            detect_urls: true,
        }
    }
}

/// Pattern detector for email and URL preservation.
///
/// This struct provides methods to detect email and URL patterns in character
/// sequences and mark them as protected from word boundary splitting.
#[derive(Debug)]
pub struct PatternDetector;

impl PatternDetector {
    /// Create a new pattern detector.
    pub fn new(_config: PatternPreservationConfig) -> Self {
        Self
    }

    /// Create a pattern detector with default configuration.
    pub fn default_config() -> Self {
        Self
    }

    /// Check if characters contain an email pattern.
    ///
    /// Pattern: local@domain where domain contains at least one '.'
    /// Examples: user@example.com, user+tag@company.co.uk
    ///
    /// # Arguments
    ///
    /// * `characters` - Sequence of characters to check
    ///
    /// # Returns
    ///
    /// true if an email pattern is detected
    pub fn has_email_pattern(characters: &[CharacterInfo]) -> bool {
        if characters.is_empty() {
            return false;
        }

        // Look for @ character
        let at_position = characters.iter().position(|ch| ch.code == 0x40); // '@'

        if let Some(at_idx) = at_position {
            // Check if there's a domain part after @ with at least one '.'
            let after_at = &characters[at_idx + 1..];

            // Domain should have at least one dot
            let has_dot = after_at.iter().any(|ch| ch.code == 0x2E); // '.'

            // Domain should have non-whitespace characters
            let has_domain_chars = after_at
                .iter()
                .any(|ch| ch.code != 0x20 && ch.code != 0x09 && ch.code != 0x0A && ch.code != 0x0D);

            return has_dot && has_domain_chars;
        }

        false
    }

    /// Check if characters contain a URL pattern.
    ///
    /// Pattern: `scheme://` where scheme is http, https, ftp, or `mailto:`
    /// Examples: `http://example.com`, `https://example.com/path`
    ///
    /// # Arguments
    ///
    /// * `characters` - Sequence of characters to check
    ///
    /// # Returns
    ///
    /// true if a URL pattern is detected
    pub fn has_url_pattern(characters: &[CharacterInfo]) -> bool {
        if characters.len() < 7 {
            // Minimum: "http://" is 7 characters
            return false;
        }

        // Convert characters to string for pattern matching
        let text: String = characters
            .iter()
            .filter_map(|ch| char::from_u32(ch.code))
            .collect();

        let text_lower = text.to_lowercase();

        // Check for scheme patterns
        text_lower.contains("http://")
            || text_lower.contains("https://")
            || text_lower.contains("ftp://")
            || text_lower.contains("mailto:")
    }

    /// Mark email and URL contexts in character array.
    ///
    /// This is the main entry point for pattern detection. It scans the character
    /// array for email and URL patterns and sets the `protected_from_split` flag
    /// on matching characters.
    ///
    /// # Arguments
    ///
    /// * `characters` - Mutable slice of characters to mark
    /// * `config` - Configuration for pattern detection
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::extractors::pattern_detector::{PatternDetector, PatternPreservationConfig};
    ///
    /// let config = PatternPreservationConfig::default();
    /// let mut characters = vec![/* ... */];
    /// PatternDetector::mark_pattern_contexts(&mut characters, &config)?;
    /// ```
    pub fn mark_pattern_contexts(
        characters: &mut [CharacterInfo],
        config: &PatternPreservationConfig,
    ) -> Result<()> {
        // Master switch check
        if !config.preserve_patterns {
            return Ok(());
        }

        // Email detection
        if config.detect_emails {
            Self::mark_email_contexts(characters);
        }

        // URL detection
        if config.detect_urls {
            Self::mark_url_contexts(characters);
        }

        Ok(())
    }

    /// Mark email contexts in character array.
    ///
    /// Finds email patterns and marks all characters in the email as protected.
    fn mark_email_contexts(characters: &mut [CharacterInfo]) {
        if characters.is_empty() {
            return;
        }

        // Find all @ positions
        let at_positions: Vec<usize> = characters
            .iter()
            .enumerate()
            .filter(|(_, ch)| ch.code == 0x40) // '@'
            .map(|(idx, _)| idx)
            .collect();

        for at_idx in at_positions {
            // Check if this looks like an email
            if !Self::is_email_at_position(characters, at_idx) {
                continue;
            }

            // Find email start (go backward to find local part)
            let start_idx = Self::find_email_start(characters, at_idx);

            // Find email end (go forward to find domain part)
            let end_idx = Self::find_email_end(characters, at_idx);

            // Mark all characters in the email range as protected
            for ch in &mut characters[start_idx..=end_idx] {
                ch.protected_from_split = true;
            }
        }
    }

    /// Check if @ at the given position is part of an email.
    fn is_email_at_position(characters: &[CharacterInfo], at_idx: usize) -> bool {
        // Must have characters before @
        if at_idx == 0 {
            return false;
        }

        // Must have characters after @
        if at_idx >= characters.len() - 1 {
            return false;
        }

        // Check for domain with dot
        let after_at = &characters[at_idx + 1..];
        let has_dot = after_at.iter().any(|ch| ch.code == 0x2E); // '.'

        has_dot
    }

    /// Find the start of an email (local part).
    fn find_email_start(characters: &[CharacterInfo], at_idx: usize) -> usize {
        let mut start = at_idx;

        // Go backward while we see email-valid characters
        while start > 0 {
            let ch = &characters[start - 1];
            if Self::is_email_char(ch.code) {
                start -= 1;
            } else {
                break;
            }
        }

        start
    }

    /// Find the end of an email (domain part).
    fn find_email_end(characters: &[CharacterInfo], at_idx: usize) -> usize {
        let mut end = at_idx;

        // Go forward while we see email-valid characters
        while end < characters.len() - 1 {
            let ch = &characters[end + 1];
            if Self::is_email_char(ch.code) {
                end += 1;
            } else {
                break;
            }
        }

        end
    }

    /// Check if a character is valid in an email address.
    fn is_email_char(code: u32) -> bool {
        matches!(code,
            0x30..=0x39 | // 0-9
            0x41..=0x5A | // A-Z
            0x61..=0x7A | // a-z
            0x2D | // -
            0x2E | // .
            0x5F | // _
            0x2B | // +
            0x40   // @
        )
    }

    /// Mark URL contexts in character array.
    ///
    /// Finds URL patterns and marks all characters in the URL as protected.
    fn mark_url_contexts(characters: &mut [CharacterInfo]) {
        if characters.len() < 7 {
            return;
        }

        // Look for scheme patterns
        let schemes = [
            ("http://", 7),
            ("https://", 8),
            ("ftp://", 6),
            ("mailto:", 7),
        ];

        for i in 0..characters.len() {
            for (scheme, len) in &schemes {
                if i + len > characters.len() {
                    continue;
                }

                // Check if characters match scheme
                let slice = &characters[i..i + len];
                if Self::matches_scheme(slice, scheme) {
                    // Find end of URL
                    let end_idx = Self::find_url_end(characters, i + len);

                    // Mark all characters in URL range as protected
                    for ch in &mut characters[i..=end_idx] {
                        ch.protected_from_split = true;
                    }

                    // Skip past this URL
                    break;
                }
            }
        }
    }

    /// Check if character slice matches a URL scheme (case-insensitive).
    fn matches_scheme(chars: &[CharacterInfo], scheme: &str) -> bool {
        if chars.len() != scheme.len() {
            return false;
        }

        for (ch, scheme_char) in chars.iter().zip(scheme.chars()) {
            let ch_lower = char::from_u32(ch.code)
                .map(|c| c.to_lowercase().next().unwrap_or(c))
                .unwrap_or('\0');

            if ch_lower != scheme_char {
                return false;
            }
        }

        true
    }

    /// Find the end of a URL (go forward until whitespace or end).
    fn find_url_end(characters: &[CharacterInfo], start_idx: usize) -> usize {
        let mut end = start_idx;

        while end < characters.len() {
            let ch = &characters[end];
            if Self::is_url_char(ch.code) {
                end += 1;
            } else {
                break;
            }
        }

        // End is now one past the last URL character, so return end - 1
        if end > start_idx {
            end - 1
        } else {
            start_idx
        }
    }

    /// Check if a character is valid in a URL.
    fn is_url_char(code: u32) -> bool {
        // URL characters: alphanumeric, and common URL punctuation
        matches!(code,
            0x30..=0x39 | // 0-9
            0x41..=0x5A | // A-Z
            0x61..=0x7A | // a-z
            0x2D | // -
            0x2E | // .
            0x5F | // _
            0x7E | // ~
            0x3A | // :
            0x2F | // /
            0x3F | // ?
            0x23 | // #
            0x5B | // [
            0x5D | // ]
            0x40 | // @
            0x21 | // !
            0x24 | // $
            0x26 | // &
            0x27 | // '
            0x28 | // (
            0x29 | // )
            0x2A | // *
            0x2B | // +
            0x2C | // ,
            0x3B | // ;
            0x3D | // =
            0x25   // %
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_char_info(code: u32) -> CharacterInfo {
        CharacterInfo {
            code,
            glyph_id: Some(1),
            width: 0.5,
            x_position: 0.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        }
    }

    fn string_to_chars(s: &str) -> Vec<CharacterInfo> {
        s.chars().map(|ch| create_char_info(ch as u32)).collect()
    }

    #[test]
    fn test_email_pattern_detection() {
        let chars = string_to_chars("user@example.com");
        assert!(PatternDetector::has_email_pattern(&chars), "Should detect email pattern");
    }

    #[test]
    fn test_email_pattern_no_domain() {
        let chars = string_to_chars("user@example");
        assert!(
            !PatternDetector::has_email_pattern(&chars),
            "Should not detect email without dot in domain"
        );
    }

    #[test]
    fn test_url_pattern_http() {
        let chars = string_to_chars("http://example.com");
        assert!(PatternDetector::has_url_pattern(&chars), "Should detect http:// URL");
    }

    #[test]
    fn test_url_pattern_https() {
        let chars = string_to_chars("https://example.com");
        assert!(PatternDetector::has_url_pattern(&chars), "Should detect https:// URL");
    }

    #[test]
    fn test_email_protection() {
        let mut chars = string_to_chars("user@example.com");
        let config = PatternPreservationConfig::default();

        PatternDetector::mark_pattern_contexts(&mut chars, &config).unwrap();

        // All characters should be protected
        for (i, ch) in chars.iter().enumerate() {
            assert!(ch.protected_from_split, "Character {} should be protected", i);
        }
    }

    #[test]
    fn test_url_protection() {
        let mut chars = string_to_chars("http://example.com");
        let config = PatternPreservationConfig::default();

        PatternDetector::mark_pattern_contexts(&mut chars, &config).unwrap();

        // All characters should be protected
        for (i, ch) in chars.iter().enumerate() {
            assert!(ch.protected_from_split, "Character {} should be protected", i);
        }
    }

    #[test]
    fn test_pattern_detection_disabled() {
        let mut chars = string_to_chars("user@example.com");
        let config = PatternPreservationConfig {
            preserve_patterns: false,
            detect_emails: true,
            detect_urls: true,
        };

        PatternDetector::mark_pattern_contexts(&mut chars, &config).unwrap();

        // No characters should be protected when disabled
        for ch in &chars {
            assert!(!ch.protected_from_split, "Characters should not be protected when disabled");
        }
    }

    #[test]
    fn test_mixed_content() {
        let mut chars = string_to_chars("Contact user@example.com for more info");
        let config = PatternPreservationConfig::default();

        PatternDetector::mark_pattern_contexts(&mut chars, &config).unwrap();

        // Extract email portion
        let email_start = "Contact ".len();
        let email_end = email_start + "user@example.com".len();

        // Email characters should be protected
        for i in email_start..email_end {
            assert!(chars[i].protected_from_split, "Email character {} should be protected", i);
        }

        // Non-email characters should not be protected
        for i in 0..email_start {
            assert!(
                !chars[i].protected_from_split,
                "Non-email character {} should not be protected",
                i
            );
        }
    }
}
