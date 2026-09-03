//! W10 exact-source shaping 요청의 bounded validation·oracle·terminal attempt 경계.
//!
//! 이 모듈은 검증된 공개 fixture의 glyph oracle을 생성하지만 layout·paint에는 적용하지 않는다. source
//! provenance와 요청 identity가 안전하고 결정적인 경우만 shaping하며, 나머지는 구조화된 reason으로 닫는다.
//! Q2-A terminal attempt와 ledger도 원문·font payload를 직렬화하지 않는다.

use rustybuzz::{Direction, Feature, Language, Script, UnicodeBuffer, Variation};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::str::FromStr;
use std::sync::Arc;
use ttf_parser::{Face, Tag};

pub(crate) const MAX_SHAPING_FONT_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_SHAPING_TEXT_CODE_POINTS: usize = 4_096;
pub(crate) const MAX_SHAPING_GLYPHS: usize = 4_096;
pub(crate) const MAX_SHAPING_FEATURES: usize = 64;
pub(crate) const MAX_SHAPING_VARIATION_AXES: usize = 16;
pub(crate) const MAX_SHAPING_LANGUAGE_BYTES: usize = 35;
pub(crate) const MAX_SHAPING_ATTEMPT_TRACE_RECORDS: usize = 4_096;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShapingExactSource<'a> {
    pub bytes: &'a [u8],
    pub face_index: u32,
    pub portable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShapingDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShapingWritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShapingFeature {
    pub tag: String,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShapingVariation {
    pub tag: String,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct ShapingRequest<'a> {
    pub source: Option<ShapingExactSource<'a>>,
    pub text: &'a str,
    pub direction: ShapingDirection,
    pub writing_mode: ShapingWritingMode,
    pub script: Option<&'a str>,
    pub language: Option<&'a str>,
    pub features: &'a [ShapingFeature],
    pub variations: &'a [ShapingVariation],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ShapingDisposition {
    Requested,
    Applied,
    Unsupported,
    Malformed,
    BoundedLimit,
    NonPortable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ShapingRejectReason {
    SourceUnavailable,
    FontByteLimitExceeded,
    MalformedSfnt,
    NonPortableSource,
    TextCodePointLimitExceeded,
    FeatureLimitExceeded,
    VariationAxisLimitExceeded,
    DirectionWritingModeMismatch,
    MalformedScriptTag,
    MalformedLanguageTag,
    MalformedFeatureTag,
    DuplicateFeatureTag,
    MalformedVariationTag,
    DuplicateVariationAxis,
    VariationValueNonFinite,
    VariationAxisUnsupported,
    VariationValueOutOfRange,
    VerticalMetricsUnavailable,
    GlyphLimitExceeded,
    InvalidHorizontalScale,
    ExactSourceIdentityMismatch,
    ExplicitInstanceSlotMismatch,
    ExplicitInstanceOverrideNotAllowed,
    CacheEntryLimitExceeded,
    ClusterMappingInvalid,
    ShapingUnavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShapingCapabilityDecision {
    pub disposition: ShapingDisposition,
    pub reason: Option<ShapingRejectReason>,
    pub font_source_sha256: Option<String>,
    pub font_bytes: usize,
    pub face_index: u32,
    pub code_point_count: usize,
    pub has_gsub: bool,
    pub has_gpos: bool,
    pub has_vertical_metrics: bool,
    pub has_vorg: bool,
    pub has_variations: bool,
}

impl ShapingCapabilityDecision {
    fn reject(
        disposition: ShapingDisposition,
        reason: ShapingRejectReason,
        font_bytes: usize,
        face_index: u32,
        digest: Option<String>,
    ) -> Self {
        Self {
            disposition,
            reason: Some(reason),
            font_source_sha256: digest,
            font_bytes,
            face_index,
            code_point_count: 0,
            has_gsub: false,
            has_gpos: false,
            has_vertical_metrics: false,
            has_vorg: false,
            has_variations: false,
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut value, "{byte:02x}").expect("String formatting cannot fail");
    }
    value
}

fn valid_tag(tag: &str) -> bool {
    tag.len() == 4 && tag.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
}

fn parse_tag(tag: &str) -> Tag {
    Tag::from_bytes(tag.as_bytes().try_into().expect("validated four-byte tag"))
}

fn valid_language(language: &str) -> bool {
    !language.is_empty()
        && language.len() <= MAX_SHAPING_LANGUAGE_BYTES
        && language.is_ascii()
        && language
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && language
            .split('-')
            .all(|subtag| !subtag.is_empty() && subtag.len() <= 8)
}

pub(crate) fn validate_shaping_request(request: &ShapingRequest<'_>) -> ShapingCapabilityDecision {
    let Some(source) = request.source else {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::Unsupported,
            ShapingRejectReason::SourceUnavailable,
            0,
            0,
            None,
        );
    };
    if source.bytes.len() > MAX_SHAPING_FONT_BYTES {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::BoundedLimit,
            ShapingRejectReason::FontByteLimitExceeded,
            source.bytes.len(),
            source.face_index,
            None,
        );
    }
    if !source.portable {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::NonPortable,
            ShapingRejectReason::NonPortableSource,
            source.bytes.len(),
            source.face_index,
            None,
        );
    }
    let source_digest = digest(source.bytes);
    let Ok(face) = Face::parse(source.bytes, source.face_index) else {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::Malformed,
            ShapingRejectReason::MalformedSfnt,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    };

    validate_shaping_request_with_face(request, source, source_digest, &face)
}

