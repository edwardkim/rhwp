//! Q4-D3 bounded bridge from a D2 vertical line sidecar to leaf-scoped text
//! alternatives.
//!
//! D3-A established the read-only mapping. D3-B stages every glyph op and the
//! portable font delta before one infallible line/resource commit. Backend
//! selection remains closed until D4.

use std::collections::HashSet;
use std::sync::Arc;

use crate::paint::{
    font_blob_resource_key, BinaryResourceKind, BinaryResourceRef, FontBlobKey, FontBlobResource,
    FontDigest, FontFaceKey, FontFaceResource, FontFallbackPolicyId, FontInstanceKey,
    FontPortability, FontResourceSource, GlyphCluster, GlyphClusterFlag, GlyphRange,
    GlyphRunDiagnostics, GlyphRunOrientation, GlyphRunReplayEligibility, LayerAffineTransform,
    LayerGlyphRunPaint, LayerNode, LayerNodeKind, LayerPoint, LayerVector, LocalizedName,
    OpenTypeFeatureSetting, PaintOp, PaintTextStyle, PaintVariantMeta, ResourceArena, ScriptTag,
    ShapeKey, ShapingEngineId, TextDirection, TextRunPlacement, TextSourceRange, TextSourceSpan,
    TextVariantKind, TextVariantQuality, VariationAxisValue, WritingMode,
    MAX_PORTABLE_FONT_BLOB_BYTES, RESOURCE_KEY_ALGORITHM,
};
use crate::renderer::render_tree::PageLayoutContext;
use crate::renderer::shaping_vertical::{
    prepare_bounded_vertical_glyph_publication_shadow, BoundedVerticalHwp5TableCellSidecar,
    VerticalGlyphPublicationLeafInput, VerticalGlyphPublicationShadow,
    VerticalGlyphPublicationShadowRejectReason, VerticalRect,
    MAX_VERTICAL_SHAPING_FONT_BYTES_PER_PAGE, MAX_VERTICAL_SHAPING_PREPARED_SOURCES_PER_PAGE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalGlyphLayerShadowRejectReason {
    WrongLineNode,
    UnsupportedLineSurface,
    TextSourceIdOverflow,
    Semantic(VerticalGlyphPublicationShadowRejectReason),
}

/// Audit one bounded line subtree without mutating layer ops or resources.
pub(crate) fn prepare_vertical_shaping_line_shadow(
    line: &LayerNode,
    first_text_source_id: u32,
    sidecar: &Arc<BoundedVerticalHwp5TableCellSidecar>,
) -> Result<VerticalGlyphPublicationShadow, VerticalGlyphLayerShadowRejectReason> {
    if line.source_node_id != Some(sidecar.line_node_id()) {
        return Err(VerticalGlyphLayerShadowRejectReason::WrongLineNode);
    }
    let LayerNodeKind::Group { children, .. } = &line.kind else {
        return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
    };
    let mut leaves = Vec::with_capacity(children.len());
    for (index, child) in children.iter().enumerate() {
        let Some(source_node_id) = child.source_node_id else {
            return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
        };
        let LayerNodeKind::Leaf { ops } = &child.kind else {
            return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
        };
        let [PaintOp::TextRun {
            bbox, run, source, ..
        }] = ops.as_slice()
        else {
            return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
        };
        let text_source_id = match source {
            Some(source) => source.id.0,
            None => first_text_source_id
                .checked_add(u32::try_from(index).unwrap_or(u32::MAX))
                .ok_or(VerticalGlyphLayerShadowRejectReason::TextSourceIdOverflow)?,
        };
        leaves.push(VerticalGlyphPublicationLeafInput {
            source_node_id,
            text_source_id,
            text: &run.text,
            is_vertical: run.is_vertical,
            bbox: VerticalRect {
                x: bbox.x,
                y: bbox.y,
                width: bbox.width,
                height: bbox.height,
            },
        });
    }
    prepare_bounded_vertical_glyph_publication_shadow(sidecar, &leaves)
        .map_err(VerticalGlyphLayerShadowRejectReason::Semantic)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerticalGlyphLinePublicationRejectReason {
    Shadow(VerticalGlyphLayerShadowRejectReason),
    FontLimitExceeded,
    FontIdentityMismatch,
    ResourceKeyConflict,
    UnsupportedFallbackStyle,
    SourceRangeOverflow,
    VariantScopeInvalid,
}

#[derive(Debug, Clone)]
struct VerticalFontResourceDelta {
    source_bytes: Arc<[u8]>,
    resource_hash_fnv1a64: u64,
    resource_fingerprint: [u8; 16],
    blob: FontBlobResource,
    face: FontFaceResource,
    data_ref: BinaryResourceRef,
    number_of_glyphs: u16,
}

#[derive(Debug, Clone)]
struct PreparedVerticalLinePublication {
    line: LayerNode,
    resource_delta: VerticalFontResourceDelta,
    claimed_text_sources: Vec<u32>,
}

fn prepare_vertical_font_delta(
    sidecar: &Arc<BoundedVerticalHwp5TableCellSidecar>,
    family: &str,
    resources: &ResourceArena,
) -> Result<VerticalFontResourceDelta, VerticalGlyphLinePublicationRejectReason> {
    let certificate = sidecar.transaction().certificate();
    let source_bytes = certificate.source_bytes_arc();
    if source_bytes.is_empty()
        || source_bytes.len() != certificate.font_bytes()
        || source_bytes.len() > MAX_PORTABLE_FONT_BLOB_BYTES
    {
        return Err(VerticalGlyphLinePublicationRejectReason::FontLimitExceeded);
    }
    if certificate.units_per_em() == 0 {
        return Err(VerticalGlyphLinePublicationRejectReason::FontIdentityMismatch);
    }

    let portable_font = certificate.portable_font();
    let digest_value = portable_font.resource_digest_blake3().to_string();
    if digest_value.len() != 64 || !digest_value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(VerticalGlyphLinePublicationRejectReason::FontIdentityMismatch);
    }
    let resource_key = font_blob_resource_key(source_bytes.len(), &digest_value);
    let digest = FontDigest {
        algorithm: RESOURCE_KEY_ALGORITHM.to_string(),
        value: digest_value,
    };
    let data_ref = BinaryResourceRef {
        kind: BinaryResourceKind::FontBlob,
        id: resource_key.clone(),
    };
    let blob_key = FontBlobKey(resource_key.clone());
    let face_key = FontFaceKey(format!("{resource_key}:face:{}", certificate.face_index()));
    let mut blob = FontBlobResource {
        id: blob_key.clone(),
        digest: Some(digest.clone()),
        source: FontResourceSource::Embedded,
        data_ref: Some(data_ref.clone()),
        portability: FontPortability::PortableBlob {
            digest,
            data_ref: data_ref.clone(),
        },
    };
    let number_of_glyphs = portable_font.number_of_glyphs();
    let mut face = FontFaceResource {
        id: face_key,
        blob_key,
        face_index: certificate.face_index(),
        postscript_name: None,
        family_names: vec![LocalizedName {
            locale: None,
            value: family.to_string(),
        }],
        style_names: Vec::new(),
        weight_class: Some(portable_font.weight_class()),
        width_class: Some(portable_font.width_class()),
        italic: Some(portable_font.italic()),
    };

    if resources
        .font_blob_bytes_for_ref(&data_ref)
        .is_some_and(|existing| existing != source_bytes.as_ref())
    {
        return Err(VerticalGlyphLinePublicationRejectReason::ResourceKeyConflict);
    }
    if let Some(existing) = resources
        .font_resources()
        .blobs
        .iter()
        .find(|existing| existing.id == blob.id)
    {
        if existing.digest != blob.digest
            || existing.data_ref != blob.data_ref
            || existing.portability != blob.portability
        {
            return Err(VerticalGlyphLinePublicationRejectReason::ResourceKeyConflict);
        }
        blob = existing.clone();
    }
    if let Some(existing) = resources
        .font_resources()
        .faces
        .iter()
        .find(|existing| existing.id == face.id)
    {
        if existing.blob_key != face.blob_key
            || existing.face_index != face.face_index
            || existing.weight_class != face.weight_class
            || existing.width_class != face.width_class
            || existing.italic != face.italic
        {
            return Err(VerticalGlyphLinePublicationRejectReason::ResourceKeyConflict);
        }
        // Family/style names are descriptive aliases, not face identity.
        // Reuse the first registered metadata instead of rejecting the same
        // portable blob when another run names it differently.
        face = existing.clone();
    }

    if resources.font_blob_bytes_for_ref(&data_ref).is_none() {
        let existing_bytes = resources
            .font_blob_resources()
            .try_fold(0usize, |total, (_, bytes)| total.checked_add(bytes.len()))
            .ok_or(VerticalGlyphLinePublicationRejectReason::FontLimitExceeded)?;
        if existing_bytes
            .checked_add(source_bytes.len())
            .is_none_or(|total| total > MAX_VERTICAL_SHAPING_FONT_BYTES_PER_PAGE)
            || resources.font_blob_count() >= MAX_VERTICAL_SHAPING_PREPARED_SOURCES_PER_PAGE
        {
            return Err(VerticalGlyphLinePublicationRejectReason::FontLimitExceeded);
        }
    }

    Ok(VerticalFontResourceDelta {
        source_bytes: Arc::clone(source_bytes),
        resource_hash_fnv1a64: portable_font.resource_hash_fnv1a64(),
        resource_fingerprint: portable_font.resource_fingerprint(),
        blob,
        face,
        data_ref,
        number_of_glyphs,
    })
}

fn build_vertical_leaf_glyph_run(
    fallback: &crate::renderer::render_tree::TextRunNode,
    source: TextSourceSpan,
    leaf: &crate::renderer::shaping_vertical::VerticalGlyphPublicationLeafShadow,
    sidecar: &Arc<BoundedVerticalHwp5TableCellSidecar>,
    face_key: FontFaceKey,
) -> Result<LayerGlyphRunPaint, VerticalGlyphLinePublicationRejectReason> {
    let paint_style = PaintTextStyle::from(&fallback.style);
    if !paint_style.is_fill_only_glyph_replay()
        || !paint_style.font_size.is_finite()
        || paint_style.font_size <= 0.0
    {
        return Err(VerticalGlyphLinePublicationRejectReason::UnsupportedFallbackStyle);
    }
    let utf8 = leaf.source_utf8_range();
    let utf16 = leaf.source_utf16_range();
    let utf8_start = u32::try_from(utf8.start)
        .map_err(|_| VerticalGlyphLinePublicationRejectReason::SourceRangeOverflow)?;
    let utf8_end = u32::try_from(utf8.end)
        .map_err(|_| VerticalGlyphLinePublicationRejectReason::SourceRangeOverflow)?;
    let utf16_start = u32::try_from(utf16.start)
        .map_err(|_| VerticalGlyphLinePublicationRejectReason::SourceRangeOverflow)?;
    let utf16_end = u32::try_from(utf16.end)
        .map_err(|_| VerticalGlyphLinePublicationRejectReason::SourceRangeOverflow)?;
    let equivalence_group = format!("text-{}", leaf.text_source_id());
    let mut variant = PaintVariantMeta::text_run_default(equivalence_group.clone());
    variant.variant_id = "verticalGlyphRun".to_string();
    variant.variant_kind = TextVariantKind::GlyphRun;
    variant.is_default_fallback = false;
    variant.requires = vec![
        "fontResources".to_string(),
        "text.glyphRun".to_string(),
        "text.glyphRun.verticalUpright".to_string(),
    ];
    variant.quality = Some(TextVariantQuality::Exact);
    variant.anchor_op_id = Some(equivalence_group);

    let identity = &sidecar.transaction().transaction().applied().identity;
    let features = identity
        .features
        .iter()
        .map(|feature| OpenTypeFeatureSetting {
            tag: feature.tag.clone(),
            enabled: feature.value != 0,
            value: Some(feature.value),
        })
        .collect();
    let variations = identity
        .variations
        .iter()
        .map(|variation| VariationAxisValue {
            tag: variation.tag.clone(),
            value: f32::from_bits(variation.value_bits),
        })
        .collect();
    let origin = leaf.origin();
    let advance = leaf.advance();
    let font_size = paint_style.font_size;

    Ok(LayerGlyphRunPaint {
        source,
        variant,
        paint_style,
        shape_key: ShapeKey {
            font_instance: FontInstanceKey {
                face_key,
                size_px: font_size,
                variations,
                synthetic_bold: false,
                synthetic_italic: false,
            },
            direction: TextDirection::Ltr,
            writing_mode: WritingMode::VerticalRl,
            script: identity.script.clone().map(ScriptTag),
            language: identity.language.clone().map(crate::paint::LanguageTag),
            features,
            shaping_engine: ShapingEngineId("rustybuzz-q4-vertical-v1".to_string()),
            fallback_policy: FontFallbackPolicyId("none".to_string()),
        },
        placement: TextRunPlacement {
            run_to_page: LayerAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 0.0,
                f: 0.0,
            },
            baseline_y: 0.0,
        },
        glyph_ids: vec![leaf.glyph_id()],
        positions: vec![LayerPoint {
            x: origin.x,
            y: origin.y,
        }],
        advances: Some(vec![LayerVector {
            dx: advance.x,
            dy: advance.y,
        }]),
        clusters: vec![GlyphCluster {
            source_range_utf8: TextSourceRange::new(utf8_start, utf8_end),
            source_range_utf16: Some(TextSourceRange::new(utf16_start, utf16_end)),
            text_range_utf8: Some(TextSourceRange::new(utf8_start, utf8_end)),
            glyph_range: GlyphRange::new(0, 1),
            flags: vec![GlyphClusterFlag::FallbackBoundary],
        }],
        direction: TextDirection::Ltr,
        bidi_level: Some(0),
        writing_mode: WritingMode::VerticalRl,
        orientation: GlyphRunOrientation::VerticalUpright,
        glyph_transforms: None,
        diagnostics: GlyphRunDiagnostics {
            quality: TextVariantQuality::Exact,
            replay_eligibility: GlyphRunReplayEligibility::Portable,
            strict_visual_eligible: true,
            max_origin_delta_px: 0.0,
            max_advance_delta_px: 0.0,
            max_residual_after_adjustment_px: 0.0,
            cluster_mismatch_count: 0,
            missing_glyph_count: 0,
            used_fallback_font_count: 0,
            reason: Some("boundedVerticalHwp5TableCellV1".to_string()),
        },
    })
}

