//! Experimental PDF frontend. Emits common IR without serializing/reparsing HWPX.
//!
//! `pdf-import` is enabled by default and can be excluded with
//! `--no-default-features`. The internal backend limits decoded outputs, but
//! full codec/font/cache allocation bounds are not yet proven.
mod extract;
mod lowering;
mod path_paint;
mod recognition;
mod structured_lowering;
mod table_detection;
mod table_hierarchy;
mod table_topology;

use crate::model::document::Document;
use pdf_oxide::{elements::PathContent, extractors::PdfImage, layout::TextSpan, PdfDocument};
use serde::Serialize;
use serde_json::Value;

/// Product adapters may stop reading after this limit plus one byte.
pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
/// Maximum selected pages; adapters must bound range expansion before allocation.
pub const MAX_PAGES: usize = 128;
const MAX_POOL_ENTRIES: usize = u16::MAX as usize;
const MAX_SOURCE_SPANS: usize = 40_000;
const MAX_TEXT_BYTES: usize = 2 * 1024 * 1024;
const MAX_DOCUMENT_IMAGE_PIXELS: u64 = 64_000_000;

#[derive(Clone, Debug, Default)]
pub struct PdfImportOptions {
    /// Zero-based, strictly increasing source pages. None imports every page.
    pub source_pages: Option<Vec<usize>>,
}

#[derive(Debug)]
pub struct PdfImportError {
    pub code: &'static str,
    pub message: &'static str,
}

impl PdfImportError {
    fn backend(error: pdf_oxide::error::Error) -> Self {
        match error {
            pdf_oxide::error::Error::ImportResourceLimit(reason) => Self::budget(reason),
            pdf_oxide::error::Error::EncryptedPdf => Self::new(
                "PDF_ENCRYPTION_UNSUPPORTED",
                "encrypted PDF import is not supported",
            ),
            _ => Self::new(
                "PDF_PARSE_FAILED",
                "PDF container or extraction could not be processed",
            ),
        }
    }
    fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
    fn budget(message: &'static str) -> Self {
        Self::new("PDF_RESOURCE_LIMIT", message)
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportMetadata {
    pub source_page_count: usize,
    pub pages: Vec<PdfPageMetadata>,
    pub paragraph_policy: &'static str,
    pub parser_allocation_bounds_proven: bool,
    pub warnings: Vec<&'static str>,
}

pub struct PdfImportResult {
    pub document: Document,
    pub metadata: PdfImportMetadata,
}

struct ExtractedPage {
    import_budget: std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
    media_box: (f32, f32, f32, f32),
    spans: Vec<TextSpan>,
    images: Vec<PdfImage>,
    image_orders: Vec<Vec<usize>>,
    paths: Vec<PathContent>,
    // Internal structural evidence only, dropped after each page's lowering.
    snapshot: Value,
}

#[derive(Default)]
struct Budget {
    spans: usize,
    text_bytes: usize,
    image_pixels: u64,
    path_operations: usize,
    cells: usize,
}

fn selected_pages(options: &PdfImportOptions, count: usize) -> Result<Vec<usize>, PdfImportError> {
    match &options.source_pages {
        None => Ok((0..count).collect()),
        Some(pages)
            if !pages.is_empty()
                && pages.len() <= MAX_PAGES
                && pages.iter().all(|p| *p < count)
                && pages.windows(2).all(|p| p[0] < p[1]) =>
        {
            Ok(pages.clone())
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
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(PdfImportError::budget("input exceeds 16 MiB"));
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err(PdfImportError::new(
            "PDF_INVALID_INPUT",
            "PDF signature missing",
        ));
    }
    // Copy only after the input budget check. Upstream internal allocations are
    // a separate, unresolved release gate; extraction counters do not bound them.
    let pdf = PdfDocument::from_bytes_with_limits(
        bytes.to_vec(),
        pdf_oxide::import_limits::PdfImportLimits::default(),
    )
    .map_err(PdfImportError::backend)?;
    if pdf.is_encrypted() {
        return Err(PdfImportError::new(
            "PDF_ENCRYPTION_UNSUPPORTED",
            "encrypted PDF import is not supported",
        ));
    }
    let count = pdf.page_count();
    pdf.check_import_budget().map_err(PdfImportError::backend)?;
    let count = count
        .map_err(|_| PdfImportError::new("PDF_PARSE_FAILED", "PDF page tree could not be read"))?;
    if count == 0 || count > MAX_PAGES {
        return Err(PdfImportError::budget("PDF must have 1 to 128 pages"));
    }
    let selected = selected_pages(options, count)?;
    let mut document = lowering::new_document().map_err(|_| {
        PdfImportError::new("PDF_LOWERING_FAILED", "IR scaffold initialization failed")
    })?;
    let template = document
        .sections
        .pop()
        .ok_or_else(|| PdfImportError::new("PDF_LOWERING_FAILED", "IR scaffold has no section"))?;
    let mut next_id = 1;
    let mut budget = Budget::default();
    let mut pages = Vec::with_capacity(selected.len());
    for source_page in selected {
        let page = extract::extract(&pdf, source_page, &mut budget)?;
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
            document.doc_info.char_shapes.len(),
            document.bin_data_content.len(),
        ] {
            if size.checked_add(extra).is_none_or(|n| n > MAX_POOL_ENTRIES) {
                return Err(PdfImportError::budget(
                    "document resource ID budget exceeded",
                ));
            }
        }
        let section = document.sections.len();
        document.sections.push(template.clone());
        let lowered = structured_lowering::lower(&page, &mut document, &mut next_id);
        pdf.check_import_budget().map_err(PdfImportError::backend)?;
        let report = lowered.map_err(|_| {
            PdfImportError::new("PDF_LOWERING_FAILED", "PDF content could not be lowered")
        })?;
        let n = |key: &str| report[key].as_u64().unwrap_or(0) as usize;
        let (left, bottom, right, top) = page.media_box;
        pages.push(PdfPageMetadata {
            source_page,
            output_section: section,
            media_box: [left, bottom, right, top],
            source_spans: page
                .spans
                .iter()
                .enumerate()
                .map(|(source_span, s)| PdfSpanEvidence {
                    source_span,
                    bounds: [
                        s.bbox.x - left,
                        top - s.bbox.y - s.font_size * 0.95,
                        s.bbox.width,
                        s.font_size,
                    ],
                    baseline: top - s.bbox.y,
                })
                .collect(),
            semantic_tables: n("semanticTables"),
            nested_tables: n("semanticNestedTables"),
            deferred_tables: report["deferredTables"].as_array().map_or(0, Vec::len),
            unsupported_paths: n("unsupportedPaths"),
        });
    }
    document.doc_properties.section_count = u16::try_from(document.sections.len())
        .map_err(|_| PdfImportError::budget("section count exceeds format range"))?;
    document
        .doc_info
        .seal_raw_provenance(&document.doc_properties);
    document.seal_body_raw_provenance();
    Ok(PdfImportResult {
        document,
        metadata: PdfImportMetadata {
            source_page_count: count, pages,
            paragraph_policy: "observed-lines; authored paragraph boundaries are not recovered",
            parser_allocation_bounds_proven: false,
            warnings: vec![
                "Experimental PDF frontend; upstream allocation bounds remain unproven",
                "Clipping, transparency, embedded fonts and complex paint are not fully reconstructed",
                "Deferred table families retain positional content; inspect page diagnostics",
            ],
        },
    })
}
