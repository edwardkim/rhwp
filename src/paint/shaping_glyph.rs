//! Dormant Q2-D3 lowering from page-local common-shaping decisions.
//!
//! This module deliberately has no product caller before Q2-D4.  It proves
//! that the final `LayerNode::source_node_id` can recover one exact sidecar and
//! losslessly express its glyph geometry with the existing glyph-run schema.

use sha2::{Digest, Sha256};

use crate::paint::{
    font_blob_resource_key, resource_digest_hex, BinaryResourceKind, BinaryResourceRef,
    EmbeddedFontFace, FontBlobKey, FontBlobResource, FontDigest, FontFaceKey, FontFaceResource,
    FontFallbackPolicyId, FontInstanceKey, FontPortability, FontResourceSource, GlyphCluster,
    GlyphClusterFlag, GlyphRange, GlyphRunDiagnostics, GlyphRunOrientation,
    GlyphRunReplayEligibility, GlyphTransform, LanguageTag, LayerAffineTransform,
    LayerGlyphRunPaint, LayerNode, LayerPoint, LayerVector, LocalizedName, OpenTypeFeatureSetting,
    PaintTextStyle, PaintVariantMeta, ResourceArena, ScriptTag, ShapeKey, ShapingEngineId,
    TextDirection, TextRunPlacement, TextSourceId, TextSourceRange, TextSourceSpan,
    TextVariantKind, TextVariantQuality, WritingMode, MAX_PORTABLE_FONT_BLOB_BYTES,
    MAX_PORTABLE_GLYPHS_PER_RUN, RESOURCE_KEY_ALGORITHM,
};
use crate::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use crate::renderer::shaping_publication::{
    HorizontalShapingPageSidecars, HorizontalShapingRunDecision,
};

const MAX_COMMON_SHAPING_FONT_BYTES_PER_PAGE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HorizontalShapingGlyphLoweringRejectReason {
    MissingSourceNode,
    MissingSidecar,
    RejectedDecision,
    UnsupportedRunSurface,
    RunRangeMismatch,
    MeasurementIdentityMismatch,
    EmbeddedFaceNotFound,
    EmbeddedFaceAmbiguous,
    EmbeddedFaceInvalid,
    MeasurementGeometryInvalid,
    ClusterMappingInvalid,
    ResourceLimitExceeded,
}

impl HorizontalShapingGlyphLoweringRejectReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::MissingSourceNode => "missingSourceNode",
            Self::MissingSidecar => "missingSidecar",
            Self::RejectedDecision => "rejectedDecision",
            Self::UnsupportedRunSurface => "unsupportedRunSurface",
            Self::RunRangeMismatch => "runRangeMismatch",
            Self::MeasurementIdentityMismatch => "measurementIdentityMismatch",
            Self::EmbeddedFaceNotFound => "embeddedFaceNotFound",
            Self::EmbeddedFaceAmbiguous => "embeddedFaceAmbiguous",
            Self::EmbeddedFaceInvalid => "embeddedFaceInvalid",
            Self::MeasurementGeometryInvalid => "measurementGeometryInvalid",
            Self::ClusterMappingInvalid => "clusterMappingInvalid",
            Self::ResourceLimitExceeded => "resourceLimitExceeded",
        }
    }
}

/// One page-local lowering attempt. Rejected attempts retain only the source
/// node and a typed reason; raw text, font bytes, and host paths are excluded.
#[derive(Debug, Clone)]
pub(crate) struct HorizontalShapingGlyphLoweringReport {
    pub source_node_id: Option<u32>,
    pub glyph_run: Option<LayerGlyphRunPaint>,
    pub reject_reason: Option<HorizontalShapingGlyphLoweringRejectReason>,
    /// A successful common run owns the one GlyphRun alternative. D4 must use
    /// this bit to skip the nominal lowerer for the same TextRun fallback.
    pub claims_glyph_run_slot: bool,
}

impl HorizontalShapingGlyphLoweringReport {
    fn rejected(
        source_node_id: Option<u32>,
        reason: HorizontalShapingGlyphLoweringRejectReason,
    ) -> Self {
        Self {
            source_node_id,
            glyph_run: None,
            reject_reason: Some(reason),
            claims_glyph_run_slot: false,
        }
    }

