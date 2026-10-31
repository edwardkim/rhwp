//! Shared finite transfer limits. These are not live-heap or execution-time caps.
#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTransferLimits {
    pub output_bytes: usize,
    pub diagnostics_bytes: usize,
}
impl PdfTransferLimits {
    pub const fn standard() -> Self {
        Self {
            output_bytes: 128 * 1024 * 1024,
            diagnostics_bytes: 32 * 1024 * 1024,
        }
    }
    pub const fn constrained() -> Self {
        Self {
            output_bytes: 32 * 1024 * 1024,
            diagnostics_bytes: 8 * 1024 * 1024,
        }
    }
}
