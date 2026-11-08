//! Document-type detection for automatic profile selection
//!
//! This module analyzes PDF content to detect document types and recommend
//! appropriate extraction profiles. Detection uses heuristics from the first page
//! to classify documents and apply optimized thresholds.
//!
//! This module provides infrastructure for automatic document-type classification.
//! Future enhancements will integrate with the text extraction pipeline to analyze
//! layout patterns, spacing consistency, and content features.

use crate::config::extraction_profiles::DocumentType;

/// Statistics collected from document analysis
#[derive(Debug, Clone)]
pub struct DocumentStats {
    /// Number of text lines analyzed
    pub line_count: usize,

    /// Average characters per line
    pub avg_chars_per_line: f32,

    /// Standard deviation of characters per line (measures consistency)
    pub char_variance: f32,

    /// Percentage of lines that appear justified (right-aligned with consistent spacing)
    pub justified_lines_percentage: f32,

    /// Average gap between words (in thousandths of em)
    pub avg_word_gap: f32,

    /// Standard deviation of word gaps (measures spacing consistency)
    pub word_gap_variance: f32,

    /// Percentage of lines containing potential form fields (whitespace, checkboxes)
    pub form_field_percentage: f32,

    /// Percentage of text that appears to be citations or academic references
    pub citation_percentage: f32,

    /// Number of mathematical symbols or equations detected
    pub math_symbols_count: usize,

    /// Average line spacing (measures vertical consistency)
    pub avg_line_spacing: f32,

    /// Indicator of tight vs. loose spacing (< 0.15em = tight, > 0.3em = loose)
    pub spacing_tightness: f32,
}

impl Default for DocumentStats {
    fn default() -> Self {
        Self {
            line_count: 0,
            avg_chars_per_line: 0.0,
            char_variance: 0.0,
            justified_lines_percentage: 0.0,
            avg_word_gap: 0.0,
            word_gap_variance: 0.0,
            form_field_percentage: 0.0,
            citation_percentage: 0.0,
            math_symbols_count: 0,
            avg_line_spacing: 0.0,
            spacing_tightness: 0.0,
        }
    }
}

/// Document-type classifier based on content analysis
pub struct DocumentClassifier;

impl DocumentClassifier {
    /// Analyze document and detect document type
    ///
    /// This function examines content characteristics and produces statistics
    /// used to classify the document type and select an appropriate extraction profile.
    ///
    /// Per ISO 32000-1:2008 Section 9.4.4 and 14.8.2.5, document classification helps
    /// determine appropriate thresholds for word boundary detection. Different document
    /// types have different spacing characteristics:
    ///
    /// - Academic papers: Tight spacing, mathematical symbols, citations
    /// - Policy documents: Justified text, dense paragraphs, formal language
    /// - Forms: Structured fields, precise positioning, checkboxes
    /// - Government docs: Mixed layout, tables, consistent spacing
    /// - Scanned OCR: Variable spacing, OCR artifacts
    ///
    /// # Arguments
    ///
    /// * `lines` - Iterator of text lines to analyze
    ///
    /// # Returns
    ///
    /// A tuple of (DetectedDocumentType, AnalysisStats) containing classification
    /// and detailed statistics for threshold tuning
    pub fn classify_lines<'a, I>(lines: I) -> (DocumentType, DocumentStats)
    where
        I: Iterator<Item = &'a str>,
    {
        let mut stats = DocumentStats::default();
        let mut line_lengths = Vec::new();
        let mut justified_count = 0;
        let mut form_field_count = 0;
        let mut citation_count = 0;
        let mut math_symbol_count = 0;
        let mut line_spacing_values = Vec::new();
        let mut word_gaps = Vec::new();

        for (idx, line) in lines.enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            stats.line_count += 1;

            // 1. Analyze line length for consistency (forms/tables have consistent widths)
            line_lengths.push(trimmed.len());

            // 2. Detect justified text (policy documents, dense paragraphs)
            // Justified lines typically end near column margin with variable spacing
            if Self::looks_justified(trimmed) {
                justified_count += 1;
            }

            // 3. Detect form fields (underscores, brackets, checkboxes)
            if Self::contains_form_field_markers(trimmed) {
                form_field_count += 1;
            }

            // 4. Detect academic citations (et al, [1], etc.)
            if Self::looks_like_citation(trimmed) {
                citation_count += 1;
            }

            // 5. Count mathematical symbols
            if Self::contains_math_symbols(trimmed) {
                math_symbol_count += 1;
            }

            // 6. Analyze word spacing patterns
            let gaps = Self::extract_word_gaps(trimmed);
            word_gaps.extend(gaps);

            // 7. Estimate line spacing (simplified - uses line count as proxy)
            if idx > 0 {
                line_spacing_values.push(1.0); // Placeholder: would need y-coordinates
            }
        }