    fn emitted(source_node_id: u32, glyph_run: LayerGlyphRunPaint) -> Self {
        Self {
            source_node_id: Some(source_node_id),
            glyph_run: Some(glyph_run),
            reject_reason: None,
            claims_glyph_run_slot: true,
        }
    }
}

fn same_f64(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1.0e-9 * left.abs().max(right.abs()).max(1.0)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut value, byte| {
            use std::fmt::Write;
            write!(&mut value, "{byte:02x}").expect("String formatting cannot fail");
            value
        })
}

fn run_surface_supported(run: &TextRunNode) -> bool {
    let style = &run.style;
    !run.text.is_empty()
        && run
            .text
            .chars()
            .take(MAX_PORTABLE_GLYPHS_PER_RUN + 1)
            .count()
            <= MAX_PORTABLE_GLYPHS_PER_RUN
        && run.layout_positions.is_none()
        && run.display_text.is_none()
        && !run.is_vertical
        && run.rotation.abs() <= f64::EPSILON
        && run.char_overlap.is_none()
        && run.border_fill_id == 0
        && matches!(run.field_marker, FieldMarkerType::None)
        && !style.bold
        && !style.italic
        && style.font_size.is_finite()
        && style.font_size > 0.0
        && style.ratio.is_finite()
        && style.ratio > 0.0
        && style.ratio <= 16.0
        && style.letter_spacing.abs() <= f64::EPSILON
        && style.extra_word_spacing.abs() <= f64::EPSILON
        && style.extra_char_spacing.abs() <= f64::EPSILON
        && style.extra_dash_advance.abs() <= f64::EPSILON
        && style.tab_leaders.is_empty()
        && style.inline_tabs.is_empty()
        && {
            let mut paint_style = PaintTextStyle::from(style);
            // Q2 geometry already carries width ratio. Existing GlyphTransform
            // is its explicit replay representation; inspect other effects with
            // an identity ratio so width ratio is not mistaken for an effect.
            paint_style.ratio = 1.0;
            paint_style.is_fill_only_glyph_replay()
        }
}

fn text_run_placement(bbox: BoundingBox, run: &TextRunNode) -> TextRunPlacement {
    let radians = run.rotation.to_radians();
    let (sin, cos) = radians.sin_cos();
    let local_origin_x = -bbox.width / 2.0;
    let local_origin_y = -bbox.height / 2.0 + run.baseline;
    let center_x = bbox.x + bbox.width / 2.0;
    let center_y = bbox.y + bbox.height / 2.0;
    TextRunPlacement {
        run_to_page: LayerAffineTransform {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: center_x + cos * local_origin_x - sin * local_origin_y,
            f: center_y + sin * local_origin_x + cos * local_origin_y,
        },
        baseline_y: 0.0,
    }
}

fn matching_embedded_face<'a>(
    decision: &HorizontalShapingRunDecision,
    run: &TextRunNode,
    fonts: &'a [EmbeddedFontFace<'a>],
) -> Result<(EmbeddedFontFace<'a>, ttf_parser::Face<'a>), HorizontalShapingGlyphLoweringRejectReason>
{
    let measurement = decision
        .measurement()
        .ok_or(HorizontalShapingGlyphLoweringRejectReason::RejectedDecision)?;
    let char_shape_id = run
        .char_shape_id
        .ok_or(HorizontalShapingGlyphLoweringRejectReason::UnsupportedRunSurface)?;
    let first = run
        .text
        .chars()
        .next()
        .ok_or(HorizontalShapingGlyphLoweringRejectReason::UnsupportedRunSurface)?;
    let language_index = crate::renderer::style_resolver::detect_lang_category(first);
    if run.text.chars().any(|character| {
        crate::renderer::style_resolver::detect_lang_category(character) != language_index
    }) {
        return Err(HorizontalShapingGlyphLoweringRejectReason::UnsupportedRunSurface);
    }
    let source = &measurement.source_handle;
    let mut matching = fonts.iter().copied().filter(|font| {
        font.char_shape_id == char_shape_id
            && font.language_index == language_index
            && font.face_index == source.face_index
            && font.bytes.len() == source.font_bytes
            && sha256_hex(font.bytes) == source.font_source_sha256
    });
    let font = matching
        .next()
        .ok_or(HorizontalShapingGlyphLoweringRejectReason::EmbeddedFaceNotFound)?;
    if matching.next().is_some() {
        return Err(HorizontalShapingGlyphLoweringRejectReason::EmbeddedFaceAmbiguous);
    }
    if font.bytes.len() > MAX_PORTABLE_FONT_BLOB_BYTES {
        return Err(HorizontalShapingGlyphLoweringRejectReason::ResourceLimitExceeded);
    }
    let face = ttf_parser::Face::parse(font.bytes, font.face_index)
        .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::EmbeddedFaceInvalid)?;
    if face.units_per_em() != measurement.units_per_em {
        return Err(HorizontalShapingGlyphLoweringRejectReason::EmbeddedFaceInvalid);
    }
    Ok((font, face))
}

