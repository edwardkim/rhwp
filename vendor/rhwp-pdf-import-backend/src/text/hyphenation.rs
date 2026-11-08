//! Hyphenation-aware text reconstruction for PDF extraction.
//!
//! This module handles the reconstruction of words that have been split across
//! line breaks with hyphens. Per PDF Specification Section 5.3.4, soft hyphens
//! (U+00AD) indicate optional line breaks, while hard hyphens (U+002D) are
//! always present.
//!
//! # Problem
//!
//! PDFs often contain text like:
//! - "Govern-" (line 1) + "ment" (line 2) → Should become "Government"
//! - "content-" (line 1) + "coding" (line 2) → Should remain "content-coding" (compound)
//!
//! # Solution
//!
//! This module provides post-processing to:
//! 1. Detect line-ending hyphens that indicate word continuation
//! 2. Distinguish between soft hyphens (word breaks) and hard hyphens (compound words)
//! 3. Reconstruct complete words while preserving intentional hyphenation
//!
//! # PDF Spec References
//!
//! - Section 5.3.4: String Objects (soft vs hard hyphens)
//! - Section 14.6: Marked Content (ActualText for true character sequences)

/// Hyphenation handler for reconstructing split words.
///
/// Processes text to join words that were split across line breaks with hyphens.
#[derive(Debug, Clone)]
pub struct HyphenationHandler {
    /// Minimum word length for the second part to trigger joining
    /// (prevents joining single letters that might be list markers)
    min_continuation_length: usize,

    /// Whether to preserve compound words (e.g., "content-coding")
    preserve_compounds: bool,
}

impl Default for HyphenationHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl HyphenationHandler {
    /// Create a new hyphenation handler with default settings.
    pub fn new() -> Self {
        Self {
            min_continuation_length: 2,
            preserve_compounds: true,
        }
    }

    /// Set minimum continuation length for word joining.
    pub fn with_min_continuation_length(mut self, len: usize) -> Self {
        self.min_continuation_length = len;
        self
    }

    /// Set whether to preserve compound words.
    pub fn with_preserve_compounds(mut self, preserve: bool) -> Self {
        self.preserve_compounds = preserve;
        self
    }

    /// Check if a line ends with a continuation hyphen.
    ///
    /// A continuation hyphen is:
    /// - A soft hyphen (U+00AD) - ALWAYS indicates word continuation
    /// - A hard hyphen (U+002D) at the end of a line with alphabetic text before
    /// - NOT a hyphen followed by whitespace (e.g., "- " is not continuation)
    ///
    /// Per PDF Specification Section 5.3.4 and Unicode Standard:
    /// - Soft hyphen (U+00AD) marks an optional line break point
    /// - When a word is broken at a soft hyphen, it should be rejoined without the hyphen
    ///
    /// # Arguments
    ///
    /// * `text` - The text to check
    ///
    /// # Returns
    ///
    /// `true` if the text ends with a continuation hyphen
    pub fn is_continuation_hyphen(text: &str) -> bool {
        let trimmed = text.trim_end();
        if trimmed.is_empty() {
            return false;
        }

        // Check for soft hyphen (U+00AD) - ALWAYS a continuation marker
        // Per PDF spec, soft hyphens indicate optional line breaks
        if trimmed.ends_with('\u{00AD}') {
            // Verify there's alphabetic content before the soft hyphen
            let before_hyphen = &trimmed[..trimmed.len() - '\u{00AD}'.len_utf8()];
            return before_hyphen
                .chars()
                .last()
                .is_some_and(|c| c.is_alphabetic());
        }

        // Check for hard hyphen (U+002D)
        if !trimmed.ends_with('-') {
            return false;
        }

        // Check that there's at least one letter before the hyphen
        // (avoids matching bullet points like "- item")
        let before_hyphen = &trimmed[..trimmed.len() - 1];
        before_hyphen
            .chars()
            .last()
            .is_some_and(|c| c.is_alphabetic())
    }

