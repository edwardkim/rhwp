//! W10-Q2 horizontal shaping의 transaction-local face cache와 bounded shadow measurement.
//!
//! W9 exact-source registry를 그대로 빌리고, cluster-aware px 결과를 현행 layout 옆에서만
//! 계산한다. composer, render tree, paint는 이 모듈을 아직 소비하지 않는다.

use super::kerning::{
    resolve_exact_font_source, ExactFontSlot, ExactFontSource as RegistryExactFontSource,
    ExactFontSourceHandle, ExactFontSourceRegistry, ExactFontSourceResolutionReason,
};
use super::shaping::{
    canonicalize_verified_shaping_request, shape_canonical_request_with_face,
    terminal_shaping_attempt_from_output, AppliedShapingRun, ShapingAttemptTrace, ShapingDirection,
    ShapingExactSource, ShapingFeature, ShapingOutputDecision, ShapingRejectReason, ShapingRequest,
    ShapingVariation, ShapingWritingMode, TerminalShapingAttempt, TerminalShapingDisposition,
    MAX_SHAPING_GLYPHS, MAX_SHAPING_TEXT_CODE_POINTS, MAX_SHAPING_VARIATION_AXES,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) const MAX_HORIZONTAL_SHAPING_CACHE_ENTRIES: usize = 4_096;
pub(crate) const MAX_HORIZONTAL_SHAPING_CLUSTERS: usize = 4_096;
pub(crate) const MAX_HORIZONTAL_SHAPING_CACHE_TEXT_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_HORIZONTAL_SHAPING_CACHE_GLYPHS: usize = 262_144;
pub(crate) const MAX_HORIZONTAL_SHAPING_CACHE_CLUSTERS: usize = 262_144;
pub(crate) const MAX_HORIZONTAL_SHAPING_INSTANCE_REQUESTS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HorizontalShapingInstanceRequestRegistration {
    Registered,
    Updated,
    AlreadyRegistered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HorizontalShapingInstanceRequestError {
    InvalidLanguageIndex,
    SourceUnavailable,
    SourceIdentityMismatch,
    MalformedSfnt,
    RequestLimitExceeded,
    RequestUnavailable,
    ShapingRejected(ShapingRejectReason),
}

impl HorizontalShapingInstanceRequestError {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::InvalidLanguageIndex => "invalidLanguageIndex",
            Self::SourceUnavailable => "sourceUnavailable",
            Self::SourceIdentityMismatch => "sourceIdentityMismatch",
            Self::MalformedSfnt => "malformedSfnt",
            Self::RequestLimitExceeded => "requestLimitExceeded",
            Self::RequestUnavailable => "requestUnavailable",
            Self::ShapingRejected(reason) => match reason {
                ShapingRejectReason::VariationAxisLimitExceeded => "variationAxisLimitExceeded",
                ShapingRejectReason::MalformedVariationTag => "malformedVariationTag",
                ShapingRejectReason::DuplicateVariationAxis => "duplicateVariationAxis",
                ShapingRejectReason::VariationValueNonFinite => "variationValueNonFinite",
                ShapingRejectReason::VariationAxisUnsupported => "variationAxisUnsupported",
                ShapingRejectReason::VariationValueOutOfRange => "variationValueOutOfRange",
                _ => "shapingRejected",
            },
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct HorizontalShapingInstanceRequestRegistry {
    entries: HashMap<ExactFontSlot, Arc<[ShapingVariation]>>,
    generation: u64,
}

impl std::fmt::Debug for HorizontalShapingInstanceRequestRegistry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HorizontalShapingInstanceRequestRegistry")
            .field("request_count", &self.entries.len())
            .field("generation", &self.generation)
            .finish()
    }
}

