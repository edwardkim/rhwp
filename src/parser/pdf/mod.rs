//! Experimental PDF frontend. Emits common IR without serializing/reparsing HWPX.
//!
//! `pdf-import` is enabled by default and can be excluded with
//! `--no-default-features`. The internal backend limits decoded outputs, but
//! full codec/font/cache allocation bounds are not yet proven.
pub(crate) mod allocation;
mod body_regions;
mod char_pool;
mod editable;
mod extract;
mod lowering;
mod path_paint;
mod policy;
mod recognition;
mod rotated_text;
mod structured_lowering;
mod table_detection;
mod table_hierarchy;
mod table_inference;
mod table_paint;
mod table_topology;
mod text_analysis;
mod text_clip;
mod value_allocation;
mod value_macro;

use crate::model::document::Document;
use pdf_oxide::import_limits::{ImportBudget, ImportBudgetSnapshot, ImportLimitFailure};
use pdf_oxide::{elements::PathContent, extractors::PdfImage, layout::TextSpan, PdfDocument};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

pub use policy::{PdfImportOverrides, PdfImportPolicy, PdfImportProfile};

/// Product adapters may stop reading after this limit plus one byte.
pub const MAX_INPUT_BYTES: usize = pdf_oxide::import_limits::DEFAULT_INPUT_BYTES;
/// Maximum selected pages; adapters must bound range expansion before allocation.
pub const MAX_PAGES: usize = 1024;
const MAX_POOL_ENTRIES: usize = u16::MAX as usize;

#[derive(Clone, Default)]
pub struct PdfImportOptions {
    /// Opening password. Never included in diagnostics or Debug output.
    pub password: Option<String>,
    /// Zero-based, strictly increasing source pages. None imports every page.
    pub source_pages: Option<Vec<usize>>,
    /// Include page-scoped observations and competing table hypotheses in metadata.
    /// Contains source text; opt-in, local review only.
    pub include_table_inference: bool,
    /// Where recognized body paragraphs are placed.
    pub layout: PdfImportLayout,
}

impl PdfImportOptions {
    pub(crate) fn with_password(value: Option<&[u8]>) -> Result<Self, PdfImportError> {
        let password = value
            .map(|bytes| {
                if bytes.len() > 1024 {
                    return Err(PdfImportError::new(
                        "PDF_IMPORT_OPTIONS",
                        "PDF password is too long",
                    ));
                }
                std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| {
                    PdfImportError::new("PDF_IMPORT_OPTIONS", "PDF password must be UTF-8")
                })
            })
            .transpose()?;
        Ok(Self {
            password,
            ..Default::default()
        })
    }
}

impl std::fmt::Debug for PdfImportOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfImportOptions")
            .field("password", &self.password.as_ref().map(|_| "[redacted]"))
            .field("source_pages", &self.source_pages)
            .field("include_table_inference", &self.include_table_inference)
            .field("layout", &self.layout)
            .finish()
    }
}

/// Placement of recognized body paragraphs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PdfImportLayout {
    /// One positioned text box per recognized body region (default). Every
    /// page keeps the source geometry; paragraphs end at region borders.
    #[default]
    Fidelity,
    /// Body paragraphs become the section's text flow with page margins,
    /// columns and page/column breaks measured from the source. Other content
    /// stays positioned on its page.
    Editable,
}

