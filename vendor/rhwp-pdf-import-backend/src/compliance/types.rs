//! PDF/A compliance types and data structures.

#![forbid(unsafe_code)]

use std::fmt;

/// PDF/A conformance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdfALevel {
    /// PDF/A-1a: Full conformance with logical structure
    A1a,
    /// PDF/A-1b: Basic conformance (visual preservation)
    A1b,
    /// PDF/A-2a: PDF 1.7 based, full conformance
    A2a,
    /// PDF/A-2b: PDF 1.7 based, basic conformance
    A2b,
    /// PDF/A-2u: PDF/A-2b plus Unicode mapping
    A2u,
    /// PDF/A-3a: PDF/A-2a plus embedded files
    A3a,
    /// PDF/A-3b: PDF/A-2b plus embedded files
    A3b,
    /// PDF/A-3u: PDF/A-3b plus Unicode mapping
    A3u,
}

impl PdfALevel {
    /// Get the PDF/A part (1, 2, or 3).
    pub fn part(&self) -> PdfAPart {
        match self {
            PdfALevel::A1a | PdfALevel::A1b => PdfAPart::Part1,
            PdfALevel::A2a | PdfALevel::A2b | PdfALevel::A2u => PdfAPart::Part2,
            PdfALevel::A3a | PdfALevel::A3b | PdfALevel::A3u => PdfAPart::Part3,
        }
    }

    /// Get the conformance level letter.
    pub fn conformance(&self) -> char {
        match self {
            PdfALevel::A1a | PdfALevel::A2a | PdfALevel::A3a => 'A',
            PdfALevel::A1b | PdfALevel::A2b | PdfALevel::A3b => 'B',
            PdfALevel::A2u | PdfALevel::A3u => 'U',
        }
    }

    /// Check if this level requires logical structure (Tagged PDF).
    pub fn requires_structure(&self) -> bool {
        matches!(self, PdfALevel::A1a | PdfALevel::A2a | PdfALevel::A3a)
    }

    /// Check if this level requires Unicode mapping.
    pub fn requires_unicode(&self) -> bool {
        matches!(
            self,
            PdfALevel::A1a | PdfALevel::A2a | PdfALevel::A2u | PdfALevel::A3a | PdfALevel::A3u
        )
    }

    /// Check if transparency is allowed.
    pub fn allows_transparency(&self) -> bool {
        !matches!(self, PdfALevel::A1a | PdfALevel::A1b)
    }

    /// Check if JPEG2000 is allowed.
    pub fn allows_jpeg2000(&self) -> bool {
        !matches!(self, PdfALevel::A1a | PdfALevel::A1b)
    }

    /// Check if arbitrary embedded files are allowed.
    pub fn allows_embedded_files(&self) -> bool {
        matches!(self, PdfALevel::A3a | PdfALevel::A3b | PdfALevel::A3u)
    }

    /// Get the XMP pdfaid:part value.
    pub fn xmp_part(&self) -> &'static str {
        match self.part() {
            PdfAPart::Part1 => "1",
            PdfAPart::Part2 => "2",
            PdfAPart::Part3 => "3",
        }
    }

    /// Get the XMP pdfaid:conformance value.
    pub fn xmp_conformance(&self) -> &'static str {
        match self.conformance() {
            'A' => "A",
            'B' => "B",
            'U' => "U",
            _ => "B",
        }
    }

    /// Parse from XMP pdfaid:part and pdfaid:conformance values.
    pub fn from_xmp(part: &str, conformance: &str) -> Option<Self> {
        match (part, conformance.to_uppercase().as_str()) {
            ("1", "A") => Some(PdfALevel::A1a),
            ("1", "B") => Some(PdfALevel::A1b),
            ("2", "A") => Some(PdfALevel::A2a),
            ("2", "B") => Some(PdfALevel::A2b),
            ("2", "U") => Some(PdfALevel::A2u),
            ("3", "A") => Some(PdfALevel::A3a),
            ("3", "B") => Some(PdfALevel::A3b),
            ("3", "U") => Some(PdfALevel::A3u),
            _ => None,
        }
    }
}

