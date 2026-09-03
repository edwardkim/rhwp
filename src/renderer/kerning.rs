//! Kerning request가 실제 pair positioning에 들어가기 전의 exact-font decision plane.
//!
//! 이 모듈은 family 이름이나 fallback 후보를 다시 찾지 않는다. 상위 font selection이 확정한
//! face bytes와 face index만 입력받고, 지원 여부와 bounded pair delta 후보를 계산한다. 실제
//! layout 적용은 후속 단계의 책임이다.

use rustybuzz::{shape, Direction, Feature, GlyphBuffer, UnicodeBuffer};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write as _;
use ttf_parser::{gpos::PositioningSubtable, kern, Face, Tag};

/// Q1에서 portable font blob 경계와 맞춰 동결한 exact source 상한.
pub(crate) const MAX_KERNING_FONT_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_KERNING_RUN_CODE_POINTS: usize = 4_096;
pub(crate) const MAX_KERNING_RUN_GLYPHS: usize = 4_096;
pub(crate) const MAX_KERNING_ADJACENT_PAIRS: usize = 4_095;
pub(crate) const MAX_KERNING_TRACE_RECORDS_PER_RUN: usize = 4_096;
/// 한 문단 transaction에서 최초 run, 공백 fallback, line-boundary 재측정이
/// 함께 소비할 수 있는 segment 상한이다.
pub(crate) const MAX_KERNING_PARAGRAPH_SEGMENTS: usize = 256;
/// 한 layout owner가 보존할 수 있는 exact face/source 수와 총 payload 상한.
///
/// 총 payload는 기존 portable/embedded page 경계(64 MiB)와 같고, 개별 source는
/// [`MAX_KERNING_FONT_BYTES`]가 먼저 제한한다. slot 수는 손상 문서가 같은 source에
/// 무제한 alias를 만드는 것을 막기 위해 별도로 제한한다.
pub(crate) const MAX_KERNING_REGISTRY_FACES: usize = 256;
pub(crate) const MAX_KERNING_REGISTRY_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_KERNING_REGISTRY_SLOTS: usize = 4_096;

/// HWP 글자모양과 언어별 font selection의 정확한 위치다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExactFontSlot {
    pub char_shape_id: u32,
    pub language_index: usize,
}

impl ExactFontSlot {
    pub(crate) fn new(char_shape_id: u32, language_index: usize) -> Self {
        Self {
            char_shape_id,
            language_index,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExactFontSource<'a> {
    pub bytes: &'a [u8],
    pub face_index: u32,
}

/// Font selection이 확정한 exact face의 source identity다.
///
/// bytes, 파일 경로, family 이름을 보존하지 않는다. layout과 host/document source
/// provider 사이에서 동일 source를 재확인하기 위한 handle이며 직렬화해도 font
/// payload나 private path가 노출되지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExactFontSourceHandle {
    pub font_source_sha256: String,
    pub font_bytes: usize,
    pub face_index: u32,
}

/// Content identity prepared once when immutable exact-source bytes enter the
/// registry. Paint resource publication can reuse it without scanning the
/// complete font during every layout or layer rebuild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactFontPortableResourceIdentity {
    digest_blake3: String,
    hash_fnv1a64: u64,
    fingerprint: [u8; 16],
}

impl ExactFontPortableResourceIdentity {
    fn from_bytes(bytes: &[u8]) -> Self {
        const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

        let hash_fnv1a64 = bytes.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        });
        let digest = blake3::hash(bytes);
        let mut fingerprint = [0; 16];
        fingerprint.copy_from_slice(&digest.as_bytes()[..16]);
        Self {
            digest_blake3: digest.to_hex().to_string(),
            hash_fnv1a64,
            fingerprint,
        }
    }

    pub(crate) fn digest_blake3(&self) -> &str {
        &self.digest_blake3
    }

    pub(crate) fn hash_fnv1a64(&self) -> u64 {
        self.hash_fnv1a64
    }

    pub(crate) fn fingerprint(&self) -> [u8; 16] {
        self.fingerprint
    }
}

#[derive(Clone)]
struct OwnedExactFontSource {
    bytes: std::sync::Arc<[u8]>,
    portable_resource: ExactFontPortableResourceIdentity,
}

/// Document/native/WASM host가 소유한 font bytes를 handle 수명 동안 빌려주는 경계다.
///
/// provider의 반환값은 신뢰하지 않는다. 반드시 [`resolve_exact_font_source`]가
/// byte length, face index, SHA-256을 다시 대사한 뒤 capability/shaping에 전달한다.
pub(crate) trait ExactFontSourceProvider {
    fn source_for_handle<'a>(
        &'a self,
        handle: &ExactFontSourceHandle,
    ) -> Option<ExactFontSource<'a>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExactFontRegistryRegistration {
    Registered,
    AlreadyRegistered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExactFontRegistryError {
    InvalidLanguageIndex,
    FontByteLimitExceeded,
    FaceLimitExceeded,
    TotalByteLimitExceeded,
    SlotLimitExceeded,
    SlotConflict,
}

impl ExactFontRegistryError {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::InvalidLanguageIndex => "invalid-language-index",
            Self::FontByteLimitExceeded => "font-byte-limit-exceeded",
            Self::FaceLimitExceeded => "face-limit-exceeded",
            Self::TotalByteLimitExceeded => "total-byte-limit-exceeded",
            Self::SlotLimitExceeded => "slot-limit-exceeded",
            Self::SlotConflict => "slot-conflict",
        }
    }
}

/// Layout owner가 보존하는 bounded exact-source registry다.
///
/// slot은 payload 없는 handle만 가리키고 source table이 immutable bytes를 소유한다.
/// 동일 handle을 여러 slot이 공유할 때 bytes는 한 번만 보존한다. 같은 slot을 다른
/// source로 덮어쓰는 동작은 selection provenance를 숨길 수 있으므로 fail-closed한다.
#[derive(Clone, Default)]
pub(crate) struct ExactFontSourceRegistry {
    slots: HashMap<ExactFontSlot, ExactFontSourceHandle>,
    sources: HashMap<ExactFontSourceHandle, OwnedExactFontSource>,
    total_source_bytes: usize,
    generation: u64,
}

impl std::fmt::Debug for ExactFontSourceRegistry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExactFontSourceRegistry")
            .field("slot_count", &self.slots.len())
            .field("source_count", &self.sources.len())
            .field("total_source_bytes", &self.total_source_bytes)
            .field("generation", &self.generation)
            .finish()
    }
}

impl ExactFontSourceRegistry {
    pub(crate) fn register(
        &mut self,
        slot: ExactFontSlot,
        source: ExactFontSource<'_>,
    ) -> Result<ExactFontRegistryRegistration, ExactFontRegistryError> {
        if slot.language_index >= 7 {
            return Err(ExactFontRegistryError::InvalidLanguageIndex);
        }
        let handle = identify_exact_font_source(source).map_err(|reason| match reason {
            ExactFontSourceResolutionReason::FontByteLimitExceeded => {
                ExactFontRegistryError::FontByteLimitExceeded
            }
            ExactFontSourceResolutionReason::SourceUnavailable
            | ExactFontSourceResolutionReason::FaceIndexMismatch
            | ExactFontSourceResolutionReason::ByteLengthMismatch
            | ExactFontSourceResolutionReason::Sha256Mismatch => {
                unreachable!("identity creation only checks the byte limit")
            }
        })?;

        if let Some(existing) = self.slots.get(&slot) {
            return if existing == &handle {
                Ok(ExactFontRegistryRegistration::AlreadyRegistered)
            } else {
                Err(ExactFontRegistryError::SlotConflict)
            };
        }
        if self.slots.len() >= MAX_KERNING_REGISTRY_SLOTS {
            return Err(ExactFontRegistryError::SlotLimitExceeded);
        }

        if !self.sources.contains_key(&handle) {
            if self.sources.len() >= MAX_KERNING_REGISTRY_FACES {
                return Err(ExactFontRegistryError::FaceLimitExceeded);
            }
            let Some(next_total) = self.total_source_bytes.checked_add(source.bytes.len()) else {
                return Err(ExactFontRegistryError::TotalByteLimitExceeded);
            };
            if next_total > MAX_KERNING_REGISTRY_BYTES {
                return Err(ExactFontRegistryError::TotalByteLimitExceeded);
            }
            self.sources.insert(
                handle.clone(),
                OwnedExactFontSource {
                    bytes: std::sync::Arc::<[u8]>::from(source.bytes.to_vec()),
                    portable_resource: ExactFontPortableResourceIdentity::from_bytes(source.bytes),
                },
            );
            self.total_source_bytes = next_total;
        }
        self.slots.insert(slot, handle);
        self.generation = self.generation.wrapping_add(1);
        Ok(ExactFontRegistryRegistration::Registered)
    }

    pub(crate) fn handle_for_slot(&self, slot: ExactFontSlot) -> Option<&ExactFontSourceHandle> {
        self.slots.get(&slot)
    }

    pub(crate) fn clear(&mut self) -> bool {
        if self.slots.is_empty() && self.sources.is_empty() {
            return false;
        }
        self.slots.clear();
        self.sources.clear();
        self.total_source_bytes = 0;
        self.generation = self.generation.wrapping_add(1);
        true
    }

    pub(crate) fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Return another owner of the exact immutable source already retained by
    /// this registry. The bytes are not copied; callers must still validate
    /// the handle through [`resolve_exact_font_source`] before using the Arc.
    pub(crate) fn source_arc_for_handle(
        &self,
        handle: &ExactFontSourceHandle,
    ) -> Option<std::sync::Arc<[u8]>> {
        self.sources
            .get(handle)
            .map(|source| std::sync::Arc::clone(&source.bytes))
    }

    /// Resolve bytes owned by this registry without hashing the complete
    /// source again.
    ///
    /// Unlike [`resolve_exact_font_source`], this is not an external-provider
    /// trust boundary. `register` created both the map key and immutable Arc
    /// from the same bytes after computing the handle, and callers can only
    /// retrieve an exact key already present in that map. Length and bounded
    /// size are still checked so a future storage change cannot silently
    /// weaken the invariant.
    pub(crate) fn resolve_owned_source_arc(
        &self,
        handle: &ExactFontSourceHandle,
    ) -> Result<std::sync::Arc<[u8]>, ExactFontSourceResolutionReason> {
        let source = self
            .sources
            .get(handle)
            .ok_or(ExactFontSourceResolutionReason::SourceUnavailable)?;
        let bytes = std::sync::Arc::clone(&source.bytes);
        if bytes.len() > MAX_KERNING_FONT_BYTES {
            return Err(ExactFontSourceResolutionReason::FontByteLimitExceeded);
        }
        if bytes.len() != handle.font_bytes {
            return Err(ExactFontSourceResolutionReason::ByteLengthMismatch);
        }
        Ok(bytes)
    }