impl PdfImportLayout {
    /// Parses the stable option spelling used by the CLI and the WASM API.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fidelity" => Some(Self::Fidelity),
            "editable" => Some(Self::Editable),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct PdfImportError {
    pub code: &'static str,
    pub message: &'static str,
    /// Opt-in local diagnostics. Display deliberately retains the stable message
    /// and does not publish backend strings that can contain source document data.
    pub diagnostics: Option<Box<PdfImportFailure>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportFailure {
    pub stage: &'static str,
    pub source_page: Option<usize>,
    pub backend_kind: Option<&'static str>,
    pub backend_message: Option<String>,
    pub resource_usage: Option<ImportBudgetSnapshot>,
    pub frontend_limit: Option<ImportLimitFailure>,
}

impl PdfImportError {
    pub(crate) fn backend(error: pdf_oxide::error::Error) -> Self {
        use pdf_oxide::error::Error;
        let kind = match &error {
            Error::ImportResourceLimit(_) => "resource_limit",
            Error::EncryptedPdf => "encrypted",
            Error::Unsupported(_)
            | Error::UnsupportedPattern(_)
            | Error::UnsupportedTextPaint(_) => "unsupported",
            Error::UnsupportedFilter(_) => "unsupported_filter",
            Error::Image(_) => "image",
            Error::Font(_) => "font",
            Error::Decode(_) => "decode",
            Error::ParseError { .. } => "parse",
            Error::InvalidPdf(_) => "invalid_pdf",
            Error::InvalidXref => "xref",
            Error::ObjectNotFound(..) => "object_not_found",
            Error::InvalidObjectType { .. } => "object_type",
            Error::UnexpectedEof => "unexpected_eof",
            _ => "backend",
        };
        let detail = error.to_string();
        let mut result = match error {
            Error::UnsupportedTextPaint(_) => Self::new(
                "PDF_TEXT_UNSUPPORTED",
                "PDF text paint is unsupported or invalid",
            ),
            pdf_oxide::error::Error::UnsupportedPattern(_) => Self::new(
                "PDF_PATTERN_UNSUPPORTED",
                "PDF pattern paint is unsupported or invalid",
            ),
            pdf_oxide::error::Error::ImportResourceLimit(reason) => Self::budget(reason),
            pdf_oxide::error::Error::EncryptedPdf => Self::new(
                "PDF_ENCRYPTION_UNSUPPORTED",
                "PDF encryption method or crypt filter combination is not supported",
            ),
            _ => Self::new(
                "PDF_PARSE_FAILED",
                "PDF container or extraction could not be processed",
            ),
        };
        result.diagnostics = Some(Box::new(PdfImportFailure {
            stage: "backend",
            source_page: None,
            backend_kind: Some(kind),
            backend_message: Some(detail),
            resource_usage: None,
            frontend_limit: None,
        }));
        result
    }
    pub(super) fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message,
            diagnostics: None,
        }
    }
    fn budget(message: &'static str) -> Self {
        Self::new("PDF_RESOURCE_LIMIT", message)
    }

    fn at(
        mut self,
        stage: &'static str,
        page: Option<usize>,
        budget: Option<&ImportBudget>,
    ) -> Self {
        let failure = self.diagnostics.get_or_insert_with(|| {
            Box::new(PdfImportFailure {
                stage,
                source_page: page,
                backend_kind: None,
                backend_message: None,
                resource_usage: None,
                frontend_limit: None,
            })
        });
        if matches!(failure.stage, "backend" | "frontend") {
            failure.stage = stage;
        }
        failure.source_page = failure.source_page.or(page);
        if let Some(budget) = budget {
            failure.resource_usage = Some(budget.snapshot());
        }
        self
    }

    fn limit(mut self, reason: &'static str, used: usize, requested: usize, limit: usize) -> Self {
        self = self.at("frontend", None, None);
        self.diagnostics
            .as_mut()
            .expect("frontend context")
            .frontend_limit = Some(ImportLimitFailure {
            reason,
            used: Some(used as u64),
            requested: Some(requested as u64),
            limit: Some(limit as u64),
        });
        self
    }
}

