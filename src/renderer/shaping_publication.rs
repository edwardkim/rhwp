//! Page-local publication contract for horizontal shaping decisions.
//!
//! Q2-D0 only establishes bounded ownership. No layout or paint consumer reads this table yet.

use std::collections::HashMap;
use std::sync::Arc;

use super::shaping::{ShapingAttemptTrace, TerminalShapingDisposition};
use super::shaping_context::HorizontalShapingMeasurement;

/// A single page cannot retain more shaping decisions than it can reasonably emit as text runs.
pub(crate) const MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS: usize = 4_096;

/// One emitted run expressed in all three text coordinate systems used by the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HorizontalShapingRunRange {
    pub scalar_start: usize,
    pub scalar_end: usize,
    pub utf8_start: usize,
    pub utf8_end: usize,
    pub utf16_start: usize,
    pub utf16_end: usize,
}

impl HorizontalShapingRunRange {
    fn is_well_formed(self) -> bool {
        self.scalar_start < self.scalar_end
            && self.utf8_start < self.utf8_end
            && self.utf16_start < self.utf16_end
    }

    fn scalar_len(self) -> usize {
        self.scalar_end.saturating_sub(self.scalar_start)
    }
}

/// Applied decisions retain the one owned measurement; rejected decisions retain trace only.
#[derive(Debug, Clone)]
pub(crate) enum HorizontalShapingRunPayload {
    Applied {
        trace: ShapingAttemptTrace,
        measurement: Arc<HorizontalShapingMeasurement>,
    },
    Rejected {
        trace: ShapingAttemptTrace,
    },
}

/// The terminal decision attached to one emitted render-tree node.
#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingRunDecision {
    registry_generation: u64,
    range: HorizontalShapingRunRange,
    payload: HorizontalShapingRunPayload,
}

impl HorizontalShapingRunDecision {
    pub(crate) fn applied(
        range: HorizontalShapingRunRange,
        trace: ShapingAttemptTrace,
        measurement: Arc<HorizontalShapingMeasurement>,
    ) -> Self {
        Self {
            registry_generation: measurement.registry_generation,
            range,
            payload: HorizontalShapingRunPayload::Applied { trace, measurement },
        }
    }

    pub(crate) fn rejected(
        registry_generation: u64,
        range: HorizontalShapingRunRange,
        trace: ShapingAttemptTrace,
    ) -> Self {
        Self {
            registry_generation,
            range,
            payload: HorizontalShapingRunPayload::Rejected { trace },
        }
    }

    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub(crate) fn range(&self) -> HorizontalShapingRunRange {
        self.range
    }

    pub(crate) fn trace(&self) -> &ShapingAttemptTrace {
        match &self.payload {
            HorizontalShapingRunPayload::Applied { trace, .. }
            | HorizontalShapingRunPayload::Rejected { trace } => trace,
        }
    }

    pub(crate) fn measurement(&self) -> Option<&Arc<HorizontalShapingMeasurement>> {
        match &self.payload {
            HorizontalShapingRunPayload::Applied { measurement, .. } => Some(measurement),
            HorizontalShapingRunPayload::Rejected { .. } => None,
        }
    }
}

/// Typed fail-closed reasons for building the page-local ownership table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HorizontalShapingSidecarRejectReason {
    MalformedRange,
    RangeMismatch,
    DispositionMismatch,
    AttemptIdentityMismatch,
    MeasurementRangeMismatch,
    StaleRegistryGeneration,
    DuplicateNode,
    EntryLimitExceeded,
}

/// Bounded `NodeId -> Arc<decision>` storage. `NodeId` is a `u32` alias in render_tree.
#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingPageSidecars {
    registry_generation: Option<u64>,
    entries: HashMap<u32, Arc<HorizontalShapingRunDecision>>,
}

impl Default for HorizontalShapingPageSidecars {
    fn default() -> Self {
        Self {
            registry_generation: None,
            entries: HashMap::new(),
        }
    }
}

impl HorizontalShapingPageSidecars {
    pub(crate) fn attach(
        &mut self,
        node_id: u32,
        expected_range: HorizontalShapingRunRange,
        decision: Arc<HorizontalShapingRunDecision>,
    ) -> Result<(), HorizontalShapingSidecarRejectReason> {
        if !expected_range.is_well_formed() || !decision.range().is_well_formed() {
            return Err(HorizontalShapingSidecarRejectReason::MalformedRange);
        }
        if expected_range != decision.range() {
            return Err(HorizontalShapingSidecarRejectReason::RangeMismatch);
        }
        match &decision.payload {
            HorizontalShapingRunPayload::Applied { trace, measurement } => {
                if trace.disposition != TerminalShapingDisposition::Applied {
                    return Err(HorizontalShapingSidecarRejectReason::DispositionMismatch);
                }
                let identity = &measurement.applied.identity;
                if trace.reason.is_some()
                    || trace.glyph_count != measurement.applied.glyphs.len()
                    || trace.settings_sha256.as_deref() != Some(identity.settings_sha256.as_str())
                    || trace.font_source_sha256.as_deref()
                        != Some(identity.font_source_sha256.as_str())
                    || measurement.source_handle.font_source_sha256 != identity.font_source_sha256
                    || measurement.source_handle.font_bytes != identity.font_bytes
                    || measurement.source_handle.face_index != identity.face_index
                {
                    return Err(HorizontalShapingSidecarRejectReason::AttemptIdentityMismatch);
                }
                if measurement.registry_generation != decision.registry_generation()
                    || measurement.code_point_count != decision.range().scalar_len()
                {
                    return Err(HorizontalShapingSidecarRejectReason::MeasurementRangeMismatch);
                }
            }
            HorizontalShapingRunPayload::Rejected { trace } => {
                if trace.disposition == TerminalShapingDisposition::Applied {
                    return Err(HorizontalShapingSidecarRejectReason::DispositionMismatch);
                }
            }
        }
        if self.entries.contains_key(&node_id) {
            return Err(HorizontalShapingSidecarRejectReason::DuplicateNode);
        }
        if self
            .registry_generation
            .is_some_and(|generation| generation != decision.registry_generation())
        {
            return Err(HorizontalShapingSidecarRejectReason::StaleRegistryGeneration);
        }
        if self.entries.len() >= MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS {
            return Err(HorizontalShapingSidecarRejectReason::EntryLimitExceeded);
        }

        self.registry_generation = Some(decision.registry_generation());
        self.entries.insert(node_id, decision);
        Ok(())
    }

    pub(crate) fn get(&self, node_id: u32) -> Option<&Arc<HorizontalShapingRunDecision>> {
        self.entries.get(&node_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn registry_generation(&self) -> Option<u64> {
        self.registry_generation
    }
}
