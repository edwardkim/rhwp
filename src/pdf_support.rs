//! Compile-time PDF support of this artifact, independent of document contents.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportSupport {
    pub enabled: bool,
    pub jbig2: bool,
    pub jpeg2000: bool,
    pub icc: bool,
    pub bounded_xref_recovery: bool,
}

/// Effective compiled support, including dependency feature unification.
/// This does not promise support for every PDF input.
pub const fn pdf_import_support() -> PdfImportSupport {
    #[cfg(feature = "pdf-import")]
    let (jbig2, jpeg2000) = pdf_oxide::image_codec_support();
    #[cfg(not(feature = "pdf-import"))]
    let (jbig2, jpeg2000) = (false, false);
    PdfImportSupport {
        enabled: cfg!(feature = "pdf-import"),
        jbig2,
        jpeg2000,
        icc: {
            #[cfg(feature = "pdf-import")]
            {
                pdf_oxide::icc_transform_support()
            }
            #[cfg(not(feature = "pdf-import"))]
            {
                false
            }
        },
        bounded_xref_recovery: cfg!(feature = "pdf-import"),
    }
}
