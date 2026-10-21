//! One product policy for pre-copy checks, page selection and backend accounting.
use pdf_oxide::import_limits::PdfImportLimits;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportPolicy {
    pub schema_version: u32,
    pub max_source_pages: usize,
    pub max_selected_pages: usize,
    pub max_path_operations: usize,
    pub max_document_path_operations: usize,
    /// Emitted image placements across selected pages, including repeated paint
    /// of one resource. Pixel/decode limits independently bound bitmap work.
    pub max_document_image_placements: usize,
    pub backend: PdfImportLimits,
}

impl Default for PdfImportPolicy {
    fn default() -> Self {
        let backend = PdfImportLimits {
            // Absolute request ceilings. Page-local work is narrowed after the
            // bounded page-tree traversal; shared collection structure retains
            // its document-wide cap. These are not live-heap limits.
            operators: 4_000_000,
            // Independent interpreter passes revisit the same parsed operators.
            // Keep their cumulative work finite at the product operator scale.
            xobject_visits: 4_000_000,
            reference_visits: 4_000_000,
            spans: 120_000,
            entries: 400_000,
            decoded_bytes: 2 * 1024 * 1024 * 1024,
            // Full CJK faces commonly carry tens of thousands of CID widths
            // and TrueType mappings each, even on a single page. Allow several
            // such faces while retaining a finite request-wide expansion cap.
            // Every entry still charges 128 decoded bytes before allocation;
            // the independently narrowed decoded budget and explicit host caps
            // remain in force. This does not change the raw decoder defaults.
            font_entries: 1_000_000,
            ..PdfImportLimits::default()
        };
        Self {
            schema_version: 1,
            // Source traversal is independently bounded by the backend visit budget.
            max_source_pages: PdfImportLimits::default().reference_visits,
            max_selected_pages: super::MAX_PAGES,
            max_path_operations: 20_000,
            max_document_path_operations: 1_000_000,
            // Same finite request scale as XObject visits; inline images also
            // consume a placement even though they do not invoke an XObject.
            max_document_image_placements: backend.xobject_visits,
            backend,
        }
    }
}

impl PdfImportPolicy {
    pub(crate) fn for_selected_pages(mut self, pages: usize) -> Self {
        self.backend.operators = self
            .backend
            .operators
            .min(pages.saturating_mul(100_000).max(250_000));
        self.backend.xobject_visits = self.backend.xobject_visits.min(self.backend.operators);
        self.backend.reference_visits = self.backend.reference_visits.min(self.backend.operators);
        self.backend.spans = self
            .backend
            .spans
            .min(pages.saturating_mul(1_000).max(40_000));
        // Collection entries cover shared container/xref data, content operand
        // arrays and paint-order/clip metadata, not just emitted page elements.
        // Keep the finite document-wide cap (and any lower host cap). Selected
        // page count cannot predict these independently bounded structures.
        self.backend.decoded_bytes = self.backend.decoded_bytes.min(
            pages
                .saturating_mul(64 * 1024 * 1024)
                .max(256 * 1024 * 1024),
        );
        self.max_document_path_operations = self
            .max_document_path_operations
            .min(pages.saturating_mul(20_000).max(80_000));
        self
    }

    pub fn max_input_bytes(&self) -> usize {
        self.backend.input_bytes
    }
}
