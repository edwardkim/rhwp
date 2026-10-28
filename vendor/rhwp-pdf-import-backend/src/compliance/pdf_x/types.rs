//! PDF/X compliance types and data structures.
//!
//! PDF/X (ISO 15930) is a subset of PDF designed for reliable print production.

use std::fmt;

/// PDF/X conformance level.
///
/// PDF/X standards define requirements for print production workflows,
/// ensuring reliable exchange of PDF files for printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdfXLevel {
    /// PDF/X-1a:2001 - CMYK and spot colors only, no transparency (ISO 15930-1)
    X1a2001,
    /// PDF/X-1a:2003 - Updated PDF/X-1a based on PDF 1.4 (ISO 15930-4)
    X1a2003,
    /// PDF/X-3:2002 - Allows ICC-based color management (ISO 15930-3)
    X32002,
    /// PDF/X-3:2003 - Updated PDF/X-3 based on PDF 1.4 (ISO 15930-6)
    X32003,
    /// PDF/X-4 - Allows transparency and layers (ISO 15930-7)
    X4,
    /// PDF/X-4p - PDF/X-4 with external ICC profiles (ISO 15930-7)
    X4p,
    /// PDF/X-5g - Allows external graphics content (ISO 15930-8)
    X5g,
    /// PDF/X-5n - External n-colorant ICC profiles (ISO 15930-8)
    X5n,
    /// PDF/X-5pg - Combination of X-5g and X-4p (ISO 15930-8)
    X5pg,
    /// PDF/X-6 - Based on PDF 2.0 (ISO 15930-9)
    X6,
}

impl PdfXLevel {
    /// Get the ISO standard number for this level.
    pub fn iso_standard(&self) -> &'static str {
        match self {
            Self::X1a2001 => "ISO 15930-1:2001",
            Self::X1a2003 => "ISO 15930-4:2003",
            Self::X32002 => "ISO 15930-3:2002",
            Self::X32003 => "ISO 15930-6:2003",
            Self::X4 | Self::X4p => "ISO 15930-7:2010",
            Self::X5g | Self::X5n | Self::X5pg => "ISO 15930-8:2010",
            Self::X6 => "ISO 15930-9:2020",
        }
    }

    /// Get the required base PDF version.
    pub fn required_pdf_version(&self) -> &'static str {
        match self {
            Self::X1a2001 | Self::X32002 => "1.3",
            Self::X1a2003 | Self::X32003 => "1.4",
            Self::X4 | Self::X4p | Self::X5g | Self::X5n | Self::X5pg => "1.6",
            Self::X6 => "2.0",
        }
    }

    /// Check if transparency is allowed in this level.
    pub fn allows_transparency(&self) -> bool {
        matches!(self, Self::X4 | Self::X4p | Self::X5g | Self::X5n | Self::X5pg | Self::X6)
    }

    /// Check if RGB color space is allowed.
    ///
    /// PDF/X-1a only allows CMYK and spot colors.
    /// PDF/X-3 and later allow ICC-based RGB.
    pub fn allows_rgb(&self) -> bool {
        !matches!(self, Self::X1a2001 | Self::X1a2003)
    }

    /// Check if layers (Optional Content Groups) are allowed.
    pub fn allows_layers(&self) -> bool {
        matches!(self, Self::X4 | Self::X4p | Self::X5g | Self::X5n | Self::X5pg | Self::X6)
    }

    /// Check if external ICC profiles are allowed.
    pub fn allows_external_icc(&self) -> bool {
        matches!(self, Self::X4p | Self::X5n | Self::X5pg)
    }

    /// Check if external graphics references are allowed.
    pub fn allows_external_graphics(&self) -> bool {
        matches!(self, Self::X5g | Self::X5pg)
    }

    /// Get the GTS_PDFXVersion value for the Info dictionary.
    pub fn gts_pdfx_version(&self) -> &'static str {
        match self {
            Self::X1a2001 => "PDF/X-1a:2001",
            Self::X1a2003 => "PDF/X-1a:2003",
            Self::X32002 => "PDF/X-3:2002",
            Self::X32003 => "PDF/X-3:2003",
            Self::X4 => "PDF/X-4",
            Self::X4p => "PDF/X-4p",
            Self::X5g => "PDF/X-5g",
            Self::X5n => "PDF/X-5n",
            Self::X5pg => "PDF/X-5pg",
            Self::X6 => "PDF/X-6",
        }
    }

    /// Get the XMP pdfxid:GTS_PDFXVersion value.
    pub fn xmp_version(&self) -> &'static str {
        self.gts_pdfx_version()
    }

    /// Parse from GTS_PDFXVersion string.
    pub fn from_gts_version(version: &str) -> Option<Self> {
        match version.trim() {
            "PDF/X-1a:2001" | "PDF/X-1:2001" => Some(Self::X1a2001),
            "PDF/X-1a:2003" | "PDF/X-1:2003" => Some(Self::X1a2003),
            "PDF/X-3:2002" => Some(Self::X32002),
            "PDF/X-3:2003" => Some(Self::X32003),
            "PDF/X-4" => Some(Self::X4),
            "PDF/X-4p" => Some(Self::X4p),
            "PDF/X-5g" => Some(Self::X5g),
            "PDF/X-5n" => Some(Self::X5n),
            "PDF/X-5pg" => Some(Self::X5pg),
            "PDF/X-6" => Some(Self::X6),
            _ => None,
        }
    }
}

