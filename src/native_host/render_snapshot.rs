//! Immutable page inputs that can outlive their native document session.
//!
//! This is an owned input contract, not a renderer, GPU context, or native handle.
//! The shared layer tree owns its image/SVG/embedded-font resources. Explicit
//! host font files are retained as immutable bytes, never reopened by replay.

use std::mem::size_of;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::fonts::NativeFontState;
use super::{NativeDocumentSession, NativeHostError};
use crate::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
use crate::renderer::render_tree::TextRunNode;

pub const MAX_SNAPSHOT_NODES: usize = 65_536;
pub const MAX_SNAPSHOT_OPERATIONS: usize = 262_144;
pub const MAX_SNAPSHOT_RESOURCES: usize = 8_192;
pub const MAX_SNAPSHOT_PAYLOAD_BYTES: usize = 128 * 1024 * 1024;
pub const MAX_SNAPSHOT_DEPTH: usize = 256;

/// Admission limits for one page. Callers may lower, but cannot raise, the
/// public hard ceilings. A zero limit rejects any input in that category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeRenderSnapshotLimits {
    pub max_nodes: usize,
    pub max_operations: usize,
    pub max_resources: usize,
    pub max_payload_bytes: usize,
    pub max_depth: usize,
}

pub(crate) fn profile_name(profile: RenderProfile) -> &'static str {
    match profile {
        RenderProfile::FastPreview => "fastPreview",
        RenderProfile::Screen => "screen",
        RenderProfile::Print => "print",
        RenderProfile::HighQuality => "highQuality",
    }
}

impl Default for NativeRenderSnapshotLimits {
    fn default() -> Self {
        Self {
            max_nodes: MAX_SNAPSHOT_NODES,
            max_operations: MAX_SNAPSHOT_OPERATIONS,
            max_resources: MAX_SNAPSHOT_RESOURCES,
            max_payload_bytes: MAX_SNAPSHOT_PAYLOAD_BYTES,
            max_depth: MAX_SNAPSHOT_DEPTH,
        }
    }
}

impl NativeRenderSnapshotLimits {
    pub(crate) fn validate(self) -> Result<(), NativeHostError> {
        if self.max_nodes > MAX_SNAPSHOT_NODES
            || self.max_operations > MAX_SNAPSHOT_OPERATIONS
            || self.max_resources > MAX_SNAPSHOT_RESOURCES
            || self.max_payload_bytes > MAX_SNAPSHOT_PAYLOAD_BYTES
            || self.max_depth > MAX_SNAPSHOT_DEPTH
        {
            return Err(NativeHostError::InvalidArgument(
                "render snapshot limits exceed the public ceilings".into(),
            ));
        }
        Ok(())
    }
}

/// Logical input accounting, not process memory or VRAM. Shared payloads are
/// conservatively charged at each retained reference. Allocator capacity,
/// auxiliary layout/style metadata, decoded resources and renderer caches are
/// outside this estimate and require separate host budgets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRenderSnapshotUsage {
    pub nodes: usize,
    pub operations: usize,
    pub resources: usize,
    pub explicit_font_bytes: usize,
    pub resource_payload_bytes: usize,
    pub text_bytes: usize,
    pub geometry_bytes: usize,
    pub structural_bytes: usize,
    pub estimated_bytes: usize,
}

/// A frozen, shared page tree plus the explicit font supply used by its session.
///
/// System fallback is intentionally not frozen or certified by this value.
/// Creating a renderer on another host may choose different system fonts for
/// characters not covered by the supplied programs. Document ID, view/device
/// generation and global retention quotas belong to the consuming host.
#[derive(Debug, Clone)]
pub struct NativeRenderSnapshot {
    tree: Arc<PageLayerTree>,
    revision: u64,
    font_generation: u64,
    page: u32,
    font_state: NativeFontState,
    font_data: Arc<[Vec<u8>]>,
    usage: NativeRenderSnapshotUsage,
}

