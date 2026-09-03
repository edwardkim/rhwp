//! Text variant grouping validation.
//!
//! Schema v1 keeps `TextRun` as the root fallback op and attaches optional
//! visual alternatives such as `GlyphRun` through variant metadata. Consumers
//! choose one variant set per equivalence group.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::paint::{
    LayerNode, LayerNodeKind, PageLayerTree, PaintOp, PaintVariantMeta, TextDecorationKind,
    TextVariantKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVisualReplayRole {
    BaseText,
    SuppressedFallback,
    CharOverlap,
    ControlMark,
    TabLeader,
    Decoration(TextDecorationKind),
    Other,
}

/// Classifies canonical text ops without translating them into backend primitives.
pub fn text_visual_replay_role(op: &PaintOp) -> TextVisualReplayRole {
    match op {
        PaintOp::TextRun { run, .. } if run.char_overlap.is_some() => {
            TextVisualReplayRole::SuppressedFallback
        }
        PaintOp::TextRun { .. } => TextVisualReplayRole::BaseText,
        PaintOp::CharOverlap { .. } => TextVisualReplayRole::CharOverlap,
        PaintOp::TextControlMark { .. } => TextVisualReplayRole::ControlMark,
        PaintOp::TabLeader { .. } => TextVisualReplayRole::TabLeader,
        PaintOp::TextDecoration { kind, .. } => TextVisualReplayRole::Decoration(*kind),
        _ => TextVisualReplayRole::Other,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextVariantScopeError {
    InvalidExplicitTextVisual {
        visual: String,
        matches: usize,
        leaf: String,
    },
    OrphanExplicitTextVisual {
        visual: String,
        matching_runs: usize,
        leaf: String,
    },
    MissingTextSourceBinding {
        leaf: String,
    },
    DuplicateTextSourceId {
        source_id: u32,
        first_leaf: String,
        second_leaf: String,
    },
    TextSourceTableCardinality {
        bound_sources: usize,
        table_entries: usize,
    },
    InvalidTextSourceEntry {
        source_id: u32,
        matches: usize,
        leaf: String,
    },
    CrossLeafGroup {
        equivalence_group: String,
        first_leaf: String,
        second_leaf: String,
    },
    MissingDefaultFallback {
        equivalence_group: String,
        leaf: String,
    },
    SourceTableMismatch {
        equivalence_group: String,
        source_id: u32,
        leaf: String,
    },
    InvalidSourceFallback {
        equivalence_group: String,
        source_id: u32,
        matches: usize,
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
            Self::InvalidExplicitTextVisual {
                visual,
                matches,
                leaf,
            } => write!(
                f,
                "TextRun in leaf `{leaf}` requires exactly one explicit canonical {visual} paint, found {matches}"
            ),
            Self::OrphanExplicitTextVisual {
                visual,
                matching_runs,
                leaf,
            } => write!(
                f,
                "explicit canonical {visual} paint in leaf `{leaf}` belongs to {matching_runs} TextRuns"
            ),
            Self::MissingTextSourceBinding { leaf } => {
                write!(f, "TextRun in leaf `{leaf}` has no canonical source binding")
            }
            Self::DuplicateTextSourceId {
                source_id,
                first_leaf,
                second_leaf,
            } => write!(
                f,
                "canonical text source {source_id} is bound in both `{first_leaf}` and `{second_leaf}`"
            ),
            Self::TextSourceTableCardinality {
                bound_sources,
                table_entries,
            } => write!(
                f,
                "PageLayerTree has {bound_sources} bound TextRuns but {table_entries} canonical text source entries"
            ),
            Self::InvalidTextSourceEntry {
                source_id,
                matches,
                leaf,
            } => write!(
                f,
                "TextRun source {source_id} in leaf `{leaf}` requires one exact canonical table entry, found {matches}"
            ),
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
            Self::SourceTableMismatch {
                equivalence_group,
                source_id,
                leaf,
            } => write!(
                f,
                "text variant group `{equivalence_group}` in leaf `{leaf}` does not match canonical text source {source_id}"
            ),
            Self::InvalidSourceFallback {
                equivalence_group,
                source_id,
                matches,
                leaf,
            } => write!(
                f,
                "text variant group `{equivalence_group}` in leaf `{leaf}` requires one TextRun fallback for source {source_id}, found {matches}"
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
    validate_text_source_identity(tree)?;
    let mut group_leaf_paths = HashMap::new();
    validate_node(&tree.root, "root".to_string(), &mut group_leaf_paths)?;
    validate_source_node(&tree.root, "root".to_string(), &tree.text_sources)?;
    validate_text_visual_node(
        &tree.root,
        "root".to_string(),
        tree.output_options.show_paragraph_marks || tree.output_options.show_control_codes,
    )
}

fn validate_text_visual_node(
    node: &LayerNode,
    path: String,
    show_text_marks: bool,
) -> Result<(), TextVariantScopeError> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for (index, child) in children.iter().enumerate() {
                validate_text_visual_node(
                    child,
                    format!("{path}/group[{index}]"),
                    show_text_marks,
                )?;
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            validate_text_visual_node(child, format!("{path}/clip"), show_text_marks)?;
        }
        LayerNodeKind::Leaf { ops } => {
            let text_runs = ops
                .iter()
                .filter_map(|op| match op {
                    PaintOp::TextRun {
                        bbox, run, source, ..
                    } => Some((bbox, run.as_ref(), source.as_ref())),
                    _ => None,
                })
                .collect::<Vec<_>>();
            for op in ops {
                let PaintOp::TextRun {
                    bbox, run, source, ..
                } = op
                else {
                    continue;
                };
                let source = source
                    .as_ref()
                    .expect("text source identity was validated above");
                for role in required_text_visual_roles(run, show_text_marks) {
                    let matches = ops
                        .iter()
                        .filter(|candidate| visual_matches_run(candidate, bbox, run, source, role))
                        .count();
                    if matches != 1 {
                        return Err(TextVariantScopeError::InvalidExplicitTextVisual {
                            visual: text_visual_name(role).to_string(),
                            matches,
                            leaf: path.clone(),
                        });
                    }
                }
            }
            for op in ops {
                let role = text_visual_replay_role(op);
                if !matches!(
                    role,
                    TextVisualReplayRole::CharOverlap
                        | TextVisualReplayRole::ControlMark
                        | TextVisualReplayRole::TabLeader
                        | TextVisualReplayRole::Decoration(_)
                ) {
                    continue;
                }
                let matching_runs = text_runs
                    .iter()
                    .filter(|(bbox, run, source)| {
                        source.is_some_and(|source| visual_matches_run(op, bbox, run, source, role))
                    })
                    .count();
                let required = text_runs.iter().any(|(bbox, run, source)| {
                    source.is_some_and(|source| {
                        visual_matches_run(op, bbox, run, source, role)
                            && required_text_visual_roles(run, show_text_marks).contains(&role)
                    })
                });
                if matching_runs != 1 || !required {
                    return Err(TextVariantScopeError::OrphanExplicitTextVisual {
                        visual: text_visual_name(role).to_string(),
                        matching_runs,
                        leaf: path.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn required_text_visual_roles(
    run: &crate::renderer::render_tree::TextRunNode,
    show_text_marks: bool,
) -> Vec<TextVisualReplayRole> {
    let mut required = if run.char_overlap.is_some() {
        vec![TextVisualReplayRole::CharOverlap]
    } else {
        let mut required = Vec::new();
        if !run.style.tab_leaders.is_empty() {
            required.push(TextVisualReplayRole::TabLeader);
        }
        if !matches!(
            run.style.underline,
            crate::model::style::UnderlineType::None
        ) {
            required.push(TextVisualReplayRole::Decoration(
                TextDecorationKind::Underline,
            ));
        }
        if run.style.strikethrough {
            required.push(TextVisualReplayRole::Decoration(
                TextDecorationKind::Strikethrough,
            ));
        }
        if run.style.emphasis_dot > 0 {
            required.push(TextVisualReplayRole::Decoration(
                TextDecorationKind::EmphasisDot,
            ));
        }
        required
    };
    if show_text_marks
        && (run.is_para_end
            || run.is_line_break_end
            || run
                .text
                .chars()
                .any(|character| matches!(character, ' ' | '\t')))
    {
        required.push(TextVisualReplayRole::ControlMark);
    }
    required
}

fn visual_matches_run(
    visual: &PaintOp,
    bbox: &crate::renderer::render_tree::BoundingBox,
    run: &crate::renderer::render_tree::TextRunNode,
    source: &crate::paint::TextSourceSpan,
    role: TextVisualReplayRole,
) -> bool {
    let candidate = visual.bounds();
    if candidate.x != bbox.x
        || candidate.y != bbox.y
        || candidate.width != bbox.width
        || candidate.height != bbox.height
        || text_visual_replay_role(visual) != role
    {
        return false;
    }
    match visual {
        PaintOp::CharOverlap {
            run: candidate_run,
            source: candidate_source,
            ..
        }
        | PaintOp::TextControlMark {
            run: candidate_run,
            source: candidate_source,
            ..
        }
        | PaintOp::TabLeader {
            run: candidate_run,
            source: candidate_source,
            ..
        }
        | PaintOp::TextDecoration {
            run: candidate_run,
            source: candidate_source,
            ..
        } => candidate_run.as_ref() == run && candidate_source.as_ref() == Some(source),
        _ => false,
    }
}

fn text_visual_name(role: TextVisualReplayRole) -> &'static str {
    match role {
        TextVisualReplayRole::CharOverlap => "CharOverlap",
        TextVisualReplayRole::ControlMark => "control-mark",
        TextVisualReplayRole::TabLeader => "TabLeader",
        TextVisualReplayRole::Decoration(TextDecorationKind::Underline) => "underline",
        TextVisualReplayRole::Decoration(TextDecorationKind::Strikethrough) => "strikethrough",
        TextVisualReplayRole::Decoration(TextDecorationKind::EmphasisDot) => "emphasis-dot",
        TextVisualReplayRole::BaseText
        | TextVisualReplayRole::SuppressedFallback
        | TextVisualReplayRole::Other => "non-visual",
    }
}

fn validate_text_source_identity(tree: &PageLayerTree) -> Result<(), TextVariantScopeError> {
    let mut runs = Vec::new();
    collect_text_runs(&tree.root, "root".to_string(), &mut runs);
    let mut source_paths = HashMap::new();
    for (leaf, _, source) in &runs {
        let Some(source) = source else {
            return Err(TextVariantScopeError::MissingTextSourceBinding { leaf: leaf.clone() });
        };
        if let Some(first_leaf) = source_paths.insert(source.id.0, leaf.clone()) {
            return Err(TextVariantScopeError::DuplicateTextSourceId {
                source_id: source.id.0,
                first_leaf,
                second_leaf: leaf.clone(),
            });
        }
    }
    if runs.len() != tree.text_sources.entries.len() {
        return Err(TextVariantScopeError::TextSourceTableCardinality {
            bound_sources: runs.len(),
            table_entries: tree.text_sources.entries.len(),
        });
    }
    for (leaf, run, source) in runs {
        let source = source
            .as_ref()
            .expect("missing bindings were rejected above");
        let matches = tree
            .text_sources
            .entries
            .iter()
            .filter(|entry| entry.matches_text_run(source, run))
            .count();
        if matches != 1 {
            return Err(TextVariantScopeError::InvalidTextSourceEntry {
                source_id: source.id.0,
                matches,
                leaf,
            });
        }
    }
    Ok(())
}

fn collect_text_runs<'a>(
    node: &'a LayerNode,
    path: String,
    runs: &mut Vec<(
        String,
        &'a crate::renderer::render_tree::TextRunNode,
        &'a Option<crate::paint::TextSourceSpan>,
    )>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for (index, child) in children.iter().enumerate() {
                collect_text_runs(child, format!("{path}/group[{index}]"), runs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            collect_text_runs(child, format!("{path}/clip"), runs);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { run, source, .. } = op {
                    runs.push((path.clone(), run, source));
                }
            }
        }
    }
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

fn op_source(op: &PaintOp) -> Option<&crate::paint::TextSourceSpan> {
    match op {
        PaintOp::GlyphRun { run, .. } => Some(&run.source),
        PaintOp::GlyphOutline { outline, .. } => Some(&outline.source),
        _ => None,
    }
}

fn validate_source_node(
    node: &LayerNode,
    path: String,
    text_sources: &crate::paint::TextSourceTable,
) -> Result<(), TextVariantScopeError> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for (index, child) in children.iter().enumerate() {
                validate_source_node(child, format!("{path}/group[{index}]"), text_sources)?;
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            validate_source_node(child, format!("{path}/clip"), text_sources)?;
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                let Some(variant) = op_variant(op) else {
                    continue;
                };
                let source = op_source(op).expect("variant ops always carry source identity");
                validate_source_fallback(ops, source, &variant, &path, text_sources)?;
            }
        }
    }
    Ok(())
}

fn validate_source_fallback(
    ops: &[PaintOp],
    source: &crate::paint::TextSourceSpan,
    variant: &PaintVariantMeta,
    leaf_path: &str,
    text_sources: &crate::paint::TextSourceTable,
) -> Result<(), TextVariantScopeError> {
    let Some(entry) = text_sources
        .entries
        .iter()
        .find(|entry| entry.id == source.id)
    else {
        return Err(TextVariantScopeError::SourceTableMismatch {
            equivalence_group: variant.equivalence_group.clone(),
            source_id: source.id.0,
            leaf: leaf_path.to_string(),
        });
    };
    if source.utf8_range != entry.utf8_range
        || source.utf16_range != entry.utf16_range
        || !source
            .stable_source_key
            .as_ref()
            .is_none_or(|key| entry.stable_source_key.as_ref() == Some(key))
    {
        return Err(TextVariantScopeError::SourceTableMismatch {
            equivalence_group: variant.equivalence_group.clone(),
            source_id: source.id.0,
            leaf: leaf_path.to_string(),
        });
    }
    let matches = ops
        .iter()
        .filter(|candidate| {
            let PaintOp::TextRun {
                run,
                source: fallback_source,
                ..
            } = candidate
            else {
                return false;
            };
            fallback_source.as_ref() == Some(source)
                && run.text == entry.text
                && crate::paint::layer_tree::stable_text_source_key(run) == entry.stable_source_key
        })
        .count();
    if matches != 1 {
        return Err(TextVariantScopeError::InvalidSourceFallback {
            equivalence_group: variant.equivalence_group.clone(),
            source_id: source.id.0,
            matches,
            leaf: leaf_path.to_string(),
        });
    }
    Ok(())
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
    use crate::paint::{
        FontFaceKey, FontFallbackPolicyId, FontInstanceKey, GlyphCluster, GlyphOutlineFillRule,
        GlyphOutlinePaintOrder, GlyphOutlinePayloadKind, GlyphOutlineStrokeCap,
        GlyphOutlineStrokeJoin, GlyphOutlineStrokeStyle, GlyphRange, GlyphRunDiagnostics,
        GlyphRunOrientation, GlyphRunReplayEligibility, LayerAffineTransform,
        LayerGlyphOutlinePaint, LayerGlyphOutlinePath, LayerGlyphRunPaint, LayerNode, LayerPoint,
        PaintTextStyle, ScriptTag, ShapeKey, ShapingEngineId, TextDirection, TextSourceId,
        TextSourceRange, TextSourceSpan, TextVariantKind, TextVariantQuality, WritingMode,
    };
    use crate::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
    use crate::renderer::{PathCommand, TextStyle};

    fn bbox() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 10.0, 10.0)
    }

    fn text_op() -> PaintOp {
        PaintOp::text_run(
            bbox(),
            TextRunNode {
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
                layout_positions: None,
                display_text: None,
            },
        )
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
        PageLayerTree::new(100.0, 100.0, root)
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