impl fmt::Display for PdfXLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.gts_pdfx_version())
    }
}

/// PDF/X validation error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XErrorCode {
    // Color errors
    /// RGB color used in PDF/X-1a (only CMYK allowed)
    RgbColorNotAllowed,
    /// Lab color space not allowed
    LabColorNotAllowed,
    /// DeviceN color space missing required info
    DeviceNInvalid,
    /// ICC profile missing
    IccProfileMissing,
    /// ICC profile invalid or incompatible
    IccProfileInvalid,
    /// Uncalibrated device color without output intent
    DeviceColorWithoutIntent,

    // Transparency errors
    /// Transparency not allowed in this PDF/X level
    TransparencyNotAllowed,
    /// Blend mode not allowed
    BlendModeNotAllowed,
    /// Soft mask not allowed
    SoftMaskNotAllowed,
    /// SMask in ExtGState not allowed
    SMaskNotAllowed,

    // Font errors
    /// Font not embedded
    FontNotEmbedded,
    /// Type 3 font not allowed
    Type3FontNotAllowed,
    /// Font missing required glyph widths
    FontMissingWidths,
    /// Font subset missing required glyphs
    FontSubsetIncomplete,

    // Metadata errors
    /// Output intent missing
    OutputIntentMissing,
    /// Output intent invalid (wrong subtype)
    OutputIntentInvalid,
    /// Output condition identifier missing
    OutputConditionMissing,
    /// Trapped key missing in Info dictionary
    TrappedKeyMissing,
    /// XMP metadata missing
    XmpMetadataMissing,
    /// XMP metadata invalid or incomplete
    XmpMetadataInvalid,
    /// GTS_PDFXVersion key missing
    GtsPdfxVersionMissing,
    /// GTS_PDFXConformance key missing (for X-1a, X-3)
    GtsPdfxConformanceMissing,

    // Box errors
    /// TrimBox or ArtBox missing
    TrimOrArtBoxMissing,
    /// BleedBox extends beyond MediaBox
    BleedBoxInvalid,
    /// TrimBox extends beyond BleedBox
    TrimBoxInvalid,
    /// MediaBox missing
    MediaBoxMissing,
    /// Page boxes inconsistent
    BoxesInconsistent,

    // Content errors
    /// Encryption not allowed
    EncryptionNotAllowed,
    /// JavaScript not allowed
    JavaScriptNotAllowed,
    /// External content reference not allowed
    ExternalContentNotAllowed,
    /// Embedded file not allowed
    EmbeddedFileNotAllowed,
    /// Form XObject invalid
    FormXObjectInvalid,
    /// PostScript XObject not allowed
    PostScriptXObjectNotAllowed,
    /// Reference XObject not allowed (except in X-5)
    ReferenceXObjectNotAllowed,

    // Annotation errors
    /// Annotation type not allowed
    AnnotationNotAllowed,
    /// PrinterMark annotation invalid
    PrinterMarkInvalid,
    /// TrapNet annotation invalid
    TrapNetInvalid,

    // Action errors
    /// Action type not allowed
    ActionNotAllowed,

    // Other errors
    /// Transfer function not allowed
    TransferFunctionNotAllowed,
    /// Halftone not allowed (device-dependent)
    HalftoneTypeNotAllowed,
    /// Alternate image not allowed
    AlternateImageNotAllowed,
    /// OPI not allowed
    OpiNotAllowed,
    /// Preseparated pages not allowed
    PreseparatedNotAllowed,
}

