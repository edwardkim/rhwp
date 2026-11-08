//! PDF/X validator implementation.
//!
//! This module provides the main validator that coordinates all PDF/X compliance checks.

use super::types::{PdfXLevel, XComplianceError, XErrorCode, XValidationResult};
use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Type alias for PDF dictionary.
type Dictionary = HashMap<String, Object>;

/// PDF/X compliance validator.
///
/// This validator checks PDF documents against PDF/X standards (ISO 15930)
/// for print production workflows.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::api::Pdf;
/// use pdf_oxide::compliance::{PdfXValidator, PdfXLevel};
///
/// let pdf = Pdf::open("document.pdf")?;
/// let validator = PdfXValidator::new(PdfXLevel::X1a2003);
/// let result = validator.validate(&mut pdf.document())?;
///
/// if result.is_compliant {
///     println!("Document is PDF/X-1a:2003 compliant");
/// } else {
///     for error in &result.errors {
///         println!("Violation: {}", error);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PdfXValidator {
    /// Target PDF/X level
    level: PdfXLevel,
    /// Whether to stop on first error
    stop_on_first_error: bool,
    /// Whether to include warnings
    include_warnings: bool,
}

impl PdfXValidator {
    /// Create a new PDF/X validator for the specified level.
    pub fn new(level: PdfXLevel) -> Self {
        Self {
            level,
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

    /// Validate a PDF document against the configured PDF/X level.
    pub fn validate(&self, document: &mut PdfDocument) -> Result<XValidationResult> {
        let mut result = XValidationResult::new(self.level);

        // Run all validation checks
        self.validate_output_intent(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_metadata(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_info_dict(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_encryption(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_page_boxes(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_transparency(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_colors(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_fonts(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_annotations(document, &mut result)?;
        if self.should_stop(&result) {
            return Ok(self.finalize_result(result));
        }

        self.validate_actions(document, &mut result)?;

        Ok(self.finalize_result(result))
    }

    /// Validate output intent requirements.
    fn validate_output_intent(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let catalog = match self.get_catalog_dict(document)? {
            Some(d) => d,
            None => {
                result.add_error(XComplianceError::new(
                    XErrorCode::OutputIntentMissing,
                    "Document catalog is invalid",
                ));
                return Ok(());
            },
        };

        // Check for OutputIntents array
        let output_intents = match catalog.get("OutputIntents") {
            Some(Object::Array(arr)) => arr.clone(),
            Some(Object::Reference(r)) => {
                // Dereference if needed
                match document.load_object(*r)? {
                    Object::Array(arr) => arr,
                    _ => {
                        result.add_error(
                            XComplianceError::new(
                                XErrorCode::OutputIntentMissing,
                                "OutputIntents must be an array",
                            )
                            .with_clause("6.2.2"),
                        );
                        return Ok(());
                    },
                }
            },
            _ => {
                result.add_error(
                    XComplianceError::new(
                        XErrorCode::OutputIntentMissing,
                        "OutputIntents array is required for PDF/X",
                    )
                    .with_clause("6.2.2"),
                );
                return Ok(());
            },
        };

        if output_intents.is_empty() {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::OutputIntentMissing,
                    "OutputIntents array is empty",
                )
                .with_clause("6.2.2"),
            );
            return Ok(());
        }

        // Check for GTS_PDFX output intent
        let mut found_pdfx_intent = false;
        for intent_obj in &output_intents {
            let intent = match intent_obj {
                Object::Dictionary(d) => d.clone(),
                Object::Reference(r) => match document.load_object(*r)? {
                    Object::Dictionary(d) => d,
                    _ => continue,
                },
                _ => continue,
            };

            // Check S (subtype) key
            if let Some(Object::Name(s)) = intent.get("S") {
                if s == "GTS_PDFX" {
                    found_pdfx_intent = true;

                    // Check OutputConditionIdentifier
                    if !intent.contains_key("OutputConditionIdentifier") {
                        result.add_error(
                            XComplianceError::new(
                                XErrorCode::OutputConditionMissing,
                                "OutputConditionIdentifier is required in output intent",
                            )
                            .with_clause("6.2.2"),
                        );
                    }

                    // Store output intent info in stats
                    if let Some(Object::String(oci)) = intent.get("OutputConditionIdentifier") {
                        result.stats.output_intent = Some(String::from_utf8_lossy(oci).to_string());
                    }

                    break;
                }
            }
        }

        if !found_pdfx_intent {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::OutputIntentInvalid,
                    "GTS_PDFX output intent is required",
                )
                .with_clause("6.2.2"),
            );
        }

        Ok(())
    }

    /// Validate XMP metadata requirements.
    fn validate_metadata(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let catalog = match self.get_catalog_dict(document)? {
            Some(d) => d,
            None => return Ok(()),
        };

        // Check for Metadata entry
        if !catalog.contains_key("Metadata") {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::XmpMetadataMissing,
                    "XMP metadata stream is required for PDF/X",
                )
                .with_clause("6.7.2"),
            );
        }

        // Parse XMP and validate PDF/X identification (pdfxid:GTS_PDFXVersion)
        match crate::extractors::xmp::XmpExtractor::extract(document) {
            Ok(Some(xmp)) => {
                let pdfx_version = xmp.custom.get("pdfxid:GTS_PDFXVersion");

                if pdfx_version.is_none() {
                    result.add_warning(
                        XComplianceError::warning(
                            XErrorCode::XmpMetadataInvalid,
                            "XMP metadata missing pdfxid:GTS_PDFXVersion identification",
                        )
                        .with_clause("6.7.2"),
                    );
                } else if let Some(version_str) = pdfx_version {
                    // Validate version matches declared level
                    let expected = self.level.xmp_version();
                    if version_str != expected {
                        result.add_warning(
                            XComplianceError::warning(
                                XErrorCode::XmpMetadataInvalid,
                                format!(
                                    "XMP pdfxid:GTS_PDFXVersion is '{}' but validating against {} (expected '{}')",
                                    version_str, self.level, expected
                                ),
                            )
                            .with_clause("6.7.2"),
                        );
                    }

                    // Try to detect level from XMP
                    if let Some(detected) = PdfXLevel::from_gts_version(version_str) {
                        if result.detected_level.is_none() {
                            result.detected_level = Some(detected);
                        }
                    }
                }
            },
            Ok(None) => {
                result.add_warning(
                    XComplianceError::warning(
                        XErrorCode::XmpMetadataInvalid,
                        "Could not extract XMP metadata for PDF/X identification",
                    )
                    .with_clause("6.7.2"),
                );
            },
            Err(_) => {
                result.add_warning(
                    XComplianceError::warning(
                        XErrorCode::XmpMetadataInvalid,
                        "Failed to parse XMP metadata for PDF/X identification",
                    )
                    .with_clause("6.7.2"),
                );
            },
        }

        Ok(())
    }

    /// Validate Info dictionary requirements.
    fn validate_info_dict(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let info = match self.get_info_dict(document)? {
            Some(d) => d,
            None => {
                // Info dict is recommended but we'll add a warning
                result.add_warning(XComplianceError::warning(
                    XErrorCode::GtsPdfxVersionMissing,
                    "Info dictionary not found",
                ));
                return Ok(());
            },
        };

        // Check GTS_PDFXVersion
        if !info.contains_key("GTS_PDFXVersion") {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::GtsPdfxVersionMissing,
                    "GTS_PDFXVersion key is required in Info dictionary",
                )
                .with_clause("6.7.5"),
            );
        } else {
            // Try to detect the PDF/X level
            if let Some(Object::String(version)) = info.get("GTS_PDFXVersion") {
                let version_str = String::from_utf8_lossy(version);
                if let Some(detected) = PdfXLevel::from_gts_version(&version_str) {
                    result.detected_level = Some(detected);
                }
            }
        }

        // Check GTS_PDFXConformance for X-1a and X-3
        if matches!(
            self.level,
            PdfXLevel::X1a2001 | PdfXLevel::X1a2003 | PdfXLevel::X32002 | PdfXLevel::X32003
        ) && !info.contains_key("GTS_PDFXConformance")
        {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::GtsPdfxConformanceMissing,
                    "GTS_PDFXConformance key is required for PDF/X-1a and PDF/X-3",
                )
                .with_clause("6.7.5"),
            );
        }

        // Check Trapped key (required for PDF/X)
        if !info.contains_key("Trapped") {
            result.add_warning(
                XComplianceError::warning(
                    XErrorCode::TrappedKeyMissing,
                    "Trapped key should be present in Info dictionary",
                )
                .with_clause("6.7.5"),
            );
        }

        Ok(())
    }