    pub(crate) fn portable_resource_identity_for_handle(
        &self,
        handle: &ExactFontSourceHandle,
    ) -> Option<ExactFontPortableResourceIdentity> {
        self.sources
            .get(handle)
            .map(|source| source.portable_resource.clone())
    }

    /// Return the immutable exact source whose byte length and caller-owned
    /// identity predicate both match. Portable key syntax remains outside the
    /// registry so kerning ownership does not depend on the paint layer.
    pub(crate) fn source_arc_matching(
        &self,
        byte_len: usize,
        mut identity_matches: impl FnMut(&[u8]) -> bool,
    ) -> Option<std::sync::Arc<[u8]>> {
        self.sources.values().find_map(|source| {
            (source.bytes.len() == byte_len && identity_matches(source.bytes.as_ref()))
                .then(|| std::sync::Arc::clone(&source.bytes))
        })
    }

    pub(crate) fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub(crate) fn total_source_bytes(&self) -> usize {
        self.total_source_bytes
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

impl ExactFontSourceProvider for ExactFontSourceRegistry {
    fn source_for_handle<'a>(
        &'a self,
        handle: &ExactFontSourceHandle,
    ) -> Option<ExactFontSource<'a>> {
        let source = self.sources.get(handle)?;
        Some(ExactFontSource {
            bytes: &source.bytes,
            face_index: handle.face_index,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct KerningParagraphCacheKey {
    text: String,
    base_position_bits: Vec<u64>,
    scalar_styles: Vec<(ExactFontSlot, bool, u64, u64)>,
    hard_boundaries: Vec<bool>,
}

/// 한 pagination/edit transaction의 네 fresh-layout 소비자가 공유하는 exact
/// source generation과 owned paragraph measurement cache다.
///
/// Cache key는 측정 입력 전체를 보존하므로 hash 충돌만으로 다른 문단 결과를
/// 재사용하지 않는다. 원문과 position 입력은 메모리에만 있고 trace/직렬화에는
/// 노출되지 않으며 문단 상한(4,096 scalar)에 묶인다.
pub(crate) struct KerningMeasurementContext {
    registry: ExactFontSourceRegistry,
    paragraph_measurements: std::sync::Mutex<
        HashMap<KerningParagraphCacheKey, std::sync::Arc<KerningParagraphMeasurement>>,
    >,
}

impl std::fmt::Debug for KerningMeasurementContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("KerningMeasurementContext")
            .field("registry", &self.registry)
            .field(
                "cached_paragraph_count",
                &self
                    .paragraph_measurements
                    .lock()
                    .map(|cache| cache.len())
                    .unwrap_or_default(),
            )
            .finish()
    }
}

impl KerningMeasurementContext {
    pub(crate) fn new(registry: ExactFontSourceRegistry) -> Self {
        Self {
            registry,
            paragraph_measurements: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn layout_session(&self) -> KerningLayoutSession<'_> {
        KerningLayoutSession::new(&self.registry)
    }

    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry.generation()
    }

    pub(crate) fn paragraph_measurement(
        &self,
        text: &str,
        base_positions: Vec<f64>,
        scalar_styles: &[KerningParagraphScalarStyle],
        hard_boundaries: &[bool],
    ) -> std::sync::Arc<KerningParagraphMeasurement> {
        let key = KerningParagraphCacheKey {
            text: text.to_owned(),
            base_position_bits: base_positions.iter().map(|value| value.to_bits()).collect(),
            scalar_styles: scalar_styles
                .iter()
                .map(|style| {
                    (
                        style.slot,
                        style.requested,
                        style.effective_font_size_px.to_bits(),
                        style.width_ratio.to_bits(),
                    )
                })
                .collect(),
            hard_boundaries: hard_boundaries.to_vec(),
        };
        if let Ok(cache) = self.paragraph_measurements.lock() {
            if let Some(measurement) = cache.get(&key) {
                return std::sync::Arc::clone(measurement);
            }
        }

        let mut transaction = self.layout_session();
        let measurement = std::sync::Arc::new(measure_kerning_paragraph_segments(
            text,
            base_positions,
            scalar_styles,
            hard_boundaries,
            &mut transaction,
        ));
        if let Ok(mut cache) = self.paragraph_measurements.lock() {
            return std::sync::Arc::clone(
                cache
                    .entry(key)
                    .or_insert_with(|| std::sync::Arc::clone(&measurement)),
            );
        }
        measurement
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExactFontSourceResolutionReason {
    SourceUnavailable,
    FontByteLimitExceeded,
    FaceIndexMismatch,
    ByteLengthMismatch,
    Sha256Mismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningCapability {
    GposKern,
    LegacyKern,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningCapabilityFallbackReason {
    FontSourceUnavailable,
    FontByteLimitExceeded,
    MalformedSfnt,
    PairTableUnsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningCapabilityDecision {
    pub capability: KerningCapability,
    pub fallback_reason: Option<KerningCapabilityFallbackReason>,
    pub font_source_sha256: Option<String>,
    pub font_bytes: usize,
    pub face_index: u32,
    pub units_per_em: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningRequest {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningRunGate {
    NotRequested,
    Eligible,
    FailClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningRunFallbackReason {
    FontSourceUnavailable,
    FontByteLimitExceeded,
    MalformedSfnt,
    PairTableUnsupported,
    RunCodePointLimitExceeded,
    RunGlyphLimitExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningPairCandidateStatus {
    NotEligible,
    AdjustmentCandidate,
    NoAdjustmentCandidate,
    FailClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningPairCandidateFallbackReason {
    RunGateNotEligible,
    RunGateInputMismatch,
    FontSourceMismatch,
    ShapingUnavailable,
    UnsupportedDirection,
    ShapedGlyphLimitExceeded,
    NominalGlyphIdentityChanged,
    NominalClusterChanged,
    KerningGlyphIdentityChanged,
    KerningClusterChanged,
    TraceRecordLimitExceeded,
}

/// 기존 문자 경계값에 exact pair delta를 반영한 공통 run 측정의 최종 상태다.
///
/// `PairAdjusted`만 기존 positions와 다른 좌표를 소유한다. 나머지 상태는
/// `base_positions`를 그대로 소비하므로 kerning-off와 fail-closed 경로가 기존
/// 측정값을 bit-for-bit 보존한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningRunMeasurementDisposition {
    ExistingPositions,
    ExactSourceUnavailable,
    NoPairAdjustment,
    PairAdjusted,
    FailClosed,
}

/// Pair candidate를 기존 문자 경계값에 투영하지 못한 이유다.
///
/// 원문과 font payload는 남기지 않는다. source/capability/candidate의 더 세부적인
/// 원인은 각각 payload-free handle과 bounded trace에 보존한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningRunMeasurementFallbackReason {
    ExactSourceUnavailable,
    RunCodePointLimitExceeded,
    BasePositionCountMismatch,
    BasePositionNonFinite,
    SourceSessionFailClosed,
    PairEngineUnavailable,
    PairCandidateFailClosed,
    InvalidStyleScale,
    InvalidUnitsPerEm,
    CandidateAccountingMismatch,
    CandidateGlyphIndexOutOfRange,
    AdjustedPositionNonFinite,
    AdjustedPositionNonMonotonic,
}

/// 기존 run 측정과 exact-font pair candidate를 합친 owned 결과다.
///
/// `pair_adjusted_positions`는 실제 delta 적용 때만 할당한다. K0, source 부재,
/// unsupported/malformed source는 `positions()`가 소유한 base slice를 그대로
/// 반환한다. 이 구조로 불필요한 대형 복제를 피하면서도 R4C/R4D가 동일 측정값을
/// line decision과 backend replay에 전달할 수 있다.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningRunMeasurement {
    pub disposition: KerningRunMeasurementDisposition,
    pub fallback_reason: Option<KerningRunMeasurementFallbackReason>,
    pub source_handle: Option<ExactFontSourceHandle>,
    pub code_point_count: usize,
    pub code_point_limit_exceeded: bool,
    pub bounded_segment_count: usize,
    pub base_positions: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pair_adjusted_positions: Option<Vec<f64>>,
    pub advance_deltas: Vec<f64>,
    pub glyph_position_deltas: Vec<KerningGlyphPositionDeltaPx>,
    pub total_width: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<KerningSourceSessionTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<KerningPairCandidateDecision>,
}

impl KerningRunMeasurement {
    #[cfg(test)]
    pub(crate) fn positions(&self) -> &[f64] {
        self.pair_adjusted_positions
            .as_deref()
            .unwrap_or(&self.base_positions)
    }
}

/// 문단의 어느 문자 범위가 어느 exact slot의 R4B 측정을 소비했는지 보존한다.
///
/// 범위는 Unicode scalar index의 half-open interval이다. 원문이나 font payload는
/// 보존하지 않는다.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningParagraphSegmentMeasurement {
    pub start_index: usize,
    pub end_index: usize,
    pub slot: ExactFontSlot,
    pub measurement: KerningRunMeasurement,
}

/// 문단의 Unicode scalar 하나가 사용할 exact slot과 pair scale이다.
///
/// `char_shape_id`와 language는 `slot`에 들어 있고, 기존 scalar advance는
/// paragraph base positions가 소유한다. pair delta 환산에 실제로 필요한 request,
/// effective font size, 장평만 여기서 다시 고정한다.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningParagraphScalarStyle {
    pub slot: ExactFontSlot,
    pub requested: bool,
    pub effective_font_size_px: f64,
    pub width_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningParagraphMeasurementDisposition {
    ExistingPositions,
    PairAdjusted,
    FailClosed,
}

/// Segment 결과를 문단의 단일 position map으로 commit하지 못한 구조적 이유다.
///
/// 개별 run의 source/capability 실패는 R4B measurement에 남고 그 segment만 기존
/// positions를 쓴다. 이 enum은 문단 전체를 rollback해야 하는 range·회계·수치
/// 불일치만 나타낸다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningParagraphMeasurementFallbackReason {
    CodePointLimitExceeded,
    ScalarStyleCountMismatch,
    HardBoundaryCountMismatch,
    SegmentLimitExceeded,
    SegmentExecutionLimitExceeded,
    LineBoundaryRangeInvalid,
    LineWidthInvalid,
    BasePositionCountMismatch,
    BasePositionNonFinite,
    BasePositionNonMonotonic,
    SegmentRangeInvalid,
    SegmentRangeOverlap,
    SegmentCodePointCountMismatch,
    SegmentBasePositionMismatch,
    SegmentAdvanceCountMismatch,
    AdjustedPositionNonFinite,
    AdjustedPositionNonMonotonic,
}

/// 한 문단의 token·long-word·line-boundary 소비자가 공유하는 owned position map.
///
/// K0와 fail-closed는 `pair_adjusted_positions`를 만들지 않고 기존
/// `base_positions`를 그대로 반환한다. 실제 pair 적용 segment가 하나라도 있을
/// 때만 문단 단위 adjusted map을 commit한다.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningParagraphMeasurement {
    pub disposition: KerningParagraphMeasurementDisposition,
    pub fallback_reason: Option<KerningParagraphMeasurementFallbackReason>,
    pub code_point_count: usize,
    pub code_point_limit_exceeded: bool,
    pub bounded_segment_count: usize,
    /// 최초 homogeneous run과 공백 fallback run을 실제 측정한 횟수다.
    pub attempted_segment_count: usize,
    /// nominal identity 실패 때문에 공백 경계 fallback을 실행한 homogeneous run 수다.
    pub whitespace_fallback_run_count: usize,
    pub segment_limit_exceeded: bool,
    pub base_positions: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pair_adjusted_positions: Option<Vec<f64>>,
    pub segments: Vec<KerningParagraphSegmentMeasurement>,
}

impl KerningParagraphMeasurement {
    pub(crate) fn positions(&self) -> &[f64] {
        self.pair_adjusted_positions
            .as_deref()
            .unwrap_or(&self.base_positions)
    }

    /// 같은 owned map에서 half-open 문자 범위의 폭을 읽는다.
    pub(crate) fn range_width(&self, start_index: usize, end_index: usize) -> Option<f64> {
        if start_index > end_index || end_index > self.code_point_count {
            return None;
        }
        let positions = self.positions();
        Some(*positions.get(end_index)? - *positions.get(start_index)?)
    }
}

/// R4B segment들을 문단의 `N+1` position map 하나로 원자적으로 합친다.
///
/// 구조 검증이나 상한이 하나라도 실패하면 일부 pair delta를 남기지 않고 문단
/// 전체를 base positions로 rollback한다. 개별 segment의 fail-closed는 이미 그
/// measurement가 zero delta를 소유하므로 다른 검증된 segment의 적용을 막지 않는다.
#[cfg(test)]
pub(crate) fn compose_kerning_paragraph_measurement(
    code_point_count: usize,
    base_positions: Vec<f64>,
    segments: Vec<KerningParagraphSegmentMeasurement>,
) -> KerningParagraphMeasurement {
    let attempted_segment_count = segments.len();
    compose_kerning_paragraph_measurement_accounted(
        code_point_count,
        base_positions,
        segments,
        attempted_segment_count,
        0,
        false,
    )
}

fn compose_kerning_paragraph_measurement_accounted(
    code_point_count: usize,
    base_positions: Vec<f64>,
    segments: Vec<KerningParagraphSegmentMeasurement>,
    attempted_segment_count: usize,
    whitespace_fallback_run_count: usize,
    execution_limit_exceeded: bool,
) -> KerningParagraphMeasurement {
    let code_point_limit_exceeded = code_point_count > MAX_KERNING_RUN_CODE_POINTS;
    let segment_limit_exceeded = execution_limit_exceeded
        || segments.len() > MAX_KERNING_PARAGRAPH_SEGMENTS
        || attempted_segment_count > MAX_KERNING_PARAGRAPH_SEGMENTS;
    let bounded_segment_count = segments.len().min(MAX_KERNING_PARAGRAPH_SEGMENTS);

    macro_rules! rollback {
        ($reason:expr $(,)?) => {
            return KerningParagraphMeasurement {
                disposition: KerningParagraphMeasurementDisposition::FailClosed,
                fallback_reason: Some($reason),
                code_point_count,
                code_point_limit_exceeded,
                bounded_segment_count,
                attempted_segment_count: attempted_segment_count
                    .min(MAX_KERNING_PARAGRAPH_SEGMENTS),
                whitespace_fallback_run_count,
                segment_limit_exceeded,
                base_positions,
                pair_adjusted_positions: None,
                segments,
            }
        };
    }

    if code_point_limit_exceeded {
        rollback!(KerningParagraphMeasurementFallbackReason::CodePointLimitExceeded);
    }
    if segment_limit_exceeded {
        rollback!(if execution_limit_exceeded {
            KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded
        } else {
            KerningParagraphMeasurementFallbackReason::SegmentLimitExceeded
        });
    }
    if base_positions.len() != code_point_count.saturating_add(1) {
        rollback!(KerningParagraphMeasurementFallbackReason::BasePositionCountMismatch);
    }
    if base_positions.iter().any(|position| !position.is_finite()) {
        rollback!(KerningParagraphMeasurementFallbackReason::BasePositionNonFinite);
    }
    if base_positions.windows(2).any(|pair| pair[1] < pair[0]) {
        rollback!(KerningParagraphMeasurementFallbackReason::BasePositionNonMonotonic);
    }

    let mut paragraph_advance_deltas = vec![0.0; code_point_count];
    let mut previous_end = 0usize;
    let mut has_pair_adjustment = false;
    for segment in &segments {
        if segment.start_index >= segment.end_index || segment.end_index > code_point_count {
            rollback!(KerningParagraphMeasurementFallbackReason::SegmentRangeInvalid);
        }
        if segment.start_index < previous_end {
            rollback!(KerningParagraphMeasurementFallbackReason::SegmentRangeOverlap);
        }
        previous_end = segment.end_index;

        let segment_len = segment.end_index - segment.start_index;
        if segment.measurement.code_point_limit_exceeded
            || segment.measurement.code_point_count != segment_len
        {
            rollback!(KerningParagraphMeasurementFallbackReason::SegmentCodePointCountMismatch,);
        }
        if segment.measurement.base_positions.len() != segment_len.saturating_add(1) {
            rollback!(KerningParagraphMeasurementFallbackReason::SegmentBasePositionMismatch,);
        }
        let segment_origin = segment.measurement.base_positions[0];
        let paragraph_origin = base_positions[segment.start_index];
        let base_matches = segment.measurement.base_positions.iter().enumerate().all(
            |(offset, segment_position)| {
                let local_segment = *segment_position - segment_origin;
                let local_paragraph =
                    base_positions[segment.start_index + offset] - paragraph_origin;
                let tolerance = 1e-9_f64.max(local_paragraph.abs() * 1e-12);
                (local_segment - local_paragraph).abs() <= tolerance
            },
        );
        if !base_matches {
            rollback!(KerningParagraphMeasurementFallbackReason::SegmentBasePositionMismatch,);
        }

        if segment.measurement.disposition == KerningRunMeasurementDisposition::PairAdjusted {
            if segment.measurement.advance_deltas.len() != segment_len {
                rollback!(KerningParagraphMeasurementFallbackReason::SegmentAdvanceCountMismatch,);
            }
            has_pair_adjustment = true;
            for (offset, delta) in segment
                .measurement
                .advance_deltas
                .iter()
                .copied()
                .enumerate()
            {
                paragraph_advance_deltas[segment.start_index + offset] += delta;
            }
        }
    }

    if !has_pair_adjustment {
        return KerningParagraphMeasurement {
            disposition: KerningParagraphMeasurementDisposition::ExistingPositions,
            fallback_reason: None,
            code_point_count,
            code_point_limit_exceeded: false,
            bounded_segment_count,
            attempted_segment_count,
            whitespace_fallback_run_count,
            segment_limit_exceeded: false,
            base_positions,
            pair_adjusted_positions: None,
            segments,
        };
    }

    let mut adjusted_positions = Vec::with_capacity(base_positions.len());
    adjusted_positions.push(base_positions[0]);
    let mut cumulative_delta = 0.0;
    for (index, base_position) in base_positions.iter().copied().enumerate().skip(1) {
        cumulative_delta += paragraph_advance_deltas[index - 1];
        let adjusted = base_position + cumulative_delta;
        if !adjusted.is_finite() {
            rollback!(KerningParagraphMeasurementFallbackReason::AdjustedPositionNonFinite);
        }
        if adjusted
            < *adjusted_positions
                .last()
                .expect("paragraph position origin")
        {
            rollback!(KerningParagraphMeasurementFallbackReason::AdjustedPositionNonMonotonic,);
        }
        adjusted_positions.push(adjusted);
    }

    KerningParagraphMeasurement {
        disposition: KerningParagraphMeasurementDisposition::PairAdjusted,
        fallback_reason: None,
        code_point_count,
        code_point_limit_exceeded: false,
        bounded_segment_count,
        attempted_segment_count,
        whitespace_fallback_run_count,
        segment_limit_exceeded: false,
        base_positions,
        pair_adjusted_positions: Some(adjusted_positions),
        segments,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningGlyphPositionDelta {
    pub glyph_index: usize,
    pub glyph_id: u32,
    pub cluster: u32,
    pub x_advance: i64,
    pub y_advance: i64,
    pub x_offset: i64,
    pub y_offset: i64,
}

/// Backend가 design-unit 환산을 다시 하지 않고 재생할 수 있는 px delta다.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningGlyphPositionDeltaPx {
    pub glyph_index: usize,
    pub glyph_id: u32,
    pub cluster: u32,
    pub x_advance: f64,
    pub y_advance: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

/// `kern=0/1` shaping 차이를 보존한 bounded 후보다. 이 결과는 아직 layout 적용 판정이 아니다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningPairCandidateDecision {
    pub status: KerningPairCandidateStatus,
    pub capability: KerningCapability,
    pub font_source_sha256: Option<String>,
    pub face_index: u32,
    pub units_per_em: Option<u16>,
    pub glyph_count: usize,
    pub examined_pair_count: usize,
    pub adjusted_position_count: usize,
    pub total_x_advance_delta: i64,
    pub position_deltas: Vec<KerningGlyphPositionDelta>,
    pub fallback_reason: Option<KerningPairCandidateFallbackReason>,
}

/// exact source 검증과 face parse를 한 번만 수행하고 여러 bounded run에서 재사용한다.
pub(crate) struct KerningPairEngine<'a> {
    face: rustybuzz::Face<'a>,
    capability: KerningCapabilityDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KerningSourceSessionStatus {
    Ready,
    FailClosed,
}

/// Layout session이 exact handle을 준비한 결과다. payload와 원문은 남기지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningSourceSessionTrace {
    pub status: KerningSourceSessionStatus,
    pub cache_hit: bool,
    pub handle: ExactFontSourceHandle,
    pub capability: KerningCapabilityDecision,
    pub resolution_reason: Option<ExactFontSourceResolutionReason>,
    pub pair_engine_reason: Option<KerningPairCandidateFallbackReason>,
}

struct KerningSourceSessionEntry<'a> {
    trace: KerningSourceSessionTrace,
    engine: Option<KerningPairEngine<'a>>,
}

/// 한 번의 layout/reflow가 공유하는 exact-source capability·pair-engine cache다.
///
/// provider가 bytes를 소유하고 session은 그 수명 안에서만 face를 빌린다. host의
/// font registry가 바뀌면 새 session을 만들어야 하므로 unavailable 결과도 한
/// session 안에서는 결정적으로 cache된다.
pub(crate) struct KerningSourceSession<'a> {
    provider: &'a dyn ExactFontSourceProvider,
    entries: HashMap<ExactFontSourceHandle, KerningSourceSessionEntry<'a>>,
}

impl From<KerningCapabilityFallbackReason> for KerningRunFallbackReason {
    fn from(value: KerningCapabilityFallbackReason) -> Self {
        match value {
            KerningCapabilityFallbackReason::FontSourceUnavailable => Self::FontSourceUnavailable,
            KerningCapabilityFallbackReason::FontByteLimitExceeded => Self::FontByteLimitExceeded,
            KerningCapabilityFallbackReason::MalformedSfnt => Self::MalformedSfnt,
            KerningCapabilityFallbackReason::PairTableUnsupported => Self::PairTableUnsupported,
        }
    }
}

/// Pair engine 전 단계의 bounded run 판정이다. `Eligible`은 적용 완료가 아니며,
/// 후속 candidate 검증과 layout commit이 최종 disposition을 결정해야 한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningRunGateDecision {
    pub request: KerningRequest,
    pub capability: KerningCapability,
    pub gate: KerningRunGate,
    pub font_source_sha256: Option<String>,
    pub font_bytes: usize,
    pub face_index: u32,
    pub units_per_em: Option<u16>,
    pub code_point_count: usize,
    pub code_point_limit_exceeded: bool,
    pub glyph_count: usize,
    pub glyph_limit_exceeded: bool,
    pub candidate_pair_count: usize,
    pub fallback_reason: Option<KerningRunFallbackReason>,
}

impl KerningCapabilityDecision {
    fn fail_closed(
        reason: KerningCapabilityFallbackReason,
        font_bytes: usize,
        face_index: u32,
        font_source_sha256: Option<String>,
    ) -> Self {
        Self {
            capability: KerningCapability::Unsupported,
            fallback_reason: Some(reason),
            font_source_sha256,
            font_bytes,
            face_index,
            units_per_em: None,
        }
    }
}

fn font_source_sha256(bytes: &[u8]) -> String {
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut digest, "{byte:02x}").expect("String formatting cannot fail");
    }
    digest
}

/// Selection 시점의 exact source에서 payload 없는 identity handle을 만든다.
pub(crate) fn identify_exact_font_source(
    source: ExactFontSource<'_>,
) -> Result<ExactFontSourceHandle, ExactFontSourceResolutionReason> {
    if source.bytes.len() > MAX_KERNING_FONT_BYTES {
        return Err(ExactFontSourceResolutionReason::FontByteLimitExceeded);
    }
    Ok(ExactFontSourceHandle {
        font_source_sha256: font_source_sha256(source.bytes),
        font_bytes: source.bytes.len(),
        face_index: source.face_index,
    })
}

/// Provider가 반환한 source가 selection handle과 정확히 같은지 bounded하게 대사한다.
pub(crate) fn resolve_exact_font_source<'a>(
    provider: &'a dyn ExactFontSourceProvider,
    handle: &ExactFontSourceHandle,
) -> Result<ExactFontSource<'a>, ExactFontSourceResolutionReason> {
    let source = provider
        .source_for_handle(handle)
        .ok_or(ExactFontSourceResolutionReason::SourceUnavailable)?;
    if source.bytes.len() > MAX_KERNING_FONT_BYTES {
        return Err(ExactFontSourceResolutionReason::FontByteLimitExceeded);
    }
    if source.face_index != handle.face_index {
        return Err(ExactFontSourceResolutionReason::FaceIndexMismatch);
    }
    if source.bytes.len() != handle.font_bytes {
        return Err(ExactFontSourceResolutionReason::ByteLengthMismatch);
    }
    if font_source_sha256(source.bytes) != handle.font_source_sha256 {
        return Err(ExactFontSourceResolutionReason::Sha256Mismatch);
    }
    Ok(source)
}

