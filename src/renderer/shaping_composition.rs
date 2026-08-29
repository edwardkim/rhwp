//! W10-Q2-D composition and emitted-run handoff for dormant horizontal shaping.
//!
//! The retained outcome and D2 mapping are still shadow data. This module fixes
//! ownership and final-run reconciliation; layout and paint do not consume it
//! before the D4 atomic activation slice.

use super::shaping::TerminalShapingDisposition;
use super::shaping_context::{
    HorizontalShapingContext, HorizontalShapingMeasurement,
    HorizontalShapingReplaySourceCertificateRejectReason,
};
use super::shaping_paragraph::{HorizontalShapingLineDisposition, HorizontalShapingLineOutcome};
use super::shaping_publication::{
    HorizontalShapingPageSidecars, HorizontalShapingRunDecision, HorizontalShapingRunRange,
    HorizontalShapingSidecarRejectReason,
};
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

/// Final render-tree run information required by the dormant D2 mapper.
///
/// The caller must describe the actual emitted surface rather than the model
/// style alone.  This keeps display projection, decoration, and split runs
/// fail-closed at the last owner boundary.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HorizontalShapingEmittedRunCandidate<'a> {
    pub node_id: u32,
    pub paragraph_text: &'a str,
    pub emitted_text: &'a str,
    pub scalar_start: usize,
    pub origin_x_px: f64,
    pub layout_positions_present: bool,
    pub display_projection_present: bool,
    pub horizontal_ltr_bidi0: bool,
    pub has_field_or_note_split: bool,
    pub has_char_overlap: bool,
    pub has_border_or_background: bool,
    pub has_decoration: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HorizontalShapingEmittedRunRejectReason {
    OutcomeNotQualified,
    UnsupportedSurface,
    EmptyRun,
    TextLimitExceeded,
    ScalarRangeOverflow,
    TextRangeMismatch,
    ExactTargetNotFound,
    MixedOrPartialTarget,
    MeasurementMismatch,
    AppliedTraceNotFound,
    GeometryMalformed,
}

/// A fully reconciled mapping. D2 computes this shadow geometry but no product
/// caller consumes it until the D4 atomic activation slice.
#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingMappedRun {
    pub node_id: u32,
    pub range: HorizontalShapingRunRange,
    pub line_width_px: f64,
    pub bbox_width_px: f64,
    pub next_origin_x_px: f64,
    pub decision: Arc<HorizontalShapingRunDecision>,
}

/// Pristine W9/K0 geometry retained as the only rollback result when the
/// dormant no-LineSeg owner transaction cannot be prepared completely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HorizontalShapingLegacyGeometry {
    pub line_width_px: f64,
    pub bbox_width_px: f64,
    pub next_origin_x_px: f64,
}

impl HorizontalShapingLegacyGeometry {
    fn is_well_formed(self) -> bool {
        self.line_width_px.is_finite()
            && self.line_width_px >= 0.0
            && self.bbox_width_px.is_finite()
            && self.bbox_width_px >= 0.0
            && self.next_origin_x_px.is_finite()
    }
}

/// Version-independent feature detection input for the first no-LineSeg lane.
/// N0 consumes this only in integration/shadow code; no product caller exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HorizontalShapingNoLineSegSurface {
    pub model_line_seg_count: usize,
    pub frame_interval_count: usize,
    pub edit_reflow: bool,
    pub stored_prefix: bool,
    pub split_cell: bool,
    pub has_inline_control: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HorizontalShapingNoLineSegOwnerRejectReason {
    LegacyGeometryMalformed,
    ModelLineSegPresent,
    FrameIntervalCountUnsupported,
    EditReflowUnsupported,
    StoredPrefixUnsupported,
    SplitCellUnsupported,
    InlineControlUnsupported,
    OutcomeNotQualified,
    TargetCountUnsupported,
    EmittedRunRejected(HorizontalShapingEmittedRunRejectReason),
    ReplaySourceRejected(HorizontalShapingReplaySourceCertificateRejectReason),
    OwnerIdentityMismatch,
}