impl fmt::Display for XErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            // Color errors
            Self::RgbColorNotAllowed => "XCOLOR-001",
            Self::LabColorNotAllowed => "XCOLOR-002",
            Self::DeviceNInvalid => "XCOLOR-003",
            Self::IccProfileMissing => "XCOLOR-004",
            Self::IccProfileInvalid => "XCOLOR-005",
            Self::DeviceColorWithoutIntent => "XCOLOR-006",
            // Transparency errors
            Self::TransparencyNotAllowed => "XTRANS-001",
            Self::BlendModeNotAllowed => "XTRANS-002",
            Self::SoftMaskNotAllowed => "XTRANS-003",
            Self::SMaskNotAllowed => "XTRANS-004",
            // Font errors
            Self::FontNotEmbedded => "XFONT-001",
            Self::Type3FontNotAllowed => "XFONT-002",
            Self::FontMissingWidths => "XFONT-003",
            Self::FontSubsetIncomplete => "XFONT-004",
            // Metadata errors
            Self::OutputIntentMissing => "XMETA-001",
            Self::OutputIntentInvalid => "XMETA-002",
            Self::OutputConditionMissing => "XMETA-003",
            Self::TrappedKeyMissing => "XMETA-004",
            Self::XmpMetadataMissing => "XMETA-005",
            Self::XmpMetadataInvalid => "XMETA-006",
            Self::GtsPdfxVersionMissing => "XMETA-007",
            Self::GtsPdfxConformanceMissing => "XMETA-008",
            // Box errors
            Self::TrimOrArtBoxMissing => "XBOX-001",
            Self::BleedBoxInvalid => "XBOX-002",
            Self::TrimBoxInvalid => "XBOX-003",
            Self::MediaBoxMissing => "XBOX-004",
            Self::BoxesInconsistent => "XBOX-005",
            // Content errors
            Self::EncryptionNotAllowed => "XCONT-001",
            Self::JavaScriptNotAllowed => "XCONT-002",
            Self::ExternalContentNotAllowed => "XCONT-003",
            Self::EmbeddedFileNotAllowed => "XCONT-004",
            Self::FormXObjectInvalid => "XCONT-005",
            Self::PostScriptXObjectNotAllowed => "XCONT-006",
            Self::ReferenceXObjectNotAllowed => "XCONT-007",
            // Annotation errors
            Self::AnnotationNotAllowed => "XANNOT-001",
            Self::PrinterMarkInvalid => "XANNOT-002",
            Self::TrapNetInvalid => "XANNOT-003",
            // Action errors
            Self::ActionNotAllowed => "XACTION-001",
            // Other errors
            Self::TransferFunctionNotAllowed => "XOTHER-001",
            Self::HalftoneTypeNotAllowed => "XOTHER-002",
            Self::AlternateImageNotAllowed => "XOTHER-003",
            Self::OpiNotAllowed => "XOTHER-004",
            Self::PreseparatedNotAllowed => "XOTHER-005",
        };
        write!(f, "{}", code)
    }
}

/// Severity level for compliance issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XSeverity {
    /// Must be fixed for compliance
    Error,
    /// Recommended to fix but not required
    Warning,
}

/// PDF/X compliance error.
#[derive(Debug, Clone)]
pub struct XComplianceError {
    /// Error code
    pub code: XErrorCode,
    /// Human-readable message
    pub message: String,
    /// Page number (0-indexed, if applicable)
    pub page: Option<usize>,
    /// Object ID (if applicable)
    pub object_id: Option<u32>,
    /// Severity level
    pub severity: XSeverity,
    /// ISO clause reference
    pub clause: Option<String>,
}

impl XComplianceError {
    /// Create a new compliance error.
    pub fn new(code: XErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            page: None,
            object_id: None,
            severity: XSeverity::Error,
            clause: None,
        }
    }

    /// Create a new compliance warning.
    pub fn warning(code: XErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            page: None,
            object_id: None,
            severity: XSeverity::Warning,
            clause: None,
        }
    }

    /// Set the page number.
    pub fn with_page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the object ID.
    pub fn with_object_id(mut self, id: u32) -> Self {
        self.object_id = Some(id);
        self
    }

    /// Set the ISO clause reference.
    pub fn with_clause(mut self, clause: impl Into<String>) -> Self {
        self.clause = Some(clause.into());
        self
    }

    /// Check if this is an error (not a warning).
    pub fn is_error(&self) -> bool {
        self.severity == XSeverity::Error
    }
}

impl fmt::Display for XComplianceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(page) = self.page {
            write!(f, " (page {})", page + 1)?;
        }
        if let Some(id) = self.object_id {
            write!(f, " (object {})", id)?;
        }
        Ok(())
    }
}

/// PDF/X validation result.
#[derive(Debug, Clone)]
pub struct XValidationResult {
    /// Whether the document is compliant
    pub is_compliant: bool,
    /// The level validated against
    pub level: PdfXLevel,
    /// Detected PDF/X level from metadata (if any)
    pub detected_level: Option<PdfXLevel>,
    /// Compliance errors
    pub errors: Vec<XComplianceError>,
    /// Compliance warnings
    pub warnings: Vec<XComplianceError>,
    /// Validation statistics
    pub stats: XValidationStats,
}

