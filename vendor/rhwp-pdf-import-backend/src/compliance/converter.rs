//! PDF/A conversion functionality.
//!
//! This module provides the ability to convert PDF documents to PDF/A compliance.
//!
//! ## Overview
//!
//! PDF/A conversion involves:
//! - Validating current compliance state
//! - Embedding all fonts
//! - Adding required XMP metadata
//! - Setting output intent with ICC profile
//! - Removing prohibited features (JavaScript, encryption, etc.)
//! - Flattening transparency (for PDF/A-1)
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::api::Pdf;
//! use pdf_oxide::compliance::{PdfAConverter, PdfALevel};
//!
//! let mut pdf = Pdf::open("document.pdf")?;
//! let converter = PdfAConverter::new(PdfALevel::A2b);
//! let result = converter.convert(&mut pdf)?;
//!
//! if result.success {
//!     pdf.save("document_pdfa.pdf")?;
//! }
//! ```
//!
//! ## Standards Reference
//!
//! - ISO 19005-1:2005 (PDF/A-1)
//! - ISO 19005-2:2011 (PDF/A-2)
//! - ISO 19005-3:2012 (PDF/A-3)

use super::types::{ComplianceError, ErrorCode, PdfALevel, ValidationResult};
use super::PdfAValidator;
use crate::document::PdfDocument;
use crate::editor::DocumentEditor;
use crate::error::Result;
use crate::object::{Object, ObjectRef};

/// Configuration options for PDF/A conversion.
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Whether to embed fonts that are not embedded.
    pub embed_fonts: bool,
    /// Whether to remove JavaScript.
    pub remove_javascript: bool,
    /// Whether to remove encryption.
    pub remove_encryption: bool,
    /// Whether to flatten transparency (for PDF/A-1).
    pub flatten_transparency: bool,
    /// Whether to remove embedded files (for PDF/A-1/2).
    pub remove_embedded_files: bool,
    /// Whether to add structure tree (for level A).
    pub add_structure: bool,
    /// sRGB ICC profile data (optional, built-in default used if None).
    pub icc_profile: Option<Vec<u8>>,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            embed_fonts: true,
            remove_javascript: true,
            remove_encryption: true,
            flatten_transparency: true,
            remove_embedded_files: true,
            add_structure: false,
            icc_profile: None,
        }
    }
}

impl ConversionConfig {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to embed fonts.
    pub fn embed_fonts(mut self, embed: bool) -> Self {
        self.embed_fonts = embed;
        self
    }

    /// Set whether to remove JavaScript.
    pub fn remove_javascript(mut self, remove: bool) -> Self {
        self.remove_javascript = remove;
        self
    }

    /// Set whether to flatten transparency.
    pub fn flatten_transparency(mut self, flatten: bool) -> Self {
        self.flatten_transparency = flatten;
        self
    }

    /// Set whether to add structure tree.
    pub fn add_structure(mut self, add: bool) -> Self {
        self.add_structure = add;
        self
    }

    /// Set custom ICC profile data.
    pub fn with_icc_profile(mut self, profile: Vec<u8>) -> Self {
        self.icc_profile = Some(profile);
        self
    }
}

/// Result of PDF/A conversion.
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Whether conversion was successful.
    pub success: bool,
    /// Target PDF/A level.
    pub level: PdfALevel,
    /// Validation result after conversion.
    pub validation: ValidationResult,
    /// Actions taken during conversion.
    pub actions: Vec<ConversionAction>,
    /// Errors that prevented conversion.
    pub errors: Vec<ConversionError>,
}

impl ConversionResult {
    /// Create a new conversion result.
    fn new(level: PdfALevel) -> Self {
        Self {
            success: false,
            level,
            validation: ValidationResult::new(level),
            actions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add an action to the result.
    fn add_action(&mut self, action: ConversionAction) {
        self.actions.push(action);
    }

    /// Add an error to the result.
    fn add_error(&mut self, error: ConversionError) {
        self.errors.push(error);
    }
}

/// Action taken during conversion.
#[derive(Debug, Clone)]
pub struct ConversionAction {
    /// Type of action.
    pub action_type: ActionType,
    /// Description of what was done.
    pub description: String,
    /// Related error code that was fixed (if any).
    pub fixed_error: Option<ErrorCode>,
}

impl ConversionAction {
    /// Create a new conversion action.
    fn new(action_type: ActionType, description: impl Into<String>) -> Self {
        Self {
            action_type,
            description: description.into(),
            fixed_error: None,
        }
    }

    /// Set the fixed error code.
    fn with_fixed_error(mut self, code: ErrorCode) -> Self {
        self.fixed_error = Some(code);
        self
    }
}

/// Types of conversion actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Added XMP metadata.
    AddedXmpMetadata,
    /// Added PDF/A identification to XMP.
    AddedPdfaIdentification,
    /// Embedded a font.
    EmbeddedFont,
    /// Added output intent.
    AddedOutputIntent,
    /// Removed JavaScript.
    RemovedJavaScript,
    /// Removed encryption.
    RemovedEncryption,
    /// Flattened transparency.
    FlattenedTransparency,
    /// Removed embedded files.
    RemovedEmbeddedFiles,
    /// Added structure tree.
    AddedStructure,
    /// Fixed annotation appearance.
    FixedAnnotation,
    /// Added document language.
    AddedLanguage,
}

/// Error during conversion.
#[derive(Debug, Clone)]
pub struct ConversionError {
    /// The error that could not be fixed.
    pub error_code: ErrorCode,
    /// Description of why it couldn't be fixed.
    pub reason: String,
}

impl ConversionError {
    /// Create a new conversion error.
    fn new(error_code: ErrorCode, reason: impl Into<String>) -> Self {
        Self {
            error_code,
            reason: reason.into(),
        }
    }
}

/// PDF/A converter for transforming documents to PDF/A compliance.
#[derive(Debug, Clone)]
pub struct PdfAConverter {
    /// Target PDF/A level.
    level: PdfALevel,
    /// Conversion configuration.
    config: ConversionConfig,
    /// Validator for checking compliance.
    validator: PdfAValidator,
}

impl PdfAConverter {
    /// Create a new PDF/A converter for the specified level.
    pub fn new(level: PdfALevel) -> Self {
        Self {
            level,
            config: ConversionConfig::default(),
            validator: PdfAValidator::new(),
        }
    }