/// 선택이 끝난 exact face source의 kerning capability를 bounded하게 판정한다.
///
/// `None`은 시스템 font 이름만 있거나 fallback 결과의 bytes를 증명할 수 없는 경우다. source가
/// 없거나 손상됐거나 상한을 넘으면 추측하지 않고 `Unsupported`로 닫는다.
#[cfg(test)]
pub(crate) fn inspect_exact_font_kerning(
    source: Option<ExactFontSource<'_>>,
) -> KerningCapabilityDecision {
    let Some(source) = source else {
        return KerningCapabilityDecision::fail_closed(
            KerningCapabilityFallbackReason::FontSourceUnavailable,
            0,
            0,
            None,
        );
    };
    if source.bytes.len() > MAX_KERNING_FONT_BYTES {
        return KerningCapabilityDecision::fail_closed(
            KerningCapabilityFallbackReason::FontByteLimitExceeded,
            source.bytes.len(),
            source.face_index,
            None,
        );
    }

    let digest = font_source_sha256(source.bytes);
    inspect_verified_exact_font_kerning(source, digest)
}

/// Handle 대사가 끝난 source를 parse한다. 이 경로에서는 SHA-256을 다시 계산하지 않는다.
fn inspect_verified_exact_font_kerning(
    source: ExactFontSource<'_>,
    digest: String,
) -> KerningCapabilityDecision {
    let Ok(face) = Face::parse(source.bytes, source.face_index) else {
        return KerningCapabilityDecision::fail_closed(
            KerningCapabilityFallbackReason::MalformedSfnt,
            source.bytes.len(),
            source.face_index,
            Some(digest),
        );
    };

    let capability = if has_gpos_kern_pair_lookup(&face) {
        KerningCapability::GposKern
    } else if has_legacy_horizontal_format0(&face) {
        KerningCapability::LegacyKern
    } else {
        KerningCapability::Unsupported
    };
    KerningCapabilityDecision {
        capability,
        fallback_reason: (capability == KerningCapability::Unsupported)
            .then_some(KerningCapabilityFallbackReason::PairTableUnsupported),
        font_source_sha256: Some(digest),
        font_bytes: source.bytes.len(),
        face_index: source.face_index,
        units_per_em: Some(face.units_per_em()),
    }
}

