//! Residual classifier. Font-environment diffs are not layout defects.

use super::catalog::PageRecord;
use super::fingerprint::{compare_fingerprints, FingerprintDelta, RasterFingerprint};
use super::wrap::{left_strip_text_deficit, WrapGeometry};
use super::ResidualClass;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassificationLimits {
    pub ink_ppm: u32,
    pub hist_l1: u32,
    pub bbox_hu: u32,
}

impl Default for ClassificationLimits {
    fn default() -> Self {
        Self {
            ink_ppm: 1_500,
            hist_l1: 120,
            bbox_hu: 40,
        }
    }
}

impl ClassificationLimits {
    pub fn from_record(record: &PageRecord) -> Self {
        Self {
            ink_ppm: record.ink_budget_ppm.max(1),
            hist_l1: record.hist_l1_budget.max(1),
            bbox_hu: record.bbox_delta_hu.max(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassificationInput<'a> {
    pub record: Option<&'a PageRecord>,
    pub oracle: &'a RasterFingerprint,
    pub candidate: &'a RasterFingerprint,
    pub wrap: Option<WrapGeometry>,
    pub face_substituted: bool,
    pub table_boxes_match: bool,
    pub line_owners_match: bool,
}

pub fn classify_page_residual(input: ClassificationInput<'_>) -> ResidualClass {
    let limits = input
        .record
        .map(ClassificationLimits::from_record)
        .unwrap_or_default();
    if let Some(geom) = input.wrap {
        let sample = left_strip_text_deficit(input.oracle, input.candidate, geom);
        if sample.flagged {
            return ResidualClass::WrapFlow;
        }
    }

    let delta = compare_fingerprints(input.oracle, input.candidate);
    if within_budget(&delta, &limits) {
        return ResidualClass::None;
    }

    if input.face_substituted
        && input.table_boxes_match
        && input.line_owners_match
        && (input.record.map(|r| r.font_env_sensitive).unwrap_or(true))
        && delta.bbox_l1 <= limits.bbox_hu.saturating_mul(2)
    {
        return ResidualClass::FontEnv;
    }

    if delta.hint == ResidualClass::TablePlace
        && input.record.map(|r| r.table_place_risk).unwrap_or(true)
    {
        return ResidualClass::TablePlace;
    }

    match delta.hint {
        ResidualClass::FontWeight | ResidualClass::FontWidth
            if input.table_boxes_match && input.line_owners_match =>
        {
            if input.face_substituted {
                ResidualClass::FontEnv
            } else {
                delta.hint
            }
        }
        ResidualClass::Glyph if input.record.map(|r| r.glyph_paint_risk).unwrap_or(true) => {
            ResidualClass::Glyph
        }
        ResidualClass::Paint => ResidualClass::Paint,
        ResidualClass::WrapFlow => ResidualClass::WrapFlow,
        ResidualClass::None => ResidualClass::None,
        other => other,
    }
}

fn within_budget(delta: &FingerprintDelta, limits: &ClassificationLimits) -> bool {
    !delta.size_mismatch
        && delta.ink_ppm_abs <= limits.ink_ppm
        && delta.hist_l1 <= limits.hist_l1
        && delta.bbox_l1 <= limits.bbox_hu
}
