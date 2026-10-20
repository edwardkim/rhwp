//! PDF/UA (Universal Accessibility) validation module.
//!
//! This module provides functionality for validating PDF documents against
//! PDF/UA (ISO 14289) accessibility standards.
//!
//! ## PDF/UA Requirements Overview
//!
//! PDF/UA documents must:
//! - Be Tagged PDFs (logical structure)
//! - Include all content in the structure tree
//! - Specify document language
//! - Use standard structure types or role-mapped types
//! - Provide alternative text for images
//! - Have accessible names for form fields
//! - Properly associate table headers with cells
//! - Follow natural reading order
//! - Not rely on JavaScript for critical content
//! - Have document title in metadata
//!
//! ## Standards Reference
//!
//! - ISO 14289-1:2014 (PDF/UA-1)
//! - ISO 14289-2 (PDF/UA-2, in development)
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::compliance::{PdfUaValidator, PdfUaLevel, UaValidationResult};
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//!
//! let validator = PdfUaValidator::new();
//! let result = validator.validate(&mut pdf, PdfUaLevel::Ua1)?;
//!
//! if result.is_compliant {
//!     println!("Document is PDF/UA-1 compliant");
//! } else {
//!     for error in &result.errors {
//!         println!("Accessibility violation: {}", error);
//!     }
//! }
//! ```

use super::types::{ComplianceWarning, WarningCode};
use crate::document::PdfDocument;
use crate::error::Result;
use crate::object::Object;
use std::fmt;

/// PDF/UA conformance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdfUaLevel {
    /// PDF/UA-1 (ISO 14289-1:2014)
    Ua1,
    /// PDF/UA-2 (in development, based on PDF 2.0)
    Ua2,
}

impl fmt::Display for PdfUaLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfUaLevel::Ua1 => write!(f, "PDF/UA-1"),
            PdfUaLevel::Ua2 => write!(f, "PDF/UA-2"),
        }
    }
}

impl PdfUaLevel {
    /// Get the XMP pdfuaid:part value.
    pub fn xmp_part(&self) -> &'static str {
        match self {
            PdfUaLevel::Ua1 => "1",
            PdfUaLevel::Ua2 => "2",
        }
    }
}

/// Result of PDF/UA validation.
#[derive(Debug, Clone)]
pub struct UaValidationResult {
    /// Whether the document is compliant with the target level.
    pub is_compliant: bool,
    /// The level validated against.
    pub level: PdfUaLevel,
    /// Accessibility errors (violations).
    pub errors: Vec<UaComplianceError>,
    /// Accessibility warnings (non-fatal issues).
    pub warnings: Vec<ComplianceWarning>,
    /// Summary statistics.
    pub stats: UaValidationStats,
}

impl Default for UaValidationResult {
    fn default() -> Self {
        Self {
            is_compliant: true,
            level: PdfUaLevel::Ua1,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: UaValidationStats::default(),
        }
    }
}

impl UaValidationResult {
    /// Create a new validation result for a specific level.
    pub fn new(level: PdfUaLevel) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: UaComplianceError) {
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

/// PDF/UA validation statistics.
#[derive(Debug, Clone, Default)]
pub struct UaValidationStats {
    /// Number of structure elements checked.
    pub structure_elements_checked: usize,
    /// Number of images checked.
    pub images_checked: usize,
    /// Number of images with alt text.
    pub images_with_alt: usize,
    /// Number of tables checked.
    pub tables_checked: usize,
    /// Number of form fields checked.
    pub form_fields_checked: usize,
    /// Number of annotations checked.
    pub annotations_checked: usize,
    /// Number of pages checked.
    pub pages_checked: usize,
}

/// PDF/UA compliance error (accessibility violation).
#[derive(Debug, Clone)]
pub struct UaComplianceError {
    /// Error code.
    pub code: UaErrorCode,
    /// Human-readable message.
    pub message: String,
    /// Location in the document (if applicable).
    pub location: Option<String>,
    /// WCAG reference (if applicable).
    pub wcag_ref: Option<String>,
    /// ISO 14289 clause reference.
    pub clause: Option<String>,
}

impl UaComplianceError {
    /// Create a new compliance error.
    pub fn new(code: UaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            wcag_ref: None,
            clause: None,
        }
    }

    /// Set the location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set the WCAG reference.
    pub fn with_wcag(mut self, wcag_ref: impl Into<String>) -> Self {
        self.wcag_ref = Some(wcag_ref.into());
        self
    }

    /// Set the ISO clause reference.
    pub fn with_clause(mut self, clause: impl Into<String>) -> Self {
        self.clause = Some(clause.into());
        self
    }
}