/// 문서 request와 exact-font capability를 결합해 pair engine 진입 가능 여부를 판정한다.
///
/// code point는 `MAX + 1`까지만 순회하고 trace 수치는 상한으로 clamp한다. 따라서 공격자가 긴
/// 문자열이나 비정상 glyph count를 주더라도 이 단계의 시간·출력 크기는 bounded하다.
pub(crate) fn decide_kerning_run_gate(
    requested: bool,
    text: &str,
    glyph_count: usize,
    capability: &KerningCapabilityDecision,
) -> KerningRunGateDecision {
    let observed_code_points = text.chars().take(MAX_KERNING_RUN_CODE_POINTS + 1).count();
    let code_point_limit_exceeded = observed_code_points > MAX_KERNING_RUN_CODE_POINTS;
    let bounded_code_points = observed_code_points.min(MAX_KERNING_RUN_CODE_POINTS);
    let glyph_limit_exceeded = glyph_count > MAX_KERNING_RUN_GLYPHS;
    let bounded_glyphs = glyph_count.min(MAX_KERNING_RUN_GLYPHS);
    let candidate_pair_count = bounded_glyphs
        .saturating_sub(1)
        .min(MAX_KERNING_ADJACENT_PAIRS);
    let request = if requested {
        KerningRequest::Enabled
    } else {
        KerningRequest::Disabled
    };

    let (gate, fallback_reason) = if !requested {
        (KerningRunGate::NotRequested, None)
    } else if code_point_limit_exceeded {
        (
            KerningRunGate::FailClosed,
            Some(KerningRunFallbackReason::RunCodePointLimitExceeded),
        )
    } else if glyph_limit_exceeded {
        (
            KerningRunGate::FailClosed,
            Some(KerningRunFallbackReason::RunGlyphLimitExceeded),
        )
    } else if capability.capability == KerningCapability::Unsupported {
        (
            KerningRunGate::FailClosed,
            Some(
                capability
                    .fallback_reason
                    .map(KerningRunFallbackReason::from)
                    .unwrap_or(KerningRunFallbackReason::PairTableUnsupported),
            ),
        )
    } else {
        (KerningRunGate::Eligible, None)
    };

    KerningRunGateDecision {
        request,
        capability: capability.capability,
        gate,
        font_source_sha256: capability.font_source_sha256.clone(),
        font_bytes: capability.font_bytes,
        face_index: capability.face_index,
        units_per_em: capability.units_per_em,
        code_point_count: bounded_code_points,
        code_point_limit_exceeded,
        glyph_count: bounded_glyphs,
        glyph_limit_exceeded,
        candidate_pair_count,
        fallback_reason,
    }
}

impl KerningPairCandidateDecision {
    fn fail_closed(
        gate: &KerningRunGateDecision,
        status: KerningPairCandidateStatus,
        reason: KerningPairCandidateFallbackReason,
    ) -> Self {
        Self {
            status,
            capability: gate.capability,
            font_source_sha256: gate.font_source_sha256.clone(),
            face_index: gate.face_index,
            units_per_em: gate.units_per_em,
            glyph_count: 0,
            examined_pair_count: 0,
            adjusted_position_count: 0,
            total_x_advance_delta: 0,
            position_deltas: Vec::new(),
            fallback_reason: Some(reason),
        }
    }
}

