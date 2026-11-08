//! PDF Text Extraction Profiles
//!
//! This module defines extraction profiles for different document types.
//! Each profile specifies thresholds and heuristics optimized for particular PDF types.
//!
//! Per ISO 32000-1:2008, word boundary detection involves heuristic thresholds
//! for converting geometric/positioning data to semantic word boundaries.
//! This module provides pre-tuned profiles for common document types.

/// Document type classification for extraction profile selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    /// Academic papers, research documents, technical reports
    /// Characteristics: Tight spacing, mathematical symbols, citations
    /// Typical fonts: Times New Roman, Computer Modern
    /// Strategy: Aggressive space insertion, preserve tight formatting
    Academic,

    /// Policy documents, legal text, regulations (e.g., GDPR)
    /// Characteristics: Justified text, dense paragraphs, formal language
    /// Typical fonts: Helvetica, Times
    /// Strategy: Conservative thresholds, handle justified text well
    Policy,

    /// Government documents, forms, official reports
    /// Characteristics: Consistent layout, fixed spacing, structured content
    /// Typical fonts: Arial, Courier
    /// Strategy: Maintain field alignment, handle form boundaries
    Government,

    /// Forms (tax forms, applications, questionnaires)
    /// Characteristics: Structured fields, precise positioning, checkboxes
    /// Typical fonts: Helvetica, Arial
    /// Strategy: Preserve field structure, handle form boundaries
    Form,

    /// Scanned documents with OCR text layer
    /// Characteristics: OCR artifacts, variable spacing, occasional errors
    /// Typical fonts: Custom OCR fonts
    /// Strategy: More lenient spacing, handle OCR noise
    ScannedOCR,

    /// Mixed or unknown document type
    /// Uses balanced default profile
    Mixed,
}

/// Extraction profile with document-type-specific thresholds
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractionProfile {
    /// Human-readable name for the profile
    pub name: &'static str,

    /// TJ offset threshold (thousandths of em)
    /// Negative values indicate word boundary signal
    /// Range: -80 to -150 (0.08em to 0.15em)
    /// Lower (more negative) = more conservative, fewer spaces
    /// Higher (less negative) = more aggressive, more spaces
    pub tj_offset_threshold: f32,

    /// Word margin ratio for geometric gap detection
    /// Ratio of space glyph width to use as threshold
    /// Range: 0.05 to 0.2 (5% to 20%)
    /// Lower = tighter spacing (academic), Higher = looser spacing
    pub word_margin_ratio: f32,

    /// Fallback space threshold as ratio of font size
    /// Used when font metrics unavailable
    /// Range: 0.1 to 0.4 (10% to 40% of font size)
    pub space_threshold_em_ratio: f32,

    /// Multiplier for space character width
    /// Affects how extra large spaces are handled
    /// Range: 0.4 to 0.8
    pub space_char_multiplier: f32,

    /// Enable adaptive threshold analysis
    /// Analyze first page to tune thresholds automatically
    pub use_adaptive_threshold: bool,

    /// Enable document-type detection
    /// Automatically classify document and apply appropriate profile
    pub enable_document_type_detection: bool,

    /// Enable email pattern detection for spacing decisions.
    ///
    /// When true, detects email-like patterns in extracted text
    /// (e.g., "user@domain" separated by spaces) and applies special spacing rules
    /// to preserve email addresses.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.10, only extracted text patterns
    /// are used - no domain-specific semantics.
    pub enable_email_detection: bool,

    /// Enable citation marker detection for spacing decisions.
    ///
    /// When true, detects superscript citation markers (typically smaller font size)
    /// and adjusts spacing rules to preserve citation formatting.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.10, font size ratios from extracted content
    /// are used for detection.
    pub enable_citation_detection: bool,
}