    /// Set the conversion configuration.
    pub fn with_config(mut self, config: ConversionConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the target PDF/A level.
    pub fn level(&self) -> PdfALevel {
        self.level
    }

    /// Convert a PDF document to PDF/A compliance.
    ///
    /// Internally builds a [`DocumentEditor`] from the document bytes, stages
    /// all mutations, commits them in a single save+reparse, then replaces the
    /// caller's document with the updated version.
    pub fn convert(&self, document: &mut PdfDocument) -> Result<ConversionResult> {
        let mut editor = DocumentEditor::from_bytes(document.source_bytes.clone())?;
        let result = self.convert_with_editor(&mut editor)?;
        // Commit staged changes and propagate the updated document to the caller.
        editor.commit_in_place()?;
        *document = editor.into_source();
        Ok(result)
    }

    /// Core conversion logic operating on a [`DocumentEditor`].
    fn convert_with_editor(&self, editor: &mut DocumentEditor) -> Result<ConversionResult> {
        use std::collections::HashSet;

        let mut result = ConversionResult::new(self.level);
        let initial_validation = self.validator.validate(editor.source_mut(), self.level)?;

        if initial_validation.is_compliant {
            result.success = true;
            result.validation = initial_validation;
            return Ok(result);
        }

        // Apply each fix once even if the same ErrorCode appears multiple times.
        let mut applied: HashSet<ErrorCode> = HashSet::new();
        for error in &initial_validation.errors {
            self.try_fix_error(editor, error, &mut result, &mut applied)?;
        }

        // OutputIntents are required unconditionally for PDF/A-2/3.
        // The validator only reports DeviceColorWithoutIntent when it recognises
        // explicit RGB/CMYK operators, but many PDFs embed colour via images or
        // external graphics states that our content parser doesn't inspect.
        // Call add_output_intent always; it no-ops if OutputIntents already exists.
        if !applied.contains(&ErrorCode::MissingOutputIntent)
            && !applied.contains(&ErrorCode::DeviceColorWithoutIntent)
        {
            self.add_output_intent(editor, &mut result)?;
        }

        // Commit staged mutations so the validator reads the updated document.
        editor.commit_in_place()?;
        let mut final_validation = self.validator.validate(editor.source_mut(), self.level)?;

        // Issue #451: certain standard-14 PostScript fonts have no open-source
        // equivalent in URW Base35 / SYSTEM_FONTDB and therefore can't be
        // embedded by `embed_font`. Treat them as known limitations rather
        // than hard errors — move the matching `FontNotEmbedded` entries
        // from `errors` to `warnings` so a converted PDF that's otherwise
        // compliant doesn't get marked as failed solely because of `Symbol`.
        downgrade_known_unembeddable_fonts(&mut final_validation);

        result.validation = final_validation.clone();
        result.success = final_validation.is_compliant;

        Ok(result)
    }

    /// Try to fix a compliance error.
    fn try_fix_error(
        &self,
        editor: &mut DocumentEditor,
        error: &ComplianceError,
        result: &mut ConversionResult,
        applied: &mut std::collections::HashSet<ErrorCode>,
    ) -> Result<()> {
        // FontNotEmbedded is location-specific (each font is a separate error);
        // all other codes are document-level and only need to be applied once.
        if error.code != ErrorCode::FontNotEmbedded && !applied.insert(error.code) {
            return Ok(());
        }
        match error.code {
            ErrorCode::MissingXmpMetadata => {
                self.add_xmp_metadata(editor, result)?;
            },
            ErrorCode::MissingPdfaIdentification => {
                self.add_pdfa_identification(editor, result)?;
            },
            ErrorCode::FontNotEmbedded => {
                if self.config.embed_fonts {
                    self.embed_font(editor, error, result)?;
                } else {
                    result.add_error(ConversionError::new(
                        error.code,
                        "Font embedding disabled in configuration",
                    ));
                }
            },
            ErrorCode::MissingOutputIntent => {
                self.add_output_intent(editor, result)?;
            },
            ErrorCode::DeviceColorWithoutIntent => {
                self.add_output_intent(editor, result)?;
            },
            ErrorCode::JavaScriptNotAllowed => {
                if self.config.remove_javascript {
                    self.remove_javascript(editor, result)?;
                } else {
                    result.add_error(ConversionError::new(
                        error.code,
                        "JavaScript removal disabled in configuration",
                    ));
                }
            },
            ErrorCode::EncryptionNotAllowed => {
                if self.config.remove_encryption {
                    self.remove_encryption(editor, result)?;
                } else {
                    result.add_error(ConversionError::new(
                        error.code,
                        "Document is encrypted and encryption removal is disabled",
                    ));
                }
            },
            ErrorCode::TransparencyNotAllowed => {
                if self.config.flatten_transparency && !self.level.allows_transparency() {
                    self.flatten_transparency(editor, result)?;
                } else if !self.level.allows_transparency() {
                    result.add_error(ConversionError::new(
                        error.code,
                        "Transparency flattening disabled for PDF/A-1",
                    ));
                }
            },
            ErrorCode::EmbeddedFileNotAllowed => {
                if self.config.remove_embedded_files && !self.level.allows_embedded_files() {
                    self.remove_embedded_files(editor, result)?;
                } else if !self.level.allows_embedded_files() {
                    result.add_error(ConversionError::new(
                        error.code,
                        "Embedded file removal disabled",
                    ));
                }
            },
            ErrorCode::MissingDocumentStructure => {
                if self.config.add_structure && self.level.requires_structure() {
                    self.add_structure(editor, result)?;
                } else if self.level.requires_structure() {
                    result.add_error(ConversionError::new(
                        error.code,
                        "Structure tree generation not available; consider using PDF/A-*b level",
                    ));
                }
            },
            ErrorCode::MissingLanguage => {
                self.add_language(editor, result)?;
            },
            ErrorCode::MissingAppearanceStream => {
                self.fix_annotation_appearance(editor, error, result)?;
            },
            // Errors that cannot be automatically fixed
            ErrorCode::FontMissingTables
            | ErrorCode::FontInvalidEncoding
            | ErrorCode::FontMissingToUnicode
            | ErrorCode::InvalidIccProfile
            | ErrorCode::IccProfileVersionMismatch
            | ErrorCode::InvalidImageColorSpace
            | ErrorCode::UnsupportedImageCompression
            | ErrorCode::LzwCompressionNotAllowed
            | ErrorCode::InvalidStructureTree
            | ErrorCode::MultimediaNotAllowed
            | ErrorCode::ExternalContentNotAllowed
            | ErrorCode::InvalidAnnotation
            | ErrorCode::InvalidAction
            | ErrorCode::LaunchActionNotAllowed
            | ErrorCode::MissingAfRelationship
            | ErrorCode::PostScriptNotAllowed
            | ErrorCode::ReferenceXObjectNotAllowed
            | ErrorCode::OptionalContentIssue
            | ErrorCode::InvalidPdfaIdentification
            | ErrorCode::XmpMetadataMismatch => {
                result.add_error(ConversionError::new(
                    error.code,
                    format!("Cannot automatically fix: {}", error.message),
                ));
            },
        }

        Ok(())
    }

    /// Add an XMP metadata stream to the document catalog.
    ///
    /// Allocates a new stream object with `/Type /Metadata /Subtype /XML`
    /// and updates the catalog's `/Metadata` entry to reference it.
    /// Per ISO 19005-1 §6.7.11 the stream must not be compressed.
    fn add_xmp_metadata(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let xmp_bytes = self.generate_xmp_metadata().into_bytes();

        // Build the XMP stream — no /Filter, XMP must be plaintext per PDF/A spec.
        let mut stream_dict = std::collections::HashMap::new();
        stream_dict.insert("Type".to_string(), Object::Name("Metadata".to_string()));
        stream_dict.insert("Subtype".to_string(), Object::Name("XML".to_string()));
        stream_dict.insert("Length".to_string(), Object::Integer(xmp_bytes.len() as i64));
        let xmp_stream = Object::Stream {
            dict: stream_dict,
            data: crate::import_limits::StreamData::from(xmp_bytes),
        };

        let xmp_id = editor.alloc_id();
        editor.insert_modified(xmp_id, xmp_stream);

        // Patch the catalog to reference the new XMP stream.
        let mut catalog = load_catalog_for_edit(editor)?;
        catalog.insert("Metadata".to_string(), Object::Reference(ObjectRef::new(xmp_id, 0)));
        let catalog_id = catalog_object_id(editor)?;
        editor.insert_modified(catalog_id, Object::Dictionary(catalog));

        result.add_action(
            ConversionAction::new(ActionType::AddedXmpMetadata, "Added XMP metadata stream")
                .with_fixed_error(ErrorCode::MissingXmpMetadata),
        );
        Ok(())
    }

    /// Inject PDF/A identification into an existing XMP metadata stream.
    ///
    /// If the catalog already has a `/Metadata` stream, the `pdfaid:part` and
    /// `pdfaid:conformance` nodes are spliced in before `</rdf:RDF>`.  If the
    /// stream is absent or unreadable, falls back to [`add_xmp_metadata`].
    fn add_pdfa_identification(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let catalog = load_catalog_for_edit(editor)?;

        let metadata_ref = match catalog.get("Metadata").and_then(|o| o.as_reference()) {
            Some(r) => r,
            None => return self.add_xmp_metadata(editor, result),
        };

        // Prefer a version already staged in this conversion run.
        let current = editor
            .get_modified(metadata_ref.id)
            .cloned()
            .or_else(|| editor.source().load_object(metadata_ref).ok());

        // Decode the stream (may be FlateDecode-compressed) before parsing as XML.
        let current_xml = match &current {
            Some(obj @ Object::Stream { .. }) => {
                let decoded = obj.decode_stream_data().unwrap_or_else(|_| {
                    if let Object::Stream { data, .. } = obj {
                        data.to_vec()
                    } else {
                        vec![]
                    }
                });
                String::from_utf8_lossy(&decoded).into_owned()
            },
            Some(_) | None => return self.add_xmp_metadata(editor, result),
        };
        let mut stream_dict = match current {
            Some(Object::Stream { dict, .. }) => dict,
            _ => unreachable!(),
        };

        let patched =
            inject_pdfaid(&current_xml, self.level.xmp_part(), self.level.xmp_conformance());
        // inject_pdfaid returns the input unchanged when </rdf:RDF> is absent.
        // In that case, fall back to rebuilding XMP from scratch.
        if patched == current_xml {
            return self.add_xmp_metadata(editor, result);
        }
        let patched_bytes = patched.into_bytes();
        stream_dict.insert("Length".to_string(), Object::Integer(patched_bytes.len() as i64));
        // Remove any compression filter — PDF/A requires plaintext XMP.
        stream_dict.remove("Filter");
        stream_dict.remove("DecodeParms");

        editor.insert_modified(
            metadata_ref.id,
            Object::Stream {
                dict: stream_dict,
                data: crate::import_limits::StreamData::from(patched_bytes),
            },
        );

        result.add_action(
            ConversionAction::new(
                ActionType::AddedPdfaIdentification,
                format!(
                    "Added PDF/A-{}{} identification",
                    self.level.xmp_part(),
                    self.level.xmp_conformance().to_lowercase()
                ),
            )
            .with_fixed_error(ErrorCode::MissingPdfaIdentification),
        );
        Ok(())
    }

    /// Embed a missing font by loading it from the system font database.
    ///
    /// Walks every page's /Resources/Font dict to find the font object whose /BaseFont
    /// matches `error.location`, then loads the matching face from fontdb and writes a
    /// FontFile2 stream + updated FontDescriptor back to the editor.
    #[cfg(feature = "rendering")]
    fn embed_font(
        &self,
        editor: &mut DocumentEditor,
        error: &ComplianceError,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let font_name = error.location.as_deref().unwrap_or("Unknown").to_string();
        let font_objects = collect_font_objects_by_name(editor.source_mut(), &font_name)?;
        if font_objects.is_empty() {
            result.add_error(ConversionError::new(
                ErrorCode::FontNotEmbedded,
                format!("Font '{}' not found in document resources", font_name),
            ));
            return Ok(());
        }

        // Load font bytes from the system font database.
        let font_bytes = match load_system_font_bytes(&font_name) {
            Some(b) => b,
            None => {
                result.add_error(ConversionError::new(
                    ErrorCode::FontNotEmbedded,
                    format!(
                        "Font '{}' not found in system fonts; install the font to embed it",
                        font_name
                    ),
                ));
                return Ok(());
            },
        };

        // Write the font file as a FontFile2 stream.
        let mut ff_dict = std::collections::HashMap::new();
        ff_dict.insert("Length".to_string(), Object::Integer(font_bytes.len() as i64));
        let ff_id = editor.alloc_id();
        editor.insert_modified(
            ff_id,
            Object::Stream {
                dict: ff_dict,
                data: crate::import_limits::StreamData::from(font_bytes),
            },
        );

        // Update every matching font object to reference a FontDescriptor with FontFile2.
        for (font_id, mut font_dict) in font_objects {
            // Build or update the FontDescriptor.
            let desc_id = match font_dict
                .get("FontDescriptor")
                .and_then(|o| o.as_reference())
            {
                Some(r) => {
                    // Load existing descriptor and add FontFile2 to it.
                    if let Ok(existing) = editor.source().load_object(r) {
                        if let Some(mut d) = existing.as_dict().cloned() {
                            d.insert(
                                "FontFile2".to_string(),
                                Object::Reference(ObjectRef::new(ff_id, 0)),
                            );
                            editor.insert_modified(r.id, Object::Dictionary(d));
                            r.id
                        } else {
                            build_font_descriptor(editor, &font_dict, ff_id)
                        }
                    } else {
                        build_font_descriptor(editor, &font_dict, ff_id)
                    }
                },
                None => build_font_descriptor(editor, &font_dict, ff_id),
            };
            font_dict.insert(
                "FontDescriptor".to_string(),
                Object::Reference(ObjectRef::new(desc_id, 0)),
            );
            editor.insert_modified(font_id, Object::Dictionary(font_dict));
        }

        result.add_action(
            ConversionAction::new(
                ActionType::EmbeddedFont,
                format!("Embedded system font '{}' as FontFile2 stream", font_name),
            )
            .with_fixed_error(ErrorCode::FontNotEmbedded),
        );
        Ok(())
    }

    /// Font embedding requires the `rendering` feature (provides fontdb).
    #[cfg(not(feature = "rendering"))]
    fn embed_font(
        &self,
        _editor: &mut DocumentEditor,
        error: &ComplianceError,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let font_name = error.location.as_deref().unwrap_or("Unknown");
        result.add_error(ConversionError::new(
            ErrorCode::FontNotEmbedded,
            format!(
                "Font '{}' cannot be embedded without the `rendering` feature \
                 (rebuild with --features rendering)",
                font_name
            ),
        ));
        Ok(())
    }

    /// Write a real /OutputIntents array with an embedded sRGB ICC profile stream.
    fn add_output_intent(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let mut catalog = load_catalog_for_edit(editor)?;
        if catalog.contains_key("OutputIntents") {
            return Ok(());
        }

        let icc_bytes: Vec<u8> = self
            .config
            .icc_profile
            .clone()
            .unwrap_or_else(|| Self::get_srgb_icc_profile().to_vec());

        // Build the ICC profile stream — uncompressed; PDF/A allows but does not
        // require compression here and plaintext is simplest for validation.
        let mut icc_dict = std::collections::HashMap::new();
        icc_dict.insert("N".to_string(), Object::Integer(icc_channel_count(&icc_bytes) as i64));
        icc_dict.insert("Length".to_string(), Object::Integer(icc_bytes.len() as i64));
        let icc_stream = Object::Stream {
            dict: icc_dict,
            data: crate::import_limits::StreamData::from(icc_bytes),
        };
        let icc_id = editor.alloc_id();
        editor.insert_modified(icc_id, icc_stream);

        // Build the OutputIntent dictionary.
        let mut oi = std::collections::HashMap::new();
        oi.insert("Type".to_string(), Object::Name("OutputIntent".to_string()));
        oi.insert("S".to_string(), Object::Name("GTS_PDFA1".to_string()));
        oi.insert(
            "OutputConditionIdentifier".to_string(),
            Object::text_string("sRGB IEC61966-2.1"),
        );
        oi.insert("RegistryName".to_string(), Object::text_string("http://www.color.org"));
        oi.insert("Info".to_string(), Object::text_string("sRGB IEC61966-2.1"));
        oi.insert("DestOutputProfile".to_string(), Object::Reference(ObjectRef::new(icc_id, 0)));

        catalog.insert("OutputIntents".to_string(), Object::Array(vec![Object::Dictionary(oi)]));
        let cat_id = catalog_object_id(editor)?;
        editor.insert_modified(cat_id, Object::Dictionary(catalog));

        result.add_action(
            ConversionAction::new(
                ActionType::AddedOutputIntent,
                "Added sRGB OutputIntent (GTS_PDFA1) with embedded ICC profile",
            )
            .with_fixed_error(ErrorCode::MissingOutputIntent),
        );
        Ok(())
    }

    /// Remove JavaScript from /Names/JavaScript, /OpenAction (if JS), and /AA entries.
    fn remove_javascript(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let mut touched = false;

        // 1. /Names /JavaScript tree
        if let Some((mut names, names_id)) = load_names_for_edit(editor)? {
            if names.remove("JavaScript").is_some() {
                store_names(editor, names, names_id)?;
                touched = true;
            }
        }

        // 2. Catalog /OpenAction if it is a JavaScript action.
        let mut catalog = load_catalog_for_edit(editor)?;
        let mut catalog_changed = false;
        if action_is_javascript(catalog.get("OpenAction"), editor) {
            catalog.remove("OpenAction");
            catalog_changed = true;
        }
        if let Some(aa_obj) = catalog.get("AA").cloned() {
            if let Some(cleaned) = strip_js_from_aa(&aa_obj, editor) {
                if cleaned.is_empty() {
                    catalog.remove("AA");
                } else {
                    catalog.insert("AA".to_string(), Object::Dictionary(cleaned));
                }
                catalog_changed = true;
            }
        }
        if catalog_changed {
            let cat_id = catalog_object_id(editor)?;
            editor.insert_modified(cat_id, Object::Dictionary(catalog));
            touched = true;
        }

        // 3. Per-page /AA entries.
        // Single tree walk; per-index get_page_ref in a loop is O(n²).
        let page_refs = editor.source().all_page_refs().unwrap_or_default();
        for page_ref in page_refs {
            let page_obj = match editor.source().load_object(page_ref) {
                Ok(o) => o,
                Err(_) => continue,
            };
            if let Object::Dictionary(mut page_dict) = page_obj {
                if let Some(aa) = page_dict.get("AA").cloned() {
                    if let Some(cleaned) = strip_js_from_aa(&aa, editor) {
                        if cleaned.is_empty() {
                            page_dict.remove("AA");
                        } else {
                            page_dict.insert("AA".to_string(), Object::Dictionary(cleaned));
                        }
                        editor.insert_modified(page_ref.id, Object::Dictionary(page_dict));
                        touched = true;
                    }
                }
            }
        }

        if touched {
            result.add_action(
                ConversionAction::new(
                    ActionType::RemovedJavaScript,
                    "Removed JavaScript from /Names, /OpenAction, and /AA entries",
                )
                .with_fixed_error(ErrorCode::JavaScriptNotAllowed),
            );
        }
        Ok(())
    }

    /// Strip the /Encrypt marker from the trailer.
    ///
    /// Safe only when content streams are already accessible — the editor's
    /// `write_full_to_writer` never writes /Encrypt unless `SaveOptions::encryption`
    /// is set, so converted output will be unencrypted as long as the source bytes
    /// were readable (i.e. no password required).
    fn remove_encryption(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        // Probe: if we can read page content the source was decrypted.
        if editor.source_mut().get_page_content_data(0).is_err() {
            result.add_error(ConversionError::new(
                ErrorCode::EncryptionNotAllowed,
                "Document content is inaccessible; cannot strip encryption safely",
            ));
            return Ok(());
        }
        // The editor's save path already omits /Encrypt from the new trailer
        // (write_full_to_writer only adds /Encrypt when SaveOptions::encryption is set).
        // No additional mutation is required — just record the action.
        result.add_action(
            ConversionAction::new(
                ActionType::RemovedEncryption,
                "Encryption marker will be absent from the saved output",
            )
            .with_fixed_error(ErrorCode::EncryptionNotAllowed),
        );
        Ok(())
    }

    /// Flatten transparency by re-rendering every page to a raster image via tiny-skia.
    #[cfg(feature = "rendering")]
    fn flatten_transparency(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let flat_bytes = crate::rendering::flatten_to_images(editor.source(), 144)?;
        editor.replace_source_bytes(flat_bytes)?;
        result.add_action(
            ConversionAction::new(
                ActionType::FlattenedTransparency,
                "Re-rendered all pages at 144 dpi to eliminate transparency",
            )
            .with_fixed_error(ErrorCode::TransparencyNotAllowed),
        );
        Ok(())
    }

    /// Transparency flattening requires the `rendering` feature.
    #[cfg(not(feature = "rendering"))]
    fn flatten_transparency(
        &self,
        _editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        result.add_error(ConversionError::new(
            ErrorCode::TransparencyNotAllowed,
            "Transparency flattening requires the `rendering` feature; use PDF/A-2b or -3b \
             (which allow transparency) or rebuild with --features rendering",
        ));
        Ok(())
    }

    /// Remove /EmbeddedFiles from the /Names dictionary.
    fn remove_embedded_files(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let removed = if let Some((mut names, names_id)) = load_names_for_edit(editor)? {
            if names.remove("EmbeddedFiles").is_some() {
                store_names(editor, names, names_id)?;
                true
            } else {
                false
            }
        } else {
            false
        };

        if removed {
            result.add_action(
                ConversionAction::new(
                    ActionType::RemovedEmbeddedFiles,
                    "Removed /EmbeddedFiles from /Names",
                )
                .with_fixed_error(ErrorCode::EmbeddedFileNotAllowed),
            );
        }
        Ok(())
    }

    /// Add a minimal StructTreeRoot skeleton and /MarkInfo /Marked true to satisfy
    /// the PDF/A-*a Tagged PDF requirements (ISO 19005-2 §6.8).
    fn add_structure(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let mut catalog = load_catalog_for_edit(editor)?;

        // Check whether both required entries already exist.
        let has_struct = catalog.contains_key("StructTreeRoot");
        let has_markinfo = catalog
            .get("MarkInfo")
            .and_then(|o| o.as_dict())
            .map(|d| matches!(d.get("Marked"), Some(Object::Boolean(true))))
            .unwrap_or(false);

        if has_struct && has_markinfo {
            return Ok(());
        }

        // /MarkInfo << /Marked true >> — tells viewers this is a tagged PDF.
        if !has_markinfo {
            let mut mark = std::collections::HashMap::new();
            mark.insert("Marked".to_string(), Object::Boolean(true));
            catalog.insert("MarkInfo".to_string(), Object::Dictionary(mark));
        }

        // /StructTreeRoot — minimal skeleton with empty /K children array.
        if !has_struct {
            let mut root_dict = std::collections::HashMap::new();
            root_dict.insert("Type".to_string(), Object::Name("StructTreeRoot".to_string()));
            root_dict.insert("K".to_string(), Object::Array(vec![]));
            let root_id = editor.alloc_id();
            editor.insert_modified(root_id, Object::Dictionary(root_dict));
            catalog.insert(
                "StructTreeRoot".to_string(),
                Object::Reference(ObjectRef::new(root_id, 0)),
            );
        }

        let cat_id = catalog_object_id(editor)?;
        editor.insert_modified(cat_id, Object::Dictionary(catalog));
        result.add_action(
            ConversionAction::new(
                ActionType::AddedStructure,
                "Added /MarkInfo /Marked true and minimal /StructTreeRoot for PDF/A-*a compliance",
            )
            .with_fixed_error(ErrorCode::MissingDocumentStructure),
        );
        Ok(())
    }

    /// Set /Lang on the catalog (default "en").
    fn add_language(
        &self,
        editor: &mut DocumentEditor,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let mut catalog = load_catalog_for_edit(editor)?;
        if !catalog.contains_key("Lang") {
            catalog.insert("Lang".to_string(), Object::text_string("en"));
            let cat_id = catalog_object_id(editor)?;
            editor.insert_modified(cat_id, Object::Dictionary(catalog));
        }
        result.add_action(
            ConversionAction::new(ActionType::AddedLanguage, "Set catalog /Lang to 'en'")
                .with_fixed_error(ErrorCode::MissingLanguage),
        );
        Ok(())
    }

    /// Synthesise a raster /AP N stream for every annotation that lacks one.
    ///
    /// Renders the annotation's bounding rect on its page via the tiny-skia engine,
    /// wraps the PNG pixels as a raw-RGB Image XObject, then embeds it inside a
    /// Form XObject that becomes the /AP /N entry.  All annotations on all pages are
    /// processed in a single call (the `applied` dedup ensures we only reach here once
    /// per `ErrorCode::MissingAppearanceStream`).
    #[cfg(feature = "rendering")]
    fn fix_annotation_appearance(
        &self,
        editor: &mut DocumentEditor,
        _error: &ComplianceError,
        result: &mut ConversionResult,
    ) -> Result<()> {
        use crate::rendering::{render_page_region, RenderOptions};

        let page_count = editor.source_mut().page_count()?;
        let mut fixed = 0usize;

        for page_idx in 0..page_count {
            // Collect annotations that are missing an /AP /N entry.
            let page_ref = match editor.source().get_page_ref(page_idx) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let page_obj = match editor.source().load_object(page_ref) {
                Ok(o) => o,
                Err(_) => continue,
            };
            let annots_arr = match page_obj.as_dict().and_then(|d| d.get("Annots")).cloned() {
                Some(a) => match editor.source().resolve_references(&a, 1) {
                    Ok(o) => o,
                    Err(_) => continue,
                },
                None => continue,
            };
            let annots = match annots_arr.as_array() {
                Some(a) => a.clone(),
                None => continue,
            };

            for annot_ref_obj in annots {
                let annot_ref = match annot_ref_obj.as_reference() {
                    Some(r) => r,
                    None => continue,
                };
                let annot_obj = match editor.source().load_object(annot_ref) {
                    Ok(o) => o,
                    Err(_) => continue,
                };
                let annot_dict = match annot_obj.as_dict() {
                    Some(d) => d.clone(),
                    None => continue,
                };

                // Skip if /AP /N already present.
                if let Some(ap) = annot_dict.get("AP") {
                    if let Ok(ap_resolved) = editor.source().resolve_references(ap, 1) {
                        if ap_resolved.as_dict().and_then(|d| d.get("N")).is_some() {
                            continue;
                        }
                    }
                }

                // Parse /Rect.
                let rect = match annot_dict.get("Rect").and_then(|r| r.as_array()) {
                    Some(arr) if arr.len() == 4 => {
                        let nums: Vec<f32> = arr
                            .iter()
                            .filter_map(|o| {
                                o.as_real()
                                    .map(|r| r as f32)
                                    .or_else(|| o.as_integer().map(|i| i as f32))
                            })
                            .collect();
                        if nums.len() == 4 {
                            (nums[0], nums[1], nums[2] - nums[0], nums[3] - nums[1])
                        } else {
                            continue;
                        }
                    },
                    _ => continue,
                };
                let (x, y, w, h) = rect;
                if w <= 0.0 || h <= 0.0 {
                    continue;
                }

                // Render the annotation region.
                let opts = RenderOptions::with_dpi(144);
                let rendered =
                    match render_page_region(editor.source(), page_idx, (x, y, w, h), &opts) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                // Decode PNG to raw RGB bytes so we can embed as a /ColorSpace /DeviceRGB image.
                let img = match image::load_from_memory(&rendered.data) {
                    Ok(i) => i.to_rgb8(),
                    Err(_) => continue,
                };
                let img_w = img.width();
                let img_h = img.height();
                let raw_rgb: Vec<u8> = img.into_raw();

                // Image XObject.
                let mut img_dict = std::collections::HashMap::new();
                img_dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
                img_dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
                img_dict.insert("Width".to_string(), Object::Integer(img_w as i64));
                img_dict.insert("Height".to_string(), Object::Integer(img_h as i64));
                img_dict.insert("ColorSpace".to_string(), Object::Name("DeviceRGB".to_string()));
                img_dict.insert("BitsPerComponent".to_string(), Object::Integer(8));
                img_dict.insert("Length".to_string(), Object::Integer(raw_rgb.len() as i64));
                let img_id = editor.alloc_id();
                editor.insert_modified(
                    img_id,
                    Object::Stream {
                        dict: img_dict,
                        data: crate::import_limits::StreamData::from(raw_rgb),
                    },
                );

                // Form XObject: content stream "q W 0 0 H cm /Im Do Q".
                let content = format!("q {} 0 0 {} 0 0 cm /Im Do Q", img_w, img_h);
                let mut res_dict = std::collections::HashMap::new();
                let mut xobj_dict = std::collections::HashMap::new();
                xobj_dict.insert("Im".to_string(), Object::Reference(ObjectRef::new(img_id, 0)));
                res_dict.insert("XObject".to_string(), Object::Dictionary(xobj_dict));
                let mut form_dict = std::collections::HashMap::new();
                form_dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
                form_dict.insert("Subtype".to_string(), Object::Name("Form".to_string()));
                form_dict.insert(
                    "BBox".to_string(),
                    Object::Array(vec![
                        Object::Integer(0),
                        Object::Integer(0),
                        Object::Integer(img_w as i64),
                        Object::Integer(img_h as i64),
                    ]),
                );
                form_dict.insert("Resources".to_string(), Object::Dictionary(res_dict));
                form_dict.insert("Length".to_string(), Object::Integer(content.len() as i64));
                let form_id = editor.alloc_id();
                editor.insert_modified(
                    form_id,
                    Object::Stream {
                        dict: form_dict,
                        data: crate::import_limits::StreamData::from(content.into_bytes()),
                    },
                );

                // /AP dict pointing to the Form XObject.
                let mut ap_dict = std::collections::HashMap::new();
                ap_dict.insert("N".to_string(), Object::Reference(ObjectRef::new(form_id, 0)));
                let mut updated_annot = annot_dict;
                updated_annot.insert("AP".to_string(), Object::Dictionary(ap_dict));
                editor.insert_modified(annot_ref.id, Object::Dictionary(updated_annot));
                fixed += 1;
            }
        }

        if fixed > 0 {
            result.add_action(
                ConversionAction::new(
                    ActionType::FixedAnnotation,
                    format!("Generated raster /AP N streams for {} annotation(s)", fixed),
                )
                .with_fixed_error(ErrorCode::MissingAppearanceStream),
            );
        }
        Ok(())
    }

    /// Annotation appearance generation requires the `rendering` feature.
    #[cfg(not(feature = "rendering"))]
    fn fix_annotation_appearance(
        &self,
        _editor: &mut DocumentEditor,
        error: &ComplianceError,
        result: &mut ConversionResult,
    ) -> Result<()> {
        let location = error.location.as_deref().unwrap_or("annotation");
        result.add_error(ConversionError::new(
            ErrorCode::MissingAppearanceStream,
            format!(
                "Cannot generate appearance stream for {} — rebuild with \
                 --features rendering to enable annotation appearance synthesis",
                location
            ),
        ));
        Ok(())
    }

    /// Generate XMP metadata for PDF/A.
    fn generate_xmp_metadata(&self) -> String {
        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:pdf="http://ns.adobe.com/pdf/1.3/"
        xmlns:xmp="http://ns.adobe.com/xap/1.0/"
        xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
      <pdfaid:part>{}</pdfaid:part>
      <pdfaid:conformance>{}</pdfaid:conformance>
      <dc:format>application/pdf</dc:format>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
            self.level.xmp_part(),
            self.level.xmp_conformance()
        )
    }