/// A rejected preparation carries only the untouched legacy geometry. It has
/// no page-sidecar or resource handle, so partial publication is unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HorizontalShapingNoLineSegOwnerRejection {
    reason: HorizontalShapingNoLineSegOwnerRejectReason,
    fallback_geometry: HorizontalShapingLegacyGeometry,
}

impl HorizontalShapingNoLineSegOwnerRejection {
    pub(crate) fn reason(&self) -> HorizontalShapingNoLineSegOwnerRejectReason {
        self.reason
    }

    pub(crate) fn fallback_geometry(&self) -> HorizontalShapingLegacyGeometry {
        self.fallback_geometry
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

/// Fully prepared but still dormant owner transaction. The four consumers keep
/// explicit Arc slots so N0 can prove pointer identity before N1 may publish.
#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingNoLineSegOwnerTransaction {
    outcome: Arc<HorizontalShapingLineOutcome>,
    line_selection_measurement: Arc<HorizontalShapingMeasurement>,
    bbox_measurement: Arc<HorizontalShapingMeasurement>,
    next_origin_measurement: Arc<HorizontalShapingMeasurement>,
    sidecar_decision: Arc<HorizontalShapingRunDecision>,
    line_width_px: f64,
    bbox_width_px: f64,
    next_origin_x_px: f64,
    fallback_geometry: HorizontalShapingLegacyGeometry,
}

impl HorizontalShapingNoLineSegOwnerTransaction {
    pub(crate) fn outcome(&self) -> &Arc<HorizontalShapingLineOutcome> {
        &self.outcome
    }

    pub(crate) fn line_selection_measurement(&self) -> &Arc<HorizontalShapingMeasurement> {
        &self.line_selection_measurement
    }

    pub(crate) fn bbox_measurement(&self) -> &Arc<HorizontalShapingMeasurement> {
        &self.bbox_measurement
    }

    pub(crate) fn next_origin_measurement(&self) -> &Arc<HorizontalShapingMeasurement> {
        &self.next_origin_measurement
    }

    pub(crate) fn sidecar_measurement(&self) -> &Arc<HorizontalShapingMeasurement> {
        self.sidecar_decision
            .measurement()
            .expect("N0 sidecar decision is applied")
    }

    pub(crate) fn line_width_px(&self) -> f64 {
        self.line_width_px
    }

    pub(crate) fn bbox_width_px(&self) -> f64 {
        self.bbox_width_px
    }

    pub(crate) fn next_origin_x_px(&self) -> f64 {
        self.next_origin_x_px
    }

