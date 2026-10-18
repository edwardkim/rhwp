//! Borrowed, validated Text IR v2 snapshots over the current PaintOp model.
//!
//! The checked JSON exporter owns grouping and validation. These views retain
//! the original Rust payloads rather than reconstructing them from rounded JSON.
//! Views are read-only: edit the source tree and lower it again to revalidate.
//! Ancestor layers, non-text visual ops and resources remain in that source tree.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::paint::{
    LayerNodeKind, PageLayerTree, PaintOp, PaintVariantMeta, TextV2ValidationIssue,
    TextV2ValidationSeverity, TextVariantKind, TextVariantQuality,
};
use crate::renderer::render_tree::BoundingBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFallbackPolicy {
    Required,
    None,
}

/// One validated text slot. The source tree must outlive this snapshot.
#[derive(Debug, Clone)]
pub struct LayerTextPaintOpV2<'a> {
    id: String,
    bbox: BoundingBox,
    default_variant_id: String,
    fallback_policy: TextFallbackPolicy,
    variants: Vec<LayerTextVariantSet<'a>>,
}

impl<'a> LayerTextPaintOpV2<'a> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn paint_order_slot_id(&self) -> &str {
        &self.id
    }

    pub fn bbox(&self) -> BoundingBox {
        self.bbox
    }

    pub fn default_variant_id(&self) -> &str {
        &self.default_variant_id
    }

    pub fn fallback_policy(&self) -> TextFallbackPolicy {
        self.fallback_policy
    }

    pub fn variants(&self) -> &[LayerTextVariantSet<'a>] {
        &self.variants
    }
}

#[derive(Debug, Clone)]
pub struct LayerTextVariantSet<'a> {
    variant_id: String,
    kind: TextVariantKind,
    required_features: Vec<String>,
    quality: Option<TextVariantQuality>,
    parts: Vec<LayerTextVariantPart<'a>>,
}

impl<'a> LayerTextVariantSet<'a> {
    pub fn variant_id(&self) -> &str {
        &self.variant_id
    }

    pub fn kind(&self) -> TextVariantKind {
        self.kind
    }

    pub fn required_features(&self) -> &[String] {
        &self.required_features
    }

    pub fn quality(&self) -> Option<TextVariantQuality> {
        self.quality
    }

    pub fn parts(&self) -> &[LayerTextVariantPart<'a>] {
        &self.parts
    }
}

#[derive(Debug, Clone)]
pub struct LayerTextVariantPart<'a> {
    payload: &'a PaintOp,
}

impl<'a> LayerTextVariantPart<'a> {
    pub fn part_index(&self) -> u32 {
        variant_meta(self.payload).map_or(0, |meta| meta.part_index)
    }

    pub fn part_count(&self) -> u32 {
        variant_meta(self.payload).map_or(1, |meta| meta.part_count)
    }

    pub fn local_paint_order(&self) -> Option<u32> {
        variant_meta(self.payload).and_then(|meta| meta.local_paint_order)
    }

    pub fn bbox(&self) -> BoundingBox {
        self.payload.bounds()
    }

    pub fn payload(&self) -> &'a PaintOp {
        self.payload
    }
}

