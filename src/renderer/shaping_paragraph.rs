//! W10-Q2-C dormant cluster-aware paragraph and line-break transaction.
//!
//! 이 모듈은 Q2-B shadow measurement를 paragraph scalar range로 합치고 final-line
//! reshape·bounded convergence를 검증한다. composer·layout·paint는 아직 소비하지 않는다.

use super::kerning::ExactFontSlot;
use super::shaping::{ShapingAttemptTrace, ShapingFeature, ShapingRejectReason};
use super::shaping_context::{
    HorizontalShapingMeasurement, HorizontalShapingRequest, HorizontalShapingTransaction,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::Arc;

pub(crate) const MAX_HORIZONTAL_SHAPING_PARAGRAPH_SEGMENTS: usize = 4_096;
pub(crate) const MAX_HORIZONTAL_SHAPING_LINE_CANDIDATES: usize = 4_097;
pub(crate) const MAX_HORIZONTAL_SHAPING_LINE_ATTEMPTS: usize = 4_096;
pub(crate) const MAX_HORIZONTAL_SHAPING_LINE_RETRIES: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HorizontalShapingFallbackOwner {
    W9K1,
    ExistingK0,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HorizontalShapingParagraphScalarStyle {
    pub slot: ExactFontSlot,
    pub effective_font_size_px: f64,
    pub width_ratio: f64,
    pub letter_spacing_px: f64,
    pub kerning: bool,
    pub bold: bool,
    pub italic: bool,
    pub superscript: bool,
    pub subscript: bool,
}

impl HorizontalShapingParagraphScalarStyle {
    fn same_shaping_identity(self, other: Self) -> bool {
        self.slot == other.slot
            && self.effective_font_size_px.to_bits() == other.effective_font_size_px.to_bits()
            && self.width_ratio.to_bits() == other.width_ratio.to_bits()
            && self.letter_spacing_px.to_bits() == other.letter_spacing_px.to_bits()
            && self.kerning == other.kerning
            && self.bold == other.bold
            && self.italic == other.italic
            && self.superscript == other.superscript
            && self.subscript == other.subscript
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingParagraphRequest<'a> {
    pub attempt_id_base: u32,
    pub text: &'a str,
    pub fallback_positions: &'a [f64],
    pub scalar_styles: &'a [HorizontalShapingParagraphScalarStyle],
    pub hard_boundaries: &'a [bool],
    pub fallback_owner: HorizontalShapingFallbackOwner,
    pub model_text_matches_shaping_text: bool,
    pub horizontal_ltr_bidi0: bool,
    pub condense_min_space: u8,
    pub has_inline_controls: bool,
    pub has_tabs: bool,
    pub has_rotation: bool,
    pub has_char_overlap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HorizontalShapingActivationDisposition {
    Eligible,
    NotTarget,
    Unsupported,
    Malformed,
    BoundedLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HorizontalShapingActivationReason {
    NoComplexRequiredText,
    TextCodePointLimitExceeded,
    ParagraphInputLengthMismatch,
    FallbackPositionNonFinite,
    FallbackPositionNonMonotonic,
    BidiAuthorityPending,
    DisplayProjectionNotSupported,
    SyntheticStyleNotSupported,
    SuperscriptSubscriptNotSupported,
    LetterSpacingSemanticsPending,
    CondenseSemanticsPending,
    InlineControlNotSupported,
    TabNotSupported,
    RotationNotSupported,
    CharOverlapNotSupported,
    InvalidScale,
    HardBoundaryCrossed,
    StyleBoundaryCrossed,
    SegmentLimitExceeded,
    ExactSourceUnavailable,
    ShapingRejected,
    MissingGlyph,
    ClusterBoundaryMismatch,
    CandidateLimitExceeded,
    CandidateInputMalformed,
    AvailableWidthMalformed,
    AttemptLimitExceeded,
    LineDecisionNotConverged,
    PublicationOwnerPending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HorizontalShapingTargetRange {
    pub scalar_start: usize,
    pub scalar_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HorizontalShapingActivationDecision {
    pub disposition: HorizontalShapingActivationDisposition,
    pub reason: Option<HorizontalShapingActivationReason>,
    pub code_point_count: usize,
    pub target_scalar_count: usize,
    pub target_ranges: Vec<HorizontalShapingTargetRange>,
}

impl HorizontalShapingActivationDecision {
    pub(crate) fn is_eligible(&self) -> bool {
        self.disposition == HorizontalShapingActivationDisposition::Eligible
            && self.reason.is_none()
            && !self.target_ranges.is_empty()
    }
}

#[derive(Debug, Clone)]
struct EligibleSegment {
    range: Range<usize>,
    style: HorizontalShapingParagraphScalarStyle,
}

#[derive(Debug, Clone, PartialEq)]
struct ParagraphWidthAtom {
    scalar_start: usize,
    scalar_end: usize,
    width_px: f64,
    shaped: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingParagraphTargetMeasurement {
    pub scalar_start: usize,
    pub scalar_end: usize,
    pub style: HorizontalShapingParagraphScalarStyle,
    pub measurement: Arc<HorizontalShapingMeasurement>,
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingParagraphMeasurement {
    pub activation: HorizontalShapingActivationDecision,
    pub fallback_owner: HorizontalShapingFallbackOwner,
    pub code_point_count: usize,
    pub fallback_total_width_px: f64,
    pub total_width_px: f64,
    pub targets: Vec<HorizontalShapingParagraphTargetMeasurement>,
    fallback_positions: Vec<f64>,
    atoms: Vec<ParagraphWidthAtom>,
}

impl HorizontalShapingParagraphMeasurement {
    pub(crate) fn is_boundary(&self, scalar_index: usize) -> bool {
        scalar_index == 0
            || scalar_index == self.code_point_count
            || self
                .atoms
                .iter()
                .any(|atom| atom.scalar_start == scalar_index || atom.scalar_end == scalar_index)
    }

    pub(crate) fn range_width(&self, scalar_start: usize, scalar_end: usize) -> Option<f64> {
        if scalar_start > scalar_end
            || scalar_end > self.code_point_count
            || !self.is_boundary(scalar_start)
            || !self.is_boundary(scalar_end)
        {
            return None;
        }
        if scalar_start == scalar_end {
            return Some(0.0);
        }
        let mut cursor = scalar_start;
        let mut width = 0.0;
        for atom in self
            .atoms
            .iter()
            .filter(|atom| atom.scalar_end > scalar_start && atom.scalar_start < scalar_end)
        {
            if atom.scalar_start != cursor || atom.scalar_end > scalar_end {
                return None;
            }
            width += atom.width_px;
            cursor = atom.scalar_end;
        }
        (cursor == scalar_end && width.is_finite()).then_some(width)
    }

    pub(crate) fn target_cluster_boundaries(&self) -> impl Iterator<Item = usize> + '_ {
        self.atoms
            .iter()
            .filter(|atom| atom.shaped)
            .map(|atom| atom.scalar_end)
    }

    fn fallback_range_width(&self, scalar_start: usize, scalar_end: usize) -> Option<f64> {
        let start = *self.fallback_positions.get(scalar_start)?;
        let end = *self.fallback_positions.get(scalar_end)?;
        let width = end - start;
        (width.is_finite() && width >= 0.0).then_some(width)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingParagraphOutcome {
    pub activation: HorizontalShapingActivationDecision,
    pub attempts: Vec<ShapingAttemptTrace>,
    pub measurement: Option<Arc<HorizontalShapingParagraphMeasurement>>,
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingLineRequest<'a> {
    pub paragraph: HorizontalShapingParagraphRequest<'a>,
    pub candidate_boundaries: &'a [usize],
    pub available_widths_px: &'a [f64],
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingFinalTargetRun {
    pub scalar_start: usize,
    pub scalar_end: usize,
    pub measurement: Arc<HorizontalShapingMeasurement>,
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingFinalLine {
    pub scalar_start: usize,
    pub scalar_end: usize,
    pub width_px: f64,
    pub target_runs: Vec<HorizontalShapingFinalTargetRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HorizontalShapingLineDisposition {
    DormantQualified,
    RolledBack,
    NotTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HorizontalShapingLineTrace {
    pub disposition: HorizontalShapingLineDisposition,
    pub reason: Option<HorizontalShapingActivationReason>,
    pub fallback_owner: HorizontalShapingFallbackOwner,
    pub retry_count: usize,
    pub attempt_count: usize,
    pub line_ranges: Vec<HorizontalShapingTargetRange>,
    pub line_widths_px: Vec<f64>,
    pub product_published: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingLineOutcome {
    pub trace: HorizontalShapingLineTrace,
    pub attempts: Vec<ShapingAttemptTrace>,
    pub paragraph_measurement: Option<Arc<HorizontalShapingParagraphMeasurement>>,
    pub lines: Vec<HorizontalShapingFinalLine>,
}

struct AttemptBudget {
    next_id: u32,
    id_exhausted: bool,
    count: usize,
    traces: Vec<ShapingAttemptTrace>,
}

impl AttemptBudget {
    fn new(first_id: u32) -> Self {
        Self {
            next_id: first_id,
            id_exhausted: false,
            count: 0,
            traces: Vec::new(),
        }
    }

    fn take_id(&mut self) -> Result<u32, HorizontalShapingActivationReason> {
        if self.count >= MAX_HORIZONTAL_SHAPING_LINE_ATTEMPTS || self.id_exhausted {
            return Err(HorizontalShapingActivationReason::AttemptLimitExceeded);
        }
        let id = self.next_id;
        if id == u32::MAX {
            self.id_exhausted = true;
        } else {
            self.next_id = id + 1;
        }
        self.count += 1;
        Ok(id)
    }

    fn record(&mut self, trace: ShapingAttemptTrace) {
        self.traces.push(trace);
    }
}

fn is_old_hangul_jamo(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x11ff | 0xa960..=0xa97f | 0xd7b0..=0xd7ff
    )
}

fn is_initial_hangul_scalar(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x11ff | 0xa960..=0xa97f | 0xd7b0..=0xd7ff | 0xac00..=0xd7a3
    )
}

fn valid_scale(style: HorizontalShapingParagraphScalarStyle) -> bool {
    style.effective_font_size_px.is_finite()
        && style.effective_font_size_px > 0.0
        && style.effective_font_size_px <= 4_096.0
        && style.width_ratio.is_finite()
        && style.width_ratio > 0.0
        && style.width_ratio <= 16.0
}

fn decision(
    disposition: HorizontalShapingActivationDisposition,
    reason: Option<HorizontalShapingActivationReason>,
    code_point_count: usize,
    segments: &[EligibleSegment],
) -> HorizontalShapingActivationDecision {
    let target_ranges = segments
        .iter()
        .map(|segment| HorizontalShapingTargetRange {
            scalar_start: segment.range.start,
            scalar_end: segment.range.end,
        })
        .collect::<Vec<_>>();
    HorizontalShapingActivationDecision {
        disposition,
        reason,
        code_point_count,
        target_scalar_count: target_ranges
            .iter()
            .map(|range| range.scalar_end - range.scalar_start)
            .sum(),
        target_ranges,
    }
}

fn rejected_decision(
    disposition: HorizontalShapingActivationDisposition,
    reason: HorizontalShapingActivationReason,
    code_point_count: usize,
) -> (HorizontalShapingActivationDecision, Vec<EligibleSegment>) {
    (
        decision(disposition, Some(reason), code_point_count, &[]),
        Vec::new(),
    )
}

fn eligible_segments(
    request: &HorizontalShapingParagraphRequest<'_>,
) -> (HorizontalShapingActivationDecision, Vec<EligibleSegment>) {
    let characters = request.text.chars().collect::<Vec<_>>();
    let code_point_count = characters.len();
    if code_point_count > super::shaping::MAX_SHAPING_TEXT_CODE_POINTS {
        return rejected_decision(
            HorizontalShapingActivationDisposition::BoundedLimit,
            HorizontalShapingActivationReason::TextCodePointLimitExceeded,
            code_point_count,
        );
    }
    if request.scalar_styles.len() != code_point_count
        || request.fallback_positions.len() != code_point_count.saturating_add(1)
        || request.hard_boundaries.len() != code_point_count.saturating_add(1)
    {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Malformed,
            HorizontalShapingActivationReason::ParagraphInputLengthMismatch,
            code_point_count,
        );
    }
    if request
        .fallback_positions
        .iter()
        .any(|value| !value.is_finite())
    {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Malformed,
            HorizontalShapingActivationReason::FallbackPositionNonFinite,
            code_point_count,
        );
    }
    if request
        .fallback_positions
        .windows(2)
        .any(|pair| pair[1] < pair[0])
    {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Malformed,
            HorizontalShapingActivationReason::FallbackPositionNonMonotonic,
            code_point_count,
        );
    }
    if !characters.iter().copied().any(is_old_hangul_jamo) {
        return rejected_decision(
            HorizontalShapingActivationDisposition::NotTarget,
            HorizontalShapingActivationReason::NoComplexRequiredText,
            code_point_count,
        );
    }
    for (index, pair) in characters.windows(2).enumerate() {
        if pair.iter().copied().all(is_old_hangul_jamo) {
            if request.hard_boundaries[index + 1] {
                return rejected_decision(
                    HorizontalShapingActivationDisposition::Unsupported,
                    HorizontalShapingActivationReason::HardBoundaryCrossed,
                    code_point_count,
                );
            }
            if !request.scalar_styles[index].same_shaping_identity(request.scalar_styles[index + 1])
            {
                return rejected_decision(
                    HorizontalShapingActivationDisposition::Unsupported,
                    HorizontalShapingActivationReason::StyleBoundaryCrossed,
                    code_point_count,
                );
            }
        }
    }
    if !request.horizontal_ltr_bidi0 {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Unsupported,
            HorizontalShapingActivationReason::BidiAuthorityPending,
            code_point_count,
        );
    }
    if !request.model_text_matches_shaping_text {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Unsupported,
            HorizontalShapingActivationReason::DisplayProjectionNotSupported,
            code_point_count,
        );
    }
    if request.condense_min_space != 0 {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Unsupported,
            HorizontalShapingActivationReason::CondenseSemanticsPending,
            code_point_count,
        );
    }
    for (flag, reason) in [
        (
            request.has_inline_controls,
            HorizontalShapingActivationReason::InlineControlNotSupported,
        ),
        (
            request.has_tabs,
            HorizontalShapingActivationReason::TabNotSupported,
        ),
        (
            request.has_rotation,
            HorizontalShapingActivationReason::RotationNotSupported,
        ),
        (
            request.has_char_overlap,
            HorizontalShapingActivationReason::CharOverlapNotSupported,
        ),
    ] {
        if flag {
            return rejected_decision(
                HorizontalShapingActivationDisposition::Unsupported,
                reason,
                code_point_count,
            );
        }
    }

    let mut segments = Vec::new();
    let mut index = 0;
    while index < code_point_count {
        if !is_initial_hangul_scalar(characters[index]) {
            index += 1;
            continue;
        }
        let start = index;
        let style = request.scalar_styles[index];
        let mut has_old_jamo = is_old_hangul_jamo(characters[index]);
        index += 1;
        while index < code_point_count
            && is_initial_hangul_scalar(characters[index])
            && !request.hard_boundaries[index]
            && style.same_shaping_identity(request.scalar_styles[index])
        {
            has_old_jamo |= is_old_hangul_jamo(characters[index]);
            index += 1;
        }
        if !has_old_jamo {
            continue;
        }
        if style.bold || style.italic {
            return rejected_decision(
                HorizontalShapingActivationDisposition::Unsupported,
                HorizontalShapingActivationReason::SyntheticStyleNotSupported,
                code_point_count,
            );
        }
        if style.superscript || style.subscript {
            return rejected_decision(
                HorizontalShapingActivationDisposition::Unsupported,
                HorizontalShapingActivationReason::SuperscriptSubscriptNotSupported,
                code_point_count,
            );
        }
        if style.letter_spacing_px != 0.0 {
            return rejected_decision(
                HorizontalShapingActivationDisposition::Unsupported,
                HorizontalShapingActivationReason::LetterSpacingSemanticsPending,
                code_point_count,
            );
        }
        if !valid_scale(style) {
            return rejected_decision(
                HorizontalShapingActivationDisposition::Malformed,
                HorizontalShapingActivationReason::InvalidScale,
                code_point_count,
            );
        }
        if segments.len() >= MAX_HORIZONTAL_SHAPING_PARAGRAPH_SEGMENTS {
            return rejected_decision(
                HorizontalShapingActivationDisposition::BoundedLimit,
                HorizontalShapingActivationReason::SegmentLimitExceeded,
                code_point_count,
            );
        }
        segments.push(EligibleSegment {
            range: start..index,
            style,
        });
    }

    if segments.is_empty() {
        return rejected_decision(
            HorizontalShapingActivationDisposition::Unsupported,
            HorizontalShapingActivationReason::BidiAuthorityPending,
            code_point_count,
        );
    }
    (
        decision(
            HorizontalShapingActivationDisposition::Eligible,
            None,
            code_point_count,
            &segments,
        ),
        segments,
    )
}

pub(crate) fn decide_horizontal_shaping_activation(
    request: &HorizontalShapingParagraphRequest<'_>,
) -> HorizontalShapingActivationDecision {
    eligible_segments(request).0
}

fn scalar_utf8_offsets(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect()
}

fn shape_segment(
    transaction: &mut HorizontalShapingTransaction<'_>,
    text: &str,
    scalar_start: usize,
    scalar_end: usize,
    style: HorizontalShapingParagraphScalarStyle,
    utf8_offsets: &[usize],
    budget: &mut AttemptBudget,
) -> Result<Arc<HorizontalShapingMeasurement>, HorizontalShapingActivationReason> {
    let attempt_id = budget.take_id()?;
    let feature = [ShapingFeature {
        tag: "kern".to_string(),
        value: u32::from(style.kerning),
    }];
    let segment_text = text
        .get(utf8_offsets[scalar_start]..utf8_offsets[scalar_end])
        .ok_or(HorizontalShapingActivationReason::ParagraphInputLengthMismatch)?;
    let outcome = transaction.shadow_measure(&HorizontalShapingRequest {
        attempt_id,
        slot: style.slot,
        text: segment_text,
        effective_font_size_px: style.effective_font_size_px,
        width_ratio: style.width_ratio,
        script: Some("Hang"),
        language: Some("ko"),
        features: &feature,
    });
    let rejection_reason = match outcome.trace.reason {
        Some(ShapingRejectReason::SourceUnavailable) => {
            HorizontalShapingActivationReason::ExactSourceUnavailable
        }
        _ => HorizontalShapingActivationReason::ShapingRejected,
    };
    budget.record(outcome.trace.clone());
    let measurement = outcome.measurement.ok_or(rejection_reason)?;
    if measurement
        .applied
        .glyphs
        .iter()
        .any(|glyph| glyph.glyph_id == 0)
    {
        return Err(HorizontalShapingActivationReason::MissingGlyph);
    }
    Ok(measurement)
}

fn prepare_paragraph_with_budget(
    transaction: &mut HorizontalShapingTransaction<'_>,
    request: &HorizontalShapingParagraphRequest<'_>,
    budget: &mut AttemptBudget,
) -> Result<Arc<HorizontalShapingParagraphMeasurement>, HorizontalShapingActivationDecision> {
    let (activation, segments) = eligible_segments(request);
    if !activation.is_eligible() {
        return Err(activation);
    }
    let utf8_offsets = scalar_utf8_offsets(request.text);
    let mut targets = Vec::with_capacity(segments.len());
    for segment in &segments {
        let measurement = match shape_segment(
            transaction,
            request.text,
            segment.range.start,
            segment.range.end,
            segment.style,
            &utf8_offsets,
            budget,
        ) {
            Ok(value) => value,
            Err(reason) => {
                return Err(decision(
                    HorizontalShapingActivationDisposition::Unsupported,
                    Some(reason),
                    activation.code_point_count,
                    &[],
                ));
            }
        };
        targets.push(HorizontalShapingParagraphTargetMeasurement {
            scalar_start: segment.range.start,
            scalar_end: segment.range.end,
            style: segment.style,
            measurement,
        });
    }

    let mut atoms = Vec::new();
    let mut scalar_index = 0;
    let mut target_index = 0;
    while scalar_index < activation.code_point_count {
        if let Some(target) = targets
            .get(target_index)
            .filter(|target| target.scalar_start == scalar_index)
        {
            for cluster in &target.measurement.clusters {
                atoms.push(ParagraphWidthAtom {
                    scalar_start: target.scalar_start + cluster.scalar_start,
                    scalar_end: target.scalar_start + cluster.scalar_end,
                    width_px: cluster.advance_px,
                    shaped: true,
                });
            }
            scalar_index = target.scalar_end;
            target_index += 1;
            continue;
        }
        let width_px =
            request.fallback_positions[scalar_index + 1] - request.fallback_positions[scalar_index];
        atoms.push(ParagraphWidthAtom {
            scalar_start: scalar_index,
            scalar_end: scalar_index + 1,
            width_px,
            shaped: false,
        });
        scalar_index += 1;
    }
    if atoms
        .windows(2)
        .any(|pair| pair[0].scalar_end != pair[1].scalar_start)
        || atoms.first().is_none_or(|atom| atom.scalar_start != 0)
        || atoms
            .last()
            .is_none_or(|atom| atom.scalar_end != activation.code_point_count)
    {
        return Err(decision(
            HorizontalShapingActivationDisposition::Malformed,
            Some(HorizontalShapingActivationReason::ClusterBoundaryMismatch),
            activation.code_point_count,
            &[],
        ));
    }
    let total_width_px = atoms.iter().map(|atom| atom.width_px).sum::<f64>();
    if !total_width_px.is_finite() || total_width_px < 0.0 {
        return Err(decision(
            HorizontalShapingActivationDisposition::Malformed,
            Some(HorizontalShapingActivationReason::ClusterBoundaryMismatch),
            activation.code_point_count,
            &[],
        ));
    }
    let fallback_total_width_px =
        request.fallback_positions[activation.code_point_count] - request.fallback_positions[0];
    Ok(Arc::new(HorizontalShapingParagraphMeasurement {
        activation,
        fallback_owner: request.fallback_owner,
        code_point_count: utf8_offsets.len().saturating_sub(1),
        fallback_total_width_px,
        total_width_px,
        targets,
        fallback_positions: request.fallback_positions.to_vec(),
        atoms,
    }))
}

pub(crate) fn prepare_horizontal_shaping_paragraph(
    transaction: &mut HorizontalShapingTransaction<'_>,
    request: &HorizontalShapingParagraphRequest<'_>,
) -> HorizontalShapingParagraphOutcome {
    let mut budget = AttemptBudget::new(request.attempt_id_base);
    match prepare_paragraph_with_budget(transaction, request, &mut budget) {
        Ok(measurement) => HorizontalShapingParagraphOutcome {
            activation: measurement.activation.clone(),
            attempts: budget.traces,
            measurement: Some(measurement),
        },
        Err(activation) => HorizontalShapingParagraphOutcome {
            activation,
            attempts: budget.traces,
            measurement: None,
        },
    }
}

fn legal_boundaries(
    measurement: &HorizontalShapingParagraphMeasurement,
    candidates: &[usize],
) -> Result<Vec<usize>, HorizontalShapingActivationReason> {
    if candidates.len() > MAX_HORIZONTAL_SHAPING_LINE_CANDIDATES {
        return Err(HorizontalShapingActivationReason::CandidateLimitExceeded);
    }
    if candidates.windows(2).any(|pair| pair[0] >= pair[1])
        || candidates
            .iter()
            .any(|candidate| *candidate > measurement.code_point_count)
    {
        return Err(HorizontalShapingActivationReason::CandidateInputMalformed);
    }
    let mut boundaries = candidates
        .iter()
        .copied()
        .filter(|candidate| measurement.is_boundary(*candidate))
        .collect::<BTreeSet<_>>();
    boundaries.insert(0);
    boundaries.insert(measurement.code_point_count);
    boundaries.extend(measurement.target_cluster_boundaries());
    Ok(boundaries.into_iter().collect())
}

fn fallback_boundaries(code_point_count: usize, candidates: &[usize]) -> Vec<usize> {
    let mut boundaries = candidates
        .iter()
        .copied()
        .filter(|candidate| *candidate <= code_point_count)
        .collect::<BTreeSet<_>>();
    boundaries.insert(0);
    boundaries.insert(code_point_count);
    boundaries.into_iter().collect()
}

fn width_limit(widths: &[f64], line_index: usize) -> Option<f64> {
    widths
        .get(line_index)
        .or_else(|| widths.last())
        .copied()
        .filter(|width| width.is_finite() && *width > 0.0)
}

fn validate_available_widths(widths: &[f64]) -> Result<(), HorizontalShapingActivationReason> {
    if widths.is_empty()
        || widths.len() > super::shaping::MAX_SHAPING_TEXT_CODE_POINTS
        || widths
            .iter()
            .any(|width| !width.is_finite() || *width <= 0.0 || *width > 1.0e9)
    {
        Err(HorizontalShapingActivationReason::AvailableWidthMalformed)
    } else {
        Ok(())
    }
}

fn choose_ranges<F>(
    code_point_count: usize,
    boundaries: &[usize],
    available_widths: &[f64],
    mut range_width: F,
) -> Result<Vec<Range<usize>>, HorizontalShapingActivationReason>
where
    F: FnMut(usize, usize) -> Result<f64, HorizontalShapingActivationReason>,
{
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < code_point_count {
        let limit = width_limit(available_widths, ranges.len())
            .ok_or(HorizontalShapingActivationReason::AvailableWidthMalformed)?;
        let ends = boundaries
            .iter()
            .copied()
            .filter(|end| *end > start)
            .collect::<Vec<_>>();
        let Some(first_end) = ends.first().copied() else {
            return Err(HorizontalShapingActivationReason::CandidateInputMalformed);
        };
        let mut selected = None;
        for end in ends.iter().copied() {
            let width = range_width(start, end)?;
            if width <= limit {
                selected = Some(end);
            } else {
                break;
            }
        }
        let end = selected.unwrap_or(first_end);
        ranges.push(start..end);
        start = end;
    }
    Ok(ranges)
}

fn shape_final_range(
    transaction: &mut HorizontalShapingTransaction<'_>,
    request: &HorizontalShapingParagraphRequest<'_>,
    paragraph: &HorizontalShapingParagraphMeasurement,
    range: Range<usize>,
    utf8_offsets: &[usize],
    budget: &mut AttemptBudget,
) -> Result<HorizontalShapingFinalLine, HorizontalShapingActivationReason> {
    let mut width_px = paragraph
        .fallback_range_width(range.start, range.end)
        .ok_or(HorizontalShapingActivationReason::ClusterBoundaryMismatch)?;
    let mut target_runs = Vec::new();
    for target in &paragraph.targets {
        let start = target.scalar_start.max(range.start);
        let end = target.scalar_end.min(range.end);
        if start >= end {
            continue;
        }
        if target
            .measurement
            .range_width(start - target.scalar_start, end - target.scalar_start)
            .is_none()
        {
            return Err(HorizontalShapingActivationReason::ClusterBoundaryMismatch);
        }
        let fallback_width = paragraph
            .fallback_range_width(start, end)
            .ok_or(HorizontalShapingActivationReason::ClusterBoundaryMismatch)?;
        let measurement = shape_segment(
            transaction,
            request.text,
            start,
            end,
            target.style,
            utf8_offsets,
            budget,
        )?;
        width_px = width_px - fallback_width + measurement.total_advance_px;
        target_runs.push(HorizontalShapingFinalTargetRun {
            scalar_start: start,
            scalar_end: end,
            measurement,
        });
    }
    if !width_px.is_finite() || width_px < 0.0 {
        return Err(HorizontalShapingActivationReason::ClusterBoundaryMismatch);
    }
    Ok(HorizontalShapingFinalLine {
        scalar_start: range.start,
        scalar_end: range.end,
        width_px,
        target_runs,
    })
}

fn choose_final_lines(
    transaction: &mut HorizontalShapingTransaction<'_>,
    request: &HorizontalShapingParagraphRequest<'_>,
    paragraph: &HorizontalShapingParagraphMeasurement,
    boundaries: &[usize],
    available_widths: &[f64],
    budget: &mut AttemptBudget,
) -> Result<Vec<HorizontalShapingFinalLine>, HorizontalShapingActivationReason> {
    let utf8_offsets = scalar_utf8_offsets(request.text);
    let mut chosen_cache =
        std::collections::HashMap::<(usize, usize), HorizontalShapingFinalLine>::new();
    let ranges = choose_ranges(
        paragraph.code_point_count,
        boundaries,
        available_widths,
        |start, end| {
            if let Some(line) = chosen_cache.get(&(start, end)) {
                return Ok(line.width_px);
            }
            let line = shape_final_range(
                transaction,
                request,
                paragraph,
                start..end,
                &utf8_offsets,
                budget,
            )?;
            let width = line.width_px;
            chosen_cache.insert((start, end), line);
            Ok(width)
        },
    )?;
    ranges
        .into_iter()
        .map(|range| {
            chosen_cache
                .remove(&(range.start, range.end))
                .ok_or(HorizontalShapingActivationReason::ClusterBoundaryMismatch)
        })
        .collect()
}

fn fallback_lines(
    request: &HorizontalShapingParagraphRequest<'_>,
    candidates: &[usize],
    available_widths: &[f64],
) -> Vec<HorizontalShapingFinalLine> {
    let count = request.text.chars().count();
    let boundaries = fallback_boundaries(count, candidates);
    choose_ranges(count, &boundaries, available_widths, |start, end| {
        let start_px = request
            .fallback_positions
            .get(start)
            .copied()
            .ok_or(HorizontalShapingActivationReason::ParagraphInputLengthMismatch)?;
        let end_px = request
            .fallback_positions
            .get(end)
            .copied()
            .ok_or(HorizontalShapingActivationReason::ParagraphInputLengthMismatch)?;
        Ok(end_px - start_px)
    })
    .unwrap_or_else(|_| (count > 0).then_some(0..count).into_iter().collect())
    .into_iter()
    .map(|range| {
        let width_px = request
            .fallback_positions
            .get(range.end)
            .zip(request.fallback_positions.get(range.start))
            .map(|(end, start)| end - start)
            .unwrap_or_default();
        HorizontalShapingFinalLine {
            scalar_start: range.start,
            scalar_end: range.end,
            width_px,
            target_runs: Vec::new(),
        }
    })
    .collect()
}

fn line_trace(
    disposition: HorizontalShapingLineDisposition,
    reason: Option<HorizontalShapingActivationReason>,
    fallback_owner: HorizontalShapingFallbackOwner,
    retry_count: usize,
    attempt_count: usize,
    lines: &[HorizontalShapingFinalLine],
) -> HorizontalShapingLineTrace {
    HorizontalShapingLineTrace {
        disposition,
        reason,
        fallback_owner,
        retry_count,
        attempt_count,
        line_ranges: lines
            .iter()
            .map(|line| HorizontalShapingTargetRange {
                scalar_start: line.scalar_start,
                scalar_end: line.scalar_end,
            })
            .collect(),
        line_widths_px: lines.iter().map(|line| line.width_px).collect(),
        product_published: false,
    }
}

pub(crate) fn run_horizontal_shaping_line_transaction(
    transaction: &mut HorizontalShapingTransaction<'_>,
    request: &HorizontalShapingLineRequest<'_>,
) -> HorizontalShapingLineOutcome {
    let mut budget = AttemptBudget::new(request.paragraph.attempt_id_base);
    if let Err(reason) = validate_available_widths(request.available_widths_px) {
        let lines = fallback_lines(
            &request.paragraph,
            request.candidate_boundaries,
            request.available_widths_px,
        );
        return HorizontalShapingLineOutcome {
            trace: line_trace(
                HorizontalShapingLineDisposition::RolledBack,
                Some(reason),
                request.paragraph.fallback_owner,
                0,
                budget.count,
                &lines,
            ),
            attempts: budget.traces,
            paragraph_measurement: None,
            lines,
        };
    }
    let paragraph =
        match prepare_paragraph_with_budget(transaction, &request.paragraph, &mut budget) {
            Ok(paragraph) => paragraph,
            Err(activation) => {
                let lines = fallback_lines(
                    &request.paragraph,
                    request.candidate_boundaries,
                    request.available_widths_px,
                );
                let disposition = if activation.disposition
                    == HorizontalShapingActivationDisposition::NotTarget
                {
                    HorizontalShapingLineDisposition::NotTarget
                } else {
                    HorizontalShapingLineDisposition::RolledBack
                };
                return HorizontalShapingLineOutcome {
                    trace: line_trace(
                        disposition,
                        activation.reason,
                        request.paragraph.fallback_owner,
                        0,
                        budget.count,
                        &lines,
                    ),
                    attempts: budget.traces,
                    paragraph_measurement: None,
                    lines,
                };
            }
        };
    let boundaries = match legal_boundaries(&paragraph, request.candidate_boundaries) {
        Ok(boundaries) => boundaries,
        Err(reason) => {
            let lines = fallback_lines(
                &request.paragraph,
                request.candidate_boundaries,
                request.available_widths_px,
            );
            return HorizontalShapingLineOutcome {
                trace: line_trace(
                    HorizontalShapingLineDisposition::RolledBack,
                    Some(reason),
                    request.paragraph.fallback_owner,
                    0,
                    budget.count,
                    &lines,
                ),
                attempts: budget.traces,
                paragraph_measurement: Some(paragraph),
                lines,
            };
        }
    };
    let initial_ranges = match choose_ranges(
        paragraph.code_point_count,
        &boundaries,
        request.available_widths_px,
        |start, end| {
            paragraph
                .range_width(start, end)
                .ok_or(HorizontalShapingActivationReason::ClusterBoundaryMismatch)
        },
    ) {
        Ok(ranges) => ranges,
        Err(reason) => {
            let lines = fallback_lines(
                &request.paragraph,
                request.candidate_boundaries,
                request.available_widths_px,
            );
            return HorizontalShapingLineOutcome {
                trace: line_trace(
                    HorizontalShapingLineDisposition::RolledBack,
                    Some(reason),
                    request.paragraph.fallback_owner,
                    0,
                    budget.count,
                    &lines,
                ),
                attempts: budget.traces,
                paragraph_measurement: Some(paragraph),
                lines,
            };
        }
    };
    let mut previous_ranges = initial_ranges;
    for retry_count in 0..=MAX_HORIZONTAL_SHAPING_LINE_RETRIES {
        let lines = match choose_final_lines(
            transaction,
            &request.paragraph,
            &paragraph,
            &boundaries,
            request.available_widths_px,
            &mut budget,
        ) {
            Ok(lines) => lines,
            Err(reason) => {
                let lines = fallback_lines(
                    &request.paragraph,
                    request.candidate_boundaries,
                    request.available_widths_px,
                );
                return HorizontalShapingLineOutcome {
                    trace: line_trace(
                        HorizontalShapingLineDisposition::RolledBack,
                        Some(reason),
                        request.paragraph.fallback_owner,
                        retry_count,
                        budget.count,
                        &lines,
                    ),
                    attempts: budget.traces,
                    paragraph_measurement: Some(paragraph),
                    lines,
                };
            }
        };
        let ranges = lines
            .iter()
            .map(|line| line.scalar_start..line.scalar_end)
            .collect::<Vec<_>>();
        if ranges == previous_ranges {
            return HorizontalShapingLineOutcome {
                trace: line_trace(
                    HorizontalShapingLineDisposition::DormantQualified,
                    Some(HorizontalShapingActivationReason::PublicationOwnerPending),
                    request.paragraph.fallback_owner,
                    retry_count,
                    budget.count,
                    &lines,
                ),
                attempts: budget.traces,
                paragraph_measurement: Some(paragraph),
                lines,
            };
        }
        previous_ranges = ranges;
    }

    let lines = fallback_lines(
        &request.paragraph,
        request.candidate_boundaries,
        request.available_widths_px,
    );
    HorizontalShapingLineOutcome {
        trace: line_trace(
            HorizontalShapingLineDisposition::RolledBack,
            Some(HorizontalShapingActivationReason::LineDecisionNotConverged),
            request.paragraph.fallback_owner,
            MAX_HORIZONTAL_SHAPING_LINE_RETRIES,
            budget.count,
            &lines,
        ),
        attempts: budget.traces,
        paragraph_measurement: Some(paragraph),
        lines,
    }
}