    /// Get the sRGB ICC profile data.
    pub fn get_srgb_icc_profile() -> &'static [u8] {
        // Minimal sRGB ICC profile header (would be replaced with full profile)
        // In a production implementation, this would be the complete sRGB profile
        include_bytes!("srgb_profile_placeholder.bin")
    }
}

/// Resolve the catalog's object ID from the trailer `/Root` entry.
fn catalog_object_id(editor: &DocumentEditor) -> Result<u32> {
    use crate::error::Error;
    editor
        .source()
        .trailer()
        .as_dict()
        .and_then(|d| d.get("Root"))
        .and_then(|r| r.as_reference())
        .map(|r| r.id)
        .ok_or_else(|| Error::InvalidPdf("Trailer missing /Root reference".to_string()))
}

/// Load the catalog dictionary for editing.
///
/// Prefers any already-staged version in `editor.modified_objects` so that
/// multiple fix methods in the same conversion run compose correctly.
fn load_catalog_for_edit(
    editor: &mut DocumentEditor,
) -> Result<std::collections::HashMap<String, Object>> {
    use crate::error::Error;
    let catalog_id = catalog_object_id(editor)?;
    if let Some(staged) = editor.get_modified(catalog_id) {
        if let Some(d) = staged.as_dict() {
            return Ok(d.clone());
        }
    }
    editor
        .source()
        .load_object(ObjectRef::new(catalog_id, 0))?
        .as_dict()
        .cloned()
        .ok_or_else(|| Error::InvalidPdf("Catalog is not a dictionary".to_string()))
}