impl fmt::Display for UaComplianceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref loc) = self.location {
            write!(f, " (at {})", loc)?;
        }
        if let Some(ref wcag) = self.wcag_ref {
            write!(f, " [WCAG {}]", wcag)?;
        }
        Ok(())
    }
}

/// Error codes for PDF/UA violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UaErrorCode {
    // Document-level errors
    /// Document is not a Tagged PDF
    NotTaggedPdf,
    /// Missing document language
    MissingLanguage,
    /// Missing document title
    MissingTitle,
    /// Document title not displayed in window title
    TitleNotDisplayed,
    /// Missing PDF/UA identification in XMP
    MissingPdfuaId,
    /// Invalid PDF/UA identification
    InvalidPdfuaId,

    // Structure errors
    /// Content not in structure tree
    ContentNotTagged,
    /// Invalid structure type
    InvalidStructureType,
    /// Role mapping missing for custom type
    MissingRoleMapping,
    /// Invalid nesting of structure elements
    InvalidStructureNesting,
    /// Heading levels not sequential
    HeadingLevelSkipped,

    // Figure/Image errors
    /// Figure missing alternative text
    FigureMissingAlt,
    /// Decorative figure not marked as artifact
    DecorativeNotArtifact,
    /// Figure caption not associated
    FigureCaptionNotAssociated,

    // Table errors
    /// Table missing headers
    TableMissingHeaders,
    /// Table header cell not TH
    TableHeaderNotTh,
    /// Table data cell not TD
    TableDataNotTd,
    /// Table headers not associated with cells
    TableHeadersNotAssociated,
    /// Table scope not specified
    TableScopeMissing,
    /// Complex table without IDs/headers
    ComplexTableNoIds,

    // Form errors
    /// Form field missing accessible name
    FormFieldMissingName,
    /// Form field missing tooltip
    FormFieldMissingTooltip,
    /// Required field not indicated
    RequiredFieldNotIndicated,
    /// Form without submit button
    FormNoSubmitButton,

    // Link errors
    /// Link text not descriptive
    LinkTextNotDescriptive,
    /// Link destination not specified
    LinkNoDestination,

    // List errors
    /// List items not properly marked
    ListItemsNotMarked,
    /// Nested list not properly structured
    NestedListInvalid,

    // Annotation errors
    /// Annotation not tagged
    AnnotationNotTagged,
    /// Annotation missing contents
    AnnotationMissingContents,
    /// Widget missing role
    WidgetMissingRole,

    // Font/Text errors
    /// Font not embedded
    FontNotEmbedded,
    /// Missing Unicode mapping
    MissingUnicodeMapping,
    /// Text without ActualText (for graphical text)
    MissingActualText,

    // Color errors
    /// Insufficient color contrast
    InsufficientContrast,
    /// Color alone conveys information
    ColorOnlyInformation,

    // Other errors
    /// JavaScript without accessible alternative
    JavaScriptNoAlternative,
    /// Audio/video without captions
    MultimediaNoCaptions,
    /// Reading order not logical
    ReadingOrderInvalid,
    /// Bookmark structure doesn't match headings
    BookmarksMismatch,
}

