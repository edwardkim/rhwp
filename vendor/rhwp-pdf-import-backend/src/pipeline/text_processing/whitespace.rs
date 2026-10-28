//! Whitespace normalization for improved text extraction quality.
//!
//! This module provides utilities for normalizing excessive whitespace in extracted text
//! while preserving intentional formatting like code blocks, tables, and paragraph breaks.

/// Normalizes whitespace in extracted text.
///
/// Handles:
/// - Collapse multiple consecutive spaces to single space
/// - Collapse multiple newlines to double newline (paragraph break)
/// - Trim leading/trailing whitespace from lines
/// - Optionally preserve layout mode (no normalization)
#[derive(Debug, Clone)]
pub struct WhitespaceNormalizer {
    /// When true, preserves layout and doesn't normalize whitespace
    preserve_layout_mode: bool,
}

impl WhitespaceNormalizer {
    /// Create a new whitespace normalizer.
    ///
    /// # Arguments
    ///
    /// * `preserve_layout_mode` - If true, preserves whitespace for layout preservation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::pipeline::text_processing::WhitespaceNormalizer;
    ///
    /// let normalizer = WhitespaceNormalizer::new(false);
    /// assert_eq!(normalizer.normalize("hello   world"), "hello world");
    /// ```
    pub fn new(preserve_layout_mode: bool) -> Self {
        Self {
            preserve_layout_mode,
        }
    }

    /// Normalize whitespace in text.
    ///
    /// In normal mode:
    /// - Collapses multiple spaces to single space
    /// - Collapses multiple newlines to double newline (paragraph break)
    /// - Trims leading/trailing whitespace from each line
    /// - Preserves single newlines (line breaks)
    ///
    /// In layout mode:
    /// - Returns text unchanged (preserves all whitespace)
    ///
    /// # Arguments
    ///
    /// * `text` - The text to normalize
    ///
    /// # Returns
    ///
    /// Normalized text string
    pub fn normalize(&self, text: &str) -> String {
        // If in layout mode, preserve all whitespace
        if self.preserve_layout_mode {
            return text.to_string();
        }

        // Split by paragraphs (double newline) first
        let paragraphs: Vec<&str> = text.split("\n\n").collect();

        let normalized_paragraphs: Vec<String> = paragraphs
            .iter()
            .map(|para| self.normalize_paragraph(para))
            .collect();

        normalized_paragraphs.join("\n\n")
    }

    /// Normalize a single paragraph (text without paragraph breaks).
    fn normalize_paragraph(&self, text: &str) -> String {
        // Split into lines
        let lines: Vec<&str> = text.lines().collect();

        let normalized_lines: Vec<String> = lines
            .iter()
            .map(|line| self.normalize_line(line))
            .filter(|line| !line.is_empty()) // Remove empty lines
            .collect();

        normalized_lines.join("\n")
    }

    /// Normalize a single line (text without newlines).
    fn normalize_line(&self, line: &str) -> String {
        // Trim leading and trailing whitespace
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return String::new();
        }

        // Replace all whitespace sequences (spaces, tabs, etc.) with single space
        let mut result = String::new();
        let mut prev_was_space = false;

        for ch in trimmed.chars() {
            if ch.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_single_space() {
        let normalizer = WhitespaceNormalizer::new(false);
        assert_eq!(normalizer.normalize("hello world"), "hello world");
    }

    #[test]
    fn test_normalize_multiple_spaces() {
        let normalizer = WhitespaceNormalizer::new(false);
        assert_eq!(normalizer.normalize("hello   world"), "hello world");
    }

    #[test]
    fn test_normalize_tabs() {
        let normalizer = WhitespaceNormalizer::new(false);
        assert_eq!(normalizer.normalize("hello\t\tworld"), "hello world");
    }

    #[test]
    fn test_preserve_layout_mode() {
        let normalizer = WhitespaceNormalizer::new(true);
        let text = "hello   world";
        assert_eq!(normalizer.normalize(text), text);
    }
}
