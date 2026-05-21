use serde::Serialize;
use std::collections::BTreeMap;

use crate::model::style::UnderlineType;
use crate::paint::{
    CacheHint, ClipKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp, TextDecorationKind,
    TextVariantKind,
};
use crate::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantSelectedReason,
    VariantSelectionBackend,
};
use crate::renderer::render_tree::{FieldMarkerType, PageBackgroundNode, TextRunNode};

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
    builder.visit_node(&tree.root, "root");
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

    fn visit_node(&mut self, node: &LayerNode, path: &str) {
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
                    self.visit_node(child, &format!("{path}/group/{index}"));
                }
            }
            LayerNodeKind::ClipRect {
                child, clip_kind, ..
            } => {
                self.push(CanvasKitReplayItem {
                    path: format!("{path}/clip"),
                    op_type: "clipRect",
                    feature: CanvasKitReplayFeature::Clip,
                    status: CanvasKitReplayStatus::Direct,
                    reason: CanvasKitReplayReason::DirectReplaySupported,
                    compat_overlay_allowed: false,
                    detail: Some(clip_kind_detail(*clip_kind).to_string()),
                });
                self.visit_node(child, &format!("{path}/clip/child"));
            }
            LayerNodeKind::Leaf { ops } => {
                for (index, op) in ops.iter().enumerate() {
                    self.push(self.item_for_op(op, format!("{path}/leaf/{index}")));
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
            feature: CanvasKitReplayFeature::CacheHint,
            status,
            reason,
            compat_overlay_allowed,
            detail: Some(format!("{cache_hint:?}")),
        });
    }

    fn item_for_op(&self, op: &PaintOp, path: String) -> CanvasKitReplayItem {
        match op {
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
                let mut item = self.transition_overlay_item(
                    path,
                    paint_op_type(op),
                    CanvasKitReplayFeature::TextSpecialVisual,
                );
                item.detail = Some("footnoteMarker".to_string());
                item
            }
            PaintOp::Image { .. } => {
                self.transition_overlay_item(path, "image", CanvasKitReplayFeature::RasterImage)
            }
            PaintOp::Equation { .. } => {
                self.transition_overlay_item(path, "equation", CanvasKitReplayFeature::Equation)
            }
            PaintOp::FormObject { .. } => {
                self.transition_overlay_item(path, "formObject", CanvasKitReplayFeature::FormObject)
            }
            PaintOp::RawSvg { .. } => {
                self.transition_overlay_item(path, "rawSvg", CanvasKitReplayFeature::RawSvgFragment)
            }
            PaintOp::Placeholder { .. } => self.transition_overlay_item(
                path,
                "placeholder",
                CanvasKitReplayFeature::Placeholder,
            ),
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
        }
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
        ClipKind::Generic => "generic",
    }
}

fn text_run_transition_detail(run: &TextRunNode) -> Option<&'static str> {
    if run.is_vertical {
        return Some("verticalText");
    }
    if run.rotation.abs() > f64::EPSILON {
        return Some("rotatedText");
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
    if run.style.shade_color != 0x00FF_FFFF {
        return Some("shadeTextEffect");
    }
    if (run.style.ratio - 1.0).abs() > f64::EPSILON {
        return Some("ratioTextEffect");
    }
    None
}