    /// Check if a word appears to be a compound word that should keep its hyphen.
    ///
    /// Compound words are identified by:
    /// - Common prefixes: non-, self-, re-, pre-, anti-, etc.
    /// - Common technical patterns: content-type, user-agent, etc.
    ///
    /// # Arguments
    ///
    /// * `first_part` - The part before the hyphen
    /// * `second_part` - The part after the hyphen
    ///
    /// # Returns
    ///
    /// `true` if this appears to be a compound word
    fn is_compound_word(first_part: &str, second_part: &str) -> bool {
        let first_lower = first_part.to_lowercase();

        // Common compound prefixes that should keep their hyphens
        let compound_prefixes = [
            "self", "non", "anti", "pre", "post", "re", "co", "ex", "multi", "semi", "sub",
            "super", "ultra", "under", "over", "cross", "inter", "intra", "counter", "mid", "well",
            "ill", "all", "half", "high", "low", "full", "part", "short", "long", "hard", "soft",
        ];

        // If first part is a compound prefix, likely a compound word
        if compound_prefixes.contains(&first_lower.as_str()) {
            return true;
        }

        // Technical compounds (common in RFC/technical documents)
        let technical_patterns = [
            ("content", "type"),
            ("content", "length"),
            ("content", "encoding"),
            ("content", "coding"),
            ("user", "agent"),
            ("cache", "control"),
            ("product", "version"),
            ("media", "type"),
        ];

        for (prefix, suffix) in &technical_patterns {
            if first_lower == *prefix && second_part.to_lowercase().starts_with(suffix) {
                return true;
            }
        }

        // If second part starts with lowercase, likely a compound (not sentence continuation)
        // e.g., "content-coding" vs "Govern-" + "ment"
        if let Some(first_char) = second_part.chars().next() {
            if first_char.is_lowercase()
                && first_part.chars().last().is_some_and(|c| c.is_lowercase())
            {
                // Both parts lowercase - more likely compound
                // But check if the combined form is a common word
                let combined = format!("{}{}", first_part, second_part);
                if is_common_word(&combined) {
                    return false; // It's a regular word split across lines
                }
                return true;
            }
        }

        false
    }

    /// Process a single line pair to potentially join hyphenated words.
    ///
    /// # Arguments
    ///
    /// * `current_line` - The line that may end with a hyphen
    /// * `next_line` - The line that may continue the word
    ///
    /// # Returns
    ///
    /// A tuple of (processed_text, consumed_next_line) where:
    /// - `processed_text` is the current line (possibly with word joined)
    /// - `consumed_next_line` is true if the next line was consumed (joined)
    pub fn process_line_pair(&self, current_line: &str, next_line: &str) -> (String, bool) {
        let trimmed_current = current_line.trim_end();

        // Check if current line ends with continuation hyphen
        if !Self::is_continuation_hyphen(trimmed_current) {
            return (current_line.to_string(), false);
        }

        let trimmed_next = next_line.trim_start();

        // Get the first word of next line
        let next_word = trimmed_next.split_whitespace().next().unwrap_or("");

        // Check minimum length for continuation
        if next_word.len() < self.min_continuation_length {
            return (current_line.to_string(), false);
        }

        // Get the last word (before hyphen) of current line
        // Handle both soft hyphen (U+00AD, 2 bytes UTF-8) and hard hyphen (U+002D, 1 byte)
        let without_hyphen = if trimmed_current.ends_with('\u{00AD}') {
            &trimmed_current[..trimmed_current.len() - '\u{00AD}'.len_utf8()]
        } else {
            &trimmed_current[..trimmed_current.len() - 1]
        };
        let last_word = without_hyphen
            .split_whitespace()
            .next_back()
            .unwrap_or(without_hyphen);

        // Check if this is a compound word that should keep hyphen
        // Note: Soft hyphens are NEVER compound word markers, only hard hyphens
        let is_soft_hyphen = trimmed_current.ends_with('\u{00AD}');
        if !is_soft_hyphen
            && self.preserve_compounds
            && Self::is_compound_word(last_word, next_word)
        {
            return (current_line.to_string(), false);
        }

        // Join the words (remove hyphen, concatenate)
        let hyphen_len = if is_soft_hyphen {
            '\u{00AD}'.len_utf8()
        } else {
            1
        };
        let prefix = &trimmed_current[..trimmed_current.len() - last_word.len() - hyphen_len];
        let joined_word = format!("{}{}", last_word, next_word);

        // Reconstruct the line with joined word
        let rest_of_next = trimmed_next[next_word.len()..].trim_start();
        let mut result = if prefix.is_empty() {
            joined_word
        } else {
            format!("{} {}", prefix.trim_end(), joined_word)
        };

        // Add remaining text from next line if any
        if !rest_of_next.is_empty() {
            result.push(' ');
            result.push_str(rest_of_next);
        }

        (result, true)
    }

    /// Process a complete text block and join hyphenated words.
    ///
    /// # Arguments
    ///
    /// * `text` - The complete text block to process
    ///
    /// # Returns
    ///
    /// The text with hyphenated words joined
    pub fn process_text(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return String::new();
        }

        let mut result = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            if i + 1 < lines.len() {
                let (processed, consumed) = self.process_line_pair(lines[i], lines[i + 1]);
                result.push(processed);
                if consumed {
                    i += 2;
                    continue;
                }
            } else {
                result.push(lines[i].to_string());
            }
            i += 1;
        }

        let mut output = result.join("\n");

        // Preserve trailing newline if the original had one
        if text.ends_with('\n') && !output.ends_with('\n') {
            output.push('\n');
        }

        output
    }
}