fn validate_measurement(
    decision: &HorizontalShapingRunDecision,
    run: &TextRunNode,
    face: &ttf_parser::Face<'_>,
) -> Result<(), HorizontalShapingGlyphLoweringRejectReason> {
    let measurement = decision
        .measurement()
        .ok_or(HorizontalShapingGlyphLoweringRejectReason::RejectedDecision)?;
    let identity = &measurement.applied.identity;
    if measurement.source_handle.font_source_sha256 != identity.font_source_sha256
        || measurement.source_handle.font_bytes != identity.font_bytes
        || measurement.source_handle.face_index != identity.face_index
        || identity.direction != "ltr"
        || identity.writing_mode != "horizontal-tb"
        || identity.script.as_deref() != Some("Hang")
        || identity.language.as_deref() != Some("ko")
        || identity.features.len() != 1
        || identity.features[0].tag != "kern"
        || identity.features[0].value != u32::from(run.style.kerning)
        || !identity.variations.is_empty()
        || measurement.glyphs_px.len() != measurement.applied.glyphs.len()
        || measurement.glyphs_px.is_empty()
        || measurement.glyphs_px.len() > MAX_PORTABLE_GLYPHS_PER_RUN
        || measurement.clusters.is_empty()
        || measurement.clusters.len() > MAX_PORTABLE_GLYPHS_PER_RUN
    {
        return Err(HorizontalShapingGlyphLoweringRejectReason::MeasurementIdentityMismatch);
    }

    let horizontal_scale =
        run.style.font_size * run.style.ratio / f64::from(measurement.units_per_em);
    let vertical_scale = run.style.font_size / f64::from(measurement.units_per_em);
    let mut pen_x = 0.0;
    for (pixel, design) in measurement
        .glyphs_px
        .iter()
        .zip(measurement.applied.glyphs.iter())
    {
        if pixel.glyph_id == 0
            || pixel.glyph_id >= u32::from(face.number_of_glyphs())
            || pixel.cluster_utf8 != design.cluster_utf8
            || !same_f64(
                pixel.x,
                pen_x + f64::from(design.x_offset) * horizontal_scale,
            )
            || !same_f64(pixel.y, f64::from(design.y_offset) * vertical_scale)
            || !same_f64(
                pixel.advance_x,
                f64::from(design.x_advance) * horizontal_scale,
            )
            || !same_f64(
                pixel.advance_y,
                f64::from(design.y_advance) * vertical_scale,
            )
        {
            return Err(HorizontalShapingGlyphLoweringRejectReason::MeasurementGeometryInvalid);
        }
        pen_x += pixel.advance_x;
    }
    if !same_f64(pen_x, measurement.total_advance_px) {
        return Err(HorizontalShapingGlyphLoweringRejectReason::MeasurementGeometryInvalid);
    }
    Ok(())
}

