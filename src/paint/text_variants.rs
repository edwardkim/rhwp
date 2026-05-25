//! Text variant grouping validation.
//!
//! Schema v1 keeps `TextRun` as the root fallback op and attaches optional
//! visual alternatives such as `GlyphRun` through variant metadata. Consumers
//! choose one variant set per equivalence group.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::paint::{
    LayerNode, LayerNodeKind, PageLayerTree, PaintOp, PaintVariantMeta, TextVariantKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextVariantScopeError {
    CrossLeafGroup {
        equivalence_group: String,
        first_leaf: String,
        second_leaf: String,
    },
    MissingDefaultFallback {
        equivalence_group: String,
        leaf: String,
    },
    MissingSidecarAnchorOpId {
        equivalence_group: String,
        variant_id: String,
        leaf: String,
    },
    InvalidSidecarAnchor {
        equivalence_group: String,
        variant_id: String,
        anchor_op_id: String,
        leaf: String,
    },
    MixedGlyphOutlinePayload {
        equivalence_group: String,
        variant_id: String,
        leaf: String,
    },
    EmptyVariantSet {
        equivalence_group: String,
        variant_id: String,
        leaf: String,
    },
    DuplicatePart {
        equivalence_group: String,
        variant_id: String,
        part_index: u32,
        leaf: String,
    },
    PartCountMismatch {
        equivalence_group: String,
        variant_id: String,
        expected: u32,
        actual: u32,
        leaf: String,
    },
}

impl fmt::Display for TextVariantScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CrossLeafGroup {
                equivalence_group,
                first_leaf,
                second_leaf,
            } => write!(
                f,
                "text variant group `{equivalence_group}` crosses leaf scope `{first_leaf}` and `{second_leaf}`"
            ),
            Self::MissingDefaultFallback {
                equivalence_group,
                leaf,
            } => write!(
                f,
                "text variant group `{equivalence_group}` in leaf `{leaf}` has no default fallback"
            ),
            Self::MissingSidecarAnchorOpId {
                equivalence_group,
                variant_id,
                leaf,
            } => write!(
                f,
                "text sidecar variant `{variant_id}` in group `{equivalence_group}` at leaf `{leaf}` has no anchorOpId"
            ),
            Self::InvalidSidecarAnchor {
                equivalence_group,
                variant_id,
                anchor_op_id,
                leaf,
            } => write!(
                f,
                "text sidecar variant `{variant_id}` in group `{equivalence_group}` at leaf `{leaf}` anchors `{anchor_op_id}`, expected the same paint-order slot"
            ),
            Self::MixedGlyphOutlinePayload {
                equivalence_group,
                variant_id,
                leaf,
            } => write!(
                f,
                "glyph outline variant `{variant_id}` in group `{equivalence_group}` at leaf `{leaf}` mixes payload families"
            ),
            Self::EmptyVariantSet {
                equivalence_group,
                variant_id,
                leaf,
            } => write!(
                f,
                "text variant `{variant_id}` in group `{equivalence_group}` at leaf `{leaf}` has zero parts"
            ),
            Self::DuplicatePart {
                equivalence_group,
                variant_id,
                part_index,
                leaf,
            } => write!(
                f,
                "text variant `{variant_id}` in group `{equivalence_group}` at leaf `{leaf}` repeats part {part_index}"
            ),
            Self::PartCountMismatch {
                equivalence_group,
                variant_id,
                expected,
                actual,
                leaf,
            } => write!(
                f,
                "text variant `{variant_id}` in group `{equivalence_group}` at leaf `{leaf}` has {actual} parts, expected {expected}"
            ),
        }
    }
}

impl std::error::Error for TextVariantScopeError {}

#[derive(Debug, Default)]
struct LeafGroupState {
    has_default_fallback: bool,
    variants: HashMap<String, VariantPartState>,
}

#[derive(Debug, Default)]
struct VariantPartState {
    expected_part_count: Option<u32>,
    parts: HashSet<u32>,
}

pub fn validate_text_variant_scope(tree: &PageLayerTree) -> Result<(), TextVariantScopeError> {
    let mut group_leaf_paths = HashMap::new();
    validate_node(&tree.root, "root".to_string(), &mut group_leaf_paths)
}