/// Load the /Names dictionary for editing, returning `(dict, indirect_id)`.
///
/// `indirect_id == 0` signals the Names dict is inline in the catalog; callers
/// must use `store_names` which routes to `load_catalog_for_edit` in that case.
fn load_names_for_edit(
    editor: &mut DocumentEditor,
) -> Result<Option<(std::collections::HashMap<String, Object>, u32)>> {
    let catalog = load_catalog_for_edit(editor)?;
    let names_obj = match catalog.get("Names") {
        Some(o) => o.clone(),
        None => return Ok(None),
    };
    match names_obj {
        Object::Reference(r) => {
            let id = r.id;
            if let Some(staged) = editor.get_modified(id) {
                if let Some(d) = staged.as_dict() {
                    return Ok(Some((d.clone(), id)));
                }
            }
            match editor.source().load_object(r) {
                Ok(Object::Dictionary(d)) => Ok(Some((d, id))),
                _ => Ok(None),
            }
        },
        Object::Dictionary(d) => Ok(Some((d, 0))),
        _ => Ok(None),
    }
}

/// Persist a (possibly modified) Names dict back to the editor.
///
/// When `names_id == 0` the dict was inline; it is re-embedded into the catalog.
/// When the dict is empty after removals the /Names entry is dropped entirely.
fn store_names(
    editor: &mut DocumentEditor,
    names: std::collections::HashMap<String, Object>,
    names_id: u32,
) -> Result<()> {
    if names_id == 0 {
        let mut catalog = load_catalog_for_edit(editor)?;
        if names.is_empty() {
            catalog.remove("Names");
        } else {
            catalog.insert("Names".to_string(), Object::Dictionary(names));
        }
        let cat_id = catalog_object_id(editor)?;
        editor.insert_modified(cat_id, Object::Dictionary(catalog));
    } else if names.is_empty() {
        // Remove the /Names reference from catalog.
        let mut catalog = load_catalog_for_edit(editor)?;
        catalog.remove("Names");
        let cat_id = catalog_object_id(editor)?;
        editor.insert_modified(cat_id, Object::Dictionary(catalog));
    } else {
        editor.insert_modified(names_id, Object::Dictionary(names));
    }
    Ok(())
}