// A renderer/context stays on its owner thread. Only these immutable inputs
// cross threads; this assertion fails if a future layer payload violates that.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NativeRenderSnapshot>();
};

impl NativeRenderSnapshot {
    pub fn tree(&self) -> &Arc<PageLayerTree> {
        &self.tree
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn font_generation(&self) -> u64 {
        self.font_generation
    }

    pub fn page(&self) -> u32 {
        self.page
    }

    pub fn profile(&self) -> RenderProfile {
        self.tree.profile
    }

    pub fn font_state(&self) -> &NativeFontState {
        &self.font_state
    }

    pub fn usage(&self) -> NativeRenderSnapshotUsage {
        self.usage
    }

    /// Construct on the render owner's thread. The existing byte-based loader
    /// borrows frozen inputs; no caller font path or RHWP_FONT_PATH is read.
    /// System FontMgr fallback remains available, as in NativeDocumentSession.
    #[cfg(feature = "native-skia")]
    pub fn create_renderer(
        &self,
    ) -> Result<crate::renderer::skia::SkiaLayerRenderer, NativeHostError> {
        Ok(crate::renderer::skia::SkiaLayerRenderer::new()
            .with_cache_limits(
                super::MAX_RENDER_CACHE_ENTRIES,
                super::MAX_RENDER_CACHE_BYTES,
            )
            .try_with_font_data(&self.font_data)?)
    }

    /// Number of retained explicit font files, also available without native-skia.
    pub fn explicit_font_count(&self) -> usize {
        self.font_data.len()
    }
}

impl NativeDocumentSession {
    pub fn render_snapshot(
        &self,
        page: u32,
        profile: RenderProfile,
    ) -> Result<NativeRenderSnapshot, NativeHostError> {
        self.render_snapshot_with_limits(page, profile, NativeRenderSnapshotLimits::default())
    }

    pub fn render_snapshot_with_limits(
        &self,
        page: u32,
        profile: RenderProfile,
        limits: NativeRenderSnapshotLimits,
    ) -> Result<NativeRenderSnapshot, NativeHostError> {
        limits.validate()?;
        self.check_page(page)?;
        let tree = self
            .core
            .build_page_layer_tree_shared_with_profile(page, profile)?;
        let usage = account_inputs(&tree, &self.font_data, limits)?;
        Ok(NativeRenderSnapshot {
            tree,
            revision: self.revision,
            font_generation: self.font_generation,
            page,
            font_state: self.fonts.clone(),
            font_data: Arc::clone(&self.font_data),
            usage,
        })
    }
}

struct Accounting {
    limits: NativeRenderSnapshotLimits,
    usage: NativeRenderSnapshotUsage,
}

fn limit(message: &str) -> NativeHostError {
    NativeHostError::LimitExceeded(message.into())
}

impl Accounting {
    fn pending(&self, current: usize, additional: usize) -> Result<(), NativeHostError> {
        self.usage
            .nodes
            .checked_add(current)
            .and_then(|value| value.checked_add(additional))
            .filter(|value| *value <= self.limits.max_nodes)
            .ok_or_else(|| limit("render snapshot pending layer count exceeds its limit"))?;
        Ok(())
    }

    fn add(&mut self, bytes: usize) -> Result<(), NativeHostError> {
        self.usage.estimated_bytes = self
            .usage
            .estimated_bytes
            .checked_add(bytes)
            .filter(|value| *value <= self.limits.max_payload_bytes)
            .ok_or_else(|| limit("render snapshot input payload exceeds its byte limit"))?;
        Ok(())
    }

    fn resource(&mut self, bytes: usize) -> Result<(), NativeHostError> {
        self.usage.resources = self
            .usage
            .resources
            .checked_add(1)
            .filter(|value| *value <= self.limits.max_resources)
            .ok_or_else(|| limit("render snapshot resource count exceeds its limit"))?;
        self.add(bytes)?;
        self.usage.resource_payload_bytes += bytes;
        Ok(())
    }

