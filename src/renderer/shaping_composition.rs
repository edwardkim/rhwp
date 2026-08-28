//! W10-Q2-D composition-owner handoff for dormant horizontal shaping.
//!
//! The retained outcome is still shadow data.  This module only fixes its
//! ownership lifetime across composition; layout and paint do not consume it.

use super::shaping_paragraph::{HorizontalShapingLineDisposition, HorizontalShapingLineOutcome};
use std::sync::Arc;

/// Preserve only a complete Q2-C qualified result, without cloning any final
/// target measurement.  Rollback and non-target outcomes keep the legacy
/// composed paragraph completely sidecar-free.
pub(crate) fn retain_qualified_horizontal_shaping_outcome(
    outcome: Arc<HorizontalShapingLineOutcome>,
) -> Option<Arc<HorizontalShapingLineOutcome>> {
    (outcome.trace.disposition == HorizontalShapingLineDisposition::DormantQualified
        && !outcome.trace.product_published
        && outcome
            .lines
            .iter()
            .any(|line| !line.target_runs.is_empty()))
    .then_some(outcome)
}