fn selected_reason_as_str(reason: VariantSelectedReason) -> &'static str {
    reason.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::style::ImageFillMode;
    use crate::paint::LayerNode;
    use crate::renderer::render_tree::{
        BoundingBox, FootnoteMarkerNode, ImageNode, PageBackgroundImage,
    };
    use crate::renderer::{GradientFillInfo, TextStyle};

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

    #[test]
    fn default_mode_forbids_hidden_image_overlay() {
        let tree = tree_with_ops(vec![PaintOp::Image {
            bbox: bbox(),
            image: ImageNode::new(1, Some(vec![1, 2, 3])),
            resolved: None,
        }]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_required_items, 1);
        assert_eq!(plan.summary.compat_overlay_items, 0);
        assert_eq!(plan.summary.hidden_overlay_violations, 1);
        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::DirectRequired);
        assert_eq!(
            plan.items[0].reason,
            CanvasKitReplayReason::HiddenOverlayForbidden
        );
        assert!(!plan.items[0].compat_overlay_allowed);
    }

    #[test]
    fn compat_mode_keeps_image_on_direct_replay_contract() {
        let tree = tree_with_ops(vec![PaintOp::Image {
            bbox: bbox(),
            image: ImageNode::new(1, Some(vec![1, 2, 3])),
            resolved: None,
        }]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);

        assert!(!plan.hidden_canvas2d_overlay_allowed);
        assert!(plan.direct_replay_required);
        assert_eq!(plan.summary.direct_required_items, 1);
        assert_eq!(plan.summary.compat_overlay_items, 0);
        assert_eq!(plan.summary.hidden_overlay_violations, 1);
        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::DirectRequired);
        assert_eq!(
            plan.items[0].reason,
            CanvasKitReplayReason::HiddenOverlayForbidden
        );
        assert!(!plan.items[0].compat_overlay_allowed);
    }

    #[test]
    fn page_background_image_and_gradient_are_policy_visible() {
        let image_background = page_background(
            Some(PageBackgroundImage {
                data: vec![1, 2, 3],
                fill_mode: ImageFillMode::FitToSize,
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
            PaintOp::PageBackground {
                bbox: bbox(),
                background: image_background,
            },
            PaintOp::PageBackground {
                bbox: bbox(),
                background: gradient_background,
            },
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
    fn simple_text_is_direct_but_text_effect_is_policy_visible() {
        let mut vertical = text_run("A");
        vertical.is_vertical = true;
        let tree = tree_with_ops(vec![
            PaintOp::TextRun {
                bbox: bbox(),
                run: text_run("A"),
            },
            PaintOp::TextRun {
                bbox: bbox(),
                run: vertical,
            },
        ]);

        let default_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(default_plan.summary.direct_items, 1);
        assert_eq!(default_plan.summary.direct_required_items, 1);
        assert_eq!(
            default_plan.items[1].detail.as_deref(),
            Some("verticalText")
        );

        let compat_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);
        assert_eq!(compat_plan.summary.direct_items, 1);
        assert_eq!(compat_plan.summary.direct_required_items, 1);
        assert_eq!(compat_plan.summary.compat_overlay_items, 0);
        assert_eq!(compat_plan.items[1].detail.as_deref(), Some("verticalText"));
    }

    #[test]
    fn text_run_op_type_matches_layer_tree_schema_name() {
        let tree = tree_with_ops(vec![PaintOp::TextRun {
            bbox: bbox(),
            run: text_run("A"),
        }]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].op_type, "textRun");
    }

    #[test]
    fn footnote_marker_is_reported_as_text_special_visual() {
        let tree = tree_with_ops(vec![PaintOp::FootnoteMarker {
            bbox: bbox(),
            marker: FootnoteMarkerNode {
                number: 1,
                text: "1)".to_string(),
                base_font_size: 12.0,
                font_family: "Test".to_string(),
                color: 0x0000_0000,
                section_index: 0,
                para_index: 0,
                control_index: 0,
            },
        }]);

        let default_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(
            default_plan.items[0].feature,
            CanvasKitReplayFeature::TextSpecialVisual
        );
        assert_eq!(
            default_plan.items[0].status,
            CanvasKitReplayStatus::DirectRequired
        );
        assert_eq!(
            default_plan.items[0].detail.as_deref(),
            Some("footnoteMarker")
        );

        let compat_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);
        assert_eq!(
            compat_plan.items[0].status,
            CanvasKitReplayStatus::DirectRequired
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
        let tree = tree_with_ops(vec![PaintOp::TextRun {
            bbox: bbox(),
            run: text_run("A"),
        }]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        let json = serde_json::to_string(&plan).expect("serialize CanvasKit replay plan");

        assert!(json.contains("\"mode\":\"default\""));
        assert!(json.contains("\"directItems\":1"));
        assert!(json.contains("\"hiddenCanvas2dOverlayAllowed\":false"));
    }
}