impl XValidationResult {
    /// Create a new validation result for a specific level.
    pub fn new(level: PdfXLevel) -> Self {
        Self {
            is_compliant: true,
            level,
            detected_level: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: XValidationStats::default(),
        }
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: XComplianceError) {
        if error.is_error() {
            self.is_compliant = false;
            self.errors.push(error);
        } else {
            self.warnings.push(error);
        }
    }

    /// Add a warning to the result.
    pub fn add_warning(&mut self, warning: XComplianceError) {
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

    /// Get total issue count (errors + warnings).
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// Validation statistics.
#[derive(Debug, Clone, Default)]
pub struct XValidationStats {
    /// Number of pages checked
    pub pages_checked: usize,
    /// Number of fonts checked
    pub fonts_checked: usize,
    /// Number of fonts embedded
    pub fonts_embedded: usize,
    /// Number of images checked
    pub images_checked: usize,
    /// Number of annotations checked
    pub annotations_checked: usize,
    /// Color spaces found in document
    pub color_spaces_found: Vec<String>,
    /// Whether transparency was detected
    pub has_transparency: bool,
    /// Whether layers (OCG) were detected
    pub has_layers: bool,
    /// Output intent type (if found)
    pub output_intent: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_x_level_properties() {
        // X-1a doesn't allow transparency or RGB
        assert!(!PdfXLevel::X1a2001.allows_transparency());
        assert!(!PdfXLevel::X1a2001.allows_rgb());
        assert!(!PdfXLevel::X1a2003.allows_layers());

        // X-3 allows RGB but not transparency
        assert!(!PdfXLevel::X32003.allows_transparency());
        assert!(PdfXLevel::X32003.allows_rgb());

        // X-4 allows everything
        assert!(PdfXLevel::X4.allows_transparency());
        assert!(PdfXLevel::X4.allows_rgb());
        assert!(PdfXLevel::X4.allows_layers());

        // X-4p allows external ICC
        assert!(PdfXLevel::X4p.allows_external_icc());
        assert!(!PdfXLevel::X4.allows_external_icc());

        // X-5g allows external graphics
        assert!(PdfXLevel::X5g.allows_external_graphics());
        assert!(!PdfXLevel::X4.allows_external_graphics());
    }

    #[test]
    fn test_pdf_x_level_versions() {
        assert_eq!(PdfXLevel::X1a2001.required_pdf_version(), "1.3");
        assert_eq!(PdfXLevel::X1a2003.required_pdf_version(), "1.4");
        assert_eq!(PdfXLevel::X4.required_pdf_version(), "1.6");
        assert_eq!(PdfXLevel::X6.required_pdf_version(), "2.0");
    }

    #[test]
    fn test_pdf_x_level_display() {
        assert_eq!(format!("{}", PdfXLevel::X1a2001), "PDF/X-1a:2001");
        assert_eq!(format!("{}", PdfXLevel::X4), "PDF/X-4");
        assert_eq!(format!("{}", PdfXLevel::X5pg), "PDF/X-5pg");
    }

    #[test]
    fn test_pdf_x_level_from_gts() {
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-1a:2001"), Some(PdfXLevel::X1a2001));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-4"), Some(PdfXLevel::X4));
        assert_eq!(PdfXLevel::from_gts_version("invalid"), None);
    }

    #[test]
    fn test_pdf_x_level_iso_standard() {
        assert_eq!(PdfXLevel::X1a2001.iso_standard(), "ISO 15930-1:2001");
        assert_eq!(PdfXLevel::X4.iso_standard(), "ISO 15930-7:2010");
        assert_eq!(PdfXLevel::X6.iso_standard(), "ISO 15930-9:2020");
    }

    #[test]
    fn test_compliance_error() {
        let error = XComplianceError::new(XErrorCode::RgbColorNotAllowed, "RGB color space found")
            .with_page(0)
            .with_clause("6.2.3");

        assert!(error.is_error());
        assert_eq!(error.page, Some(0));
        assert_eq!(error.clause, Some("6.2.3".to_string()));

        let display = format!("{}", error);
        assert!(display.contains("[XCOLOR-001]"));
        assert!(display.contains("page 1"));
    }

    #[test]
    fn test_validation_result() {
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        assert!(result.is_compliant);
        assert!(!result.has_errors());

        result.add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "Font not embedded"));

        assert!(!result.is_compliant);
        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
    }
}
