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
        Self {
            schema_version: 1,
            max_source_pages: super::MAX_PAGES,
            max_selected_pages: super::MAX_PAGES,
            max_path_operations: 20_000,
            max_document_path_operations: 80_000,
            backend: PdfImportLimits::default(),
        }
    }
}

impl PdfImportPolicy {
    pub fn max_input_bytes(&self) -> usize {
        self.backend.input_bytes
    }
}