fn validate_node(
    node: &LayerNode,
    path: String,
    group_leaf_paths: &mut HashMap<String, String>,
) -> Result<(), TextVariantScopeError> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for (index, child) in children.iter().enumerate() {
                validate_node(child, format!("{path}/group[{index}]"), group_leaf_paths)?;
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            validate_node(child, format!("{path}/clip"), group_leaf_paths)?;
        }
        LayerNodeKind::Leaf { ops } => {
            validate_leaf(ops, path, group_leaf_paths)?;
        }
    }
    Ok(())
}

fn validate_leaf(
    ops: &[PaintOp],
    leaf_path: String,
    group_leaf_paths: &mut HashMap<String, String>,
) -> Result<(), TextVariantScopeError> {
    let mut groups = HashMap::<String, LeafGroupState>::new();
    let has_text_run_fallback = ops.iter().any(|op| matches!(op, PaintOp::TextRun { .. }));
    for op in ops {
        let Some(variant) = op_variant(op) else {
            continue;
        };
        validate_sidecar_anchor(&variant, &leaf_path)?;
        if let PaintOp::GlyphOutline { outline, .. } = op {
            if !outline.has_exclusive_payload_family() {
                return Err(TextVariantScopeError::MixedGlyphOutlinePayload {
                    equivalence_group: variant.equivalence_group.clone(),
                    variant_id: variant.variant_id.clone(),
                    leaf: leaf_path,
                });
            }
        }
        if let Some(first_leaf) = group_leaf_paths.get(&variant.equivalence_group) {
            if first_leaf != &leaf_path {
                return Err(TextVariantScopeError::CrossLeafGroup {
                    equivalence_group: variant.equivalence_group.clone(),
                    first_leaf: first_leaf.clone(),
                    second_leaf: leaf_path,
                });
            }
        } else {
            group_leaf_paths.insert(variant.equivalence_group.clone(), leaf_path.clone());
        }

        let group = groups.entry(variant.equivalence_group.clone()).or_default();
        group.has_default_fallback |= has_text_run_fallback || variant.is_default_fallback;
        let state = group
            .variants
            .entry(variant.variant_id.clone())
            .or_default();
        match state.expected_part_count {
            Some(expected) if expected != variant.part_count => {
                return Err(TextVariantScopeError::PartCountMismatch {
                    equivalence_group: variant.equivalence_group.clone(),
                    variant_id: variant.variant_id.clone(),
                    expected,
                    actual: variant.part_count,
                    leaf: leaf_path,
                });
            }
            Some(_) => {}
            None => {
                state.expected_part_count = Some(variant.part_count);
            }
        }
        if variant.part_count == 0 {
            return Err(TextVariantScopeError::EmptyVariantSet {
                equivalence_group: variant.equivalence_group.clone(),
                variant_id: variant.variant_id.clone(),
                leaf: leaf_path,
            });
        }
        if !state.parts.insert(variant.part_index) {
            return Err(TextVariantScopeError::DuplicatePart {
                equivalence_group: variant.equivalence_group.clone(),
                variant_id: variant.variant_id.clone(),
                part_index: variant.part_index,
                leaf: leaf_path,
            });
        }
    }

    for (equivalence_group, group) in groups {
        if !group.has_default_fallback {
            return Err(TextVariantScopeError::MissingDefaultFallback {
                equivalence_group,
                leaf: leaf_path,
            });
        }
        for (variant_id, state) in group.variants {
            let expected = state.expected_part_count.unwrap_or_default();
            let actual = state.parts.len() as u32;
            if expected != actual || !(0..expected).all(|index| state.parts.contains(&index)) {
                return Err(TextVariantScopeError::PartCountMismatch {
                    equivalence_group: equivalence_group.clone(),
                    variant_id,
                    expected,
                    actual,
                    leaf: leaf_path,
                });
            }
        }
    }
    Ok(())
}

fn op_variant(op: &PaintOp) -> Option<PaintVariantMeta> {
    match op {
        PaintOp::GlyphRun { run, .. } => Some(run.variant.clone()),
        PaintOp::GlyphOutline { outline, .. } => Some(outline.variant.clone()),
        _ => None,
    }
}