impl HorizontalShapingInstanceRequestRegistry {
    /// Validate and canonicalize one explicit slot request against the exact
    /// source already selected by the layout owner. No request state changes
    /// until source identity, SFNT, axis, range, and resource checks all pass.
    pub(crate) fn set_verified(
        &mut self,
        sources: &ExactFontSourceRegistry,
        slot: ExactFontSlot,
        variations: &[ShapingVariation],
    ) -> Result<HorizontalShapingInstanceRequestRegistration, HorizontalShapingInstanceRequestError>
    {
        use HorizontalShapingInstanceRequestError as Error;
        use HorizontalShapingInstanceRequestRegistration as Registration;

        if slot.language_index >= 7 {
            return Err(Error::InvalidLanguageIndex);
        }
        if variations.len() > MAX_SHAPING_VARIATION_AXES {
            return Err(Error::ShapingRejected(
                ShapingRejectReason::VariationAxisLimitExceeded,
            ));
        }
        let handle = sources
            .handle_for_slot(slot)
            .cloned()
            .ok_or(Error::SourceUnavailable)?;
        let source =
            resolve_exact_font_source(sources, &handle).map_err(|reason| match reason {
                ExactFontSourceResolutionReason::SourceUnavailable => Error::SourceUnavailable,
                ExactFontSourceResolutionReason::FontByteLimitExceeded
                | ExactFontSourceResolutionReason::FaceIndexMismatch
                | ExactFontSourceResolutionReason::ByteLengthMismatch
                | ExactFontSourceResolutionReason::Sha256Mismatch => Error::SourceIdentityMismatch,
            })?;
        let face = ttf_parser::Face::parse(source.bytes, source.face_index)
            .map_err(|_| Error::MalformedSfnt)?;
        let request = ShapingRequest {
            source: Some(ShapingExactSource {
                bytes: source.bytes,
                face_index: source.face_index,
                portable: true,
            }),
            text: "",
            direction: ShapingDirection::LeftToRight,
            writing_mode: ShapingWritingMode::HorizontalTb,
            script: None,
            language: None,
            features: &[],
            variations,
        };
        let identity =
            canonicalize_verified_shaping_request(&request, &face, &handle.font_source_sha256)
                .map_err(|decision| {
                    Error::ShapingRejected(
                        decision
                            .reason
                            .unwrap_or(ShapingRejectReason::ShapingUnavailable),
                    )
                })?;
        let canonical = identity
            .variations
            .iter()
            .map(|variation| ShapingVariation {
                tag: variation.tag.clone(),
                value: f32::from_bits(variation.value_bits),
            })
            .collect::<Vec<_>>();
        if self.entries.get(&slot).is_some_and(|existing| {
            existing.len() == canonical.len()
                && existing.iter().zip(&canonical).all(|(left, right)| {
                    left.tag == right.tag && left.value.to_bits() == right.value.to_bits()
                })
        }) {
            return Ok(Registration::AlreadyRegistered);
        }
        if !self.entries.contains_key(&slot)
            && self.entries.len() >= MAX_HORIZONTAL_SHAPING_INSTANCE_REQUESTS
        {
            return Err(Error::RequestLimitExceeded);
        }
        let registration = if self.entries.contains_key(&slot) {
            Registration::Updated
        } else {
            Registration::Registered
        };
        self.entries.insert(slot, Arc::from(canonical));
        self.generation = self.generation.wrapping_add(1);
        Ok(registration)
    }

    pub(crate) fn clear(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        self.entries.clear();
        self.generation = self.generation.wrapping_add(1);
        true
    }

