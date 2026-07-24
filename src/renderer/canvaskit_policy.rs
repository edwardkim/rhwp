use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::model::image::ImageEffect;
use crate::model::shape::TextWrap;
use crate::model::style::ImageFillMode;
use crate::paint::{
    paint_op_replay_plane_with_layer, CacheHint, ClipKind, LayerNode, LayerNodeKind, PageLayerTree,
    PaintOp, PaintReplayPlane, RenderProfile, ResolvedImageKind, ResolvedImagePayload,
    TextDecorationKind, TextVariantKind,
};
use crate::renderer::composer::expand_pua_display_text;
use crate::renderer::equation::{
    layout::{LayoutBox, LayoutKind},
    symbols::{DecoKind, FontStyleKind},
};
use crate::renderer::image_header::canvaskit_encoded_image_header;
use crate::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantSelectedReason,
    VariantSelectionBackend,
};
use crate::renderer::layout::compute_char_positions;
use crate::renderer::render_tree::{
    EllipseNode, ImageNode, LineNode, PageBackgroundNode, PageRenderTree, PathNode, RectangleNode,
    RenderLayerInfo, RenderNodeType, TextRunNode,
};
use crate::renderer::{ArrowStyle, LineRenderType, ShapeStyle, StrokeDash};

const OLD_HANGUL_FONT_FAMILY: &str = "Source Han Serif K Old Hangul";

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
    pub required_font_families: Vec<String>,
    pub required_font_families_complete: bool,
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

pub const CANVASKIT_DOCUMENT_PREFLIGHT_SCHEMA_VERSION: u32 = 1;
pub const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_PAGES: u32 = 128;
pub const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_WORK_UNITS: u32 = 50_000;
pub const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_BLOCKERS: u32 = 32;
pub const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_REQUIRED_FONT_FAMILIES: u32 = 256;

const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_DETAIL_BYTES: usize = 256;
const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_FONT_FAMILY_BYTES: usize = 256;
const CANVASKIT_DOCUMENT_PREFLIGHT_WORK_UNIT_BYTES: usize = 4 * 1024;
const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_TREE_DEPTH: usize = 256;
const CANVASKIT_DOCUMENT_PREFLIGHT_PRELOWER_UNIT_BYTES: usize = 1024;
const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_RENDER_TREE_DEPTH: usize = 128;
const CANVASKIT_DOCUMENT_PREFLIGHT_MAX_TEXT_BYTES: usize = 1024 * 1024;
const CANVASKIT_MAX_ENCODED_IMAGE_BASE64_BYTES: usize = 24 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitDocumentPreflightLimits {
    pub max_pages: u32,
    pub max_work_units: u32,
    pub max_blockers: u32,
    pub max_required_font_families: u32,
}