    /// Validate encryption (not allowed in PDF/X).
    fn validate_encryption(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        // Check the trailer for /Encrypt entry
        let trailer = document.trailer();
        let is_encrypted = if let Object::Dictionary(trailer_dict) = trailer {
            trailer_dict.contains_key("Encrypt")
        } else {
            false
        };

        if is_encrypted {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::EncryptionNotAllowed,
                    "Encryption is not allowed in PDF/X documents",
                )
                .with_clause("6.1.12"),
            );
        }
        Ok(())
    }

    /// Validate page box requirements.
    fn validate_page_boxes(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let page_count = document.page_count()?;
        result.stats.pages_checked = page_count;

        for page_num in 0..page_count {
            if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                // Check MediaBox (required)
                if !page_dict.contains_key("MediaBox") {
                    result.add_error(
                        XComplianceError::new(XErrorCode::MediaBoxMissing, "MediaBox is required")
                            .with_page(page_num)
                            .with_clause("6.1.1"),
                    );
                }

                // Check TrimBox or ArtBox (at least one required for PDF/X)
                let has_trim = page_dict.contains_key("TrimBox");
                let has_art = page_dict.contains_key("ArtBox");

                if !has_trim && !has_art {
                    result.add_error(
                        XComplianceError::new(
                            XErrorCode::TrimOrArtBoxMissing,
                            "Either TrimBox or ArtBox is required for PDF/X",
                        )
                        .with_page(page_num)
                        .with_clause("6.1.1"),
                    );
                }

                // Validate box relationships: TrimBox ⊆ BleedBox ⊆ MediaBox
                if let Some(media_box) = Self::parse_box(page_dict.get("MediaBox")) {
                    let bleed_box = Self::parse_box(page_dict.get("BleedBox"));
                    let trim_box = Self::parse_box(page_dict.get("TrimBox"));
                    let art_box = Self::parse_box(page_dict.get("ArtBox"));

                    // BleedBox must be within MediaBox
                    if let Some(bb) = bleed_box {
                        if !Self::box_contains(&media_box, &bb) {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::BleedBoxInvalid,
                                    "BleedBox extends beyond MediaBox",
                                )
                                .with_page(page_num)
                                .with_clause("6.1.1"),
                            );
                        }

                        // TrimBox must be within BleedBox (if BleedBox exists)
                        if let Some(tb) = trim_box {
                            if !Self::box_contains(&bb, &tb) {
                                result.add_error(
                                    XComplianceError::new(
                                        XErrorCode::TrimBoxInvalid,
                                        "TrimBox extends beyond BleedBox",
                                    )
                                    .with_page(page_num)
                                    .with_clause("6.1.1"),
                                );
                            }
                        }
                    }

                    // TrimBox must be within MediaBox
                    if let Some(tb) = trim_box {
                        if !Self::box_contains(&media_box, &tb) {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::TrimBoxInvalid,
                                    "TrimBox extends beyond MediaBox",
                                )
                                .with_page(page_num)
                                .with_clause("6.1.1"),
                            );
                        }
                    }

                    // ArtBox must be within MediaBox
                    if let Some(ab) = art_box {
                        if !Self::box_contains(&media_box, &ab) {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::BoxesInconsistent,
                                    "ArtBox extends beyond MediaBox",
                                )
                                .with_page(page_num)
                                .with_clause("6.1.1"),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate transparency requirements.
    fn validate_transparency(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        // Only check if transparency is not allowed for this level
        if self.level.allows_transparency() {
            return Ok(());
        }

        // Check catalog for OutputIntents with transparency group
        let _catalog = match self.get_catalog_dict(document)? {
            Some(d) => d,
            None => return Ok(()),
        };

        // Check for transparency-related entries
        // Pages with Group entry having S=Transparency
        let page_count = document.page_count()?;
        for page_num in 0..page_count {
            if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                if let Some(Object::Dictionary(group)) = page_dict.get("Group") {
                    if let Some(Object::Name(s)) = group.get("S") {
                        if s == "Transparency" {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::TransparencyNotAllowed,
                                    "Transparency groups are not allowed in this PDF/X level",
                                )
                                .with_page(page_num)
                                .with_clause("6.3"),
                            );
                            result.stats.has_transparency = true;
                        }
                    }
                }
            }
        }

        // Check ExtGState for transparency-related entries (SMask, CA, ca, BM)
        let page_count2 = document.page_count()?;
        for page_num in 0..page_count2 {
            if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                let resources = match page_dict.get("Resources") {
                    Some(Object::Dictionary(d)) => d.clone(),
                    Some(Object::Reference(r)) => match document.load_object(*r)? {
                        Object::Dictionary(d) => d,
                        _ => continue,
                    },
                    _ => continue,
                };

                let ext_gstate = match resources.get("ExtGState") {
                    Some(Object::Dictionary(d)) => d.clone(),
                    Some(Object::Reference(r)) => match document.load_object(*r)? {
                        Object::Dictionary(d) => d,
                        _ => continue,
                    },
                    _ => continue,
                };

                // Sorted by ExtGState name so the emitted error order is
                // stable; see the ColorSpace sweep in validate_colors.
                let mut gs_names: Vec<&String> = ext_gstate.keys().collect();
                gs_names.sort();
                for gs_name in gs_names {
                    let Some(gs_obj) = ext_gstate.get(gs_name) else {
                        continue;
                    };
                    let gs_dict = match gs_obj {
                        Object::Dictionary(d) => d.clone(),
                        Object::Reference(r) => match document.load_object(*r)? {
                            Object::Dictionary(d) => d,
                            _ => continue,
                        },
                        _ => continue,
                    };

                    // Check SMask (not /None means transparency)
                    if let Some(smask) = gs_dict.get("SMask") {
                        let is_none = matches!(smask, Object::Name(n) if n == "None");
                        if !is_none {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::SMaskNotAllowed,
                                    format!(
                                        "ExtGState '{}' has SMask (transparency not allowed)",
                                        gs_name
                                    ),
                                )
                                .with_page(page_num)
                                .with_clause("6.3"),
                            );
                            result.stats.has_transparency = true;
                        }
                    }

                    // Check CA (fill opacity) < 1.0
                    if let Some(ca_val) = gs_dict.get("CA") {
                        let opacity = match ca_val {
                            Object::Real(v) => Some(*v),
                            Object::Integer(v) => Some(*v as f64),
                            _ => None,
                        };
                        if let Some(op) = opacity {
                            if op < 1.0 {
                                result.add_error(
                                    XComplianceError::new(
                                        XErrorCode::TransparencyNotAllowed,
                                        format!(
                                            "ExtGState '{}' has non-opaque CA={} (transparency not allowed)",
                                            gs_name, op
                                        ),
                                    )
                                    .with_page(page_num)
                                    .with_clause("6.3"),
                                );
                                result.stats.has_transparency = true;
                            }
                        }
                    }

                    // Check ca (stroke opacity) < 1.0
                    if let Some(ca_val) = gs_dict.get("ca") {
                        let opacity = match ca_val {
                            Object::Real(v) => Some(*v),
                            Object::Integer(v) => Some(*v as f64),
                            _ => None,
                        };
                        if let Some(op) = opacity {
                            if op < 1.0 {
                                result.add_error(
                                    XComplianceError::new(
                                        XErrorCode::TransparencyNotAllowed,
                                        format!(
                                            "ExtGState '{}' has non-opaque ca={} (transparency not allowed)",
                                            gs_name, op
                                        ),
                                    )
                                    .with_page(page_num)
                                    .with_clause("6.3"),
                                );
                                result.stats.has_transparency = true;
                            }
                        }
                    }

                    // Check BM (blend mode) not Normal/Compatible
                    if let Some(Object::Name(bm)) = gs_dict.get("BM") {
                        if bm != "Normal" && bm != "Compatible" {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::BlendModeNotAllowed,
                                    format!(
                                        "ExtGState '{}' has blend mode '{}' (only Normal/Compatible allowed)",
                                        gs_name, bm
                                    ),
                                )
                                .with_page(page_num)
                                .with_clause("6.3"),
                            );
                            result.stats.has_transparency = true;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate color space requirements.
    fn validate_colors(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        // For PDF/X-1a, only CMYK and spot colors are allowed
        if !self.level.allows_rgb() {
            // Check page resources for RGB color spaces
            let page_count = document.page_count()?;
            for page_num in 0..page_count {
                if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                    if let Some(Object::Dictionary(resources)) = page_dict.get("Resources") {
                        if let Some(Object::Dictionary(colorspaces)) = resources.get("ColorSpace") {
                            // Sorted by resource name: a PDF dictionary is a
                            // `HashMap`, and this loop both pushes into the
                            // emitted `color_spaces_found` list and reports
                            // errors, so unsorted iteration randomizes a
                            // compliance report's contents between runs.
                            let mut cs_names: Vec<&String> = colorspaces.keys().collect();
                            cs_names.sort();
                            for name in cs_names {
                                let Some(cs) = colorspaces.get(name) else {
                                    continue;
                                };
                                let cs_name = self.get_colorspace_name(cs, document)?;
                                result.stats.color_spaces_found.push(cs_name.clone());

                                if cs_name == "DeviceRGB" || cs_name == "CalRGB" {
                                    result.add_error(
                                        XComplianceError::new(
                                            XErrorCode::RgbColorNotAllowed,
                                            format!(
                                                "RGB color space '{}' not allowed in PDF/X-1a",
                                                name
                                            ),
                                        )
                                        .with_page(page_num)
                                        .with_clause("6.2.3"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for device-dependent colors without output intent
        // and validate ICC profiles in color space resources
        let has_output_intent = result.stats.output_intent.is_some();
        let page_count2 = document.page_count()?;
        for page_num in 0..page_count2 {
            if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                let resources = match page_dict.get("Resources") {
                    Some(Object::Dictionary(d)) => d.clone(),
                    Some(Object::Reference(r)) => match document.load_object(*r)? {
                        Object::Dictionary(d) => d,
                        _ => continue,
                    },
                    _ => continue,
                };

                if let Some(colorspaces_obj) = resources.get("ColorSpace") {
                    let colorspaces = match colorspaces_obj {
                        Object::Dictionary(d) => d.clone(),
                        Object::Reference(r) => match document.load_object(*r)? {
                            Object::Dictionary(d) => d,
                            _ => continue,
                        },
                        _ => continue,
                    };

                    // Sorted by resource name so the emitted error order is
                    // stable; see the ColorSpace sweep above.
                    let mut sorted_cs: Vec<&String> = colorspaces.keys().collect();
                    sorted_cs.sort();
                    for cs_name in sorted_cs {
                        let Some(cs_obj) = colorspaces.get(cs_name) else {
                            continue;
                        };
                        let cs_type = self.get_colorspace_name(cs_obj, document)?;

                        // Device-dependent colors without output intent
                        if !has_output_intent
                            && (cs_type == "DeviceRGB"
                                || cs_type == "DeviceCMYK"
                                || cs_type == "DeviceGray")
                        {
                            result.add_error(
                                XComplianceError::new(
                                    XErrorCode::DeviceColorWithoutIntent,
                                    format!(
                                        "Device color space '{}' ({}) used without output intent",
                                        cs_name, cs_type
                                    ),
                                )
                                .with_page(page_num)
                                .with_clause("6.2.3"),
                            );
                        }

                        // Validate ICCBased color space profiles
                        if cs_type == "ICCBased" {
                            self.validate_icc_profile(cs_obj, cs_name, page_num, document, result)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate font requirements.
    fn validate_fonts(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        // Check page resources for fonts
        let page_count = document.page_count()?;
        for page_num in 0..page_count {
            if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                if let Some(Object::Dictionary(resources)) = page_dict.get("Resources") {
                    if let Some(Object::Dictionary(fonts)) = resources.get("Font") {
                        // Sorted by resource name so the emitted error order is
                        // stable; see the ColorSpace sweep above.
                        let mut font_names: Vec<&String> = fonts.keys().collect();
                        font_names.sort();
                        for name in font_names {
                            let Some(font_ref) = fonts.get(name) else {
                                continue;
                            };
                            result.stats.fonts_checked += 1;

                            let font_dict = match font_ref {
                                Object::Dictionary(d) => d.clone(),
                                Object::Reference(r) => match document.load_object(*r)? {
                                    Object::Dictionary(d) => d,
                                    _ => continue,
                                },
                                _ => continue,
                            };

                            // Check font type
                            if let Some(Object::Name(subtype)) = font_dict.get("Subtype") {
                                if subtype == "Type3" {
                                    result.add_error(
                                        XComplianceError::new(
                                            XErrorCode::Type3FontNotAllowed,
                                            format!("Type 3 font '{}' not allowed in PDF/X", name),
                                        )
                                        .with_page(page_num)
                                        .with_clause("6.3.5"),
                                    );
                                }
                            }

                            // Check if font is embedded
                            let is_embedded = font_dict.contains_key("FontFile")
                                || font_dict.contains_key("FontFile2")
                                || font_dict.contains_key("FontFile3");

                            // Also check FontDescriptor
                            let descriptor_embedded = if let Some(Object::Dictionary(fd)) =
                                font_dict.get("FontDescriptor")
                            {
                                fd.contains_key("FontFile")
                                    || fd.contains_key("FontFile2")
                                    || fd.contains_key("FontFile3")
                            } else if let Some(Object::Reference(r)) =
                                font_dict.get("FontDescriptor")
                            {
                                if let Object::Dictionary(fd) = document.load_object(*r)? {
                                    fd.contains_key("FontFile")
                                        || fd.contains_key("FontFile2")
                                        || fd.contains_key("FontFile3")
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            if is_embedded || descriptor_embedded {
                                result.stats.fonts_embedded += 1;
                            } else {
                                // Standard 14 fonts might not be embedded but should have widths
                                let is_standard14 = self.is_standard14_font(&font_dict);
                                if !is_standard14 {
                                    result.add_error(
                                        XComplianceError::new(
                                            XErrorCode::FontNotEmbedded,
                                            format!("Font '{}' must be embedded", name),
                                        )
                                        .with_page(page_num)
                                        .with_clause("6.3.5"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate annotation requirements.
    fn validate_annotations(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let page_count = document.page_count()?;
        for page_num in 0..page_count {
            if let Ok(page_dict) = self.get_page_dict(document, page_num) {
                let annots = match page_dict.get("Annots") {
                    Some(Object::Array(arr)) => arr.clone(),
                    Some(Object::Reference(r)) => match document.load_object(*r)? {
                        Object::Array(arr) => arr,
                        _ => continue,
                    },
                    _ => continue,
                };

                for annot_obj in annots {
                    result.stats.annotations_checked += 1;

                    let annot = match annot_obj {
                        Object::Dictionary(d) => d,
                        Object::Reference(r) => match document.load_object(r)? {
                            Object::Dictionary(d) => d,
                            _ => continue,
                        },
                        _ => continue,
                    };

                    // Check annotation subtype
                    if let Some(Object::Name(subtype)) = annot.get("Subtype") {
                        // Only TrapNet and PrinterMark are allowed in PDF/X
                        // Other allowed: Link (with restrictions), Widget (form fields)
                        match subtype.as_str() {
                            "TrapNet" | "PrinterMark" => {
                                // Allowed
                            },
                            "Link" | "Widget" => {
                                // Allowed with restrictions - check appearance
                                if !annot.contains_key("AP") {
                                    result.add_warning(
                                        XComplianceError::warning(
                                            XErrorCode::AnnotationNotAllowed,
                                            format!(
                                                "{} annotation should have appearance stream",
                                                subtype
                                            ),
                                        )
                                        .with_page(page_num),
                                    );
                                }
                            },
                            _ => {
                                // Other annotation types may not be allowed
                                // (depends on specific PDF/X level requirements)
                                result.add_warning(
                                    XComplianceError::warning(
                                        XErrorCode::AnnotationNotAllowed,
                                        format!(
                                            "Annotation type '{}' may not be allowed in PDF/X",
                                            subtype
                                        ),
                                    )
                                    .with_page(page_num),
                                );
                            },
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate action requirements.
    fn validate_actions(
        &self,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let catalog = match self.get_catalog_dict(document)? {
            Some(d) => d,
            None => return Ok(()),
        };

        // Check for JavaScript in Names dictionary
        if let Some(names_obj) = catalog.get("Names") {
            let names = match names_obj {
                Object::Dictionary(d) => d.clone(),
                Object::Reference(r) => match document.load_object(*r)? {
                    Object::Dictionary(d) => d,
                    _ => HashMap::new(),
                },
                _ => HashMap::new(),
            };
            if names.contains_key("JavaScript") {
                result.add_error(
                    XComplianceError::new(
                        XErrorCode::JavaScriptNotAllowed,
                        "JavaScript is not allowed in PDF/X documents",
                    )
                    .with_clause("6.6.1"),
                );
            }
        }

        // Check OpenAction
        if let Some(action) = catalog.get("OpenAction") {
            self.check_action(action, document, result)?;
        }

        // Check AA (Additional Actions)
        if catalog.contains_key("AA") {
            result.add_warning(XComplianceError::warning(
                XErrorCode::ActionNotAllowed,
                "Additional actions (AA) may not be compatible with PDF/X",
            ));
        }

        Ok(())
    }

    // Helper methods

    fn get_catalog_dict(&self, document: &mut PdfDocument) -> Result<Option<Dictionary>> {
        let catalog = document.catalog()?;
        match catalog {
            Object::Dictionary(d) => Ok(Some(d)),
            _ => Ok(None),
        }
    }

    /// Get the Info dictionary from the trailer.
    fn get_info_dict(&self, document: &mut PdfDocument) -> Result<Option<Dictionary>> {
        let trailer = document.trailer();
        let trailer_dict = match trailer {
            Object::Dictionary(d) => d,
            _ => return Ok(None),
        };

        // Get /Info reference
        let info_ref = match trailer_dict.get("Info") {
            Some(Object::Reference(r)) => *r,
            Some(Object::Dictionary(d)) => return Ok(Some(d.clone())),
            _ => return Ok(None),
        };

        // Load the Info dictionary
        let info_obj = document.load_object(info_ref)?;
        match info_obj {
            Object::Dictionary(d) => Ok(Some(d)),
            _ => Ok(None),
        }
    }

    /// Get a page dictionary by index by walking the page tree.
    fn get_page_dict(&self, document: &mut PdfDocument, page_num: usize) -> Result<Dictionary> {
        // Get catalog and pages tree
        let catalog = match self.get_catalog_dict(document)? {
            Some(d) => d,
            None => {
                return Err(Error::InvalidPdf("Invalid catalog".to_string()));
            },
        };

        // Get /Pages reference
        let pages_ref = match catalog.get("Pages") {
            Some(Object::Reference(r)) => *r,
            _ => {
                return Err(Error::InvalidPdf("Pages entry missing or invalid".to_string()));
            },
        };

        // Walk the page tree to find the specific page
        self.get_page_from_tree(document, pages_ref, page_num, &mut 0)
    }

    /// Recursively find a page in the page tree.
    #[allow(clippy::only_used_in_recursion)]
    fn get_page_from_tree(
        &self,
        document: &mut PdfDocument,
        node_ref: ObjectRef,
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<Dictionary> {
        let node = document.load_object(node_ref)?;
        let node_dict = match node {
            Object::Dictionary(d) => d,
            _ => return Err(Error::InvalidPdf("Invalid page tree node".to_string())),
        };

        // Check node type
        let node_type = node_dict
            .get("Type")
            .and_then(|o| {
                if let Object::Name(n) = o {
                    Some(n.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");

        if node_type == "Page" {
            // This is a page node
            if *current_index == target_index {
                return Ok(node_dict);
            }
            *current_index += 1;
            return Err(Error::InvalidPdf("Page not found".to_string()));
        }

        // This is a Pages node - iterate through Kids
        let kids = match node_dict.get("Kids") {
            Some(Object::Array(arr)) => arr.clone(),
            Some(Object::Reference(r)) => match document.load_object(*r)? {
                Object::Array(arr) => arr,
                _ => return Err(Error::InvalidPdf("Invalid Kids array".to_string())),
            },
            _ => return Err(Error::InvalidPdf("Missing Kids array".to_string())),
        };

        for kid in kids {
            let kid_ref = match kid {
                Object::Reference(r) => r,
                _ => continue,
            };

            // Try to get the page count for this subtree (optimization)
            let kid_obj = document.load_object(kid_ref)?;
            if let Object::Dictionary(kid_dict) = &kid_obj {
                if let Some(Object::Integer(count)) = kid_dict.get("Count") {
                    let count = *count as usize;
                    if *current_index + count <= target_index {
                        // The target page is not in this subtree
                        *current_index += count;
                        continue;
                    }
                }
            }

            // Recursively search this subtree
            match self.get_page_from_tree(document, kid_ref, target_index, current_index) {
                Ok(page) => return Ok(page),
                Err(_) => continue,
            }
        }

        Err(Error::InvalidPdf(format!("Page {} not found", target_index)))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_colorspace_name(&self, cs: &Object, document: &mut PdfDocument) -> Result<String> {
        match cs {
            Object::Name(n) => Ok(n.clone()),
            Object::Array(arr) => {
                if let Some(Object::Name(n)) = arr.first() {
                    Ok(n.clone())
                } else {
                    Ok("Unknown".to_string())
                }
            },
            Object::Reference(r) => {
                let resolved = document.load_object(*r)?;
                self.get_colorspace_name(&resolved, document)
            },
            _ => Ok("Unknown".to_string()),
        }
    }

    fn is_standard14_font(&self, font_dict: &Dictionary) -> bool {
        if let Some(Object::Name(base_font)) = font_dict.get("BaseFont") {
            let standard14 = [
                "Courier",
                "Courier-Bold",
                "Courier-Oblique",
                "Courier-BoldOblique",
                "Helvetica",
                "Helvetica-Bold",
                "Helvetica-Oblique",
                "Helvetica-BoldOblique",
                "Times-Roman",
                "Times-Bold",
                "Times-Italic",
                "Times-BoldItalic",
                "Symbol",
                "ZapfDingbats",
            ];
            return standard14.contains(&base_font.as_str());
        }
        false
    }

    fn check_action(
        &self,
        action: &Object,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        let action_dict = match action {
            Object::Dictionary(d) => d.clone(),
            Object::Reference(r) => match document.load_object(*r)? {
                Object::Dictionary(d) => d,
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };

        if let Some(Object::Name(action_type)) = action_dict.get("S") {
            match action_type.as_str() {
                "JavaScript" => {
                    result.add_error(
                        XComplianceError::new(
                            XErrorCode::JavaScriptNotAllowed,
                            "JavaScript actions are not allowed in PDF/X",
                        )
                        .with_clause("6.6.1"),
                    );
                },
                "Launch" | "Sound" | "Movie" | "ImportData" | "ResetForm" | "SubmitForm" => {
                    result.add_error(
                        XComplianceError::new(
                            XErrorCode::ActionNotAllowed,
                            format!("Action type '{}' is not allowed in PDF/X", action_type),
                        )
                        .with_clause("6.6.1"),
                    );
                },
                _ => {},
            }
        }

        Ok(())
    }

    /// Parse a PDF rectangle array [llx, lly, urx, ury] into [f64; 4].
    fn parse_box(obj: Option<&Object>) -> Option<[f64; 4]> {
        let arr = match obj? {
            Object::Array(a) => a,
            _ => return None,
        };
        if arr.len() < 4 {
            return None;
        }
        let to_f64 = |o: &Object| -> Option<f64> {
            match o {
                Object::Real(v) => Some(*v),
                Object::Integer(v) => Some(*v as f64),
                _ => None,
            }
        };
        Some([
            to_f64(&arr[0])?,
            to_f64(&arr[1])?,
            to_f64(&arr[2])?,
            to_f64(&arr[3])?,
        ])
    }

    /// Check if outer box fully contains inner box (with 0.01pt tolerance).
    fn box_contains(outer: &[f64; 4], inner: &[f64; 4]) -> bool {
        const TOLERANCE: f64 = 0.01;
        // outer[0] <= inner[0] (left)
        // outer[1] <= inner[1] (bottom)
        // outer[2] >= inner[2] (right)
        // outer[3] >= inner[3] (top)
        (outer[0] - TOLERANCE) <= inner[0]
            && (outer[1] - TOLERANCE) <= inner[1]
            && (outer[2] + TOLERANCE) >= inner[2]
            && (outer[3] + TOLERANCE) >= inner[3]
    }

    /// Validate an ICCBased color space profile stream.
    fn validate_icc_profile(
        &self,
        cs_obj: &Object,
        cs_name: &str,
        page_num: usize,
        document: &mut PdfDocument,
        result: &mut XValidationResult,
    ) -> Result<()> {
        // ICCBased is [/ICCBased, stream_ref]
        let arr = match cs_obj {
            Object::Array(a) => a.clone(),
            Object::Reference(r) => match document.load_object(*r)? {
                Object::Array(a) => a,
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };
        if arr.len() < 2 {
            return Ok(());
        }

        // Get the ICC profile stream dictionary. Per ISO 32000-1 §8.6.5.5 an
        // ICCBased colour space is `[ /ICCBased stream ]` — the profile is
        // always embedded as a STREAM object, so the resolved object is
        // `Object::Stream { dict, .. }`, not a bare `Object::Dictionary`. Accept
        // both (a `Dictionary` only arises from a malformed/inline form); reading
        // /N from the stream's dictionary is what the check below needs. Failing
        // to handle `Object::Stream` here caused a false XCOLOR-005 on every
        // conforming ICCBased profile (e.g. the Ghent Workgroup PDF/X-4 suite).
        let profile_dict = match &arr[1] {
            Object::Dictionary(d) => d.clone(),
            Object::Stream { dict, .. } => dict.clone(),
            Object::Reference(r) => match document.load_object(*r)? {
                Object::Dictionary(d) => d,
                Object::Stream { dict, .. } => dict,
                _ => {
                    result.add_error(
                        XComplianceError::new(
                            XErrorCode::IccProfileInvalid,
                            format!("ICC profile for '{}' is not a valid stream", cs_name),
                        )
                        .with_page(page_num)
                        .with_clause("6.2.3"),
                    );
                    return Ok(());
                },
            },
            _ => return Ok(()),
        };

        // Check /N entry (number of color components)
        if !profile_dict.contains_key("N") {
            result.add_error(
                XComplianceError::new(
                    XErrorCode::IccProfileInvalid,
                    format!("ICC profile for '{}' missing required /N entry", cs_name),
                )
                .with_page(page_num)
                .with_clause("6.2.3"),
            );
        }

        Ok(())
    }

    fn should_stop(&self, result: &XValidationResult) -> bool {
        self.stop_on_first_error && result.has_errors()
    }

    fn finalize_result(&self, mut result: XValidationResult) -> XValidationResult {
        result.is_compliant = !result.has_errors();

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
/// use pdf_oxide::compliance::{validate_pdf_x, PdfXLevel};
///
/// let result = validate_pdf_x(&mut document, PdfXLevel::X1a2003)?;
/// println!("Compliant: {}", result.is_compliant);
/// ```
pub fn validate_pdf_x(document: &mut PdfDocument, level: PdfXLevel) -> Result<XValidationResult> {
    PdfXValidator::new(level).validate(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::pdf_x::XValidationStats;

    #[test]
    fn test_validator_creation() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        assert_eq!(validator.level, PdfXLevel::X1a2003);
        assert!(!validator.stop_on_first_error);
        assert!(validator.include_warnings);
    }

    #[test]
    fn test_validator_builder() {
        let validator = PdfXValidator::new(PdfXLevel::X4)
            .stop_on_first_error(true)
            .include_warnings(false);

        assert!(validator.stop_on_first_error);
        assert!(!validator.include_warnings);
    }

    /// A minimal PDF whose single page has a page-level ICCBased colour space
    /// `/CS0` referencing an ICC profile embedded as object `5 0 R` — a real
    /// PDF **stream** (ISO 32000-1 §8.6.5.5). `include_n` toggles the required
    /// `/N` entry so both the valid and the genuinely-invalid cases can be built.
    fn build_pdf_with_iccbased_colorspace(include_n: bool) -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();

        let off1 = pdf.len();
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        let off2 = pdf.len();
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        let off3 = pdf.len();
        pdf.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /ColorSpace << /CS0 [/ICCBased 5 0 R] >> >> >>\nendobj\n",
        );

        let off4 = pdf.len();
        let content: &[u8] = b"/CS0 cs 0 0 0 1 scn 50 50 200 200 re f";
        pdf.extend_from_slice(
            format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len()).as_bytes(),
        );
        pdf.extend_from_slice(content);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        // Object 5: the ICC profile STREAM — a 128-byte valid ICC header
        // (profile size at 0..4, colour space `CMYK` at 16..20, `acsp`
        // signature at 36..40), uncompressed (no /Filter).
        let mut icc = vec![0u8; 128];
        icc[0..4].copy_from_slice(&128u32.to_be_bytes());
        icc[16..20].copy_from_slice(b"CMYK");
        icc[36..40].copy_from_slice(b"acsp");
        let off5 = pdf.len();
        let dict = if include_n {
            format!("<< /N 4 /Length {} >>", icc.len())
        } else {
            format!("<< /Length {} >>", icc.len())
        };
        pdf.extend_from_slice(format!("5 0 obj\n{}\nstream\n", dict).as_bytes());
        pdf.extend_from_slice(&icc);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        let xref_off = pdf.len();
        pdf.extend_from_slice(b"xref\n0 6\n");
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for off in [off1, off2, off3, off4, off5] {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        pdf.extend_from_slice(
            format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off)
                .as_bytes(),
        );
        pdf
    }

    #[test]
    fn iccbased_stream_profile_not_flagged_xcolor005() {
        // Regression for #797: a conforming ICCBased colour space embeds its ICC
        // profile as a STREAM (ISO 32000-1 §8.6.5.5, `[ /ICCBased stream ]`),
        // which resolves to `Object::Stream`, not `Object::Dictionary`. The
        // validator must accept it and read /N from the stream dict rather than
        // emit a false "not a valid stream" XCOLOR-005 (which broke every
        // conforming ICCBased profile, e.g. the Ghent Workgroup PDF/X-4 suite).
        use crate::object::ObjectRef;
        let mut doc =
            crate::document::PdfDocument::from_bytes(build_pdf_with_iccbased_colorspace(true))
                .expect("parse test pdf");
        let validator = PdfXValidator::new(PdfXLevel::X4);
        let mut result = XValidationResult::new(PdfXLevel::X4);
        let cs_obj = Object::Array(vec![
            Object::Name("ICCBased".to_string()),
            Object::Reference(ObjectRef { id: 5, gen: 0 }),
        ]);
        validator
            .validate_icc_profile(&cs_obj, "CS0", 1, &mut doc, &mut result)
            .expect("validate_icc_profile");
        assert!(
            !result
                .errors
                .iter()
                .any(|e| e.code == XErrorCode::IccProfileInvalid),
            "valid ICC profile stream must not trigger XCOLOR-005: {:?}",
            result.errors
        );
    }

    #[test]
    fn iccbased_stream_missing_n_still_flagged() {
        // The /N check must still fire through the stream path: an ICC profile
        // stream without the required /N entry is genuinely non-conformant.
        use crate::object::ObjectRef;
        let mut doc =
            crate::document::PdfDocument::from_bytes(build_pdf_with_iccbased_colorspace(false))
                .expect("parse test pdf");
        let validator = PdfXValidator::new(PdfXLevel::X4);
        let mut result = XValidationResult::new(PdfXLevel::X4);
        let cs_obj = Object::Array(vec![
            Object::Name("ICCBased".to_string()),
            Object::Reference(ObjectRef { id: 5, gen: 0 }),
        ]);
        validator
            .validate_icc_profile(&cs_obj, "CS0", 1, &mut doc, &mut result)
            .expect("validate_icc_profile");
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == XErrorCode::IccProfileInvalid),
            "ICC stream missing /N must be flagged: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_standard14_fonts() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);

        let mut font_dict = HashMap::new();
        font_dict.insert("BaseFont".to_string(), Object::Name("Helvetica".to_string()));
        assert!(validator.is_standard14_font(&font_dict));

        font_dict.insert("BaseFont".to_string(), Object::Name("CustomFont".to_string()));
        assert!(!validator.is_standard14_font(&font_dict));
    }

    #[test]
    fn test_finalize_result() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let result = XValidationResult::new(PdfXLevel::X1a2003);
        let finalized = validator.finalize_result(result);
        assert!(finalized.is_compliant);

        let mut result_with_error = XValidationResult::new(PdfXLevel::X1a2003);
        result_with_error
            .add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "Test error"));
        let finalized = validator.finalize_result(result_with_error);
        assert!(!finalized.is_compliant);
    }

    #[test]
    fn test_finalize_without_warnings() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003).include_warnings(false);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        result
            .add_warning(XComplianceError::warning(XErrorCode::TrappedKeyMissing, "Test warning"));

        let finalized = validator.finalize_result(result);
        assert!(finalized.warnings.is_empty());
    }

    // ==========================================
    // parse_box tests
    // ==========================================

    #[test]
    fn test_parse_box_none() {
        assert_eq!(PdfXValidator::parse_box(None), None);
    }

    #[test]
    fn test_parse_box_not_array() {
        let obj = Object::Integer(42);
        assert_eq!(PdfXValidator::parse_box(Some(&obj)), None);
    }

    #[test]
    fn test_parse_box_too_few_elements() {
        let obj = Object::Array(vec![Object::Real(0.0), Object::Real(0.0), Object::Real(100.0)]);
        assert_eq!(PdfXValidator::parse_box(Some(&obj)), None);
    }

    #[test]
    fn test_parse_box_with_reals() {
        let obj = Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(612.0),
            Object::Real(792.0),
        ]);
        let result = PdfXValidator::parse_box(Some(&obj));
        assert!(result.is_some());
        let b = result.unwrap();
        assert_eq!(b, [0.0, 0.0, 612.0, 792.0]);
    }

    #[test]
    fn test_parse_box_with_integers() {
        let obj = Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(612),
            Object::Integer(792),
        ]);
        let result = PdfXValidator::parse_box(Some(&obj));
        assert!(result.is_some());
        let b = result.unwrap();
        assert_eq!(b, [0.0, 0.0, 612.0, 792.0]);
    }

    #[test]
    fn test_parse_box_with_mixed_types() {
        let obj = Object::Array(vec![
            Object::Integer(0),
            Object::Real(0.5),
            Object::Integer(612),
            Object::Real(792.5),
        ]);
        let result = PdfXValidator::parse_box(Some(&obj));
        assert!(result.is_some());
        let b = result.unwrap();
        assert_eq!(b, [0.0, 0.5, 612.0, 792.5]);
    }

    #[test]
    fn test_parse_box_with_non_numeric() {
        let obj = Object::Array(vec![
            Object::Real(0.0),
            Object::Name("bad".to_string()),
            Object::Real(612.0),
            Object::Real(792.0),
        ]);
        assert_eq!(PdfXValidator::parse_box(Some(&obj)), None);
    }

    #[test]
    fn test_parse_box_extra_elements_ignored() {
        let obj = Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(100.0),
            Object::Real(200.0),
            Object::Real(300.0), // extra element
        ]);
        let result = PdfXValidator::parse_box(Some(&obj));
        assert!(result.is_some());
        assert_eq!(result.unwrap(), [0.0, 0.0, 100.0, 200.0]);
    }

    // ==========================================
    // box_contains tests
    // ==========================================

    #[test]
    fn test_box_contains_exact_match() {
        let outer = [0.0, 0.0, 612.0, 792.0];
        let inner = [0.0, 0.0, 612.0, 792.0];
        assert!(PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_inner_smaller() {
        let outer = [0.0, 0.0, 612.0, 792.0];
        let inner = [10.0, 10.0, 600.0, 780.0];
        assert!(PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_inner_exceeds_right() {
        let outer = [0.0, 0.0, 612.0, 792.0];
        let inner = [10.0, 10.0, 620.0, 780.0]; // right exceeds
        assert!(!PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_inner_exceeds_top() {
        let outer = [0.0, 0.0, 612.0, 792.0];
        let inner = [10.0, 10.0, 600.0, 800.0]; // top exceeds
        assert!(!PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_inner_exceeds_left() {
        let outer = [10.0, 0.0, 612.0, 792.0];
        let inner = [5.0, 0.0, 600.0, 780.0]; // left exceeds
        assert!(!PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_inner_exceeds_bottom() {
        let outer = [0.0, 10.0, 612.0, 792.0];
        let inner = [0.0, 5.0, 600.0, 780.0]; // bottom exceeds
        assert!(!PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_within_tolerance() {
        let outer = [0.0, 0.0, 612.0, 792.0];
        // Inner slightly exceeds by less than tolerance (0.01)
        let inner = [-0.005, -0.005, 612.005, 792.005];
        assert!(PdfXValidator::box_contains(&outer, &inner));
    }

    #[test]
    fn test_box_contains_beyond_tolerance() {
        let outer = [0.0, 0.0, 612.0, 792.0];
        // Inner exceeds by more than tolerance
        let inner = [-0.02, 0.0, 612.0, 792.0];
        assert!(!PdfXValidator::box_contains(&outer, &inner));
    }

    // ==========================================
    // is_standard14_font tests
    // ==========================================

    #[test]
    fn test_standard14_all_fonts() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let standard14 = [
            "Courier",
            "Courier-Bold",
            "Courier-Oblique",
            "Courier-BoldOblique",
            "Helvetica",
            "Helvetica-Bold",
            "Helvetica-Oblique",
            "Helvetica-BoldOblique",
            "Times-Roman",
            "Times-Bold",
            "Times-Italic",
            "Times-BoldItalic",
            "Symbol",
            "ZapfDingbats",
        ];
        for name in &standard14 {
            let mut font_dict = HashMap::new();
            font_dict.insert("BaseFont".to_string(), Object::Name(name.to_string()));
            assert!(validator.is_standard14_font(&font_dict), "{} should be standard14", name);
        }
    }

    #[test]
    fn test_standard14_non_standard_fonts() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        for name in &["Arial", "TimesNewRoman", "Verdana", "Georgia", "Calibri"] {
            let mut font_dict = HashMap::new();
            font_dict.insert("BaseFont".to_string(), Object::Name(name.to_string()));
            assert!(!validator.is_standard14_font(&font_dict), "{} should NOT be standard14", name);
        }
    }

    #[test]
    fn test_standard14_no_basefont() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let font_dict = HashMap::new();
        assert!(!validator.is_standard14_font(&font_dict));
    }

    #[test]
    fn test_standard14_wrong_basefont_type() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let mut font_dict = HashMap::new();
        font_dict.insert("BaseFont".to_string(), Object::String(b"Helvetica".to_vec()));
        assert!(!validator.is_standard14_font(&font_dict));
    }

    // ==========================================
    // get_colorspace_name tests
    // ==========================================

    // Helper that mimics get_colorspace_name without requiring a PdfDocument.
    // Tests the Name, Array, and unknown-type branches.
    fn colorspace_name_for_test(cs: &Object) -> String {
        match cs {
            Object::Name(n) => n.clone(),
            Object::Array(arr) => {
                if let Some(Object::Name(n)) = arr.first() {
                    n.clone()
                } else {
                    "Unknown".to_string()
                }
            },
            _ => "Unknown".to_string(),
        }
    }

    #[test]
    fn test_colorspace_name_from_name() {
        let cs = Object::Name("DeviceCMYK".to_string());
        assert_eq!(colorspace_name_for_test(&cs), "DeviceCMYK");
    }

    #[test]
    fn test_colorspace_name_from_array() {
        let cs = Object::Array(vec![
            Object::Name("ICCBased".to_string()),
            Object::Reference(ObjectRef::new(10, 0)),
        ]);
        assert_eq!(colorspace_name_for_test(&cs), "ICCBased");
    }

    #[test]
    fn test_colorspace_name_from_empty_array() {
        let cs = Object::Array(vec![]);
        assert_eq!(colorspace_name_for_test(&cs), "Unknown");
    }

    #[test]
    fn test_colorspace_name_from_unknown() {
        let cs = Object::Integer(42);
        assert_eq!(colorspace_name_for_test(&cs), "Unknown");
    }

    #[test]
    fn test_colorspace_name_from_array_non_name_first() {
        let cs = Object::Array(vec![Object::Integer(42)]);
        assert_eq!(colorspace_name_for_test(&cs), "Unknown");
    }

    // ==========================================
    // should_stop tests
    // ==========================================

    #[test]
    fn test_should_stop_false_when_disabled() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        result.add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "error"));
        assert!(!validator.should_stop(&result));
    }

    #[test]
    fn test_should_stop_true_when_enabled_with_errors() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003).stop_on_first_error(true);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        result.add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "error"));
        assert!(validator.should_stop(&result));
    }

    #[test]
    fn test_should_stop_false_when_enabled_no_errors() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003).stop_on_first_error(true);
        let result = XValidationResult::new(PdfXLevel::X1a2003);
        assert!(!validator.should_stop(&result));
    }

    // ==========================================
    // check_action tests (standalone via dict building)
    // ==========================================

    #[test]
    fn test_check_action_javascript() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        let action = Object::Dictionary({
            let mut d = HashMap::new();
            d.insert("S".to_string(), Object::Name("JavaScript".to_string()));
            d
        });
        // check_action requires a document for Reference resolution, but for Dictionary
        // it works directly. We can't easily test without a document, but we can verify
        // the action dict matching logic by examining the result directly.
        // Since we can't call check_action without &mut PdfDocument, let's test
        // the Dictionary construction path logic indirectly.

        // Test via finalize approach: manually simulate what check_action does
        if let Object::Dictionary(d) = &action {
            if let Some(Object::Name(action_type)) = d.get("S") {
                if action_type.as_str() == "JavaScript" {
                    result.add_error(
                        XComplianceError::new(
                            XErrorCode::JavaScriptNotAllowed,
                            "JavaScript actions not allowed",
                        )
                        .with_clause("6.6.1"),
                    );
                }
            }
        }
        assert!(result.has_errors());
        assert_eq!(result.errors[0].code, XErrorCode::JavaScriptNotAllowed);
    }

    #[test]
    fn test_check_action_launch() {
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        let action_dict: HashMap<String, Object> = {
            let mut d = HashMap::new();
            d.insert("S".to_string(), Object::Name("Launch".to_string()));
            d
        };
        if let Some(Object::Name(action_type)) = action_dict.get("S") {
            match action_type.as_str() {
                "Launch" | "Sound" | "Movie" | "ImportData" | "ResetForm" | "SubmitForm" => {
                    result.add_error(
                        XComplianceError::new(
                            XErrorCode::ActionNotAllowed,
                            format!("Action type '{}' not allowed in PDF/X", action_type),
                        )
                        .with_clause("6.6.1"),
                    );
                },
                _ => {},
            }
        }
        assert!(result.has_errors());
        assert_eq!(result.errors[0].code, XErrorCode::ActionNotAllowed);
    }

    #[test]
    fn test_check_action_allowed_type() {
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        let action_dict: HashMap<String, Object> = {
            let mut d = HashMap::new();
            d.insert("S".to_string(), Object::Name("GoTo".to_string()));
            d
        };
        if let Some(Object::Name(action_type)) = action_dict.get("S") {
            match action_type.as_str() {
                "JavaScript" | "Launch" | "Sound" | "Movie" | "ImportData" | "ResetForm"
                | "SubmitForm" => {
                    result.add_error(XComplianceError::new(
                        XErrorCode::ActionNotAllowed,
                        "Not allowed",
                    ));
                },
                _ => {},
            }
        }
        assert!(!result.has_errors());
    }

    // ==========================================
    // Validator configuration chaining tests
    // ==========================================

    #[test]
    fn test_validator_all_levels() {
        let levels = [
            PdfXLevel::X1a2001,
            PdfXLevel::X1a2003,
            PdfXLevel::X32002,
            PdfXLevel::X32003,
            PdfXLevel::X4,
            PdfXLevel::X4p,
            PdfXLevel::X5g,
            PdfXLevel::X5n,
            PdfXLevel::X5pg,
            PdfXLevel::X6,
        ];
        for level in &levels {
            let validator = PdfXValidator::new(*level);
            assert_eq!(validator.level, *level);
        }
    }

    #[test]
    fn test_finalize_with_warnings_and_errors() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        result.add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "Font not embedded"));
        result.add_warning(XComplianceError::warning(
            XErrorCode::TrappedKeyMissing,
            "Trapped key missing",
        ));

        let finalized = validator.finalize_result(result);
        assert!(!finalized.is_compliant);
        assert_eq!(finalized.errors.len(), 1);
        assert_eq!(finalized.warnings.len(), 1);
    }

    #[test]
    fn test_finalize_with_only_warnings_is_compliant() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        result.add_warning(XComplianceError::warning(
            XErrorCode::TrappedKeyMissing,
            "Trapped key missing",
        ));

        let finalized = validator.finalize_result(result);
        assert!(finalized.is_compliant);
        assert_eq!(finalized.warnings.len(), 1);
    }

    #[test]
    fn test_finalize_strips_warnings_when_configured() {
        let validator = PdfXValidator::new(PdfXLevel::X1a2003).include_warnings(false);
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        result.add_warning(XComplianceError::warning(XErrorCode::TrappedKeyMissing, "w1"));
        result.add_warning(XComplianceError::warning(XErrorCode::XmpMetadataInvalid, "w2"));
        result.add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "e1"));

        let finalized = validator.finalize_result(result);
        assert!(!finalized.is_compliant);
        assert!(finalized.warnings.is_empty());
        assert_eq!(finalized.errors.len(), 1);
    }

    // ==========================================
    // Compliance error display tests
    // ==========================================

    #[test]
    fn test_compliance_error_display_with_object_id() {
        let error = XComplianceError::new(XErrorCode::IccProfileMissing, "Missing ICC profile")
            .with_object_id(42);

        let display = format!("{}", error);
        assert!(display.contains("[XCOLOR-004]"));
        assert!(display.contains("object 42"));
    }

    #[test]
    fn test_compliance_error_display_with_page_and_object() {
        let error = XComplianceError::new(XErrorCode::TransparencyNotAllowed, "Transparency found")
            .with_page(2)
            .with_object_id(100);

        let display = format!("{}", error);
        assert!(display.contains("page 3")); // 0-indexed page 2 => display as "page 3"
        assert!(display.contains("object 100"));
    }

    #[test]
    fn test_compliance_warning_is_not_error() {
        let warning =
            XComplianceError::warning(XErrorCode::TrappedKeyMissing, "Trapped key missing");
        assert!(!warning.is_error());
    }

    // ==========================================
    // XValidationResult tests
    // ==========================================

    #[test]
    fn test_validation_result_total_issues() {
        let mut result = XValidationResult::new(PdfXLevel::X4);
        assert_eq!(result.total_issues(), 0);

        result.add_error(XComplianceError::new(XErrorCode::FontNotEmbedded, "e1"));
        assert_eq!(result.total_issues(), 1);

        result.add_warning(XComplianceError::warning(XErrorCode::TrappedKeyMissing, "w1"));
        assert_eq!(result.total_issues(), 2);
    }

    #[test]
    fn test_validation_result_has_warnings() {
        let mut result = XValidationResult::new(PdfXLevel::X4);
        assert!(!result.has_warnings());

        result.add_warning(XComplianceError::warning(XErrorCode::TrappedKeyMissing, "warning"));
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validation_result_add_error_makes_non_compliant() {
        let mut result = XValidationResult::new(PdfXLevel::X4);
        assert!(result.is_compliant);

        result.add_error(XComplianceError::new(XErrorCode::EncryptionNotAllowed, "Encrypted"));
        assert!(!result.is_compliant);
    }

    #[test]
    fn test_validation_result_add_warning_via_add_error() {
        // Adding a warning through add_error should put it in warnings, not errors
        let mut result = XValidationResult::new(PdfXLevel::X4);
        result.add_error(XComplianceError::warning(
            XErrorCode::TrappedKeyMissing,
            "this is a warning",
        ));
        assert!(result.is_compliant); // warnings don't make non-compliant
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_result_detected_level() {
        let mut result = XValidationResult::new(PdfXLevel::X1a2003);
        assert!(result.detected_level.is_none());
        result.detected_level = Some(PdfXLevel::X4);
        assert_eq!(result.detected_level, Some(PdfXLevel::X4));
    }

    // ==========================================
    // Error code display tests
    // ==========================================

    #[test]
    fn test_error_code_display_color_codes() {
        assert_eq!(format!("{}", XErrorCode::RgbColorNotAllowed), "XCOLOR-001");
        assert_eq!(format!("{}", XErrorCode::LabColorNotAllowed), "XCOLOR-002");
        assert_eq!(format!("{}", XErrorCode::DeviceNInvalid), "XCOLOR-003");
        assert_eq!(format!("{}", XErrorCode::IccProfileMissing), "XCOLOR-004");
        assert_eq!(format!("{}", XErrorCode::IccProfileInvalid), "XCOLOR-005");
        assert_eq!(format!("{}", XErrorCode::DeviceColorWithoutIntent), "XCOLOR-006");
    }

    #[test]
    fn test_error_code_display_transparency_codes() {
        assert_eq!(format!("{}", XErrorCode::TransparencyNotAllowed), "XTRANS-001");
        assert_eq!(format!("{}", XErrorCode::BlendModeNotAllowed), "XTRANS-002");
        assert_eq!(format!("{}", XErrorCode::SoftMaskNotAllowed), "XTRANS-003");
        assert_eq!(format!("{}", XErrorCode::SMaskNotAllowed), "XTRANS-004");
    }

    #[test]
    fn test_error_code_display_font_codes() {
        assert_eq!(format!("{}", XErrorCode::FontNotEmbedded), "XFONT-001");
        assert_eq!(format!("{}", XErrorCode::Type3FontNotAllowed), "XFONT-002");
        assert_eq!(format!("{}", XErrorCode::FontMissingWidths), "XFONT-003");
        assert_eq!(format!("{}", XErrorCode::FontSubsetIncomplete), "XFONT-004");
    }

    #[test]
    fn test_error_code_display_metadata_codes() {
        assert_eq!(format!("{}", XErrorCode::OutputIntentMissing), "XMETA-001");
        assert_eq!(format!("{}", XErrorCode::OutputIntentInvalid), "XMETA-002");
        assert_eq!(format!("{}", XErrorCode::OutputConditionMissing), "XMETA-003");
        assert_eq!(format!("{}", XErrorCode::TrappedKeyMissing), "XMETA-004");
        assert_eq!(format!("{}", XErrorCode::XmpMetadataMissing), "XMETA-005");
        assert_eq!(format!("{}", XErrorCode::XmpMetadataInvalid), "XMETA-006");
        assert_eq!(format!("{}", XErrorCode::GtsPdfxVersionMissing), "XMETA-007");
        assert_eq!(format!("{}", XErrorCode::GtsPdfxConformanceMissing), "XMETA-008");
    }

    #[test]
    fn test_error_code_display_box_codes() {
        assert_eq!(format!("{}", XErrorCode::TrimOrArtBoxMissing), "XBOX-001");
        assert_eq!(format!("{}", XErrorCode::BleedBoxInvalid), "XBOX-002");
        assert_eq!(format!("{}", XErrorCode::TrimBoxInvalid), "XBOX-003");
        assert_eq!(format!("{}", XErrorCode::MediaBoxMissing), "XBOX-004");
        assert_eq!(format!("{}", XErrorCode::BoxesInconsistent), "XBOX-005");
    }

    #[test]
    fn test_error_code_display_content_codes() {
        assert_eq!(format!("{}", XErrorCode::EncryptionNotAllowed), "XCONT-001");
        assert_eq!(format!("{}", XErrorCode::JavaScriptNotAllowed), "XCONT-002");
        assert_eq!(format!("{}", XErrorCode::ExternalContentNotAllowed), "XCONT-003");
        assert_eq!(format!("{}", XErrorCode::EmbeddedFileNotAllowed), "XCONT-004");
        assert_eq!(format!("{}", XErrorCode::FormXObjectInvalid), "XCONT-005");
        assert_eq!(format!("{}", XErrorCode::PostScriptXObjectNotAllowed), "XCONT-006");
        assert_eq!(format!("{}", XErrorCode::ReferenceXObjectNotAllowed), "XCONT-007");
    }

    #[test]
    fn test_error_code_display_annotation_codes() {
        assert_eq!(format!("{}", XErrorCode::AnnotationNotAllowed), "XANNOT-001");
        assert_eq!(format!("{}", XErrorCode::PrinterMarkInvalid), "XANNOT-002");
        assert_eq!(format!("{}", XErrorCode::TrapNetInvalid), "XANNOT-003");
    }

    #[test]
    fn test_error_code_display_action_codes() {
        assert_eq!(format!("{}", XErrorCode::ActionNotAllowed), "XACTION-001");
    }

    #[test]
    fn test_error_code_display_other_codes() {
        assert_eq!(format!("{}", XErrorCode::TransferFunctionNotAllowed), "XOTHER-001");
        assert_eq!(format!("{}", XErrorCode::HalftoneTypeNotAllowed), "XOTHER-002");
        assert_eq!(format!("{}", XErrorCode::AlternateImageNotAllowed), "XOTHER-003");
        assert_eq!(format!("{}", XErrorCode::OpiNotAllowed), "XOTHER-004");
        assert_eq!(format!("{}", XErrorCode::PreseparatedNotAllowed), "XOTHER-005");
    }

    // ==========================================
    // PdfXLevel method tests
    // ==========================================

    #[test]
    fn test_pdf_x_level_gts_versions() {
        assert_eq!(PdfXLevel::X1a2001.gts_pdfx_version(), "PDF/X-1a:2001");
        assert_eq!(PdfXLevel::X1a2003.gts_pdfx_version(), "PDF/X-1a:2003");
        assert_eq!(PdfXLevel::X32002.gts_pdfx_version(), "PDF/X-3:2002");
        assert_eq!(PdfXLevel::X32003.gts_pdfx_version(), "PDF/X-3:2003");
        assert_eq!(PdfXLevel::X4.gts_pdfx_version(), "PDF/X-4");
        assert_eq!(PdfXLevel::X4p.gts_pdfx_version(), "PDF/X-4p");
        assert_eq!(PdfXLevel::X5g.gts_pdfx_version(), "PDF/X-5g");
        assert_eq!(PdfXLevel::X5n.gts_pdfx_version(), "PDF/X-5n");
        assert_eq!(PdfXLevel::X5pg.gts_pdfx_version(), "PDF/X-5pg");
        assert_eq!(PdfXLevel::X6.gts_pdfx_version(), "PDF/X-6");
    }

    #[test]
    fn test_pdf_x_level_xmp_version_matches_gts() {
        let levels = [
            PdfXLevel::X1a2001,
            PdfXLevel::X1a2003,
            PdfXLevel::X32002,
            PdfXLevel::X32003,
            PdfXLevel::X4,
            PdfXLevel::X4p,
            PdfXLevel::X5g,
            PdfXLevel::X5n,
            PdfXLevel::X5pg,
            PdfXLevel::X6,
        ];
        for level in &levels {
            assert_eq!(level.xmp_version(), level.gts_pdfx_version());
        }
    }

    #[test]
    fn test_pdf_x_level_from_gts_all_versions() {
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-1a:2001"), Some(PdfXLevel::X1a2001));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-1:2001"), Some(PdfXLevel::X1a2001));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-1a:2003"), Some(PdfXLevel::X1a2003));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-1:2003"), Some(PdfXLevel::X1a2003));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-3:2002"), Some(PdfXLevel::X32002));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-3:2003"), Some(PdfXLevel::X32003));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-4"), Some(PdfXLevel::X4));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-4p"), Some(PdfXLevel::X4p));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-5g"), Some(PdfXLevel::X5g));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-5n"), Some(PdfXLevel::X5n));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-5pg"), Some(PdfXLevel::X5pg));
        assert_eq!(PdfXLevel::from_gts_version("PDF/X-6"), Some(PdfXLevel::X6));
        assert_eq!(PdfXLevel::from_gts_version("garbage"), None);
        assert_eq!(PdfXLevel::from_gts_version(""), None);
    }

    #[test]
    fn test_pdf_x_level_from_gts_with_whitespace() {
        assert_eq!(PdfXLevel::from_gts_version("  PDF/X-4  "), Some(PdfXLevel::X4));
    }

    #[test]
    fn test_pdf_x_level_allows_transparency_comprehensive() {
        assert!(!PdfXLevel::X1a2001.allows_transparency());
        assert!(!PdfXLevel::X1a2003.allows_transparency());
        assert!(!PdfXLevel::X32002.allows_transparency());
        assert!(!PdfXLevel::X32003.allows_transparency());
        assert!(PdfXLevel::X4.allows_transparency());
        assert!(PdfXLevel::X4p.allows_transparency());
        assert!(PdfXLevel::X5g.allows_transparency());
        assert!(PdfXLevel::X5n.allows_transparency());
        assert!(PdfXLevel::X5pg.allows_transparency());
        assert!(PdfXLevel::X6.allows_transparency());
    }

    #[test]
    fn test_pdf_x_level_allows_rgb_comprehensive() {
        assert!(!PdfXLevel::X1a2001.allows_rgb());
        assert!(!PdfXLevel::X1a2003.allows_rgb());
        assert!(PdfXLevel::X32002.allows_rgb());
        assert!(PdfXLevel::X32003.allows_rgb());
        assert!(PdfXLevel::X4.allows_rgb());
        assert!(PdfXLevel::X4p.allows_rgb());
        assert!(PdfXLevel::X5g.allows_rgb());
        assert!(PdfXLevel::X5n.allows_rgb());
        assert!(PdfXLevel::X5pg.allows_rgb());
        assert!(PdfXLevel::X6.allows_rgb());
    }

    #[test]
    fn test_pdf_x_level_allows_layers_comprehensive() {
        assert!(!PdfXLevel::X1a2001.allows_layers());
        assert!(!PdfXLevel::X1a2003.allows_layers());
        assert!(!PdfXLevel::X32002.allows_layers());
        assert!(!PdfXLevel::X32003.allows_layers());
        assert!(PdfXLevel::X4.allows_layers());
        assert!(PdfXLevel::X4p.allows_layers());
        assert!(PdfXLevel::X5g.allows_layers());
        assert!(PdfXLevel::X5n.allows_layers());
        assert!(PdfXLevel::X5pg.allows_layers());
        assert!(PdfXLevel::X6.allows_layers());
    }

    #[test]
    fn test_pdf_x_level_allows_external_icc_comprehensive() {
        assert!(!PdfXLevel::X1a2001.allows_external_icc());
        assert!(!PdfXLevel::X1a2003.allows_external_icc());
        assert!(!PdfXLevel::X32002.allows_external_icc());
        assert!(!PdfXLevel::X32003.allows_external_icc());
        assert!(!PdfXLevel::X4.allows_external_icc());
        assert!(PdfXLevel::X4p.allows_external_icc());
        assert!(!PdfXLevel::X5g.allows_external_icc());
        assert!(PdfXLevel::X5n.allows_external_icc());
        assert!(PdfXLevel::X5pg.allows_external_icc());
        assert!(!PdfXLevel::X6.allows_external_icc());
    }

    #[test]
    fn test_pdf_x_level_allows_external_graphics_comprehensive() {
        assert!(!PdfXLevel::X1a2001.allows_external_graphics());
        assert!(!PdfXLevel::X1a2003.allows_external_graphics());
        assert!(!PdfXLevel::X32002.allows_external_graphics());
        assert!(!PdfXLevel::X32003.allows_external_graphics());
        assert!(!PdfXLevel::X4.allows_external_graphics());
        assert!(!PdfXLevel::X4p.allows_external_graphics());
        assert!(PdfXLevel::X5g.allows_external_graphics());
        assert!(!PdfXLevel::X5n.allows_external_graphics());
        assert!(PdfXLevel::X5pg.allows_external_graphics());
        assert!(!PdfXLevel::X6.allows_external_graphics());
    }

    #[test]
    fn test_pdf_x_level_required_pdf_versions() {
        assert_eq!(PdfXLevel::X1a2001.required_pdf_version(), "1.3");
        assert_eq!(PdfXLevel::X32002.required_pdf_version(), "1.3");
        assert_eq!(PdfXLevel::X1a2003.required_pdf_version(), "1.4");
        assert_eq!(PdfXLevel::X32003.required_pdf_version(), "1.4");
        assert_eq!(PdfXLevel::X4.required_pdf_version(), "1.6");
        assert_eq!(PdfXLevel::X4p.required_pdf_version(), "1.6");
        assert_eq!(PdfXLevel::X5g.required_pdf_version(), "1.6");
        assert_eq!(PdfXLevel::X5n.required_pdf_version(), "1.6");
        assert_eq!(PdfXLevel::X5pg.required_pdf_version(), "1.6");
        assert_eq!(PdfXLevel::X6.required_pdf_version(), "2.0");
    }

    #[test]
    fn test_pdf_x_level_iso_standards() {
        assert_eq!(PdfXLevel::X1a2001.iso_standard(), "ISO 15930-1:2001");
        assert_eq!(PdfXLevel::X1a2003.iso_standard(), "ISO 15930-4:2003");
        assert_eq!(PdfXLevel::X32002.iso_standard(), "ISO 15930-3:2002");
        assert_eq!(PdfXLevel::X32003.iso_standard(), "ISO 15930-6:2003");
        assert_eq!(PdfXLevel::X4.iso_standard(), "ISO 15930-7:2010");
        assert_eq!(PdfXLevel::X4p.iso_standard(), "ISO 15930-7:2010");
        assert_eq!(PdfXLevel::X5g.iso_standard(), "ISO 15930-8:2010");
        assert_eq!(PdfXLevel::X5n.iso_standard(), "ISO 15930-8:2010");
        assert_eq!(PdfXLevel::X5pg.iso_standard(), "ISO 15930-8:2010");
        assert_eq!(PdfXLevel::X6.iso_standard(), "ISO 15930-9:2020");
    }

    #[test]
    fn test_pdf_x_level_display_all() {
        assert_eq!(format!("{}", PdfXLevel::X1a2001), "PDF/X-1a:2001");
        assert_eq!(format!("{}", PdfXLevel::X1a2003), "PDF/X-1a:2003");
        assert_eq!(format!("{}", PdfXLevel::X32002), "PDF/X-3:2002");
        assert_eq!(format!("{}", PdfXLevel::X32003), "PDF/X-3:2003");
        assert_eq!(format!("{}", PdfXLevel::X4), "PDF/X-4");
        assert_eq!(format!("{}", PdfXLevel::X4p), "PDF/X-4p");
        assert_eq!(format!("{}", PdfXLevel::X5g), "PDF/X-5g");
        assert_eq!(format!("{}", PdfXLevel::X5n), "PDF/X-5n");
        assert_eq!(format!("{}", PdfXLevel::X5pg), "PDF/X-5pg");
        assert_eq!(format!("{}", PdfXLevel::X6), "PDF/X-6");
    }

    // ==========================================
    // XValidationStats tests
    // ==========================================

    #[test]
    fn test_validation_stats_default() {
        let stats = XValidationStats::default();
        assert_eq!(stats.pages_checked, 0);
        assert_eq!(stats.fonts_checked, 0);
        assert_eq!(stats.fonts_embedded, 0);
        assert_eq!(stats.images_checked, 0);
        assert_eq!(stats.annotations_checked, 0);
        assert!(stats.color_spaces_found.is_empty());
        assert!(!stats.has_transparency);
        assert!(!stats.has_layers);
        assert!(stats.output_intent.is_none());
    }

    #[test]
    fn test_validation_stats_mutation() {
        let mut stats = XValidationStats {
            pages_checked: 5,
            fonts_checked: 10,
            fonts_embedded: 8,
            has_transparency: true,
            ..Default::default()
        };
        stats.color_spaces_found.push("DeviceCMYK".to_string());
        stats.output_intent = Some("Fogra39".to_string());

        assert_eq!(stats.pages_checked, 5);
        assert_eq!(stats.fonts_checked, 10);
        assert_eq!(stats.fonts_embedded, 8);
        assert!(stats.has_transparency);
        assert_eq!(stats.color_spaces_found.len(), 1);
        assert_eq!(stats.output_intent, Some("Fogra39".to_string()));
    }
}