    pub(crate) fn request_count(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    fn request_for_slot(&self, slot: ExactFontSlot) -> Option<Arc<[ShapingVariation]>> {
        self.entries.get(&slot).cloned()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingRequest<'a> {
    pub attempt_id: u32,
    pub slot: ExactFontSlot,
    pub text: &'a str,
    pub effective_font_size_px: f64,
    pub width_ratio: f64,
    pub script: Option<&'a str>,
    pub language: Option<&'a str>,
    pub features: &'a [ShapingFeature],
    pub variations: &'a [ShapingVariation],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HorizontalShapingCacheKey {
    registry_generation: u64,
    instance_request: Option<HorizontalShapingInstanceRequestProvenance>,
    source_handle: ExactFontSourceHandle,
    text: String,
    effective_font_size_bits: u64,
    width_ratio_bits: u64,
    script: Option<String>,
    language: Option<String>,
    features: Vec<(String, u32)>,
    variations: Vec<(String, u32)>,
}

impl HorizontalShapingCacheKey {
    fn new(
        registry_generation: u64,
        instance_request: Option<HorizontalShapingInstanceRequestProvenance>,
        source_handle: ExactFontSourceHandle,
        request: &HorizontalShapingRequest<'_>,
        identity: &super::shaping::CanonicalShapingIdentity,
    ) -> Self {
        Self {
            registry_generation,
            instance_request,
            source_handle,
            text: request.text.to_owned(),
            effective_font_size_bits: request.effective_font_size_px.to_bits(),
            width_ratio_bits: request.width_ratio.to_bits(),
            script: request.script.map(str::to_owned),
            language: request.language.map(str::to_owned),
            features: request
                .features
                .iter()
                .map(|feature| (feature.tag.clone(), feature.value))
                .collect(),
            variations: identity
                .variations
                .iter()
                .map(|variation| (variation.tag.clone(), variation.value_bits))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HorizontalShapingFaceKey {
    source_handle: ExactFontSourceHandle,
    variations: Vec<(String, u32)>,
}

impl HorizontalShapingFaceKey {
    fn new(
        source_handle: ExactFontSourceHandle,
        identity: &super::shaping::CanonicalShapingIdentity,
    ) -> Self {
        Self {
            source_handle,
            variations: identity
                .variations
                .iter()
                .map(|variation| (variation.tag.clone(), variation.value_bits))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HorizontalShapingGlyphPx {
    pub glyph_id: u32,
    pub cluster_utf8: u32,
    pub x: f64,
    pub y: f64,
    pub advance_x: f64,
    pub advance_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HorizontalShapingCluster {
    pub utf8_start: usize,
    pub utf8_end: usize,
    pub scalar_start: usize,
    pub scalar_end: usize,
    pub glyph_start: usize,
    pub glyph_end: usize,
    pub advance_px: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HorizontalShapingMeasurement {
    pub registry_generation: u64,
    pub instance_request: Option<HorizontalShapingInstanceRequestProvenance>,
    pub source_handle: ExactFontSourceHandle,
    pub code_point_count: usize,
    pub units_per_em: u16,
    pub total_advance_px: f64,
    pub glyphs_px: Vec<HorizontalShapingGlyphPx>,
    pub clusters: Vec<HorizontalShapingCluster>,
    pub applied: Arc<AppliedShapingRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct HorizontalShapingInstanceRequestProvenance {
    pub slot: ExactFontSlot,
    pub request_generation: u64,
}

/// Page-local proof that replay uses the exact immutable source selected for
/// shaping. Its Debug form is deliberately redacted so font bytes cannot enter
/// traces or diagnostics accidentally.
#[derive(Clone)]
pub(crate) struct HorizontalShapingReplaySourceCertificate {
    registry_generation: u64,
    source_handle: ExactFontSourceHandle,
    source_bytes: Arc<[u8]>,
    units_per_em: u16,
}

impl std::fmt::Debug for HorizontalShapingReplaySourceCertificate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HorizontalShapingReplaySourceCertificate")
            .field("registry_generation", &self.registry_generation)
            .field("source_handle", &self.source_handle)
            .field("source_bytes_len", &self.source_bytes.len())
            .field("units_per_em", &self.units_per_em)
            .finish()
    }
}

impl HorizontalShapingReplaySourceCertificate {
    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub(crate) fn source_handle(&self) -> &ExactFontSourceHandle {
        &self.source_handle
    }

    pub(crate) fn source_bytes(&self) -> &[u8] {
        &self.source_bytes
    }

    pub(crate) fn source_bytes_arc(&self) -> &Arc<[u8]> {
        &self.source_bytes
    }

    pub(crate) fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HorizontalShapingReplaySourceCertificateRejectReason {
    StaleRegistryGeneration,
    SourceUnavailable,
    SourceIdentityMismatch,
    FaceInvalid,
    UnitsPerEmMismatch,
}

impl HorizontalShapingReplaySourceCertificateRejectReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::StaleRegistryGeneration => "staleRegistryGeneration",
            Self::SourceUnavailable => "sourceUnavailable",
            Self::SourceIdentityMismatch => "sourceIdentityMismatch",
            Self::FaceInvalid => "faceInvalid",
            Self::UnitsPerEmMismatch => "unitsPerEmMismatch",
        }
    }
}

impl HorizontalShapingMeasurement {
    /// Unicode scalar 범위가 shaping cluster 경계에 정확히 맞을 때만 폭을 반환한다.
    pub(crate) fn range_width(&self, scalar_start: usize, scalar_end: usize) -> Option<f64> {
        if scalar_start > scalar_end || scalar_end > self.code_point_count {
            return None;
        }
        let is_boundary = |index: usize| {
            index == 0
                || index == self.code_point_count
                || self
                    .clusters
                    .iter()
                    .any(|cluster| cluster.scalar_start == index || cluster.scalar_end == index)
        };
        if !is_boundary(scalar_start) || !is_boundary(scalar_end) {
            return None;
        }
        if scalar_start == scalar_end {
            return Some(0.0);
        }

        let mut cursor = scalar_start;
        let mut width = 0.0;
        for cluster in self.clusters.iter().filter(|cluster| {
            cluster.scalar_end > scalar_start && cluster.scalar_start < scalar_end
        }) {
            if cluster.scalar_start != cursor || cluster.scalar_end > scalar_end {
                return None;
            }
            width += cluster.advance_px;
            cursor = cluster.scalar_end;
        }
        (cursor == scalar_end && width.is_finite()).then_some(width)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingShadowOutcome {
    pub trace: ShapingAttemptTrace,
    pub cache_hit: bool,
    pub measurement: Option<Arc<HorizontalShapingMeasurement>>,
}

impl HorizontalShapingShadowOutcome {
    pub(crate) fn is_applied(&self) -> bool {
        self.trace.disposition == TerminalShapingDisposition::Applied && self.measurement.is_some()
    }
}

#[derive(Default)]
struct HorizontalShapingResultCache {
    entries: HashMap<HorizontalShapingCacheKey, Arc<HorizontalShapingMeasurement>>,
    text_bytes: usize,
    glyphs: usize,
    clusters: usize,
}

pub(crate) struct HorizontalShapingContext {
    registry: ExactFontSourceRegistry,
    instance_requests: HorizontalShapingInstanceRequestRegistry,
    cache_limit: usize,
    result_cache: Mutex<HorizontalShapingResultCache>,
}

impl std::fmt::Debug for HorizontalShapingContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HorizontalShapingContext")
            .field("registry_generation", &self.registry.generation())
            .field(
                "instance_request_generation",
                &self.instance_requests.generation(),
            )
            .field(
                "instance_request_count",
                &self.instance_requests.request_count(),
            )
            .field("cache_limit", &self.cache_limit)
            .field("cached_result_count", &self.cached_result_count())
            .finish()
    }
}

impl HorizontalShapingContext {
    pub(crate) fn new(registry: ExactFontSourceRegistry) -> Self {
        Self::with_instance_requests_and_cache_limit(
            registry,
            HorizontalShapingInstanceRequestRegistry::default(),
            MAX_HORIZONTAL_SHAPING_CACHE_ENTRIES,
        )
    }

    pub(crate) fn with_cache_limit(registry: ExactFontSourceRegistry, cache_limit: usize) -> Self {
        Self::with_instance_requests_and_cache_limit(
            registry,
            HorizontalShapingInstanceRequestRegistry::default(),
            cache_limit,
        )
    }

    pub(crate) fn with_instance_requests(
        registry: ExactFontSourceRegistry,
        instance_requests: HorizontalShapingInstanceRequestRegistry,
    ) -> Self {
        Self::with_instance_requests_and_cache_limit(
            registry,
            instance_requests,
            MAX_HORIZONTAL_SHAPING_CACHE_ENTRIES,
        )
    }

    fn with_instance_requests_and_cache_limit(
        registry: ExactFontSourceRegistry,
        instance_requests: HorizontalShapingInstanceRequestRegistry,
        cache_limit: usize,
    ) -> Self {
        Self {
            registry,
            instance_requests,
            cache_limit: cache_limit.min(MAX_HORIZONTAL_SHAPING_CACHE_ENTRIES),
            result_cache: Mutex::new(HorizontalShapingResultCache::default()),
        }
    }

    pub(crate) fn transaction(&self) -> HorizontalShapingTransaction<'_> {
        HorizontalShapingTransaction::new(self)
    }

    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry.generation()
    }

    pub(crate) fn instance_request_generation(&self) -> u64 {
        self.instance_requests.generation()
    }

    pub(crate) fn instance_request_count(&self) -> usize {
        self.instance_requests.request_count()
    }

    pub(crate) fn explicit_instance_transaction(
        &self,
        slot: ExactFontSlot,
    ) -> Result<
        HorizontalShapingExplicitInstanceTransaction<'_>,
        HorizontalShapingInstanceRequestError,
    > {
        let variations = self
            .instance_requests
            .request_for_slot(slot)
            .ok_or(HorizontalShapingInstanceRequestError::RequestUnavailable)?;
        Ok(HorizontalShapingExplicitInstanceTransaction {
            transaction: self.transaction(),
            slot,
            variations,
            provenance: HorizontalShapingInstanceRequestProvenance {
                slot,
                request_generation: self.instance_requests.generation(),
            },
        })
    }

    pub(crate) fn cached_result_count(&self) -> usize {
        self.result_cache
            .lock()
            .map(|cache| cache.entries.len())
            .unwrap_or_default()
    }

    /// Certify the exact registry-owned bytes for one completed measurement.
    /// Validation is repeated at this ownership boundary, then only the Arc is
    /// cloned so the page sidecar does not duplicate the font payload.
    pub(crate) fn certify_replay_source(
        &self,
        measurement: &HorizontalShapingMeasurement,
    ) -> Result<
        Arc<HorizontalShapingReplaySourceCertificate>,
        HorizontalShapingReplaySourceCertificateRejectReason,
    > {
        use HorizontalShapingReplaySourceCertificateRejectReason as Reject;

        if measurement.registry_generation != self.registry.generation() {
            return Err(Reject::StaleRegistryGeneration);
        }
        resolve_exact_font_source(&self.registry, &measurement.source_handle).map_err(
            |reason| match reason {
                ExactFontSourceResolutionReason::SourceUnavailable => Reject::SourceUnavailable,
                ExactFontSourceResolutionReason::FontByteLimitExceeded
                | ExactFontSourceResolutionReason::FaceIndexMismatch
                | ExactFontSourceResolutionReason::ByteLengthMismatch
                | ExactFontSourceResolutionReason::Sha256Mismatch => Reject::SourceIdentityMismatch,
            },
        )?;
        let source_bytes = self
            .registry
            .source_arc_for_handle(&measurement.source_handle)
            .ok_or(Reject::SourceUnavailable)?;
        let face = ttf_parser::Face::parse(&source_bytes, measurement.source_handle.face_index)
            .map_err(|_| Reject::FaceInvalid)?;
        let units_per_em = face.units_per_em();
        if units_per_em != measurement.units_per_em {
            return Err(Reject::UnitsPerEmMismatch);
        }
        Ok(Arc::new(HorizontalShapingReplaySourceCertificate {
            registry_generation: measurement.registry_generation,
            source_handle: measurement.source_handle.clone(),
            source_bytes,
            units_per_em,
        }))
    }
}

struct HorizontalShapingPreparedSource<'a> {
    source: RegistryExactFontSource<'a>,
    face: ttf_parser::Face<'a>,
}

pub(crate) struct HorizontalShapingTransaction<'a> {
    context: &'a HorizontalShapingContext,
    registry_generation: u64,
    prepared_sources: HashMap<ExactFontSourceHandle, HorizontalShapingPreparedSource<'a>>,
    faces: HashMap<HorizontalShapingFaceKey, rustybuzz::Face<'a>>,
    prepared_source_count: usize,
    parsed_face_count: usize,
    result_cache_hit_count: usize,
    result_cache_miss_count: usize,
}

pub(crate) struct HorizontalShapingExplicitInstanceTransaction<'a> {
    transaction: HorizontalShapingTransaction<'a>,
    slot: ExactFontSlot,
    variations: Arc<[ShapingVariation]>,
    provenance: HorizontalShapingInstanceRequestProvenance,
}

impl HorizontalShapingExplicitInstanceTransaction<'_> {
    pub(crate) fn registry_generation(&self) -> u64 {
        self.transaction.registry_generation()
    }

    pub(crate) fn request_generation(&self) -> u64 {
        self.provenance.request_generation
    }

    pub(crate) fn shadow_measure(
        &mut self,
        request: &HorizontalShapingRequest<'_>,
    ) -> HorizontalShapingShadowOutcome {
        if request.slot != self.slot {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::Malformed,
                ShapingRejectReason::ExplicitInstanceSlotMismatch,
            );
        }
        if !request.variations.is_empty() {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::Malformed,
                ShapingRejectReason::ExplicitInstanceOverrideNotAllowed,
            );
        }
        let owned_request = HorizontalShapingRequest {
            attempt_id: request.attempt_id,
            slot: request.slot,
            text: request.text,
            effective_font_size_px: request.effective_font_size_px,
            width_ratio: request.width_ratio,
            script: request.script,
            language: request.language,
            features: request.features,
            variations: &self.variations,
        };
        self.transaction
            .shadow_measure_with_instance_request(&owned_request, Some(self.provenance))
    }
}