impl ExtractionProfile {
    /// Conservative profile - current default behavior
    /// Minimal space insertion, matches established behavior
    pub const CONSERVATIVE: Self = Self {
        name: "Conservative (Default)",
        tj_offset_threshold: -120.0,    // Most conservative TJ threshold
        word_margin_ratio: 0.1,         // Very tight geometric gap
        space_threshold_em_ratio: 0.25, // Large fallback threshold
        space_char_multiplier: 0.5,     // Treat spaces normally
        use_adaptive_threshold: false,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// TJ-heavy profile — calibrated for documents
    /// that emit entire paragraphs as one `TJ` array with kerning
    /// between every glyph (the kreuzberg `tiny.pdf` /
    /// `Loremipsumdolorsitamet` shape). The threshold of -100.0
    /// catches word-boundary kerning these PDFs use while staying
    /// safely above the in-word adjustment range. Callers opt in
    /// via `TextExtractionConfig::with_profile(ExtractionProfile::TJ_HEAVY)`.
    pub const TJ_HEAVY: Self = Self {
        name: "TJ-Heavy (Lorem-Ipsum-style PDFs)",
        tj_offset_threshold: -100.0,
        word_margin_ratio: 0.1,
        space_threshold_em_ratio: 0.25,
        space_char_multiplier: 0.5,
        use_adaptive_threshold: false,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Aggressive profile - liberal space insertion
    /// More spaces, helps with documents that suppress spacing
    pub const AGGRESSIVE: Self = Self {
        name: "Aggressive",
        tj_offset_threshold: -80.0,     // Less conservative
        word_margin_ratio: 0.2,         // More generous geometric gap
        space_threshold_em_ratio: 0.15, // Smaller fallback threshold
        space_char_multiplier: 0.8,     // Emphasize space characters
        use_adaptive_threshold: false,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Balanced profile - middle ground
    /// Reasonable for general documents
    pub const BALANCED: Self = Self {
        name: "Balanced",
        tj_offset_threshold: -100.0,
        word_margin_ratio: 0.15,
        space_threshold_em_ratio: 0.2,
        space_char_multiplier: 0.65,
        use_adaptive_threshold: false,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Academic profile - optimized for research papers
    /// Tight spacing, preserve mathematical content
    /// Per analysis: arxiv papers, conference proceedings, technical reports
    pub const ACADEMIC: Self = Self {
        name: "Academic",
        tj_offset_threshold: -105.0, // More aggressive for tight spacing
        word_margin_ratio: 0.12,     // Tight geometric gaps
        space_threshold_em_ratio: 0.18, // Smaller threshold
        space_char_multiplier: 0.6,  // Normal space handling
        use_adaptive_threshold: true,
        enable_document_type_detection: false,
        enable_email_detection: true,
        enable_citation_detection: true,
    };

    /// Policy profile - optimized for legal/policy documents
    /// Handles justified text, dense paragraphs (GDPR, regulations)
    /// Per analysis: GDPR, government regulations, policy documents
    pub const POLICY: Self = Self {
        name: "Policy",
        tj_offset_threshold: -110.0,    // Conservative for justified text
        word_margin_ratio: 0.18,        // More generous gaps
        space_threshold_em_ratio: 0.22, // Moderate fallback
        space_char_multiplier: 0.7,     // Emphasize spaces slightly
        use_adaptive_threshold: true,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Form profile - optimized for structured forms
    /// Preserves field alignment and boundaries
    /// Per analysis: IRS forms, applications, questionnaires
    pub const FORM: Self = Self {
        name: "Form",
        tj_offset_threshold: -120.0,   // Conservative for forms
        word_margin_ratio: 0.08,       // Very tight spacing
        space_threshold_em_ratio: 0.2, // Standard fallback
        space_char_multiplier: 0.5,    // Normal space handling
        use_adaptive_threshold: false,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Government profile - optimized for government documents
    /// Handles mixed formats: reports, tables, structured content
    pub const GOVERNMENT: Self = Self {
        name: "Government",
        tj_offset_threshold: -105.0,
        word_margin_ratio: 0.14,
        space_threshold_em_ratio: 0.2,
        space_char_multiplier: 0.65,
        use_adaptive_threshold: true,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// OCR profile - optimized for scanned documents
    /// More lenient spacing, handles OCR artifacts
    pub const SCANNED_OCR: Self = Self {
        name: "Scanned OCR",
        tj_offset_threshold: -85.0,     // More aggressive for OCR spacing
        word_margin_ratio: 0.2,         // Generous for OCR variability
        space_threshold_em_ratio: 0.15, // Smaller threshold
        space_char_multiplier: 0.75,    // Emphasize spaces
        use_adaptive_threshold: true,
        enable_document_type_detection: false,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Adaptive profile - auto-tunes based on document analysis
    /// Analyzes first page to determine optimal thresholds
    pub const ADAPTIVE: Self = Self {
        name: "Adaptive",
        tj_offset_threshold: -100.0,
        word_margin_ratio: 0.15,
        space_threshold_em_ratio: 0.2,
        space_char_multiplier: 0.65,
        use_adaptive_threshold: true,
        enable_document_type_detection: true,
        enable_email_detection: false,
        enable_citation_detection: false,
    };

    /// Create a profile for a specific document type
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        match doc_type {
            DocumentType::Academic => Self::ACADEMIC,
            DocumentType::Policy => Self::POLICY,
            DocumentType::Government => Self::GOVERNMENT,
            DocumentType::Form => Self::FORM,
            DocumentType::ScannedOCR => Self::SCANNED_OCR,
            DocumentType::Mixed => Self::BALANCED,
        }
    }

    /// List all available profiles for user selection
    pub fn all_profiles() -> &'static [&'static str] {
        &[
            Self::CONSERVATIVE.name,
            Self::AGGRESSIVE.name,
            Self::BALANCED.name,
            Self::ACADEMIC.name,
            Self::POLICY.name,
            Self::FORM.name,
            Self::GOVERNMENT.name,
            Self::SCANNED_OCR.name,
            Self::ADAPTIVE.name,
        ]
    }

    /// Get a profile by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "Conservative (Default)" => Some(Self::CONSERVATIVE),
            "Aggressive" => Some(Self::AGGRESSIVE),
            "Balanced" => Some(Self::BALANCED),
            "Academic" => Some(Self::ACADEMIC),
            "Policy" => Some(Self::POLICY),
            "Form" => Some(Self::FORM),
            "Government" => Some(Self::GOVERNMENT),
            "Scanned OCR" => Some(Self::SCANNED_OCR),
            "Adaptive" => Some(Self::ADAPTIVE),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        assert_eq!(ExtractionProfile::CONSERVATIVE.name, "Conservative (Default)");
        assert_eq!(ExtractionProfile::ACADEMIC.name, "Academic");
        assert_eq!(ExtractionProfile::POLICY.name, "Policy");
    }

    #[test]
    fn test_profiles_by_document_type() {
        assert_eq!(
            ExtractionProfile::for_document_type(DocumentType::Academic),
            ExtractionProfile::ACADEMIC
        );
        assert_eq!(
            ExtractionProfile::for_document_type(DocumentType::Policy),
            ExtractionProfile::POLICY
        );
    }

    #[test]
    fn test_profile_by_name() {
        assert!(ExtractionProfile::by_name("Academic").is_some());
        assert!(ExtractionProfile::by_name("InvalidProfile").is_none());
    }
}