impl fmt::Display for PdfALevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PdfALevel::A1a => "PDF/A-1a",
            PdfALevel::A1b => "PDF/A-1b",
            PdfALevel::A2a => "PDF/A-2a",
            PdfALevel::A2b => "PDF/A-2b",
            PdfALevel::A2u => "PDF/A-2u",
            PdfALevel::A3a => "PDF/A-3a",
            PdfALevel::A3b => "PDF/A-3b",
            PdfALevel::A3u => "PDF/A-3u",
        };
        write!(f, "{}", name)
    }
}

/// PDF/A part (version).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdfAPart {
    /// PDF/A-1 (based on PDF 1.4)
    Part1,
    /// PDF/A-2 (based on PDF 1.7)
    Part2,
    /// PDF/A-3 (based on PDF 1.7, with embedded files)
    Part3,
}

impl fmt::Display for PdfAPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfAPart::Part1 => write!(f, "PDF/A-1"),
            PdfAPart::Part2 => write!(f, "PDF/A-2"),
            PdfAPart::Part3 => write!(f, "PDF/A-3"),
        }
    }
}

/// Result of PDF/A validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the document is compliant with the target level.
    pub is_compliant: bool,
    /// The level validated against.
    pub level: PdfALevel,
    /// Detected PDF/A level from XMP metadata (if any).
    pub detected_level: Option<PdfALevel>,
    /// Compliance errors (violations).
    pub errors: Vec<ComplianceError>,
    /// Compliance warnings (non-fatal issues).
    pub warnings: Vec<ComplianceWarning>,
    /// Summary statistics.
    pub stats: ValidationStats,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_compliant: false,
            level: PdfALevel::A2b,
            detected_level: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats::default(),
        }
    }
}

impl ValidationResult {
    /// Create a new validation result for a specific level.
    pub fn new(level: PdfALevel) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: ComplianceError) {
        self.errors.push(error);
        self.is_compliant = false;
    }

    /// Add a warning to the result.
    pub fn add_warning(&mut self, warning: ComplianceWarning) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validation statistics.
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    /// Number of fonts checked.
    pub fonts_checked: usize,
    /// Number of fonts embedded.
    pub fonts_embedded: usize,
    /// Number of images checked.
    pub images_checked: usize,
    /// Number of color spaces checked.
    pub color_spaces_checked: usize,
    /// Number of annotations checked.
    pub annotations_checked: usize,
    /// Number of pages checked.
    pub pages_checked: usize,
}

/// Compliance error (violation).
#[derive(Debug, Clone)]
pub struct ComplianceError {
    /// Error code.
    pub code: ErrorCode,
    /// Human-readable message.
    pub message: String,
    /// Location in the document (if applicable).
    pub location: Option<String>,
    /// Clause reference in the standard.
    pub clause: Option<String>,
}

impl ComplianceError {
    /// Create a new compliance error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            clause: None,
        }
    }

    /// Set the location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set the clause reference.
    pub fn with_clause(mut self, clause: impl Into<String>) -> Self {
        self.clause = Some(clause.into());
        self
    }
}

impl fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref loc) = self.location {
            write!(f, " (at {})", loc)?;
        }
        Ok(())
    }
}