impl<'a> HorizontalShapingTransaction<'a> {
    fn new(context: &'a HorizontalShapingContext) -> Self {
        Self {
            context,
            registry_generation: context.registry.generation(),
            prepared_sources: HashMap::new(),
            faces: HashMap::new(),
            prepared_source_count: 0,
            parsed_face_count: 0,
            result_cache_hit_count: 0,
            result_cache_miss_count: 0,
        }
    }

    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub(crate) fn parsed_face_count(&self) -> usize {
        self.parsed_face_count
    }

    pub(crate) fn prepared_source_count(&self) -> usize {
        self.prepared_source_count
    }

    pub(crate) fn result_cache_hit_count(&self) -> usize {
        self.result_cache_hit_count
    }

    pub(crate) fn result_cache_miss_count(&self) -> usize {
        self.result_cache_miss_count
    }

    pub(crate) fn shadow_measure(
        &mut self,
        request: &HorizontalShapingRequest<'_>,
    ) -> HorizontalShapingShadowOutcome {
        self.shadow_measure_with_instance_request(request, None)
    }

    fn shadow_measure_with_instance_request(
        &mut self,
        request: &HorizontalShapingRequest<'_>,
        instance_request: Option<HorizontalShapingInstanceRequestProvenance>,
    ) -> HorizontalShapingShadowOutcome {
        if !valid_scale(request.effective_font_size_px, request.width_ratio) {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::Malformed,
                ShapingRejectReason::InvalidHorizontalScale,
            );
        }
        let observed_code_points = request
            .text
            .chars()
            .take(MAX_SHAPING_TEXT_CODE_POINTS + 1)
            .count();
        if observed_code_points > MAX_SHAPING_TEXT_CODE_POINTS {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::BoundedLimit,
                ShapingRejectReason::TextCodePointLimitExceeded,
            );
        }

        let Some(handle) = self.context.registry.handle_for_slot(request.slot).cloned() else {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::Unsupported,
                ShapingRejectReason::SourceUnavailable,
            );
        };
        if !self.prepared_sources.contains_key(&handle) {
            let registry: &'a ExactFontSourceRegistry = &self.context.registry;
            let source = match resolve_exact_font_source(registry, &handle) {
                Ok(source) => source,
                Err(reason) => return resolution_rejected_outcome(request.attempt_id, reason),
            };
            let Ok(face) = ttf_parser::Face::parse(source.bytes, source.face_index) else {
                return rejected_outcome(
                    request.attempt_id,
                    TerminalShapingDisposition::Malformed,
                    ShapingRejectReason::MalformedSfnt,
                );
            };
            self.prepared_sources.insert(
                handle.clone(),
                HorizontalShapingPreparedSource { source, face },
            );
            self.prepared_source_count = self.prepared_source_count.saturating_add(1);
        }
        let prepared = self
            .prepared_sources
            .get(&handle)
            .expect("prepared exact shaping source");
        let source = prepared.source;
        let shaping_request = ShapingRequest {
            source: Some(ShapingExactSource {
                bytes: source.bytes,
                face_index: source.face_index,
                portable: true,
            }),
            text: request.text,
            direction: ShapingDirection::LeftToRight,
            writing_mode: ShapingWritingMode::HorizontalTb,
            script: request.script,
            language: request.language,
            features: request.features,
            variations: request.variations,
        };
        let identity = match canonicalize_verified_shaping_request(
            &shaping_request,
            &prepared.face,
            &handle.font_source_sha256,
        ) {
            Ok(identity) => identity,
            Err(decision) => {
                return terminal_outcome(terminal_shaping_attempt_from_output(
                    request.attempt_id,
                    ShapingOutputDecision {
                        disposition: decision.disposition,
                        reason: decision.reason,
                        identity: None,
                        glyph_count: 0,
                        glyphs: Vec::new(),
                    },
                ));
            }
        };
        let key = HorizontalShapingCacheKey::new(
            self.registry_generation,
            instance_request,
            handle.clone(),
            request,
            &identity,
        );
        let cache = match self.context.result_cache.lock() {
            Ok(cache) => cache,
            Err(_) => {
                return rejected_outcome(
                    request.attempt_id,
                    TerminalShapingDisposition::Unsupported,
                    ShapingRejectReason::ShapingUnavailable,
                );
            }
        };
        if let Some(measurement) = cache.entries.get(&key) {
            self.result_cache_hit_count = self.result_cache_hit_count.saturating_add(1);
            return cached_outcome(request.attempt_id, Arc::clone(measurement));
        }
        if cache.entries.len() >= self.context.cache_limit {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::BoundedLimit,
                ShapingRejectReason::CacheEntryLimitExceeded,
            );
        }
        drop(cache);
        self.result_cache_miss_count = self.result_cache_miss_count.saturating_add(1);

        let face_key = HorizontalShapingFaceKey::new(handle.clone(), &identity);
        if !self.faces.contains_key(&face_key) {
            let Some(face) = rustybuzz::Face::from_slice(source.bytes, source.face_index) else {
                return rejected_outcome(
                    request.attempt_id,
                    TerminalShapingDisposition::Unsupported,
                    ShapingRejectReason::ShapingUnavailable,
                );
            };
            self.faces.insert(face_key.clone(), face);
            self.parsed_face_count = self.parsed_face_count.saturating_add(1);
        }
        let face = self
            .faces
            .get_mut(&face_key)
            .expect("prepared exact instance face");
        let Ok(units_per_em) = u16::try_from(face.units_per_em()) else {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::Malformed,
                ShapingRejectReason::ShapingUnavailable,
            );
        };
        if units_per_em == 0 {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::Malformed,
                ShapingRejectReason::ShapingUnavailable,
            );
        }
        let attempt = terminal_shaping_attempt_from_output(
            request.attempt_id,
            shape_canonical_request_with_face(&shaping_request, identity, face),
        );
        let Some(applied) = attempt.applied.as_ref().map(Arc::clone) else {
            return terminal_outcome(attempt);
        };
        let measurement = match build_measurement(
            self.registry_generation,
            instance_request,
            handle,
            request,
            units_per_em,
            applied,
        ) {
            Ok(measurement) => Arc::new(measurement),
            Err(reason) => {
                return rejected_applied_outcome(request.attempt_id, &attempt, reason);
            }
        };

        let mut cache = match self.context.result_cache.lock() {
            Ok(cache) => cache,
            Err(_) => {
                return rejected_outcome(
                    request.attempt_id,
                    TerminalShapingDisposition::Unsupported,
                    ShapingRejectReason::ShapingUnavailable,
                );
            }
        };
        if let Some(existing) = cache.entries.get(&key) {
            self.result_cache_hit_count = self.result_cache_hit_count.saturating_add(1);
            return cached_outcome(request.attempt_id, Arc::clone(existing));
        }
        let next_text_bytes = cache.text_bytes.checked_add(key.text.len());
        let next_glyphs = cache.glyphs.checked_add(measurement.glyphs_px.len());
        let next_clusters = cache.clusters.checked_add(measurement.clusters.len());
        if cache.entries.len() >= self.context.cache_limit
            || next_text_bytes.is_none_or(|value| value > MAX_HORIZONTAL_SHAPING_CACHE_TEXT_BYTES)
            || next_glyphs.is_none_or(|value| value > MAX_HORIZONTAL_SHAPING_CACHE_GLYPHS)
            || next_clusters.is_none_or(|value| value > MAX_HORIZONTAL_SHAPING_CACHE_CLUSTERS)
        {
            return rejected_outcome(
                request.attempt_id,
                TerminalShapingDisposition::BoundedLimit,
                ShapingRejectReason::CacheEntryLimitExceeded,
            );
        }
        cache.text_bytes = next_text_bytes.expect("bounded text bytes");
        cache.glyphs = next_glyphs.expect("bounded glyph count");
        cache.clusters = next_clusters.expect("bounded cluster count");
        cache.entries.insert(key, Arc::clone(&measurement));
        drop(cache);
        HorizontalShapingShadowOutcome {
            trace: attempt.trace,
            cache_hit: false,
            measurement: Some(measurement),
        }
    }
}