impl fmt::Display for UaErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            // Document-level
            UaErrorCode::NotTaggedPdf => "UA-DOC-001",
            UaErrorCode::MissingLanguage => "UA-DOC-002",
            UaErrorCode::MissingTitle => "UA-DOC-003",
            UaErrorCode::TitleNotDisplayed => "UA-DOC-004",
            UaErrorCode::MissingPdfuaId => "UA-DOC-005",
            UaErrorCode::InvalidPdfuaId => "UA-DOC-006",

            // Structure
            UaErrorCode::ContentNotTagged => "UA-STRUCT-001",
            UaErrorCode::InvalidStructureType => "UA-STRUCT-002",
            UaErrorCode::MissingRoleMapping => "UA-STRUCT-003",
            UaErrorCode::InvalidStructureNesting => "UA-STRUCT-004",
            UaErrorCode::HeadingLevelSkipped => "UA-STRUCT-005",

            // Figure
            UaErrorCode::FigureMissingAlt => "UA-FIG-001",
            UaErrorCode::DecorativeNotArtifact => "UA-FIG-002",
            UaErrorCode::FigureCaptionNotAssociated => "UA-FIG-003",

            // Table
            UaErrorCode::TableMissingHeaders => "UA-TBL-001",
            UaErrorCode::TableHeaderNotTh => "UA-TBL-002",
            UaErrorCode::TableDataNotTd => "UA-TBL-003",
            UaErrorCode::TableHeadersNotAssociated => "UA-TBL-004",
            UaErrorCode::TableScopeMissing => "UA-TBL-005",
            UaErrorCode::ComplexTableNoIds => "UA-TBL-006",

            // Form
            UaErrorCode::FormFieldMissingName => "UA-FORM-001",
            UaErrorCode::FormFieldMissingTooltip => "UA-FORM-002",
            UaErrorCode::RequiredFieldNotIndicated => "UA-FORM-003",
            UaErrorCode::FormNoSubmitButton => "UA-FORM-004",

            // Link
            UaErrorCode::LinkTextNotDescriptive => "UA-LINK-001",
            UaErrorCode::LinkNoDestination => "UA-LINK-002",

            // List
            UaErrorCode::ListItemsNotMarked => "UA-LIST-001",
            UaErrorCode::NestedListInvalid => "UA-LIST-002",

            // Annotation
            UaErrorCode::AnnotationNotTagged => "UA-ANNOT-001",
            UaErrorCode::AnnotationMissingContents => "UA-ANNOT-002",
            UaErrorCode::WidgetMissingRole => "UA-ANNOT-003",

            // Font/Text
            UaErrorCode::FontNotEmbedded => "UA-TEXT-001",
            UaErrorCode::MissingUnicodeMapping => "UA-TEXT-002",
            UaErrorCode::MissingActualText => "UA-TEXT-003",

            // Color
            UaErrorCode::InsufficientContrast => "UA-COLOR-001",
            UaErrorCode::ColorOnlyInformation => "UA-COLOR-002",

            // Other
            UaErrorCode::JavaScriptNoAlternative => "UA-OTHER-001",
            UaErrorCode::MultimediaNoCaptions => "UA-OTHER-002",
            UaErrorCode::ReadingOrderInvalid => "UA-OTHER-003",
            UaErrorCode::BookmarksMismatch => "UA-OTHER-004",
        };
        write!(f, "{}", code)
    }
}

/// PDF/UA validator.
///
/// Validates PDF documents against PDF/UA (ISO 14289) accessibility standards.
pub struct PdfUaValidator {
    /// Check for heading level sequence.
    check_heading_sequence: bool,
    /// Check for color contrast (approximate).
    check_color_contrast: bool,
    /// Custom structure types that are allowed.
    allowed_custom_types: Vec<String>,
}

impl Default for PdfUaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfUaValidator {
    /// Create a new PDF/UA validator with default settings.
    pub fn new() -> Self {
        Self {
            check_heading_sequence: true,
            check_color_contrast: false, // Requires rendering, disabled by default
            allowed_custom_types: Vec::new(),
        }
    }

    /// Enable or disable heading sequence checking.
    pub fn check_heading_sequence(mut self, enabled: bool) -> Self {
        self.check_heading_sequence = enabled;
        self
    }

    /// Enable or disable color contrast checking.
    pub fn check_color_contrast(mut self, enabled: bool) -> Self {
        self.check_color_contrast = enabled;
        self
    }