/// Q3-2 capability와 source bytes를 한 번 대사해 재사용 가능한 pair engine을 준비한다.
#[cfg(test)]
pub(crate) fn prepare_kerning_pair_engine<'a>(
    source: ExactFontSource<'a>,
    expected: &KerningCapabilityDecision,
) -> Result<KerningPairEngine<'a>, KerningPairCandidateFallbackReason> {
    let capability = inspect_exact_font_kerning(Some(source));
    if capability.capability != expected.capability
        || capability.font_source_sha256 != expected.font_source_sha256
        || capability.font_bytes != expected.font_bytes
        || capability.face_index != expected.face_index
        || capability.units_per_em != expected.units_per_em
    {
        return Err(KerningPairCandidateFallbackReason::FontSourceMismatch);
    }
    prepare_verified_kerning_pair_engine(source, capability)
}

/// 이미 exact-source 대사와 capability parse가 끝난 source로 pair engine을 만든다.
fn prepare_verified_kerning_pair_engine<'a>(
    source: ExactFontSource<'a>,
    capability: KerningCapabilityDecision,
) -> Result<KerningPairEngine<'a>, KerningPairCandidateFallbackReason> {
    let face = rustybuzz::Face::from_slice(source.bytes, source.face_index)
        .ok_or(KerningPairCandidateFallbackReason::ShapingUnavailable)?;
    Ok(KerningPairEngine { face, capability })
}

impl<'a> KerningSourceSession<'a> {
    pub(crate) fn new(provider: &'a dyn ExactFontSourceProvider) -> Self {
        Self {
            provider,
            entries: HashMap::new(),
        }
    }

    /// Exact handle 하나를 session에 준비한다.
    ///
    /// 최초 호출만 provider 조회, SHA-256 대사, SFNT parse를 수행한다. 성공과 실패를 모두
    /// cache하므로 동일 layout/reflow 중 host 상태 변화가 결과를 비결정적으로 바꾸지 않는다.
    pub(crate) fn prepare(&mut self, handle: &ExactFontSourceHandle) -> KerningSourceSessionTrace {
        if let Some(entry) = self.entries.get(handle) {
            let mut trace = entry.trace.clone();
            trace.cache_hit = true;
            return trace;
        }

        let provider: &'a dyn ExactFontSourceProvider = self.provider;
        let entry = match resolve_exact_font_source(provider, handle) {
            Err(reason) => KerningSourceSessionEntry {
                trace: KerningSourceSessionTrace {
                    status: KerningSourceSessionStatus::FailClosed,
                    cache_hit: false,
                    handle: handle.clone(),
                    capability: capability_for_resolution_failure(handle, reason),
                    resolution_reason: Some(reason),
                    pair_engine_reason: None,
                },
                engine: None,
            },
            Ok(source) => {
                let capability =
                    inspect_verified_exact_font_kerning(source, handle.font_source_sha256.clone());
                if capability.capability == KerningCapability::Unsupported {
                    KerningSourceSessionEntry {
                        trace: KerningSourceSessionTrace {
                            status: KerningSourceSessionStatus::FailClosed,
                            cache_hit: false,
                            handle: handle.clone(),
                            capability,
                            resolution_reason: None,
                            pair_engine_reason: None,
                        },
                        engine: None,
                    }
                } else {
                    match prepare_verified_kerning_pair_engine(source, capability.clone()) {
                        Ok(engine) => KerningSourceSessionEntry {
                            trace: KerningSourceSessionTrace {
                                status: KerningSourceSessionStatus::Ready,
                                cache_hit: false,
                                handle: handle.clone(),
                                capability,
                                resolution_reason: None,
                                pair_engine_reason: None,
                            },
                            engine: Some(engine),
                        },
                        Err(reason) => KerningSourceSessionEntry {
                            trace: KerningSourceSessionTrace {
                                status: KerningSourceSessionStatus::FailClosed,
                                cache_hit: false,
                                handle: handle.clone(),
                                capability,
                                resolution_reason: None,
                                pair_engine_reason: Some(reason),
                            },
                            engine: None,
                        },
                    }
                }
            }
        };

        let trace = entry.trace.clone();
        self.entries.insert(handle.clone(), entry);
        trace
    }

    /// `prepare`가 성공한 exact handle의 engine만 빌려준다.
    pub(crate) fn engine(&self, handle: &ExactFontSourceHandle) -> Option<&KerningPairEngine<'a>> {
        self.entries.get(handle)?.engine.as_ref()
    }
}

/// 한 layout/reflow transaction에서 slot binding과 per-face source cache를 함께 고정한다.
///
/// registry는 transaction 수명 동안 불변으로 빌리므로 generation과 slot→handle
/// 대응이 중간에 바뀔 수 없다. source bytes는 외부 registry가 계속 소유하고 이
/// 객체는 복제하거나 trace에 싣지 않는다.
pub(crate) struct KerningLayoutSession<'a> {
    registry: &'a ExactFontSourceRegistry,
    registry_generation: u64,
    source_session: KerningSourceSession<'a>,
}

impl<'a> KerningLayoutSession<'a> {
    pub(crate) fn new(registry: &'a ExactFontSourceRegistry) -> Self {
        Self {
            registry,
            registry_generation: registry.generation(),
            source_session: KerningSourceSession::new(registry),
        }
    }

    #[cfg(test)]
    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub(crate) fn source_handle(&self, slot: ExactFontSlot) -> Option<&ExactFontSourceHandle> {
        self.registry.handle_for_slot(slot)
    }

    /// Slot 해소와 R4B run 측정을 같은 transaction에서 수행한다.
    ///
    /// K0는 slot lookup도 생략하고 R4B의 source-미접근 fast path로 바로 들어간다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn measure_run(
        &mut self,
        slot: ExactFontSlot,
        text: &str,
        requested: bool,
        base_positions: Vec<f64>,
        effective_font_size_px: f64,
        width_ratio: f64,
    ) -> KerningRunMeasurement {
        let handle = requested
            .then(|| self.registry.handle_for_slot(slot).cloned())
            .flatten();
        compute_kerning_run_measurement(
            text,
            requested,
            base_positions,
            effective_font_size_px,
            width_ratio,
            handle.as_ref(),
            &mut self.source_session,
        )
    }
}

fn paragraph_measurement_fail_closed(
    code_point_count: usize,
    code_point_limit_exceeded: bool,
    base_positions: Vec<f64>,
    reason: KerningParagraphMeasurementFallbackReason,
    attempted_segment_count: usize,
    whitespace_fallback_run_count: usize,
    segment_limit_exceeded: bool,
) -> KerningParagraphMeasurement {
    KerningParagraphMeasurement {
        disposition: KerningParagraphMeasurementDisposition::FailClosed,
        fallback_reason: Some(reason),
        code_point_count,
        code_point_limit_exceeded,
        bounded_segment_count: 0,
        attempted_segment_count: attempted_segment_count.min(MAX_KERNING_PARAGRAPH_SEGMENTS),
        whitespace_fallback_run_count,
        segment_limit_exceeded,
        base_positions,
        pair_adjusted_positions: None,
        segments: Vec::new(),
    }
}

fn same_kerning_scalar_style(
    left: KerningParagraphScalarStyle,
    right: KerningParagraphScalarStyle,
) -> bool {
    left.slot == right.slot
        && left.requested == right.requested
        && left.effective_font_size_px.to_bits() == right.effective_font_size_px.to_bits()
        && left.width_ratio.to_bits() == right.width_ratio.to_bits()
}

fn needs_whitespace_identity_fallback(measurement: &KerningRunMeasurement) -> bool {
    matches!(
        measurement
            .candidate
            .as_ref()
            .and_then(|candidate| candidate.fallback_reason),
        Some(
            KerningPairCandidateFallbackReason::NominalGlyphIdentityChanged
                | KerningPairCandidateFallbackReason::NominalClusterChanged
        )
    )
}

#[allow(clippy::too_many_arguments)]
fn measure_kerning_paragraph_range(
    text: &str,
    byte_offsets: &[usize],
    base_positions: &[f64],
    start_index: usize,
    end_index: usize,
    style: KerningParagraphScalarStyle,
    transaction: &mut KerningLayoutSession<'_>,
    attempted_segment_count: &mut usize,
) -> Option<KerningParagraphSegmentMeasurement> {
    if *attempted_segment_count >= MAX_KERNING_PARAGRAPH_SEGMENTS {
        return None;
    }
    *attempted_segment_count += 1;
    let segment_text = &text[byte_offsets[start_index]..byte_offsets[end_index]];
    let measurement = transaction.measure_run(
        style.slot,
        segment_text,
        style.requested,
        base_positions[start_index..=end_index].to_vec(),
        style.effective_font_size_px,
        style.width_ratio,
    );
    Some(KerningParagraphSegmentMeasurement {
        start_index,
        end_index,
        slot: style.slot,
        measurement,
    })
}