/// Error codes for PDF/A violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Metadata errors
    /// Missing XMP metadata
    MissingXmpMetadata,
    /// Missing PDF/A identification in XMP
    MissingPdfaIdentification,
    /// Invalid PDF/A identification
    InvalidPdfaIdentification,
    /// XMP metadata not synchronized with document info
    XmpMetadataMismatch,

    // Font errors
    /// Font not embedded
    FontNotEmbedded,
    /// Font missing required tables
    FontMissingTables,
    /// Font has invalid encoding
    FontInvalidEncoding,
    /// Font missing ToUnicode CMap
    FontMissingToUnicode,

    // Color errors
    /// Device-dependent color used without output intent
    DeviceColorWithoutIntent,
    /// Missing output intent
    MissingOutputIntent,
    /// Invalid ICC profile
    InvalidIccProfile,
    /// Incompatible ICC profile version
    IccProfileVersionMismatch,

    // Image errors
    /// Image uses unsupported compression
    UnsupportedImageCompression,
    /// Image has invalid color space
    InvalidImageColorSpace,
    /// LZW compression not allowed
    LzwCompressionNotAllowed,

    // Structure errors
    /// Missing document structure (for level A)
    MissingDocumentStructure,
    /// Invalid structure tree
    InvalidStructureTree,
    /// Missing language specification
    MissingLanguage,

    // Content errors
    /// Transparency used (PDF/A-1)
    TransparencyNotAllowed,
    /// JavaScript present
    JavaScriptNotAllowed,
    /// Audio/video content present
    MultimediaNotAllowed,
    /// External content reference
    ExternalContentNotAllowed,
    /// Encryption present
    EncryptionNotAllowed,

    // Annotation errors
    /// Invalid annotation
    InvalidAnnotation,
    /// Widget annotation without appearance stream
    MissingAppearanceStream,

    // Action errors
    /// Invalid action type
    InvalidAction,
    /// Launch action not allowed
    LaunchActionNotAllowed,

    // File errors
    /// Embedded file not allowed (PDF/A-1, PDF/A-2)
    EmbeddedFileNotAllowed,
    /// Embedded file missing AF relationship (PDF/A-3)
    MissingAfRelationship,

    // Other errors
    /// PostScript XObject not allowed
    PostScriptNotAllowed,
    /// Reference XObject not allowed
    ReferenceXObjectNotAllowed,
    /// Optional content (layers) issue
    OptionalContentIssue,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            ErrorCode::MissingXmpMetadata => "XMP-001",
            ErrorCode::MissingPdfaIdentification => "XMP-002",
            ErrorCode::InvalidPdfaIdentification => "XMP-003",
            ErrorCode::XmpMetadataMismatch => "XMP-004",
            ErrorCode::FontNotEmbedded => "FONT-001",
            ErrorCode::FontMissingTables => "FONT-002",
            ErrorCode::FontInvalidEncoding => "FONT-003",
            ErrorCode::FontMissingToUnicode => "FONT-004",
            ErrorCode::DeviceColorWithoutIntent => "COLOR-001",
            ErrorCode::MissingOutputIntent => "COLOR-002",
            ErrorCode::InvalidIccProfile => "COLOR-003",
            ErrorCode::IccProfileVersionMismatch => "COLOR-004",
            ErrorCode::UnsupportedImageCompression => "IMAGE-001",
            ErrorCode::InvalidImageColorSpace => "IMAGE-002",
            ErrorCode::LzwCompressionNotAllowed => "IMAGE-003",
            ErrorCode::MissingDocumentStructure => "STRUCT-001",
            ErrorCode::InvalidStructureTree => "STRUCT-002",
            ErrorCode::MissingLanguage => "STRUCT-003",
            ErrorCode::TransparencyNotAllowed => "CONTENT-001",
            ErrorCode::JavaScriptNotAllowed => "CONTENT-002",
            ErrorCode::MultimediaNotAllowed => "CONTENT-003",
            ErrorCode::ExternalContentNotAllowed => "CONTENT-004",
            ErrorCode::EncryptionNotAllowed => "CONTENT-005",
            ErrorCode::InvalidAnnotation => "ANNOT-001",
            ErrorCode::MissingAppearanceStream => "ANNOT-002",
            ErrorCode::InvalidAction => "ACTION-001",
            ErrorCode::LaunchActionNotAllowed => "ACTION-002",
            ErrorCode::EmbeddedFileNotAllowed => "FILE-001",
            ErrorCode::MissingAfRelationship => "FILE-002",
            ErrorCode::PostScriptNotAllowed => "XOBJ-001",
            ErrorCode::ReferenceXObjectNotAllowed => "XOBJ-002",
            ErrorCode::OptionalContentIssue => "OC-001",
        };
        write!(f, "{}", code)
    }
}

/// Compliance warning (non-fatal issue).
#[derive(Debug, Clone)]
pub struct ComplianceWarning {
    /// Warning code.
    pub code: WarningCode,
    /// Human-readable message.
    pub message: String,
    /// Location in the document (if applicable).
    pub location: Option<String>,
}

