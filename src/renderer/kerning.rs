//! Kerning request가 실제 pair positioning에 들어가기 전의 exact-font decision plane.
//!
//! 이 모듈은 family 이름이나 fallback 후보를 다시 찾지 않는다. 상위 font selection이 확정한
//! face bytes와 face index만 입력받고, 지원 여부와 bounded pair delta 후보를 계산한다. 실제
//! layout 적용은 후속 단계의 책임이다.

use rustybuzz::{shape, Direction, Feature, GlyphBuffer, UnicodeBuffer};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use ttf_parser::{gpos::PositioningSubtable, kern, Face, Tag};

/// Q1에서 portable font blob 경계와 맞춰 동결한 exact source 상한.
pub(crate) const MAX_KERNING_FONT_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_KERNING_RUN_CODE_POINTS: usize = 4_096;
pub(crate) const MAX_KERNING_RUN_GLYPHS: usize = 4_096;
pub(crate) const MAX_KERNING_ADJACENT_PAIRS: usize = 4_095;
pub(crate) const MAX_KERNING_TRACE_RECORDS_PER_RUN: usize = 4_096;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExactFontSource<'a> {
    pub bytes: &'a [u8],
    pub face_index: u32,
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

/// 선택이 끝난 exact face source의 kerning capability를 bounded하게 판정한다.
///
/// `None`은 시스템 font 이름만 있거나 fallback 결과의 bytes를 증명할 수 없는 경우다. source가
/// 없거나 손상됐거나 상한을 넘으면 추측하지 않고 `Unsupported`로 닫는다.
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

    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(source.bytes) {
        write!(&mut digest, "{byte:02x}").expect("String formatting cannot fail");
    }
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
    let face = rustybuzz::Face::from_slice(source.bytes, source.face_index)
        .ok_or(KerningPairCandidateFallbackReason::ShapingUnavailable)?;
    Ok(KerningPairEngine { face, capability })
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
