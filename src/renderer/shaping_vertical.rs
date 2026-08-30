//! W10-Q4-C dormant vertical shaping geometry transaction.
//!
//! This module joins the exact Q4-A shaping output and Q4-B typed source intent,
//! Q4-D1 registers this module and adds an exact-source-bound dormant context.
//! Product layout, sidecar, paint, and backend callers remain closed until their
//! separately approved Q4-D2~D4 slices.

use super::kerning::{
    ExactFontPortableResourceIdentity, ExactFontSlot, ExactFontSourceHandle,
    ExactFontSourceRegistry, ExactFontSourceResolutionReason,
};
use super::shaping::{
    canonicalize_verified_shaping_request, shape_canonical_request_with_face,
    terminal_shaping_attempt, terminal_shaping_attempt_from_output, AppliedShapingRun,
    ShapingAttemptTrace, ShapingDirection, ShapingExactSource, ShapingFeature, ShapingRejectReason,
    ShapingRequest, ShapingVariation, ShapingWritingMode, TerminalShapingAttempt,
    TerminalShapingDisposition,
};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;
use ttf_parser::{Face, GlyphId};

pub(crate) const MAX_VERTICAL_SHAPING_PAGE_SIDECARS: usize = 4_096;
pub(crate) const MAX_VERTICAL_SHAPING_PREPARED_SOURCES_PER_PAGE: usize = 64;
pub(crate) const MAX_VERTICAL_SHAPING_FONT_BYTES_PER_PAGE: usize = 64 * 1024 * 1024;
pub(crate) const NOTO_SANS_KR_REGULAR_SHA256: &str =
    "6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalIntentSurface {
    Hwp5TableCell,
    Hwp5TextBox,
    HwpxTableCell,
    HwpxTextBox,
    HwpxSection,
    HwpxMasterPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalLatinOrientation {
    NotApplicable,
    Sideways,
    Upright,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TypedVerticalIntent {
    writing_mode: ShapingWritingMode,
    latin_orientation: VerticalLatinOrientation,
}

impl TypedVerticalIntent {
    pub(crate) fn horizontal() -> Self {
        Self {
            writing_mode: ShapingWritingMode::HorizontalTb,
            latin_orientation: VerticalLatinOrientation::NotApplicable,
        }
    }

    pub(crate) fn vertical_rl(latin_orientation: VerticalLatinOrientation) -> Self {
        Self {
            writing_mode: ShapingWritingMode::VerticalRl,
            latin_orientation,
        }
    }

    pub(crate) fn vertical_lr(latin_orientation: VerticalLatinOrientation) -> Self {
        Self {
            writing_mode: ShapingWritingMode::VerticalLr,
            latin_orientation,
        }
    }

    pub(crate) fn writing_mode(self) -> ShapingWritingMode {
        self.writing_mode
    }

    pub(crate) fn latin_orientation(self) -> VerticalLatinOrientation {
        self.latin_orientation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalIntentDisposition {
    Supported(TypedVerticalIntent),
    UnsupportedRaw,
}

pub(crate) fn adapt_hwp5_vertical_intent(
    surface: VerticalIntentSurface,
    raw: u8,
) -> VerticalIntentDisposition {
    if !matches!(
        surface,
        VerticalIntentSurface::Hwp5TableCell | VerticalIntentSurface::Hwp5TextBox
    ) {
        return VerticalIntentDisposition::UnsupportedRaw;
    }
    match raw {
        0 => VerticalIntentDisposition::Supported(TypedVerticalIntent::horizontal()),
        1 => VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
            VerticalLatinOrientation::Sideways,
        )),
        2 => VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
            VerticalLatinOrientation::Upright,
        )),
        _ => VerticalIntentDisposition::UnsupportedRaw,
    }
}