/// Returns true if the given object (direct or reference) is a JavaScript action.
fn action_is_javascript(obj: Option<&Object>, editor: &DocumentEditor) -> bool {
    let Some(action) = obj else { return false };
    let resolved = match action {
        Object::Reference(r) => editor.source().load_object(*r).unwrap_or(Object::Null),
        other => other.clone(),
    };
    matches!(
        resolved
            .as_dict()
            .and_then(|d| d.get("S"))
            .and_then(|o| o.as_name()),
        Some("JavaScript")
    )
}

/// Remove all /AA sub-actions whose /S is /JavaScript.
///
/// Returns `Some(cleaned_dict)` if anything was removed, `None` if unchanged.
fn strip_js_from_aa(
    aa: &Object,
    editor: &DocumentEditor,
) -> Option<std::collections::HashMap<String, Object>> {
    let aa_dict = match aa {
        Object::Reference(r) => editor.source().load_object(*r).ok()?.as_dict()?.clone(),
        Object::Dictionary(d) => d.clone(),
        _ => return None,
    };
    let before = aa_dict.len();
    let mut out = aa_dict;
    out.retain(|_, action_obj| {
        let resolved = match action_obj {
            Object::Reference(r) => editor.source().load_object(*r).unwrap_or(Object::Null),
            other => other.clone(),
        };
        !matches!(
            resolved
                .as_dict()
                .and_then(|d| d.get("S"))
                .and_then(|o| o.as_name()),
            Some("JavaScript")
        )
    });
    if out.len() != before {
        Some(out)
    } else {
        None
    }
}