fn valid_scale(font_size_px: f64, width_ratio: f64) -> bool {
    font_size_px.is_finite()
        && font_size_px > 0.0
        && font_size_px <= 4_096.0
        && width_ratio.is_finite()
        && width_ratio > 0.0
        && width_ratio <= 16.0
}

fn terminal_outcome(attempt: TerminalShapingAttempt) -> HorizontalShapingShadowOutcome {
    HorizontalShapingShadowOutcome {
        trace: attempt.trace,
        cache_hit: false,
        measurement: None,
    }
}

fn rejected_outcome(
    attempt_id: u32,
    disposition: TerminalShapingDisposition,
    reason: ShapingRejectReason,
) -> HorizontalShapingShadowOutcome {
    HorizontalShapingShadowOutcome {
        trace: ShapingAttemptTrace {
            attempt_id,
            disposition,
            reason: Some(reason),
            settings_sha256: None,
            font_source_sha256: None,
            glyph_count: 0,
        },
        cache_hit: false,
        measurement: None,
    }
}

fn resolution_rejected_outcome(
    attempt_id: u32,
    reason: ExactFontSourceResolutionReason,
) -> HorizontalShapingShadowOutcome {
    let shaping_reason = match reason {
        ExactFontSourceResolutionReason::SourceUnavailable => {
            ShapingRejectReason::SourceUnavailable
        }
        ExactFontSourceResolutionReason::FontByteLimitExceeded => {
            ShapingRejectReason::FontByteLimitExceeded
        }
        ExactFontSourceResolutionReason::FaceIndexMismatch
        | ExactFontSourceResolutionReason::ByteLengthMismatch
        | ExactFontSourceResolutionReason::Sha256Mismatch => {
            ShapingRejectReason::ExactSourceIdentityMismatch
        }
    };
    rejected_outcome(
        attempt_id,
        TerminalShapingDisposition::Unsupported,
        shaping_reason,
    )
}