/// 문단을 style/language/control 경계에서 한 번 분할하고 R4B 측정을 실행한다.
///
/// `scalar_styles`는 Unicode scalar와 1:1이고 `hard_boundaries`는 `N+1` 문자
/// 경계와 1:1이다. inline control이 scalar 사이에 있으면 그 위치를 `true`로
/// 넘긴다. 탭과 강제 줄바꿈은 문자 자체로 차단한다. nominal glyph/cluster identity
/// 실패 run만 공백 경계의 비공백 sub-run으로 한 번 재분할한다.
///
/// 최초 run과 fallback sub-run을 합쳐 256회까지만 측정한다. 다음 측정이 필요하면
/// 이미 계산한 일부 결과를 버리고 문단 전체를 base positions로 rollback한다.
pub(crate) fn measure_kerning_paragraph_segments(
    text: &str,
    base_positions: Vec<f64>,
    scalar_styles: &[KerningParagraphScalarStyle],
    hard_boundaries: &[bool],
    transaction: &mut KerningLayoutSession<'_>,
) -> KerningParagraphMeasurement {
    let observed_code_points = text.chars().take(MAX_KERNING_RUN_CODE_POINTS + 1).count();
    let code_point_limit_exceeded = observed_code_points > MAX_KERNING_RUN_CODE_POINTS;
    let code_point_count = observed_code_points.min(MAX_KERNING_RUN_CODE_POINTS);
    if code_point_limit_exceeded {
        return paragraph_measurement_fail_closed(
            code_point_count,
            true,
            base_positions,
            KerningParagraphMeasurementFallbackReason::CodePointLimitExceeded,
            0,
            0,
            false,
        );
    }
    if scalar_styles.len() != code_point_count {
        return paragraph_measurement_fail_closed(
            code_point_count,
            false,
            base_positions,
            KerningParagraphMeasurementFallbackReason::ScalarStyleCountMismatch,
            0,
            0,
            false,
        );
    }
    if hard_boundaries.len() != code_point_count.saturating_add(1) {
        return paragraph_measurement_fail_closed(
            code_point_count,
            false,
            base_positions,
            KerningParagraphMeasurementFallbackReason::HardBoundaryCountMismatch,
            0,
            0,
            false,
        );
    }
    if base_positions.len() != code_point_count.saturating_add(1) {
        return paragraph_measurement_fail_closed(
            code_point_count,
            false,
            base_positions,
            KerningParagraphMeasurementFallbackReason::BasePositionCountMismatch,
            0,
            0,
            false,
        );
    }
    if base_positions.iter().any(|position| !position.is_finite()) {
        return paragraph_measurement_fail_closed(
            code_point_count,
            false,
            base_positions,
            KerningParagraphMeasurementFallbackReason::BasePositionNonFinite,
            0,
            0,
            false,
        );
    }
    if base_positions.windows(2).any(|pair| pair[1] < pair[0]) {
        return paragraph_measurement_fail_closed(
            code_point_count,
            false,
            base_positions,
            KerningParagraphMeasurementFallbackReason::BasePositionNonMonotonic,
            0,
            0,
            false,
        );
    }

    let mut characters = Vec::with_capacity(code_point_count);
    let mut byte_offsets = Vec::with_capacity(code_point_count.saturating_add(1));
    for (byte_offset, character) in text.char_indices() {
        byte_offsets.push(byte_offset);
        characters.push(character);
    }
    byte_offsets.push(text.len());

    let mut segments = Vec::new();
    let mut attempted_segment_count = 0usize;
    let mut whitespace_fallback_run_count = 0usize;
    let mut start_index = 0usize;
    while start_index < code_point_count {
        if matches!(characters[start_index], '\t' | '\n' | '\r') {
            start_index += 1;
            continue;
        }

        let style = scalar_styles[start_index];
        let mut end_index = start_index + 1;
        while end_index < code_point_count
            && !hard_boundaries[end_index]
            && !matches!(characters[end_index], '\t' | '\n' | '\r')
            && same_kerning_scalar_style(style, scalar_styles[end_index])
        {
            end_index += 1;
        }

        let Some(initial) = measure_kerning_paragraph_range(
            text,
            &byte_offsets,
            &base_positions,
            start_index,
            end_index,
            style,
            transaction,
            &mut attempted_segment_count,
        ) else {
            return paragraph_measurement_fail_closed(
                code_point_count,
                false,
                base_positions,
                KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded,
                attempted_segment_count,
                whitespace_fallback_run_count,
                true,
            );
        };

        let has_whitespace = characters[start_index..end_index]
            .iter()
            .any(|character| character.is_whitespace());
        if needs_whitespace_identity_fallback(&initial.measurement) && has_whitespace {
            whitespace_fallback_run_count += 1;
            let mut fallback_start = start_index;
            while fallback_start < end_index {
                while fallback_start < end_index && characters[fallback_start].is_whitespace() {
                    fallback_start += 1;
                }
                if fallback_start == end_index {
                    break;
                }
                let mut fallback_end = fallback_start + 1;
                while fallback_end < end_index && !characters[fallback_end].is_whitespace() {
                    fallback_end += 1;
                }
                let Some(fallback) = measure_kerning_paragraph_range(
                    text,
                    &byte_offsets,
                    &base_positions,
                    fallback_start,
                    fallback_end,
                    style,
                    transaction,
                    &mut attempted_segment_count,
                ) else {
                    return paragraph_measurement_fail_closed(
                        code_point_count,
                        false,
                        base_positions,
                        KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded,
                        attempted_segment_count,
                        whitespace_fallback_run_count,
                        true,
                    );
                };
                segments.push(fallback);
                fallback_start = fallback_end;
            }
        } else {
            segments.push(initial);
        }
        start_index = end_index;
    }

    compose_kerning_paragraph_measurement_accounted(
        code_point_count,
        base_positions,
        segments,
        attempted_segment_count,
        whitespace_fallback_run_count,
        false,
    )
}

/// 긴 단어의 최초 후보와 boundary-safe 재측정 결과다.
///
/// 원문이나 source payload 없이 문자 경계와 폭, bounded 실행 회계만 남긴다.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KerningLineBoundaryDecision {
    pub initial_end_index: usize,
    pub final_end_index: usize,
    pub final_width: f64,
    pub overflow_forced: bool,
    pub attempted_segment_count: usize,
}

/// R4C-1 문단 측정과 같은 transaction에서 token·긴 단어·line boundary를 읽는다.
///
/// 최초 후보는 문단의 owned positions를 소비한다. 실제 줄 경계는 잘린 앞/뒤 glyph
/// pair가 같은 run에 남지 않도록 해당 substring을 R4B로 다시 측정한다. 재측정도
/// 최초 segmentation과 같은 256회 예산을 이어받는다.
pub(crate) struct KerningParagraphBreakSession<'input, 'registry, 'transaction> {
    text: &'input str,
    characters: Vec<char>,
    byte_offsets: Vec<usize>,
    base_positions: &'input [f64],
    scalar_styles: &'input [KerningParagraphScalarStyle],
    hard_boundaries: &'input [bool],
    paragraph: &'input KerningParagraphMeasurement,
    transaction: &'transaction mut KerningLayoutSession<'registry>,
    attempted_segment_count: usize,
    failed_reason: Option<KerningParagraphMeasurementFallbackReason>,
    boundary_width_cache: HashMap<(usize, usize), f64>,
}