fn lower_clusters(
    decision: &HorizontalShapingRunDecision,
    run: &TextRunNode,
) -> Result<Vec<GlyphCluster>, HorizontalShapingGlyphLoweringRejectReason> {
    let measurement = decision
        .measurement()
        .ok_or(HorizontalShapingGlyphLoweringRejectReason::RejectedDecision)?;
    let mut utf8_offsets = Vec::with_capacity(measurement.code_point_count + 1);
    let mut utf16_offsets = Vec::with_capacity(measurement.code_point_count + 1);
    utf8_offsets.push(0usize);
    utf16_offsets.push(0usize);
    let mut utf8_cursor = 0usize;
    let mut utf16_cursor = 0usize;
    for character in run.text.chars() {
        utf8_cursor = utf8_cursor
            .checked_add(character.len_utf8())
            .ok_or(HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        utf16_cursor = utf16_cursor
            .checked_add(character.len_utf16())
            .ok_or(HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        utf8_offsets.push(utf8_cursor);
        utf16_offsets.push(utf16_cursor);
    }

    let mut scalar_cursor = 0usize;
    let mut glyph_cursor = 0usize;
    let mut cluster_advance = 0.0;
    let mut lowered = Vec::with_capacity(measurement.clusters.len());
    for cluster in &measurement.clusters {
        if cluster.scalar_start != scalar_cursor
            || cluster.glyph_start != glyph_cursor
            || cluster.scalar_start >= cluster.scalar_end
            || cluster.glyph_start >= cluster.glyph_end
            || cluster.scalar_end > measurement.code_point_count
            || cluster.glyph_end > measurement.glyphs_px.len()
            || utf8_offsets.get(cluster.scalar_start).copied() != Some(cluster.utf8_start)
            || utf8_offsets.get(cluster.scalar_end).copied() != Some(cluster.utf8_end)
            || !cluster.advance_px.is_finite()
        {
            return Err(HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid);
        }
        let utf8_start = u32::try_from(cluster.utf8_start)
            .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        let utf8_end = u32::try_from(cluster.utf8_end)
            .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        let utf16_start = u32::try_from(utf16_offsets[cluster.scalar_start])
            .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        let utf16_end = u32::try_from(utf16_offsets[cluster.scalar_end])
            .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        let glyph_start = u32::try_from(cluster.glyph_start)
            .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        let glyph_end = u32::try_from(cluster.glyph_end)
            .map_err(|_| HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid)?;
        let flags = if cluster.scalar_end - cluster.scalar_start
            > cluster.glyph_end - cluster.glyph_start
        {
            vec![GlyphClusterFlag::Ligature]
        } else {
            Vec::new()
        };
        lowered.push(GlyphCluster {
            source_range_utf8: TextSourceRange::new(utf8_start, utf8_end),
            source_range_utf16: Some(TextSourceRange::new(utf16_start, utf16_end)),
            text_range_utf8: Some(TextSourceRange::new(utf8_start, utf8_end)),
            glyph_range: GlyphRange::new(glyph_start, glyph_end),
            flags,
        });
        scalar_cursor = cluster.scalar_end;
        glyph_cursor = cluster.glyph_end;
        cluster_advance += cluster.advance_px;
    }
    if scalar_cursor != measurement.code_point_count
        || glyph_cursor != measurement.glyphs_px.len()
        || !same_f64(cluster_advance, measurement.total_advance_px)
    {
        return Err(HorizontalShapingGlyphLoweringRejectReason::ClusterMappingInvalid);
    }
    Ok(lowered)
}

fn register_portable_face(
    font: EmbeddedFontFace<'_>,
    face: &ttf_parser::Face<'_>,
    resources: &mut ResourceArena,
) -> FontFaceKey {
    let digest_value = resource_digest_hex(font.bytes);
    let resource_key = font_blob_resource_key(font.bytes.len(), &digest_value);
    let blob_key = FontBlobKey(resource_key.clone());
    let face_key = FontFaceKey(format!("{resource_key}:face:{}", font.face_index));
    let digest = FontDigest {
        algorithm: RESOURCE_KEY_ALGORITHM.to_string(),
        value: digest_value,
    };
    let data_ref = BinaryResourceRef {
        kind: BinaryResourceKind::FontBlob,
        id: resource_key,
    };
    resources.intern_font_blob_bytes(font.bytes);
    if !resources
        .font_resources()
        .blobs
        .iter()
        .any(|blob| blob.id == blob_key)
    {
        resources.font_resources_mut().blobs.push(FontBlobResource {
            id: blob_key.clone(),
            digest: Some(digest.clone()),
            source: FontResourceSource::Embedded,
            data_ref: Some(data_ref.clone()),
            portability: FontPortability::PortableBlob {
                digest: digest.clone(),
                data_ref: data_ref.clone(),
            },
        });
    }
    if !resources
        .font_resources()
        .faces
        .iter()
        .any(|registered| registered.id == face_key)
    {
        let mut family_names = vec![LocalizedName {
            locale: None,
            value: font.family.to_string(),
        }];
        if let Some(alternate) = font.alternate_family.filter(|name| *name != font.family) {
            family_names.push(LocalizedName {
                locale: None,
                value: alternate.to_string(),
            });
        }
        resources.font_resources_mut().faces.push(FontFaceResource {
            id: face_key.clone(),
            blob_key,
            face_index: font.face_index,
            postscript_name: None,
            family_names,
            style_names: Vec::new(),
            weight_class: Some(face.weight().to_number()),
            width_class: Some(face.width().to_number()),
            italic: Some(face.is_italic()),
        });
    }
    face_key
}

/// Lower one exact page-local decision without adding it to a product layer.
/// The original TextRun operation is owned by the caller and is never removed.
pub(crate) fn lower_horizontal_shaping_layer_node_shadow(
    node: &LayerNode,
    bbox: BoundingBox,
    run: &TextRunNode,
    text_source_id: u32,
    sidecars: &HorizontalShapingPageSidecars,
    fonts: &[EmbeddedFontFace<'_>],
    resources: &mut ResourceArena,
) -> HorizontalShapingGlyphLoweringReport {
    let Some(source_node_id) = node.source_node_id else {
        return HorizontalShapingGlyphLoweringReport::rejected(
            None,
            HorizontalShapingGlyphLoweringRejectReason::MissingSourceNode,
        );
    };
    let Some(decision) = sidecars.get(source_node_id) else {
        return HorizontalShapingGlyphLoweringReport::rejected(
            Some(source_node_id),
            HorizontalShapingGlyphLoweringRejectReason::MissingSidecar,
        );
    };
    if decision.measurement().is_none() {
        return HorizontalShapingGlyphLoweringReport::rejected(
            Some(source_node_id),
            HorizontalShapingGlyphLoweringRejectReason::RejectedDecision,
        );
    }
    if !run_surface_supported(run) {
        return HorizontalShapingGlyphLoweringReport::rejected(
            Some(source_node_id),
            HorizontalShapingGlyphLoweringRejectReason::UnsupportedRunSurface,
        );
    }
    let range = decision.range();
    let scalar_count = run.text.chars().count();
    if range.scalar_end - range.scalar_start != scalar_count
        || range.utf8_end - range.utf8_start != run.text.len()
        || range.utf16_end - range.utf16_start != run.text.encode_utf16().count()
    {
        return HorizontalShapingGlyphLoweringReport::rejected(
            Some(source_node_id),
            HorizontalShapingGlyphLoweringRejectReason::RunRangeMismatch,
        );
    }

    let (font, face) = match matching_embedded_face(decision, run, fonts) {
        Ok(value) => value,
        Err(reason) => {
            return HorizontalShapingGlyphLoweringReport::rejected(Some(source_node_id), reason)
        }
    };
    if let Err(reason) = validate_measurement(decision, run, &face) {
        return HorizontalShapingGlyphLoweringReport::rejected(Some(source_node_id), reason);
    }
    let clusters = match lower_clusters(decision, run) {
        Ok(value) => value,
        Err(reason) => {
            return HorizontalShapingGlyphLoweringReport::rejected(Some(source_node_id), reason)
        }
    };
    let measurement = decision
        .measurement()
        .expect("measurement was validated above");
    let glyph_ids = measurement
        .glyphs_px
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect();
    let positions = measurement
        .glyphs_px
        .iter()
        .map(|glyph| LayerPoint {
            x: glyph.x,
            y: glyph.y,
        })
        .collect();
    let advances = measurement
        .glyphs_px
        .iter()
        .map(|glyph| LayerVector {
            dx: glyph.advance_x,
            dy: glyph.advance_y,
        })
        .collect();
    let glyph_transforms = ((run.style.ratio - 1.0).abs() > f64::EPSILON).then(|| {
        vec![
            GlyphTransform {
                xx: run.style.ratio as f32,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            };
            measurement.glyphs_px.len()
        ]
    });

    // All validation precedes resource mutation, so a rejected attempt cannot
    // leave a partial portable-font publication behind.
    let digest_value = resource_digest_hex(font.bytes);
    let data_ref = BinaryResourceRef {
        kind: BinaryResourceKind::FontBlob,
        id: font_blob_resource_key(font.bytes.len(), &digest_value),
    };
    if resources.font_blob_bytes_for_ref(&data_ref).is_none() {
        let Some(existing_bytes) = resources
            .font_blob_resources()
            .try_fold(0usize, |total, (_, bytes)| total.checked_add(bytes.len()))
        else {
            return HorizontalShapingGlyphLoweringReport::rejected(
                Some(source_node_id),
                HorizontalShapingGlyphLoweringRejectReason::ResourceLimitExceeded,
            );
        };
        if existing_bytes
            .checked_add(font.bytes.len())
            .is_none_or(|total| total > MAX_COMMON_SHAPING_FONT_BYTES_PER_PAGE)
        {
            return HorizontalShapingGlyphLoweringReport::rejected(
                Some(source_node_id),
                HorizontalShapingGlyphLoweringRejectReason::ResourceLimitExceeded,
            );
        }
    }
    let face_key = register_portable_face(font, &face, resources);
    let equivalence_group = format!("text-{text_source_id}");
    let mut variant = PaintVariantMeta::text_run_default(equivalence_group.clone());
    variant.variant_id = "glyphRun".to_string();
    variant.variant_kind = TextVariantKind::GlyphRun;
    variant.is_default_fallback = false;
    variant.requires = vec!["fontResources".to_string(), "text.glyphRun".to_string()];
    variant.quality = Some(TextVariantQuality::Exact);
    variant.anchor_op_id = Some(equivalence_group);
    let identity = &measurement.applied.identity;
    let features = identity
        .features
        .iter()
        .map(|feature| OpenTypeFeatureSetting {
            tag: feature.tag.clone(),
            enabled: feature.value != 0,
            value: Some(feature.value),
        })
        .collect();
    let glyph_run = LayerGlyphRunPaint {
        source: TextSourceSpan {
            id: TextSourceId(text_source_id),
            utf8_range: TextSourceRange::new(0, run.text.len() as u32),
            utf16_range: TextSourceRange::new(0, run.text.encode_utf16().count() as u32),
            stable_source_key: None,
        },
        variant,
        paint_style: PaintTextStyle::from(&run.style),
        shape_key: ShapeKey {
            font_instance: FontInstanceKey {
                face_key,
                size_px: run.style.font_size,
                variations: Vec::new(),
                synthetic_bold: false,
                synthetic_italic: false,
            },
            direction: TextDirection::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            script: identity.script.clone().map(ScriptTag),
            language: identity.language.clone().map(LanguageTag),
            features,
            shaping_engine: ShapingEngineId("rustybuzz-q2-v1".to_string()),
            fallback_policy: FontFallbackPolicyId("none".to_string()),
        },
        placement: text_run_placement(bbox, run),
        glyph_ids,
        positions,
        advances: Some(advances),
        clusters,
        direction: TextDirection::Ltr,
        bidi_level: Some(0),
        writing_mode: WritingMode::HorizontalTb,
        orientation: GlyphRunOrientation::Horizontal,
        glyph_transforms,
        diagnostics: GlyphRunDiagnostics {
            quality: TextVariantQuality::Exact,
            replay_eligibility: GlyphRunReplayEligibility::Portable,
            // The payload geometry is exact, but #5821 compressed-ratio draw
            // authority must be reconciled before any strict replay selector
            // may publish this shadow run in Q2-D4.
            strict_visual_eligible: false,
            max_origin_delta_px: 0.0,
            max_advance_delta_px: 0.0,
            max_residual_after_adjustment_px: 0.0,
            cluster_mismatch_count: 0,
            missing_glyph_count: 0,
            used_fallback_font_count: 0,
            reason: Some("q2CommonShapingReplayAuthorityPending".to_string()),
        },
    };
    HorizontalShapingGlyphLoweringReport::emitted(source_node_id, glyph_run)
}