/// Check if a word is a common English word (basic dictionary).
///
/// This is used to identify when a hyphenated word should be joined
/// vs when it's a compound word.
fn is_common_word(word: &str) -> bool {
    // Common words that are often split across lines
    let common_words = [
        "government",
        "department",
        "information",
        "administration",
        "documentation",
        "implementation",
        "communication",
        "organization",
        "representation",
        "transportation",
        "investigation",
        "determination",
        "consideration",
        "recommendation",
        "responsibility",
        "understanding",
        "international",
        "environmental",
        "constitutional",
        "congressional",
        "agricultural",
        "professional",
        "manufacturing",
        "requirements",
        "development",
        "management",
        "performance",
        "maintenance",
        "compliance",
        "procedures",
        "regulations",
        "activities",
        "operations",
        "provisions",
        "conditions",
        "limitations",
        "applications",
        "publications",
    ];

    let lower = word.to_lowercase();
    common_words.contains(&lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_continuation_hyphen_basic() {
        assert!(HyphenationHandler::is_continuation_hyphen("Govern-"));
        assert!(HyphenationHandler::is_continuation_hyphen("content-"));
        assert!(HyphenationHandler::is_continuation_hyphen("word-  ")); // with trailing space
    }

    #[test]
    fn test_is_continuation_hyphen_negative() {
        assert!(!HyphenationHandler::is_continuation_hyphen("- bullet"));
        assert!(!HyphenationHandler::is_continuation_hyphen(""));
        assert!(!HyphenationHandler::is_continuation_hyphen("no hyphen"));
        assert!(!HyphenationHandler::is_continuation_hyphen("123-")); // number before hyphen
    }

    #[test]
    fn test_is_continuation_hyphen_soft_hyphen() {
        // Soft hyphen (U+00AD) should be treated as continuation
        assert!(HyphenationHandler::is_continuation_hyphen("Govern\u{00AD}"));
        assert!(HyphenationHandler::is_continuation_hyphen("busi\u{00AD}"));
        assert!(HyphenationHandler::is_continuation_hyphen("word\u{00AD}  ")); // with trailing space
    }

    #[test]
    fn test_is_continuation_hyphen_soft_hyphen_negative() {
        // Soft hyphen without alphabetic char before should not be continuation
        assert!(!HyphenationHandler::is_continuation_hyphen("123\u{00AD}"));
        assert!(!HyphenationHandler::is_continuation_hyphen("\u{00AD}")); // just soft hyphen
    }

    #[test]
    fn test_is_compound_word() {
        assert!(HyphenationHandler::is_compound_word("self", "regulation"));
        assert!(HyphenationHandler::is_compound_word("non", "linear"));
        assert!(HyphenationHandler::is_compound_word("content", "type"));
    }

    #[test]
    fn test_process_line_pair_join() {
        let handler = HyphenationHandler::new();
        let (result, consumed) = handler.process_line_pair("Govern-", "ment of the");
        assert!(consumed);
        assert_eq!(result, "Government of the");
    }

    #[test]
    fn test_process_line_pair_preserve_compound() {
        let handler = HyphenationHandler::new();
        let (result, consumed) = handler.process_line_pair("self-", "regulation");
        assert!(!consumed); // Should NOT join compound words
        assert_eq!(result, "self-");
    }

    #[test]
    fn test_process_line_pair_soft_hyphen() {
        let handler = HyphenationHandler::new();
        // Soft hyphen should ALWAYS join (never a compound word marker)
        let (result, consumed) = handler.process_line_pair("busi\u{00AD}", "ness today");
        assert!(consumed);
        assert_eq!(result, "business today");
    }

    #[test]
    fn test_process_line_pair_soft_hyphen_always_joins() {
        let handler = HyphenationHandler::new();
        // Even compound-looking words with soft hyphen should join
        // because soft hyphen is ONLY used for optional line breaks
        let (result, consumed) = handler.process_line_pair("self\u{00AD}", "regulation");
        assert!(consumed);
        assert_eq!(result, "selfregulation");
    }

    #[test]
    fn test_process_text_multiple_lines() {
        let handler = HyphenationHandler::new();
        let text = "The Govern-\nment issued a\nstate-\nment today.";
        let result = handler.process_text(text);
        // Should join "Govern-ment" but preserve "state-ment" as separate lines
        // (state-ment is a common word, should be joined)
        assert!(result.contains("Government"));
    }

    #[test]
    fn test_process_text_no_hyphen() {
        let handler = HyphenationHandler::new();
        let text = "Normal text\nwith no hyphens\nat line ends.";
        let result = handler.process_text(text);
        assert_eq!(result, text);
    }
}
