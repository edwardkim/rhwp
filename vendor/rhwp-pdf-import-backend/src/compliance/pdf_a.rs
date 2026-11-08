//! PDF/A validator implementation.
//!
//! This module provides the main validator that coordinates all PDF/A compliance checks.

use super::types::{PdfALevel, ValidationResult};
use super::validators;
use crate::document::PdfDocument;
use crate::error::Result;

/// PDF/A compliance validator.
///
/// This validator checks PDF documents against PDF/A standards
/// (ISO 19005-1, 19005-2, 19005-3) and reports any violations.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::api::Pdf;
/// use pdf_oxide::compliance::{PdfAValidator, PdfALevel};
///
/// let pdf = Pdf::open("document.pdf")?;
/// let validator = PdfAValidator::new();
/// let result = validator.validate(pdf.document(), PdfALevel::A2b)?;
///
/// if result.is_compliant {
///     println!("Document is PDF/A-2b compliant");
/// } else {
///     for error in &result.errors {
///         println!("Violation: {}", error);
///     }
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct PdfAValidator {
    /// Whether to stop on first error
    stop_on_first_error: bool,
    /// Whether to include warnings in validation
    include_warnings: bool,
}

impl PdfAValidator {
    /// Create a new PDF/A validator with default settings.
    pub fn new() -> Self {
        Self {
            stop_on_first_error: false,
            include_warnings: true,
        }
    }

    /// Configure whether to stop validation on the first error.
    pub fn stop_on_first_error(mut self, stop: bool) -> Self {
        self.stop_on_first_error = stop;
        self
    }

    /// Configure whether to include warnings in the validation result.
    pub fn include_warnings(mut self, include: bool) -> Self {
        self.include_warnings = include;
        self
    }

    /// Validate a PDF document against the specified PDF/A level.
    ///
    /// # Arguments
    ///
    /// * `document` - The PDF document to validate
    /// * `level` - The PDF/A conformance level to validate against
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing compliance status, errors, and warnings.
    pub fn validate(
        &self,
        document: &mut PdfDocument,
        level: PdfALevel,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new(level);

        // Macro to run a validator and check for early return
        macro_rules! run_validator {
            ($validator:expr) => {
                $validator(document, level, &mut result)?;
                if self.should_stop(&result) {
                    return Ok(self.finalize_result(result));
                }
            };
        }

        // Run all validators in order
        run_validator!(validators::validate_xmp_metadata);
        run_validator!(validators::validate_fonts);
        run_validator!(validators::validate_colors);
        run_validator!(validators::validate_encryption);
        run_validator!(validators::validate_transparency);
        run_validator!(validators::validate_structure);
        run_validator!(validators::validate_javascript);
        run_validator!(validators::validate_embedded_files);

        // Last validator doesn't need early return check
        validators::validate_annotations(document, level, &mut result)?;

        Ok(self.finalize_result(result))
    }

    /// Check a specific aspect of PDF/A compliance.
    ///
    /// This allows checking individual requirements without running
    /// the full validation suite.
    pub fn check_fonts(
        &self,
        document: &mut PdfDocument,
        level: PdfALevel,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new(level);
        validators::validate_fonts(document, level, &mut result)?;
        Ok(self.finalize_result(result))
    }

    /// Check XMP metadata compliance.
    pub fn check_metadata(
        &self,
        document: &mut PdfDocument,
        level: PdfALevel,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new(level);
        validators::validate_xmp_metadata(document, level, &mut result)?;
        Ok(self.finalize_result(result))
    }

    /// Check color space compliance.
    pub fn check_colors(
        &self,
        document: &mut PdfDocument,
        level: PdfALevel,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new(level);
        validators::validate_colors(document, level, &mut result)?;
        Ok(self.finalize_result(result))
    }

    /// Check transparency usage (relevant for PDF/A-1).
    pub fn check_transparency(
        &self,
        document: &mut PdfDocument,
        level: PdfALevel,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new(level);
        validators::validate_transparency(document, level, &mut result)?;
        Ok(self.finalize_result(result))
    }

    /// Check document structure (relevant for level A conformance).
    pub fn check_structure(
        &self,
        document: &mut PdfDocument,
        level: PdfALevel,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new(level);
        validators::validate_structure(document, level, &mut result)?;
        Ok(self.finalize_result(result))
    }

    /// Determine if validation should stop based on current result.
    fn should_stop(&self, result: &ValidationResult) -> bool {
        self.stop_on_first_error && result.has_errors()
    }

    /// Finalize the validation result.
    fn finalize_result(&self, mut result: ValidationResult) -> ValidationResult {
        // Set compliance status
        result.is_compliant = !result.has_errors();

        // Remove warnings if not requested
        if !self.include_warnings {
            result.warnings.clear();
        }

        result
    }
}

/// Quick validation function for common use cases.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::compliance::{validate_pdf_a, PdfALevel};
///
/// let result = validate_pdf_a(&mut document, PdfALevel::A2b)?;
/// println!("Compliant: {}", result.is_compliant);
/// ```
pub fn validate_pdf_a(document: &mut PdfDocument, level: PdfALevel) -> Result<ValidationResult> {
    PdfAValidator::new().validate(document, level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = PdfAValidator::new();
        assert!(!validator.stop_on_first_error);
        assert!(validator.include_warnings);
    }

    #[test]
    fn test_validator_builder() {
        let validator = PdfAValidator::new()
            .stop_on_first_error(true)
            .include_warnings(false);

        assert!(validator.stop_on_first_error);
        assert!(!validator.include_warnings);
    }

    #[test]
    fn test_validation_result_finalization() {
        let validator = PdfAValidator::new();
        let result = ValidationResult::new(PdfALevel::A2b);
        let finalized = validator.finalize_result(result);

        // No errors means compliant
        assert!(finalized.is_compliant);
    }

    #[test]
    fn test_validation_result_finalization_without_warnings() {
        let validator = PdfAValidator::new().include_warnings(false);
        let mut result = ValidationResult::new(PdfALevel::A2b);
        result.add_warning(super::super::types::ComplianceWarning::new(
            super::super::types::WarningCode::MissingRecommendedMetadata,
            "Test warning",
        ));

        let finalized = validator.finalize_result(result);
        assert!(finalized.warnings.is_empty());
    }
}