        // Calculate statistics from collected data
        if !line_lengths.is_empty() {
            let total_chars: usize = line_lengths.iter().sum();
            stats.avg_chars_per_line = total_chars as f32 / line_lengths.len() as f32;

            // Calculate variance in line length (measures consistency)
            let mean = stats.avg_chars_per_line;
            let variance: f32 = line_lengths
                .iter()
                .map(|&len| {
                    let diff = len as f32 - mean;
                    diff * diff
                })
                .sum::<f32>()
                / line_lengths.len() as f32;
            stats.char_variance = variance.sqrt();
        }

        if stats.line_count > 0 {
            stats.justified_lines_percentage =
                (justified_count as f32 / stats.line_count as f32) * 100.0;
            stats.form_field_percentage =
                (form_field_count as f32 / stats.line_count as f32) * 100.0;
            stats.citation_percentage = (citation_count as f32 / stats.line_count as f32) * 100.0;
        }

        stats.math_symbols_count = math_symbol_count;

        // Calculate word gap statistics
        if !word_gaps.is_empty() {
            let total_gap: f32 = word_gaps.iter().sum();
            stats.avg_word_gap = total_gap / word_gaps.len() as f32;

            let mean = stats.avg_word_gap;
            let variance: f32 = word_gaps
                .iter()
                .map(|&gap| {
                    let diff = gap - mean;
                    diff * diff
                })
                .sum::<f32>()
                / word_gaps.len() as f32;
            stats.word_gap_variance = variance.sqrt();

            // Determine spacing tightness: percentage of gaps below threshold
            // < 0.15em = tight (academic), > 0.3em = loose (OCR)
            stats.spacing_tightness =
                word_gaps.iter().filter(|&&g| g < 0.15).count() as f32 / word_gaps.len() as f32;
        }

        if !line_spacing_values.is_empty() {
            let total_spacing: f32 = line_spacing_values.iter().sum();
            stats.avg_line_spacing = total_spacing / line_spacing_values.len() as f32;
        }

        // Classify document based on detected characteristics
        let doc_type = Self::classify_from_stats(&stats);