fn cached_outcome(
    attempt_id: u32,
    measurement: Arc<HorizontalShapingMeasurement>,
) -> HorizontalShapingShadowOutcome {
    HorizontalShapingShadowOutcome {
        trace: ShapingAttemptTrace {
            attempt_id,
            disposition: TerminalShapingDisposition::Applied,
            reason: None,
            settings_sha256: Some(measurement.applied.identity.settings_sha256.clone()),
            font_source_sha256: Some(measurement.applied.identity.font_source_sha256.clone()),
            glyph_count: measurement.applied.glyphs.len(),
        },
        cache_hit: true,
        measurement: Some(measurement),
    }
}

fn rejected_applied_outcome(
    attempt_id: u32,
    attempt: &TerminalShapingAttempt,
    reason: ShapingRejectReason,
) -> HorizontalShapingShadowOutcome {
    HorizontalShapingShadowOutcome {
        trace: ShapingAttemptTrace {
            attempt_id,
            disposition: TerminalShapingDisposition::Malformed,
            reason: Some(reason),
            settings_sha256: attempt.trace.settings_sha256.clone(),
            font_source_sha256: attempt.trace.font_source_sha256.clone(),
            glyph_count: 0,
        },
        cache_hit: false,
        measurement: None,
    }
}

fn build_measurement(
    registry_generation: u64,
    instance_request: Option<HorizontalShapingInstanceRequestProvenance>,
    source_handle: ExactFontSourceHandle,
    request: &HorizontalShapingRequest<'_>,
    units_per_em: u16,
    applied: Arc<AppliedShapingRun>,
) -> Result<HorizontalShapingMeasurement, ShapingRejectReason> {
    let horizontal_scale =
        request.effective_font_size_px * request.width_ratio / f64::from(units_per_em);
    let vertical_scale = request.effective_font_size_px / f64::from(units_per_em);
    if !horizontal_scale.is_finite() || !vertical_scale.is_finite() {
        return Err(ShapingRejectReason::InvalidHorizontalScale);
    }

    let mut pen_x = 0.0;
    let mut glyphs_px = Vec::with_capacity(applied.glyphs.len());
    for glyph in &applied.glyphs {
        let x = pen_x + f64::from(glyph.x_offset) * horizontal_scale;
        let y = f64::from(glyph.y_offset) * vertical_scale;
        let advance_x = f64::from(glyph.x_advance) * horizontal_scale;
        let advance_y = f64::from(glyph.y_advance) * vertical_scale;
        if !x.is_finite() || !y.is_finite() || !advance_x.is_finite() || !advance_y.is_finite() {
            return Err(ShapingRejectReason::InvalidHorizontalScale);
        }
        glyphs_px.push(HorizontalShapingGlyphPx {
            glyph_id: glyph.glyph_id,
            cluster_utf8: glyph.cluster_utf8,
            x,
            y,
            advance_x,
            advance_y,
        });
        pen_x += advance_x;
    }
    if !pen_x.is_finite() || pen_x < 0.0 || glyphs_px.len() > MAX_SHAPING_GLYPHS {
        return Err(ShapingRejectReason::ClusterMappingInvalid);
    }
    let clusters = build_clusters(request.text, &glyphs_px)?;
    if clusters.len() > MAX_HORIZONTAL_SHAPING_CLUSTERS {
        return Err(ShapingRejectReason::ClusterMappingInvalid);
    }
    let cluster_advance = clusters
        .iter()
        .map(|cluster| cluster.advance_px)
        .sum::<f64>();
    if !cluster_advance.is_finite() || (cluster_advance - pen_x).abs() > 1.0e-9 {
        return Err(ShapingRejectReason::ClusterMappingInvalid);
    }

    Ok(HorizontalShapingMeasurement {
        registry_generation,
        instance_request,
        source_handle,
        code_point_count: request.text.chars().count(),
        units_per_em,
        total_advance_px: pen_x,
        glyphs_px,
        clusters,
        applied,
    })
}