fn prepare_vertical_line_publication(
    line: &LayerNode,
    first_text_source_id: u32,
    sidecar: &Arc<BoundedVerticalHwp5TableCellSidecar>,
    resources: &ResourceArena,
) -> Result<PreparedVerticalLinePublication, VerticalGlyphLinePublicationRejectReason> {
    let shadow = prepare_vertical_shaping_line_shadow(line, first_text_source_id, sidecar)
        .map_err(VerticalGlyphLinePublicationRejectReason::Shadow)?;
    let LayerNodeKind::Group { children, .. } = &line.kind else {
        return Err(VerticalGlyphLinePublicationRejectReason::Shadow(
            VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface,
        ));
    };
    let family = children
        .first()
        .and_then(|child| match &child.kind {
            LayerNodeKind::Leaf { ops } => match ops.as_slice() {
                [PaintOp::TextRun { run, .. }] => Some(run.style.font_family.as_str()),
                _ => None,
            },
            _ => None,
        })
        .ok_or(VerticalGlyphLinePublicationRejectReason::UnsupportedFallbackStyle)?;
    let resource_delta = prepare_vertical_font_delta(sidecar, family, resources)?;
    if shadow
        .leaves()
        .iter()
        .any(|leaf| leaf.glyph_id() >= u32::from(resource_delta.number_of_glyphs))
    {
        return Err(VerticalGlyphLinePublicationRejectReason::FontIdentityMismatch);
    }

    let mut staged_line = line.clone();
    let LayerNodeKind::Group {
        children: staged_children,
        ..
    } = &mut staged_line.kind
    else {
        unreachable!("shadow preparation accepted only a group");
    };
    let mut claimed_text_sources = Vec::with_capacity(shadow.leaves().len());
    for (child, leaf) in staged_children.iter_mut().zip(shadow.leaves()) {
        let LayerNodeKind::Leaf { ops } = &mut child.kind else {
            return Err(VerticalGlyphLinePublicationRejectReason::Shadow(
                VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface,
            ));
        };
        let [PaintOp::TextRun { bbox, run, source }] = ops.as_slice() else {
            return Err(VerticalGlyphLinePublicationRejectReason::Shadow(
                VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface,
            ));
        };
        if run.style.font_family != family {
            return Err(VerticalGlyphLinePublicationRejectReason::UnsupportedFallbackStyle);
        }
        // The glyph alternative must carry the exact identity already bound to
        // its fallback; synthesizing a parallel span loses stable source keys.
        let source = source
            .clone()
            .unwrap_or_else(|| TextSourceSpan::for_text_run(leaf.text_source_id(), run));
        let glyph_run = build_vertical_leaf_glyph_run(
            run,
            source,
            leaf,
            sidecar,
            resource_delta.face.id.clone(),
        )?;
        ops.push(PaintOp::GlyphRun {
            bbox: *bbox,
            run: Box::new(glyph_run),
        });
        claimed_text_sources.push(leaf.text_source_id());
    }

    let staged_tree = crate::paint::PageLayerTree::new(
        staged_line.bounds.width,
        staged_line.bounds.height,
        staged_line.clone(),
    );
    crate::paint::validate_text_variant_scope(&staged_tree)
        .map_err(|_| VerticalGlyphLinePublicationRejectReason::VariantScopeInvalid)?;
    Ok(PreparedVerticalLinePublication {
        line: staged_line,
        resource_delta,
        claimed_text_sources,
    })
}