/// Walk every page's /Resources/Font dict and return `(object_id, font_dict)` for
/// each indirect font object whose /BaseFont matches `target`.
///
/// Used by `embed_font()` to locate exactly the objects that need a FontFile2 stream.
#[cfg(feature = "rendering")]
fn collect_font_objects_by_name(
    doc: &mut PdfDocument,
    target: &str,
) -> Result<Vec<(u32, std::collections::HashMap<String, Object>)>> {
    let page_count = doc.page_count()?;
    let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut found: Vec<(u32, std::collections::HashMap<String, Object>)> = Vec::new();

    for idx in 0..page_count {
        let page_ref = match doc.get_page_ref(idx) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let page_obj = match doc.load_object(page_ref) {
            Ok(o) => o,
            Err(_) => continue,
        };
        let resources = match page_obj.as_dict().and_then(|d| d.get("Resources")).cloned() {
            Some(r) => match doc.resolve_references(&r, 2) {
                Ok(o) => o,
                Err(_) => continue,
            },
            None => continue,
        };
        let font_map = match resources.as_dict().and_then(|d| d.get("Font")).cloned() {
            Some(f) => match doc.resolve_references(&f, 1) {
                Ok(o) => o,
                Err(_) => continue,
            },
            None => continue,
        };
        let fonts = match font_map.as_dict() {
            Some(d) => d.clone(),
            None => continue,
        };
        for font_obj in fonts.values() {
            let font_ref = match font_obj.as_reference() {
                Some(r) => r,
                None => continue,
            };
            if !seen.insert(font_ref.id) {
                continue;
            }
            let resolved = match doc.load_object(font_ref) {
                Ok(o) => o,
                Err(_) => continue,
            };
            let fd = match resolved.as_dict() {
                Some(d) => d.clone(),
                None => continue,
            };
            let base_font = fd.get("BaseFont").and_then(|o| o.as_name()).unwrap_or("");
            if base_font == target {
                found.push((font_ref.id, fd));
            }
        }
    }
    Ok(found)
}

/// Map the 14 standard PDF Type1 PostScript names to open-source system font
/// families. The URW Base 35 collection ships by default on most Linux
/// distributions and is metrically equivalent to the Adobe originals.
#[cfg(feature = "rendering")]
fn std14_alias(ps_name: &str) -> Option<(&'static str, fontdb::Weight, fontdb::Style)> {
    match ps_name {
        // Helvetica family → Nimbus Sans / Liberation Sans
        "Helvetica" => Some(("Nimbus Sans", fontdb::Weight::NORMAL, fontdb::Style::Normal)),
        "Helvetica-Bold" => Some(("Nimbus Sans", fontdb::Weight::BOLD, fontdb::Style::Normal)),
        "Helvetica-Oblique" => Some(("Nimbus Sans", fontdb::Weight::NORMAL, fontdb::Style::Italic)),
        "Helvetica-BoldOblique" => {
            Some(("Nimbus Sans", fontdb::Weight::BOLD, fontdb::Style::Italic))
        },
        // Times family → Nimbus Roman / C059
        "Times-Roman" => Some(("Nimbus Roman", fontdb::Weight::NORMAL, fontdb::Style::Normal)),
        "Times-Bold" => Some(("Nimbus Roman", fontdb::Weight::BOLD, fontdb::Style::Normal)),
        "Times-Italic" => Some(("Nimbus Roman", fontdb::Weight::NORMAL, fontdb::Style::Italic)),
        "Times-BoldItalic" => Some(("Nimbus Roman", fontdb::Weight::BOLD, fontdb::Style::Italic)),
        // Courier family → Nimbus Mono PS / Liberation Mono
        "Courier" => Some(("Nimbus Mono PS", fontdb::Weight::NORMAL, fontdb::Style::Normal)),
        "Courier-Bold" => Some(("Nimbus Mono PS", fontdb::Weight::BOLD, fontdb::Style::Normal)),
        "Courier-Oblique" => {
            Some(("Nimbus Mono PS", fontdb::Weight::NORMAL, fontdb::Style::Italic))
        },
        "Courier-BoldOblique" => {
            Some(("Nimbus Mono PS", fontdb::Weight::BOLD, fontdb::Style::Italic))
        },
        _ => None,
    }
}

/// Parse the number of colorant channels from an ICC profile header (bytes 16–19).
/// Falls back to 3 (sRGB) when the profile is too short or the color space is unknown.
fn icc_channel_count(icc: &[u8]) -> i32 {
    // ICC.1 §7.2.6: colour space field is a 4-byte ASCII string at offset 16.
    if icc.len() < 20 {
        return 3;
    }
    match &icc[16..20] {
        b"XYZ " | b"Lab " | b"RGB " | b"Luv " | b"YCbr" | b"Yxy " => 3,
        b"CMYK" => 4,
        b"GRAY" => 1,
        b"HSV " | b"HLS " => 3,
        b"CMY " => 3,
        b"2CLR" => 2,
        b"3CLR" => 3,
        b"4CLR" => 4,
        b"5CLR" => 5,
        b"6CLR" => 6,
        b"7CLR" => 7,
        b"8CLR" => 8,
        _ => 3,
    }
}

#[cfg(feature = "rendering")]
static CONVERTER_FONTDB: std::sync::OnceLock<std::sync::Arc<fontdb::Database>> =
    std::sync::OnceLock::new();

#[cfg(feature = "rendering")]
fn converter_fontdb() -> std::sync::Arc<fontdb::Database> {
    CONVERTER_FONTDB
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            std::sync::Arc::new(db)
        })
        .clone()
}

#[cfg(feature = "rendering")]
fn load_system_font_bytes(font_name: &str) -> Option<Vec<u8>> {
    let db = converter_fontdb();

    // Strip subset tag prefix "ABCDEF+" if present.
    let clean = {
        let s = font_name.trim_start_matches(|c: char| c.is_ascii_uppercase());
        s.strip_prefix('+').unwrap_or(font_name)
    };

    // Build candidate family names: try exact PS name first, then std14 alias,
    // then the base family split on '-'.
    let weight = if clean.contains("Bold") {
        fontdb::Weight::BOLD
    } else {
        fontdb::Weight::NORMAL
    };
    let style = if clean.contains("Italic") || clean.contains("Oblique") {
        fontdb::Style::Italic
    } else {
        fontdb::Style::Normal
    };

    // Collect candidate (family, weight, style) tuples in priority order.
    let mut candidates: Vec<(&str, fontdb::Weight, fontdb::Style)> = Vec::new();
    if let Some(alias) = std14_alias(clean) {
        candidates.push(alias);
        // Also try Liberation equivalents as fallback.
        let lib_family = match alias.0 {
            "Nimbus Sans" => Some("Liberation Sans"),
            "Nimbus Roman" => Some("Liberation Serif"),
            "Nimbus Mono PS" => Some("Liberation Mono"),
            _ => None,
        };
        if let Some(lf) = lib_family {
            candidates.push((lf, alias.1, alias.2));
        }
    }
    // Also try the name as-is and the base family (split on '-').
    candidates.push((clean, weight, style));
    let base_family = clean.split('-').next().unwrap_or(clean);
    if base_family != clean {
        candidates.push((base_family, weight, style));
    }

    for (family, w, s) in candidates {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: w,
            style: s,
            stretch: fontdb::Stretch::Normal,
        };
        if let Some(id) = db.query(&query) {
            let mut result: Option<Vec<u8>> = None;
            db.with_face_data(id, |data, _index| {
                result = Some(data.to_vec());
            });
            if result.is_some() {
                return result;
            }
        }
    }
    None
}

/// Build a new FontDescriptor dict pointing to `ff_id` as /FontFile2.
///
/// Reads as many metrics as possible from `font_dict` (/Ascent, /Descent, etc.)
/// and fills sensible defaults for anything missing.
#[cfg(feature = "rendering")]
fn build_font_descriptor(
    editor: &mut DocumentEditor,
    font_dict: &std::collections::HashMap<String, Object>,
    ff_id: u32,
) -> u32 {
    let base_font = font_dict
        .get("BaseFont")
        .and_then(|o| o.as_name())
        .unwrap_or("Unknown")
        .to_string();
    let mut d = std::collections::HashMap::new();
    d.insert("Type".to_string(), Object::Name("FontDescriptor".to_string()));
    d.insert("FontName".to_string(), Object::Name(base_font));
    d.insert("Flags".to_string(), Object::Integer(32)); // Nonsymbolic
    d.insert("ItalicAngle".to_string(), Object::Integer(0));
    d.insert("Ascent".to_string(), Object::Integer(800));
    d.insert("Descent".to_string(), Object::Integer(-200));
    d.insert("CapHeight".to_string(), Object::Integer(700));
    d.insert("StemV".to_string(), Object::Integer(80));
    d.insert(
        "FontBBox".to_string(),
        Object::Array(vec![
            Object::Integer(-100),
            Object::Integer(-200),
            Object::Integer(1000),
            Object::Integer(800),
        ]),
    );
    d.insert("FontFile2".to_string(), Object::Reference(ObjectRef::new(ff_id, 0)));
    let id = editor.alloc_id();
    editor.insert_modified(id, Object::Dictionary(d));
    id
}