fn validate_shaping_request_with_face(
    request: &ShapingRequest<'_>,
    source: ShapingExactSource<'_>,
    source_digest: String,
    face: &Face<'_>,
) -> ShapingCapabilityDecision {
    let observed_code_points = request
        .text
        .chars()
        .take(MAX_SHAPING_TEXT_CODE_POINTS + 1)
        .count();
    if observed_code_points > MAX_SHAPING_TEXT_CODE_POINTS {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::BoundedLimit,
            ShapingRejectReason::TextCodePointLimitExceeded,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }
    if request.features.len() > MAX_SHAPING_FEATURES {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::BoundedLimit,
            ShapingRejectReason::FeatureLimitExceeded,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }
    if request.variations.len() > MAX_SHAPING_VARIATION_AXES {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::BoundedLimit,
            ShapingRejectReason::VariationAxisLimitExceeded,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }

    let compatible_direction = matches!(
        (request.direction, request.writing_mode),
        (
            ShapingDirection::LeftToRight | ShapingDirection::RightToLeft,
            ShapingWritingMode::HorizontalTb
        ) | (
            ShapingDirection::TopToBottom,
            ShapingWritingMode::VerticalRl | ShapingWritingMode::VerticalLr
        )
    );
    if !compatible_direction {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::Malformed,
            ShapingRejectReason::DirectionWritingModeMismatch,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }
    if request
        .script
        .is_some_and(|tag| !valid_tag(tag) || Script::from_iso15924_tag(parse_tag(tag)).is_none())
    {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::Malformed,
            ShapingRejectReason::MalformedScriptTag,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }
    if request.language.is_some_and(|tag| !valid_language(tag)) {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::Malformed,
            ShapingRejectReason::MalformedLanguageTag,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }

    let mut feature_tags = HashSet::with_capacity(request.features.len());
    for feature in request.features {
        if !valid_tag(&feature.tag) {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Malformed,
                ShapingRejectReason::MalformedFeatureTag,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        }
        if !feature_tags.insert(feature.tag.as_str()) {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Malformed,
                ShapingRejectReason::DuplicateFeatureTag,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        }
    }

    let mut variation_tags = HashSet::with_capacity(request.variations.len());
    for variation in request.variations {
        if !valid_tag(&variation.tag) {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Malformed,
                ShapingRejectReason::MalformedVariationTag,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        }
        if !variation_tags.insert(variation.tag.as_str()) {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Malformed,
                ShapingRejectReason::DuplicateVariationAxis,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        }
        if !variation.value.is_finite() {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Malformed,
                ShapingRejectReason::VariationValueNonFinite,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        }
        let requested_tag = parse_tag(&variation.tag);
        let Some(axis) = face
            .variation_axes()
            .into_iter()
            .find(|axis| axis.tag == requested_tag)
        else {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Unsupported,
                ShapingRejectReason::VariationAxisUnsupported,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        };
        if variation.value < axis.min_value || variation.value > axis.max_value {
            return ShapingCapabilityDecision::reject(
                ShapingDisposition::Malformed,
                ShapingRejectReason::VariationValueOutOfRange,
                source.bytes.len(),
                source.face_index,
                Some(source_digest),
            );
        }
    }

    let tables = face.tables();
    let vertical = tables.vhea.is_some() && tables.vmtx.is_some();
    if !matches!(request.writing_mode, ShapingWritingMode::HorizontalTb) && !vertical {
        return ShapingCapabilityDecision::reject(
            ShapingDisposition::Unsupported,
            ShapingRejectReason::VerticalMetricsUnavailable,
            source.bytes.len(),
            source.face_index,
            Some(source_digest),
        );
    }

    ShapingCapabilityDecision {
        disposition: ShapingDisposition::Requested,
        reason: None,
        font_source_sha256: Some(source_digest),
        font_bytes: source.bytes.len(),
        face_index: source.face_index,
        code_point_count: observed_code_points,
        has_gsub: tables.gsub.is_some(),
        has_gpos: tables.gpos.is_some(),
        has_vertical_metrics: vertical,
        has_vorg: tables.vorg.is_some(),
        has_variations: !face.variation_axes().is_empty(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanonicalShapingFeature {
    pub tag: String,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanonicalShapingVariation {
    pub tag: String,
    pub value_bits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanonicalShapingIdentity {
    pub settings_sha256: String,
    pub font_source_sha256: String,
    pub font_bytes: usize,
    pub face_index: u32,
    pub direction: String,
    pub writing_mode: String,
    pub script: Option<String>,
    pub language: Option<String>,
    pub features: Vec<CanonicalShapingFeature>,
    pub variations: Vec<CanonicalShapingVariation>,
}

fn hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn direction_name(direction: ShapingDirection) -> &'static str {
    match direction {
        ShapingDirection::LeftToRight => "ltr",
        ShapingDirection::RightToLeft => "rtl",
        ShapingDirection::TopToBottom => "ttb",
    }
}

fn writing_mode_name(mode: ShapingWritingMode) -> &'static str {
    match mode {
        ShapingWritingMode::HorizontalTb => "horizontal-tb",
        ShapingWritingMode::VerticalRl => "vertical-rl",
        ShapingWritingMode::VerticalLr => "vertical-lr",
    }
}

pub(crate) fn canonicalize_shaping_request(
    request: &ShapingRequest<'_>,
) -> Result<CanonicalShapingIdentity, ShapingCapabilityDecision> {
    let decision = validate_shaping_request(request);
    if decision.disposition != ShapingDisposition::Requested {
        return Err(decision);
    }

    let source = request
        .source
        .expect("validated shaping request has an exact source");
    let face = Face::parse(source.bytes, source.face_index)
        .expect("validated shaping request has a parseable face");
    Ok(canonicalize_validated_shaping_request(
        request, decision, &face,
    ))
}

pub(crate) fn canonicalize_verified_shaping_request(
    request: &ShapingRequest<'_>,
    face: &Face<'_>,
    source_digest: &str,
) -> Result<CanonicalShapingIdentity, ShapingCapabilityDecision> {
    let source = request
        .source
        .expect("verified shaping request has an exact source");
    let decision =
        validate_shaping_request_with_face(request, source, source_digest.to_owned(), face);
    if decision.disposition != ShapingDisposition::Requested {
        return Err(decision);
    }
    Ok(canonicalize_validated_shaping_request(
        request, decision, face,
    ))
}

fn canonicalize_validated_shaping_request(
    request: &ShapingRequest<'_>,
    decision: ShapingCapabilityDecision,
    face: &Face<'_>,
) -> CanonicalShapingIdentity {
    let mut variations = request
        .variations
        .iter()
        .filter_map(|axis| {
            let requested_tag = parse_tag(&axis.tag);
            let default_value = face
                .variation_axes()
                .into_iter()
                .find(|candidate| candidate.tag == requested_tag)
                .expect("validated variation axis exists")
                .def_value;
            let default_value = if default_value == 0.0 {
                0.0
            } else {
                default_value
            };
            let value = if axis.value == 0.0 { 0.0 } else { axis.value };
            (value.to_bits() != default_value.to_bits()).then(|| CanonicalShapingVariation {
                tag: axis.tag.clone(),
                value_bits: value.to_bits(),
            })
        })
        .collect::<Vec<_>>();
    variations.sort_by(|left, right| left.tag.cmp(&right.tag));
    let features = request
        .features
        .iter()
        .map(|feature| CanonicalShapingFeature {
            tag: feature.tag.clone(),
            value: feature.value,
        })
        .collect::<Vec<_>>();
    let script = request.script.map(|tag| {
        let mut bytes = tag.as_bytes().to_ascii_lowercase();
        bytes[0] = bytes[0].to_ascii_uppercase();
        String::from_utf8(bytes).expect("validated ASCII script")
    });
    let language = request.language.map(str::to_ascii_lowercase);
    let font_source_sha256 = decision
        .font_source_sha256
        .clone()
        .expect("requested source has digest");
    let direction = direction_name(request.direction).to_string();
    let writing_mode = writing_mode_name(request.writing_mode).to_string();

    let mut hasher = Sha256::new();
    hash_field(&mut hasher, font_source_sha256.as_bytes());
    hasher.update((decision.font_bytes as u64).to_be_bytes());
    hasher.update(decision.face_index.to_be_bytes());
    hash_field(&mut hasher, direction.as_bytes());
    hash_field(&mut hasher, writing_mode.as_bytes());
    hash_field(
        &mut hasher,
        script.as_deref().unwrap_or_default().as_bytes(),
    );
    hash_field(
        &mut hasher,
        language.as_deref().unwrap_or_default().as_bytes(),
    );
    hasher.update((features.len() as u64).to_be_bytes());
    for feature in &features {
        hash_field(&mut hasher, feature.tag.as_bytes());
        hasher.update(feature.value.to_be_bytes());
    }
    hasher.update((variations.len() as u64).to_be_bytes());
    for variation in &variations {
        hash_field(&mut hasher, variation.tag.as_bytes());
        hasher.update(variation.value_bits.to_be_bytes());
    }
    let settings_sha256 =
        hasher
            .finalize()
            .iter()
            .fold(String::with_capacity(64), |mut value, byte| {
                write!(&mut value, "{byte:02x}").expect("String formatting cannot fail");
                value
            });

    CanonicalShapingIdentity {
        settings_sha256,
        font_source_sha256,
        font_bytes: decision.font_bytes,
        face_index: decision.face_index,
        direction,
        writing_mode,
        script,
        language,
        features,
        variations,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShapingGlyphRecord {
    pub glyph_id: u32,
    pub cluster_utf8: u32,
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShapingOutputDecision {
    pub disposition: ShapingDisposition,
    pub reason: Option<ShapingRejectReason>,
    pub identity: Option<CanonicalShapingIdentity>,
    pub glyph_count: usize,
    pub glyphs: Vec<ShapingGlyphRecord>,
}

fn rejected_output(decision: ShapingCapabilityDecision) -> ShapingOutputDecision {
    ShapingOutputDecision {
        disposition: decision.disposition,
        reason: decision.reason,
        identity: None,
        glyph_count: 0,
        glyphs: Vec::new(),
    }
}

pub(crate) fn shape_canonical_request_with_face(
    request: &ShapingRequest<'_>,
    identity: CanonicalShapingIdentity,
    face: &mut rustybuzz::Face<'_>,
) -> ShapingOutputDecision {
    let mut variations = face
        .variation_axes()
        .into_iter()
        .map(|axis| Variation {
            tag: axis.tag,
            value: axis.def_value,
        })
        .collect::<Vec<_>>();
    for requested in &identity.variations {
        let tag = parse_tag(&requested.tag);
        let variation = variations
            .iter_mut()
            .find(|variation| variation.tag == tag)
            .expect("canonical variation axis exists on the face");
        variation.value = f32::from_bits(requested.value_bits);
    }
    face.set_variations(&variations);

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(request.text);
    buffer.set_direction(match request.direction {
        ShapingDirection::LeftToRight => Direction::LeftToRight,
        ShapingDirection::RightToLeft => Direction::RightToLeft,
        ShapingDirection::TopToBottom => Direction::TopToBottom,
    });
    if let Some(script) = identity.script.as_deref() {
        buffer.set_script(
            Script::from_iso15924_tag(parse_tag(script)).expect("validated script tag"),
        );
    }
    if let Some(language) = identity.language.as_deref() {
        buffer.set_language(Language::from_str(language).expect("validated language tag"));
    }
    buffer.guess_segment_properties();
    let features = identity
        .features
        .iter()
        .map(|feature| Feature::new(parse_tag(&feature.tag), feature.value, ..))
        .collect::<Vec<_>>();
    let glyph_buffer = rustybuzz::shape(face, &features, buffer);
    if glyph_buffer.len() > MAX_SHAPING_GLYPHS {
        return ShapingOutputDecision {
            disposition: ShapingDisposition::BoundedLimit,
            reason: Some(ShapingRejectReason::GlyphLimitExceeded),
            identity: Some(identity),
            glyph_count: 0,
            glyphs: Vec::new(),
        };
    }
    let glyphs = glyph_buffer
        .glyph_infos()
        .iter()
        .zip(glyph_buffer.glyph_positions())
        .map(|(info, position)| ShapingGlyphRecord {
            glyph_id: info.glyph_id,
            cluster_utf8: info.cluster,
            x_advance: position.x_advance,
            y_advance: position.y_advance,
            x_offset: position.x_offset,
            y_offset: position.y_offset,
        })
        .collect::<Vec<_>>();
    ShapingOutputDecision {
        disposition: ShapingDisposition::Applied,
        reason: None,
        identity: Some(identity),
        glyph_count: glyphs.len(),
        glyphs,
    }
}

pub(crate) fn shape_bounded_request(request: &ShapingRequest<'_>) -> ShapingOutputDecision {
    let identity = match canonicalize_shaping_request(request) {
        Ok(identity) => identity,
        Err(decision) => return rejected_output(decision),
    };
    let source = request.source.expect("canonical request has exact source");
    let Some(mut face) = rustybuzz::Face::from_slice(source.bytes, source.face_index) else {
        return ShapingOutputDecision {
            disposition: ShapingDisposition::Unsupported,
            reason: Some(ShapingRejectReason::ShapingUnavailable),
            identity: Some(identity),
            glyph_count: 0,
            glyphs: Vec::new(),
        };
    };
    shape_canonical_request_with_face(request, identity, &mut face)
}

/// Terminal attempt에는 pre-shaping 상태인 `requested`를 노출하지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum TerminalShapingDisposition {
    Applied,
    Unsupported,
    Malformed,
    BoundedLimit,
    NonPortable,
}

impl TerminalShapingDisposition {
    fn from_output(disposition: ShapingDisposition) -> Option<Self> {
        match disposition {
            ShapingDisposition::Requested => None,
            ShapingDisposition::Applied => Some(Self::Applied),
            ShapingDisposition::Unsupported => Some(Self::Unsupported),
            ShapingDisposition::Malformed => Some(Self::Malformed),
            ShapingDisposition::BoundedLimit => Some(Self::BoundedLimit),
            ShapingDisposition::NonPortable => Some(Self::NonPortable),
        }
    }
}

/// 원문·font payload 없이 한 shaping attempt의 terminal 상태만 남기는 bounded trace다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShapingAttemptTrace {
    pub attempt_id: u32,
    pub disposition: TerminalShapingDisposition,
    pub reason: Option<ShapingRejectReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_source_sha256: Option<String>,
    pub glyph_count: usize,
}

/// Layout·paint가 나중에 같은 결과를 공유할 수 있도록 applied payload를 한 번만 소유한다.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AppliedShapingRun {
    pub identity: CanonicalShapingIdentity,
    pub glyphs: Vec<ShapingGlyphRecord>,
}

#[derive(Debug, Clone)]
pub(crate) struct TerminalShapingAttempt {
    pub trace: ShapingAttemptTrace,
    pub applied: Option<Arc<AppliedShapingRun>>,
}

impl TerminalShapingAttempt {
    #[cfg(test)]
    pub(crate) fn is_applied(&self) -> bool {
        self.trace.disposition == TerminalShapingDisposition::Applied && self.applied.is_some()
    }
}

/// Q1 output을 Q2의 terminal owner로 승격한다. 이 함수도 layout·paint를 바꾸지 않는다.
pub(crate) fn terminal_shaping_attempt(
    attempt_id: u32,
    request: &ShapingRequest<'_>,
) -> TerminalShapingAttempt {
    terminal_shaping_attempt_from_output(attempt_id, shape_bounded_request(request))
}

pub(crate) fn terminal_shaping_attempt_from_output(
    attempt_id: u32,
    output: ShapingOutputDecision,
) -> TerminalShapingAttempt {
    let Some(disposition) = TerminalShapingDisposition::from_output(output.disposition) else {
        return TerminalShapingAttempt {
            trace: ShapingAttemptTrace {
                attempt_id,
                disposition: TerminalShapingDisposition::Unsupported,
                reason: Some(ShapingRejectReason::ShapingUnavailable),
                settings_sha256: None,
                font_source_sha256: None,
                glyph_count: 0,
            },
            applied: None,
        };
    };

    let settings_sha256 = output
        .identity
        .as_ref()
        .map(|identity| identity.settings_sha256.clone());
    let font_source_sha256 = output
        .identity
        .as_ref()
        .map(|identity| identity.font_source_sha256.clone());
    if disposition == TerminalShapingDisposition::Applied {
        let valid_applied = output.reason.is_none()
            && output.identity.is_some()
            && output.glyph_count == output.glyphs.len()
            && output.glyph_count <= MAX_SHAPING_GLYPHS;
        if !valid_applied {
            return TerminalShapingAttempt {
                trace: ShapingAttemptTrace {
                    attempt_id,
                    disposition: TerminalShapingDisposition::Unsupported,
                    reason: Some(ShapingRejectReason::ShapingUnavailable),
                    settings_sha256,
                    font_source_sha256,
                    glyph_count: 0,
                },
                applied: None,
            };
        }
        let identity = output.identity.expect("validated applied identity");
        let glyphs = output.glyphs;
        let glyph_count = glyphs.len();
        return TerminalShapingAttempt {
            trace: ShapingAttemptTrace {
                attempt_id,
                disposition,
                reason: None,
                settings_sha256,
                font_source_sha256,
                glyph_count,
            },
            applied: Some(Arc::new(AppliedShapingRun { identity, glyphs })),
        };
    }

    TerminalShapingAttempt {
        trace: ShapingAttemptTrace {
            attempt_id,
            disposition,
            reason: output.reason,
            settings_sha256,
            font_source_sha256,
            glyph_count: 0,
        },
        applied: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ShapingAttemptLedgerStatus {
    Complete,
    Truncated,
}

/// 페이지 공개 trace 후보를 고정 상한으로만 수집한다. applied glyph payload는 ledger에 넣지 않는다.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundedShapingAttemptLedger {
    status: ShapingAttemptLedgerStatus,
    record_limit: usize,
    records: Vec<ShapingAttemptTrace>,
    omitted_records: usize,
}

impl Default for BoundedShapingAttemptLedger {
    fn default() -> Self {
        Self {
            status: ShapingAttemptLedgerStatus::Complete,
            record_limit: MAX_SHAPING_ATTEMPT_TRACE_RECORDS,
            records: Vec::new(),
            omitted_records: 0,
        }
    }
}

impl BoundedShapingAttemptLedger {
    #[cfg(test)]
    pub(crate) fn record(&mut self, trace: &ShapingAttemptTrace) {
        if self.records.len() < self.record_limit {
            self.records.push(trace.clone());
        } else {
            self.status = ShapingAttemptLedgerStatus::Truncated;
            self.omitted_records = self.omitted_records.saturating_add(1);
        }
    }

    #[cfg(test)]
    pub(crate) fn record_count(&self) -> usize {
        self.records.len()
    }

    #[cfg(test)]
    pub(crate) fn status(&self) -> ShapingAttemptLedgerStatus {
        self.status
    }

    #[cfg(test)]
    pub(crate) fn record_limit(&self) -> usize {
        self.record_limit
    }

    #[cfg(test)]
    pub(crate) fn omitted_record_count(&self) -> usize {
        self.omitted_records
    }
}
