//! Opt-in schema-v2 envelopes over the canonical schema-v1 paint projection.
//!
//! Reusing the v1 writer preserves source binding, explicit visual mirrors,
//! resource keys and output options. The public/default writer is unchanged.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::paint::{
    LayerNodeKind, PageLayerTree, PaintOp, TextV2CompatibilityProfile, TextV2Diagnostics,
    TextV2ValidationIssue, TextV2ValidationSeverity, TextVariantKind,
};
use crate::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, TextVariantSelectionReport,
    VariantSelectionBackend,
};
use crate::schema_registry::TEXT_IR_V2_ENVELOPE_SCHEMA_VERSION;

impl PageLayerTree {
    /// One text envelope per anchored paint slot, including its TextRun fallback.
    pub fn to_json_v2_compat(&self) -> Result<String, Vec<TextV2ValidationIssue>> {
        self.write_text_v2_export(None)
    }

    /// Fallback-free outline envelopes. Every text slot must have a complete,
    /// resource-proven outline variant accepted by the shared replay contract.
    pub fn to_json_v2_strict_glyph_outline(&self) -> Result<String, Vec<TextV2ValidationIssue>> {
        self.write_text_v2_export(Some(TextVariantKind::GlyphOutline))
    }

    /// Fallback-free exact-font glyph envelopes. Unsupported font, geometry or
    /// style contracts fail the whole export instead of silently omitting text.
    pub fn to_json_v2_strict_glyph_run(&self) -> Result<String, Vec<TextV2ValidationIssue>> {
        self.write_text_v2_export(Some(TextVariantKind::GlyphRun))
    }

