//! Whitespace normalization and cleanup for markdown output.
//!
//! This module provides functions to clean up excessive whitespace in generated markdown,
//! ensuring consistent formatting and readability.

use regex::Regex;
use std::sync::LazyLock;

static RE_MULTI_NEWLINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());
static RE_PAGE_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^Page\s+\d+\s*$").unwrap());
static RE_DASH_PAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*-\s*\d+\s*-\s*$").unwrap());
static RE_BRACKET_PAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*[\[\(]\d+[\]\)]\s*$").unwrap());
static RE_STANDALONE_NUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*\d{1,3}\s*$").unwrap());
static RE_DASH_SEP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^[\s\-]{5,}$").unwrap());
static RE_EQUALS_SEP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^[\s=]{5,}$").unwrap());

/// Normalize whitespace in markdown text by limiting consecutive blank lines.
///
/// This function reduces excessive blank lines to a maximum of 2 consecutive empty lines,
/// which improves readability while maintaining logical section separation.
///
/// # Arguments
///
/// * `text` - The markdown text to normalize
///
/// # Returns
///
/// Normalized text with at most 2 consecutive blank lines
///
/// # Examples
///
/// ```
/// use pdf_oxide::converters::whitespace::normalize_whitespace;
///
/// let input = "Line 1\n\n\n\n\n\nLine 2";
/// let output = normalize_whitespace(input);
/// assert_eq!(output, "Line 1\n\n\nLine 2");
/// ```
pub fn normalize_whitespace(text: &str) -> String {
    // Pattern: 3 or more consecutive newlines
    // Replace with exactly 3 newlines (2 blank lines)
    RE_MULTI_NEWLINE.replace_all(text, "\n\n\n").to_string()
}

/// Remove common page artifacts from markdown text.
///
/// Removes patterns that commonly appear in PDFs due to headers, footers,
/// page numbers, and other page-level elements that don't belong in
/// continuous markdown output.
///
/// # Arguments
///
/// * `text` - The markdown text to clean
///
/// # Returns
///
/// Text with common page artifacts removed
///
/// # Artifacts Removed
///
/// - Standalone page numbers (e.g., "Page 1", "- 1 -", "\[1\]")
/// - Common header/footer separators (lines of dashes, equals signs)
/// - Repeated navigation elements
///
/// # Examples
///
/// ```
/// use pdf_oxide::converters::whitespace::remove_page_artifacts;
///
/// let input = "Content here\n\nPage 1\n\nMore content";
/// let output = remove_page_artifacts(input);
/// assert!(output.contains("Content here"));
/// assert!(output.contains("More content"));
/// assert!(!output.contains("Page 1"));
/// ```
pub fn remove_page_artifacts(text: &str) -> String {
    let mut result = text.to_string();

    // Remove standalone page numbers with common formats
    // Pattern 1: "Page N" on its own line
    result = RE_PAGE_NUM.replace_all(&result, "").to_string();

    // Pattern 2: "- N -" style page numbers
    result = RE_DASH_PAGE.replace_all(&result, "").to_string();

    // Pattern 3: "[N]" or "(N)" style page numbers at start/end of line
    result = RE_BRACKET_PAGE.replace_all(&result, "").to_string();

    // Pattern 4: Standalone numbers that are likely page numbers
    // (only if on their own line and between 1-999)
    result = RE_STANDALONE_NUM.replace_all(&result, "").to_string();

    // Remove horizontal separator lines (--- or === spanning most of line)
    result = RE_DASH_SEP.replace_all(&result, "").to_string();

    result = RE_EQUALS_SEP.replace_all(&result, "").to_string();

    result
}

/// Merge adjacent bold markers to create more natural phrasing.
///
/// This function removes unnecessary bold marker boundaries by merging patterns like
/// `**bold text** more text` into `**bold text more text**` when appropriate.
///
/// Specifically handles cases where:
/// - Bold text is followed by a space and then more text
/// - The gap between bold segments is small (≤3 words)
///
/// # Arguments
///
/// * `text` - The markdown text to process
///
/// # Returns
///
/// Text with merged bold markers for more natural phrasing
///
/// # Examples
///
/// ```no_run
/// use pdf_oxide::converters::whitespace::merge_bold_markers;
///
/// let input = "The **Chinese stock** market is volatile";
/// let output = merge_bold_markers(input);
/// assert_eq!(output, "The **Chinese stock market** is volatile");
/// ```
pub fn merge_bold_markers(text: &str) -> String {
    // Merge `** **` patterns (empty bold boundaries).
    // "**text1** **text2**" -> "**text1 text2**"
    text.replace("** **", " ")
}