/// Splice `pdfaid:part` and `pdfaid:conformance` into an existing XMP packet.
///
/// Inserts a fresh `<rdf:Description>` carrying the PDF/A identification
/// immediately before the closing `</rdf:RDF>` tag.  Multiple `rdf:Description`
/// elements in a single RDF block are legal per the RDF spec and used by all
/// major PDF/A toolkits.  String-level splicing avoids a heavyweight XML
/// dependency (KISS).
fn inject_pdfaid(xml: &str, part: &str, conformance: &str) -> String {
    let block = format!(
        "    <rdf:Description rdf:about=\"\" \
xmlns:pdfaid=\"http://www.aiim.org/pdfa/ns/id/\">\n\
      <pdfaid:part>{part}</pdfaid:part>\n\
      <pdfaid:conformance>{conformance}</pdfaid:conformance>\n\
    </rdf:Description>\n"
    );
    if let Some(idx) = xml.rfind("</rdf:RDF>") {
        let mut out = String::with_capacity(xml.len() + block.len());
        out.push_str(&xml[..idx]);
        out.push_str(&block);
        out.push_str(&xml[idx..]);
        out
    } else {
        // XMP has no RDF block — caller should have routed to add_xmp_metadata.
        xml.to_string()
    }
}

/// Quick conversion function for common use cases.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::compliance::{convert_to_pdf_a, PdfALevel};
///
/// let result = convert_to_pdf_a(&mut document, PdfALevel::A2b)?;
/// if result.success {
///     println!("Conversion successful");
/// }
/// ```
pub fn convert_to_pdf_a(document: &mut PdfDocument, level: PdfALevel) -> Result<ConversionResult> {
    PdfAConverter::new(level).convert(document)
}

/// Standard-14 PostScript font names that have no open-source equivalent
/// reachable from `SYSTEM_FONTDB` / URW Base35 and therefore cannot be
/// embedded by `embed_font`. Issue #451.
///
/// `Symbol` is the headline case — its glyph repertoire is symbolic
/// (Greek letters, math operators, arrows), not Latin, and URW++'s
/// closest analogue (`StandardSymbolsPS`) doesn't ship in the standard
/// `urw-base35-fonts` distribution. `ZapfDingbats` is the same shape of
/// problem and shows the same symptom on PDFs that use it.
const KNOWN_UNEMBEDDABLE_FONTS: &[&str] = &["Symbol", "ZapfDingbats"];

/// Move `FontNotEmbedded` errors whose location names a known-
/// unembeddable standard-14 font into the `warnings` vector, and flip
/// `is_compliant` if doing so leaves no remaining errors. The user
/// still sees that the font wasn't embedded — just not as a hard fail.
fn downgrade_known_unembeddable_fonts(validation: &mut crate::compliance::types::ValidationResult) {
    use crate::compliance::types::{ComplianceWarning, ErrorCode, WarningCode};

    let (downgraded, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut validation.errors)
        .into_iter()
        .partition(|e| {
            e.code == ErrorCode::FontNotEmbedded
                && e.location
                    .as_deref()
                    .is_some_and(is_known_unembeddable_font)
        });

    validation.errors = kept;
    for err in downgraded {
        let mut warning = ComplianceWarning::new(
            WarningCode::KnownUnembeddableFont,
            format!(
                "{} (downgraded from error per #451 — no open-source equivalent available)",
                err.message
            ),
        );
        if let Some(loc) = err.location {
            warning = warning.with_location(loc);
        }
        validation.warnings.push(warning);
    }

    if validation.errors.is_empty() {
        validation.is_compliant = true;
    }
}

