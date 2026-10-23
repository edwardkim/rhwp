//! Compile-time PDF support of this artifact, independent of document contents.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportSupport {
    pub enabled: bool,
    pub jbig2: bool,
    pub jpeg2000: bool,
}

/// Selected Cargo features; this does not promise support for every PDF input.
pub const fn pdf_import_support() -> PdfImportSupport {
    PdfImportSupport {
        enabled: cfg!(feature = "pdf-import"),
        jbig2: cfg!(feature = "pdf-jbig2"),
        jpeg2000: cfg!(feature = "pdf-jpeg2000"),
    }
}