impl<'input, 'registry, 'transaction>
    KerningParagraphBreakSession<'input, 'registry, 'transaction>
{
    pub(crate) fn new(
        text: &'input str,
        scalar_styles: &'input [KerningParagraphScalarStyle],
        hard_boundaries: &'input [bool],
        paragraph: &'input KerningParagraphMeasurement,
        transaction: &'transaction mut KerningLayoutSession<'registry>,
    ) -> Result<Self, KerningParagraphMeasurementFallbackReason> {
        if paragraph.disposition == KerningParagraphMeasurementDisposition::FailClosed {
            return Err(paragraph
                .fallback_reason
                .unwrap_or(KerningParagraphMeasurementFallbackReason::LineBoundaryRangeInvalid));
        }
        let characters: Vec<char> = text.chars().take(MAX_KERNING_RUN_CODE_POINTS + 1).collect();
        if characters.len() > MAX_KERNING_RUN_CODE_POINTS
            || characters.len() != paragraph.code_point_count
        {
            return Err(KerningParagraphMeasurementFallbackReason::CodePointLimitExceeded);
        }
        if scalar_styles.len() != characters.len() {
            return Err(KerningParagraphMeasurementFallbackReason::ScalarStyleCountMismatch);
        }
        if hard_boundaries.len() != characters.len().saturating_add(1) {
            return Err(KerningParagraphMeasurementFallbackReason::HardBoundaryCountMismatch);
        }
        if paragraph.base_positions.len() != characters.len().saturating_add(1) {
            return Err(KerningParagraphMeasurementFallbackReason::BasePositionCountMismatch);
        }

        let mut byte_offsets = Vec::with_capacity(characters.len().saturating_add(1));
        byte_offsets.extend(text.char_indices().map(|(offset, _)| offset));
        byte_offsets.push(text.len());
        Ok(Self {
            text,
            characters,
            byte_offsets,
            base_positions: &paragraph.base_positions,
            scalar_styles,
            hard_boundaries,
            paragraph,
            transaction,
            attempted_segment_count: paragraph.attempted_segment_count,
            failed_reason: None,
            boundary_width_cache: HashMap::new(),
        })
    }

    /// Token total과 최초 긴 단어 후보가 읽는 공통 문단 position range다.
    #[cfg(test)]
    pub(crate) fn range_width(&self, start_index: usize, end_index: usize) -> Option<f64> {
        self.paragraph.range_width(start_index, end_index)
    }

    #[cfg(test)]
    pub(crate) fn attempted_segment_count(&self) -> usize {
        self.attempted_segment_count
    }

    pub(crate) fn failed_reason(&self) -> Option<KerningParagraphMeasurementFallbackReason> {
        self.failed_reason
    }

    fn fail(&mut self, reason: KerningParagraphMeasurementFallbackReason) -> Option<f64> {
        self.failed_reason.get_or_insert(reason);
        None
    }

    fn measure_boundary_segment(
        &mut self,
        start_index: usize,
        end_index: usize,
        style: KerningParagraphScalarStyle,
    ) -> Option<KerningParagraphSegmentMeasurement> {
        measure_kerning_paragraph_range(
            self.text,
            &self.byte_offsets,
            self.base_positions,
            start_index,
            end_index,
            style,
            self.transaction,
            &mut self.attempted_segment_count,
        )
    }

    /// 실제 줄 substring을 다시 측정해 경계를 가로지르던 pair adjustment를 제거한다.
    pub(crate) fn boundary_width(&mut self, start_index: usize, end_index: usize) -> Option<f64> {
        if self.failed_reason.is_some() {
            return None;
        }
        if start_index > end_index || end_index > self.characters.len() {
            return self.fail(KerningParagraphMeasurementFallbackReason::LineBoundaryRangeInvalid);
        }
        if let Some(width) = self
            .boundary_width_cache
            .get(&(start_index, end_index))
            .copied()
        {
            return Some(width);
        }
        if start_index == end_index {
            self.boundary_width_cache
                .insert((start_index, end_index), 0.0);
            return Some(0.0);
        }

        let mut pair_delta = 0.0;
        let mut run_start = start_index;
        while run_start < end_index {
            if matches!(self.characters[run_start], '\t' | '\n' | '\r') {
                run_start += 1;
                continue;
            }
            let style = self.scalar_styles[run_start];
            let mut run_end = run_start + 1;
            while run_end < end_index
                && !self.hard_boundaries[run_end]
                && !matches!(self.characters[run_end], '\t' | '\n' | '\r')
                && same_kerning_scalar_style(style, self.scalar_styles[run_end])
            {
                run_end += 1;
            }

            let Some(initial) = self.measure_boundary_segment(run_start, run_end, style) else {
                return self.fail(
                    KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded,
                );
            };
            let has_whitespace = self.characters[run_start..run_end]
                .iter()
                .any(|character| character.is_whitespace());
            if needs_whitespace_identity_fallback(&initial.measurement) && has_whitespace {
                let mut fallback_start = run_start;
                while fallback_start < run_end {
                    while fallback_start < run_end
                        && self.characters[fallback_start].is_whitespace()
                    {
                        fallback_start += 1;
                    }
                    if fallback_start == run_end {
                        break;
                    }
                    let mut fallback_end = fallback_start + 1;
                    while fallback_end < run_end && !self.characters[fallback_end].is_whitespace() {
                        fallback_end += 1;
                    }
                    let Some(fallback) =
                        self.measure_boundary_segment(fallback_start, fallback_end, style)
                    else {
                        return self.fail(
                            KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded,
                        );
                    };
                    pair_delta += fallback.measurement.total_width
                        - positions_width(&fallback.measurement.base_positions);
                    fallback_start = fallback_end;
                }
            } else {
                pair_delta += initial.measurement.total_width
                    - positions_width(&initial.measurement.base_positions);
            }
            run_start = run_end;
        }

        let base_width = self.base_positions[end_index] - self.base_positions[start_index];
        let width = base_width + pair_delta;
        if !width.is_finite() || width < 0.0 {
            return self.fail(KerningParagraphMeasurementFallbackReason::LineWidthInvalid);
        }
        self.boundary_width_cache
            .insert((start_index, end_index), width);
        Some(width)
    }

    /// 기존 base width에 더할 boundary-safe pair delta만 반환한다.
    pub(crate) fn boundary_pair_adjustment(
        &mut self,
        start_index: usize,
        end_index: usize,
    ) -> Option<f64> {
        let width = self.boundary_width(start_index, end_index)?;
        Some(width - (self.base_positions[end_index] - self.base_positions[start_index]))
    }

    /// 같은 positions에서 최초 후보를 고른 뒤 boundary-safe width로 bounded 재탐색한다.
    #[cfg(test)]
    pub(crate) fn find_fitting_end(
        &mut self,
        start_index: usize,
        end_index: usize,
        available_width: f64,
    ) -> Option<KerningLineBoundaryDecision> {
        if start_index >= end_index
            || end_index > self.characters.len()
            || !available_width.is_finite()
            || available_width < 0.0
        {
            self.fail(if !available_width.is_finite() || available_width < 0.0 {
                KerningParagraphMeasurementFallbackReason::LineWidthInvalid
            } else {
                KerningParagraphMeasurementFallbackReason::LineBoundaryRangeInvalid
            });
            return None;
        }

        let mut initial_low = start_index;
        let mut initial_high = end_index;
        while initial_low < initial_high {
            let mid = initial_low + (initial_high - initial_low).div_ceil(2);
            let width = self.range_width(start_index, mid)?;
            if width <= available_width {
                initial_low = mid;
            } else {
                initial_high = mid - 1;
            }
        }
        let initial_end_index = initial_low;

        // 반드시 실제 후보 substring을 한 번 재측정한다. 아래 binary search는
        // cache를 공유하므로 같은 경계를 다시 shape하지 않는다.
        let _ = self.boundary_width(start_index, initial_end_index)?;
        let mut low = start_index;
        let mut high = end_index;
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            let width = self.boundary_width(start_index, mid)?;
            if width <= available_width {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        let (final_end_index, overflow_forced) = if low == start_index {
            (start_index + 1, true)
        } else {
            (low, false)
        };
        let final_width = self.boundary_width(start_index, final_end_index)?;
        Some(KerningLineBoundaryDecision {
            initial_end_index,
            final_end_index,
            final_width,
            overflow_forced,
            attempted_segment_count: self.attempted_segment_count,
        })
    }
}

/// 기존 문자 경계값과 exact source session을 하나의 owned run 측정으로 결합한다.
///
/// 호출자는 기존 measurement가 이미 script scale, 장평, glyph-relative 자간과
/// extra spacing을 적용한 `base_positions`를 넘긴다. 이 함수는 마지막 단계에서만
/// pair design unit을 `effective_font_size_px * width_ratio / units_per_em`으로
/// 환산한다. 따라서 pair delta에는 letter spacing이 다시 곱해지지 않는다.
pub(crate) fn compute_kerning_run_measurement(
    text: &str,
    requested: bool,
    base_positions: Vec<f64>,
    effective_font_size_px: f64,
    width_ratio: f64,
    source_handle: Option<&ExactFontSourceHandle>,
    session: &mut KerningSourceSession<'_>,
) -> KerningRunMeasurement {
    let observed_code_points = text.chars().take(MAX_KERNING_RUN_CODE_POINTS + 1).count();
    let code_point_limit_exceeded = observed_code_points > MAX_KERNING_RUN_CODE_POINTS;
    let code_point_count = observed_code_points.min(MAX_KERNING_RUN_CODE_POINTS);
    let bounded_segment_count = usize::from(code_point_count > 0);
    let source_handle = source_handle.cloned();

    let baseline = |disposition, fallback_reason, session, candidate, base_positions: Vec<f64>| {
        let total_width = positions_width(&base_positions);
        KerningRunMeasurement {
            disposition,
            fallback_reason,
            source_handle: source_handle.clone(),
            code_point_count,
            code_point_limit_exceeded,
            bounded_segment_count,
            advance_deltas: if code_point_limit_exceeded {
                Vec::new()
            } else {
                vec![0.0; code_point_count]
            },
            glyph_position_deltas: Vec::new(),
            total_width,
            base_positions,
            pair_adjusted_positions: None,
            session,
            candidate,
        }
    };

    // K0는 source 조회, parse, shaping, base positions 검증을 전부 건너뛴다.
    if !requested {
        return baseline(
            KerningRunMeasurementDisposition::ExistingPositions,
            None,
            None,
            None,
            base_positions,
        );
    }
    if code_point_limit_exceeded {
        return baseline(
            KerningRunMeasurementDisposition::FailClosed,
            Some(KerningRunMeasurementFallbackReason::RunCodePointLimitExceeded),
            None,
            None,
            base_positions,
        );
    }
    if base_positions.len() != code_point_count.saturating_add(1) {
        return baseline(
            KerningRunMeasurementDisposition::FailClosed,
            Some(KerningRunMeasurementFallbackReason::BasePositionCountMismatch),
            None,
            None,
            base_positions,
        );
    }
    if base_positions.iter().any(|position| !position.is_finite()) {
        return baseline(
            KerningRunMeasurementDisposition::FailClosed,
            Some(KerningRunMeasurementFallbackReason::BasePositionNonFinite),
            None,
            None,
            base_positions,
        );
    }

    let Some(handle) = source_handle.as_ref() else {
        return baseline(
            KerningRunMeasurementDisposition::ExactSourceUnavailable,
            Some(KerningRunMeasurementFallbackReason::ExactSourceUnavailable),
            None,
            None,
            base_positions,
        );
    };
    let session_trace = session.prepare(handle);
    if session_trace.status != KerningSourceSessionStatus::Ready {
        return baseline(
            KerningRunMeasurementDisposition::FailClosed,
            Some(KerningRunMeasurementFallbackReason::SourceSessionFailClosed),
            Some(session_trace),
            None,
            base_positions,
        );
    }
    let Some(engine) = session.engine(handle) else {
        return baseline(
            KerningRunMeasurementDisposition::FailClosed,
            Some(KerningRunMeasurementFallbackReason::PairEngineUnavailable),
            Some(session_trace),
            None,
            base_positions,
        );
    };
    let gate =
        decide_kerning_run_gate(requested, text, code_point_count, &session_trace.capability);
    let candidate = compute_kerning_pair_candidate(text, engine, &gate);
    match candidate.status {
        KerningPairCandidateStatus::NotEligible | KerningPairCandidateStatus::FailClosed => {
            baseline(
                KerningRunMeasurementDisposition::FailClosed,
                Some(KerningRunMeasurementFallbackReason::PairCandidateFailClosed),
                Some(session_trace),
                Some(candidate),
                base_positions,
            )
        }
        KerningPairCandidateStatus::NoAdjustmentCandidate => baseline(
            KerningRunMeasurementDisposition::NoPairAdjustment,
            None,
            Some(session_trace),
            Some(candidate),
            base_positions,
        ),
        KerningPairCandidateStatus::AdjustmentCandidate => apply_pair_candidate(
            base_positions,
            effective_font_size_px,
            width_ratio,
            source_handle,
            code_point_count,
            bounded_segment_count,
            session_trace,
            candidate,
        ),
    }
}

fn apply_pair_candidate(
    base_positions: Vec<f64>,
    effective_font_size_px: f64,
    width_ratio: f64,
    source_handle: Option<ExactFontSourceHandle>,
    code_point_count: usize,
    bounded_segment_count: usize,
    session: KerningSourceSessionTrace,
    candidate: KerningPairCandidateDecision,
) -> KerningRunMeasurement {
    let fail_closed = |reason, candidate, base_positions: Vec<f64>| KerningRunMeasurement {
        disposition: KerningRunMeasurementDisposition::FailClosed,
        fallback_reason: Some(reason),
        source_handle: source_handle.clone(),
        code_point_count,
        code_point_limit_exceeded: false,
        bounded_segment_count,
        advance_deltas: vec![0.0; code_point_count],
        glyph_position_deltas: Vec::new(),
        total_width: positions_width(&base_positions),
        base_positions,
        pair_adjusted_positions: None,
        session: Some(session.clone()),
        candidate: Some(candidate),
    };

    if !effective_font_size_px.is_finite()
        || effective_font_size_px <= 0.0
        || !width_ratio.is_finite()
        || width_ratio <= 0.0
    {
        return fail_closed(
            KerningRunMeasurementFallbackReason::InvalidStyleScale,
            candidate,
            base_positions,
        );
    }
    let Some(units_per_em) = candidate.units_per_em.filter(|units| *units > 0) else {
        return fail_closed(
            KerningRunMeasurementFallbackReason::InvalidUnitsPerEm,
            candidate,
            base_positions,
        );
    };
    let observed_total = candidate
        .position_deltas
        .iter()
        .try_fold(0_i64, |total, delta| total.checked_add(delta.x_advance));
    if observed_total != Some(candidate.total_x_advance_delta) {
        return fail_closed(
            KerningRunMeasurementFallbackReason::CandidateAccountingMismatch,
            candidate,
            base_positions,
        );
    }

    let scale = effective_font_size_px * width_ratio / f64::from(units_per_em);
    if !scale.is_finite() {
        return fail_closed(
            KerningRunMeasurementFallbackReason::InvalidStyleScale,
            candidate,
            base_positions,
        );
    }
    let mut advance_deltas = vec![0.0; code_point_count];
    let mut glyph_position_deltas = Vec::with_capacity(candidate.position_deltas.len());
    for delta in &candidate.position_deltas {
        let Some(advance) = advance_deltas.get_mut(delta.glyph_index) else {
            return fail_closed(
                KerningRunMeasurementFallbackReason::CandidateGlyphIndexOutOfRange,
                candidate,
                base_positions,
            );
        };
        *advance += delta.x_advance as f64 * scale;
        let scaled = KerningGlyphPositionDeltaPx {
            glyph_index: delta.glyph_index,
            glyph_id: delta.glyph_id,
            cluster: delta.cluster,
            x_advance: delta.x_advance as f64 * scale,
            y_advance: delta.y_advance as f64 * scale,
            x_offset: delta.x_offset as f64 * scale,
            y_offset: delta.y_offset as f64 * scale,
        };
        if !advance.is_finite()
            || !scaled.x_advance.is_finite()
            || !scaled.y_advance.is_finite()
            || !scaled.x_offset.is_finite()
            || !scaled.y_offset.is_finite()
        {
            return fail_closed(
                KerningRunMeasurementFallbackReason::AdjustedPositionNonFinite,
                candidate,
                base_positions,
            );
        }
        glyph_position_deltas.push(scaled);
    }

    let mut pair_adjusted_positions = Vec::with_capacity(base_positions.len());
    pair_adjusted_positions.push(base_positions[0]);
    let mut cumulative_delta = 0.0;
    for (index, base) in base_positions.iter().copied().enumerate().skip(1) {
        cumulative_delta += advance_deltas[index - 1];
        let adjusted = base + cumulative_delta;
        if !adjusted.is_finite() {
            return fail_closed(
                KerningRunMeasurementFallbackReason::AdjustedPositionNonFinite,
                candidate,
                base_positions,
            );
        }
        if adjusted < *pair_adjusted_positions.last().expect("initial position") {
            return fail_closed(
                KerningRunMeasurementFallbackReason::AdjustedPositionNonMonotonic,
                candidate,
                base_positions,
            );
        }
        pair_adjusted_positions.push(adjusted);
    }

    KerningRunMeasurement {
        disposition: KerningRunMeasurementDisposition::PairAdjusted,
        fallback_reason: None,
        source_handle,
        code_point_count,
        code_point_limit_exceeded: false,
        bounded_segment_count,
        total_width: positions_width(&pair_adjusted_positions),
        base_positions,
        pair_adjusted_positions: Some(pair_adjusted_positions),
        advance_deltas,
        glyph_position_deltas,
        session: Some(session),
        candidate: Some(candidate),
    }
}

fn positions_width(positions: &[f64]) -> f64 {
    positions
        .first()
        .zip(positions.last())
        .map(|(first, last)| last - first)
        .unwrap_or(0.0)
}

fn capability_for_resolution_failure(
    handle: &ExactFontSourceHandle,
    reason: ExactFontSourceResolutionReason,
) -> KerningCapabilityDecision {
    let fallback_reason = match reason {
        ExactFontSourceResolutionReason::FontByteLimitExceeded => {
            KerningCapabilityFallbackReason::FontByteLimitExceeded
        }
        ExactFontSourceResolutionReason::SourceUnavailable
        | ExactFontSourceResolutionReason::FaceIndexMismatch
        | ExactFontSourceResolutionReason::ByteLengthMismatch
        | ExactFontSourceResolutionReason::Sha256Mismatch => {
            KerningCapabilityFallbackReason::FontSourceUnavailable
        }
    };
    KerningCapabilityDecision::fail_closed(
        fallback_reason,
        handle.font_bytes,
        handle.face_index,
        Some(handle.font_source_sha256.clone()),
    )
}

/// 동일 exact face를 `kern=0`과 `kern=1`로 shaping해 위치 delta 후보만 계산한다.
///
/// nominal glyph·cluster 및 off/on glyph·cluster가 하나라도 달라지면 기존 layout에 안전하게 투영할 수
/// 없으므로 fail-closed한다. 반환된 `AdjustmentCandidate`도 아직 positions에 적용됐다는 뜻은 아니다.
pub(crate) fn compute_kerning_pair_candidate(
    text: &str,
    engine: &KerningPairEngine<'_>,
    gate: &KerningRunGateDecision,
) -> KerningPairCandidateDecision {
    if gate.gate != KerningRunGate::Eligible {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::NotEligible,
            KerningPairCandidateFallbackReason::RunGateNotEligible,
        );
    }

    let observed_code_points = text.chars().take(MAX_KERNING_RUN_CODE_POINTS + 1).count();
    if observed_code_points > MAX_KERNING_RUN_CODE_POINTS
        || observed_code_points != gate.code_point_count
        || observed_code_points != gate.glyph_count
    {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::RunGateInputMismatch,
        );
    }

    if engine.capability.capability != gate.capability
        || engine.capability.font_source_sha256 != gate.font_source_sha256
        || engine.capability.font_bytes != gate.font_bytes
        || engine.capability.face_index != gate.face_index
        || engine.capability.units_per_em != gate.units_per_em
    {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::FontSourceMismatch,
        );
    }

    let nominal: Vec<(u32, u32)> = text
        .char_indices()
        .map(|(cluster, character)| {
            (
                engine
                    .face
                    .glyph_index(character)
                    .map_or(0, |glyph| u32::from(glyph.0)),
                u32::try_from(cluster).expect("bounded run cluster fits in u32"),
            )
        })
        .collect();

    let (direction, off) = shape_with_kerning(&engine.face, text, 0);
    let (_, on) = shape_with_kerning(&engine.face, text, 1);
    if direction != Direction::LeftToRight {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::UnsupportedDirection,
        );
    }
    if off.len() > MAX_KERNING_RUN_GLYPHS || on.len() > MAX_KERNING_RUN_GLYPHS {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::ShapedGlyphLimitExceeded,
        );
    }
    if off.len() != nominal.len()
        || off
            .glyph_infos()
            .iter()
            .zip(&nominal)
            .any(|(glyph, expected)| glyph.glyph_id != expected.0)
    {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::NominalGlyphIdentityChanged,
        );
    }
    if off
        .glyph_infos()
        .iter()
        .zip(&nominal)
        .any(|(glyph, expected)| glyph.cluster != expected.1)
    {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::NominalClusterChanged,
        );
    }
    if off.len() != on.len()
        || off
            .glyph_infos()
            .iter()
            .zip(on.glyph_infos())
            .any(|(left, right)| left.glyph_id != right.glyph_id)
    {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::KerningGlyphIdentityChanged,
        );
    }
    if off
        .glyph_infos()
        .iter()
        .zip(on.glyph_infos())
        .any(|(left, right)| left.cluster != right.cluster)
    {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::KerningClusterChanged,
        );
    }
    if off.len() > MAX_KERNING_TRACE_RECORDS_PER_RUN {
        return KerningPairCandidateDecision::fail_closed(
            gate,
            KerningPairCandidateStatus::FailClosed,
            KerningPairCandidateFallbackReason::TraceRecordLimitExceeded,
        );
    }

    let mut position_deltas = Vec::new();
    let mut total_x_advance_delta = 0_i64;
    for (index, ((glyph, off_position), on_position)) in off
        .glyph_infos()
        .iter()
        .zip(off.glyph_positions())
        .zip(on.glyph_positions())
        .enumerate()
    {
        let delta = KerningGlyphPositionDelta {
            glyph_index: index,
            glyph_id: glyph.glyph_id,
            cluster: glyph.cluster,
            x_advance: i64::from(on_position.x_advance) - i64::from(off_position.x_advance),
            y_advance: i64::from(on_position.y_advance) - i64::from(off_position.y_advance),
            x_offset: i64::from(on_position.x_offset) - i64::from(off_position.x_offset),
            y_offset: i64::from(on_position.y_offset) - i64::from(off_position.y_offset),
        };
        total_x_advance_delta += delta.x_advance;
        if delta.x_advance != 0
            || delta.y_advance != 0
            || delta.x_offset != 0
            || delta.y_offset != 0
        {
            position_deltas.push(delta);
        }
    }

    KerningPairCandidateDecision {
        status: if position_deltas.is_empty() {
            KerningPairCandidateStatus::NoAdjustmentCandidate
        } else {
            KerningPairCandidateStatus::AdjustmentCandidate
        },
        capability: gate.capability,
        font_source_sha256: gate.font_source_sha256.clone(),
        face_index: gate.face_index,
        units_per_em: gate.units_per_em,
        glyph_count: off.len(),
        examined_pair_count: off.len().saturating_sub(1).min(MAX_KERNING_ADJACENT_PAIRS),
        adjusted_position_count: position_deltas.len(),
        total_x_advance_delta,
        position_deltas,
        fallback_reason: None,
    }
}