pub(crate) fn adapt_hwpx_vertical_intent(
    surface: VerticalIntentSurface,
    raw: &str,
) -> VerticalIntentDisposition {
    if !matches!(
        surface,
        VerticalIntentSurface::HwpxTableCell
            | VerticalIntentSurface::HwpxTextBox
            | VerticalIntentSurface::HwpxSection
            | VerticalIntentSurface::HwpxMasterPage
    ) {
        return VerticalIntentDisposition::UnsupportedRaw;
    }
    match raw {
        "HORIZONTAL" => VerticalIntentDisposition::Supported(TypedVerticalIntent::horizontal()),
        "VERTICAL" => VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
            VerticalLatinOrientation::Sideways,
        )),
        "VERTICALALL" => VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
            VerticalLatinOrientation::Upright,
        )),
        _ => VerticalIntentDisposition::UnsupportedRaw,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalRunClass {
    CjkUpright,
    LatinSideways,
    LatinUpright,
    CjkPunctuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalGlyphTransform {
    Upright,
    RotateClockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalPoint {
    pub x: f64,
    pub y: f64,
}

impl VerticalPoint {
    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl VerticalRect {
    fn is_well_formed(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.width >= 0.0
            && self.height.is_finite()
            && self.height >= 0.0
    }

    fn union(self, other: Self) -> Self {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalLegacyGeometry {
    pub bbox: VerticalRect,
    pub next_inline_origin: VerticalPoint,
    pub next_column_origin: VerticalPoint,
}

impl VerticalLegacyGeometry {
    fn is_well_formed(self) -> bool {
        self.bbox.is_well_formed()
            && self.next_inline_origin.is_finite()
            && self.next_column_origin.is_finite()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VerticalGlyphGeometry {
    pub glyph_id: u32,
    pub cluster_utf8_range: Range<usize>,
    pub origin: VerticalPoint,
    pub bbox: VerticalRect,
    pub transform: VerticalGlyphTransform,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VerticalShapingGeometry {
    pub run_class: VerticalRunClass,
    pub writing_mode: ShapingWritingMode,
    pub glyphs: Vec<VerticalGlyphGeometry>,
    pub bbox: VerticalRect,
    pub inline_advance_px: f64,
    pub next_inline_origin: VerticalPoint,
    pub next_column_origin: VerticalPoint,
}

#[derive(Debug, Clone)]
pub(crate) struct DormantVerticalShapingTransaction {
    trace: ShapingAttemptTrace,
    applied: Arc<AppliedShapingRun>,
    geometry: Arc<VerticalShapingGeometry>,
    fallback_geometry: VerticalLegacyGeometry,
}

impl DormantVerticalShapingTransaction {
    pub(crate) fn trace(&self) -> &ShapingAttemptTrace {
        &self.trace
    }

    pub(crate) fn applied(&self) -> &Arc<AppliedShapingRun> {
        &self.applied
    }

    pub(crate) fn line_geometry(&self) -> &Arc<VerticalShapingGeometry> {
        &self.geometry
    }

    pub(crate) fn bbox_geometry(&self) -> &Arc<VerticalShapingGeometry> {
        &self.geometry
    }

    pub(crate) fn next_origin_geometry(&self) -> &Arc<VerticalShapingGeometry> {
        &self.geometry
    }

    pub(crate) fn fallback_geometry(&self) -> VerticalLegacyGeometry {
        self.fallback_geometry
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DormantVerticalShapingRequest<'a> {
    pub attempt_id: u32,
    pub shaping: ShapingRequest<'a>,
    pub intent: TypedVerticalIntent,
    pub font_size_px: f64,
    pub origin: VerticalPoint,
    pub column_pitch_px: f64,
    pub fallback_geometry: VerticalLegacyGeometry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DormantVerticalShapingRejectReason {
    LegacyGeometryMalformed,
    HorizontalIntentUnsupported,
    DirectionIntentMismatch,
    FontSizeInvalid,
    OriginInvalid,
    ColumnPitchInvalid,
    EmptyRun,
    CharacterClassUnsupported,
    MixedRunUnsupported,
    VariationGeometryUnsupported,
    ShapingRejected(ShapingRejectReason),
    AppliedPayloadMissing,
    FontUnitsPerEmInvalid,
    GlyphIdOutOfRange,
    MissingGlyph,
    GlyphBoundsUnavailable,
    ClusterMappingInvalid,
    VerticalAdvanceInvalid,
    GeometryMalformed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DormantVerticalShapingRejection {
    reason: DormantVerticalShapingRejectReason,
    fallback_geometry: VerticalLegacyGeometry,
}

impl DormantVerticalShapingRejection {
    pub(crate) fn reason(&self) -> DormantVerticalShapingRejectReason {
        self.reason
    }

    pub(crate) fn fallback_geometry(&self) -> VerticalLegacyGeometry {
        self.fallback_geometry
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

fn reject(
    reason: DormantVerticalShapingRejectReason,
    fallback_geometry: VerticalLegacyGeometry,
) -> DormantVerticalShapingRejection {
    DormantVerticalShapingRejection {
        reason,
        fallback_geometry,
    }
}

/// Immutable exact-source snapshot for the first bounded vertical owner.
///
/// D1 deliberately exposes no layout or publication method. A caller can only
/// prepare a certified dormant transaction whose product flag remains false.
pub(crate) struct VerticalShapingContext {
    registry: ExactFontSourceRegistry,
}

impl std::fmt::Debug for VerticalShapingContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerticalShapingContext")
            .field("registry_generation", &self.registry.generation())
            .field("slot_count", &self.registry.slot_count())
            .field("source_count", &self.registry.source_count())
            .field("total_source_bytes", &self.registry.total_source_bytes())
            .finish()
    }
}

impl VerticalShapingContext {
    pub(crate) fn new(registry: ExactFontSourceRegistry) -> Self {
        Self { registry }
    }

    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry.generation()
    }

    pub(crate) fn slot_count(&self) -> usize {
        self.registry.slot_count()
    }

    pub(crate) fn source_count(&self) -> usize {
        self.registry.source_count()
    }

    pub(crate) fn prepare_dormant(
        &self,
        request: VerticalShapingContextRequest<'_>,
    ) -> Result<CertifiedDormantVerticalShapingTransaction, VerticalShapingContextRejection> {
        let fallback = request.fallback_geometry;
        if !fallback.is_well_formed() {
            return Err(context_reject(
                VerticalShapingContextRejectReason::Dormant(
                    DormantVerticalShapingRejectReason::LegacyGeometryMalformed,
                ),
                fallback,
            ));
        }
        let handle = self
            .registry
            .handle_for_slot(request.slot)
            .cloned()
            .ok_or_else(|| {
                context_reject(
                    VerticalShapingContextRejectReason::SourceUnavailable,
                    fallback,
                )
            })?;
        let source_bytes = self
            .registry
            .resolve_owned_source_arc(&handle)
            .map_err(|reason| context_source_rejection(reason, fallback))?;
        let portable_resource = self
            .registry
            .portable_resource_identity_for_handle(&handle)
            .ok_or_else(|| {
                context_reject(
                    VerticalShapingContextRejectReason::SourceUnavailable,
                    fallback,
                )
            })?;
        let face = Face::parse(source_bytes.as_ref(), handle.face_index).map_err(|_| {
            context_reject(
                VerticalShapingContextRejectReason::CertificateFaceInvalid,
                fallback,
            )
        })?;
        let units_per_em = face.units_per_em();
        if units_per_em == 0 {
            return Err(context_reject(
                VerticalShapingContextRejectReason::Dormant(
                    DormantVerticalShapingRejectReason::FontUnitsPerEmInvalid,
                ),
                fallback,
            ));
        }
        let shaping = ShapingRequest {
            source: Some(ShapingExactSource {
                bytes: source_bytes.as_ref(),
                face_index: handle.face_index,
                portable: true,
            }),
            text: request.text,
            direction: ShapingDirection::TopToBottom,
            writing_mode: request.intent.writing_mode(),
            script: request.script,
            language: request.language,
            features: request.features,
            variations: request.variations,
        };
        let transaction = prepare_verified_dormant_vertical_shaping_transaction(
            DormantVerticalShapingRequest {
                attempt_id: request.attempt_id,
                shaping,
                intent: request.intent,
                font_size_px: request.font_size_px,
                origin: request.origin,
                column_pitch_px: request.column_pitch_px,
                fallback_geometry: fallback,
            },
            &face,
            &handle.font_source_sha256,
        )
        .map_err(|rejection| {
            context_reject(
                VerticalShapingContextRejectReason::Dormant(rejection.reason()),
                rejection.fallback_geometry(),
            )
        })?;
        let portable_font =
            VerticalShapingPortableFontMetadata::from_face(portable_resource, &face);
        Ok(CertifiedDormantVerticalShapingTransaction {
            transaction,
            certificate: Arc::new(VerticalShapingSourceCertificate {
                registry_generation: self.registry.generation(),
                slot: request.slot,
                handle,
                source_bytes,
                units_per_em,
                portable_font,
            }),
        })
    }
}

pub(crate) struct VerticalShapingContextRequest<'a> {
    pub attempt_id: u32,
    pub slot: ExactFontSlot,
    pub text: &'a str,
    pub intent: TypedVerticalIntent,
    pub font_size_px: f64,
    pub origin: VerticalPoint,
    pub column_pitch_px: f64,
    pub fallback_geometry: VerticalLegacyGeometry,
    pub script: Option<&'a str>,
    pub language: Option<&'a str>,
    pub features: &'a [ShapingFeature],
    pub variations: &'a [ShapingVariation],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalShapingContextRejectReason {
    SourceUnavailable,
    SourceIdentityMismatch,
    CertificateFaceInvalid,
    Dormant(DormantVerticalShapingRejectReason),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalShapingContextRejection {
    reason: VerticalShapingContextRejectReason,
    fallback_geometry: VerticalLegacyGeometry,
}

impl VerticalShapingContextRejection {
    pub(crate) fn reason(&self) -> VerticalShapingContextRejectReason {
        self.reason
    }

    pub(crate) fn fallback_geometry(&self) -> VerticalLegacyGeometry {
        self.fallback_geometry
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

fn context_reject(
    reason: VerticalShapingContextRejectReason,
    fallback_geometry: VerticalLegacyGeometry,
) -> VerticalShapingContextRejection {
    VerticalShapingContextRejection {
        reason,
        fallback_geometry,
    }
}

fn context_source_rejection(
    reason: ExactFontSourceResolutionReason,
    fallback_geometry: VerticalLegacyGeometry,
) -> VerticalShapingContextRejection {
    let reason = match reason {
        ExactFontSourceResolutionReason::SourceUnavailable => {
            VerticalShapingContextRejectReason::SourceUnavailable
        }
        ExactFontSourceResolutionReason::FontByteLimitExceeded
        | ExactFontSourceResolutionReason::FaceIndexMismatch
        | ExactFontSourceResolutionReason::ByteLengthMismatch
        | ExactFontSourceResolutionReason::Sha256Mismatch => {
            VerticalShapingContextRejectReason::SourceIdentityMismatch
        }
    };
    context_reject(reason, fallback_geometry)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerticalShapingPortableFontMetadata {
    resource_digest_blake3: String,
    resource_hash_fnv1a64: u64,
    resource_fingerprint: [u8; 16],
    number_of_glyphs: u16,
    weight_class: u16,
    width_class: u16,
    italic: bool,
}

impl VerticalShapingPortableFontMetadata {
    fn from_face(resource: ExactFontPortableResourceIdentity, face: &Face<'_>) -> Self {
        Self {
            resource_digest_blake3: resource.digest_blake3().to_string(),
            resource_hash_fnv1a64: resource.hash_fnv1a64(),
            resource_fingerprint: resource.fingerprint(),
            number_of_glyphs: face.number_of_glyphs(),
            weight_class: face.weight().to_number(),
            width_class: face.width().to_number(),
            italic: face.is_italic(),
        }
    }

    pub(crate) fn resource_digest_blake3(&self) -> &str {
        &self.resource_digest_blake3
    }

    pub(crate) fn resource_hash_fnv1a64(&self) -> u64 {
        self.resource_hash_fnv1a64
    }

    pub(crate) fn resource_fingerprint(&self) -> [u8; 16] {
        self.resource_fingerprint
    }

    pub(crate) fn number_of_glyphs(&self) -> u16 {
        self.number_of_glyphs
    }

    pub(crate) fn weight_class(&self) -> u16 {
        self.weight_class
    }

    pub(crate) fn width_class(&self) -> u16 {
        self.width_class
    }

    pub(crate) fn italic(&self) -> bool {
        self.italic
    }
}

#[derive(Clone)]
pub(crate) struct VerticalShapingSourceCertificate {
    registry_generation: u64,
    slot: ExactFontSlot,
    handle: ExactFontSourceHandle,
    source_bytes: Arc<[u8]>,
    units_per_em: u16,
    portable_font: VerticalShapingPortableFontMetadata,
}

impl std::fmt::Debug for VerticalShapingSourceCertificate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerticalShapingSourceCertificate")
            .field("registry_generation", &self.registry_generation)
            .field("slot", &self.slot)
            .field("source_handle", &self.handle)
            .field("source_bytes_len", &self.source_bytes.len())
            .field("units_per_em", &self.units_per_em)
            .field("portable_font", &self.portable_font)
            .finish()
    }
}

impl VerticalShapingSourceCertificate {
    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub(crate) fn slot(&self) -> ExactFontSlot {
        self.slot
    }

    pub(crate) fn font_source_sha256(&self) -> &str {
        &self.handle.font_source_sha256
    }

    pub(crate) fn font_bytes(&self) -> usize {
        self.handle.font_bytes
    }

    pub(crate) fn face_index(&self) -> u32 {
        self.handle.face_index
    }

    pub(crate) fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    pub(crate) fn source_bytes_arc(&self) -> &Arc<[u8]> {
        &self.source_bytes
    }

    pub(crate) fn portable_font(&self) -> &VerticalShapingPortableFontMetadata {
        &self.portable_font
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CertifiedDormantVerticalShapingTransaction {
    transaction: DormantVerticalShapingTransaction,
    certificate: Arc<VerticalShapingSourceCertificate>,
}

impl CertifiedDormantVerticalShapingTransaction {
    pub(crate) fn transaction(&self) -> &DormantVerticalShapingTransaction {
        &self.transaction
    }

    pub(crate) fn certificate(&self) -> &Arc<VerticalShapingSourceCertificate> {
        &self.certificate
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

/// Q4-D2 page-local owner for one bounded HWP5 vertical table-cell source run.
///
/// Paint publication remains closed: D2 stores the certified Q4-C owner beside
/// the committed fallback nodes, while Q4-D3 will be the first consumer allowed
/// to lower it to a glyph run.
#[derive(Clone)]
pub(crate) struct BoundedVerticalHwp5TableCellSidecar {
    line_node_id: u32,
    transaction: Arc<CertifiedDormantVerticalShapingTransaction>,
    source_text_sha256: [u8; 32],
    source_utf8_bytes: usize,
    source_utf16_units: usize,
}

impl std::fmt::Debug for BoundedVerticalHwp5TableCellSidecar {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundedVerticalHwp5TableCellSidecar")
            .field("line_node_id", &self.line_node_id)
            .field("source_utf8_bytes", &self.source_utf8_bytes)
            .field("source_utf16_units", &self.source_utf16_units)
            .field("transaction", &self.transaction)
            .finish()
    }
}

impl BoundedVerticalHwp5TableCellSidecar {
    pub(crate) fn new(
        line_node_id: u32,
        transaction: Arc<CertifiedDormantVerticalShapingTransaction>,
        source_text: &str,
    ) -> Self {
        let source_text_sha256 = Sha256::digest(source_text.as_bytes()).into();
        Self {
            line_node_id,
            transaction,
            source_text_sha256,
            source_utf8_bytes: source_text.len(),
            source_utf16_units: source_text.encode_utf16().count(),
        }
    }

    pub(crate) fn line_node_id(&self) -> u32 {
        self.line_node_id
    }

    pub(crate) fn transaction(&self) -> &Arc<CertifiedDormantVerticalShapingTransaction> {
        &self.transaction
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VerticalGlyphPublicationLeafInput<'a> {
    pub source_node_id: u32,
    pub text_source_id: u32,
    pub text: &'a str,
    pub is_vertical: bool,
    pub bbox: VerticalRect,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VerticalGlyphPublicationLeafShadow {
    source_node_id: u32,
    text_source_id: u32,
    source_utf8_range: Range<usize>,
    source_utf16_range: Range<usize>,
    glyph_index: usize,
    glyph_id: u32,
    origin: VerticalPoint,
    bbox: VerticalRect,
    advance: VerticalPoint,
}

impl VerticalGlyphPublicationLeafShadow {
    pub(crate) fn source_node_id(&self) -> u32 {
        self.source_node_id
    }

    pub(crate) fn text_source_id(&self) -> u32 {
        self.text_source_id
    }

    pub(crate) fn source_utf8_range(&self) -> Range<usize> {
        self.source_utf8_range.clone()
    }

    pub(crate) fn source_utf16_range(&self) -> Range<usize> {
        self.source_utf16_range.clone()
    }

    pub(crate) fn glyph_index(&self) -> usize {
        self.glyph_index
    }

    pub(crate) fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    pub(crate) fn origin(&self) -> VerticalPoint {
        self.origin
    }

    pub(crate) fn bbox(&self) -> VerticalRect {
        self.bbox
    }

    pub(crate) fn advance(&self) -> VerticalPoint {
        self.advance
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VerticalGlyphPublicationShadow {
    line_node_id: u32,
    leaves: Vec<VerticalGlyphPublicationLeafShadow>,
    registry_generation: u64,
    font_source_sha256: String,
    font_bytes: usize,
    face_index: u32,
}

impl VerticalGlyphPublicationShadow {
    pub(crate) fn line_node_id(&self) -> u32 {
        self.line_node_id
    }

    pub(crate) fn leaves(&self) -> &[VerticalGlyphPublicationLeafShadow] {
        &self.leaves
    }

    pub(crate) fn registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub(crate) fn font_source_sha256(&self) -> &str {
        &self.font_source_sha256
    }

    pub(crate) fn font_bytes(&self) -> usize {
        self.font_bytes
    }

    pub(crate) fn face_index(&self) -> u32 {
        self.face_index
    }

    pub(crate) fn product_published(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalGlyphPublicationShadowRejectReason {
    OwnerIdentityMismatch,
    SourceIdentityMismatch,
    LeafCountMismatch,
    NodeSequenceMismatch,
    TextSourceSequenceMismatch,
    UnsupportedFallbackLeaf,
    ClusterTextMismatch,
    GeometryMismatch,
}

fn same_vertical_scalar(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1.0e-9 * left.abs().max(right.abs()).max(1.0)
}

fn same_vertical_rect(left: VerticalRect, right: VerticalRect) -> bool {
    same_vertical_scalar(left.x, right.x)
        && same_vertical_scalar(left.y, right.y)
        && same_vertical_scalar(left.width, right.width)
        && same_vertical_scalar(left.height, right.height)
}

/// Q4-D3-A read-only mapping from one certified D2 line to its leaf-scoped
/// text sources. The returned DTO owns no raw text or font bytes and cannot
/// mutate a render tree, layer tree, or resource arena.
pub(crate) fn prepare_bounded_vertical_glyph_publication_shadow(
    sidecar: &Arc<BoundedVerticalHwp5TableCellSidecar>,
    leaves: &[VerticalGlyphPublicationLeafInput<'_>],
) -> Result<VerticalGlyphPublicationShadow, VerticalGlyphPublicationShadowRejectReason> {
    let certified = sidecar.transaction();
    let transaction = certified.transaction();
    let geometry = transaction.line_geometry();
    if certified.product_published()
        || transaction.product_published()
        || !Arc::ptr_eq(geometry, transaction.bbox_geometry())
        || !Arc::ptr_eq(geometry, transaction.next_origin_geometry())
    {
        return Err(VerticalGlyphPublicationShadowRejectReason::OwnerIdentityMismatch);
    }
    let certificate = certified.certificate();
    let identity = &transaction.applied().identity;
    if certificate.font_source_sha256() != NOTO_SANS_KR_REGULAR_SHA256
        || certificate.font_source_sha256() != identity.font_source_sha256
        || certificate.font_bytes() != certificate.source_bytes_arc().len()
        || certificate.font_bytes() != identity.font_bytes
        || certificate.face_index() != identity.face_index
    {
        return Err(VerticalGlyphPublicationShadowRejectReason::SourceIdentityMismatch);
    }
    if leaves.is_empty() || leaves.len() != geometry.glyphs.len() {
        return Err(VerticalGlyphPublicationShadowRejectReason::LeafCountMismatch);
    }

    let mut text_hasher = Sha256::new();
    let mut cumulative_utf8 = 0usize;
    let mut cumulative_utf16 = 0usize;
    let first_text_source_id = leaves[0].text_source_id;
    let mut mapped = Vec::with_capacity(leaves.len());
    for (index, (leaf, glyph)) in leaves.iter().zip(&geometry.glyphs).enumerate() {
        let expected_node_id = sidecar
            .line_node_id()
            .checked_add(u32::try_from(index).unwrap_or(u32::MAX))
            .and_then(|value| value.checked_add(1))
            .ok_or(VerticalGlyphPublicationShadowRejectReason::NodeSequenceMismatch)?;
        if leaf.source_node_id != expected_node_id {
            return Err(VerticalGlyphPublicationShadowRejectReason::NodeSequenceMismatch);
        }
        let expected_text_source_id = first_text_source_id
            .checked_add(u32::try_from(index).unwrap_or(u32::MAX))
            .ok_or(VerticalGlyphPublicationShadowRejectReason::TextSourceSequenceMismatch)?;
        if leaf.text_source_id != expected_text_source_id {
            return Err(VerticalGlyphPublicationShadowRejectReason::TextSourceSequenceMismatch);
        }
        if !leaf.is_vertical || leaf.text.chars().count() != 1 || !leaf.bbox.is_well_formed() {
            return Err(VerticalGlyphPublicationShadowRejectReason::UnsupportedFallbackLeaf);
        }
        let next_utf8 = cumulative_utf8
            .checked_add(leaf.text.len())
            .ok_or(VerticalGlyphPublicationShadowRejectReason::ClusterTextMismatch)?;
        let leaf_utf16 = leaf.text.encode_utf16().count();
        let next_utf16 = cumulative_utf16
            .checked_add(leaf_utf16)
            .ok_or(VerticalGlyphPublicationShadowRejectReason::ClusterTextMismatch)?;
        if glyph.cluster_utf8_range != (cumulative_utf8..next_utf8) {
            return Err(VerticalGlyphPublicationShadowRejectReason::ClusterTextMismatch);
        }
        if glyph.glyph_id == 0
            || !matches!(glyph.transform, VerticalGlyphTransform::Upright)
            || !same_vertical_rect(leaf.bbox, glyph.bbox)
        {
            return Err(VerticalGlyphPublicationShadowRejectReason::GeometryMismatch);
        }
        let next_origin = geometry
            .glyphs
            .get(index + 1)
            .map(|next| next.origin)
            .unwrap_or(geometry.next_inline_origin);
        let advance = VerticalPoint {
            x: next_origin.x - glyph.origin.x,
            y: next_origin.y - glyph.origin.y,
        };
        if !glyph.origin.is_finite() || !advance.is_finite() {
            return Err(VerticalGlyphPublicationShadowRejectReason::GeometryMismatch);
        }
        text_hasher.update(leaf.text.as_bytes());
        mapped.push(VerticalGlyphPublicationLeafShadow {
            source_node_id: leaf.source_node_id,
            text_source_id: leaf.text_source_id,
            source_utf8_range: 0..leaf.text.len(),
            source_utf16_range: 0..leaf_utf16,
            glyph_index: index,
            glyph_id: glyph.glyph_id,
            origin: glyph.origin,
            bbox: glyph.bbox,
            advance,
        });
        cumulative_utf8 = next_utf8;
        cumulative_utf16 = next_utf16;
    }
    let text_sha256: [u8; 32] = text_hasher.finalize().into();
    if cumulative_utf8 != sidecar.source_utf8_bytes
        || cumulative_utf16 != sidecar.source_utf16_units
        || text_sha256 != sidecar.source_text_sha256
    {
        return Err(VerticalGlyphPublicationShadowRejectReason::ClusterTextMismatch);
    }

    Ok(VerticalGlyphPublicationShadow {
        line_node_id: sidecar.line_node_id(),
        leaves: mapped,
        registry_generation: certificate.registry_generation(),
        font_source_sha256: certificate.font_source_sha256().to_string(),
        font_bytes: certificate.font_bytes(),
        face_index: certificate.face_index(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalShapingSidecarRejectReason {
    ZeroNode,
    OwnerIdentityMismatch,
    SourceIdentityMismatch,
    StaleRegistryGeneration,
    DuplicateNode,
    EntryLimitExceeded,
    ResourceLimitExceeded,
    NodeSequenceMismatch,
    NodeSequenceOverflow,
}

/// Bounded page-local vertical owner table. Every check is completed before
/// any map, generation, or resource-budget mutation.
#[derive(Debug, Clone, Default)]
pub(crate) struct VerticalShapingPageSidecars {
    registry_generation: Option<u64>,
    entries: HashMap<u32, Arc<BoundedVerticalHwp5TableCellSidecar>>,
    reserved_source_identities: HashSet<(u64, String, usize, u32)>,
    reserved_source_bytes: usize,
}

impl VerticalShapingPageSidecars {
    pub(crate) fn attach_bounded_hwp5_table_cell_atomic(
        &mut self,
        sidecar: Arc<BoundedVerticalHwp5TableCellSidecar>,
    ) -> Result<(), VerticalShapingSidecarRejectReason> {
        let node_id = sidecar.line_node_id();
        if node_id == 0 {
            return Err(VerticalShapingSidecarRejectReason::ZeroNode);
        }
        let certified = sidecar.transaction();
        let transaction = certified.transaction();
        let line = transaction.line_geometry();
        if transaction.product_published()
            || !Arc::ptr_eq(line, transaction.bbox_geometry())
            || !Arc::ptr_eq(line, transaction.next_origin_geometry())
        {
            return Err(VerticalShapingSidecarRejectReason::OwnerIdentityMismatch);
        }
        let certificate = certified.certificate();
        if certificate.font_source_sha256() != NOTO_SANS_KR_REGULAR_SHA256
            || certificate.font_bytes() != certificate.source_bytes_arc().len()
            || certificate.source_bytes_arc().len() > super::shaping::MAX_SHAPING_FONT_BYTES
        {
            return Err(VerticalShapingSidecarRejectReason::SourceIdentityMismatch);
        }
        if self.entries.contains_key(&node_id) {
            return Err(VerticalShapingSidecarRejectReason::DuplicateNode);
        }
        if self
            .registry_generation
            .is_some_and(|generation| generation != certificate.registry_generation())
        {
            return Err(VerticalShapingSidecarRejectReason::StaleRegistryGeneration);
        }
        if self.entries.len() >= MAX_VERTICAL_SHAPING_PAGE_SIDECARS {
            return Err(VerticalShapingSidecarRejectReason::EntryLimitExceeded);
        }

        let identity = (
            certificate.registry_generation(),
            certificate.font_source_sha256().to_string(),
            certificate.font_bytes(),
            certificate.face_index(),
        );
        let is_new_source = !self.reserved_source_identities.contains(&identity);
        let next_source_count = self.reserved_source_identities.len() + usize::from(is_new_source);
        let next_source_bytes = if is_new_source {
            self.reserved_source_bytes
                .checked_add(certificate.source_bytes_arc().len())
                .ok_or(VerticalShapingSidecarRejectReason::ResourceLimitExceeded)?
        } else {
            self.reserved_source_bytes
        };
        if next_source_count > MAX_VERTICAL_SHAPING_PREPARED_SOURCES_PER_PAGE
            || next_source_bytes > MAX_VERTICAL_SHAPING_FONT_BYTES_PER_PAGE
        {
            return Err(VerticalShapingSidecarRejectReason::ResourceLimitExceeded);
        }

        self.registry_generation = Some(certificate.registry_generation());
        self.entries.insert(node_id, sidecar);
        if is_new_source {
            self.reserved_source_identities.insert(identity);
            self.reserved_source_bytes = next_source_bytes;
        }
        Ok(())
    }

    pub(crate) fn get(&self, node_id: u32) -> Option<&Arc<BoundedVerticalHwp5TableCellSidecar>> {
        self.entries.get(&node_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn registry_generation(&self) -> Option<u64> {
        self.registry_generation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterClass {
    Cjk,
    Latin,
    CjkPunctuation,
    Unsupported,
}

fn character_class(character: char) -> CharacterClass {
    let value = u32::from(character);
    if matches!(character, '\u{2014}' | '\u{2026}') {
        CharacterClass::CjkPunctuation
    } else if character.is_ascii_alphanumeric() {
        CharacterClass::Latin
    } else if matches!(
        value,
        0x1100..=0x11ff
            | 0x2e80..=0x2fff
            | 0x3040..=0x30ff
            | 0x3130..=0x318f
            | 0x31a0..=0x31bf
            | 0x3400..=0x4dbf
            | 0x4e00..=0x9fff
            | 0xac00..=0xd7af
            | 0xf900..=0xfaff
    ) {
        CharacterClass::Cjk
    } else {
        CharacterClass::Unsupported
    }
}

fn classify_run(
    text: &str,
    intent: TypedVerticalIntent,
) -> Result<VerticalRunClass, DormantVerticalShapingRejectReason> {
    let mut class = None;
    for character in text.chars() {
        let next = character_class(character);
        if next == CharacterClass::Unsupported {
            return Err(DormantVerticalShapingRejectReason::CharacterClassUnsupported);
        }
        if class.is_some_and(|current| current != next) {
            return Err(DormantVerticalShapingRejectReason::MixedRunUnsupported);
        }
        class = Some(next);
    }
    match class.ok_or(DormantVerticalShapingRejectReason::EmptyRun)? {
        CharacterClass::Cjk => Ok(VerticalRunClass::CjkUpright),
        CharacterClass::Latin => match intent.latin_orientation() {
            VerticalLatinOrientation::Sideways => Ok(VerticalRunClass::LatinSideways),
            VerticalLatinOrientation::Upright => Ok(VerticalRunClass::LatinUpright),
            VerticalLatinOrientation::NotApplicable => {
                Err(DormantVerticalShapingRejectReason::CharacterClassUnsupported)
            }
        },
        CharacterClass::CjkPunctuation => Ok(VerticalRunClass::CjkPunctuation),
        CharacterClass::Unsupported => {
            Err(DormantVerticalShapingRejectReason::CharacterClassUnsupported)
        }
    }
}

fn cluster_ranges(
    text: &str,
    applied: &AppliedShapingRun,
) -> Result<Vec<Range<usize>>, DormantVerticalShapingRejectReason> {
    let mut starts = applied
        .glyphs
        .iter()
        .map(|glyph| usize::try_from(glyph.cluster_utf8))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| DormantVerticalShapingRejectReason::ClusterMappingInvalid)?;
    if starts.is_empty() {
        return Err(DormantVerticalShapingRejectReason::ClusterMappingInvalid);
    }
    if starts
        .iter()
        .any(|start| *start >= text.len() || !text.is_char_boundary(*start))
    {
        return Err(DormantVerticalShapingRejectReason::ClusterMappingInvalid);
    }
    let mut boundaries = starts.clone();
    boundaries.sort_unstable();
    boundaries.dedup();
    if boundaries.first().copied() != Some(0) {
        return Err(DormantVerticalShapingRejectReason::ClusterMappingInvalid);
    }
    boundaries.push(text.len());
    starts
        .drain(..)
        .map(|start| {
            let index = boundaries
                .binary_search(&start)
                .map_err(|_| DormantVerticalShapingRejectReason::ClusterMappingInvalid)?;
            let end = boundaries
                .get(index + 1)
                .copied()
                .ok_or(DormantVerticalShapingRejectReason::ClusterMappingInvalid)?;
            Ok(start..end)
        })
        .collect()
}

fn glyph_rect(
    bounds: ttf_parser::Rect,
    origin: VerticalPoint,
    scale: f64,
    transform: VerticalGlyphTransform,
) -> VerticalRect {
    let (left, top, right, bottom) = match transform {
        VerticalGlyphTransform::Upright => (
            origin.x + f64::from(bounds.x_min) * scale,
            origin.y - f64::from(bounds.y_max) * scale,
            origin.x + f64::from(bounds.x_max) * scale,
            origin.y - f64::from(bounds.y_min) * scale,
        ),
        VerticalGlyphTransform::RotateClockwise90 => (
            origin.x + f64::from(bounds.y_min) * scale,
            origin.y + f64::from(bounds.x_min) * scale,
            origin.x + f64::from(bounds.y_max) * scale,
            origin.y + f64::from(bounds.x_max) * scale,
        ),
    };
    VerticalRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn validate_dormant_vertical_shaping_request(
    request: &DormantVerticalShapingRequest<'_>,
) -> Result<VerticalRunClass, DormantVerticalShapingRejection> {
    use DormantVerticalShapingRejectReason as Reject;

    let fallback = request.fallback_geometry;
    if !fallback.is_well_formed() {
        return Err(reject(Reject::LegacyGeometryMalformed, fallback));
    }
    if request.intent.writing_mode() == ShapingWritingMode::HorizontalTb {
        return Err(reject(Reject::HorizontalIntentUnsupported, fallback));
    }
    if request.shaping.direction != ShapingDirection::TopToBottom
        || request.shaping.writing_mode != request.intent.writing_mode()
    {
        return Err(reject(Reject::DirectionIntentMismatch, fallback));
    }
    if !request.font_size_px.is_finite() || request.font_size_px <= 0.0 {
        return Err(reject(Reject::FontSizeInvalid, fallback));
    }
    if !request.origin.is_finite() {
        return Err(reject(Reject::OriginInvalid, fallback));
    }
    if !request.column_pitch_px.is_finite() || request.column_pitch_px <= 0.0 {
        return Err(reject(Reject::ColumnPitchInvalid, fallback));
    }
    let run_class = classify_run(request.shaping.text, request.intent)
        .map_err(|reason| reject(reason, fallback))?;
    if !request.shaping.variations.is_empty() {
        return Err(reject(Reject::VariationGeometryUnsupported, fallback));
    }
    Ok(run_class)
}

pub(crate) fn prepare_dormant_vertical_shaping_transaction(
    request: DormantVerticalShapingRequest<'_>,
) -> Result<DormantVerticalShapingTransaction, DormantVerticalShapingRejection> {
    use DormantVerticalShapingRejectReason as Reject;

    let fallback = request.fallback_geometry;
    let run_class = validate_dormant_vertical_shaping_request(&request)?;

    let attempt = terminal_shaping_attempt(request.attempt_id, &request.shaping);
    let source = request
        .shaping
        .source
        .ok_or_else(|| reject(Reject::AppliedPayloadMissing, fallback))?;
    let face = Face::parse(source.bytes, source.face_index)
        .map_err(|_| reject(Reject::AppliedPayloadMissing, fallback))?;
    finish_dormant_vertical_shaping_transaction(request, run_class, attempt, &face)
}

fn prepare_verified_dormant_vertical_shaping_transaction(
    request: DormantVerticalShapingRequest<'_>,
    face: &Face<'_>,
    source_digest: &str,
) -> Result<DormantVerticalShapingTransaction, DormantVerticalShapingRejection> {
    use DormantVerticalShapingRejectReason as Reject;

    let fallback = request.fallback_geometry;
    let run_class = validate_dormant_vertical_shaping_request(&request)?;
    let identity = canonicalize_verified_shaping_request(&request.shaping, face, source_digest)
        .map_err(|decision| {
            reject(
                Reject::ShapingRejected(
                    decision
                        .reason
                        .unwrap_or(ShapingRejectReason::ShapingUnavailable),
                ),
                fallback,
            )
        })?;
    let source = request
        .shaping
        .source
        .ok_or_else(|| reject(Reject::AppliedPayloadMissing, fallback))?;
    let mut shaping_face = rustybuzz::Face::from_slice(source.bytes, source.face_index)
        .ok_or_else(|| {
            reject(
                Reject::ShapingRejected(ShapingRejectReason::ShapingUnavailable),
                fallback,
            )
        })?;
    let attempt = terminal_shaping_attempt_from_output(
        request.attempt_id,
        shape_canonical_request_with_face(&request.shaping, identity, &mut shaping_face),
    );
    finish_dormant_vertical_shaping_transaction(request, run_class, attempt, face)
}

fn finish_dormant_vertical_shaping_transaction(
    request: DormantVerticalShapingRequest<'_>,
    run_class: VerticalRunClass,
    attempt: TerminalShapingAttempt,
    face: &Face<'_>,
) -> Result<DormantVerticalShapingTransaction, DormantVerticalShapingRejection> {
    use DormantVerticalShapingRejectReason as Reject;

    let fallback = request.fallback_geometry;
    if attempt.trace.disposition != TerminalShapingDisposition::Applied {
        return Err(reject(
            Reject::ShapingRejected(
                attempt
                    .trace
                    .reason
                    .unwrap_or(ShapingRejectReason::ShapingUnavailable),
            ),
            fallback,
        ));
    }
    let trace = attempt.trace;
    let applied = attempt
        .applied
        .ok_or_else(|| reject(Reject::AppliedPayloadMissing, fallback))?;
    let units_per_em = face.units_per_em();
    if units_per_em == 0 {
        return Err(reject(Reject::FontUnitsPerEmInvalid, fallback));
    }
    let scale = request.font_size_px / f64::from(units_per_em);
    let ranges = cluster_ranges(request.shaping.text, &applied)
        .map_err(|reason| reject(reason, fallback))?;
    let transform = if run_class == VerticalRunClass::LatinSideways {
        VerticalGlyphTransform::RotateClockwise90
    } else {
        VerticalGlyphTransform::Upright
    };

    let mut pen_x = 0_i64;
    let mut pen_y = 0_i64;
    let mut glyphs = Vec::with_capacity(applied.glyphs.len());
    let mut run_bbox: Option<VerticalRect> = None;
    for (glyph, cluster_utf8_range) in applied.glyphs.iter().zip(ranges) {
        if glyph.y_advance > 0 {
            return Err(reject(Reject::VerticalAdvanceInvalid, fallback));
        }
        let glyph_id = u16::try_from(glyph.glyph_id)
            .map_err(|_| reject(Reject::GlyphIdOutOfRange, fallback))?;
        if glyph_id == 0 {
            return Err(reject(Reject::MissingGlyph, fallback));
        }
        let bounds = face
            .glyph_bounding_box(GlyphId(glyph_id))
            .ok_or_else(|| reject(Reject::GlyphBoundsUnavailable, fallback))?;
        let origin = VerticalPoint {
            x: request.origin.x + (pen_x as f64 + f64::from(glyph.x_offset)) * scale,
            y: request.origin.y - (pen_y as f64 + f64::from(glyph.y_offset)) * scale,
        };
        let bbox = glyph_rect(bounds, origin, scale, transform);
        if !origin.is_finite() || !bbox.is_well_formed() {
            return Err(reject(Reject::GeometryMalformed, fallback));
        }
        run_bbox = Some(match run_bbox {
            Some(current) => current.union(bbox),
            None => bbox,
        });
        glyphs.push(VerticalGlyphGeometry {
            glyph_id: glyph.glyph_id,
            cluster_utf8_range,
            origin,
            bbox,
            transform,
        });
        pen_x = pen_x
            .checked_add(i64::from(glyph.x_advance))
            .ok_or_else(|| reject(Reject::GeometryMalformed, fallback))?;
        pen_y = pen_y
            .checked_add(i64::from(glyph.y_advance))
            .ok_or_else(|| reject(Reject::GeometryMalformed, fallback))?;
    }
    let bbox = run_bbox.ok_or_else(|| reject(Reject::GeometryMalformed, fallback))?;
    let inline_advance_px = -(pen_y as f64) * scale;
    let next_inline_origin = VerticalPoint {
        x: request.origin.x + pen_x as f64 * scale,
        y: request.origin.y + inline_advance_px,
    };
    let next_column_origin = VerticalPoint {
        x: request.origin.x
            + match request.intent.writing_mode() {
                ShapingWritingMode::VerticalRl => -request.column_pitch_px,
                ShapingWritingMode::VerticalLr => request.column_pitch_px,
                ShapingWritingMode::HorizontalTb => 0.0,
            },
        y: request.origin.y,
    };
    if !bbox.is_well_formed()
        || !inline_advance_px.is_finite()
        || inline_advance_px <= 0.0
        || !next_inline_origin.is_finite()
        || !next_column_origin.is_finite()
    {
        return Err(reject(Reject::GeometryMalformed, fallback));
    }

    Ok(DormantVerticalShapingTransaction {
        trace,
        applied,
        geometry: Arc::new(VerticalShapingGeometry {
            run_class,
            writing_mode: request.intent.writing_mode(),
            glyphs,
            bbox,
            inline_advance_px,
            next_inline_origin,
            next_column_origin,
        }),
        fallback_geometry: fallback,
    })
}
