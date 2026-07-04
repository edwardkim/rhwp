use serde::Serialize;
use std::collections::BTreeMap;

use crate::model::image::ImageEffect;
use crate::model::shape::TextWrap;
use crate::model::style::{ImageFillMode, UnderlineType};
use crate::paint::{
    paint_op_replay_plane_with_layer, CacheHint, ClipKind, LayerNode, LayerNodeKind, PageLayerTree,
    PaintOp, PaintReplayPlane, ResolvedImageKind, ResolvedImagePayload, TextDecorationKind,
    TextVariantKind,
};
use crate::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantSelectedReason,
    VariantSelectionBackend,
};
use crate::renderer::render_tree::{
    FieldMarkerType, ImageNode, PageBackgroundNode, RenderLayerInfo, TextRunNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CanvasKitReplayMode {
    Default,
    Compat,
}

impl CanvasKitReplayMode {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "default" => Some(Self::Default),
            "compat" | "compatibility" => Some(Self::Compat),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Compat => "compat",
        }
    }

    fn policy(self) -> CanvasKitReplayPolicy {
        match self {
            // P17 intentionally keeps both public modes on the same direct replay
            // contract. `compat` is still accepted for API/URL compatibility and
            // future conservative direct-replay tuning, but it must not mean a
            // hidden Canvas2D paint overlay.
            Self::Default | Self::Compat => CanvasKitReplayPolicy::DIRECT_ONLY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanvasKitReplayPolicy {
    hidden_canvas2d_overlay_allowed: bool,
    direct_replay_required: bool,
}

impl CanvasKitReplayPolicy {
    const DIRECT_ONLY: Self = Self {
        hidden_canvas2d_overlay_allowed: false,
        direct_replay_required: true,
    };
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitReplayPlan {
    pub mode: CanvasKitReplayMode,
    pub hidden_canvas2d_overlay_allowed: bool,
    pub direct_replay_required: bool,
    pub summary: CanvasKitReplaySummary,
    pub items: Vec<CanvasKitReplayItem>,
    pub text_variants: Vec<CanvasKitTextVariantReport>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitReplaySummary {
    pub total_items: u32,
    pub direct_items: u32,
    pub direct_required_items: u32,
    pub compat_overlay_items: u32,
    pub text_fallback_items: u32,
    pub unsupported_items: u32,
    pub hidden_overlay_violations: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitReplayItem {
    pub path: String,
    pub op_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_plane: Option<PaintReplayPlane>,
    pub feature: CanvasKitReplayFeature,
    pub status: CanvasKitReplayStatus,
    pub reason: CanvasKitReplayReason,
    pub compat_overlay_allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CanvasKitReplayFeature {
    PageBackground,
    VectorShape,
    RasterImage,
    Equation,
    FormObject,
    RawSvgFragment,
    Placeholder,
    TextRun,
    TextSpecialVisual,
    TextVariant,
    Clip,
    CacheHint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CanvasKitReplayStatus {
    Direct,
    DirectRequired,
    CompatOverlay,
    TextFallback,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CanvasKitReplayReason {
    DirectReplaySupported,
    DirectReplayRequired,
    CompatOverlayAllowed,
    HiddenOverlayForbidden,
    ExplicitTextRunFallback,
    UnsupportedFeature,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitTextVariantReport {
    pub equivalence_group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_variant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_variant_kind: Option<&'static str>,
    pub selected_reason: &'static str,
    pub fallback_required: bool,
    pub rejected_variants: Vec<CanvasKitRejectedTextVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitRejectedTextVariant {
    pub variant_id: String,
    pub variant_kind: &'static str,
    pub reasons: Vec<&'static str>,
}

pub fn analyze_canvaskit_replay_plan(
    tree: &PageLayerTree,
    mode: CanvasKitReplayMode,
) -> CanvasKitReplayPlan {
    let variant_reports = analyze_text_variant_selection(
        tree,
        TextVariantSelectionOptions {
            backend: VariantSelectionBackend::CanvasKit,
            allow_colrv1_stage1_color_graph: true,
            ..TextVariantSelectionOptions::canvaskit()
        },
    );
    let selected_variants = variant_reports
        .iter()
        .filter_map(|report| {
            let variant_id = report.selected_variant_id.as_ref()?;
            let variant_kind = report.selected_variant_kind?;
            Some((
                report.equivalence_group.clone(),
                SelectedTextVariant {
                    variant_id: variant_id.clone(),
                    variant_kind,
                    fallback_required: report.fallback_required,
                },
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let mut builder = CanvasKitReplayPlanBuilder::new(mode, selected_variants);
    builder.visit_node(&tree.root, "root", None);
    let text_variants = variant_reports
        .into_iter()
        .map(|report| CanvasKitTextVariantReport {
            equivalence_group: report.equivalence_group,
            selected_variant_id: report.selected_variant_id,
            selected_variant_kind: report.selected_variant_kind.map(TextVariantKind::as_str),
            selected_reason: selected_reason_as_str(report.selected_reason),
            fallback_required: report.fallback_required,
            rejected_variants: report
                .rejected_variants
                .into_iter()
                .map(|rejected| CanvasKitRejectedTextVariant {
                    variant_id: rejected.variant_id,
                    variant_kind: rejected.variant_kind.as_str(),
                    reasons: rejected
                        .reasons
                        .into_iter()
                        .map(|reason| reason.as_str())
                        .collect(),
                })
                .collect(),
        })
        .collect();
    builder.finish(text_variants)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedTextVariant {
    variant_id: String,
    variant_kind: TextVariantKind,
    fallback_required: bool,
}

struct CanvasKitReplayPlanBuilder {
    mode: CanvasKitReplayMode,
    policy: CanvasKitReplayPolicy,
    selected_variants: BTreeMap<String, SelectedTextVariant>,
    summary: CanvasKitReplaySummary,
    items: Vec<CanvasKitReplayItem>,
}

impl CanvasKitReplayPlanBuilder {
    fn new(
        mode: CanvasKitReplayMode,
        selected_variants: BTreeMap<String, SelectedTextVariant>,
    ) -> Self {
        Self {
            mode,
            policy: mode.policy(),
            selected_variants,
            summary: CanvasKitReplaySummary::default(),
            items: Vec::new(),
        }
    }

    fn finish(self, text_variants: Vec<CanvasKitTextVariantReport>) -> CanvasKitReplayPlan {
        CanvasKitReplayPlan {
            mode: self.mode,
            hidden_canvas2d_overlay_allowed: self.policy.hidden_canvas2d_overlay_allowed,
            direct_replay_required: self.policy.direct_replay_required,
            summary: self.summary,
            items: self.items,
            text_variants,
        }
    }

    fn visit_node(
        &mut self,
        node: &LayerNode,
        path: &str,
        inherited_layer: Option<RenderLayerInfo>,
    ) {
        let active_layer = node.layer.or(inherited_layer);
        match &node.kind {
            LayerNodeKind::Group {
                children,
                cache_hint,
                ..
            } => {
                if !matches!(cache_hint, CacheHint::None) {
                    self.push_cache_hint_item(path, *cache_hint);
                }
                for (index, child) in children.iter().enumerate() {
                    self.visit_node(child, &format!("{path}/group/{index}"), active_layer);
                }
            }
            LayerNodeKind::ClipRect {
                child, clip_kind, ..
            } => {
                self.push(CanvasKitReplayItem {
                    path: format!("{path}/clip"),
                    op_type: "clipRect",
                    replay_plane: None,
                    feature: CanvasKitReplayFeature::Clip,
                    status: CanvasKitReplayStatus::Direct,
                    reason: CanvasKitReplayReason::DirectReplaySupported,
                    compat_overlay_allowed: false,
                    detail: Some(clip_kind_detail(*clip_kind).to_string()),
                });
                self.visit_node(child, &format!("{path}/clip/child"), active_layer);
            }
            LayerNodeKind::Leaf { ops } => {
                for (index, op) in ops.iter().enumerate() {
                    self.push(self.item_for_op(op, format!("{path}/leaf/{index}"), active_layer));
                }
            }
        }
    }

    fn push_cache_hint_item(&mut self, path: &str, cache_hint: CacheHint) {
        let (status, reason, compat_overlay_allowed) =
            if self.policy.hidden_canvas2d_overlay_allowed {
                (
                    CanvasKitReplayStatus::CompatOverlay,
                    CanvasKitReplayReason::CompatOverlayAllowed,
                    true,
                )
            } else {
                (
                    CanvasKitReplayStatus::DirectRequired,
                    CanvasKitReplayReason::HiddenOverlayForbidden,
                    false,
                )
            };
        self.push(CanvasKitReplayItem {
            path: format!("{path}/cacheHint"),
            op_type: "cacheHint",
            replay_plane: None,
            feature: CanvasKitReplayFeature::CacheHint,
            status,
            reason,
            compat_overlay_allowed,
            detail: Some(format!("{cache_hint:?}")),
        });
    }

    fn item_for_op(
        &self,
        op: &PaintOp,
        path: String,
        layer: Option<RenderLayerInfo>,
    ) -> CanvasKitReplayItem {
        let mut item = match op {
            PaintOp::PageBackground { background, .. } => {
                self.page_background_item(path, background)
            }
            PaintOp::Line { .. }
            | PaintOp::Rectangle { .. }
            | PaintOp::Ellipse { .. }
            | PaintOp::Path { .. } => {
                direct_item(path, paint_op_type(op), CanvasKitReplayFeature::VectorShape)
            }
            PaintOp::FootnoteMarker { .. } => {
                let mut item = direct_item(
                    path,
                    paint_op_type(op),
                    CanvasKitReplayFeature::TextSpecialVisual,
                );
                item.detail = Some("footnoteMarker".to_string());
                item
            }
            PaintOp::Image {
                image, resolved, ..
            } => self.image_item(path, image, resolved.as_deref()),
            PaintOp::Equation { .. } => {
                let mut item = self.transition_overlay_item(
                    path,
                    "equation",
                    CanvasKitReplayFeature::Equation,
                );
                item.detail = Some("unsupportedDirectReplay".to_string());
                item
            }
            PaintOp::FormObject { .. } => {
                let mut item = direct_item(path, "formObject", CanvasKitReplayFeature::FormObject);
                item.detail = Some("basicStaticReplay".to_string());
                item
            }
            PaintOp::RawSvg { .. } => {
                let mut item = self.transition_overlay_item(
                    path,
                    "rawSvg",
                    CanvasKitReplayFeature::RawSvgFragment,
                );
                item.detail = Some("unsupportedDirectReplay".to_string());
                item
            }
            PaintOp::Placeholder { .. } => {
                let mut item =
                    direct_item(path, "placeholder", CanvasKitReplayFeature::Placeholder);
                item.detail = Some("basicStaticReplay".to_string());
                item
            }
            PaintOp::TextRun { run, .. } => self.text_run_item(path, run),
            PaintOp::CharOverlap { .. }
            | PaintOp::TextControlMark { .. }
            | PaintOp::TabLeader { .. }
            | PaintOp::TextDecoration { .. } => self.transition_overlay_item(
                path,
                paint_op_type(op),
                CanvasKitReplayFeature::TextSpecialVisual,
            ),
            PaintOp::GlyphRun { run, .. } => self.text_variant_item(
                path,
                "glyphRun",
                &run.variant.equivalence_group,
                &run.variant.variant_id,
                TextVariantKind::GlyphRun,
            ),
            PaintOp::GlyphOutline { outline, .. } => self.text_variant_item(
                path,
                "glyphOutline",
                &outline.variant.equivalence_group,
                &outline.variant.variant_id,
                TextVariantKind::GlyphOutline,
            ),
        };
        item.replay_plane = Some(paint_op_replay_plane_with_layer(op, layer));
        item
    }

    fn page_background_item(
        &self,
        path: String,
        background: &PageBackgroundNode,
    ) -> CanvasKitReplayItem {
        if background.image.is_some() {
            let mut item = self.transition_overlay_item(
                path,
                "pageBackground",
                CanvasKitReplayFeature::RasterImage,
            );
            item.detail = Some("imageFill".to_string());
            item
        } else if background.gradient.is_some() {
            let mut item = self.transition_overlay_item(
                path,
                "pageBackground",
                CanvasKitReplayFeature::PageBackground,
            );
            item.detail = Some("gradientFill".to_string());
            item
        } else {
            direct_item(
                path,
                "pageBackground",
                CanvasKitReplayFeature::PageBackground,
            )
        }
    }

    fn text_run_item(&self, path: String, run: &TextRunNode) -> CanvasKitReplayItem {
        if let Some(detail) = text_run_transition_detail(run) {
            let mut item =
                self.transition_overlay_item(path, "textRun", CanvasKitReplayFeature::TextRun);
            item.detail = Some(detail.to_string());
            item
        } else {
            direct_item(path, "textRun", CanvasKitReplayFeature::TextRun)
        }
    }

    fn image_item(
        &self,
        path: String,
        image: &ImageNode,
        resolved: Option<&ResolvedImagePayload>,
    ) -> CanvasKitReplayItem {
        let detail = image_transition_detail(image, resolved);
        if image_can_replay_directly(image, resolved) {
            let mut item = direct_item(path, "image", CanvasKitReplayFeature::RasterImage);
            item.detail = detail;
            item
        } else {
            let mut item =
                self.transition_overlay_item(path, "image", CanvasKitReplayFeature::RasterImage);
            item.detail = detail;
            item
        }
    }

    fn text_variant_item(
        &self,
        path: String,
        op_type: &'static str,
        equivalence_group: &str,
        variant_id: &str,
        variant_kind: TextVariantKind,
    ) -> CanvasKitReplayItem {
        let selected = self.selected_variants.get(equivalence_group);
        if selected.is_some_and(|selected| {
            !selected.fallback_required
                && selected.variant_id == variant_id
                && selected.variant_kind == variant_kind
        }) {
            return CanvasKitReplayItem {
                path,
                op_type,
                replay_plane: None,
                feature: CanvasKitReplayFeature::TextVariant,
                status: CanvasKitReplayStatus::DirectRequired,
                reason: CanvasKitReplayReason::DirectReplayRequired,
                compat_overlay_allowed: false,
                detail: Some(format!("selectedVariant={variant_id}")),
            };
        }
        CanvasKitReplayItem {
            path,
            op_type,
            replay_plane: None,
            feature: CanvasKitReplayFeature::TextVariant,
            status: CanvasKitReplayStatus::TextFallback,
            reason: CanvasKitReplayReason::ExplicitTextRunFallback,
            compat_overlay_allowed: false,
            detail: Some(format!("fallbackVariantGroup={equivalence_group}")),
        }
    }

    fn transition_overlay_item(
        &self,
        path: String,
        op_type: &'static str,
        feature: CanvasKitReplayFeature,
    ) -> CanvasKitReplayItem {
        if self.policy.hidden_canvas2d_overlay_allowed {
            CanvasKitReplayItem {
                path,
                op_type,
                replay_plane: None,
                feature,
                status: CanvasKitReplayStatus::CompatOverlay,
                reason: CanvasKitReplayReason::CompatOverlayAllowed,
                compat_overlay_allowed: true,
                detail: None,
            }
        } else {
            CanvasKitReplayItem {
                path,
                op_type,
                replay_plane: None,
                feature,
                status: CanvasKitReplayStatus::DirectRequired,
                reason: CanvasKitReplayReason::HiddenOverlayForbidden,
                compat_overlay_allowed: false,
                detail: None,
            }
        }
    }

    fn push(&mut self, item: CanvasKitReplayItem) {
        self.summary.total_items += 1;
        match item.status {
            CanvasKitReplayStatus::Direct => self.summary.direct_items += 1,
            CanvasKitReplayStatus::DirectRequired => self.summary.direct_required_items += 1,
            CanvasKitReplayStatus::CompatOverlay => self.summary.compat_overlay_items += 1,
            CanvasKitReplayStatus::TextFallback => self.summary.text_fallback_items += 1,
            CanvasKitReplayStatus::Unsupported => self.summary.unsupported_items += 1,
        }
        if matches!(item.reason, CanvasKitReplayReason::HiddenOverlayForbidden) {
            self.summary.hidden_overlay_violations += 1;
        }
        self.items.push(item);
    }
}

fn direct_item(
    path: String,
    op_type: &'static str,
    feature: CanvasKitReplayFeature,
) -> CanvasKitReplayItem {
    CanvasKitReplayItem {
        path,
        op_type,
        replay_plane: None,
        feature,
        status: CanvasKitReplayStatus::Direct,
        reason: CanvasKitReplayReason::DirectReplaySupported,
        compat_overlay_allowed: false,
        detail: None,
    }
}

fn paint_op_type(op: &PaintOp) -> &'static str {
    match op {
        PaintOp::PageBackground { .. } => "pageBackground",
        PaintOp::TextRun { .. } => "textRun",
        PaintOp::GlyphRun { .. } => "glyphRun",
        PaintOp::GlyphOutline { .. } => "glyphOutline",
        PaintOp::CharOverlap { .. } => "charOverlap",
        PaintOp::TextControlMark { .. } => "textControlMark",
        PaintOp::TabLeader { .. } => "tabLeader",
        PaintOp::TextDecoration {
            kind: TextDecorationKind::Underline,
            ..
        } => "underline",
        PaintOp::TextDecoration {
            kind: TextDecorationKind::Strikethrough,
            ..
        } => "strikethrough",
        PaintOp::TextDecoration {
            kind: TextDecorationKind::EmphasisDot,
            ..
        } => "emphasisDot",
        PaintOp::FootnoteMarker { .. } => "footnoteMarker",
        PaintOp::Line { .. } => "line",
        PaintOp::Rectangle { .. } => "rectangle",
        PaintOp::Ellipse { .. } => "ellipse",
        PaintOp::Path { .. } => "path",
        PaintOp::Image { .. } => "image",
        PaintOp::Equation { .. } => "equation",
        PaintOp::FormObject { .. } => "formObject",
        PaintOp::Placeholder { .. } => "placeholder",
        PaintOp::RawSvg { .. } => "rawSvg",
    }
}

fn clip_kind_detail(clip_kind: ClipKind) -> &'static str {
    match clip_kind {
        ClipKind::Body => "body",
        ClipKind::TableCell => "tableCell",
        ClipKind::TextBox => "textBox",
        ClipKind::Generic => "generic",
    }
}

fn text_run_transition_detail(run: &TextRunNode) -> Option<&'static str> {
    if run.is_vertical {
        return Some("verticalText");
    }
    if run.char_overlap.is_some() {
        return Some("charOverlap");
    }
    if run.field_marker != FieldMarkerType::None || run.is_para_end || run.is_line_break_end {
        return Some("controlMark");
    }
    if !run.style.tab_leaders.is_empty() {
        return Some("tabLeader");
    }
    if !matches!(run.style.underline, UnderlineType::None) || run.style.strikethrough {
        return Some("textDecoration");
    }
    if run.style.emphasis_dot != 0 {
        return Some("emphasisDot");
    }
    if run.style.outline_type != 0 {
        return Some("outlineTextEffect");
    }
    if run.style.shadow_type != 0 {
        return Some("shadowTextEffect");
    }
    if run.style.emboss {
        return Some("embossTextEffect");
    }
    if run.style.engrave {
        return Some("engraveTextEffect");
    }
    if run.style.superscript {
        return Some("superscriptTextEffect");
    }
    if run.style.subscript {
        return Some("subscriptTextEffect");
    }
    if run.style.shade_color != 0x00FF_FFFF {
        return Some("shadeTextEffect");
    }
    if (run.style.ratio - 1.0).abs() > f64::EPSILON {
        return Some("ratioTextEffect");
    }
    None
}

fn image_transition_detail(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> Option<String> {
    let mut detail = Vec::new();
    let has_replayable_payload = image_has_replayable_payload(image, resolved);
    if let Some(payload) = resolved {
        detail.push(format!(
            "resolved={}",
            resolved_image_kind_detail(payload.kind)
        ));
    }
    if image.external_path.is_some() {
        detail.push("externalImage".to_string());
        if has_replayable_payload {
            detail.push("injectedImageData".to_string());
        } else {
            detail.push("missingImageData".to_string());
        }
    } else if !has_replayable_payload {
        detail.push("missingImageData".to_string());
    }
    if let Some(fill_mode) = image.fill_mode {
        detail.push(format!("fillMode={}", image_fill_mode_detail(fill_mode)));
    }
    if image.crop.is_some() {
        detail.push("crop".to_string());
    }
    let effects_are_baked = resolved.is_some_and(|payload| payload.suppress_effects);
    if !effects_are_baked && !matches!(image.effect, ImageEffect::RealPic) {
        detail.push(format!("effect={}", image_effect_detail(image.effect)));
    }
    if !effects_are_baked && (image.brightness != 0 || image.contrast != 0) {
        detail.push(format!(
            "adjustment=brightness:{},contrast:{}",
            image.brightness, image.contrast
        ));
    }
    if let Some(wrap) = image.text_wrap {
        detail.push(format!("wrap={}", text_wrap_detail(wrap)));
    }
    if image.transform.has_transform() {
        detail.push("transform".to_string());
    }
    if image.header_footer_ref.is_some() {
        detail.push("headerFooterImage".to_string());
    }
    if detail.is_empty() {
        None
    } else {
        Some(detail.join(";"))
    }
}

fn image_can_replay_directly(image: &ImageNode, resolved: Option<&ResolvedImagePayload>) -> bool {
    let has_replayable_payload = image_has_replayable_payload(image, resolved);
    let effects_are_supported = resolved.is_some_and(|payload| payload.suppress_effects)
        || (matches!(image.effect, ImageEffect::RealPic)
            && image.brightness == 0
            && image.contrast == 0);
    has_replayable_payload && effects_are_supported
}

fn image_has_replayable_payload(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> bool {
    resolved.is_some() || image.data.as_ref().is_some_and(|data| !data.is_empty())
}

fn resolved_image_kind_detail(value: ResolvedImageKind) -> &'static str {
    match value {
        ResolvedImageKind::FormatConverted => "formatConverted",
        ResolvedImageKind::BakedWatermark => "bakedWatermark",
    }
}

fn image_effect_detail(value: ImageEffect) -> &'static str {
    match value {
        ImageEffect::RealPic => "realPic",
        ImageEffect::GrayScale => "grayScale",
        ImageEffect::BlackWhite => "blackWhite",
        ImageEffect::Pattern8x8 => "pattern8x8",
    }
}

fn image_fill_mode_detail(value: ImageFillMode) -> &'static str {
    match value {
        ImageFillMode::TileAll => "tileAll",
        ImageFillMode::TileHorzTop => "tileHorzTop",
        ImageFillMode::TileHorzBottom => "tileHorzBottom",
        ImageFillMode::TileVertLeft => "tileVertLeft",
        ImageFillMode::TileVertRight => "tileVertRight",
        ImageFillMode::FitToSize => "fitToSize",
        ImageFillMode::Total => "total",
        ImageFillMode::Center => "center",
        ImageFillMode::CenterTop => "centerTop",
        ImageFillMode::CenterBottom => "centerBottom",
        ImageFillMode::LeftCenter => "leftCenter",
        ImageFillMode::LeftTop => "leftTop",
        ImageFillMode::LeftBottom => "leftBottom",
        ImageFillMode::RightCenter => "rightCenter",
        ImageFillMode::RightTop => "rightTop",
        ImageFillMode::RightBottom => "rightBottom",
        ImageFillMode::None => "none",
    }
}

fn text_wrap_detail(value: TextWrap) -> &'static str {
    match value {
        TextWrap::Square => "square",
        TextWrap::Tight => "tight",
        TextWrap::Through => "through",
        TextWrap::TopAndBottom => "topAndBottom",
        TextWrap::BehindText => "behindText",
        TextWrap::InFrontOfText => "inFrontOfText",
    }
}

fn selected_reason_as_str(reason: VariantSelectedReason) -> &'static str {
    reason.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::FormType;
    use crate::model::style::ImageFillMode;
    use crate::paint::{GroupKind, LayerNode, ResolvedImageKind, ResolvedImagePayload};
    use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
    use crate::renderer::render_tree::{
        BoundingBox, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
        PageBackgroundImage, PlaceholderNode, RawSvgNode, RectangleNode, RenderLayerInfo,
    };
    use crate::renderer::{GradientFillInfo, ShapeStyle, TextStyle};

    fn bbox() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 20.0, 20.0)
    }

    fn tree_with_ops(ops: Vec<PaintOp>) -> PageLayerTree {
        PageLayerTree::new(100.0, 100.0, LayerNode::leaf(bbox(), None, ops))
    }

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

    fn page_background(
        image: Option<PageBackgroundImage>,
        gradient: Option<Box<GradientFillInfo>>,
    ) -> PageBackgroundNode {
        PageBackgroundNode {
            background_color: None,
            border_color: None,
            border_width: 0.0,
            gradient,
            image,
        }
    }

    fn equation_node() -> EquationNode {
        EquationNode {
            svg_content: "<text>x</text>".to_string(),
            layout_box: LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 12.0,
                baseline: 10.0,
                kind: LayoutKind::Text("x".to_string()),
            },
            color_str: "#000000".to_string(),
            color: 0x00000000,
            font_size: 12.0,
            section_index: None,
            para_index: None,
            control_index: None,
            cell_index: None,
            cell_para_index: None,
            note_ref: None,
        }
    }

    #[test]
    fn default_mode_reports_simple_image_as_direct() {
        let tree = tree_with_ops(vec![PaintOp::image(
            bbox(),
            ImageNode::new(1, Some(vec![1, 2, 3])),
            None,
        )]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_items, 1);
        assert_eq!(plan.summary.direct_required_items, 0);
        assert_eq!(plan.summary.compat_overlay_items, 0);
        assert_eq!(plan.summary.hidden_overlay_violations, 0);
        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            plan.items[0].reason,
            CanvasKitReplayReason::DirectReplaySupported
        );
        assert!(!plan.items[0].compat_overlay_allowed);
    }

    #[test]
    fn compat_mode_reports_simple_image_as_direct() {
        let tree = tree_with_ops(vec![PaintOp::image(
            bbox(),
            ImageNode::new(1, Some(vec![1, 2, 3])),
            None,
        )]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);

        assert!(!plan.hidden_canvas2d_overlay_allowed);
        assert!(plan.direct_replay_required);
        assert_eq!(plan.summary.direct_items, 1);
        assert_eq!(plan.summary.direct_required_items, 0);
        assert_eq!(plan.summary.compat_overlay_items, 0);
        assert_eq!(plan.summary.hidden_overlay_violations, 0);
        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            plan.items[0].reason,
            CanvasKitReplayReason::DirectReplaySupported
        );
        assert!(!plan.items[0].compat_overlay_allowed);
    }

    #[test]
    fn image_replay_plan_reports_direct_geometry_payload() {
        let mut image = ImageNode::new(1, Some(vec![1, 2, 3]));
        image.fill_mode = Some(ImageFillMode::Center);
        image.crop = Some((10, 20, 90, 80));
        image.transform.rotation = 15.0;

        let tree = tree_with_ops(vec![PaintOp::image(bbox(), image, None)]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("fillMode=center;crop;transform")
        );
    }

    #[test]
    fn image_replay_plan_reports_unimplemented_image_effects() {
        let mut image = ImageNode::new(1, Some(vec![1, 2, 3]));
        image.effect = ImageEffect::GrayScale;
        image.brightness = 10;
        image.contrast = -20;

        let tree = tree_with_ops(vec![PaintOp::image(bbox(), image, None)]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::DirectRequired);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("effect=grayScale;adjustment=brightness:10,contrast:-20")
        );
    }

    #[test]
    fn image_replay_plan_treats_baked_watermark_payload_as_direct() {
        let mut image = ImageNode::new(1, Some(vec![1, 2, 3]));
        image.effect = ImageEffect::GrayScale;
        image.brightness = 70;
        image.contrast = -50;

        let tree = tree_with_ops(vec![PaintOp::image(
            bbox(),
            image,
            Some(ResolvedImagePayload {
                data: vec![4, 5, 6],
                mime: "image/png",
                kind: ResolvedImageKind::BakedWatermark,
                suppress_effects: true,
            }),
        )]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("resolved=bakedWatermark")
        );
    }

    #[test]
    fn replay_plan_items_expose_paint_replay_planes() {
        let mut behind = ImageNode::new(1, Some(vec![1, 2, 3]));
        behind.text_wrap = Some(TextWrap::BehindText);
        let mut front = ImageNode::new(2, Some(vec![4, 5, 6]));
        front.text_wrap = Some(TextWrap::InFrontOfText);

        let tree = tree_with_ops(vec![
            PaintOp::page_background(bbox(), page_background(None, None)),
            PaintOp::image(bbox(), behind, None),
            PaintOp::text_run(bbox(), text_run("A")),
            PaintOp::image(bbox(), front, None),
        ]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(
            plan.items
                .iter()
                .map(|item| item.replay_plane)
                .collect::<Vec<_>>(),
            vec![
                Some(PaintReplayPlane::Background),
                Some(PaintReplayPlane::BehindText),
                Some(PaintReplayPlane::Flow),
                Some(PaintReplayPlane::InFrontOfText),
            ]
        );
    }

    #[test]
    fn replay_plan_uses_layer_metadata_for_non_image_ops() {
        let layered_rect = LayerNode::leaf(
            bbox(),
            None,
            vec![PaintOp::rectangle(
                bbox(),
                RectangleNode::new(0.0, ShapeStyle::default(), None),
            )],
        )
        .with_layer(Some(RenderLayerInfo::new(Some(TextWrap::BehindText), 1, 1)));
        let flow_text =
            LayerNode::leaf(bbox(), None, vec![PaintOp::text_run(bbox(), text_run("A"))]);
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::group(
                bbox(),
                None,
                vec![flow_text, layered_rect],
                CacheHint::None,
                GroupKind::Generic,
            ),
        );

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(
            plan.items
                .iter()
                .map(|item| item.replay_plane)
                .collect::<Vec<_>>(),
            vec![
                Some(PaintReplayPlane::Flow),
                Some(PaintReplayPlane::BehindText)
            ]
        );
    }

    #[test]
    fn image_replay_plan_reports_external_path_with_embedded_data() {
        let mut image = ImageNode::new(1, Some(vec![1, 2, 3]));
        image.external_path = Some("linked-image.png".to_string());

        let tree = tree_with_ops(vec![PaintOp::image(bbox(), image, None)]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("externalImage;injectedImageData")
        );
    }

    #[test]
    fn image_replay_plan_reports_external_path_without_payload_as_missing() {
        let mut image = ImageNode::new(1, None);
        image.external_path = Some("linked-image.png".to_string());

        let tree = tree_with_ops(vec![PaintOp::image(bbox(), image, None)]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::DirectRequired);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("externalImage;missingImageData")
        );
    }

    #[test]
    fn page_background_image_and_gradient_are_policy_visible() {
        let image_background = page_background(
            Some(PageBackgroundImage {
                data: vec![1, 2, 3],
                fill_mode: ImageFillMode::FitToSize,
                brightness: 0,
                contrast: 0,
                effect: crate::model::image::ImageEffect::RealPic,
            }),
            None,
        );
        let gradient_background = page_background(
            None,
            Some(Box::new(GradientFillInfo {
                gradient_type: 1,
                angle: 0,
                center_x: 50,
                center_y: 50,
                colors: vec![0x0000_0000, 0x00FF_FFFF],
                positions: vec![0.0, 1.0],
            })),
        );
        let tree = tree_with_ops(vec![
            PaintOp::page_background(bbox(), image_background),
            PaintOp::page_background(bbox(), gradient_background),
        ]);

        let default_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(default_plan.summary.direct_required_items, 2);
        assert_eq!(
            default_plan.items[0].feature,
            CanvasKitReplayFeature::RasterImage
        );
        assert_eq!(default_plan.items[0].detail.as_deref(), Some("imageFill"));
        assert_eq!(
            default_plan.items[1].feature,
            CanvasKitReplayFeature::PageBackground
        );
        assert_eq!(
            default_plan.items[1].detail.as_deref(),
            Some("gradientFill")
        );

        let compat_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);
        assert!(!compat_plan.hidden_canvas2d_overlay_allowed);
        assert!(compat_plan.direct_replay_required);
        assert_eq!(compat_plan.summary.direct_required_items, 2);
        assert_eq!(compat_plan.summary.compat_overlay_items, 0);
    }

    #[test]
    fn object_fragment_replay_gaps_use_runtime_diagnostic_detail() {
        let tree = tree_with_ops(vec![
            PaintOp::equation(bbox(), equation_node()),
            PaintOp::raw_svg(
                bbox(),
                RawSvgNode {
                    svg: "<g><path d=\"M0 0H1\"/></g>".to_string(),
                },
            ),
        ]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_required_items, 2);
        assert_eq!(plan.items[0].op_type, "equation");
        assert_eq!(plan.items[0].feature, CanvasKitReplayFeature::Equation);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("unsupportedDirectReplay")
        );
        assert_eq!(plan.items[1].op_type, "rawSvg");
        assert_eq!(
            plan.items[1].feature,
            CanvasKitReplayFeature::RawSvgFragment
        );
        assert_eq!(
            plan.items[1].detail.as_deref(),
            Some("unsupportedDirectReplay")
        );
    }

    #[test]
    fn simple_text_is_direct_but_text_effect_is_policy_visible() {
        let mut rotated = text_run("A");
        rotated.rotation = 15.0;
        let mut vertical = text_run("A");
        vertical.is_vertical = true;
        let mut superscript = text_run("A");
        superscript.style.superscript = true;
        let mut subscript = text_run("A");
        subscript.style.subscript = true;
        let tree = tree_with_ops(vec![
            PaintOp::text_run(bbox(), text_run("A")),
            PaintOp::text_run(bbox(), rotated),
            PaintOp::text_run(bbox(), vertical),
            PaintOp::text_run(bbox(), superscript),
            PaintOp::text_run(bbox(), subscript),
        ]);

        let default_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(default_plan.summary.direct_items, 2);
        assert_eq!(default_plan.summary.direct_required_items, 3);
        assert_eq!(
            default_plan.items[2].detail.as_deref(),
            Some("verticalText")
        );
        assert_eq!(
            default_plan.items[3].detail.as_deref(),
            Some("superscriptTextEffect")
        );
        assert_eq!(
            default_plan.items[4].detail.as_deref(),
            Some("subscriptTextEffect")
        );

        let compat_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);
        assert_eq!(compat_plan.summary.direct_items, 2);
        assert_eq!(compat_plan.summary.direct_required_items, 3);
        assert_eq!(compat_plan.summary.compat_overlay_items, 0);
        assert_eq!(compat_plan.items[2].detail.as_deref(), Some("verticalText"));
        assert_eq!(
            compat_plan.items[3].detail.as_deref(),
            Some("superscriptTextEffect")
        );
        assert_eq!(
            compat_plan.items[4].detail.as_deref(),
            Some("subscriptTextEffect")
        );
    }

    #[test]
    fn text_run_op_type_matches_layer_tree_schema_name() {
        let tree = tree_with_ops(vec![PaintOp::text_run(bbox(), text_run("A"))]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].op_type, "textRun");
    }

    #[test]
    fn static_canvaskit_runtime_ops_are_reported_as_direct() {
        let tree = tree_with_ops(vec![PaintOp::footnote_marker(
            bbox(),
            FootnoteMarkerNode {
                number: 1,
                text: "1)".to_string(),
                base_font_size: 12.0,
                font_family: "Test".to_string(),
                color: 0x0000_0000,
                section_index: 0,
                para_index: 0,
                control_index: 0,
            },
        )]);

        let default_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(
            default_plan.items[0].feature,
            CanvasKitReplayFeature::TextSpecialVisual
        );
        assert_eq!(default_plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            default_plan.items[0].detail.as_deref(),
            Some("footnoteMarker")
        );

        let compat_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);
        assert_eq!(compat_plan.items[0].status, CanvasKitReplayStatus::Direct);
    }

    #[test]
    fn form_and_placeholder_match_studio_direct_runtime_branches() {
        let form = FormObjectNode {
            form_type: FormType::CheckBox,
            caption: "Agree".to_string(),
            text: String::new(),
            fore_color: "#111111".to_string(),
            back_color: "#ffffff".to_string(),
            value: 1,
            enabled: true,
            section_index: 0,
            para_index: 0,
            control_index: 0,
            name: "check1".to_string(),
            cell_location: None,
        };
        let placeholder = PlaceholderNode {
            fill_color: 0x00FF_FFFF,
            stroke_color: 0x0000_0000,
            label: "OLE".to_string(),
        };
        let tree = tree_with_ops(vec![
            PaintOp::form_object(bbox(), form),
            PaintOp::placeholder(bbox(), placeholder),
        ]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(
            plan.items
                .iter()
                .map(|item| item.status)
                .collect::<Vec<_>>(),
            vec![CanvasKitReplayStatus::Direct, CanvasKitReplayStatus::Direct]
        );
        assert_eq!(
            plan.items
                .iter()
                .map(|item| item.detail.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("basicStaticReplay"), Some("basicStaticReplay")]
        );
    }

    #[test]
    fn mode_parser_defaults_empty_string() {
        assert_eq!(
            CanvasKitReplayMode::from_str(""),
            Some(CanvasKitReplayMode::Default)
        );
        assert_eq!(
            CanvasKitReplayMode::from_str("compatibility"),
            Some(CanvasKitReplayMode::Compat)
        );
        assert_eq!(CanvasKitReplayMode::from_str("canvas2d"), None);
    }

    #[test]
    fn replay_plan_serializes_mode_and_summary() {
        let tree = tree_with_ops(vec![PaintOp::text_run(bbox(), text_run("A"))]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        let json = serde_json::to_string(&plan).expect("serialize CanvasKit replay plan");

        assert!(json.contains("\"mode\":\"default\""));
        assert!(json.contains("\"directItems\":1"));
        assert!(json.contains("\"hiddenCanvas2dOverlayAllowed\":false"));
        assert!(json.contains("\"replayPlane\":\"flow\""));
    }
}