    pub(crate) fn fallback_geometry(&self) -> HorizontalShapingLegacyGeometry {
        self.fallback_geometry
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

fn no_lineseg_rejection(
    reason: HorizontalShapingNoLineSegOwnerRejectReason,
    fallback_geometry: HorizontalShapingLegacyGeometry,
) -> HorizontalShapingNoLineSegOwnerRejection {
    HorizontalShapingNoLineSegOwnerRejection {
        reason,
        fallback_geometry,
    }
}

/// Prepare all no-LineSeg owner consumers without touching composition, page
/// sidecars, resources, or product geometry. Any failure returns the exact
/// legacy snapshot and discards every staged Arc together.
pub(crate) fn prepare_horizontal_shaping_no_lineseg_owner_transaction(
    context: &HorizontalShapingContext,
    outcome: Arc<HorizontalShapingLineOutcome>,
    surface: HorizontalShapingNoLineSegSurface,
    candidate: HorizontalShapingEmittedRunCandidate<'_>,
    fallback_geometry: HorizontalShapingLegacyGeometry,
) -> Result<HorizontalShapingNoLineSegOwnerTransaction, HorizontalShapingNoLineSegOwnerRejection> {
    use HorizontalShapingNoLineSegOwnerRejectReason as Reject;

    if !fallback_geometry.is_well_formed() {
        return Err(no_lineseg_rejection(
            Reject::LegacyGeometryMalformed,
            fallback_geometry,
        ));
    }
    if surface.model_line_seg_count != 0 {
        return Err(no_lineseg_rejection(
            Reject::ModelLineSegPresent,
            fallback_geometry,
        ));
    }
    if surface.frame_interval_count != 1 {
        return Err(no_lineseg_rejection(
            Reject::FrameIntervalCountUnsupported,
            fallback_geometry,
        ));
    }
    if surface.edit_reflow {
        return Err(no_lineseg_rejection(
            Reject::EditReflowUnsupported,
            fallback_geometry,
        ));
    }
    if surface.stored_prefix {
        return Err(no_lineseg_rejection(
            Reject::StoredPrefixUnsupported,
            fallback_geometry,
        ));
    }
    if surface.split_cell {
        return Err(no_lineseg_rejection(
            Reject::SplitCellUnsupported,
            fallback_geometry,
        ));
    }
    if surface.has_inline_control {
        return Err(no_lineseg_rejection(
            Reject::InlineControlUnsupported,
            fallback_geometry,
        ));
    }
    if outcome.trace.disposition != HorizontalShapingLineDisposition::DormantQualified
        || outcome.trace.product_published
    {
        return Err(no_lineseg_rejection(
            Reject::OutcomeNotQualified,
            fallback_geometry,
        ));
    }
    if outcome.lines.len() != 1 || outcome.lines[0].target_runs.len() != 1 {
        return Err(no_lineseg_rejection(
            Reject::TargetCountUnsupported,
            fallback_geometry,
        ));
    }

    let line_selection_measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let mapped = map_horizontal_shaping_emitted_run(&outcome, candidate).map_err(|reason| {
        no_lineseg_rejection(Reject::EmittedRunRejected(reason), fallback_geometry)
    })?;
    let bbox_measurement =
        Arc::clone(mapped.decision.measurement().ok_or_else(|| {
            no_lineseg_rejection(Reject::OwnerIdentityMismatch, fallback_geometry)
        })?);
    let next_origin_measurement = Arc::clone(&bbox_measurement);
    let sidecar_decision =
        certify_horizontal_shaping_mapped_run(context, &mapped).map_err(|reason| {
            no_lineseg_rejection(Reject::ReplaySourceRejected(reason), fallback_geometry)
        })?;
    let Some(sidecar_measurement) = sidecar_decision.measurement() else {
        return Err(no_lineseg_rejection(
            Reject::OwnerIdentityMismatch,
            fallback_geometry,
        ));
    };
    if !Arc::ptr_eq(&line_selection_measurement, &bbox_measurement)
        || !Arc::ptr_eq(&line_selection_measurement, &next_origin_measurement)
        || !Arc::ptr_eq(&line_selection_measurement, sidecar_measurement)
        || !same_width(
            mapped.line_width_px,
            line_selection_measurement.total_advance_px,
        )
        || !same_width(
            mapped.bbox_width_px,
            line_selection_measurement.total_advance_px,
        )
        || !same_width(
            mapped.next_origin_x_px - candidate.origin_x_px,
            line_selection_measurement.total_advance_px,
        )
    {
        return Err(no_lineseg_rejection(
            Reject::OwnerIdentityMismatch,
            fallback_geometry,
        ));
    }

    Ok(HorizontalShapingNoLineSegOwnerTransaction {
        outcome,
        line_selection_measurement,
        bbox_measurement,
        next_origin_measurement,
        sidecar_decision,
        line_width_px: mapped.line_width_px,
        bbox_width_px: mapped.bbox_width_px,
        next_origin_x_px: mapped.next_origin_x_px,
        fallback_geometry,
    })
}

/// Project a paragraph scalar range into the renderer's three coordinate
/// systems. The scan is bounded before either offset vector can grow.
pub(crate) fn project_horizontal_shaping_run_range(
    paragraph_text: &str,
    scalar_start: usize,
    scalar_end: usize,
) -> Result<HorizontalShapingRunRange, HorizontalShapingEmittedRunRejectReason> {
    if scalar_start >= scalar_end {
        return Err(HorizontalShapingEmittedRunRejectReason::EmptyRun);
    }
    let mut utf8_offsets = Vec::new();
    let mut utf16_offsets = Vec::new();
    utf8_offsets.push(0usize);
    utf16_offsets.push(0usize);
    let mut utf16_cursor = 0usize;
    for character in paragraph_text
        .chars()
        .take(super::shaping::MAX_SHAPING_TEXT_CODE_POINTS + 1)
    {
        utf16_cursor = utf16_cursor
            .checked_add(character.len_utf16())
            .ok_or(HorizontalShapingEmittedRunRejectReason::ScalarRangeOverflow)?;
        let next_utf8 = utf8_offsets
            .last()
            .copied()
            .and_then(|offset| offset.checked_add(character.len_utf8()))
            .ok_or(HorizontalShapingEmittedRunRejectReason::ScalarRangeOverflow)?;
        utf8_offsets.push(next_utf8);
        utf16_offsets.push(utf16_cursor);
    }
    if utf8_offsets.len().saturating_sub(1) > super::shaping::MAX_SHAPING_TEXT_CODE_POINTS {
        return Err(HorizontalShapingEmittedRunRejectReason::TextLimitExceeded);
    }
    let (Some(&utf8_start), Some(&utf8_end), Some(&utf16_start), Some(&utf16_end)) = (
        utf8_offsets.get(scalar_start),
        utf8_offsets.get(scalar_end),
        utf16_offsets.get(scalar_start),
        utf16_offsets.get(scalar_end),
    ) else {
        return Err(HorizontalShapingEmittedRunRejectReason::ScalarRangeOverflow);
    };
    Ok(HorizontalShapingRunRange {
        scalar_start,
        scalar_end,
        utf8_start,
        utf8_end,
        utf16_start,
        utf16_end,
    })
}

fn same_width(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1.0e-9 * left.abs().max(right.abs()).max(1.0)
}

/// Reconcile one final emitted run with exactly one full Q2-C target.
///
/// The resulting width and next origin are all derived from the retained
/// measurement once. No `TextRunNode`, bbox, or line is modified in D2.
pub(crate) fn map_horizontal_shaping_emitted_run(
    outcome: &Arc<HorizontalShapingLineOutcome>,
    candidate: HorizontalShapingEmittedRunCandidate<'_>,
) -> Result<HorizontalShapingMappedRun, HorizontalShapingEmittedRunRejectReason> {
    if outcome.trace.disposition != HorizontalShapingLineDisposition::DormantQualified
        || outcome.trace.product_published
    {
        return Err(HorizontalShapingEmittedRunRejectReason::OutcomeNotQualified);
    }
    if candidate.layout_positions_present
        || candidate.display_projection_present
        || !candidate.horizontal_ltr_bidi0
        || candidate.has_field_or_note_split
        || candidate.has_char_overlap
        || candidate.has_border_or_background
        || candidate.has_decoration
    {
        return Err(HorizontalShapingEmittedRunRejectReason::UnsupportedSurface);
    }
    let emitted_scalar_count = candidate
        .emitted_text
        .chars()
        .take(super::shaping::MAX_SHAPING_TEXT_CODE_POINTS + 1)
        .count();
    if emitted_scalar_count > super::shaping::MAX_SHAPING_TEXT_CODE_POINTS {
        return Err(HorizontalShapingEmittedRunRejectReason::TextLimitExceeded);
    }
    if emitted_scalar_count == 0 {
        return Err(HorizontalShapingEmittedRunRejectReason::EmptyRun);
    }
    let scalar_end = candidate
        .scalar_start
        .checked_add(emitted_scalar_count)
        .ok_or(HorizontalShapingEmittedRunRejectReason::ScalarRangeOverflow)?;
    let range = project_horizontal_shaping_run_range(
        candidate.paragraph_text,
        candidate.scalar_start,
        scalar_end,
    )?;
    if candidate
        .paragraph_text
        .get(range.utf8_start..range.utf8_end)
        != Some(candidate.emitted_text)
        || range.utf8_end - range.utf8_start != candidate.emitted_text.len()
        || range.utf16_end - range.utf16_start != candidate.emitted_text.encode_utf16().count()
    {
        return Err(HorizontalShapingEmittedRunRejectReason::TextRangeMismatch);
    }

    let mut matching = outcome.lines.iter().filter_map(|line| {
        line.target_runs
            .iter()
            .find(|target| {
                target.scalar_start == range.scalar_start && target.scalar_end == range.scalar_end
            })
            .map(|target| (line, target))
    });
    let Some((line, target)) = matching.next() else {
        return Err(HorizontalShapingEmittedRunRejectReason::ExactTargetNotFound);
    };
    if matching.next().is_some()
        || line.scalar_start != range.scalar_start
        || line.scalar_end != range.scalar_end
        || line.target_runs.len() != 1
    {
        return Err(HorizontalShapingEmittedRunRejectReason::MixedOrPartialTarget);
    }
    let measurement = &target.measurement;
    if measurement.code_point_count != emitted_scalar_count
        || !same_width(line.width_px, measurement.total_advance_px)
    {
        return Err(HorizontalShapingEmittedRunRejectReason::MeasurementMismatch);
    }
    let Some(trace) = outcome.attempts.iter().rev().find(|trace| {
        trace.disposition == TerminalShapingDisposition::Applied
            && trace.reason.is_none()
            && trace.glyph_count == measurement.applied.glyphs.len()
            && trace.settings_sha256.as_deref()
                == Some(measurement.applied.identity.settings_sha256.as_str())
            && trace.font_source_sha256.as_deref()
                == Some(measurement.applied.identity.font_source_sha256.as_str())
    }) else {
        return Err(HorizontalShapingEmittedRunRejectReason::AppliedTraceNotFound);
    };
    let advance = measurement.total_advance_px;
    let next_origin_x_px = candidate.origin_x_px + advance;
    if !candidate.origin_x_px.is_finite()
        || !advance.is_finite()
        || advance <= 0.0
        || !next_origin_x_px.is_finite()
    {
        return Err(HorizontalShapingEmittedRunRejectReason::GeometryMalformed);
    }
    let decision = Arc::new(HorizontalShapingRunDecision::applied(
        range,
        trace.clone(),
        Arc::clone(measurement),
    ));
    Ok(HorizontalShapingMappedRun {
        node_id: candidate.node_id,
        range,
        line_width_px: advance,
        bbox_width_px: advance,
        next_origin_x_px,
        decision,
    })
}

/// Exercise the D0 ownership boundary with the exact NodeId and reconciled
/// range. Product layout does not call this helper until D4.
pub(crate) fn attach_horizontal_shaping_mapped_run(
    sidecars: &mut HorizontalShapingPageSidecars,
    mapped: &HorizontalShapingMappedRun,
) -> Result<(), HorizontalShapingSidecarRejectReason> {
    sidecars.attach(mapped.node_id, mapped.range, Arc::clone(&mapped.decision))
}

/// D4 activation 직전에 D2 mapping의 exact measurement와 현재 registry source를
/// 다시 대사한다. 인증된 decision이 만들어져도 page sidecar에 attach되기 전에는
/// layout geometry를 바꿀 수 없다.
pub(crate) fn certify_horizontal_shaping_mapped_run(
    context: &HorizontalShapingContext,
    mapped: &HorizontalShapingMappedRun,
) -> Result<Arc<HorizontalShapingRunDecision>, HorizontalShapingReplaySourceCertificateRejectReason>
{
    let measurement = mapped
        .decision
        .measurement()
        .expect("D2 mapped decisions always retain an applied measurement");
    let certificate = context.certify_replay_source(measurement)?;
    Ok(Arc::new(
        HorizontalShapingRunDecision::applied_with_replay_source_certificate(
            mapped.range,
            mapped.decision.trace().clone(),
            Arc::clone(measurement),
            certificate,
        ),
    ))
}