impl CanvasKitDocumentPreflightLimits {
    pub const FIXED: Self = Self {
        max_pages: CANVASKIT_DOCUMENT_PREFLIGHT_MAX_PAGES,
        max_work_units: CANVASKIT_DOCUMENT_PREFLIGHT_MAX_WORK_UNITS,
        max_blockers: CANVASKIT_DOCUMENT_PREFLIGHT_MAX_BLOCKERS,
        max_required_font_families: CANVASKIT_DOCUMENT_PREFLIGHT_MAX_REQUIRED_FONT_FAMILIES,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CanvasKitDocumentPreflightStatus {
    Eligible,
    Ineligible,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CanvasKitDocumentPreflightBlockerCode {
    PageLimitExceeded,
    WorkLimitExceeded,
    PageBuildFailed,
    HiddenCanvas2dOverlayRequired,
    Unsupported,
    TextFallback,
    CompatOverlay,
}

impl CanvasKitDocumentPreflightBlockerCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::PageLimitExceeded => "pageLimitExceeded",
            Self::WorkLimitExceeded => "workLimitExceeded",
            Self::PageBuildFailed => "pageBuildFailed",
            Self::HiddenCanvas2dOverlayRequired => "hiddenCanvas2dOverlayRequired",
            Self::Unsupported => "unsupported",
            Self::TextFallback => "textFallback",
            Self::CompatOverlay => "compatOverlay",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitDocumentPreflightBlocker {
    pub page_index: u32,
    pub code: CanvasKitDocumentPreflightBlockerCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasKitDocumentPreflight {
    pub schema_version: u32,
    pub mode: CanvasKitReplayMode,
    pub profile: &'static str,
    pub status: CanvasKitDocumentPreflightStatus,
    pub eligible: bool,
    pub complete: bool,
    pub page_count: u32,
    pub scanned_pages: u32,
    pub scanned_work_units: u32,
    pub limits: CanvasKitDocumentPreflightLimits,
    pub summary: CanvasKitReplaySummary,
    pub blockers: Vec<CanvasKitDocumentPreflightBlocker>,
    pub required_font_families: Vec<String>,
    pub capability_digest: String,
}

#[derive(Debug, Clone)]
pub enum CanvasKitPreflightPageBuild {
    Complete {
        tree: Box<PageLayerTree>,
        prelower_work_units: u32,
    },
    WorkLimitExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasKitBoundedWorkCount {
    Complete(u32),
    Exceeded,
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

impl CanvasKitReplayFeature {
    fn as_str(self) -> &'static str {
        match self {
            Self::PageBackground => "pageBackground",
            Self::VectorShape => "vectorShape",
            Self::RasterImage => "rasterImage",
            Self::Equation => "equation",
            Self::FormObject => "formObject",
            Self::RawSvgFragment => "rawSvgFragment",
            Self::Placeholder => "placeholder",
            Self::TextRun => "textRun",
            Self::TextSpecialVisual => "textSpecialVisual",
            Self::TextVariant => "textVariant",
            Self::Clip => "clip",
            Self::CacheHint => "cacheHint",
        }
    }
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

impl CanvasKitReplayStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::DirectRequired => "directRequired",
            Self::CompatOverlay => "compatOverlay",
            Self::TextFallback => "textFallback",
            Self::Unsupported => "unsupported",
        }
    }
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

impl CanvasKitReplayReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::DirectReplaySupported => "directReplaySupported",
            Self::DirectReplayRequired => "directReplayRequired",
            Self::CompatOverlayAllowed => "compatOverlayAllowed",
            Self::HiddenOverlayForbidden => "hiddenOverlayForbidden",
            Self::ExplicitTextRunFallback => "explicitTextRunFallback",
            Self::UnsupportedFeature => "unsupportedFeature",
        }
    }
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

pub fn analyze_canvaskit_document_preflight<F, E>(
    page_count: u32,
    mode: CanvasKitReplayMode,
    profile: RenderProfile,
    build_page: F,
) -> CanvasKitDocumentPreflight
where
    F: FnMut(u32, u32) -> Result<CanvasKitPreflightPageBuild, E>,
    E: fmt::Display,
{
    analyze_canvaskit_document_preflight_with_limits(
        page_count,
        mode,
        profile,
        CanvasKitDocumentPreflightLimits::FIXED,
        build_page,
    )
}

fn analyze_canvaskit_document_preflight_with_limits<F, E>(
    page_count: u32,
    mode: CanvasKitReplayMode,
    profile: RenderProfile,
    limits: CanvasKitDocumentPreflightLimits,
    mut build_page: F,
) -> CanvasKitDocumentPreflight
where
    F: FnMut(u32, u32) -> Result<CanvasKitPreflightPageBuild, E>,
    E: fmt::Display,
{
    let mut preflight = CanvasKitDocumentPreflightAccumulator::new(
        page_count,
        mode,
        render_profile_name(profile),
        limits,
    );

    if page_count > limits.max_pages {
        preflight.mark_incomplete(
            limits.max_pages,
            CanvasKitDocumentPreflightBlockerCode::PageLimitExceeded,
            Some(format!(
                "pageCount={page_count};maxPages={}",
                limits.max_pages
            )),
        );
        return preflight.finish();
    }

    for page_index in 0..page_count {
        let remaining_work_units = limits
            .max_work_units
            .saturating_sub(preflight.scanned_work_units);
        if remaining_work_units == 0 {
            preflight.mark_incomplete(
                page_index,
                CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded,
                Some(format!("maxWorkUnits={}", limits.max_work_units)),
            );
            break;
        }

        let (tree, prelower_work_units) = match build_page(page_index, remaining_work_units) {
            Ok(CanvasKitPreflightPageBuild::Complete {
                tree,
                prelower_work_units,
            }) => (tree, prelower_work_units),
            Ok(CanvasKitPreflightPageBuild::WorkLimitExceeded) => {
                preflight.scanned_work_units = limits.max_work_units;
                preflight.mark_incomplete(
                    page_index,
                    CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded,
                    Some(format!(
                        "stage=preLowering;maxWorkUnits={};remainingWorkUnits={remaining_work_units}",
                        limits.max_work_units
                    )),
                );
                break;
            }
            Err(error) => {
                preflight.mark_incomplete(
                    page_index,
                    CanvasKitDocumentPreflightBlockerCode::PageBuildFailed,
                    Some(error.to_string()),
                );
                break;
            }
        };
        let layer_work_units = match count_layer_tree_work_units(&tree, remaining_work_units) {
            CanvasKitBoundedWorkCount::Complete(work_units) => work_units,
            CanvasKitBoundedWorkCount::Exceeded => {
                preflight.scanned_work_units = limits.max_work_units;
                preflight.mark_incomplete(
                    page_index,
                    CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded,
                    Some(format!(
                        "maxWorkUnits={};remainingWorkUnits={remaining_work_units}",
                        limits.max_work_units
                    )),
                );
                break;
            }
        };
        let page_work_units = prelower_work_units.max(layer_work_units);
        if page_work_units > remaining_work_units {
            preflight.scanned_work_units = limits.max_work_units;
            preflight.mark_incomplete(
                page_index,
                CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded,
                Some(format!(
                    "stage=combined;maxWorkUnits={};remainingWorkUnits={remaining_work_units}",
                    limits.max_work_units
                )),
            );
            break;
        }

        let plan = analyze_canvaskit_replay_plan(&tree, mode);
        if !preflight.record_page(page_index, page_work_units, plan) {
            break;
        }
    }

    preflight.finish()
}

/// Estimates lowering cost while the cached PageRenderTree is still borrowed.
/// This gate runs before embedded-font parsing and PageLayerTree allocation.
pub fn estimate_canvaskit_page_lowering_work(
    tree: &PageRenderTree,
    max_work_units: u32,
) -> CanvasKitBoundedWorkCount {
    let max_work_units = max_work_units as usize;
    let mut work_units = 0usize;
    let mut pending = vec![(&tree.root, 0usize)];

    while let Some((node, depth)) = pending.pop() {
        if !node.visible {
            continue;
        }
        if depth > CANVASKIT_DOCUMENT_PREFLIGHT_MAX_RENDER_TREE_DEPTH {
            return CanvasKitBoundedWorkCount::Exceeded;
        }
        let Some(node_work_units) = render_node_prelower_work_units(&node.node_type) else {
            return CanvasKitBoundedWorkCount::Exceeded;
        };
        let Some(next_work_units) = work_units.checked_add(node_work_units) else {
            return CanvasKitBoundedWorkCount::Exceeded;
        };
        work_units = next_work_units;
        if minimum_work_exceeds_limit(
            work_units,
            pending.len(),
            node.children.len(),
            max_work_units,
        ) {
            return CanvasKitBoundedWorkCount::Exceeded;
        }
        pending.extend(node.children.iter().rev().map(|child| (child, depth + 1)));
    }

    CanvasKitBoundedWorkCount::Complete(work_units as u32)
}

fn render_node_prelower_work_units(node_type: &RenderNodeType) -> Option<usize> {
    let (base_units, payload_bytes, text_like) = match node_type {
        // A TextRun may lower to the fallback plus special-visual and strict
        // text sidecars, so reserve the largest currently supported expansion.
        RenderNodeType::TextRun(run) => (
            10usize,
            run.text
                .len()
                .checked_add(run.style.font_family.len())?
                .checked_add(
                    run.style
                        .tab_stops
                        .len()
                        .checked_mul(std::mem::size_of::<crate::renderer::TabStop>())?,
                )?,
            true,
        ),
        RenderNodeType::Path(path) => (2usize.checked_add(path.commands.len())?, 0, false),
        RenderNodeType::Image(image) => (2, image.data.as_ref().map_or(0, Vec::len), false),
        RenderNodeType::PageBackground(background) => (
            2,
            background
                .image
                .as_ref()
                .map_or(0, |image| image.data.len()),
            false,
        ),
        RenderNodeType::Equation(equation) => (2, equation.svg_content.len(), true),
        RenderNodeType::RawSvg(raw) => (2, raw.svg.len(), true),
        RenderNodeType::FormObject(form) => (
            2,
            form.caption
                .len()
                .checked_add(form.text.len())?
                .checked_add(form.name.len())?,
            true,
        ),
        RenderNodeType::Placeholder(placeholder) => (2, placeholder.label.len(), true),
        RenderNodeType::FootnoteMarker(marker) => (
            2,
            marker.text.len().checked_add(marker.font_family.len())?,
            true,
        ),
        RenderNodeType::Line(_) | RenderNodeType::Rectangle(_) | RenderNodeType::Ellipse(_) => {
            (2, 0, false)
        }
        RenderNodeType::Page(_)
        | RenderNodeType::MasterPage
        | RenderNodeType::Header
        | RenderNodeType::Footer
        | RenderNodeType::Body { .. }
        | RenderNodeType::Column(_)
        | RenderNodeType::FootnoteArea
        | RenderNodeType::TextLine(_)
        | RenderNodeType::Table(_)
        | RenderNodeType::TableCell(_)
        | RenderNodeType::Group(_)
        | RenderNodeType::TextBox => (1, 0, false),
    };
    if text_like && payload_bytes > CANVASKIT_DOCUMENT_PREFLIGHT_MAX_TEXT_BYTES {
        return None;
    }
    base_units.checked_add(payload_bytes.div_ceil(CANVASKIT_DOCUMENT_PREFLIGHT_PRELOWER_UNIT_BYTES))
}

fn count_layer_tree_work_units(
    tree: &PageLayerTree,
    max_work_units: u32,
) -> CanvasKitBoundedWorkCount {
    let max_work_units = max_work_units as usize;
    let resource_bytes = tree
        .resources
        .image_resources()
        .map(|(_, bytes)| bytes.len())
        .chain(
            tree.resources
                .svg_resources()
                .map(|(_, fragment)| fragment.len()),
        )
        .chain(
            tree.resources
                .font_blob_resources()
                .map(|(_, bytes)| bytes.len()),
        )
        .try_fold(0usize, |total, bytes| total.checked_add(bytes));
    let Some(resource_bytes) = resource_bytes else {
        return CanvasKitBoundedWorkCount::Exceeded;
    };
    let mut work_units = payload_work_units(resource_bytes);
    if work_units > max_work_units {
        return CanvasKitBoundedWorkCount::Exceeded;
    }
    let mut pending = vec![(&tree.root, 0usize)];

    while let Some((node, depth)) = pending.pop() {
        if depth > CANVASKIT_DOCUMENT_PREFLIGHT_MAX_TREE_DEPTH {
            return CanvasKitBoundedWorkCount::Exceeded;
        }
        let Some(next_work_units) = work_units.checked_add(1) else {
            return CanvasKitBoundedWorkCount::Exceeded;
        };
        work_units = next_work_units;
        if work_units > max_work_units {
            return CanvasKitBoundedWorkCount::Exceeded;
        }

        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                if minimum_work_exceeds_limit(
                    work_units,
                    pending.len(),
                    children.len(),
                    max_work_units,
                ) {
                    return CanvasKitBoundedWorkCount::Exceeded;
                }
                pending.extend(children.iter().rev().map(|child| (child, depth + 1)));
            }
            LayerNodeKind::ClipRect { child, .. } => {
                if minimum_work_exceeds_limit(work_units, pending.len(), 1, max_work_units) {
                    return CanvasKitBoundedWorkCount::Exceeded;
                }
                pending.push((child, depth + 1));
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    let Some(next_work_units) = work_units.checked_add(paint_op_work_units(op))
                    else {
                        return CanvasKitBoundedWorkCount::Exceeded;
                    };
                    work_units = next_work_units;
                    if work_units > max_work_units {
                        return CanvasKitBoundedWorkCount::Exceeded;
                    }
                }
                if minimum_work_exceeds_limit(work_units, pending.len(), 0, max_work_units) {
                    return CanvasKitBoundedWorkCount::Exceeded;
                }
            }
        }
    }

    CanvasKitBoundedWorkCount::Complete(work_units as u32)
}

fn payload_work_units(bytes: usize) -> usize {
    bytes.div_ceil(CANVASKIT_DOCUMENT_PREFLIGHT_WORK_UNIT_BYTES)
}

fn additional_payload_work_units(bytes: usize) -> usize {
    bytes
        .saturating_sub(1)
        .checked_div(CANVASKIT_DOCUMENT_PREFLIGHT_WORK_UNIT_BYTES)
        .unwrap_or_default()
}

fn paint_op_work_units(op: &PaintOp) -> usize {
    let repeated_visual_units = match op {
        PaintOp::TextRun { run, .. } => run.text.chars().count(),
        PaintOp::CharOverlap { run, .. } => run.text.chars().count(),
        PaintOp::TextControlMark { run, .. } => bounded_text_char_count(&run.text),
        PaintOp::TabLeader { run, .. } => {
            bounded_text_char_count(&run.text).saturating_add(run.style.tab_leaders.len())
        }
        PaintOp::TextDecoration { run, .. } => text_decoration_position_count(run),
        _ => 0,
    };
    let payload_bytes = match op {
        PaintOp::PageBackground { background, .. } => background
            .image
            .as_ref()
            .map_or(0, |image| image.data.len()),
        PaintOp::TextRun { run, .. }
        | PaintOp::CharOverlap { run, .. }
        | PaintOp::TextControlMark { run, .. }
        | PaintOp::TabLeader { run, .. }
        | PaintOp::TextDecoration { run, .. } => run
            .text
            .len()
            .saturating_add(run.style.font_family.len())
            .saturating_add(
                run.style
                    .tab_stops
                    .len()
                    .saturating_mul(std::mem::size_of::<crate::renderer::TabStop>()),
            ),
        PaintOp::GlyphRun { run, .. } => run
            .glyph_ids
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(
                run.positions
                    .len()
                    .saturating_mul(std::mem::size_of::<crate::paint::LayerPoint>()),
            )
            .saturating_add(
                run.clusters
                    .len()
                    .saturating_mul(std::mem::size_of::<crate::paint::GlyphCluster>()),
            ),
        PaintOp::GlyphOutline { outline, .. } => {
            outline.paths.iter().fold(0usize, |total, path| {
                total.saturating_add(
                    path.commands
                        .len()
                        .saturating_mul(std::mem::size_of::<crate::renderer::PathCommand>()),
                )
            })
        }
        PaintOp::Path { path, .. } => path
            .commands
            .len()
            .saturating_mul(std::mem::size_of::<crate::renderer::PathCommand>()),
        PaintOp::Image {
            image, resolved, ..
        } => resolved
            .as_deref()
            .map(|payload| payload.data.len())
            .or_else(|| image.data.as_ref().map(Vec::len))
            .unwrap_or_default(),
        PaintOp::Equation { equation, .. } => equation.svg_content.len(),
        PaintOp::FormObject { form, .. } => form
            .caption
            .len()
            .saturating_add(form.text.len())
            .saturating_add(form.name.len()),
        PaintOp::RawSvg { raw, .. } => raw.svg.len(),
        PaintOp::FootnoteMarker { marker, .. } => {
            marker.text.len().saturating_add(marker.font_family.len())
        }
        PaintOp::Line { .. }
        | PaintOp::Rectangle { .. }
        | PaintOp::Ellipse { .. }
        | PaintOp::Placeholder { .. } => 0,
    };
    1usize
        .saturating_add(additional_payload_work_units(payload_bytes))
        .saturating_add(repeated_visual_units)
}