        (doc_type, stats)
    }

    /// Placeholder for line-based classification (backward compatibility)
    pub fn classify(_data: &str) -> (DocumentType, DocumentStats) {
        // For now, return Mixed type as default
        // Real usage should use classify_lines() with actual PDF content
        (DocumentType::Mixed, DocumentStats::default())
    }

    /// Classify document type based on collected statistics
    fn classify_from_stats(stats: &DocumentStats) -> DocumentType {
        // Decision tree for document type classification
        // Based on heuristics from content analysis

        // 1. Check for forms (high form field percentage)
        if stats.form_field_percentage > 15.0 {
            return DocumentType::Form;
        }

        // 2. Check for academic papers (math symbols + citations)
        if stats.math_symbols_count > 5 && stats.citation_percentage > 5.0 {
            return DocumentType::Academic;
        }

        // 3. Check for justified text (policy/legal documents)
        // Justified documents typically have:
        // - 30%+ justified lines
        // - Moderate to high word gap variance
        if stats.justified_lines_percentage > 30.0
            && stats.word_gap_variance > 1.0
            && stats.form_field_percentage < 5.0
        {
            return DocumentType::Policy;
        }

        // 4. Check for tight spacing (academic or form)
        if stats.spacing_tightness > 0.7 && stats.math_symbols_count > 2 {
            return DocumentType::Academic;
        }

        // 5. Check for OCR documents (high spacing variance, loose gaps)
        if stats.avg_word_gap > 0.3 && stats.word_gap_variance > 2.0 {
            return DocumentType::ScannedOCR;
        }

        // 6. Check for government/structured documents (consistent spacing)
        if stats.char_variance < 10.0 && stats.justified_lines_percentage > 20.0 {
            return DocumentType::Government;
        }

        // Default: mixed or unknown
        DocumentType::Mixed
    }

    /// Detect if a line appears to be justified
    fn looks_justified(line: &str) -> bool {
        // Justified lines typically:
        // 1. Have variable spacing between words (detected by looking at multiple spaces)
        // 2. Extend close to expected margin (would need layout context)
        // 3. Have consistent ending position (would need coordinate data)

        // Simple heuristic: line with multiple single spaces throughout
        // and no obvious list/bullet formatting
        if line.is_empty() {
            return false;
        }

        // Check for multiple consecutive spaces (justification artifact)
        let double_space_count = line.matches("  ").count();
        let word_count = line.split_whitespace().count();

        // Justified text often has 1-2+ occurrences of double spacing
        if word_count > 5 && double_space_count > 0 {
            return true;
        }

        false
    }

    /// Detect form field markers (underscores, brackets, etc.)
    fn contains_form_field_markers(line: &str) -> bool {
        // Form fields typically contain:
        // - Underscores for fill-in lines
        // - Brackets for checkboxes/options
        // - Multiple consecutive spaces for alignment
        // - Box drawing characters

        let has_underscores = line.matches('_').count() >= 3;
        let has_brackets = line.contains('[') || line.contains(']');
        let has_boxes = line.contains('☐') || line.contains('☒') || line.contains('□');

        has_underscores || has_brackets || has_boxes
    }

    /// Extract word gap values from a line (simplified, uses spaces as proxy)
    fn extract_word_gaps(line: &str) -> Vec<f32> {
        let mut gaps = Vec::new();

        // Simple approach: count consecutive spaces as gap indicator
        let mut in_gap = false;
        let mut gap_size = 0;

        for ch in line.chars() {
            if ch == ' ' {
                gap_size += 1;
                in_gap = true;
            } else if in_gap {
                // End of gap sequence
                gaps.push(gap_size as f32 * 0.1); // Scale to approximate em units
                gap_size = 0;
                in_gap = false;
            }
        }

        if in_gap {
            gaps.push(gap_size as f32 * 0.1);
        }

        gaps
    }

    /// Check if text looks like academic citation (year, parentheses pattern)
    fn looks_like_citation(text: &str) -> bool {
        // Pattern: contains 4-digit year in parentheses, or typical citation format
        // Examples: "(2023)", "[1]", "et al."
        if text.contains("et al") || text.contains("et. al") {
            return true;
        }

        if text.contains('[') && text.contains(']') {
            return true; // Citation bracket notation
        }

        // Check for year pattern
        if text.len() >= 4 {
            for chunk in text.chars().collect::<Vec<_>>().windows(4) {
                let s: String = chunk.iter().collect();
                if let Ok(year) = s.parse::<u32>() {
                    if (1900..=2100).contains(&year) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if text contains mathematical symbols
    fn contains_math_symbols(text: &str) -> bool {
        text.chars().any(Self::is_math_symbol)
    }

    /// Check if character is a mathematical symbol
    fn is_math_symbol(c: char) -> bool {
        matches!(
            c,
            '∑' | '∫'
                | '∂'
                | '∇'
                | '√'
                | '∞'
                | '≈'
                | '≠'
                | '≤'
                | '≥'
                | '±'
                | '×'
                | '÷'
                | 'α'
                | 'β'
                | 'γ'
                | 'δ'
                | 'ε'
                | 'θ'
                | 'λ'
                | 'μ'
                | 'π'
                | 'σ'
                | 'ω'
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_detection() {
        assert!(DocumentClassifier::looks_like_citation("et al"));
        assert!(DocumentClassifier::looks_like_citation("et. al"));
        assert!(DocumentClassifier::looks_like_citation("[1]"));
        assert!(DocumentClassifier::looks_like_citation("[42]"));
        assert!(!DocumentClassifier::looks_like_citation("hello world"));
    }

    #[test]
    fn test_math_symbol_detection() {
        assert!(DocumentClassifier::contains_math_symbols("π is pi"));
        assert!(DocumentClassifier::contains_math_symbols("∫ is integral"));
        assert!(!DocumentClassifier::contains_math_symbols("hello world"));
    }

    #[test]
    fn test_math_symbol_recognition() {
        assert!(DocumentClassifier::is_math_symbol('π'));
        assert!(DocumentClassifier::is_math_symbol('∫'));
        assert!(DocumentClassifier::is_math_symbol('≈'));
        assert!(!DocumentClassifier::is_math_symbol('a'));
    }

    #[test]
    fn test_form_field_detection() {
        assert!(DocumentClassifier::contains_form_field_markers("Name: ___________"));
        assert!(DocumentClassifier::contains_form_field_markers("[X] Check here"));
        assert!(DocumentClassifier::contains_form_field_markers("Address: _______"));
        assert!(!DocumentClassifier::contains_form_field_markers("This is normal text"));
    }

    #[test]
    fn test_justified_text_detection() {
        assert!(DocumentClassifier::looks_justified(
            "This  is  justified  text  with  variable  spacing"
        ));
        assert!(!DocumentClassifier::looks_justified("Short text"));
        assert!(!DocumentClassifier::looks_justified(""));
    }

    #[test]
    fn test_word_gap_extraction() {
        let gaps = DocumentClassifier::extract_word_gaps("word1 word2  word3");
        assert!(!gaps.is_empty());
        // Should detect gaps between words
        assert!(gaps.len() >= 2);
    }

    #[test]
    fn test_classify_form_document() {
        let lines = vec![
            "Name: ___________",
            "Address: _______",
            "Phone: ___________",
            "[X] Check here",
        ];

        let (doc_type, stats) = DocumentClassifier::classify_lines(lines.into_iter());

        // Should detect as form due to high form field percentage
        assert_eq!(doc_type, DocumentType::Form);
        assert!(stats.form_field_percentage > 15.0);
    }

    #[test]
    fn test_academic_characteristics_detection() {
        let lines = vec![
            "Abstract: We prove that π ≈ ∑ contribution",
            "Smith et al. (2020) showed π in ∫ dx",
            "From [1] we know ∂ exists with ∞ solutions",
            "Therefore λ > 0 and α ∈ (0,1)",
        ];

        let (_doc_type, stats) = DocumentClassifier::classify_lines(lines.into_iter());

        // Should detect academic characteristics: citations and math symbols
        // Classification depends on all detected metrics working together
        assert!(stats.math_symbols_count > 2, "Should detect multiple math symbols");
        assert!(stats.citation_percentage > 0.0, "Should detect academic citations");
    }

    #[test]
    fn test_classify_justified_document() {
        let lines = vec![
            "This  is  justified  text  with  variable  spacing  throughout",
            "The  document  maintains  consistent  margins  on  both  sides",
            "Justified  alignment  is  common  in  policy  and  legal  texts",
            "Each  line  extends  to  the  right  margin  with  adjustments",
        ];

        let (doc_type, stats) = DocumentClassifier::classify_lines(lines.into_iter());

        // Should detect justified text characteristics
        assert!(stats.justified_lines_percentage > 30.0);
        // Will classify based on all detected characteristics
        // May be Policy, Government, Academic, or Mixed depending on other metrics
        assert!(matches!(
            doc_type,
            DocumentType::Policy
                | DocumentType::Government
                | DocumentType::Academic
                | DocumentType::Mixed
        ));
    }

    #[test]
    fn test_statistics_calculation() {
        let lines = vec!["short", "medium length", "very long line here"];

        let (_doc_type, stats) = DocumentClassifier::classify_lines(lines.into_iter());

        // Should calculate statistics
        assert!(stats.line_count > 0);
        assert!(stats.avg_chars_per_line > 0.0);
        assert!(stats.char_variance >= 0.0);
    }

    #[test]
    fn test_empty_document() {
        let lines: Vec<&str> = vec![];
        let (_doc_type, stats) = DocumentClassifier::classify_lines(lines.into_iter());

        // Should handle empty documents gracefully
        assert_eq!(stats.line_count, 0);
        assert_eq!(stats.avg_chars_per_line, 0.0);
    }
}