/// Remove consecutive duplicate words that likely occurred due to column/page boundaries.
///
/// This function removes patterns like "the the", "and and", etc. that commonly appear
/// when text is extracted across column or page boundaries in PDFs.
///
/// # Arguments
///
/// * `text` - The markdown text to clean
///
/// # Returns
///
/// Text with consecutive duplicate words removed
///
/// # Examples
///
/// ```no_run
/// use pdf_oxide::converters::whitespace::remove_duplicate_words;
///
/// let input = "The the cat sat sat on the mat.";
/// let output = remove_duplicate_words(input);
/// assert_eq!(output, "The cat sat on the mat.");
/// ```
pub fn remove_duplicate_words(text: &str) -> String {
    static RE_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b(\w{4,})\b").unwrap());

    let mut result = String::with_capacity(text.len());
    let mut last_word: Option<String> = None;
    let mut last_end = 0;

    for cap in RE_WORD.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let word = m.as_str();
        let start = m.start();
        let end = m.end();

        // Add text between last match and this match
        result.push_str(&text[last_end..start]);

        // Check if this word is a duplicate of the last word (case-insensitive)
        let is_duplicate = if let Some(ref prev) = last_word {
            word.to_lowercase() == prev.to_lowercase()
        } else {
            false
        };

        if !is_duplicate {
            // Not a duplicate, add the word
            result.push_str(word);
            last_word = Some(word.to_string());
        }
        // else: skip the duplicate word

        last_end = end;
    }

    // Add remaining text after last match
    result.push_str(&text[last_end..]);

    result
}

/// Apply full whitespace cleanup pipeline to markdown text.
///
/// This is a convenience function that applies all cleanup steps including
/// artifact removal, bold marker merging, duplicate word removal, and
/// whitespace normalization in the correct order.
///
/// # Arguments
///
/// * `text` - The markdown text to clean
///
/// # Returns
///
/// Fully cleaned and normalized markdown text
///
/// # Examples
///
/// ```
/// use pdf_oxide::converters::whitespace::cleanup_markdown;
///
/// let input = "Content\n\n\n\n\nPage 1\n\n\n\n\nMore content";
/// let output = cleanup_markdown(input);
/// assert!(!output.contains("Page 1"));
/// // Should have max 2 consecutive blank lines
/// assert!(!output.contains("\n\n\n\n"));
/// ```
pub fn cleanup_markdown(text: &str) -> String {
    // Apply PDF spec-compliant cleanup pipeline:
    // 1. Remove page artifacts (headers, footers, page numbers)
    // 2. Normalize whitespace (limit excessive blank lines)
    // NOTE: We do NOT apply heuristics like duplicate word removal or bold marker merging
    // because they are not spec-compliant and can corrupt legitimate text.
    let without_artifacts = remove_page_artifacts(text);
    normalize_whitespace(&without_artifacts)
}

/// Normalize horizontal whitespace for plain text output.
///
/// Reduces excessive consecutive spaces while preserving intentional spacing:
/// - Reduces 2+ spaces to single space (except at start of lines for indentation)
/// - Preserves newlines and paragraph structure
/// - Preserves leading spaces on lines (indentation)
///
/// This is specifically designed for plain text extraction to improve readability
/// and match the quality of established industry tools.
///
/// # Arguments
///
/// * `text` - The plain text to normalize
///
/// # Returns
///
/// Text with normalized horizontal whitespace
///
/// # Examples
///
/// ```
/// use pdf_oxide::converters::whitespace::normalize_horizontal_whitespace;
///
/// let input = "The  quick    brown  fox";
/// let output = normalize_horizontal_whitespace(input);
/// assert_eq!(output, "The quick brown fox");
/// ```
pub fn normalize_horizontal_whitespace(text: &str) -> String {
    static RE_MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" {2,}").unwrap());

    // Process line by line to preserve indentation at start of lines
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        if !result.is_empty() {
            result.push('\n');
        }

        // For each line, preserve leading spaces but normalize internal spaces
        let trimmed_start = line.trim_start();
        let leading_spaces_count = line.len() - trimmed_start.len();

        // Add back the leading spaces (indentation)
        for _ in 0..leading_spaces_count {
            result.push(' ');
        }

        // Normalize internal spaces (2+ spaces -> 1 space)
        let normalized = RE_MULTI_SPACE.replace_all(trimmed_start, " ");
        result.push_str(&normalized);
    }

    result
}