fn validate_sidecar_anchor(
    variant: &PaintVariantMeta,
    leaf_path: &str,
) -> Result<(), TextVariantScopeError> {
    if variant.variant_kind != TextVariantKind::GlyphOutline {
        return Ok(());
    }
    let Some(anchor_op_id) = &variant.anchor_op_id else {
        return Err(TextVariantScopeError::MissingSidecarAnchorOpId {
            equivalence_group: variant.equivalence_group.clone(),
            variant_id: variant.variant_id.clone(),
            leaf: leaf_path.to_string(),
        });
    };
    // Schema v1 does not assign an explicit op id to the fallback TextRun.
    // The equivalence group is the exported paint-order slot id, so P14
    // sidecars anchor to that slot until per-op ids exist.
    if anchor_op_id != &variant.equivalence_group {
        return Err(TextVariantScopeError::InvalidSidecarAnchor {
            equivalence_group: variant.equivalence_group.clone(),
            variant_id: variant.variant_id.clone(),
            anchor_op_id: anchor_op_id.clone(),
            leaf: leaf_path.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::resources::ResourceArena;
    use crate::paint::{
        FontFaceKey, FontFallbackPolicyId, FontInstanceKey, GlyphCluster, GlyphOutlineFillRule,
        GlyphOutlinePaintOrder, GlyphOutlinePayloadKind, GlyphOutlineStrokeCap,
        GlyphOutlineStrokeJoin, GlyphOutlineStrokeStyle, GlyphRange, GlyphRunDiagnostics,
        GlyphRunOrientation, GlyphRunReplayEligibility, LayerAffineTransform,
        LayerGlyphOutlinePaint, LayerGlyphOutlinePath, LayerGlyphRunPaint, LayerNode,
        LayerOutputOptions, LayerPoint, PaintTextStyle, RenderProfile, ScriptTag, ShapeKey,
        ShapingEngineId, TextDirection, TextSourceId, TextSourceRange, TextSourceSpan,
        TextSourceTable, TextVariantKind, TextVariantQuality, WritingMode,
    };
    use crate::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
    use crate::renderer::{PathCommand, TextStyle};

    fn bbox() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 10.0, 10.0)
    }

    fn text_op() -> PaintOp {
        PaintOp::TextRun {
            bbox: bbox(),
            run: TextRunNode {
                text: "A".to_string(),
                style: TextStyle::default(),
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 10.0,
                field_marker: FieldMarkerType::None,
            },
        }
    }

    fn glyph_op(variant: PaintVariantMeta) -> PaintOp {
        PaintOp::GlyphRun {
            bbox: bbox(),
            run: Box::new(LayerGlyphRunPaint {
                source: TextSourceSpan {
                    id: TextSourceId(0),
                    utf8_range: TextSourceRange::new(0, 1),
                    utf16_range: TextSourceRange::new(0, 1),
                    stable_source_key: None,
                },
                variant,
                paint_style: PaintTextStyle::from(&TextStyle::default()),
                shape_key: ShapeKey {
                    font_instance: FontInstanceKey {
                        face_key: FontFaceKey("face-0".to_string()),
                        size_px: 12.0,
                        variations: Vec::new(),
                        synthetic_bold: false,
                        synthetic_italic: false,
                    },
                    direction: TextDirection::Ltr,
                    writing_mode: WritingMode::HorizontalTb,
                    script: Some(ScriptTag("DFLT".to_string())),
                    language: None,
                    features: Vec::new(),
                    shaping_engine: ShapingEngineId("test".to_string()),
                    fallback_policy: FontFallbackPolicyId("none".to_string()),
                },
                placement: crate::paint::TextRunPlacement {
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
                glyph_ids: vec![42],
                positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
                advances: None,
                clusters: vec![GlyphCluster {
                    source_range_utf8: TextSourceRange::new(0, 1),
                    source_range_utf16: Some(TextSourceRange::new(0, 1)),
                    text_range_utf8: Some(TextSourceRange::new(0, 1)),
                    glyph_range: GlyphRange::new(0, 1),
                    flags: Vec::new(),
                }],
                direction: TextDirection::Ltr,
                bidi_level: None,
                writing_mode: WritingMode::HorizontalTb,
                orientation: GlyphRunOrientation::Horizontal,
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
                    reason: None,
                },
            }),
        }
    }

    fn glyph_outline_op(variant: PaintVariantMeta) -> PaintOp {
        PaintOp::GlyphOutline {
            bbox: bbox(),
            outline: Box::new(LayerGlyphOutlinePaint {
                source: TextSourceSpan {
                    id: TextSourceId(0),
                    utf8_range: TextSourceRange::new(0, 1),
                    utf16_range: TextSourceRange::new(0, 1),
                    stable_source_key: None,
                },
                variant,
                payload_kind: GlyphOutlinePayloadKind::MonochromeFill,
                color_layers: None,
                bitmap_glyph: None,
                svg_glyph: None,
                paint_style: PaintTextStyle::from(&TextStyle::default()),
                placement: crate::paint::TextRunPlacement {
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
                paths: vec![LayerGlyphOutlinePath {
                    glyph_id: 42,
                    source_range_utf8: TextSourceRange::new(0, 1),
                    glyph_range: GlyphRange::new(0, 1),
                    commands: vec![
                        PathCommand::MoveTo(0.0, 0.0),
                        PathCommand::LineTo(8.0, 0.0),
                        PathCommand::ClosePath,
                    ],
                    fill_rule: GlyphOutlineFillRule::NonZero,
                }],
                stroke: None,
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
                    reason: None,
                },
            }),
        }
    }

    fn tree(root: LayerNode) -> PageLayerTree {
        PageLayerTree {
            page_width: 100.0,
            page_height: 100.0,
            profile: RenderProfile::default(),
            output_options: LayerOutputOptions::default(),
            root,
            resources: ResourceArena::default(),
            text_sources: TextSourceTable::default(),
        }
    }

    #[test]
    fn accepts_variant_set_inside_one_leaf() {
        let glyph_part_0 = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphRun".to_string(),
            variant_kind: TextVariantKind::GlyphRun,
            part_index: 0,
            part_count: 2,
            is_default_fallback: false,
            requires: vec!["fontResources".to_string(), "text.glyphRun".to_string()],
            quality: None,
            anchor_op_id: None,
            local_paint_order: None,
        };
        let mut glyph_part_1 = glyph_part_0.clone();
        glyph_part_1.part_index = 1;
        let tree = tree(LayerNode::leaf(
            bbox(),
            None,
            vec![text_op(), glyph_op(glyph_part_0), glyph_op(glyph_part_1)],
        ));
        validate_text_variant_scope(&tree).unwrap();
    }

    #[test]
    fn rejects_cross_leaf_variant_group() {
        let tree = tree(LayerNode::group(
            bbox(),
            None,
            vec![
                LayerNode::leaf(
                    bbox(),
                    None,
                    vec![glyph_op(PaintVariantMeta::text_run_default("text-1"))],
                ),
                LayerNode::leaf(
                    bbox(),
                    None,
                    vec![glyph_op(PaintVariantMeta::text_run_default("text-1"))],
                ),
            ],
            crate::paint::CacheHint::None,
            crate::paint::GroupKind::Generic,
        ));
        assert!(matches!(
            validate_text_variant_scope(&tree),
            Err(TextVariantScopeError::CrossLeafGroup { .. })
        ));
    }

    #[test]
    fn rejects_incomplete_variant_parts() {
        let glyph_part = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphRun".to_string(),
            variant_kind: TextVariantKind::GlyphRun,
            part_index: 0,
            part_count: 2,
            is_default_fallback: false,
            requires: Vec::new(),
            quality: None,
            anchor_op_id: None,
            local_paint_order: None,
        };
        let tree = tree(LayerNode::leaf(
            bbox(),
            None,
            vec![text_op(), glyph_op(glyph_part)],
        ));
        assert!(matches!(
            validate_text_variant_scope(&tree),
            Err(TextVariantScopeError::PartCountMismatch { .. })
        ));
    }

    #[test]
    fn rejects_variant_group_without_default_fallback() {
        let glyph_part = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphRun".to_string(),
            variant_kind: TextVariantKind::GlyphRun,
            part_index: 0,
            part_count: 1,
            is_default_fallback: false,
            requires: Vec::new(),
            quality: None,
            anchor_op_id: None,
            local_paint_order: None,
        };
        let tree = tree(LayerNode::leaf(bbox(), None, vec![glyph_op(glyph_part)]));
        assert!(matches!(
            validate_text_variant_scope(&tree),
            Err(TextVariantScopeError::MissingDefaultFallback { .. })
        ));
    }

    #[test]
    fn accepts_glyph_outline_sidecar_anchored_to_same_slot() {
        let mut outline = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphOutline".to_string(),
            variant_kind: TextVariantKind::GlyphOutline,
            part_index: 0,
            part_count: 1,
            is_default_fallback: false,
            requires: vec!["text.glyphOutline".to_string()],
            quality: Some(TextVariantQuality::Exact),
            anchor_op_id: Some("text-1".to_string()),
            local_paint_order: Some(0),
        };
        let single_part_tree = tree(LayerNode::leaf(
            bbox(),
            None,
            vec![text_op(), glyph_outline_op(outline.clone())],
        ));
        validate_text_variant_scope(&single_part_tree).unwrap();

        outline.part_count = 2;
        let mut part_1 = outline.clone();
        part_1.part_index = 1;
        let multipart_tree = tree(LayerNode::leaf(
            bbox(),
            None,
            vec![
                text_op(),
                glyph_outline_op(outline),
                glyph_outline_op(part_1),
            ],
        ));
        validate_text_variant_scope(&multipart_tree).unwrap();
    }

    #[test]
    fn rejects_glyph_outline_without_anchor() {
        let outline = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphOutline".to_string(),
            variant_kind: TextVariantKind::GlyphOutline,
            part_index: 0,
            part_count: 1,
            is_default_fallback: false,
            requires: vec!["text.glyphOutline".to_string()],
            quality: Some(TextVariantQuality::Exact),
            anchor_op_id: None,
            local_paint_order: None,
        };
        let tree = tree(LayerNode::leaf(
            bbox(),
            None,
            vec![text_op(), glyph_outline_op(outline)],
        ));
        assert!(matches!(
            validate_text_variant_scope(&tree),
            Err(TextVariantScopeError::MissingSidecarAnchorOpId { .. })
        ));
    }

    #[test]
    fn rejects_glyph_outline_anchored_to_different_slot() {
        let outline = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphOutline".to_string(),
            variant_kind: TextVariantKind::GlyphOutline,
            part_index: 0,
            part_count: 1,
            is_default_fallback: false,
            requires: vec!["text.glyphOutline".to_string()],
            quality: Some(TextVariantQuality::Exact),
            anchor_op_id: Some("text-2".to_string()),
            local_paint_order: None,
        };
        let tree = tree(LayerNode::leaf(
            bbox(),
            None,
            vec![text_op(), glyph_outline_op(outline)],
        ));
        assert!(matches!(
            validate_text_variant_scope(&tree),
            Err(TextVariantScopeError::InvalidSidecarAnchor { .. })
        ));
    }

    #[test]
    fn rejects_mixed_glyph_outline_payload_family() {
        let outline = PaintVariantMeta {
            equivalence_group: "text-1".to_string(),
            variant_id: "glyphOutline".to_string(),
            variant_kind: TextVariantKind::GlyphOutline,
            part_index: 0,
            part_count: 1,
            is_default_fallback: false,
            requires: vec!["text.glyphOutline".to_string()],
            quality: Some(TextVariantQuality::Exact),
            anchor_op_id: Some("text-1".to_string()),
            local_paint_order: None,
        };
        let mut op = glyph_outline_op(outline);
        if let PaintOp::GlyphOutline { outline, .. } = &mut op {
            outline.stroke = Some(GlyphOutlineStrokeStyle {
                color: 0x00000000,
                width: 1.0,
                join: GlyphOutlineStrokeJoin::Miter,
                cap: GlyphOutlineStrokeCap::Butt,
                miter_limit: 2.0,
                paint_order: GlyphOutlinePaintOrder::FillThenStroke,
            });
        }
        let tree = tree(LayerNode::leaf(bbox(), None, vec![text_op(), op]));
        assert!(matches!(
            validate_text_variant_scope(&tree),
            Err(TextVariantScopeError::MixedGlyphOutlinePayload { .. })
        ));
    }
}