impl PageLayerTree {
    /// Checked compatibility slots in their fallback paint-order positions.
    pub fn text_v2_slots(&self) -> Result<Vec<LayerTextPaintOpV2<'_>>, Vec<TextV2ValidationIssue>> {
        self.typed_text_v2_slots(self.to_json_v2_compat()?)
    }

    /// The same validation used by the opt-in compatibility JSON writer.
    pub fn validate_text_v2_slots(&self) -> Vec<TextV2ValidationIssue> {
        self.to_json_v2_compat().err().unwrap_or_default()
    }

    /// Fallback-free snapshots require the same resource proof as strict JSON.
    pub fn text_v2_slots_strict_glyph_outline(
        &self,
    ) -> Result<Vec<LayerTextPaintOpV2<'_>>, Vec<TextV2ValidationIssue>> {
        self.typed_text_v2_slots(self.to_json_v2_strict_glyph_outline()?)
    }

    pub fn text_v2_slots_strict_glyph_run(
        &self,
    ) -> Result<Vec<LayerTextPaintOpV2<'_>>, Vec<TextV2ValidationIssue>> {
        self.typed_text_v2_slots(self.to_json_v2_strict_glyph_run()?)
    }

    fn typed_text_v2_slots(
        &self,
        json: String,
    ) -> Result<Vec<LayerTextPaintOpV2<'_>>, Vec<TextV2ValidationIssue>> {
        let value: Value = serde_json::from_str(&json).map_err(|_| projection_issue())?;
        let mut payloads = BTreeMap::new();
        let mut nodes = vec![&self.root];
        while let Some(node) = nodes.pop() {
            match &node.kind {
                LayerNodeKind::Group { children, .. } => nodes.extend(children),
                LayerNodeKind::ClipRect { child, .. } => nodes.push(child),
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        if let Some(key) = payload_key(op) {
                            payloads.insert(key, op);
                        }
                    }
                }
            }
        }
        let mut slots = Vec::new();
        let mut nodes = vec![&value["root"]];
        while let Some(node) = nodes.pop() {
            match node["kind"].as_str() {
                Some("group") => {
                    nodes.extend(
                        node["children"]
                            .as_array()
                            .ok_or_else(projection_issue)?
                            .iter()
                            .rev(),
                    );
                }
                Some("clipRect") => nodes.push(&node["child"]),
                Some("leaf") => {
                    for slot in node["ops"].as_array().ok_or_else(projection_issue)? {
                        if slot["type"] != "text" {
                            continue;
                        }
                        let mut variants = Vec::new();
                        let mut fallback_bbox = None;
                        for variant in slot["variants"].as_array().ok_or_else(projection_issue)? {
                            let variant_id =
                                variant["variantId"].as_str().ok_or_else(projection_issue)?;
                            let mut parts = Vec::new();
                            for part in variant["parts"].as_array().ok_or_else(projection_issue)? {
                                let source_id = part["payload"]["source"]["id"]
                                    .as_u64()
                                    .and_then(|id| u32::try_from(id).ok())
                                    .ok_or_else(projection_issue)?;
                                let part_index = part["partIndex"]
                                    .as_u64()
                                    .and_then(|index| u32::try_from(index).ok())
                                    .ok_or_else(projection_issue)?;
                                let payload = payloads
                                    .get(&(source_id, variant_id, part_index))
                                    .copied()
                                    .ok_or_else(projection_issue)?;
                                fallback_bbox = payloads
                                    .get(&(source_id, "textRun", 0))
                                    .map(|op| op.bounds());
                                parts.push(LayerTextVariantPart { payload });
                            }
                            let first = parts.first().ok_or_else(projection_issue)?.payload;
                            let kind = match first {
                                PaintOp::TextRun { .. } => TextVariantKind::TextRun,
                                PaintOp::GlyphRun { .. } => TextVariantKind::GlyphRun,
                                PaintOp::GlyphOutline { .. } => TextVariantKind::GlyphOutline,
                                _ => return Err(projection_issue()),
                            };
                            let required_features = variant["requiredFeatures"]
                                .as_array()
                                .ok_or_else(projection_issue)?
                                .iter()
                                .map(|feature| {
                                    feature
                                        .as_str()
                                        .map(str::to_string)
                                        .ok_or_else(projection_issue)
                                })
                                .collect::<Result<_, _>>()?;
                            variants.push(LayerTextVariantSet {
                                variant_id: variant_id.to_string(),
                                kind,
                                required_features,
                                quality: variant_meta(first).and_then(|meta| meta.quality),
                                parts,
                            });
                        }
                        slots.push(LayerTextPaintOpV2 {
                            id: slot["id"]
                                .as_str()
                                .ok_or_else(projection_issue)?
                                .to_string(),
                            bbox: fallback_bbox.ok_or_else(projection_issue)?,
                            default_variant_id: slot["defaultVariantId"]
                                .as_str()
                                .ok_or_else(projection_issue)?
                                .to_string(),
                            fallback_policy: match slot["fallbackPolicy"].as_str() {
                                Some("required") => TextFallbackPolicy::Required,
                                Some("none") => TextFallbackPolicy::None,
                                _ => return Err(projection_issue()),
                            },
                            variants,
                        });
                    }
                }
                _ => return Err(projection_issue()),
            }
        }
        Ok(slots)
    }
}

/// Returns this slot's original v1 text payloads, with each variant's parts in
/// local paint order. Keep the original ancestor layers, resources, source table
/// and separate non-text visual ops; this does not reconstruct a whole page.
pub fn downgrade_text_v2_op_to_v1_compat(
    slot: &LayerTextPaintOpV2<'_>,
) -> Result<Vec<PaintOp>, Vec<TextV2ValidationIssue>> {
    if slot.fallback_policy != TextFallbackPolicy::Required {
        return Err(vec![TextV2ValidationIssue {
            severity: TextV2ValidationSeverity::Error,
            code: "fallbackRequiredTextRunMissing",
            slot_id: Some(slot.id.clone()),
            leaf_path: None,
            message: "fallback-free text cannot be downgraded without reconstructing TextRun"
                .to_string(),
        }]);
    }
    let mut ops = Vec::new();
    for variant in &slot.variants {
        let mut parts = variant.parts.iter().collect::<Vec<_>>();
        parts.sort_by_key(|part| {
            (
                part.local_paint_order().unwrap_or(part.part_index()),
                part.part_index(),
            )
        });
        ops.extend(parts.into_iter().map(|part| part.payload.clone()));
    }
    Ok(ops)
}

fn variant_meta(op: &PaintOp) -> Option<&PaintVariantMeta> {
    match op {
        PaintOp::GlyphRun { run, .. } => Some(&run.variant),
        PaintOp::GlyphOutline { outline, .. } => Some(&outline.variant),
        _ => None,
    }
}

fn payload_key(op: &PaintOp) -> Option<(u32, &str, u32)> {
    let source = match op {
        PaintOp::TextRun {
            source: Some(source),
            ..
        } => source,
        PaintOp::GlyphRun { run, .. } => &run.source,
        PaintOp::GlyphOutline { outline, .. } => &outline.source,
        _ => return None,
    };
    let meta = variant_meta(op);
    Some((
        source.id.0,
        meta.map_or("textRun", |meta| meta.variant_id.as_str()),
        meta.map_or(0, |meta| meta.part_index),
    ))
}

fn projection_issue() -> Vec<TextV2ValidationIssue> {
    vec![TextV2ValidationIssue {
        severity: TextV2ValidationSeverity::Error,
        code: "invalidTextV2Projection",
        slot_id: None,
        leaf_path: None,
        message: "checked text envelope does not map to its canonical PaintOp payload".to_string(),
    }]
}