    fn write_text_v2_export(
        &self,
        strict_kind: Option<TextVariantKind>,
    ) -> Result<String, Vec<TextV2ValidationIssue>> {
        // Public PaintOp construction can violate metadata invariants. Reject
        // that before the shared selector reaches its internal exhaustive kind
        // match (which assumes validated producer output).
        let mut nodes = vec![&self.root];
        while let Some(node) = nodes.pop() {
            match &node.kind {
                LayerNodeKind::Group { children, .. } => nodes.extend(children),
                LayerNodeKind::ClipRect { child, .. } => nodes.push(child),
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        let (variant, expected) = match op {
                            PaintOp::GlyphRun { run, .. } => {
                                (&run.variant, TextVariantKind::GlyphRun)
                            }
                            PaintOp::GlyphOutline { outline, .. } => {
                                (&outline.variant, TextVariantKind::GlyphOutline)
                            }
                            _ => continue,
                        };
                        if variant.variant_kind != expected
                            || variant.is_default_fallback
                            || variant.variant_id.is_empty()
                            || variant.equivalence_group.is_empty()
                        {
                            return Err(vec![issue(
                                "variantKindMismatch",
                                Some(variant.equivalence_group.clone()),
                                "root",
                                "glyph sidecar metadata does not describe its payload kind"
                                    .to_string(),
                            )]);
                        }
                    }
                }
            }
        }
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            self,
            TextV2CompatibilityProfile::V2Compat,
        );
        if diagnostics.has_errors() {
            return Err(diagnostics.validation_issues);
        }
        // Selection uses the same portable resource/geometry guards as browser
        // replay, not the optimistic diagnostics-only availability bit. This
        // includes bounded TTC/variable/vertical capabilities when implemented.
        let reports = strict_kind
            .map(|kind| {
                analyze_text_variant_selection(
                    self,
                    TextVariantSelectionOptions {
                        backend: VariantSelectionBackend::CanvasKitBrowser,
                        prefer_strict_outline: kind == TextVariantKind::GlyphOutline,
                        allow_colrv0_color_layers: true,
                        allow_colrv1_stage1_color_graph: true,
                        allow_bitmap_glyph: true,
                        allow_svg_glyph: true,
                        ..TextVariantSelectionOptions::canvaskit()
                    },
                )
            })
            .unwrap_or_default();
        let reports = reports
            .into_iter()
            .map(|report| (report.equivalence_group.clone(), report))
            .collect();
        let mut value: Value = serde_json::from_str(&self.to_json()).map_err(|error| {
            vec![issue(
                "invalidPaintSerialization",
                None,
                "root",
                format!("canonical paint JSON is not finite/valid: {error}"),
            )]
        })?;
        let mut issues = Vec::new();
        let mut used_variants = BTreeSet::new();
        let mut required_features = BTreeSet::from([
            "text.variants".to_string(),
            "text.paintOrderSlot".to_string(),
        ]);
        envelope_node(
            &mut value["root"],
            "root",
            strict_kind,
            &reports,
            &mut used_variants,
            &mut required_features,
            &mut issues,
        );
        if !issues.is_empty() {
            return Err(issues);
        }

        value["schemaVersion"] = json!(TEXT_IR_V2_ENVELOPE_SCHEMA_VERSION);
        value["schemaMinorVersion"] = json!(0);
        value["schema"] = json!({ "major": 2, "minor": 0 });
        value["layout"] = json!({ "profile": "hwpCompat", "measurementAuthority": "legacyHwpPositions", "shapedMeasurement": "diagnosticsOnly" });
        let strict = strict_kind.is_some();
        if strict {
            required_features.insert("text.strictVisualFallbackFree".to_string());
        }
        for name in [
            "text.variants",
            "text.paintOrderSlot",
            "text.strictVisualFallbackFree",
        ] {
            append_feature(&mut value["knownFeatures"], name);
        }
        for feature in &required_features {
            append_feature(&mut value["usedFeatures"], feature);
        }
        value["requiredFeatures"] = json!(required_features);
        value["text"]["defaultVariant"] =
            json!(strict_kind.map_or("textRun", TextVariantKind::as_str));
        value["text"]["variants"] = json!(used_variants);
        value["text"]["fallbackRequired"] = json!(!strict);
        let text_v2 = &mut value["textV2"];
        text_v2["profile"] = json!(if strict {
            "strictVisual"
        } else {
            "compatibility"
        });
        text_v2["canonicalOp"] = json!("text");
        text_v2["compatibilityProfile"] = json!(if strict {
            "fallbackFreeStrict"
        } else {
            "v2Compat"
        });
        text_v2["fallbackPolicy"] = json!(if strict { "none" } else { "required" });
        text_v2["fallbackRequired"] = json!(!strict);
        text_v2["strictVisualFallbackFree"] = json!(strict);
        text_v2["paintOrderSlots"] = json!("required");
        // A strict export no longer contains the compatibility fallback. Keep
        // diagnostics as evidence without claiming that v1 downgrade is lossless.
        if strict {
            text_v2["downgradePath"] = json!("requiresTextRunFallbackReconstruction");
            if let Some(slots) = text_v2["slotDiagnostics"].as_array_mut() {
                for slot in slots {
                    slot["fallbackPresent"] = json!(false);
                    slot["strictVariantAvailable"] = json!(true);
                    slot["fallbackReason"] = Value::Null;
                }
            }
        }
        serde_json::to_string(&value).map_err(|error| {
            vec![issue(
                "invalidPaintSerialization",
                None,
                "root",
                error.to_string(),
            )]
        })
    }
}

fn append_feature(features: &mut Value, feature: &str) {
    if let Some(features) = features.as_array_mut() {
        if !features.iter().any(|value| value == feature) {
            features.push(json!(feature));
        }
    }
}

fn issue(
    code: &'static str,
    slot_id: Option<String>,
    leaf_path: &str,
    message: String,
) -> TextV2ValidationIssue {
    TextV2ValidationIssue {
        severity: TextV2ValidationSeverity::Error,
        code,
        slot_id,
        leaf_path: Some(leaf_path.to_string()),
        message,
    }
}

fn slot_id(op: &Value) -> Option<String> {
    if !matches!(
        op["type"].as_str(),
        Some("textRun" | "glyphRun" | "glyphOutline")
    ) {
        return None;
    }
    op["variant"]["equivalenceGroup"]
        .as_str()
        .map(str::to_string)
        .or_else(|| op["source"]["id"].as_u64().map(|id| format!("text-{id}")))
}