    /// Add allowed custom structure types.
    pub fn allow_custom_types(mut self, types: Vec<String>) -> Self {
        self.allowed_custom_types = types;
        self
    }

    /// Validate a document against the specified PDF/UA level.
    pub fn validate(
        &self,
        document: &mut PdfDocument,
        level: PdfUaLevel,
    ) -> Result<UaValidationResult> {
        let mut result = UaValidationResult::new(level);

        // Core PDF/UA validations
        self.validate_tagged_pdf(document, &mut result)?;
        self.validate_language(document, &mut result)?;
        self.validate_title(document, &mut result)?;
        self.validate_structure_tree(document, &mut result)?;
        self.validate_figures(document, &mut result)?;
        self.validate_tables(document, &mut result)?;
        self.validate_form_fields(document, &mut result)?;
        self.validate_annotations(document, &mut result)?;

        // Set compliance status
        result.is_compliant = result.errors.is_empty();

        Ok(result)
    }

    /// Check if document is a Tagged PDF.
    fn validate_tagged_pdf(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        let catalog = document.catalog()?;
        let catalog_dict = match catalog {
            Object::Dictionary(d) => d,
            _ => {
                result.add_error(
                    UaComplianceError::new(UaErrorCode::NotTaggedPdf, "Invalid document catalog")
                        .with_clause("7.1"),
                );
                return Ok(());
            },
        };

        // Check for MarkInfo with Marked = true
        let is_marked = if let Some(mark_info) = catalog_dict.get("MarkInfo") {
            let resolved_mark_info = document.resolve_references(mark_info, 1)?;
            if let Object::Dictionary(mi) = resolved_mark_info {
                matches!(mi.get("Marked"), Some(Object::Boolean(true)))
            } else {
                false
            }
        } else {
            false
        };

        if !is_marked {
            result.add_error(
                UaComplianceError::new(
                    UaErrorCode::NotTaggedPdf,
                    "Document must be a Tagged PDF (MarkInfo/Marked = true)",
                )
                .with_clause("7.1")
                .with_wcag("1.3.1"),
            );
        }

        // Check for StructTreeRoot
        if !catalog_dict.contains_key("StructTreeRoot") {
            result.add_error(
                UaComplianceError::new(
                    UaErrorCode::NotTaggedPdf,
                    "Document must have a structure tree (StructTreeRoot)",
                )
                .with_clause("7.1")
                .with_wcag("1.3.1"),
            );
        }

        Ok(())
    }