fn positioned_control_mark_count(run: &TextRunNode) -> usize {
    let inline_marks = if matches!(
        run.field_marker,
        crate::renderer::render_tree::FieldMarkerType::None
    ) {
        run.text
            .chars()
            .filter(|ch| matches!(ch, ' ' | '\t'))
            .count()
    } else {
        0
    };
    inline_marks.saturating_add((run.is_para_end || run.is_line_break_end) as usize)
}

fn bounded_text_char_count(text: &str) -> usize {
    text.chars()
        .take(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1)
        .count()
}

fn text_decoration_position_count(run: &TextRunNode) -> usize {
    let source: String = run
        .text
        .chars()
        .take(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1)
        .collect();
    if source.chars().count() > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN {
        return crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1;
    }
    expand_pua_display_text(&source)
        .chars()
        .take(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1)
        .count()
}

fn text_visual_geometry_is_valid(
    bbox: &crate::renderer::render_tree::BoundingBox,
    run: &TextRunNode,
) -> bool {
    [
        bbox.x,
        bbox.y,
        bbox.width,
        bbox.height,
        run.baseline,
        run.rotation,
        run.style.font_size,
        run.style.ratio,
    ]
    .iter()
    .all(|value| value.is_finite())
        && bbox.width >= 0.0
        && bbox.height >= 0.0
        && run.style.font_size > 0.0
        && run.style.ratio > 0.0
        && compute_char_positions(
            &run.text
                .chars()
                .take(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN)
                .collect::<String>(),
            &run.style,
        )
        .iter()
        .all(|position| position.is_finite())
}

fn minimum_work_exceeds_limit(
    work_units: usize,
    pending_nodes: usize,
    added_nodes: usize,
    max_work_units: usize,
) -> bool {
    work_units
        .checked_add(pending_nodes)
        .and_then(|value| value.checked_add(added_nodes))
        .is_none_or(|minimum| minimum > max_work_units)
}

struct CanvasKitDocumentPreflightAccumulator {
    mode: CanvasKitReplayMode,
    profile: &'static str,
    page_count: u32,
    limits: CanvasKitDocumentPreflightLimits,
    complete: bool,
    scanned_pages: u32,
    scanned_work_units: u32,
    summary: CanvasKitReplaySummary,
    blockers: Vec<CanvasKitDocumentPreflightBlocker>,
    required_font_families: BTreeSet<String>,
    digest: CanvasKitCapabilityDigest,
}

impl CanvasKitDocumentPreflightAccumulator {
    fn new(
        page_count: u32,
        mode: CanvasKitReplayMode,
        profile: &'static str,
        limits: CanvasKitDocumentPreflightLimits,
    ) -> Self {
        Self {
            mode,
            profile,
            page_count,
            limits,
            complete: true,
            scanned_pages: 0,
            scanned_work_units: 0,
            summary: CanvasKitReplaySummary::default(),
            blockers: Vec::new(),
            required_font_families: BTreeSet::new(),
            digest: CanvasKitCapabilityDigest::new(mode, profile, page_count, limits),
        }
    }

    fn record_page(&mut self, page_index: u32, work_units: u32, plan: CanvasKitReplayPlan) -> bool {
        self.scanned_pages = self.scanned_pages.saturating_add(1);
        self.scanned_work_units = self.scanned_work_units.saturating_add(work_units);
        self.summary.merge(&plan.summary);
        self.digest.record_page(page_index, work_units);

        let mut required_font_families_complete = plan.required_font_families_complete;
        for font_family in plan.required_font_families {
            self.digest
                .record_required_font_family(page_index, &font_family);
            if self.required_font_families.contains(&font_family) {
                continue;
            }
            if self.required_font_families.len() >= self.limits.max_required_font_families as usize
            {
                required_font_families_complete = false;
                continue;
            }
            self.required_font_families.insert(font_family);
        }

        for item in plan.items {
            self.digest.record_item(page_index, &item);
            let Some(code) = blocker_code_for_item(&item) else {
                continue;
            };
            self.push_capability_blocker(CanvasKitDocumentPreflightBlocker {
                page_index,
                code,
                op_type: Some(item.op_type),
                detail: item.detail.map(bounded_blocker_detail),
            });
        }
        if !required_font_families_complete {
            self.mark_incomplete(
                page_index,
                CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded,
                Some(format!(
                    "stage=requiredFontFamilies;maxRequiredFontFamilies={}",
                    self.limits.max_required_font_families
                )),
            );
            return false;
        }
        true
    }

    fn push_capability_blocker(&mut self, blocker: CanvasKitDocumentPreflightBlocker) {
        if self.blockers.len() < self.limits.max_blockers as usize {
            self.blockers.push(blocker);
        }
    }

    fn mark_incomplete(
        &mut self,
        page_index: u32,
        code: CanvasKitDocumentPreflightBlockerCode,
        detail: Option<String>,
    ) {
        self.complete = false;
        let detail = detail.map(bounded_blocker_detail);
        self.digest
            .record_incomplete(page_index, code, detail.as_deref());
        let blocker = CanvasKitDocumentPreflightBlocker {
            page_index,
            code,
            op_type: None,
            detail,
        };
        let max_blockers = self.limits.max_blockers as usize;
        if self.blockers.len() < max_blockers {
            self.blockers.push(blocker);
        } else if max_blockers > 0 {
            self.blockers[max_blockers - 1] = blocker;
        }
    }

    fn finish(self) -> CanvasKitDocumentPreflight {
        let eligible = self.complete
            && self.summary.hidden_overlay_violations == 0
            && self.summary.unsupported_items == 0
            && self.summary.text_fallback_items == 0
            && self.summary.compat_overlay_items == 0;
        let status = if !self.complete {
            CanvasKitDocumentPreflightStatus::Incomplete
        } else if eligible {
            CanvasKitDocumentPreflightStatus::Eligible
        } else {
            CanvasKitDocumentPreflightStatus::Ineligible
        };
        let capability_digest = self.digest.finish(
            status,
            self.complete,
            self.scanned_pages,
            self.scanned_work_units,
            &self.summary,
        );

        CanvasKitDocumentPreflight {
            schema_version: CANVASKIT_DOCUMENT_PREFLIGHT_SCHEMA_VERSION,
            mode: self.mode,
            profile: self.profile,
            status,
            eligible,
            complete: self.complete,
            page_count: self.page_count,
            scanned_pages: self.scanned_pages,
            scanned_work_units: self.scanned_work_units,
            limits: self.limits,
            summary: self.summary,
            blockers: self.blockers,
            required_font_families: self.required_font_families.into_iter().collect(),
            capability_digest,
        }
    }
}

impl CanvasKitReplaySummary {
    fn merge(&mut self, other: &Self) {
        self.total_items = self.total_items.saturating_add(other.total_items);
        self.direct_items = self.direct_items.saturating_add(other.direct_items);
        self.direct_required_items = self
            .direct_required_items
            .saturating_add(other.direct_required_items);
        self.compat_overlay_items = self
            .compat_overlay_items
            .saturating_add(other.compat_overlay_items);
        self.text_fallback_items = self
            .text_fallback_items
            .saturating_add(other.text_fallback_items);
        self.unsupported_items = self
            .unsupported_items
            .saturating_add(other.unsupported_items);
        self.hidden_overlay_violations = self
            .hidden_overlay_violations
            .saturating_add(other.hidden_overlay_violations);
    }
}

fn blocker_code_for_item(
    item: &CanvasKitReplayItem,
) -> Option<CanvasKitDocumentPreflightBlockerCode> {
    if matches!(item.reason, CanvasKitReplayReason::HiddenOverlayForbidden) {
        return Some(CanvasKitDocumentPreflightBlockerCode::HiddenCanvas2dOverlayRequired);
    }
    match item.status {
        CanvasKitReplayStatus::CompatOverlay => {
            Some(CanvasKitDocumentPreflightBlockerCode::CompatOverlay)
        }
        CanvasKitReplayStatus::TextFallback => {
            Some(CanvasKitDocumentPreflightBlockerCode::TextFallback)
        }
        CanvasKitReplayStatus::Unsupported => {
            Some(CanvasKitDocumentPreflightBlockerCode::Unsupported)
        }
        CanvasKitReplayStatus::Direct | CanvasKitReplayStatus::DirectRequired => None,
    }
}