fn envelope_node(
    node: &mut Value,
    path: &str,
    strict_kind: Option<TextVariantKind>,
    reports: &BTreeMap<String, TextVariantSelectionReport>,
    used_variants: &mut BTreeSet<String>,
    required_features: &mut BTreeSet<String>,
    issues: &mut Vec<TextV2ValidationIssue>,
) {
    match node["kind"].as_str() {
        Some("group") => {
            if let Some(children) = node["children"].as_array_mut() {
                for (index, child) in children.iter_mut().enumerate() {
                    envelope_node(
                        child,
                        &format!("{path}/group[{index}]"),
                        strict_kind,
                        reports,
                        used_variants,
                        required_features,
                        issues,
                    );
                }
            }
        }
        Some("clipRect") => envelope_node(
            &mut node["child"],
            &format!("{path}/clip"),
            strict_kind,
            reports,
            used_variants,
            required_features,
            issues,
        ),
        Some("leaf") => {
            let ops = node["ops"]
                .as_array_mut()
                .map(std::mem::take)
                .unwrap_or_default();
            let mut by_slot = BTreeMap::<String, Vec<Value>>::new();
            for op in &ops {
                if let Some(id) = slot_id(op) {
                    by_slot.entry(id).or_default().push(op.clone());
                }
            }
            let mut emitted = Vec::with_capacity(ops.len());
            for op in ops {
                if op["type"] != "textRun" {
                    if !matches!(op["type"].as_str(), Some("glyphRun" | "glyphOutline")) {
                        emitted.push(op);
                    }
                    continue;
                }
                let Some(id) = slot_id(&op) else {
                    issues.push(issue(
                        "textSourceMissing",
                        None,
                        path,
                        "TextRun has no canonical source id".to_string(),
                    ));
                    continue;
                };
                let parts = by_slot.remove(&id).unwrap_or_default();
                let selected = if let Some(kind) = strict_kind {
                    match reports.get(&id) {
                        Some(report)
                            if report.selected_variant_kind == Some(kind)
                                && !report.fallback_required =>
                        {
                            report.selected_variant_id.as_deref()
                        }
                        _ => {
                            issues.push(issue("strictVisualVariantMissing", Some(id), path, format!("fallback-free {} export requires a complete, resource-proven variant for every text slot", kind.as_str())));
                            continue;
                        }
                    }
                } else {
                    None
                };
                let variants = envelope_variants(parts, selected, path, &id, issues);
                for variant in &variants {
                    if let Some(kind) = variant["kind"].as_str() {
                        used_variants.insert(kind.to_string());
                        if strict_kind.is_some() {
                            required_features.insert(
                                if kind == "glyphRun" {
                                    "text.glyphRun"
                                } else {
                                    "text.outlineGlyph"
                                }
                                .to_string(),
                            );
                            if kind == "glyphRun" {
                                required_features.insert("fontResources".to_string());
                            }
                        }
                    }
                    if strict_kind.is_some() {
                        if let Some(features) = variant["requiredFeatures"].as_array() {
                            required_features.extend(
                                features
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .map(str::to_string),
                            );
                        }
                        for part in variant["parts"].as_array().into_iter().flatten() {
                            if let Some(payload_kind) = part["payload"]["payloadKind"].as_str() {
                                required_features
                                    .insert(format!("text.glyphOutline.{payload_kind}"));
                            }
                            let payload = &part["payload"];
                            if !payload["svgGlyph"]["fontEmBox"].is_null() {
                                required_features
                                    .insert("text.glyphOutline.svgGlyph.fontEmBox".to_string());
                            }
                            if let Some(format) = payload["colorLayers"]["colorFormat"].as_str() {
                                required_features
                                    .insert(format!("text.glyphOutline.colorLayers.{format}"));
                            }
                            for node in payload["colorLayers"]["paintGraph"]["nodes"]
                                .as_array()
                                .into_iter()
                                .flatten()
                            {
                                match node["kind"].as_str() {
                                    Some("composite") => {
                                        required_features.insert(
                                            "text.glyphOutline.colorLayers.colrV1.sourceOver"
                                                .to_string(),
                                        );
                                    }
                                    Some("clip") => {
                                        required_features.insert(
                                            "text.glyphOutline.colorLayers.colrV1.clip".to_string(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                emitted.push(json!({
                    "id": id, "type": "text", "bbox": op["bbox"],
                    "paintOrderSlotId": id, "selectionPolicy": "exclusiveVariantSet",
                    "defaultVariantId": selected.unwrap_or("textRun"),
                    "fallbackPolicy": if strict_kind.is_some() { "none" } else { "required" },
                    "variants": variants,
                }));
            }
            // A second sidecar group may claim the same source. The canonical
            // v1 writer anchors only one group to that TextRun; silently dropping
            // the remaining group would make even a compat export lossy.
            for id in by_slot.into_keys() {
                issues.push(issue(
                    "unanchoredTextVariant",
                    Some(id),
                    path,
                    "text variant group has no exported TextRun slot".to_string(),
                ));
            }
            node["ops"] = Value::Array(emitted);
        }
        _ => {}
    }
}

fn envelope_variants(
    parts: Vec<Value>,
    selected: Option<&str>,
    path: &str,
    slot: &str,
    issues: &mut Vec<TextV2ValidationIssue>,
) -> Vec<Value> {
    let mut variants = Vec::<Value>::new();
    for mut payload in parts {
        let kind = payload["type"].as_str().unwrap_or("textRun").to_string();
        let meta = payload["variant"].clone();
        let id = meta["variantId"].as_str().unwrap_or("textRun").to_string();
        if selected.is_some_and(|selected| selected != id) {
            continue;
        }
        let part_index = meta["partIndex"].as_u64().unwrap_or(0);
        let part_count = meta["partCount"].as_u64().unwrap_or(1);
        payload["id"] = json!(if kind == "textRun" {
            format!("op-{slot}")
        } else {
            format!("op-{slot}-{id}-{part_index}")
        });
        let mut part =
            json!({ "partIndex": part_index, "partCount": part_count, "payload": payload });
        if !meta["localPaintOrder"].is_null() {
            part["localPaintOrder"] = meta["localPaintOrder"].clone();
        }
        let index = if let Some(index) = variants
            .iter()
            .position(|variant| variant["variantId"] == id)
        {
            index
        } else {
            let mut variant =
                json!({ "variantId": id, "kind": kind, "requiredFeatures": [], "parts": [] });
            if !meta["quality"].is_null() {
                variant["quality"] = meta["quality"].clone();
            }
            variants.push(variant);
            variants.len() - 1
        };
        let variant = &mut variants[index];
        if variant["kind"] != kind {
            issues.push(issue(
                "variantKindMismatch",
                Some(slot.to_string()),
                path,
                format!("variant {id} mixes text payload kinds"),
            ));
        }
        if let Some(features) = meta["requires"].as_array() {
            for feature in features.iter().filter_map(Value::as_str) {
                append_feature(&mut variant["requiredFeatures"], feature);
            }
        }
        variant["parts"]
            .as_array_mut()
            .expect("constructed parts array")
            .push(part);
    }
    for variant in &variants {
        let parts = variant["parts"]
            .as_array()
            .expect("constructed parts array");
        let expected = parts
            .first()
            .and_then(|part| part["partCount"].as_u64())
            .unwrap_or(0);
        let indices = parts
            .iter()
            .filter_map(|part| part["partIndex"].as_u64())
            .collect::<BTreeSet<_>>();
        if expected == 0
            || expected != parts.len() as u64
            || indices.len() != parts.len()
            || parts
                .iter()
                .any(|part| part["partCount"].as_u64() != Some(expected))
            || indices
                .iter()
                .enumerate()
                .any(|(expected, actual)| *actual != expected as u64)
        {
            issues.push(issue(
                "variantPartsIncomplete",
                Some(slot.to_string()),
                path,
                "text variant parts must form exactly one complete set".to_string(),
            ));
        }
    }
    variants
}