    fn text(&mut self, text: &str) -> Result<(), NativeHostError> {
        self.add(text.len())?;
        self.usage.text_bytes += text.len();
        Ok(())
    }

    fn array<T>(&mut self, values: &[T]) -> Result<(), NativeHostError> {
        let bytes = values
            .len()
            .checked_mul(size_of::<T>())
            .ok_or_else(|| limit("render snapshot array byte count overflow"))?;
        self.add(bytes)?;
        self.usage.geometry_bytes += bytes;
        Ok(())
    }

    fn text_run(&mut self, run: &TextRunNode) -> Result<(), NativeHostError> {
        self.text(&run.text)?;
        self.text(&run.style.font_family)?;
        if let Some(display) = &run.display_text {
            self.text(display)?;
        }
        if let Some(positions) = &run.layout_positions {
            self.array(positions)?;
        }
        self.array(&run.style.tab_stops)?;
        self.array(&run.style.tab_leaders)?;
        self.array(&run.style.inline_tabs)
    }

    fn operation(&mut self, operation: &PaintOp) -> Result<(), NativeHostError> {
        match operation {
            PaintOp::Image {
                image, resolved, ..
            } => {
                if let Some(bytes) = &image.data {
                    self.resource(bytes.len())?;
                }
                if let Some(resolved) = resolved {
                    self.resource(resolved.data.len())?;
                }
            }
            PaintOp::RawSvg { raw, .. } => {
                self.resource(raw.svg.len())?;
                if let Some(source) = &raw.emf_source {
                    self.resource(source.data.len())?;
                }
            }
            PaintOp::PageBackground { background, .. } => {
                if let Some(image) = &background.image {
                    self.resource(image.data.len())?;
                }
                if let Some(gradient) = &background.gradient {
                    self.array(&gradient.colors)?;
                    self.array(&gradient.positions)?;
                }
            }
            PaintOp::TextRun { run, .. }
            | PaintOp::CharOverlap { run, .. }
            | PaintOp::TextControlMark { run, .. }
            | PaintOp::TabLeader { run, .. }
            | PaintOp::TextDecoration { run, .. } => self.text_run(run)?,
            PaintOp::GlyphRun { run, .. } => {
                self.array(&run.glyph_ids)?;
                self.array(&run.positions)?;
                self.array(&run.clusters)?;
                if let Some(advances) = &run.advances {
                    self.array(advances)?;
                }
                if let Some(transforms) = &run.glyph_transforms {
                    self.array(transforms)?;
                }
            }
            PaintOp::GlyphOutline { outline, .. } => {
                self.array(&outline.paths)?;
                for path in &outline.paths {
                    self.array(&path.commands)?;
                }
                if let Some(color) = &outline.color_layers {
                    self.array(&color.layers)?;
                    for layer in &color.layers {
                        if let Some(commands) = &layer.commands {
                            self.array(commands)?;
                        }
                    }
                    if let Some(graph) = &color.paint_graph {
                        self.array(&graph.nodes)?;
                        for node in &graph.nodes {
                            if let Some(path) = &node.solid_path {
                                self.array(&path.commands)?;
                            }
                            if let Some(path) = &node.linear_gradient_path {
                                self.array(&path.commands)?;
                                self.array(&path.gradient.stops)?;
                            }
                            if let Some(path) = &node.radial_gradient_path {
                                self.array(&path.commands)?;
                                self.array(&path.gradient.stops)?;
                            }
                            if let Some(path) = &node.sweep_gradient_path {
                                self.array(&path.commands)?;
                                self.array(&path.gradient.stops)?;
                            }
                            if let Some(clip) = &node.clip {
                                self.array(&clip.clip_commands)?;
                            }
                        }
                    }
                }
            }
            PaintOp::Path { path, .. } => {
                self.array(&path.commands)?;
                if let Some(gradient) = &path.gradient {
                    self.array(&gradient.colors)?;
                    self.array(&gradient.positions)?;
                }
            }
            PaintOp::Rectangle { rect, .. } => {
                if let Some(gradient) = &rect.gradient {
                    self.array(&gradient.colors)?;
                    self.array(&gradient.positions)?;
                }
            }
            PaintOp::Ellipse { ellipse, .. } => {
                if let Some(gradient) = &ellipse.gradient {
                    self.array(&gradient.colors)?;
                    self.array(&gradient.positions)?;
                }
            }
            PaintOp::Equation { equation, .. } => {
                self.resource(equation.svg_content.len())?;
                self.text(&equation.script)?;
            }
            PaintOp::ControlLabel { label, .. } => self.text(label)?,
            PaintOp::FootnoteMarker { marker, .. } => {
                self.text(&marker.text)?;
                self.text(&marker.font_family)?;
            }
            PaintOp::FormObject { form, .. } => {
                self.text(&form.caption)?;
                self.text(&form.text)?;
                if let Some(display) = &form.display_text {
                    self.text(display)?;
                }
            }
            PaintOp::Placeholder { placeholder, .. } => self.text(&placeholder.label)?,
            PaintOp::Line { .. } => {}
        }
        Ok(())
    }
}

fn account_inputs(
    tree: &PageLayerTree,
    fonts: &[Vec<u8>],
    limits: NativeRenderSnapshotLimits,
) -> Result<NativeRenderSnapshotUsage, NativeHostError> {
    let mut account = Accounting {
        limits,
        usage: NativeRenderSnapshotUsage::default(),
    };
    for bytes in fonts {
        account.resource(bytes.len())?;
        account.usage.explicit_font_bytes += bytes.len();
    }
    for (_, bytes) in tree.resources.image_resources() {
        account.resource(bytes.len())?;
    }
    for (_, svg) in tree.resources.svg_resources() {
        account.resource(svg.len())?;
    }
    for (_, bytes) in tree.resources.font_blob_resources() {
        account.resource(bytes.len())?;
    }
    account.array(&tree.text_sources.entries)?;
    for source in &tree.text_sources.entries {
        account.text(&source.text)?;
        if let Some(key) = &source.stable_source_key {
            account.text(key)?;
        }
        account.array(&source.annotations)?;
    }

    // The pending stack is bounded before children are pushed. No recursion or
    // host callback is used, and the shared IR itself is never cloned here.
    let mut pending = vec![(&tree.root, 1usize)];
    while let Some((node, depth)) = pending.pop() {
        if depth > limits.max_depth {
            return Err(limit("render snapshot layer depth exceeds its limit"));
        }
        account.usage.nodes = account
            .usage
            .nodes
            .checked_add(1)
            .filter(|value| *value <= limits.max_nodes)
            .ok_or_else(|| limit("render snapshot layer count exceeds its limit"))?;
        account.add(size_of::<LayerNode>())?;
        account.usage.structural_bytes += size_of::<LayerNode>();
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                account.pending(pending.len(), children.len())?;
                pending.extend(children.iter().rev().map(|child| (child, depth + 1)));
            }
            LayerNodeKind::ClipRect { child, .. } => {
                account.pending(pending.len(), 1)?;
                pending.push((child, depth + 1));
            }
            LayerNodeKind::ClipPath { paths, child } => {
                account.array(paths)?;
                for path in paths {
                    account.array(&path.commands)?;
                }
                account.pending(pending.len(), 1)?;
                pending.push((child, depth + 1));
            }
            LayerNodeKind::Leaf { ops } => {
                account.usage.operations = account
                    .usage
                    .operations
                    .checked_add(ops.len())
                    .filter(|value| *value <= limits.max_operations)
                    .ok_or_else(|| limit("render snapshot operation count exceeds its limit"))?;
                let bytes = ops
                    .len()
                    .checked_mul(size_of::<PaintOp>())
                    .ok_or_else(|| limit("render snapshot operation byte count overflow"))?;
                account.add(bytes)?;
                account.usage.structural_bytes += bytes;
                for operation in ops {
                    account.operation(operation)?;
                }
            }
        }
    }
    Ok(account.usage)
}