fn shape_with_kerning(
    face: &rustybuzz::Face<'_>,
    text: &str,
    feature_value: u32,
) -> (Direction, GlyphBuffer) {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    let direction = buffer.direction();
    let feature = Feature::new(Tag::from_bytes(b"kern"), feature_value, ..);
    (direction, shape(face, &[feature], buffer))
}

fn has_gpos_kern_pair_lookup(face: &Face<'_>) -> bool {
    let Some(gpos) = face.tables().gpos else {
        return false;
    };
    let Some(feature) = gpos.features.find(Tag::from_bytes(b"kern")) else {
        return false;
    };
    feature.lookup_indices.into_iter().any(|lookup_index| {
        gpos.lookups.get(lookup_index).is_some_and(|lookup| {
            lookup
                .subtables
                .into_iter::<PositioningSubtable<'_>>()
                .any(|subtable| matches!(subtable, PositioningSubtable::Pair(_)))
        })
    })
}

fn has_legacy_horizontal_format0(face: &Face<'_>) -> bool {
    face.tables().kern.is_some_and(|table| {
        table.subtables.into_iter().any(|subtable| {
            subtable.horizontal
                && !subtable.variable
                && !subtable.has_cross_stream
                && !subtable.has_state_machine
                && matches!(subtable.format, kern::Format::Format0(_))
        })
    })
}