fn bounded_blocker_detail(mut detail: String) -> String {
    if detail.len() <= CANVASKIT_DOCUMENT_PREFLIGHT_MAX_DETAIL_BYTES {
        return detail;
    }
    let mut truncate_at = CANVASKIT_DOCUMENT_PREFLIGHT_MAX_DETAIL_BYTES.saturating_sub(3);
    while !detail.is_char_boundary(truncate_at) {
        truncate_at = truncate_at.saturating_sub(1);
    }
    detail.truncate(truncate_at);
    detail.push_str("...");
    detail
}

fn render_profile_name(profile: RenderProfile) -> &'static str {
    match profile {
        RenderProfile::FastPreview => "fastPreview",
        RenderProfile::Screen => "screen",
        RenderProfile::Print => "print",
        RenderProfile::HighQuality => "highQuality",
    }
}

struct CanvasKitCapabilityDigest(blake3::Hasher);

impl CanvasKitCapabilityDigest {
    fn new(
        mode: CanvasKitReplayMode,
        profile: &str,
        page_count: u32,
        limits: CanvasKitDocumentPreflightLimits,
    ) -> Self {
        let mut digest = Self(blake3::Hasher::new());
        digest.0.update(b"rhwp.canvaskit.document-preflight.v1\0");
        digest.record_str(mode.as_str());
        digest.record_str(profile);
        digest.record_u32(page_count);
        digest.record_u32(limits.max_pages);
        digest.record_u32(limits.max_work_units);
        digest.record_u32(limits.max_blockers);
        digest.record_u32(limits.max_required_font_families);
        digest
    }

    fn record_page(&mut self, page_index: u32, work_units: u32) {
        self.0.update(b"page\0");
        self.record_u32(page_index);
        self.record_u32(work_units);
    }

    fn record_item(&mut self, page_index: u32, item: &CanvasKitReplayItem) {
        self.0.update(b"item\0");
        self.record_u32(page_index);
        self.record_str(item.op_type);
        self.record_str(item.feature.as_str());
        self.record_str(item.status.as_str());
        self.record_str(item.reason.as_str());
        self.record_optional_str(item.detail.as_deref());
    }

    fn record_required_font_family(&mut self, page_index: u32, font_family: &str) {
        self.0.update(b"required-font-family\0");
        self.record_u32(page_index);
        self.record_str(font_family);
    }

    fn record_incomplete(
        &mut self,
        page_index: u32,
        code: CanvasKitDocumentPreflightBlockerCode,
        detail: Option<&str>,
    ) {
        self.0.update(b"incomplete\0");
        self.record_u32(page_index);
        self.record_str(code.as_str());
        self.record_optional_str(detail);
    }

    fn finish(
        mut self,
        status: CanvasKitDocumentPreflightStatus,
        complete: bool,
        scanned_pages: u32,
        scanned_work_units: u32,
        summary: &CanvasKitReplaySummary,
    ) -> String {
        self.0.update(b"result\0");
        self.record_str(match status {
            CanvasKitDocumentPreflightStatus::Eligible => "eligible",
            CanvasKitDocumentPreflightStatus::Ineligible => "ineligible",
            CanvasKitDocumentPreflightStatus::Incomplete => "incomplete",
        });
        self.0.update(&[u8::from(complete)]);
        self.record_u32(scanned_pages);
        self.record_u32(scanned_work_units);
        self.record_u32(summary.total_items);
        self.record_u32(summary.direct_items);
        self.record_u32(summary.direct_required_items);
        self.record_u32(summary.compat_overlay_items);
        self.record_u32(summary.text_fallback_items);
        self.record_u32(summary.unsupported_items);
        self.record_u32(summary.hidden_overlay_violations);
        format!("blake3:{}", self.0.finalize().to_hex())
    }

    fn record_u32(&mut self, value: u32) {
        self.0.update(&value.to_le_bytes());
    }

    fn record_str(&mut self, value: &str) {
        self.0.update(&(value.len() as u64).to_le_bytes());
        self.0.update(value.as_bytes());
    }

    fn record_optional_str(&mut self, value: Option<&str>) {
        self.0.update(&[u8::from(value.is_some())]);
        if let Some(value) = value {
            self.record_str(value);
        }
    }
}