impl ComplianceWarning {
    /// Create a new compliance warning.
    pub fn new(code: WarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
        }
    }

    /// Set the location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

impl fmt::Display for ComplianceWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref loc) = self.location {
            write!(f, " (at {})", loc)?;
        }
        Ok(())
    }
}

/// Warning codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningCode {
    /// Deprecated feature used
    DeprecatedFeature,
    /// Large file size
    LargeFileSize,
    /// Missing recommended metadata
    MissingRecommendedMetadata,
    /// Font subset very small
    SmallFontSubset,
    /// High-resolution image
    HighResolutionImage,
    /// Complex structure
    ComplexStructure,
    /// Partial check performed (full validation requires additional features)
    PartialCheck,
    /// A required-to-embed standard-14 PostScript font has no open-
    /// source equivalent available to the conversion pipeline
    /// (e.g. `Symbol`, `ZapfDingbats`). The PDF/A pipeline tracks
    /// this as a known limitation rather than a hard error so a
    /// document that's otherwise compliant doesn't fail solely
    /// because of one unembeddable symbolic font. See issue #451.
    KnownUnembeddableFont,
}

impl fmt::Display for WarningCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            WarningCode::DeprecatedFeature => "WARN-001",
            WarningCode::LargeFileSize => "WARN-002",
            WarningCode::MissingRecommendedMetadata => "WARN-003",
            WarningCode::SmallFontSubset => "WARN-004",
            WarningCode::HighResolutionImage => "WARN-005",
            WarningCode::ComplexStructure => "WARN-006",
            WarningCode::PartialCheck => "WARN-007",
            WarningCode::KnownUnembeddableFont => "WARN-008",
        };
        write!(f, "{}", code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_a_level_properties() {
        assert_eq!(PdfALevel::A1a.part(), PdfAPart::Part1);
        assert_eq!(PdfALevel::A2b.part(), PdfAPart::Part2);
        assert_eq!(PdfALevel::A3u.part(), PdfAPart::Part3);

        assert!(PdfALevel::A1a.requires_structure());
        assert!(!PdfALevel::A1b.requires_structure());
        assert!(PdfALevel::A2a.requires_structure());

        assert!(!PdfALevel::A1a.allows_transparency());
        assert!(PdfALevel::A2b.allows_transparency());

        assert!(!PdfALevel::A2b.allows_embedded_files());
        assert!(PdfALevel::A3b.allows_embedded_files());
    }

    #[test]
    fn test_pdf_a_level_xmp() {
        assert_eq!(PdfALevel::A1b.xmp_part(), "1");
        assert_eq!(PdfALevel::A1b.xmp_conformance(), "B");
        assert_eq!(PdfALevel::A2u.xmp_conformance(), "U");
    }

    #[test]
    fn test_pdf_a_level_from_xmp() {
        assert_eq!(PdfALevel::from_xmp("1", "A"), Some(PdfALevel::A1a));
        assert_eq!(PdfALevel::from_xmp("2", "b"), Some(PdfALevel::A2b));
        assert_eq!(PdfALevel::from_xmp("3", "U"), Some(PdfALevel::A3u));
        assert_eq!(PdfALevel::from_xmp("4", "A"), None);
    }

    #[test]
    fn test_pdf_a_level_display() {
        assert_eq!(format!("{}", PdfALevel::A1b), "PDF/A-1b");
        assert_eq!(format!("{}", PdfALevel::A2u), "PDF/A-2u");
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new(PdfALevel::A2b);
        assert!(!result.is_compliant);
        assert!(!result.has_errors());

        result.add_error(ComplianceError::new(
            ErrorCode::FontNotEmbedded,
            "Font 'Arial' is not embedded",
        ));
        assert!(result.has_errors());
        assert!(!result.is_compliant);
    }

    #[test]
    fn test_compliance_error_display() {
        let error = ComplianceError::new(ErrorCode::FontNotEmbedded, "Font not embedded")
            .with_location("Page 1");
        let display = format!("{}", error);
        assert!(display.contains("[FONT-001]"));
        assert!(display.contains("Page 1"));
    }
}
