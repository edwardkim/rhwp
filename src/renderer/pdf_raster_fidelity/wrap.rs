//! Stage 73 wrap-exclusion detector, without gym / visual_sweep.

use super::fingerprint::RasterFingerprint;
use super::ResidualClass;

/// Minimum PDF left-strip ink density (Stage 73 used 0.025).
pub const LEFT_STRIP_PDF_INK_MIN: f32 = 0.025;
/// rhwp/PDF left-strip ink ratio that flags a dropped wrap prefix.
pub const LEFT_STRIP_RHWP_RATIO_MAX: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WrapGeometry {
    pub page_width: u32,
    pub page_height: u32,
    /// Right-side table left edge in CSS px.
    pub table_left: u32,
    pub table_top: u32,
    pub table_right: u32,
    pub table_bottom: u32,
}

impl WrapGeometry {
    pub fn left_strip_width(&self) -> u32 {
        self.table_left.min(self.page_width)
    }

    pub fn is_right_side_table(&self) -> bool {
        self.table_left * 2 >= self.page_width && self.table_right > self.table_left
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WrapStripSample {
    pub oracle_ink: u32,
    pub candidate_ink: u32,
    pub oracle_area: u32,
    pub density: f32,
    pub ratio: f32,
    pub flagged: bool,
}

pub fn left_strip_text_deficit(
    oracle: &RasterFingerprint,
    candidate: &RasterFingerprint,
    geometry: WrapGeometry,
) -> WrapStripSample {
    let area = geometry
        .left_strip_width()
        .saturating_mul(geometry.page_height)
        .max(1);
    let oracle_ink = oracle.left_strip_ink;
    let candidate_ink = candidate.left_strip_ink;
    let density = oracle_ink as f32 / area as f32;
    let ratio = if oracle_ink == 0 {
        1.0
    } else {
        candidate_ink as f32 / oracle_ink as f32
    };
    let flagged = geometry.is_right_side_table()
        && density >= LEFT_STRIP_PDF_INK_MIN
        && ratio <= LEFT_STRIP_RHWP_RATIO_MAX;
    WrapStripSample {
        oracle_ink,
        candidate_ink,
        oracle_area: area,
        density,
        ratio,
        flagged,
    }
}

pub fn wrap_residual(sample: WrapStripSample) -> ResidualClass {
    if sample.flagged {
        ResidualClass::WrapFlow
    } else {
        ResidualClass::None
    }
}