impl std::fmt::Display for PdfImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for PdfImportError {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfSpanEvidence {
    pub source_span: usize,
    /// Page-local top-left coordinates in points; baseline is independent.
    pub bounds: [f32; 4],
    pub baseline: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageMetadata {
    pub source_page: usize,
    pub output_section: usize,
    pub media_box: [f32; 4],
    pub source_spans: Vec<PdfSpanEvidence>,
    pub semantic_tables: usize,
    pub nested_tables: usize,
    pub deferred_tables: usize,
    pub unsupported_paths: usize,
    pub table_inference: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportMetadata {
    pub pdf_import_support: crate::pdf_support::PdfImportSupport,
    pub source_page_count: usize,
    pub pages: Vec<PdfPageMetadata>,
    pub paragraph_policy: &'static str,
    pub layout: PdfImportLayout,
    /// Editable layout only: sections, measured margins and moved paragraphs.
    #[serde(skip_serializing_if = "Value::is_null")]
    pub editable_layout: Value,
    pub parser_allocation_bounds_proven: bool,
    pub warnings: Vec<&'static str>,
    pub resource_policy: PdfImportPolicy,
    pub resource_usage: ImportBudgetSnapshot,
}

pub struct PdfImportResult {
    pub document: Document,
    pub metadata: PdfImportMetadata,
}

struct ExtractedPage {
    import_budget: std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
    media_box: (f32, f32, f32, f32),
    spans: Vec<TextSpan>,
    span_orders: Vec<Vec<usize>>,
    span_fonts: Vec<Option<std::sync::Arc<pdf_oxide::fonts::FontInfo>>>,
    span_clips: Vec<Option<std::sync::Arc<pdf_oxide::elements::PathClip>>>,
    span_paints: Vec<Option<crate::model::text_paint::TextPaint>>,
    images: Vec<PdfImage>,
    image_orders: Vec<Vec<usize>>,
    paths: Vec<PathContent>,
    // Internal structural evidence only, dropped after each page's lowering.
    snapshot: Value,
    recognition: recognition::PageRecognition,
    table_inference: table_inference::Ledger,
}

#[derive(Default)]
struct Budget {
    policy: PdfImportPolicy,
    spans: usize,
    text_bytes: usize,
    image_pixels: u64,
    image_placements: usize,
    path_operations: usize,
    cells: usize,
}

fn selected_pages(
    options: &PdfImportOptions,
    count: usize,
    policy: &PdfImportPolicy,
    allocation: allocation::Allocation<'_>,
) -> Result<Vec<usize>, PdfImportError> {
    match &options.source_pages {
        None if count <= policy.max_selected_pages => allocation.collect(0..count),
        None => Err(
            PdfImportError::budget("selected page count exceeds request policy").limit(
                "selected pages",
                0,
                count,
                policy.max_selected_pages,
            ),
        ),
        Some(pages)
            if !pages.is_empty()
                && pages.len() <= policy.max_selected_pages
                && pages.iter().all(|p| *p < count)
                && pages.windows(2).all(|p| p[0] < p[1]) =>
        {
            allocation.copy_slice(pages)
        }
        _ => Err(PdfImportError::new(
            "PDF_PAGE_SELECTION",
            "source pages must be nonempty, in range and strictly increasing",
        )),
    }
}

/// Compile all requested PDF pages atomically into a single resource-owning IR.
///
/// There is no HWPX or POC executable dependency. Any failed page discards the
/// private builder. See module-level security limitation before enabling callers.
pub fn parse_pdf_with_options(
    bytes: &[u8],
    options: &PdfImportOptions,
) -> Result<PdfImportResult, PdfImportError> {
    parse_pdf_with_policy(bytes, options, PdfImportPolicy::default())
}

/// Import with an explicit trusted-host policy. Limits remain finite and request-local;
/// callers can lower them without duplicating the parser's validation or accounting.
pub fn parse_pdf_with_policy(
    bytes: &[u8],
    options: &PdfImportOptions,
    policy: PdfImportPolicy,
) -> Result<PdfImportResult, PdfImportError> {
    policy.validate()?;
    let request = Arc::new(ImportBudget::new(policy.backend).map_err(PdfImportError::backend)?);
    parse_pdf_with_request(bytes, options, policy, request)
}

/// Page conversion progress, independent of final layout/serialization time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PdfImportProgress {
    /// One-based original PDF page currently being converted.
    pub source_page: usize,
    pub source_page_count: usize,
    /// Number of selected pages whose conversion has completed.
    pub completed_pages: usize,
    pub total_pages: usize,
}

pub(crate) fn parse_pdf_with_request(
    bytes: &[u8],
    options: &PdfImportOptions,
    policy: PdfImportPolicy,
    request: Arc<ImportBudget>,
) -> Result<PdfImportResult, PdfImportError> {
    parse_pdf_with_request_and_progress(bytes, options, policy, request, &mut |_| Ok(()))
}

pub(crate) fn parse_pdf_with_request_and_progress(
    bytes: &[u8],
    options: &PdfImportOptions,
    policy: PdfImportPolicy,
    request: Arc<ImportBudget>,
    progress: &mut dyn FnMut(PdfImportProgress) -> Result<(), PdfImportError>,
) -> Result<PdfImportResult, PdfImportError> {
    policy.validate()?;
    if bytes.len() > policy.max_input_bytes() {
        return Err(
            PdfImportError::budget("input exceeds request byte limit").limit(
                "input bytes",
                0,
                bytes.len(),
                policy.max_input_bytes(),
            ),
        );
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err(PdfImportError::new(
            "PDF_INVALID_INPUT",
            "PDF signature missing",
        ));
    }
    // Copy only after the input budget check. Upstream internal allocations are
    // a separate, unresolved release gate; extraction counters do not bound them.
    let allocation = allocation::Allocation(&request);
    let input = allocation.copy_slice(bytes)?;
    let pdf = PdfDocument::from_bytes_with_budget(input, Arc::clone(&request))
        .map_err(|error| PdfImportError::backend(error).at("container", None, Some(&request)))?;
    if pdf.is_encrypted() {
        let password = options.password.as_deref().unwrap_or("");
        if password.len() > 1024 {
            return Err(PdfImportError::new(
                "PDF_IMPORT_OPTIONS",
                "PDF password is too long",
            ));
        }
        if !pdf.authenticate(password.as_bytes()).map_err(|error| {
            PdfImportError::backend(error).at("authentication", None, Some(&request))
        })? {
            return Err(if options.password.is_some() {
                PdfImportError::new("PDF_PASSWORD_INVALID", "PDF opening password is incorrect")
            } else {
                PdfImportError::new("PDF_PASSWORD_REQUIRED", "PDF opening password is required")
            });
        }
        // Maintainer policy (#8415): successful user/owner authentication allows
        // editable import; advisory copy/modify flags do not add another gate.
    }
    let count = pdf.page_count();
    pdf.check_import_budget()
        .map_err(|error| PdfImportError::backend(error).at("page_tree", None, Some(&request)))?;
    let count = count.map_err(|error| {
        let mut error = PdfImportError::backend(error).at("page_tree", None, Some(&request));
        error.message = "PDF page tree could not be read";
        error
    })?;
    if count == 0 || count > policy.max_source_pages {
        return Err(
            PdfImportError::budget("source page count exceeds request policy")
                .limit("source pages", 0, count, policy.max_source_pages)
                .at("page_tree", None, Some(&request)),
        );
    }
    let selected = selected_pages(options, count, &policy, allocation)?;
    let policy = policy.for_selected_pages(selected.len());
    request
        .restrict_work_limits(policy.backend.decoded_bytes, policy.backend.operators)
        .map_err(|error| {
            PdfImportError::backend(error).at("selection_policy", None, Some(&request))
        })?;
    request
        .restrict_structure_limits(policy.backend.spans, policy.backend.entries)
        .map_err(|error| {
            PdfImportError::backend(error).at("selection_policy", None, Some(&request))
        })?;
    request
        .restrict_traversal_limits(
            policy.backend.reference_visits,
            policy.backend.xobject_visits,
        )
        .map_err(|error| {
            PdfImportError::backend(error).at("selection_policy", None, Some(&request))
        })?;
    let mut document = lowering::new_document().map_err(|_| {
        PdfImportError::new("PDF_LOWERING_FAILED", "IR scaffold initialization failed")
    })?;
    let template = document
        .sections
        .pop()
        .ok_or_else(|| PdfImportError::new("PDF_LOWERING_FAILED", "IR scaffold has no section"))?;
    let mut next_id = 1;
    let mut character_pool = char_pool::Pool::new(&document, Arc::clone(&request))?;
    let mut budget = Budget {
        policy,
        ..Budget::default()
    };
    let mut pages = allocation::Allocation(&request).vec(selected.len())?;
    let mut body_frames = Vec::new();
    let total_pages = selected.len();
    for (index, source_page) in selected.into_iter().enumerate() {
        let update = PdfImportProgress {
            source_page: source_page + 1,
            source_page_count: count,
            completed_pages: index,
            total_pages,
        };
        progress(update)?;
        let page = extract::extract(
            &pdf,
            source_page,
            &mut budget,
            options.include_table_inference,
        )
        .map_err(|error| error.at("extract", Some(source_page), Some(&request)))?;
        // Reserve a conservative upper bound before any u16 resource-ID cast in
        // lowering. All pages share these pools; no page-local IDs are appended.
        let tables = page.snapshot["tableDetection"]["tables"]
            .as_array()
            .ok_or_else(|| {
                PdfImportError::new("PDF_RECOGNITION_FAILED", "missing table evidence")
            })?;
        let cells: usize = tables
            .iter()
            .map(|t| t["cells"].as_array().map_or(0, Vec::len))
            .sum();
        // Each observed span can introduce a paragraph/style; cells and inline
        // table hosts add at most three resource records each on this lowering path.
        let extra = page
            .spans
            .len()
            .checked_add(cells * 3 + tables.len() * 2 + 1 + page.images.len() + page.paths.len())
            .ok_or_else(|| PdfImportError::budget("style count overflow"))?;
        for size in [
            document.doc_info.para_shapes.len(),
            document.doc_info.border_fills.len(),
            document.doc_info.tab_defs.len(),
            document.doc_info.font_faces[0].len(),
            document.bin_data_content.len(),
        ] {
            if size.checked_add(extra).is_none_or(|n| n > MAX_POOL_ENTRIES) {
                return Err(PdfImportError::budget(
                    "document resource ID budget exceeded",
                ));
            }
        }
        // Character-shape references are u32 (CharShapeRef), unlike the u16
        // paragraph/font/binary IDs above. Do not impose their narrower range.
        if document
            .doc_info
            .char_shapes
            .len()
            .checked_add(extra)
            .is_none_or(|n| n > u32::MAX as usize)
        {
            return Err(PdfImportError::budget("character shape ID budget exceeded"));
        }
        let section = document.sections.len();
        document.sections.push(template.clone());
        let first_character_shape = document.doc_info.char_shapes.len();
        let lowered = structured_lowering::lower(&page, &mut document, &mut next_id);
        pdf.check_import_budget().map_err(|error| {
            PdfImportError::backend(error).at("lower", Some(source_page), Some(&request))
        })?;
        let report =
            lowered.map_err(|detail| {
                let mut error =
                    PdfImportError::new("PDF_LOWERING_FAILED", "PDF content could not be lowered")
                        .at("lower", Some(source_page), Some(&request));
                // Keep document-derived detail in opt-in diagnostics, never Display.
                error.diagnostics.as_mut().unwrap().backend_message = Some(detail);
                error
            })?;
        if options.layout == PdfImportLayout::Editable {
            let frames = editable::page_frames(section, &report, allocation)?;
            allocation.push(&mut body_frames, frames)?;
        }
        character_pool.compact_page(&mut document, first_character_shape, section)?;
        let n = |key: &str| report[key].as_u64().unwrap_or(0) as usize;
        let (left, bottom, right, top) = page.media_box;
        pages.push(PdfPageMetadata {
            source_page,
            output_section: section,
            media_box: [left, bottom, right, top],
            source_spans: allocation.collect(page.spans.iter().enumerate().map(
                |(source_span, s)| PdfSpanEvidence {
                    source_span,
                    bounds: [
                        s.bbox.x - left,
                        top - s.bbox.y - s.font_size * 0.95,
                        s.bbox.width,
                        s.font_size,
                    ],
                    baseline: top - s.bbox.y,
                },
            ))?,
            semantic_tables: n("semanticTables"),
            nested_tables: n("semanticNestedTables"),
            deferred_tables: report["deferredTables"].as_array().map_or(0, Vec::len),
            unsupported_paths: n("unsupportedPaths"),
            table_inference: page.table_inference.report(
                &page.snapshot["tableDetection"],
                &report,
                &page.paths,
                &page.recognition.report,
                options.include_table_inference,
            )?,
        });
        progress(PdfImportProgress {
            completed_pages: index + 1,
            ..update
        })?;
    }
    let editable_layout = if options.layout == PdfImportLayout::Editable {
        let report = editable::reflow(&mut document, &body_frames, allocation)?;
        // Page metadata follows the page into its reflowed section.
        let mut section_of = std::collections::BTreeMap::new();
        for (index, section) in report["sections"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            for page in section["pages"].as_array().into_iter().flatten() {
                if let Some(source) = page["sourceSection"].as_u64() {
                    allocation.insert(&mut section_of, source as usize, index)?;
                }
            }
        }
        for page in &mut pages {
            if let Some(&section) = section_of.get(&page.output_section) {
                page.output_section = section;
            }
        }
        report
    } else {
        Value::Null
    };
    document.doc_properties.section_count = u16::try_from(document.sections.len())
        .map_err(|_| PdfImportError::budget("section count exceeds format range"))?;
    document
        .doc_info
        .seal_raw_provenance(&document.doc_properties);
    document.seal_body_raw_provenance();
    Ok(PdfImportResult {
        document,
        metadata: PdfImportMetadata {
            pdf_import_support: crate::pdf_support::pdf_import_support(),
            source_page_count: count, pages,
            paragraph_policy: "observed-lines; authored paragraph boundaries are not recovered",
            layout: options.layout,
            editable_layout,
            parser_allocation_bounds_proven: false,
            warnings: vec![
                "Experimental PDF frontend; upstream allocation bounds remain unproven",
                "Clipping, transparency, embedded fonts and complex paint are not fully reconstructed",
                "Deferred table families retain positional content; inspect page diagnostics",
            ],
            resource_policy: policy,
            resource_usage: request.snapshot(),
        },
    })
}