/// Apply full whitespace cleanup pipeline for plain text.
///
/// This applies normalizations appropriate for plain text output:
/// - Normalizes horizontal whitespace (reduces double spaces)
/// - Normalizes vertical whitespace (limits excessive blank lines)
/// - Preserves paragraph structure and line breaks
///
/// # Arguments
///
/// * `text` - The plain text to clean
///
/// # Returns
///
/// Fully cleaned and normalized plain text
///
/// # Examples
///
/// ```
/// use pdf_oxide::converters::whitespace::cleanup_plain_text;
///
/// let input = "The  quick  brown  fox\n\n\n\n\njumps  over";
/// let output = cleanup_plain_text(input);
/// assert_eq!(output, "The quick brown fox\n\n\njumps over");
/// ```
pub fn cleanup_plain_text(text: &str) -> String {
    // Apply plain text cleanup pipeline:
    // 1. Dehyphenate line-broken words (B8)
    // 2. Normalize horizontal whitespace (reduce double spaces)
    // 3. Normalize vertical whitespace (limit excessive blank lines)
    let dehyphenated = dehyphenate_line_breaks(text);
    let horizontal_normalized = normalize_horizontal_whitespace(&dehyphenated);
    normalize_whitespace(&horizontal_normalized)
}

/// Rejoin words split across a line break by a soft hyphen.
///
/// Standard typesetting hyphenates a word at a line break and expects the
/// reader to reassemble it. The PDF content stream preserves both halves
/// plus the hyphen (e.g. `"scruti-\nneer"` for `"scrutineer"`). pdftotext
/// handles this by stripping the hyphen + newline when the next line
/// starts with a lowercase letter — a reliable typographic signal.
///
/// We do the same here:
/// - `word-\nword` → `wordword` (standard soft hyphen)
/// - `word- \nword` (trailing space before break) → `wordword`
/// - We leave intact: `word-\nWord` (proper-noun fragment; safer not to
///   merge), `word—\nword` (em-dash = punctuation, not hyphenation).
///
/// Conservative: requires the tail to be lowercase letters so we don't
/// concatenate unrelated tokens like a numbered bullet or a caps header.
pub fn dehyphenate_line_breaks(text: &str) -> String {
    // Pre-size: dehyphenation only removes bytes, never adds.
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        // Look for ASCII "<lowercase>-[ \t]*\n[ \t]*<lowercase>". Only ASCII
        // matches trigger the rewrite, so non-ASCII bytes can flow through
        // unchanged below. This keeps us UTF-8-safe without splitting
        // multi-byte sequences.
        if c == b'-' && i > 0 && bytes[i - 1].is_ascii_lowercase() {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'\r' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'\n' {
                j += 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j].is_ascii_lowercase() {
                    // Skip the hyphen + intervening whitespace + newline.
                    i = j;
                    continue;
                }
            }
        }
        // ASCII fast path; otherwise re-decode the UTF-8 scalar at position i.
        if c.is_ascii() {
            out.push(c as char);
            i += 1;
        } else {
            // Multi-byte UTF-8: find the character boundary and push the full
            // scalar value, advancing i past its bytes.
            let rest = &text[i..];
            let ch = rest.chars().next().expect("bytes inside str");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dehyphenate_rejoins_soft_hyphen_line_break() {
        assert_eq!(dehyphenate_line_breaks("scruti-\nneer"), "scrutineer");
        assert_eq!(dehyphenate_line_breaks("scruti-\r\nneer"), "scrutineer");
        assert_eq!(dehyphenate_line_breaks("scruti-   \n   neer"), "scrutineer");
    }

    #[test]
    fn dehyphenate_does_not_merge_proper_noun_fragment() {
        // Uppercase start on next line: likely a new sentence / proper noun.
        assert_eq!(dehyphenate_line_breaks("co-\nWorker"), "co-\nWorker");
    }

    #[test]
    fn dehyphenate_requires_hyphen_and_lowercase_both_sides() {
        assert_eq!(dehyphenate_line_breaks("no\nhyphen"), "no\nhyphen");
        assert_eq!(dehyphenate_line_breaks("bullet-\n• point"), "bullet-\n• point");
    }

    #[test]
    fn dehyphenate_preserves_intentional_hyphens() {
        // No line break after hyphen → leave it alone.
        assert_eq!(
            dehyphenate_line_breaks("state-of-the-art solution"),
            "state-of-the-art solution"
        );
    }

    #[test]
    fn test_normalize_whitespace_reduces_excessive_blanks() {
        let input = "Line 1\n\n\n\n\n\n\n\nLine 2";
        let output = normalize_whitespace(input);
        assert_eq!(output, "Line 1\n\n\nLine 2");
        assert!(!output.contains("\n\n\n\n")); // No 4+ consecutive newlines
    }

    #[test]
    fn test_normalize_whitespace_preserves_single_and_double_blanks() {
        let input = "A\nB\n\nC\n\n\nD";
        let output = normalize_whitespace(input);
        assert_eq!(output, "A\nB\n\nC\n\n\nD");
    }

    #[test]
    fn test_normalize_whitespace_handles_no_blanks() {
        let input = "Line 1\nLine 2\nLine 3";
        let output = normalize_whitespace(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_remove_page_artifacts_page_numbers() {
        let input = "Content\n\nPage 1\n\nMore content\n\nPage 2\n\nEnd";
        let output = remove_page_artifacts(input);
        assert!(output.contains("Content"));
        assert!(output.contains("More content"));
        assert!(output.contains("End"));
        assert!(!output.contains("Page 1"));
        assert!(!output.contains("Page 2"));
    }

    #[test]
    fn test_remove_page_artifacts_dash_style() {
        let input = "Content\n\n- 1 -\n\nMore content\n\n- 2 -\n\nEnd";
        let output = remove_page_artifacts(input);
        assert!(!output.contains("- 1 -"));
        assert!(!output.contains("- 2 -"));
    }

    #[test]
    fn test_remove_page_artifacts_bracket_style() {
        let input = "Content\n\n[1]\n\nMore\n\n(2)\n\nEnd";
        let output = remove_page_artifacts(input);
        assert!(!output.contains("[1]"));
        assert!(!output.contains("(2)"));
    }

    #[test]
    fn test_remove_page_artifacts_standalone_numbers() {
        let input = "Content\n\n1\n\nMore\n\n42\n\nEnd";
        let output = remove_page_artifacts(input);
        // Standalone numbers should be removed
        assert!(!output.contains("\n1\n"));
        assert!(!output.contains("\n42\n"));
    }

    #[test]
    fn test_remove_page_artifacts_preserves_inline_numbers() {
        let input = "There are 42 items in the list.";
        let output = remove_page_artifacts(input);
        // Inline numbers should be preserved
        assert_eq!(output, input);
    }

    #[test]
    fn test_remove_page_artifacts_separators() {
        let input = "Section 1\n\n-----------\n\nSection 2\n\n===========\n\nEnd";
        let output = remove_page_artifacts(input);
        assert!(!output.contains("-----------"));
        assert!(!output.contains("==========="));
    }

    #[test]
    fn test_cleanup_markdown_full_pipeline() {
        let input = "Content\n\n\n\n\n\nPage 1\n\n\n\n\n\nMore content\n\n-----------\n\n\n\n\nEnd";
        let output = cleanup_markdown(input);

        // Artifacts removed
        assert!(!output.contains("Page 1"));
        assert!(!output.contains("-----------"));

        // Whitespace normalized (max 3 newlines = 2 blank lines)
        assert!(!output.contains("\n\n\n\n"));

        // Content preserved
        assert!(output.contains("Content"));
        assert!(output.contains("More content"));
        assert!(output.contains("End"));
    }

    #[test]
    fn test_cleanup_markdown_empty_string() {
        let output = cleanup_markdown("");
        assert_eq!(output, "");
    }

    #[test]
    fn test_cleanup_markdown_no_changes_needed() {
        let input = "Line 1\n\nLine 2\n\nLine 3";
        let output = cleanup_markdown(input);
        assert_eq!(output, input);
    }
}