    /// Check for document language specification.
    fn validate_language(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        let catalog = document.catalog()?;
        let catalog_dict = match catalog {
            Object::Dictionary(d) => d,
            _ => return Ok(()),
        };

        // Check for /Lang in catalog
        if !catalog_dict.contains_key("Lang") {
            result.add_error(
                UaComplianceError::new(
                    UaErrorCode::MissingLanguage,
                    "Document must specify a primary language (/Lang in catalog)",
                )
                .with_clause("7.2")
                .with_wcag("3.1.1"),
            );
        } else {
            // Validate language tag format (should be BCP 47)
            if let Some(Object::String(lang)) = catalog_dict.get("Lang") {
                let lang_str = String::from_utf8_lossy(lang);
                if lang_str.is_empty() || !is_valid_language_tag(&lang_str) {
                    result.add_warning(ComplianceWarning::new(
                        WarningCode::MissingRecommendedMetadata,
                        format!("Language tag '{}' may not be a valid BCP 47 tag", lang_str),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check for document title.
    fn validate_title(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        let catalog = document.catalog()?;
        let catalog_dict = match catalog {
            Object::Dictionary(d) => d,
            _ => return Ok(()),
        };

        // Check ViewerPreferences for DisplayDocTitle
        let display_title = if let Some(vp) = catalog_dict.get("ViewerPreferences") {
            let resolved_vp = document.resolve_references(vp, 1)?;
            if let Object::Dictionary(vp_dict) = resolved_vp {
                matches!(vp_dict.get("DisplayDocTitle"), Some(Object::Boolean(true)))
            } else {
                false
            }
        } else {
            false
        };

        if !display_title {
            result.add_error(
                UaComplianceError::new(
                    UaErrorCode::TitleNotDisplayed,
                    "ViewerPreferences/DisplayDocTitle must be true",
                )
                .with_clause("7.1")
                .with_wcag("2.4.2"),
            );
        }

        // Check for title in document info or XMP metadata
        // Clone the trailer to avoid borrow conflicts
        let trailer = document.trailer().clone();
        let has_title = if let Object::Dictionary(trailer_dict) = trailer {
            if let Some(info_ref) = trailer_dict.get("Info") {
                let info_obj = document.resolve_references(info_ref, 1)?;
                if let Object::Dictionary(info_dict) = info_obj {
                    if let Some(Object::String(title)) = info_dict.get("Title") {
                        !title.is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !has_title {
            result.add_error(
                UaComplianceError::new(
                    UaErrorCode::MissingTitle,
                    "Document must have a title in document info or XMP metadata",
                )
                .with_clause("7.1")
                .with_wcag("2.4.2"),
            );
        }

        Ok(())
    }

    /// Validate structure tree.
    fn validate_structure_tree(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        let catalog = document.catalog()?;
        let catalog_dict = match catalog {
            Object::Dictionary(d) => d,
            _ => return Ok(()),
        };

        let struct_tree_root = match catalog_dict.get("StructTreeRoot") {
            Some(obj) => document.resolve_references(obj, 1)?,
            None => return Ok(()), // Already reported in validate_tagged_pdf
        };

        if let Object::Dictionary(str_dict) = struct_tree_root {
            // Check for RoleMap (custom structure types need role mapping)
            let has_role_map = str_dict.contains_key("RoleMap");

            // Check for K (children)
            if !str_dict.contains_key("K") {
                result.add_warning(ComplianceWarning::new(
                    WarningCode::PartialCheck,
                    "Structure tree root has no children",
                ));
            }

            // If checking heading sequence, validate heading order
            if self.check_heading_sequence {
                // This would require traversing the structure tree
                // For now, add a warning about partial checking
                result.add_warning(ComplianceWarning::new(
                    WarningCode::PartialCheck,
                    "Heading sequence validation requires full structure tree traversal",
                ));
            }

            result.stats.structure_elements_checked += 1;

            // Suppress unused variable warning
            let _ = has_role_map;
        }

        Ok(())
    }

    /// Validate figures (images) have alternative text.
    fn validate_figures(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        // Figure validation requires traversing structure tree and checking Alt attributes
        // For now, add a warning about partial checking
        result.add_warning(ComplianceWarning::new(
            WarningCode::PartialCheck,
            "Figure alt text validation requires structure tree traversal",
        ));

        // Suppress unused variable warning
        let _ = document;

        Ok(())
    }

    /// Validate tables have proper headers.
    fn validate_tables(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        // Table validation requires traversing structure tree
        // For now, add a warning about partial checking
        result.add_warning(ComplianceWarning::new(
            WarningCode::PartialCheck,
            "Table header validation requires structure tree traversal",
        ));

        // Suppress unused variable warning
        let _ = document;

        Ok(())
    }

    /// Validate form fields have accessible names.
    fn validate_form_fields(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        let catalog = document.catalog()?;
        let catalog_dict = match catalog {
            Object::Dictionary(d) => d,
            _ => return Ok(()),
        };

        // Check for AcroForm
        let acro_form = match catalog_dict.get("AcroForm") {
            Some(obj) => document.resolve_references(obj, 1)?,
            None => return Ok(()), // No forms
        };

        if let Object::Dictionary(form_dict) = acro_form {
            // Check for Fields
            if let Some(fields) = form_dict.get("Fields") {
                let resolved_fields = document.resolve_references(fields, 1)?;
                if let Object::Array(fields_arr) = resolved_fields {
                    for field in &fields_arr {
                        let resolved_field = document.resolve_references(field, 1)?;
                        if let Object::Dictionary(field_dict) = resolved_field {
                            // Check for TU (tooltip/accessible name)
                            if !field_dict.contains_key("TU") && !field_dict.contains_key("T") {
                                result.add_warning(ComplianceWarning::new(
                                    WarningCode::MissingRecommendedMetadata,
                                    "Form field missing TU (tooltip) or T (name)",
                                ));
                            }
                            result.stats.form_fields_checked += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate annotations are accessible.
    fn validate_annotations(
        &self,
        document: &mut PdfDocument,
        result: &mut UaValidationResult,
    ) -> Result<()> {
        // Annotation validation requires page-level access
        // For now, add a warning about partial checking
        result.add_warning(ComplianceWarning::new(
            WarningCode::PartialCheck,
            "Full annotation validation requires page-level access",
        ));

        // Suppress unused variable warning
        let _ = document;

        Ok(())
    }
}

/// Simple validation of BCP 47 language tag format.
fn is_valid_language_tag(tag: &str) -> bool {
    // Basic validation: 2-3 letter primary tag, optionally followed by subtags
    let parts: Vec<&str> = tag.split('-').collect();
    if parts.is_empty() {
        return false;
    }

    // Primary language subtag should be 2-3 letters
    let primary = parts[0];
    if primary.len() < 2 || primary.len() > 3 || !primary.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    true
}

/// Convenience function to validate a document against PDF/UA-1.
pub fn validate_pdf_ua(
    document: &mut PdfDocument,
    level: PdfUaLevel,
) -> Result<UaValidationResult> {
    let validator = PdfUaValidator::new();
    validator.validate(document, level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_ua_level_display() {
        assert_eq!(format!("{}", PdfUaLevel::Ua1), "PDF/UA-1");
        assert_eq!(format!("{}", PdfUaLevel::Ua2), "PDF/UA-2");
    }

    #[test]
    fn test_pdf_ua_level_xmp() {
        assert_eq!(PdfUaLevel::Ua1.xmp_part(), "1");
        assert_eq!(PdfUaLevel::Ua2.xmp_part(), "2");
    }

    #[test]
    fn test_validation_result() {
        let mut result = UaValidationResult::new(PdfUaLevel::Ua1);
        assert!(result.is_compliant);
        assert!(!result.has_errors());

        result.add_error(UaComplianceError::new(UaErrorCode::NotTaggedPdf, "Not tagged"));
        assert!(result.has_errors());
        assert!(!result.is_compliant);
    }

    #[test]
    fn test_compliance_error_display() {
        let error = UaComplianceError::new(UaErrorCode::FigureMissingAlt, "Image without alt text")
            .with_location("Page 1")
            .with_wcag("1.1.1");
        let display = format!("{}", error);
        assert!(display.contains("[UA-FIG-001]"));
        assert!(display.contains("Page 1"));
        assert!(display.contains("WCAG 1.1.1"));
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", UaErrorCode::NotTaggedPdf), "UA-DOC-001");
        assert_eq!(format!("{}", UaErrorCode::FigureMissingAlt), "UA-FIG-001");
        assert_eq!(format!("{}", UaErrorCode::TableMissingHeaders), "UA-TBL-001");
    }

    #[test]
    fn test_language_tag_validation() {
        assert!(is_valid_language_tag("en"));
        assert!(is_valid_language_tag("en-US"));
        assert!(is_valid_language_tag("zh-Hans"));
        assert!(is_valid_language_tag("de-AT"));
        assert!(!is_valid_language_tag(""));
        assert!(!is_valid_language_tag("e")); // Too short
        assert!(!is_valid_language_tag("english")); // Too long for primary
    }

    #[test]
    fn test_validator_builder() {
        let validator = PdfUaValidator::new()
            .check_heading_sequence(false)
            .check_color_contrast(true)
            .allow_custom_types(vec!["MyHeading".to_string()]);

        assert!(!validator.check_heading_sequence);
        assert!(validator.check_color_contrast);
        assert!(validator
            .allowed_custom_types
            .contains(&"MyHeading".to_string()));
    }
}