fn commit_vertical_line_publication(
    line: &mut LayerNode,
    resources: &mut ResourceArena,
    prepared: PreparedVerticalLinePublication,
) -> Vec<u32> {
    let delta = prepared.resource_delta;
    if resources.font_blob_bytes_for_ref(&delta.data_ref).is_none() {
        resources.intern_prepared_font_blob_arc(
            delta.source_bytes,
            delta.resource_hash_fnv1a64,
            delta.resource_fingerprint,
            delta.data_ref.id.clone(),
        );
    }
    if !resources
        .font_resources()
        .blobs
        .iter()
        .any(|existing| existing.id == delta.blob.id)
    {
        resources.font_resources_mut().blobs.push(delta.blob);
    }
    if !resources
        .font_resources()
        .faces
        .iter()
        .any(|existing| existing.id == delta.face.id)
    {
        resources.font_resources_mut().faces.push(delta.face);
    }
    *line = prepared.line;
    prepared.claimed_text_sources
}

/// Publish every accepted bounded vertical line before horizontal and nominal
/// lowerers run. Failed lines leave both the subtree and resource arena intact.
pub(crate) fn lower_vertical_shaping_page_sidecars(
    root: &mut LayerNode,
    frame: &PageLayoutContext,
    resources: &mut ResourceArena,
) -> HashSet<u32> {
    fn lower_node(
        node: &mut LayerNode,
        frame: &PageLayoutContext,
        resources: &mut ResourceArena,
        next_text_source_id: &mut u32,
        claimed: &mut HashSet<u32>,
    ) {
        if let Some(sidecar) = node
            .source_node_id
            .and_then(|node_id| frame.vertical_shaping_sidecar(node_id))
        {
            if let Ok(prepared) =
                prepare_vertical_line_publication(node, *next_text_source_id, sidecar, resources)
            {
                claimed.extend(commit_vertical_line_publication(node, resources, prepared));
            }
        }
        match &mut node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    lower_node(child, frame, resources, next_text_source_id, claimed);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => {
                lower_node(child, frame, resources, next_text_source_id, claimed);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextRun { source, .. } = op {
                        *next_text_source_id = match source {
                            Some(source) => {
                                (*next_text_source_id).max(source.id.0.saturating_add(1))
                            }
                            None => next_text_source_id.saturating_add(1),
                        };
                    }
                }
            }
        }
    }

    let mut next_text_source_id = 0;
    let mut claimed = HashSet::new();
    lower_node(
        root,
        frame,
        resources,
        &mut next_text_source_id,
        &mut claimed,
    );
    claimed
}