pub fn analyze_canvaskit_replay_plan(
    tree: &PageLayerTree,
    mode: CanvasKitReplayMode,
) -> CanvasKitReplayPlan {
    let variant_reports = analyze_text_variant_selection(
        tree,
        TextVariantSelectionOptions {
            backend: VariantSelectionBackend::CanvasKitBrowser,
            prefer_strict_outline: true,
            allow_colrv1_stage1_color_graph: true,
            allow_bitmap_glyph: true,
            allow_svg_glyph: true,
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
    let mut builder = CanvasKitReplayPlanBuilder::new(mode, tree.profile, selected_variants);
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
    profile: RenderProfile,
    selected_variants: BTreeMap<String, SelectedTextVariant>,
    summary: CanvasKitReplaySummary,
    items: Vec<CanvasKitReplayItem>,
    required_font_families: BTreeSet<String>,
    required_font_families_complete: bool,
}

impl CanvasKitReplayPlanBuilder {
    fn new(
        mode: CanvasKitReplayMode,
        profile: RenderProfile,
        selected_variants: BTreeMap<String, SelectedTextVariant>,
    ) -> Self {
        Self {
            mode,
            policy: mode.policy(),
            profile,
            selected_variants,
            summary: CanvasKitReplaySummary::default(),
            items: Vec::new(),
            required_font_families: BTreeSet::new(),
            required_font_families_complete: true,
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
            required_font_families: self.required_font_families.into_iter().collect(),
            required_font_families_complete: self.required_font_families_complete,
        }
    }

    fn visit_node(
        &mut self,
        root: &LayerNode,
        root_path: &str,
        inherited_layer: Option<RenderLayerInfo>,
    ) {
        let mut pending = vec![(root, root_path.to_string(), inherited_layer)];
        while let Some((node, path, inherited_layer)) = pending.pop() {
            let active_layer = node.layer.or(inherited_layer);
            match &node.kind {
                LayerNodeKind::Group {
                    children,
                    cache_hint,
                    ..
                } => {
                    if !matches!(cache_hint, CacheHint::None) {
                        self.push_cache_hint_item(&path, *cache_hint);
                    }
                    for (index, child) in children.iter().enumerate().rev() {
                        pending.push((child, format!("{path}/group/{index}"), active_layer));
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
                    pending.push((child, format!("{path}/clip/child"), active_layer));
                }
                LayerNodeKind::Leaf { ops } => {
                    self.collect_leaf_required_font_families(ops);
                    for (index, op) in ops.iter().enumerate() {
                        self.push(self.item_for_op(
                            op,
                            format!("{path}/leaf/{index}"),
                            active_layer,
                        ));
                    }
                }
            }
        }
    }

    fn collect_leaf_required_font_families(&mut self, ops: &[PaintOp]) {
        let variant_groups = ops
            .iter()
            .filter_map(|op| match op {
                PaintOp::GlyphRun { run, .. } => Some(run.variant.equivalence_group.as_str()),
                PaintOp::GlyphOutline { outline, .. } => {
                    Some(outline.variant.equivalence_group.as_str())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let text_run_selected = variant_groups.is_empty()
            || variant_groups.iter().any(|group| {
                self.selected_variants.get(*group).is_none_or(|selected| {
                    selected.fallback_required
                        || matches!(selected.variant_kind, TextVariantKind::TextRun)
                })
            });

        for op in ops {
            match op {
                PaintOp::TextRun { run, .. } if text_run_selected => {
                    self.record_required_font_family(&run.style.font_family);
                    let display_text = expand_pua_display_text(&run.text);
                    if crate::renderer::contains_old_hangul_jamo(&display_text) {
                        self.record_required_font_family(OLD_HANGUL_FONT_FAMILY);
                    }
                }
                PaintOp::FootnoteMarker { marker, .. } => {
                    self.record_required_font_family(&marker.font_family);
                }
                _ => {}
            }
        }
    }

    fn record_required_font_family(&mut self, font_family: &str) {
        let font_family = font_family.trim();
        if font_family.is_empty() || self.required_font_families.contains(font_family) {
            return;
        }
        if font_family.len() > CANVASKIT_DOCUMENT_PREFLIGHT_MAX_FONT_FAMILY_BYTES
            || self.required_font_families.len()
                >= CANVASKIT_DOCUMENT_PREFLIGHT_MAX_REQUIRED_FONT_FAMILIES as usize
        {
            self.required_font_families_complete = false;
            return;
        }
        self.required_font_families.insert(font_family.to_string());
    }

    fn push_cache_hint_item(&mut self, path: &str, cache_hint: CacheHint) {
        self.push(CanvasKitReplayItem {
            path: format!("{path}/cacheHint"),
            op_type: "cacheHint",
            replay_plane: None,
            feature: CanvasKitReplayFeature::CacheHint,
            status: CanvasKitReplayStatus::Direct,
            reason: CanvasKitReplayReason::DirectReplaySupported,
            compat_overlay_allowed: false,
            detail: Some(format!("ignored:{cache_hint:?}")),
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
            PaintOp::Line { line, .. } => {
                self.vector_shape_item(path, "line", line_transition_detail(line))
            }
            PaintOp::Rectangle { rect, .. } => {
                self.vector_shape_item(path, "rectangle", rectangle_transition_detail(rect))
            }
            PaintOp::Ellipse { ellipse, .. } => {
                self.vector_shape_item(path, "ellipse", ellipse_transition_detail(ellipse))
            }
            PaintOp::Path { path: shape, .. } => {
                self.vector_shape_item(path, "path", path_transition_detail(shape))
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
            PaintOp::Equation { equation, .. }
                if canvaskit_equation_layout_is_supported(&equation.layout_box) =>
            {
                let mut item = direct_item(path, "equation", CanvasKitReplayFeature::Equation);
                item.detail = Some("boundedSemanticLayout".to_string());
                item
            }
            PaintOp::Equation { .. } => {
                let mut item = self.transition_overlay_item(
                    path,
                    "equation",
                    CanvasKitReplayFeature::Equation,
                );
                item.detail = Some("unsupportedSemanticLayout".to_string());
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
            PaintOp::Placeholder { placeholder, .. } => {
                let mut item =
                    direct_item(path, "placeholder", CanvasKitReplayFeature::Placeholder);
                item.detail = Some(match placeholder.kind {
                    crate::renderer::render_tree::PlaceholderKind::Ole => {
                        "basicStaticReplay".to_string()
                    }
                    crate::renderer::render_tree::PlaceholderKind::MissingPicture
                        if matches!(
                            self.profile,
                            RenderProfile::Print | RenderProfile::HighQuality
                        ) =>
                    {
                        "missingPictureSuppressedPrintEquivalent".to_string()
                    }
                    crate::renderer::render_tree::PlaceholderKind::MissingPicture => {
                        "missingPictureEditorVisual".to_string()
                    }
                });
                item
            }
            PaintOp::TextRun { run, .. } => self.text_run_item(path, run),
            PaintOp::CharOverlap { bbox, run } => {
                let detail = if bounded_text_char_count(&run.text)
                    > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN
                {
                    Some("visualItemLimitExceeded")
                } else if !text_visual_geometry_is_valid(bbox, run) {
                    Some("invalidGeometry")
                } else if run.is_vertical {
                    Some("verticalText")
                } else if run.rotation.abs() > f64::EPSILON {
                    Some("rotatedText")
                } else if run
                    .char_overlap
                    .as_ref()
                    .is_none_or(|overlap| overlap.border_type > 4)
                {
                    Some("invalidCharOverlap")
                } else {
                    None
                };
                if let Some(detail) = detail {
                    let mut item = self.transition_overlay_item(
                        path,
                        "charOverlap",
                        CanvasKitReplayFeature::TextSpecialVisual,
                    );
                    item.detail = Some(detail.to_string());
                    item
                } else {
                    direct_item(
                        path,
                        "charOverlap",
                        CanvasKitReplayFeature::TextSpecialVisual,
                    )
                }
            }
            PaintOp::TextControlMark { bbox, run } => {
                let detail = if bounded_text_char_count(&run.text)
                    > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN
                    || positioned_control_mark_count(run)
                        > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN
                {
                    Some("visualItemLimitExceeded")
                } else if !text_visual_geometry_is_valid(bbox, run) {
                    Some("invalidGeometry")
                } else if run.is_vertical {
                    Some("verticalText")
                } else if run.rotation.abs() > f64::EPSILON {
                    Some("rotatedText")
                } else {
                    None
                };
                if let Some(detail) = detail {
                    let mut item = self.transition_overlay_item(
                        path,
                        "textControlMark",
                        CanvasKitReplayFeature::TextSpecialVisual,
                    );
                    item.detail = Some(detail.to_string());
                    item
                } else {
                    direct_item(
                        path,
                        "textControlMark",
                        CanvasKitReplayFeature::TextSpecialVisual,
                    )
                }
            }
            PaintOp::TabLeader { bbox, run } => {
                let detail = if bounded_text_char_count(&run.text)
                    > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN
                    || run.style.tab_leaders.len()
                        > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN
                {
                    Some("visualItemLimitExceeded")
                } else if !text_visual_geometry_is_valid(bbox, run) {
                    Some("invalidGeometry")
                } else if run.is_vertical {
                    Some("verticalText")
                } else if run.rotation.abs() > f64::EPSILON {
                    Some("rotatedText")
                } else if run.style.tab_leaders.iter().any(|leader| {
                    !leader.start_x.is_finite()
                        || !leader.end_x.is_finite()
                        || leader.end_x <= leader.start_x
                        || leader.fill_type > 11
                }) {
                    Some("invalidTabLeader")
                } else {
                    None
                };
                if let Some(detail) = detail {
                    let mut item = self.transition_overlay_item(
                        path,
                        "tabLeader",
                        CanvasKitReplayFeature::TextSpecialVisual,
                    );
                    item.detail = Some(detail.to_string());
                    item
                } else {
                    direct_item(path, "tabLeader", CanvasKitReplayFeature::TextSpecialVisual)
                }
            }
            PaintOp::TextDecoration { bbox, run, kind } => {
                let too_many_items = text_decoration_position_count(run)
                    > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN;
                let detail = if too_many_items {
                    Some("visualItemLimitExceeded")
                } else if !text_visual_geometry_is_valid(bbox, run) {
                    Some("invalidGeometry")
                } else if run.is_vertical {
                    Some("verticalText")
                } else if run.rotation.abs() > f64::EPSILON {
                    Some("rotatedText")
                } else if match kind {
                    TextDecorationKind::Underline => run.style.underline_shape > 12,
                    TextDecorationKind::Strikethrough => run.style.strike_shape > 12,
                    TextDecorationKind::EmphasisDot => run.style.emphasis_dot > 6,
                } {
                    Some("unsupportedTextDecoration")
                } else {
                    None
                };
                if let Some(detail) = detail {
                    let mut item = self.transition_overlay_item(
                        path,
                        paint_op_type(op),
                        CanvasKitReplayFeature::TextSpecialVisual,
                    );
                    item.detail = Some(detail.to_string());
                    item
                } else {
                    direct_item(
                        path,
                        paint_op_type(op),
                        CanvasKitReplayFeature::TextSpecialVisual,
                    )
                }
            }
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

    fn vector_shape_item(
        &self,
        path: String,
        op_type: &'static str,
        transition_detail: Option<&'static str>,
    ) -> CanvasKitReplayItem {
        if let Some(detail) = transition_detail {
            let mut item =
                self.transition_overlay_item(path, op_type, CanvasKitReplayFeature::VectorShape);
            item.detail = Some(detail.to_string());
            item
        } else {
            direct_item(path, op_type, CanvasKitReplayFeature::VectorShape)
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
        if selected.is_some_and(|selected| !selected.fallback_required) {
            return CanvasKitReplayItem {
                path,
                op_type,
                replay_plane: None,
                feature: CanvasKitReplayFeature::TextVariant,
                status: CanvasKitReplayStatus::Direct,
                reason: CanvasKitReplayReason::DirectReplaySupported,
                compat_overlay_allowed: false,
                detail: Some(format!("unselectedVariant={variant_id}")),
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

fn canvaskit_equation_layout_is_supported(layout: &LayoutBox) -> bool {
    const MAX_DEPTH: usize = 64;
    const MAX_NODES: usize = 4096;
    const MAX_TEXT_UTF16_UNITS: usize = 4096;

    fn visit(
        layout: &LayoutBox,
        depth: usize,
        remaining_nodes: &mut usize,
        max_text_utf16_units: usize,
    ) -> bool {
        if depth > MAX_DEPTH
            || *remaining_nodes == 0
            || ![
                layout.x,
                layout.y,
                layout.width,
                layout.height,
                layout.baseline,
            ]
            .into_iter()
            .all(f64::is_finite)
            || layout.width < 0.0
            || layout.height < 0.0
        {
            return false;
        }
        *remaining_nodes -= 1;

        let text_supported =
            |text: &str| !text.is_empty() && text.encode_utf16().count() <= max_text_utf16_units;
        let child = |child: &LayoutBox, remaining_nodes: &mut usize| {
            visit(child, depth + 1, remaining_nodes, max_text_utf16_units)
        };
        match &layout.kind {
            LayoutKind::Row(children) => children.iter().all(|item| child(item, remaining_nodes)),
            LayoutKind::Text(text)
            | LayoutKind::Number(text)
            | LayoutKind::Symbol(text)
            | LayoutKind::MathSymbol(text)
            | LayoutKind::Function(text) => text_supported(text),
            LayoutKind::Fraction { numer, denom } => {
                child(numer, remaining_nodes) && child(denom, remaining_nodes)
            }
            LayoutKind::Atop { top, bottom } => {
                child(top, remaining_nodes) && child(bottom, remaining_nodes)
            }
            LayoutKind::Sqrt { index, body } => {
                index
                    .as_deref()
                    .is_none_or(|item| child(item, remaining_nodes))
                    && child(body, remaining_nodes)
            }
            LayoutKind::Superscript { base, sup } => {
                child(base, remaining_nodes) && child(sup, remaining_nodes)
            }
            LayoutKind::Subscript { base, sub } => {
                child(base, remaining_nodes) && child(sub, remaining_nodes)
            }
            LayoutKind::SubSup { base, sub, sup } => {
                child(base, remaining_nodes)
                    && child(sub, remaining_nodes)
                    && child(sup, remaining_nodes)
            }
            LayoutKind::BigOp { symbol, sub, sup } => {
                text_supported(symbol)
                    && sub
                        .as_deref()
                        .is_none_or(|item| child(item, remaining_nodes))
                    && sup
                        .as_deref()
                        .is_none_or(|item| child(item, remaining_nodes))
            }
            LayoutKind::Limit { sub, .. } => sub
                .as_deref()
                .is_none_or(|item| child(item, remaining_nodes)),
            LayoutKind::Matrix { cells, .. } => cells
                .iter()
                .flatten()
                .all(|item| child(item, remaining_nodes)),
            LayoutKind::Rel { arrow, over, under } => {
                child(over, remaining_nodes)
                    && child(arrow, remaining_nodes)
                    && under
                        .as_deref()
                        .is_none_or(|item| child(item, remaining_nodes))
            }
            LayoutKind::EqAlign { rows } => rows
                .iter()
                .all(|(left, right)| child(left, remaining_nodes) && child(right, remaining_nodes)),
            LayoutKind::Paren { left, right, body } => {
                (left.is_empty() || text_supported(left))
                    && (right.is_empty() || text_supported(right))
                    && child(body, remaining_nodes)
            }
            LayoutKind::Decoration { kind, body } => {
                matches!(
                    kind,
                    DecoKind::Hat
                        | DecoKind::Dot
                        | DecoKind::DDot
                        | DecoKind::Bar
                        | DecoKind::Vec
                        | DecoKind::Dyad
                        | DecoKind::Under
                        | DecoKind::Underline
                        | DecoKind::Overline
                        | DecoKind::StrikeThrough
                ) && child(body, remaining_nodes)
            }
            LayoutKind::FontStyle { style, body } => {
                matches!(
                    style,
                    FontStyleKind::Roman | FontStyleKind::Italic | FontStyleKind::Bold
                ) && child(body, remaining_nodes)
            }
            LayoutKind::Space(width) => width.is_finite(),
            LayoutKind::Newline | LayoutKind::Empty => true,
        }
    }

    let mut remaining_nodes = MAX_NODES;
    visit(layout, 0, &mut remaining_nodes, MAX_TEXT_UTF16_UNITS)
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

fn shape_style_transition_detail(style: &ShapeStyle) -> Option<&'static str> {
    if style.pattern.is_some() {
        return Some("patternFill");
    }
    if style.shadow.is_some() {
        return Some("shapeShadow");
    }
    if !matches!(style.stroke_dash, StrokeDash::Solid) {
        return Some("strokeDash");
    }
    None
}

fn line_transition_detail(line: &LineNode) -> Option<&'static str> {
    if line.transform.has_transform() {
        return Some("lineTransform");
    }
    if line.style.shadow.is_some() {
        return Some("lineShadow");
    }
    if !matches!(line.style.dash, StrokeDash::Solid) {
        return Some("strokeDash");
    }
    if !matches!(line.style.line_type, LineRenderType::Single) {
        return Some("compoundLine");
    }
    if !matches!(line.style.start_arrow, ArrowStyle::None)
        || !matches!(line.style.end_arrow, ArrowStyle::None)
    {
        return Some("lineArrow");
    }
    None
}

fn rectangle_transition_detail(rect: &RectangleNode) -> Option<&'static str> {
    if rect.gradient.is_some() {
        return Some("gradientFill");
    }
    if rect.transform.has_transform() {
        return Some("shapeTransform");
    }
    shape_style_transition_detail(&rect.style)
}

fn ellipse_transition_detail(ellipse: &EllipseNode) -> Option<&'static str> {
    if ellipse.gradient.is_some() {
        return Some("gradientFill");
    }
    if ellipse.transform.has_transform() {
        return Some("shapeTransform");
    }
    shape_style_transition_detail(&ellipse.style)
}

fn path_transition_detail(path: &PathNode) -> Option<&'static str> {
    if path.gradient.is_some() {
        return Some("gradientFill");
    }
    if let Some(detail) = shape_style_transition_detail(&path.style) {
        return Some(detail);
    }
    let line_style = path.line_style.as_ref()?;
    if line_style.shadow.is_some() {
        return Some("lineShadow");
    }
    if !matches!(line_style.dash, StrokeDash::Solid) {
        return Some("strokeDash");
    }
    if !matches!(line_style.line_type, LineRenderType::Single) {
        return Some("compoundLine");
    }
    if !matches!(line_style.start_arrow, ArrowStyle::None)
        || !matches!(line_style.end_arrow, ArrowStyle::None)
    {
        return Some("lineArrow");
    }
    None
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
    if run.style.shade_color & 0x00FF_FFFF != 0x00FF_FFFF {
        return Some("shadeTextEffect");
    }
    if (run.style.ratio - 1.0).abs() > f64::EPSILON {
        return Some("ratioTextEffect");
    }
    if run.style.superscript || run.style.subscript {
        let display_text = expand_pua_display_text(&run.text);
        if !display_text
            .bytes()
            .all(|byte| (0x20..=0x7e).contains(&byte))
        {
            return Some("scriptTextRequiresShaping");
        }
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
    if image_payload_bytes(image, resolved)
        .is_some_and(|bytes| !canvaskit_encoded_image_is_replayable(bytes))
    {
        detail.push("unsupportedEncodedImage".to_string());
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
    let payload_is_supported =
        image_payload_bytes(image, resolved).is_some_and(canvaskit_encoded_image_is_replayable);
    let effects_are_supported = resolved.is_some_and(|payload| payload.suppress_effects)
        || (matches!(image.effect, ImageEffect::RealPic)
            && image.brightness == 0
            && image.contrast == 0);
    has_replayable_payload && payload_is_supported && effects_are_supported
}

fn image_payload_bytes<'a>(
    image: &'a ImageNode,
    resolved: Option<&'a ResolvedImagePayload>,
) -> Option<&'a [u8]> {
    resolved
        .map(|payload| payload.data.as_slice())
        .or(image.data.as_deref())
}

fn canvaskit_encoded_image_is_replayable(bytes: &[u8]) -> bool {
    if bytes.is_empty()
        || bytes.len().div_ceil(3).saturating_mul(4) > CANVASKIT_MAX_ENCODED_IMAGE_BASE64_BYTES
    {
        return false;
    }
    canvaskit_encoded_image_header(bytes).is_some_and(|header| header.is_within_decode_limits())
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
    use crate::renderer::composer::CharOverlapInfo;
    use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
    use crate::renderer::render_tree::{
        BoundingBox, EquationNode, FieldMarkerType, FootnoteMarkerNode, FormObjectNode, ImageNode,
        PageBackgroundImage, PlaceholderNode, RawSvgNode, RectangleNode, RenderLayerInfo,
    };
    use crate::renderer::{GradientFillInfo, ShapeStyle, TextStyle};
    use image::ImageFormat;
    use std::io::Cursor;

    fn bbox() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 20.0, 20.0)
    }

    fn tree_with_ops(ops: Vec<PaintOp>) -> PageLayerTree {
        PageLayerTree::new(100.0, 100.0, LayerNode::leaf(bbox(), None, ops))
    }

    fn preflight_page(tree: PageLayerTree) -> CanvasKitPreflightPageBuild {
        CanvasKitPreflightPageBuild::Complete {
            tree: Box::new(tree),
            prelower_work_units: 0,
        }
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

    fn fixture_png() -> Vec<u8> {
        include_bytes!("../../assets/logo/logo-32.png").to_vec()
    }

    fn image_node(bin_data_id: u16) -> ImageNode {
        ImageNode::new(bin_data_id, Some(fixture_png()))
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
        let tree = tree_with_ops(vec![PaintOp::image(bbox(), image_node(1), None)]);

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
        let tree = tree_with_ops(vec![PaintOp::image(bbox(), image_node(1), None)]);

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
    fn cache_hints_are_non_visual_and_do_not_block_direct_replay() {
        let tree = PageLayerTree::new(
            100.0,
            100.0,
            LayerNode::group(
                bbox(),
                None,
                vec![LayerNode::leaf(
                    bbox(),
                    None,
                    vec![PaintOp::text_run(bbox(), text_run("A"))],
                )],
                CacheHint::StaticSubtree,
                GroupKind::Generic,
            ),
        );

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_items, 2);
        assert_eq!(plan.summary.hidden_overlay_violations, 0);
        assert_eq!(plan.items[0].op_type, "cacheHint");
        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("ignored:StaticSubtree")
        );
    }

    #[test]
    fn unsupported_vector_styles_do_not_pass_browser_preflight() {
        let mut rect_style = ShapeStyle::default();
        rect_style.stroke_color = Some(0x0000_0000);
        rect_style.stroke_width = 1.0;
        rect_style.stroke_dash = StrokeDash::Dash;
        let rect = RectangleNode::new(0.0, rect_style, None);

        let mut line_style = crate::renderer::LineStyle::default();
        line_style.end_arrow = ArrowStyle::Arrow;
        let line = LineNode::new(0.0, 0.0, 20.0, 20.0, line_style);
        let tree = tree_with_ops(vec![
            PaintOp::rectangle(bbox(), rect),
            PaintOp::line(bbox(), line),
        ]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_required_items, 2);
        assert_eq!(plan.summary.hidden_overlay_violations, 2);
        assert_eq!(plan.items[0].detail.as_deref(), Some("strokeDash"));
        assert_eq!(plan.items[1].detail.as_deref(), Some("lineArrow"));
    }

    #[test]
    fn unselected_text_sidecars_do_not_make_a_direct_outline_ineligible() {
        let mut selected = BTreeMap::new();
        selected.insert(
            "text-0".to_string(),
            SelectedTextVariant {
                variant_id: "outline".to_string(),
                variant_kind: TextVariantKind::GlyphOutline,
                fallback_required: false,
            },
        );
        let builder = CanvasKitReplayPlanBuilder::new(
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            selected,
        );

        let outline = builder.text_variant_item(
            "outline".to_string(),
            "glyphOutline",
            "text-0",
            "outline",
            TextVariantKind::GlyphOutline,
        );
        let glyph_run = builder.text_variant_item(
            "run".to_string(),
            "glyphRun",
            "text-0",
            "run",
            TextVariantKind::GlyphRun,
        );

        assert_eq!(outline.status, CanvasKitReplayStatus::DirectRequired);
        assert_eq!(glyph_run.status, CanvasKitReplayStatus::Direct);
        assert_eq!(glyph_run.detail.as_deref(), Some("unselectedVariant=run"));
    }

    #[test]
    fn image_replay_plan_reports_direct_geometry_payload() {
        let mut image = image_node(1);
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
        let mut image = image_node(1);
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
        let mut image = image_node(1);
        image.effect = ImageEffect::GrayScale;
        image.brightness = 70;
        image.contrast = -50;

        let tree = tree_with_ops(vec![PaintOp::image(
            bbox(),
            image,
            Some(ResolvedImagePayload {
                data: fixture_png(),
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
        let mut behind = image_node(1);
        behind.text_wrap = Some(TextWrap::BehindText);
        let mut front = image_node(2);
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
        let mut image = image_node(1);
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
    fn image_preflight_rejects_payloads_the_browser_decoder_will_reject() {
        let mut tiff = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(1, 1)
            .write_to(&mut tiff, ImageFormat::Tiff)
            .expect("encode TIFF fixture");

        for bytes in [vec![1, 2, 3], tiff.into_inner()] {
            let tree = tree_with_ops(vec![PaintOp::image(
                bbox(),
                ImageNode::new(1, Some(bytes)),
                None,
            )]);

            let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

            assert_eq!(plan.items[0].status, CanvasKitReplayStatus::DirectRequired);
            assert_eq!(
                plan.items[0].detail.as_deref(),
                Some("unsupportedEncodedImage")
            );
        }
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
    fn equation_layout_is_direct_while_raw_svg_remains_a_replay_gap() {
        let tree = tree_with_ops(vec![
            PaintOp::equation(bbox(), equation_node()),
            PaintOp::raw_svg(
                bbox(),
                RawSvgNode::new("<g><path d=\"M0 0H1\"/></g>".to_string()),
            ),
        ]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_items, 1);
        assert_eq!(plan.summary.direct_required_items, 1);
        assert_eq!(plan.items[0].op_type, "equation");
        assert_eq!(plan.items[0].feature, CanvasKitReplayFeature::Equation);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("boundedSemanticLayout")
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
    fn unsupported_equation_styles_are_not_reported_as_direct() {
        let mut equation = equation_node();
        equation.layout_box.kind = LayoutKind::FontStyle {
            style: FontStyleKind::Blackboard,
            body: Box::new(LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 12.0,
                baseline: 10.0,
                kind: LayoutKind::Text("x".to_string()),
            }),
        };
        let tree = tree_with_ops(vec![PaintOp::equation(bbox(), equation)]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.summary.direct_items, 0);
        assert_eq!(plan.summary.direct_required_items, 1);
        assert_eq!(plan.summary.hidden_overlay_violations, 1);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("unsupportedSemanticLayout")
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
        assert_eq!(default_plan.summary.direct_items, 4);
        assert_eq!(default_plan.summary.direct_required_items, 1);
        assert_eq!(
            default_plan.items[2].detail.as_deref(),
            Some("verticalText")
        );
        assert_eq!(default_plan.items[3].status, CanvasKitReplayStatus::Direct);
        assert_eq!(default_plan.items[4].status, CanvasKitReplayStatus::Direct);

        let compat_plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Compat);
        assert_eq!(compat_plan.summary.direct_items, 4);
        assert_eq!(compat_plan.summary.direct_required_items, 1);
        assert_eq!(compat_plan.summary.compat_overlay_items, 0);
        assert_eq!(compat_plan.items[2].detail.as_deref(), Some("verticalText"));
        assert_eq!(compat_plan.items[3].status, CanvasKitReplayStatus::Direct);
        assert_eq!(compat_plan.items[4].status, CanvasKitReplayStatus::Direct);
    }

    #[test]
    fn nonvisual_control_metadata_and_opaque_white_shade_keep_text_direct() {
        let mut run = text_run("A");
        run.is_para_end = true;
        run.is_line_break_end = true;
        run.field_marker = FieldMarkerType::FieldBegin;
        run.style.shade_color = 0xFFFF_FFFF;
        let tree = tree_with_ops(vec![PaintOp::text_run(bbox(), run.clone())]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);

        assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(plan.summary.hidden_overlay_violations, 0);

        let mark_tree = tree_with_ops(vec![PaintOp::text_control_mark(bbox(), run)]);
        let mark_plan = analyze_canvaskit_replay_plan(&mark_tree, CanvasKitReplayMode::Default);
        assert_eq!(mark_plan.items[0].op_type, "textControlMark");
        assert_eq!(mark_plan.items[0].status, CanvasKitReplayStatus::Direct);
        assert_eq!(mark_plan.summary.hidden_overlay_violations, 0);
    }

    #[test]
    fn external_text_special_visuals_are_direct_replay_items() {
        let mut superscript = text_run("AB");
        superscript.style.superscript = true;
        superscript.char_overlap = Some(CharOverlapInfo {
            border_type: 1,
            inner_char_size: 100,
        });
        superscript
            .style
            .tab_leaders
            .push(crate::renderer::TabLeaderInfo {
                start_x: 4.0,
                end_x: 20.0,
                fill_type: 3,
            });
        superscript.style.underline = crate::model::style::UnderlineType::Bottom;
        let tree = tree_with_ops(vec![
            PaintOp::text_run(bbox(), superscript.clone()),
            PaintOp::char_overlap(bbox(), superscript.clone()),
            PaintOp::tab_leader(bbox(), superscript.clone()),
            PaintOp::text_decoration(bbox(), superscript, TextDecorationKind::Underline),
        ]);

        for mode in [CanvasKitReplayMode::Default, CanvasKitReplayMode::Compat] {
            let plan = analyze_canvaskit_replay_plan(&tree, mode);
            assert_eq!(plan.summary.direct_items, 4);
            assert_eq!(plan.summary.direct_required_items, 0);
            assert!(plan
                .items
                .iter()
                .all(|item| item.status == CanvasKitReplayStatus::Direct));
        }
    }

    #[test]
    fn malformed_vertical_and_rotated_text_special_visuals_stay_fail_closed() {
        let mut invalid_overlap = text_run("A");
        invalid_overlap.char_overlap = Some(CharOverlapInfo {
            border_type: 9,
            inner_char_size: 100,
        });
        let mut invalid_leader = text_run("A");
        invalid_leader
            .style
            .tab_leaders
            .push(crate::renderer::TabLeaderInfo {
                start_x: 20.0,
                end_x: 4.0,
                fill_type: 12,
            });
        let mut vertical_mark = text_run("A");
        vertical_mark.is_vertical = true;
        let mut rotated = text_run("A");
        rotated.rotation = 15.0;
        rotated.char_overlap = Some(CharOverlapInfo {
            border_type: 1,
            inner_char_size: 100,
        });
        rotated
            .style
            .tab_leaders
            .push(crate::renderer::TabLeaderInfo {
                start_x: 1.0,
                end_x: 8.0,
                fill_type: 1,
            });
        let tree = tree_with_ops(vec![
            PaintOp::char_overlap(bbox(), invalid_overlap),
            PaintOp::tab_leader(bbox(), invalid_leader),
            PaintOp::text_control_mark(bbox(), vertical_mark),
            PaintOp::char_overlap(bbox(), rotated.clone()),
            PaintOp::text_control_mark(bbox(), rotated.clone()),
            PaintOp::tab_leader(bbox(), rotated.clone()),
            PaintOp::text_decoration(bbox(), rotated, TextDecorationKind::Underline),
        ]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(plan.summary.direct_required_items, 7);
        assert_eq!(plan.items[0].detail.as_deref(), Some("invalidCharOverlap"));
        assert_eq!(plan.items[1].detail.as_deref(), Some("invalidTabLeader"));
        assert_eq!(plan.items[2].detail.as_deref(), Some("verticalText"));
        for item in &plan.items[3..] {
            assert_eq!(item.detail.as_deref(), Some("rotatedText"));
        }
    }

    #[test]
    fn text_special_visual_work_and_item_counts_are_bounded() {
        let marks = text_run(&" ".repeat(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1));
        let tree = tree_with_ops(vec![PaintOp::text_control_mark(bbox(), marks)]);

        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(plan.summary.direct_required_items, 1);
        assert_eq!(
            plan.items[0].detail.as_deref(),
            Some("visualItemLimitExceeded")
        );
        assert_eq!(
            count_layer_tree_work_units(&tree, 100),
            CanvasKitBoundedWorkCount::Exceeded
        );

        let decoration =
            text_run(&"A".repeat(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1));
        let decoration_tree = tree_with_ops(vec![PaintOp::text_decoration(
            bbox(),
            decoration,
            TextDecorationKind::Underline,
        )]);
        let decoration_plan =
            analyze_canvaskit_replay_plan(&decoration_tree, CanvasKitReplayMode::Default);
        assert_eq!(decoration_plan.summary.direct_required_items, 1);
        assert_eq!(
            decoration_plan.items[0].detail.as_deref(),
            Some("visualItemLimitExceeded")
        );
        assert_eq!(
            count_layer_tree_work_units(&decoration_tree, 100),
            CanvasKitBoundedWorkCount::Exceeded
        );

        let mut malformed = text_run("A");
        malformed.baseline = f64::NAN;
        let malformed_tree = tree_with_ops(vec![PaintOp::text_control_mark(bbox(), malformed)]);
        let malformed_plan =
            analyze_canvaskit_replay_plan(&malformed_tree, CanvasKitReplayMode::Default);
        assert_eq!(
            malformed_plan.items[0].detail.as_deref(),
            Some("invalidGeometry")
        );

        let text_tree = tree_with_ops(vec![PaintOp::text_run(bbox(), text_run(&"A".repeat(101)))]);
        assert_eq!(
            count_layer_tree_work_units(&text_tree, 100),
            CanvasKitBoundedWorkCount::Exceeded
        );
    }

    #[test]
    fn shaped_script_text_stays_policy_visible() {
        for text in ["가", "e\u{0301}", "\u{F012B}"] {
            let mut superscript = text_run(text);
            superscript.style.superscript = true;
            let tree = tree_with_ops(vec![PaintOp::text_run(bbox(), superscript)]);

            for mode in [CanvasKitReplayMode::Default, CanvasKitReplayMode::Compat] {
                let plan = analyze_canvaskit_replay_plan(&tree, mode);
                assert_eq!(plan.summary.direct_required_items, 1, "text={text:?}");
                assert_eq!(
                    plan.items[0].status,
                    CanvasKitReplayStatus::DirectRequired,
                    "text={text:?}"
                );
                assert_eq!(
                    plan.items[0].detail.as_deref(),
                    Some("scriptTextRequiresShaping"),
                    "text={text:?}"
                );
            }
        }
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
        let placeholder = PlaceholderNode::new(0x00FF_FFFF, 0x0000_0000, "OLE".to_string());
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
    fn missing_picture_policy_tracks_editor_and_print_equivalent_profiles() {
        let op = PaintOp::placeholder(
            bbox(),
            PlaceholderNode::missing_picture(None, None, None, None),
        );
        let screen = analyze_canvaskit_replay_plan(
            &tree_with_ops(vec![op.clone()]),
            CanvasKitReplayMode::Default,
        );
        let print_tree = PageLayerTree::with_profile(
            100.0,
            100.0,
            LayerNode::leaf(bbox(), None, vec![op]),
            RenderProfile::Print,
        );
        let print = analyze_canvaskit_replay_plan(&print_tree, CanvasKitReplayMode::Default);

        assert_eq!(
            screen.items[0].detail.as_deref(),
            Some("missingPictureEditorVisual")
        );
        assert_eq!(
            print.items[0].detail.as_deref(),
            Some("missingPictureSuppressedPrintEquivalent")
        );
    }

    #[test]
    fn document_preflight_marks_direct_only_tree_eligible() {
        let tree = tree_with_ops(vec![PaintOp::text_run(bbox(), text_run("A"))]);
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 16,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(preflight.status, CanvasKitDocumentPreflightStatus::Eligible);
        assert!(preflight.eligible);
        assert!(preflight.complete);
        assert_eq!(preflight.scanned_pages, 1);
        assert_eq!(preflight.scanned_work_units, 3);
        assert_eq!(preflight.summary.direct_items, 1);
        assert!(preflight.blockers.is_empty());
        assert_eq!(preflight.required_font_families, ["Test"]);
        assert!(preflight.capability_digest.starts_with("blake3:"));
    }

    #[test]
    fn document_preflight_requires_old_hangul_shaping_font_for_pua_projection() {
        let tree = tree_with_ops(vec![PaintOp::text_run(bbox(), text_run("\u{F53A}"))]);
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 16,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(preflight.status, CanvasKitDocumentPreflightStatus::Eligible);
        assert_eq!(
            preflight.required_font_families,
            [OLD_HANGUL_FONT_FAMILY, "Test"]
        );
    }

    #[test]
    fn prelower_estimate_bounds_text_expansion_before_layer_allocation() {
        let mut tree = PageRenderTree::new(0, 100.0, 100.0);
        tree.root
            .children
            .push(crate::renderer::render_tree::RenderNode::new(
                1,
                RenderNodeType::TextRun(text_run("A")),
                bbox(),
            ));

        assert_eq!(
            estimate_canvaskit_page_lowering_work(&tree, 12),
            CanvasKitBoundedWorkCount::Complete(12)
        );
        assert_eq!(
            estimate_canvaskit_page_lowering_work(&tree, 11),
            CanvasKitBoundedWorkCount::Exceeded
        );
    }

    #[test]
    fn prelower_estimate_rejects_oversized_text_like_payloads() {
        let mut tree = PageRenderTree::new(0, 100.0, 100.0);
        tree.root
            .children
            .push(crate::renderer::render_tree::RenderNode::new(
                1,
                RenderNodeType::RawSvg(RawSvgNode::new(
                    "x".repeat(CANVASKIT_DOCUMENT_PREFLIGHT_MAX_TEXT_BYTES + 1),
                )),
                bbox(),
            ));

        assert_eq!(
            estimate_canvaskit_page_lowering_work(&tree, 50_000),
            CanvasKitBoundedWorkCount::Exceeded
        );
    }

    #[test]
    fn document_preflight_marks_raw_svg_transition_ineligible() {
        let tree = tree_with_ops(vec![PaintOp::raw_svg(
            bbox(),
            RawSvgNode::new("<path d=\"M0 0H1\"/>".to_string()),
        )]);
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 16,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Ineligible
        );
        assert!(!preflight.eligible);
        assert!(preflight.complete);
        assert_eq!(preflight.summary.hidden_overlay_violations, 1);
        assert_eq!(preflight.blockers.len(), 1);
        assert_eq!(
            preflight.blockers[0].code,
            CanvasKitDocumentPreflightBlockerCode::HiddenCanvas2dOverlayRequired
        );
        assert_eq!(preflight.blockers[0].op_type, Some("rawSvg"));
        let json = serde_json::to_value(&preflight).expect("serialize document preflight");
        assert_eq!(
            json["blockers"][0]["code"].as_str(),
            Some("hiddenCanvas2dOverlayRequired")
        );
    }

    #[test]
    fn document_preflight_stops_before_replay_analysis_at_work_limit() {
        let tree = tree_with_ops(vec![
            PaintOp::text_run(bbox(), text_run("A")),
            PaintOp::raw_svg(bbox(), RawSvgNode::new("<path d=\"M0 0H1\"/>".to_string())),
        ]);
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 2,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Incomplete
        );
        assert!(!preflight.eligible);
        assert!(!preflight.complete);
        assert_eq!(preflight.scanned_pages, 0);
        assert_eq!(preflight.scanned_work_units, 2);
        assert_eq!(preflight.summary.total_items, 0);
        assert_eq!(preflight.blockers.len(), 1);
        assert_eq!(
            preflight.blockers[0].code,
            CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded
        );
    }

    #[test]
    fn document_preflight_counts_large_leaf_payloads_against_the_work_limit() {
        let tree = tree_with_ops(vec![PaintOp::raw_svg(
            bbox(),
            RawSvgNode::new("x".repeat(12 * 1024)),
        )]);
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 3,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Incomplete
        );
        assert_eq!(
            preflight.blockers[0].code,
            CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded
        );
    }

    #[test]
    fn document_preflight_rejects_pathological_layer_depth_before_plan_building() {
        let mut root =
            LayerNode::leaf(bbox(), None, vec![PaintOp::text_run(bbox(), text_run("A"))]);
        for _ in 0..=CANVASKIT_DOCUMENT_PREFLIGHT_MAX_TREE_DEPTH {
            root = LayerNode::group(
                bbox(),
                None,
                vec![root],
                CacheHint::None,
                GroupKind::Generic,
            );
        }
        let tree = PageLayerTree::new(100.0, 100.0, root);
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 1_000,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Incomplete
        );
        assert_eq!(preflight.scanned_pages, 0);
    }

    #[test]
    fn document_preflight_converts_page_build_error_to_incomplete() {
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 16,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            |_, _| Err::<CanvasKitPreflightPageBuild, _>("layout failed"),
        );

        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Incomplete
        );
        assert_eq!(preflight.scanned_pages, 0);
        assert_eq!(preflight.scanned_work_units, 0);
        assert_eq!(
            preflight.blockers[0].code,
            CanvasKitDocumentPreflightBlockerCode::PageBuildFailed
        );
        assert_eq!(
            preflight.blockers[0].detail.as_deref(),
            Some("layout failed")
        );
    }

    #[test]
    fn document_preflight_stops_before_page_build_at_page_limit() {
        let tree = tree_with_ops(Vec::new());
        let mut build_calls = 0;
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            2,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 1,
                max_work_units: 16,
                max_blockers: 4,
                max_required_font_families: 8,
            },
            |_, _| {
                build_calls += 1;
                Ok::<_, &'static str>(preflight_page(tree.clone()))
            },
        );

        assert_eq!(build_calls, 0);
        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Incomplete
        );
        assert_eq!(preflight.scanned_pages, 0);
        assert_eq!(preflight.scanned_work_units, 0);
        assert_eq!(preflight.blockers[0].page_index, 1);
        assert_eq!(
            preflight.blockers[0].code,
            CanvasKitDocumentPreflightBlockerCode::PageLimitExceeded
        );
    }

    #[test]
    fn document_preflight_bounds_blockers_without_losing_summary() {
        let tree = tree_with_ops(
            (0..3)
                .map(|_| {
                    PaintOp::raw_svg(bbox(), RawSvgNode::new("<path d=\"M0 0H1\"/>".to_string()))
                })
                .collect(),
        );
        let preflight = analyze_canvaskit_document_preflight_with_limits(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Screen,
            CanvasKitDocumentPreflightLimits {
                max_pages: 4,
                max_work_units: 16,
                max_blockers: 2,
                max_required_font_families: 8,
            },
            move |_, _| Ok::<_, &'static str>(preflight_page(tree.clone())),
        );

        assert_eq!(
            preflight.status,
            CanvasKitDocumentPreflightStatus::Ineligible
        );
        assert_eq!(preflight.summary.hidden_overlay_violations, 3);
        assert_eq!(preflight.blockers.len(), 2);
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
