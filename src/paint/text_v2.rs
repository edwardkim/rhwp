//! Text IR v2 diagnostics and compatibility profile guards.
//!
//! P13 keeps the schema-v1 flattened `TextRun`/`GlyphRun` export as the
//! compatibility writer. This module adds structured diagnostics that explain
//! whether a future schema-v2 text slot can be promoted without silently
//! dropping the required `TextRun` fallback.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Serialize;

use crate::document_core::helpers::json_escape as raw_json_escape;
use crate::model::style::UnderlineType;
use crate::paint::{
    GlyphOutlinePayloadKind, GlyphRunDiagnostics, GlyphRunOrientation, GlyphRunReplayEligibility,
    LayerGlyphOutlinePaint, LayerGlyphRunPaint, LayerNode, LayerNodeKind, PageLayerTree, PaintOp,
    ResourceArena, TextVariantKind, TextVariantQuality, RESOURCE_KEY_ALGORITHM,
};
use crate::renderer::render_tree::{FieldMarkerType, TextRunNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextV2CompatibilityProfile {
    V1Compat,
    V2Compat,
    FallbackFreeStrict,
}

impl TextV2CompatibilityProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V1Compat => "v1Compat",
            Self::V2Compat => "v2Compat",
            Self::FallbackFreeStrict => "fallbackFreeStrict",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextV2ValidationSeverity {
    Info,
    Warning,
    Error,
}

impl TextV2ValidationSeverity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextV2LineBreakRiskLevel {
    NoChangeLikely,
    ChangePossible,
    ChangeLikely,
}

impl TextV2LineBreakRiskLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::NoChangeLikely => "noChangeLikely",
            Self::ChangePossible => "changePossible",
            Self::ChangeLikely => "changeLikely",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextV2ValidationIssue {
    pub severity: TextV2ValidationSeverity,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaf_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextV2VariantDiagnostic {
    pub variant_id: String,
    pub variant_kind: &'static str,
    pub required_features: Vec<String>,
    pub part_count: u32,
    pub present_part_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<&'static str>,
    pub strict_visual_eligible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextV2SlotDiagnostic {
    pub paint_order_slot_id: String,
    pub equivalence_group: String,
    pub leaf_path: String,
    pub fallback_present: bool,
    pub strict_variant_available: bool,
    pub variants: Vec<TextV2VariantDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextV2LineBreakRisk {
    pub leaf_path: String,
    pub text_preview: String,
    pub risk: TextV2LineBreakRiskLevel,
    pub reasons: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextV2Diagnostics {
    pub compatibility_profile: TextV2CompatibilityProfile,
    pub fallback_required: bool,
    pub downgrade_path: &'static str,
    pub slot_diagnostics: Vec<TextV2SlotDiagnostic>,
    pub validation_issues: Vec<TextV2ValidationIssue>,
    pub line_break_risks: Vec<TextV2LineBreakRisk>,
}

impl TextV2Diagnostics {
    pub fn from_layer_tree(tree: &PageLayerTree) -> Self {
        Self::from_layer_tree_with_profile(tree, TextV2CompatibilityProfile::V1Compat)
    }

    pub fn from_layer_tree_with_profile(
        tree: &PageLayerTree,
        profile: TextV2CompatibilityProfile,
    ) -> Self {
        let mut slots = Vec::new();
        let mut line_break_risks = Vec::new();
        collect_node(
            &tree.root,
            &tree.resources,
            "root".to_string(),
            &mut slots,
            &mut line_break_risks,
        );
        let mut validation_issues = Vec::new();
        if let Err(error) = crate::paint::validate_text_variant_scope(tree) {
            validation_issues.push(TextV2ValidationIssue {
                severity: TextV2ValidationSeverity::Error,
                code: "schemaV1VariantScopeInvalid",
                slot_id: None,
                leaf_path: None,
                message: error.to_string(),
            });
        }
        for slot in &slots {
            if !slot.fallback_present {
                validation_issues.push(TextV2ValidationIssue {
                    severity: TextV2ValidationSeverity::Error,
                    code: "schemaV1TextRunFallbackMissing",
                    slot_id: Some(slot.paint_order_slot_id.clone()),
                    leaf_path: Some(slot.leaf_path.clone()),
                    message: format!(
                        "text slot `{}` has no schema-v1 TextRun fallback",
                        slot.paint_order_slot_id
                    ),
                });
            }
            if matches!(profile, TextV2CompatibilityProfile::FallbackFreeStrict)
                && !slot.strict_variant_available
            {
                validation_issues.push(TextV2ValidationIssue {
                    severity: TextV2ValidationSeverity::Error,
                    code: "fallbackFreeStrictVariantMissing",
                    slot_id: Some(slot.paint_order_slot_id.clone()),
                    leaf_path: Some(slot.leaf_path.clone()),
                    message: format!(
                        "fallback-free text profile requires a strict visual variant for `{}`",
                        slot.equivalence_group
                    ),
                });
            }
        }
        if matches!(profile, TextV2CompatibilityProfile::FallbackFreeStrict) && slots.is_empty() {
            validation_issues.push(TextV2ValidationIssue {
                severity: TextV2ValidationSeverity::Error,
                code: "fallbackFreeTextVariantMissing",
                slot_id: None,
                leaf_path: None,
                message:
                    "fallback-free text profile requires at least one strict text variant slot"
                        .to_string(),
            });
        }
        Self {
            compatibility_profile: profile,
            fallback_required: !matches!(profile, TextV2CompatibilityProfile::FallbackFreeStrict),
            downgrade_path: "schemaV1FlattenedTextRunAndGlyphRun",
            slot_diagnostics: slots,
            validation_issues,
            line_break_risks,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.validation_issues
            .iter()
            .any(|issue| issue.severity == TextV2ValidationSeverity::Error)
    }

    pub fn to_json(&self) -> String {
        let mut buf = String::with_capacity(1024);
        self.write_json(&mut buf);
        buf
    }

    pub fn write_json(&self, buf: &mut String) {
        buf.push('{');
        let _ = write!(
            buf,
            "\"compatibilityProfile\":{},\"fallbackRequired\":{},\"downgradePath\":{},\"slotDiagnostics\":[",
            json_string(self.compatibility_profile.as_str()),
            self.fallback_required,
            json_string(self.downgrade_path)
        );
        for (index, slot) in self.slot_diagnostics.iter().enumerate() {
            if index > 0 {
                buf.push(',');
            }
            write_slot_diagnostic(buf, slot);
        }
        buf.push_str("],\"validationIssues\":[");
        for (index, issue) in self.validation_issues.iter().enumerate() {
            if index > 0 {
                buf.push(',');
            }
            write_validation_issue(buf, issue);
        }
        buf.push_str("],\"lineBreakRisks\":[");
        for (index, risk) in self.line_break_risks.iter().enumerate() {
            if index > 0 {
                buf.push(',');
            }
            write_line_break_risk(buf, risk);
        }
        buf.push_str("]}");
    }
}

fn write_slot_diagnostic(buf: &mut String, slot: &TextV2SlotDiagnostic) {
    let _ = write!(
        buf,
        "{{\"paintOrderSlotId\":{},\"equivalenceGroup\":{},\"leafPath\":{},\"fallbackPresent\":{},\"strictVariantAvailable\":{},\"variants\":[",
        json_string(&slot.paint_order_slot_id),
        json_string(&slot.equivalence_group),
        json_string(&slot.leaf_path),
        slot.fallback_present,
        slot.strict_variant_available
    );
    for (index, variant) in slot.variants.iter().enumerate() {
        if index > 0 {
            buf.push(',');
        }
        write_variant_diagnostic(buf, variant);
    }
    buf.push(']');
    if let Some(reason) = &slot.fallback_reason {
        let _ = write!(buf, ",\"fallbackReason\":{}", json_string(reason));
    }
    buf.push('}');
}

fn write_variant_diagnostic(buf: &mut String, variant: &TextV2VariantDiagnostic) {
    let _ = write!(
        buf,
        "{{\"variantId\":{},\"variantKind\":{},\"requiredFeatures\":[",
        json_string(&variant.variant_id),
        json_string(variant.variant_kind)
    );
    for (index, feature) in variant.required_features.iter().enumerate() {
        if index > 0 {
            buf.push(',');
        }
        buf.push_str(&json_string(feature));
    }
    let _ = write!(
        buf,
        "],\"partCount\":{},\"presentPartCount\":{}",
        variant.part_count, variant.present_part_count
    );
    if let Some(quality) = variant.quality {
        let _ = write!(buf, ",\"quality\":{}", json_string(quality));
    }
    let _ = write!(
        buf,
        ",\"strictVisualEligible\":{}",
        variant.strict_visual_eligible
    );
    if let Some(reason) = &variant.fallback_reason {
        let _ = write!(buf, ",\"fallbackReason\":{}", json_string(reason));
    }
    buf.push('}');
}

fn write_validation_issue(buf: &mut String, issue: &TextV2ValidationIssue) {
    let _ = write!(
        buf,
        "{{\"severity\":{},\"code\":{}",
        json_string(issue.severity.as_str()),
        json_string(issue.code)
    );
    if let Some(slot_id) = &issue.slot_id {
        let _ = write!(buf, ",\"slotId\":{}", json_string(slot_id));
    }
    if let Some(leaf_path) = &issue.leaf_path {
        let _ = write!(buf, ",\"leafPath\":{}", json_string(leaf_path));
    }
    let _ = write!(buf, ",\"message\":{}}}", json_string(&issue.message));
}

fn write_line_break_risk(buf: &mut String, risk: &TextV2LineBreakRisk) {
    let _ = write!(
        buf,
        "{{\"leafPath\":{},\"textPreview\":{},\"risk\":{},\"reasons\":[",
        json_string(&risk.leaf_path),
        json_string(&risk.text_preview),
        json_string(risk.risk.as_str())
    );
    for (index, reason) in risk.reasons.iter().enumerate() {
        if index > 0 {
            buf.push(',');
        }
        buf.push_str(&json_string(reason));
    }
    buf.push_str("]}");
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", raw_json_escape(value))
}

fn collect_node(
    node: &LayerNode,
    resources: &ResourceArena,
    path: String,
    slots: &mut Vec<TextV2SlotDiagnostic>,
    line_break_risks: &mut Vec<TextV2LineBreakRisk>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for (index, child) in children.iter().enumerate() {
                collect_node(
                    child,
                    resources,
                    format!("{path}/group[{index}]"),
                    slots,
                    line_break_risks,
                );
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            collect_node(
                child,
                resources,
                format!("{path}/clip"),
                slots,
                line_break_risks,
            );
        }
        LayerNodeKind::Leaf { ops } => {
            let text_fallback_present = ops.iter().any(|op| matches!(op, PaintOp::TextRun { .. }));
            let mut groups = BTreeMap::<String, BTreeMap<String, VariantAccumulator>>::new();
            for op in ops {
                match op {
                    PaintOp::TextRun { run, .. } => {
                        if let Some(risk) = line_break_risk_for_run(&path, run) {
                            line_break_risks.push(risk);
                        }
                    }
                    PaintOp::GlyphRun { run, .. } => {
                        let variant = &run.variant;
                        let group = groups.entry(variant.equivalence_group.clone()).or_default();
                        let entry = group.entry(variant.variant_id.clone()).or_insert_with(|| {
                            VariantAccumulator {
                                variant_id: variant.variant_id.clone(),
                                variant_kind: variant.variant_kind,
                                required_features: variant
                                    .requires
                                    .iter()
                                    .cloned()
                                    .collect::<BTreeSet<_>>(),
                                part_count: variant.part_count,
                                present_parts: BTreeSet::new(),
                                strict_parts: BTreeSet::new(),
                                quality: variant.quality,
                                fallback_reason: None,
                            }
                        });
                        entry
                            .required_features
                            .extend(variant.requires.iter().cloned());
                        entry.part_count = entry.part_count.max(variant.part_count);
                        entry.present_parts.insert(variant.part_index);
                        entry.quality = entry.quality.or(variant.quality);
                        if glyph_run_is_strict(run, resources) {
                            entry.strict_parts.insert(variant.part_index);
                        }
                        entry.merge_fallback_reason(glyph_run_fallback_reason(run, resources));
                    }
                    PaintOp::GlyphOutline { outline, .. } => {
                        let variant = &outline.variant;
                        let group = groups.entry(variant.equivalence_group.clone()).or_default();
                        let entry = group.entry(variant.variant_id.clone()).or_insert_with(|| {
                            VariantAccumulator {
                                variant_id: variant.variant_id.clone(),
                                variant_kind: variant.variant_kind,
                                required_features: variant
                                    .requires
                                    .iter()
                                    .cloned()
                                    .collect::<BTreeSet<_>>(),
                                part_count: variant.part_count,
                                present_parts: BTreeSet::new(),
                                strict_parts: BTreeSet::new(),
                                quality: variant.quality,
                                fallback_reason: None,
                            }
                        });
                        entry
                            .required_features
                            .extend(variant.requires.iter().cloned());
                        entry.part_count = entry.part_count.max(variant.part_count);
                        entry.present_parts.insert(variant.part_index);
                        entry.quality = entry.quality.or(variant.quality);
                        if glyph_outline_is_strict(outline) {
                            entry.strict_parts.insert(variant.part_index);
                        }
                        entry.merge_fallback_reason(glyph_outline_fallback_reason(outline));
                    }
                    _ => {}
                }
            }
            for (equivalence_group, variants) in groups {
                let variant_diagnostics = variants
                    .into_values()
                    .map(|variant| variant.finish())
                    .collect::<Vec<_>>();
                let strict_variant_available = variant_diagnostics
                    .iter()
                    .any(|variant| variant.strict_visual_eligible);
                let fallback_reason = if !text_fallback_present {
                    Some("missingTextRunFallback".to_string())
                } else if !strict_variant_available {
                    variant_diagnostics
                        .iter()
                        .find_map(|variant| variant.fallback_reason.clone())
                        .or_else(|| Some("strictVariantUnavailable".to_string()))
                } else {
                    None
                };
                slots.push(TextV2SlotDiagnostic {
                    paint_order_slot_id: equivalence_group.clone(),
                    equivalence_group,
                    leaf_path: path.clone(),
                    fallback_present: text_fallback_present,
                    strict_variant_available,
                    variants: variant_diagnostics,
                    fallback_reason,
                });
            }
        }
    }
}

#[derive(Debug)]
struct VariantAccumulator {
    variant_id: String,
    variant_kind: TextVariantKind,
    required_features: BTreeSet<String>,
    part_count: u32,
    present_parts: BTreeSet<u32>,
    strict_parts: BTreeSet<u32>,
    quality: Option<TextVariantQuality>,
    fallback_reason: Option<String>,
}

impl VariantAccumulator {
    fn merge_fallback_reason(&mut self, reason: Option<String>) {
        let Some(reason) = reason else {
            return;
        };
        if self.fallback_reason.as_deref().is_none_or(|current| {
            fallback_reason_priority(&reason) < fallback_reason_priority(current)
        }) {
            self.fallback_reason = Some(reason);
        }
    }

    fn finish(self) -> TextV2VariantDiagnostic {
        let present_part_count = self.present_parts.len() as u32;
        let strict_visual_eligible = self.part_count > 0
            && present_part_count == self.part_count
            && (0..self.part_count).all(|index| self.strict_parts.contains(&index));
        TextV2VariantDiagnostic {
            variant_id: self.variant_id,
            variant_kind: self.variant_kind.as_str(),
            required_features: self.required_features.into_iter().collect(),
            part_count: self.part_count,
            present_part_count,
            quality: self.quality.map(TextVariantQuality::as_str),
            strict_visual_eligible,
            fallback_reason: self.fallback_reason,
        }
    }
}

fn fallback_reason_priority(reason: &str) -> u8 {
    match reason {
        "missingGlyph" | "clusterMismatch" | "unsplitFallbackFont" => 0,
        "strictVisualIneligible" | "strictVariantUnavailable" => 1,
        _ => 2,
    }
}

fn glyph_run_diagnostics_are_strict(diagnostics: &GlyphRunDiagnostics) -> bool {
    diagnostics.strict_visual_eligible
        && matches!(
            diagnostics.quality,
            TextVariantQuality::Exact | TextVariantQuality::PositionAdjusted
        )
        && matches!(
            diagnostics.replay_eligibility,
            GlyphRunReplayEligibility::Portable
        )
        && diagnostics.missing_glyph_count == 0
        && diagnostics.cluster_mismatch_count == 0
        && diagnostics.used_fallback_font_count == 0
}

fn glyph_run_is_strict(run: &LayerGlyphRunPaint, resources: &ResourceArena) -> bool {
    glyph_run_diagnostics_are_strict(&run.diagnostics)
        && matches!(run.orientation, GlyphRunOrientation::Horizontal)
        && run.glyph_transforms.is_none()
        && glyph_run_font_proof_fallback_reason(run, resources).is_none()
}

fn glyph_run_fallback_reason(
    run: &LayerGlyphRunPaint,
    resources: &ResourceArena,
) -> Option<String> {
    if glyph_run_is_strict(run, resources) {
        return None;
    }
    if let Some(reason) = glyph_run_diagnostics_fallback_reason(&run.diagnostics) {
        return Some(reason);
    }
    if matches!(run.orientation, GlyphRunOrientation::MixedPerGlyph) {
        return Some("mixedPerGlyphAuthorityPending".to_string());
    }
    if !matches!(run.orientation, GlyphRunOrientation::Horizontal) {
        return Some("verticalGlyphOrientationAuthorityPending".to_string());
    }
    if run.glyph_transforms.is_some() {
        return Some("glyphTransformAuthorityPending".to_string());
    }
    if !run.shape_key.font_instance.variations.is_empty() {
        return Some("variationUnsupported".to_string());
    }
    if let Some(reason) = glyph_run_font_proof_fallback_reason(run, resources) {
        return Some(reason);
    }
    None
}

fn glyph_run_diagnostics_fallback_reason(diagnostics: &GlyphRunDiagnostics) -> Option<String> {
    if glyph_run_diagnostics_are_strict(diagnostics) {
        return None;
    }
    if let Some(reason) = &diagnostics.reason {
        return Some(reason.clone());
    }
    if diagnostics.missing_glyph_count > 0 {
        return Some("missingGlyph".to_string());
    }
    if diagnostics.cluster_mismatch_count > 0 {
        return Some("clusterMismatch".to_string());
    }
    if diagnostics.used_fallback_font_count > 0 {
        return Some("unsplitFallbackFont".to_string());
    }
    match diagnostics.replay_eligibility {
        GlyphRunReplayEligibility::Portable => {}
        GlyphRunReplayEligibility::ConditionalExternalFont => {
            return Some("externalFontNotVerified".to_string());
        }
        GlyphRunReplayEligibility::LocalDiagnosticOnly
        | GlyphRunReplayEligibility::NotReplayable => {
            return Some("fontNotPortable".to_string());
        }
    }
    if !diagnostics.strict_visual_eligible {
        return Some("strictVisualIneligible".to_string());
    }
    Some("strictVariantUnavailable".to_string())
}

fn glyph_run_font_proof_fallback_reason(
    run: &LayerGlyphRunPaint,
    resources: &ResourceArena,
) -> Option<String> {
    if !matches!(
        run.diagnostics.replay_eligibility,
        GlyphRunReplayEligibility::Portable
    ) {
        return None;
    }

    let font_resources = resources.font_resources();
    let Some(face) = font_resources
        .faces
        .iter()
        .find(|face| face.id == run.shape_key.font_instance.face_key)
    else {
        return Some("fontFaceMissing".to_string());
    };
    if face.face_index != 0 {
        return Some("faceIndexUnsupported".to_string());
    }
    let Some(blob) = font_resources
        .blobs
        .iter()
        .find(|blob| blob.id == face.blob_key)
    else {
        return Some("fontBlobMissing".to_string());
    };
    let crate::paint::FontPortability::PortableBlob { digest, data_ref } = &blob.portability else {
        return Some("fontBlobNotPortable".to_string());
    };
    if blob.data_ref.as_ref() != Some(data_ref) {
        return Some("fontBlobDataRefMismatch".to_string());
    }
    match resources.font_blob_bytes_for_ref(data_ref) {
        Some(bytes) => {
            let actual_digest = crate::paint::resource_digest_hex(bytes);
            if !font_digest_matches_resource_digest(digest, &actual_digest)
                || !blob.digest.as_ref().is_none_or(|digest| {
                    font_digest_matches_resource_digest(digest, &actual_digest)
                })
            {
                return Some("fontBlobDigestMismatch".to_string());
            }
        }
        None => {
            return Some("fontBlobBytesMissing".to_string());
        }
    }
    None
}

fn font_digest_matches_resource_digest(digest: &crate::paint::FontDigest, actual: &str) -> bool {
    digest.algorithm == RESOURCE_KEY_ALGORITHM && digest.value == actual
}

fn glyph_outline_is_strict(outline: &LayerGlyphOutlinePaint) -> bool {
    glyph_run_diagnostics_are_strict(&outline.diagnostics)
        && !outline.paths.is_empty()
        && outline.paint_style.is_fill_only_glyph_replay()
        && match outline.payload_kind {
            GlyphOutlinePayloadKind::MonochromeFill => outline.stroke.is_none(),
            GlyphOutlinePayloadKind::MonochromeFillStroke => outline
                .stroke
                .as_ref()
                .is_some_and(|stroke| stroke.is_strict_subset()),
            GlyphOutlinePayloadKind::ColorLayers
            | GlyphOutlinePayloadKind::BitmapGlyph
            | GlyphOutlinePayloadKind::SvgGlyph => false,
        }
}

fn glyph_outline_fallback_reason(outline: &LayerGlyphOutlinePaint) -> Option<String> {
    if glyph_outline_is_strict(outline) {
        return None;
    }
    if let Some(reason) = glyph_run_diagnostics_fallback_reason(&outline.diagnostics) {
        return Some(reason);
    }
    if outline.paths.is_empty()
        && matches!(
            outline.payload_kind,
            GlyphOutlinePayloadKind::MonochromeFill | GlyphOutlinePayloadKind::MonochromeFillStroke
        )
    {
        return Some("emptyGlyphOutline".to_string());
    }
    if !outline.paint_style.is_fill_only_glyph_replay() {
        return Some("unsupportedPaintEffect".to_string());
    }
    match outline.payload_kind {
        GlyphOutlinePayloadKind::MonochromeFill if outline.stroke.is_some() => {
            return Some("unexpectedOutlineStroke".to_string());
        }
        GlyphOutlinePayloadKind::MonochromeFillStroke
            if !outline
                .stroke
                .as_ref()
                .is_some_and(|stroke| stroke.is_strict_subset()) =>
        {
            return Some("unsupportedOutlineStroke".to_string());
        }
        GlyphOutlinePayloadKind::ColorLayers => {
            return Some("unsupportedColorGlyph".to_string());
        }
        GlyphOutlinePayloadKind::BitmapGlyph => {
            return Some("unsupportedBitmapGlyph".to_string());
        }
        GlyphOutlinePayloadKind::SvgGlyph => {
            return Some("unsupportedSvgGlyph".to_string());
        }
        _ => {}
    }
    None
}

fn line_break_risk_for_run(leaf_path: &str, run: &TextRunNode) -> Option<TextV2LineBreakRisk> {
    let mut reasons = Vec::new();
    if run.char_overlap.is_some() {
        reasons.push("charOverlap");
    }
    if run.is_vertical {
        reasons.push("verticalText");
    }
    if run.rotation.abs() > f64::EPSILON {
        reasons.push("rotatedText");
    }
    if (run.style.ratio - 1.0).abs() > f64::EPSILON {
        reasons.push("widthRatio");
    }
    if run.style.letter_spacing.abs() > f64::EPSILON
        || run.style.extra_word_spacing.abs() > f64::EPSILON
        || run.style.extra_char_spacing.abs() > f64::EPSILON
        || run.style.extra_dash_advance.abs() > f64::EPSILON
    {
        reasons.push("distributedSpacing");
    }
    if !run.style.tab_leaders.is_empty() || !run.style.inline_tabs.is_empty() {
        reasons.push("tabLeaderOrInlineTab");
    }
    if run.style.underline != UnderlineType::None
        || run.style.strikethrough
        || run.style.outline_type != 0
        || run.style.shadow_type != 0
        || run.style.emboss
        || run.style.engrave
        || run.style.emphasis_dot != 0
        || run.style.shade_color != 0x00FF_FFFF
    {
        reasons.push("textVisualEffect");
    }
    if run.field_marker != FieldMarkerType::None {
        reasons.push("fieldMarker");
    }
    if run.is_line_break_end {
        reasons.push("explicitLineBreakEnd");
    }
    if run.is_para_end {
        reasons.push("paragraphEndMarker");
    }
    if reasons.is_empty() {
        return None;
    }
    let risk = if reasons
        .iter()
        .any(|reason| matches!(*reason, "charOverlap" | "tabLeaderOrInlineTab"))
    {
        TextV2LineBreakRiskLevel::ChangeLikely
    } else {
        TextV2LineBreakRiskLevel::ChangePossible
    };
    Some(TextV2LineBreakRisk {
        leaf_path: leaf_path.to_string(),
        text_preview: run.text.chars().take(32).collect(),
        risk,
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{
        font_blob_resource_key, resource_digest_hex, BinaryResourceKind, BinaryResourceRef,
        FontBlobKey, FontBlobResource, FontDigest, FontFaceKey, FontFaceResource,
        FontFallbackPolicyId, FontInstanceKey, FontPortability, FontResourceSource, GlyphCluster,
        GlyphOutlineFillRule, GlyphOutlinePayloadKind, GlyphRange, GlyphRunOrientation,
        GlyphTransform, LayerAffineTransform, LayerGlyphOutlinePaint, LayerGlyphOutlinePath,
        LayerGlyphRunPaint, LayerNode, LayerPoint, PaintTextStyle, ResourceArena, ScriptTag,
        ShapeKey, ShapingEngineId, TextDirection, TextSourceId, TextSourceRange, TextSourceSpan,
        WritingMode,
    };
    use crate::renderer::render_tree::BoundingBox;
    use crate::renderer::{PathCommand, TextStyle};

    fn text_run(text: &str) -> TextRunNode {
        TextRunNode {
            text: text.to_string(),
            style: TextStyle {
                font_family: "Test".to_string(),
                font_size: 12.0,
                shade_color: 0x00FF_FFFF,
                ..Default::default()
            },
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
            baseline: 12.0,
            field_marker: FieldMarkerType::None,
        }
    }

    fn text_op(text: &str) -> PaintOp {
        PaintOp::text_run(BoundingBox::new(0.0, 0.0, 20.0, 20.0), text_run(text))
    }

    fn add_portable_font_resources(resources: &mut ResourceArena) {
        let font_bytes = [0_u8, 1, 2, 3];
        resources.intern_font_blob_bytes(&font_bytes);
        let blob_key = FontBlobKey("blob-0".to_string());
        let face_key = FontFaceKey("face-0".to_string());
        let digest_value = resource_digest_hex(font_bytes);
        let digest = FontDigest {
            algorithm: RESOURCE_KEY_ALGORITHM.to_string(),
            value: digest_value.clone(),
        };
        let data_ref = BinaryResourceRef {
            kind: BinaryResourceKind::FontBlob,
            id: font_blob_resource_key(font_bytes.len(), &digest_value),
        };
        resources.font_resources_mut().blobs.push(FontBlobResource {
            id: blob_key.clone(),
            digest: Some(digest.clone()),
            source: FontResourceSource::Embedded,
            data_ref: Some(data_ref.clone()),
            portability: FontPortability::PortableBlob { digest, data_ref },
        });
        resources.font_resources_mut().faces.push(FontFaceResource {
            id: face_key,
            blob_key,
            face_index: 0,
            postscript_name: None,
            family_names: Vec::new(),
            style_names: Vec::new(),
            weight_class: None,
            width_class: None,
            italic: None,
        });
    }

    fn glyph_op(reason: Option<&str>, missing_glyph_count: u32) -> PaintOp {
        glyph_op_for_text("A", reason, missing_glyph_count)
    }

    fn glyph_op_for_text(text: &str, reason: Option<&str>, missing_glyph_count: u32) -> PaintOp {
        glyph_op_part_for_text(text, reason, missing_glyph_count, 0, 1)
    }

    fn glyph_op_part(
        reason: Option<&str>,
        missing_glyph_count: u32,
        part_index: u32,
        part_count: u32,
    ) -> PaintOp {
        glyph_op_part_for_text("A", reason, missing_glyph_count, part_index, part_count)
    }

    fn glyph_op_part_for_text(
        text: &str,
        reason: Option<&str>,
        missing_glyph_count: u32,
        part_index: u32,
        part_count: u32,
    ) -> PaintOp {
        let utf8_len = text.len() as u32;
        let utf16_len = text.encode_utf16().count() as u32;
        PaintOp::GlyphRun {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            run: Box::new(LayerGlyphRunPaint {
                source: TextSourceSpan {
                    id: TextSourceId(0),
                    utf8_range: TextSourceRange::new(0, utf8_len),
                    utf16_range: TextSourceRange::new(0, utf16_len),
                    stable_source_key: None,
                },
                variant: {
                    let mut variant = crate::paint::PaintVariantMeta::text_run_default("text-0");
                    variant.variant_id = "glyphRun".to_string();
                    variant.variant_kind = TextVariantKind::GlyphRun;
                    variant.is_default_fallback = false;
                    variant.part_index = part_index;
                    variant.part_count = part_count;
                    variant.requires =
                        vec!["fontResources".to_string(), "text.glyphRun".to_string()];
                    variant.quality = Some(TextVariantQuality::Exact);
                    variant
                },
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
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
                        f: 12.0,
                    },
                    baseline_y: 0.0,
                },
                glyph_ids: vec![42],
                positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
                advances: None,
                clusters: vec![GlyphCluster {
                    source_range_utf8: TextSourceRange::new(0, utf8_len),
                    source_range_utf16: Some(TextSourceRange::new(0, utf16_len)),
                    text_range_utf8: Some(TextSourceRange::new(0, utf8_len)),
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
                    strict_visual_eligible: missing_glyph_count == 0,
                    max_origin_delta_px: 0.0,
                    max_advance_delta_px: 0.0,
                    max_residual_after_adjustment_px: 0.0,
                    cluster_mismatch_count: 0,
                    missing_glyph_count,
                    used_fallback_font_count: 0,
                    reason: reason.map(str::to_string),
                },
            }),
        }
    }

    fn glyph_outline_op() -> PaintOp {
        PaintOp::GlyphOutline {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            outline: Box::new(LayerGlyphOutlinePaint {
                source: TextSourceSpan {
                    id: TextSourceId(0),
                    utf8_range: TextSourceRange::new(0, 1),
                    utf16_range: TextSourceRange::new(0, 1),
                    stable_source_key: None,
                },
                variant: {
                    let mut variant = crate::paint::PaintVariantMeta::text_run_default("text-0");
                    variant.variant_id = "glyphOutline".to_string();
                    variant.variant_kind = TextVariantKind::GlyphOutline;
                    variant.is_default_fallback = false;
                    variant.requires = vec![
                        "text.glyphOutline".to_string(),
                        "text.glyphOutline.strictSidecar".to_string(),
                    ];
                    variant.quality = Some(TextVariantQuality::Exact);
                    variant.anchor_op_id = Some("text-0".to_string());
                    variant
                },
                payload_kind: GlyphOutlinePayloadKind::MonochromeFill,
                color_layers: None,
                bitmap_glyph: None,
                svg_glyph: None,
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
                placement: crate::paint::TextRunPlacement {
                    run_to_page: LayerAffineTransform {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                        f: 12.0,
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

    #[test]
    fn reports_strict_glyph_slot_without_validation_errors() {
        let mut tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), glyph_op(None, 0)],
            ),
        );
        add_portable_font_resources(&mut tree.resources);
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );
        assert_eq!(diagnostics.slot_diagnostics.len(), 1);
        assert!(diagnostics.slot_diagnostics[0].strict_variant_available);
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn rejects_fallback_free_profile_without_font_resource_proof() {
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), glyph_op(None, 0)],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert!(!diagnostics.slot_diagnostics[0].strict_variant_available);
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("fontFaceMissing")
        );
        assert_eq!(
            diagnostics.slot_diagnostics[0].variants[0]
                .fallback_reason
                .as_deref(),
            Some("fontFaceMissing")
        );
    }

    #[test]
    fn reports_strict_glyph_outline_slot_without_validation_errors() {
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), glyph_outline_op()],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );
        assert_eq!(diagnostics.slot_diagnostics.len(), 1);
        assert!(diagnostics.slot_diagnostics[0].strict_variant_available);
        assert_eq!(
            diagnostics.slot_diagnostics[0].variants[0].variant_kind,
            "glyphOutline"
        );
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn reports_glyph_outline_fallback_reason_when_payload_is_not_strict() {
        let mut op = glyph_outline_op();
        if let PaintOp::GlyphOutline { outline, .. } = &mut op {
            outline.paths.clear();
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), op],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );
        assert!(diagnostics.has_errors());
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("emptyGlyphOutline")
        );
    }

    #[test]
    fn resource_glyph_fallback_reason_is_not_masked_by_empty_paths() {
        let mut op = glyph_outline_op();
        let PaintOp::GlyphOutline { outline, .. } = &mut op else {
            panic!("expected glyph outline");
        };
        outline.paths.clear();
        outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
        assert_eq!(
            glyph_outline_fallback_reason(outline).as_deref(),
            Some("unsupportedBitmapGlyph")
        );
        outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
        assert_eq!(
            glyph_outline_fallback_reason(outline).as_deref(),
            Some("unsupportedSvgGlyph")
        );
    }

    #[test]
    fn serializes_requested_profile_without_default_profile_fallback() {
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A")],
            ),
        );
        let json = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        )
        .to_json();
        assert!(json.contains("\"compatibilityProfile\":\"fallbackFreeStrict\""));
        assert!(json.contains("\"fallbackRequired\":false"));
    }

    #[test]
    fn rejects_fallback_free_profile_without_strict_variant() {
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), glyph_op(Some("missingGlyph"), 1)],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );
        assert!(diagnostics.has_errors());
        assert_eq!(
            diagnostics.validation_issues[0].code,
            "fallbackFreeStrictVariantMissing"
        );
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("missingGlyph")
        );
    }

    #[test]
    fn rejects_fallback_free_profile_when_only_some_parts_are_strict() {
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![
                    text_op("A"),
                    glyph_op_part(None, 0, 0, 2),
                    glyph_op_part(Some("missingGlyph"), 1, 1, 2),
                ],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );
        assert!(diagnostics.has_errors());
        assert!(!diagnostics.slot_diagnostics[0].strict_variant_available);
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("missingGlyph")
        );
    }

    #[test]
    fn reports_mixed_per_glyph_as_authority_pending() {
        let mut op = glyph_op(None, 0);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.orientation = GlyphRunOrientation::MixedPerGlyph;
            run.glyph_transforms = Some(vec![GlyphTransform {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            }]);
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), op],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert!(!diagnostics.slot_diagnostics[0].strict_variant_available);
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("mixedPerGlyphAuthorityPending")
        );
        assert_eq!(
            diagnostics.slot_diagnostics[0].variants[0]
                .fallback_reason
                .as_deref(),
            Some("mixedPerGlyphAuthorityPending")
        );
    }

    #[test]
    fn reports_glyph_transform_as_authority_pending() {
        let mut op = glyph_op(None, 0);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.glyph_transforms = Some(vec![GlyphTransform {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            }]);
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), op],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert!(!diagnostics.slot_diagnostics[0].strict_variant_available);
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("glyphTransformAuthorityPending")
        );
    }

    #[test]
    fn reports_vertical_glyph_orientation_as_authority_pending() {
        let mut op = glyph_op(None, 0);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.orientation = GlyphRunOrientation::VerticalUpright;
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), op],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert!(!diagnostics.slot_diagnostics[0].strict_variant_available);
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("verticalGlyphOrientationAuthorityPending")
        );
    }

    #[test]
    fn producer_diagnostic_reason_takes_priority_over_authority_pending_reason() {
        let mut op = glyph_op(None, 1);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.orientation = GlyphRunOrientation::VerticalUpright;
            run.glyph_transforms = Some(vec![GlyphTransform {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            }]);
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), op],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("missingGlyph")
        );
        assert_eq!(
            diagnostics.slot_diagnostics[0].variants[0]
                .fallback_reason
                .as_deref(),
            Some("missingGlyph")
        );
    }

    #[test]
    fn producer_diagnostic_reason_takes_priority_across_variant_parts() {
        let mut authority_part = glyph_op_part(None, 0, 0, 2);
        if let PaintOp::GlyphRun { run, .. } = &mut authority_part {
            run.glyph_transforms = Some(vec![GlyphTransform {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            }]);
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), authority_part, glyph_op_part(None, 1, 1, 2)],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("missingGlyph")
        );
        assert_eq!(
            diagnostics.slot_diagnostics[0].variants[0]
                .fallback_reason
                .as_deref(),
            Some("missingGlyph")
        );
    }

    #[test]
    fn glyph_outline_producer_diagnostic_reason_takes_priority_over_payload_reason() {
        let mut op = glyph_outline_op();
        if let PaintOp::GlyphOutline { outline, .. } = &mut op {
            outline.paths.clear();
            outline.diagnostics.missing_glyph_count = 1;
        }
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![text_op("A"), op],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert!(diagnostics.has_errors());
        assert_eq!(
            diagnostics.slot_diagnostics[0].fallback_reason.as_deref(),
            Some("missingGlyph")
        );
        assert_eq!(
            diagnostics.slot_diagnostics[0].variants[0]
                .fallback_reason
                .as_deref(),
            Some("missingGlyph")
        );
    }

    #[test]
    fn reports_line_break_risk_context_for_complex_text_runs() {
        let mut run = text_run("A\tB");
        run.style.inline_tabs.push([0, 0, 0, 0, 0, 0, 0]);
        run.is_line_break_end = true;
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                    run,
                )],
            ),
        );
        let diagnostics = TextV2Diagnostics::from_layer_tree(&tree);
        assert_eq!(diagnostics.line_break_risks.len(), 1);
        assert_eq!(
            diagnostics.line_break_risks[0].risk,
            TextV2LineBreakRiskLevel::ChangeLikely
        );
        assert!(diagnostics.line_break_risks[0]
            .reasons
            .contains(&"tabLeaderOrInlineTab"));
    }

    #[test]
    fn keeps_line_break_risk_report_only_when_strict_variant_exists() {
        let mut run = text_run("A\tB");
        run.style.inline_tabs.push([0, 0, 0, 0, 0, 0, 0]);
        let mut tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 100.0, 100.0),
                None,
                vec![
                    PaintOp::text_run(BoundingBox::new(0.0, 0.0, 20.0, 20.0), run),
                    glyph_op_for_text("A\tB", None, 0),
                ],
            ),
        );
        add_portable_font_resources(&mut tree.resources);
        let diagnostics = TextV2Diagnostics::from_layer_tree_with_profile(
            &tree,
            TextV2CompatibilityProfile::FallbackFreeStrict,
        );

        assert_eq!(diagnostics.line_break_risks.len(), 1);
        assert!(diagnostics.slot_diagnostics[0].strict_variant_available);
        assert!(!diagnostics.has_errors());
    }
}
