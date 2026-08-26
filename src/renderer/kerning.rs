//! Kerning request가 실제 pair positioning에 들어가기 전의 exact-font capability 경계.
//!
//! 이 모듈은 family 이름이나 fallback 후보를 다시 찾지 않는다. 상위 font selection이 확정한
//! face bytes와 face index만 입력받고, 지원 여부를 기능 탐지한다. 실제 pair delta 계산과
//! layout 적용은 후속 단계의 책임이다.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use ttf_parser::{gpos::PositioningSubtable, kern, Face, Tag};

/// Q1에서 portable font blob 경계와 맞춰 동결한 exact source 상한.
pub(crate) const MAX_KERNING_FONT_BYTES: usize = 32 * 1024 * 1024;

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