fn build_clusters(
    text: &str,
    glyphs: &[HorizontalShapingGlyphPx],
) -> Result<Vec<HorizontalShapingCluster>, ShapingRejectReason> {
    if glyphs.is_empty() {
        return if text.is_empty() {
            Ok(Vec::new())
        } else {
            Err(ShapingRejectReason::ClusterMappingInvalid)
        };
    }
    let mut groups = Vec::<(usize, usize, usize)>::new();
    for (glyph_index, glyph) in glyphs.iter().enumerate() {
        let cluster_start = usize::try_from(glyph.cluster_utf8)
            .map_err(|_| ShapingRejectReason::ClusterMappingInvalid)?;
        if cluster_start > text.len() || !text.is_char_boundary(cluster_start) {
            return Err(ShapingRejectReason::ClusterMappingInvalid);
        }
        match groups.last_mut() {
            Some((last_start, _, glyph_end)) if *last_start == cluster_start => {
                *glyph_end = glyph_index + 1;
            }
            Some((last_start, _, _)) if *last_start > cluster_start => {
                return Err(ShapingRejectReason::ClusterMappingInvalid);
            }
            _ => groups.push((cluster_start, glyph_index, glyph_index + 1)),
        }
    }
    if groups.first().map(|group| group.0) != Some(0) {
        return Err(ShapingRejectReason::ClusterMappingInvalid);
    }

    let scalar_by_utf8 = text
        .char_indices()
        .enumerate()
        .map(|(scalar_index, (utf8_index, _))| (utf8_index, scalar_index))
        .chain(std::iter::once((text.len(), text.chars().count())))
        .collect::<HashMap<_, _>>();
    let mut clusters = Vec::with_capacity(groups.len());
    for (group_index, (utf8_start, glyph_start, glyph_end)) in groups.iter().copied().enumerate() {
        let utf8_end = groups
            .get(group_index + 1)
            .map(|group| group.0)
            .unwrap_or(text.len());
        if utf8_end <= utf8_start || !text.is_char_boundary(utf8_end) {
            return Err(ShapingRejectReason::ClusterMappingInvalid);
        }
        let advance_px = glyphs[glyph_start..glyph_end]
            .iter()
            .map(|glyph| glyph.advance_x)
            .sum::<f64>();
        if !advance_px.is_finite() || advance_px < 0.0 {
            return Err(ShapingRejectReason::ClusterMappingInvalid);
        }
        clusters.push(HorizontalShapingCluster {
            utf8_start,
            utf8_end,
            scalar_start: *scalar_by_utf8
                .get(&utf8_start)
                .ok_or(ShapingRejectReason::ClusterMappingInvalid)?,
            scalar_end: *scalar_by_utf8
                .get(&utf8_end)
                .ok_or(ShapingRejectReason::ClusterMappingInvalid)?,
            glyph_start,
            glyph_end,
            advance_px,
        });
    }
    Ok(clusters)
}