/// Exact-match (with subset-prefix tolerance) check that a font's
/// PostScript name is one of the known-unembeddable Standard14 fonts.
///
/// PDF subset fonts use the `ABCDEF+FontName` convention (six tag chars,
/// plus sign, base name). We accept either the exact base name or a
/// `+BaseName` suffix; plain substring matching would mis-match
/// arbitrary fonts like `MySymbolFont` or `ZapfDingbatsITC`.
fn is_known_unembeddable_font(name: &str) -> bool {
    KNOWN_UNEMBEDDABLE_FONTS
        .iter()
        .any(|&base| name == base || name.split('+').next_back() == Some(base))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_config_default() {
        let config = ConversionConfig::default();
        assert!(config.embed_fonts);
        assert!(config.remove_javascript);
        assert!(config.remove_encryption);
        assert!(config.flatten_transparency);
    }

    #[test]
    fn downgrade_symbol_font_to_warning() {
        // Issue #451: a `FontNotEmbedded` error for the standard-14
        // `Symbol` font (which has no open-source equivalent) gets moved
        // from `errors` to `warnings`, and the validation flips to
        // compliant if it was the sole remaining error.
        use crate::compliance::types::{ComplianceError, ErrorCode, ValidationResult, WarningCode};

        let mut v = ValidationResult::new(PdfALevel::A1b);
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'Symbol' not embedded")
                .with_location("Symbol"),
        );
        v.is_compliant = false;

        downgrade_known_unembeddable_fonts(&mut v);

        assert!(v.errors.is_empty(), "Symbol error should have been moved");
        assert_eq!(v.warnings.len(), 1, "exactly one warning should be emitted");
        assert_eq!(v.warnings[0].code, WarningCode::KnownUnembeddableFont);
        assert!(
            v.is_compliant,
            "validation should flip to compliant once the only remaining error was downgraded"
        );
    }

    #[test]
    fn downgrade_does_not_mask_other_font_errors() {
        // Non-Symbol font-not-embedded errors stay as errors and the
        // result remains non-compliant.
        use crate::compliance::types::{ComplianceError, ErrorCode, ValidationResult};

        let mut v = ValidationResult::new(PdfALevel::A1b);
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'Helvetica' not embedded")
                .with_location("Helvetica"),
        );
        v.is_compliant = false;

        downgrade_known_unembeddable_fonts(&mut v);

        assert_eq!(v.errors.len(), 1, "non-Symbol error should stay");
        assert!(v.warnings.is_empty(), "no warning should be emitted");
        assert!(!v.is_compliant, "still non-compliant");
    }

    #[test]
    fn downgrade_handles_zapf_dingbats() {
        // ZapfDingbats has the same shape of problem as Symbol.
        use crate::compliance::types::{ComplianceError, ErrorCode, ValidationResult};

        let mut v = ValidationResult::new(PdfALevel::A1b);
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'ZapfDingbats' not embedded")
                .with_location("ZapfDingbats"),
        );
        v.is_compliant = false;

        downgrade_known_unembeddable_fonts(&mut v);

        assert!(v.errors.is_empty());
        assert_eq!(v.warnings.len(), 1);
        assert!(v.is_compliant);
    }

    #[test]
    fn downgrade_accepts_subset_prefix() {
        // PDF subset fonts use the `ABCDEF+FontName` six-tag-char prefix
        // convention (ISO 32000-1 §9.6.4). The downgrader must match
        // these as the underlying Standard14 font.
        use crate::compliance::types::{ComplianceError, ErrorCode, ValidationResult};

        let mut v = ValidationResult::new(PdfALevel::A1b);
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'ABCDEF+Symbol' not embedded")
                .with_location("ABCDEF+Symbol"),
        );
        v.is_compliant = false;

        downgrade_known_unembeddable_fonts(&mut v);
        assert!(v.errors.is_empty(), "subset-prefixed Symbol must downgrade");
        assert_eq!(v.warnings.len(), 1);
        assert!(v.is_compliant);
    }

    #[test]
    fn downgrade_rejects_substring_match() {
        // Names that merely _contain_ "Symbol" or "ZapfDingbats" as a
        // substring must NOT match — `MySymbolFont`, `ZapfDingbatsITC`,
        // etc. are arbitrary fonts unrelated to the unembeddable
        // Standard14 set.
        use crate::compliance::types::{ComplianceError, ErrorCode, ValidationResult};

        let mut v = ValidationResult::new(PdfALevel::A1b);
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'MySymbolFont' not embedded")
                .with_location("MySymbolFont"),
        );
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'ZapfDingbatsITC' not embedded")
                .with_location("ZapfDingbatsITC"),
        );
        v.is_compliant = false;

        downgrade_known_unembeddable_fonts(&mut v);
        assert_eq!(v.errors.len(), 2, "neither bogus name should be downgraded");
        assert!(v.warnings.is_empty());
        assert!(!v.is_compliant);
    }

    #[test]
    fn downgrade_propagates_location_to_warning() {
        // Copilot review #463: dropping `err.location` when downgrading
        // strips structured context callers may rely on. Propagate it.
        use crate::compliance::types::{ComplianceError, ErrorCode, ValidationResult};

        let mut v = ValidationResult::new(PdfALevel::A1b);
        v.errors.push(
            ComplianceError::new(ErrorCode::FontNotEmbedded, "font 'Symbol' not embedded")
                .with_location("ABCDEF+Symbol"),
        );

        downgrade_known_unembeddable_fonts(&mut v);
        assert_eq!(v.warnings.len(), 1);
        assert_eq!(v.warnings[0].location.as_deref(), Some("ABCDEF+Symbol"));
    }

    #[test]
    fn test_conversion_config_builder() {
        let config = ConversionConfig::new()
            .embed_fonts(false)
            .remove_javascript(false)
            .flatten_transparency(false);

        assert!(!config.embed_fonts);
        assert!(!config.remove_javascript);
        assert!(!config.flatten_transparency);
    }

    #[test]
    fn test_converter_creation() {
        let converter = PdfAConverter::new(PdfALevel::A2b);
        assert_eq!(converter.level(), PdfALevel::A2b);
    }

    #[test]
    fn test_conversion_result() {
        let mut result = ConversionResult::new(PdfALevel::A2b);
        assert!(!result.success);
        assert_eq!(result.level, PdfALevel::A2b);
        assert!(result.actions.is_empty());
        assert!(result.errors.is_empty());

        result.add_action(ConversionAction::new(ActionType::AddedXmpMetadata, "Test action"));
        assert_eq!(result.actions.len(), 1);

        result.add_error(ConversionError::new(ErrorCode::FontNotEmbedded, "Test error"));
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_xmp_generation() {
        let converter = PdfAConverter::new(PdfALevel::A2b);
        let xmp = converter.generate_xmp_metadata();

        assert!(xmp.contains("<pdfaid:part>2</pdfaid:part>"));
        assert!(xmp.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
    }

    #[test]
    fn test_action_type() {
        let action = ConversionAction::new(ActionType::AddedXmpMetadata, "Added metadata")
            .with_fixed_error(ErrorCode::MissingXmpMetadata);

        assert_eq!(action.action_type, ActionType::AddedXmpMetadata);
        assert_eq!(action.fixed_error, Some(ErrorCode::MissingXmpMetadata));
    }

    #[test]
    fn test_conversion_config_remove_encryption() {
        let config = ConversionConfig::default();
        assert!(config.remove_encryption);
        assert!(config.remove_embedded_files);
        assert!(!config.add_structure);
        assert!(config.icc_profile.is_none());
    }

    #[test]
    fn test_conversion_config_add_structure() {
        let config = ConversionConfig::new().add_structure(true);
        assert!(config.add_structure);
    }

    #[test]
    fn test_conversion_config_with_icc_profile() {
        let profile = vec![1, 2, 3, 4];
        let config = ConversionConfig::new().with_icc_profile(profile.clone());
        assert_eq!(config.icc_profile.unwrap(), profile);
    }

    #[test]
    fn test_converter_with_config() {
        let config = ConversionConfig::new().embed_fonts(false);
        let converter = PdfAConverter::new(PdfALevel::A1b).with_config(config);
        assert_eq!(converter.level(), PdfALevel::A1b);
    }

    #[test]
    fn test_converter_levels() {
        for level in [
            PdfALevel::A1a,
            PdfALevel::A1b,
            PdfALevel::A2a,
            PdfALevel::A2b,
            PdfALevel::A2u,
            PdfALevel::A3a,
            PdfALevel::A3b,
            PdfALevel::A3u,
        ] {
            let converter = PdfAConverter::new(level);
            assert_eq!(converter.level(), level);
        }
    }

    #[test]
    fn test_xmp_generation_a1b() {
        let converter = PdfAConverter::new(PdfALevel::A1b);
        let xmp = converter.generate_xmp_metadata();
        assert!(xmp.contains("<pdfaid:part>1</pdfaid:part>"));
        assert!(xmp.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
        assert!(xmp.contains("xmpmeta"));
        assert!(xmp.contains("xpacket"));
    }

    #[test]
    fn test_xmp_generation_a3a() {
        let converter = PdfAConverter::new(PdfALevel::A3a);
        let xmp = converter.generate_xmp_metadata();
        assert!(xmp.contains("<pdfaid:part>3</pdfaid:part>"));
        assert!(xmp.contains("<pdfaid:conformance>A</pdfaid:conformance>"));
    }

    #[test]
    fn test_xmp_generation_a2u() {
        let converter = PdfAConverter::new(PdfALevel::A2u);
        let xmp = converter.generate_xmp_metadata();
        assert!(xmp.contains("<pdfaid:part>2</pdfaid:part>"));
        assert!(xmp.contains("<pdfaid:conformance>U</pdfaid:conformance>"));
    }

    #[test]
    fn test_conversion_result_new() {
        let result = ConversionResult::new(PdfALevel::A2b);
        assert!(!result.success);
        assert_eq!(result.level, PdfALevel::A2b);
        assert!(result.actions.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_conversion_result_add_action() {
        let mut result = ConversionResult::new(PdfALevel::A1b);
        result.add_action(ConversionAction::new(ActionType::RemovedJavaScript, "Removed JS"));
        result
            .add_action(ConversionAction::new(ActionType::RemovedEncryption, "Removed encryption"));
        assert_eq!(result.actions.len(), 2);
    }

    #[test]
    fn test_conversion_result_add_error() {
        let mut result = ConversionResult::new(PdfALevel::A1b);
        result.add_error(ConversionError::new(ErrorCode::FontNotEmbedded, "Cannot embed font"));
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_code, ErrorCode::FontNotEmbedded);
        assert_eq!(result.errors[0].reason, "Cannot embed font");
    }

    #[test]
    fn test_conversion_action_debug_clone() {
        let action = ConversionAction::new(ActionType::AddedLanguage, "Added en");
        let cloned = action.clone();
        assert_eq!(cloned.action_type, ActionType::AddedLanguage);
        let debug = format!("{:?}", action);
        assert!(debug.contains("AddedLanguage"));
    }

    #[test]
    fn test_conversion_error_debug_clone() {
        let error = ConversionError::new(ErrorCode::EncryptionNotAllowed, "Encrypted");
        let cloned = error.clone();
        assert_eq!(cloned.error_code, ErrorCode::EncryptionNotAllowed);
        let debug = format!("{:?}", error);
        assert!(debug.contains("EncryptionNotAllowed"));
    }

    #[test]
    fn test_all_action_types() {
        let types = vec![
            ActionType::AddedXmpMetadata,
            ActionType::AddedPdfaIdentification,
            ActionType::EmbeddedFont,
            ActionType::AddedOutputIntent,
            ActionType::RemovedJavaScript,
            ActionType::RemovedEncryption,
            ActionType::FlattenedTransparency,
            ActionType::RemovedEmbeddedFiles,
            ActionType::AddedStructure,
            ActionType::FixedAnnotation,
            ActionType::AddedLanguage,
        ];
        for t in types {
            let copy = t;
            assert_eq!(t, copy);
            let debug = format!("{:?}", t);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_converter_debug_clone() {
        let converter = PdfAConverter::new(PdfALevel::A2b);
        let cloned = converter.clone();
        assert_eq!(cloned.level(), PdfALevel::A2b);
        let debug = format!("{:?}", converter);
        assert!(debug.contains("PdfAConverter"));
    }

    #[test]
    fn test_srgb_icc_profile() {
        let profile = PdfAConverter::get_srgb_icc_profile();
        assert!(!profile.is_empty());
    }

    #[test]
    fn test_conversion_config_debug_clone() {
        let config = ConversionConfig::new().embed_fonts(false);
        let cloned = config.clone();
        assert!(!cloned.embed_fonts);
        let debug = format!("{:?}", config);
        assert!(debug.contains("ConversionConfig"));
    }

    #[test]
    fn test_conversion_result_debug_clone() {
        let result = ConversionResult::new(PdfALevel::A1a);
        let cloned = result.clone();
        assert_eq!(cloned.level, PdfALevel::A1a);
        let debug = format!("{:?}", result);
        assert!(debug.contains("ConversionResult"));
    }
}
