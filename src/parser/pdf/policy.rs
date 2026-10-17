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
    pub backend: PdfImportLimits,
}

impl Default for PdfImportPolicy {
    fn default() -> Self {
        let backend = PdfImportLimits {
            // Absolute request work ceilings, narrowed to the selected page count
            // after the bounded page-tree traversal. These are not live-heap limits.
            operators: 4_000_000,
            decoded_bytes: 2 * 1024 * 1024 * 1024,
            ..PdfImportLimits::default()
        };
        Self {
            schema_version: 1,
            // Source traversal is independently bounded by the backend visit budget.
            max_source_pages: backend.reference_visits,
            max_selected_pages: super::MAX_PAGES,
            max_path_operations: 20_000,
            max_document_path_operations: 1_000_000,
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
