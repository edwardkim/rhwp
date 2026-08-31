//! Page-local publication contract for horizontal shaping decisions.
//!
//! Q2-D0 established bounded ownership. D4 consumes certified decisions for stored rows, while
//! Q2-D5-N1 may atomically reserve the same bounded source budget before a qualified no-LineSeg
//! layout publishes geometry.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::shaping::{ShapingAttemptTrace, TerminalShapingDisposition};
use super::shaping_context::{
    HorizontalShapingMeasurement, HorizontalShapingReplaySourceCertificate,
};

/// A single page cannot retain more shaping decisions than it can reasonably emit as text runs.
pub(crate) const MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS: usize = 4_096;
pub(crate) const MAX_HORIZONTAL_SHAPING_PREPARED_SOURCES_PER_PAGE: usize = 256;
pub(crate) const MAX_HORIZONTAL_SHAPING_FONT_BYTES_PER_PAGE: usize = 64 * 1024 * 1024;

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
        replay_source_certificate: Option<Arc<HorizontalShapingReplaySourceCertificate>>,
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
            payload: HorizontalShapingRunPayload::Applied {
                trace,
                measurement,
                replay_source_certificate: None,
            },
        }
    }

    pub(crate) fn applied_with_replay_source_certificate(
        range: HorizontalShapingRunRange,
        trace: ShapingAttemptTrace,
        measurement: Arc<HorizontalShapingMeasurement>,
        replay_source_certificate: Arc<HorizontalShapingReplaySourceCertificate>,
    ) -> Self {
        Self {
            registry_generation: measurement.registry_generation,
            range,
            payload: HorizontalShapingRunPayload::Applied {
                trace,
                measurement,
                replay_source_certificate: Some(replay_source_certificate),
            },
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

    pub(crate) fn replay_source_certificate(
        &self,
    ) -> Option<&Arc<HorizontalShapingReplaySourceCertificate>> {
        match &self.payload {
            HorizontalShapingRunPayload::Applied {
                replay_source_certificate,
                ..
            } => replay_source_certificate.as_ref(),
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
    ReplaySourceCertificateMismatch,
    StaleRegistryGeneration,
    DuplicateNode,
    EntryLimitExceeded,
    ResourceLimitExceeded,
}

/// Bounded `NodeId -> Arc<decision>` storage. `NodeId` is a `u32` alias in render_tree.
#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingPageSidecars {
    registry_generation: Option<u64>,
    entries: HashMap<u32, Arc<HorizontalShapingRunDecision>>,
    reserved_source_identities: HashSet<(u64, String, usize, u32)>,
    reserved_source_bytes: usize,
}

impl Default for HorizontalShapingPageSidecars {
    fn default() -> Self {
        Self {
            registry_generation: None,
            entries: HashMap::new(),
            reserved_source_identities: HashSet::new(),
            reserved_source_bytes: 0,
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
            HorizontalShapingRunPayload::Applied {
                trace,
                measurement,
                replay_source_certificate,
            } => {
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
                if replay_source_certificate
                    .as_ref()
                    .is_some_and(|certificate| {
                        certificate.registry_generation() != measurement.registry_generation
                            || certificate.source_handle() != &measurement.source_handle
                            || certificate.source_bytes().len()
                                != measurement.source_handle.font_bytes
                            || certificate.units_per_em() != measurement.units_per_em
                    })
                {
                    return Err(
                        HorizontalShapingSidecarRejectReason::ReplaySourceCertificateMismatch,
                    );
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

    /// N1 attaches a certified decision only after the page can also reserve
    /// the same unique-source budget that paint lowering enforces. All checks
    /// precede both mutations; a failed attach cannot leave a partial resource
    /// reservation or sidecar behind.
    pub(crate) fn attach_no_lineseg_atomic(
        &mut self,
        node_id: u32,
        expected_range: HorizontalShapingRunRange,
        decision: Arc<HorizontalShapingRunDecision>,
    ) -> Result<(), HorizontalShapingSidecarRejectReason> {
        let certificate = decision
            .replay_source_certificate()
            .ok_or(HorizontalShapingSidecarRejectReason::ReplaySourceCertificateMismatch)?;
        let source = certificate.source_handle();
        if certificate.source_bytes().len() > crate::paint::MAX_PORTABLE_FONT_BLOB_BYTES {
            return Err(HorizontalShapingSidecarRejectReason::ResourceLimitExceeded);
        }
        let identity = (
            certificate.registry_generation(),
            source.font_source_sha256.clone(),
            source.font_bytes,
            source.face_index,
        );
        let is_new_source = !self.reserved_source_identities.contains(&identity);
        let next_source_count = self.reserved_source_identities.len() + usize::from(is_new_source);
        let next_source_bytes = if is_new_source {
            self.reserved_source_bytes
                .checked_add(certificate.source_bytes().len())
                .ok_or(HorizontalShapingSidecarRejectReason::ResourceLimitExceeded)?
        } else {
            self.reserved_source_bytes
        };
        if next_source_count > MAX_HORIZONTAL_SHAPING_PREPARED_SOURCES_PER_PAGE
            || next_source_bytes > MAX_HORIZONTAL_SHAPING_FONT_BYTES_PER_PAGE
        {
            return Err(HorizontalShapingSidecarRejectReason::ResourceLimitExceeded);
        }

        self.attach(node_id, expected_range, decision)?;
        if is_new_source {
            self.reserved_source_identities.insert(identity);
            self.reserved_source_bytes = next_source_bytes;
        }
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

    pub(crate) fn reserved_source_count(&self) -> usize {
        self.reserved_source_identities.len()
    }

    pub(crate) fn reserved_source_bytes(&self) -> usize {
        self.reserved_source_bytes
    }
}
