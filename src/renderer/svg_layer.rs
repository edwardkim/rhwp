use super::layer_renderer::{LayerRenderResult, LayerRenderer};
use super::render_tree::RenderLayerInfo;
use super::svg::{
    LayerDebugOverlayProjection, OverlayBounds, OverlayImageInfo, OverlayTableInfo,
    OverlayVposReset, SvgPaintDisposition, SvgRenderer,
};
use crate::error::HwpError;
use crate::paint::{
    paint_op_replay_plane_with_layer, render_layer_replay_plane, text_visual_replay_role,
    validate_text_variant_scope, ClipKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp,
    PaintReplayPlane, TextVisualReplayRole,
};

/// Translates the canonical `PageLayerTree` paint contract into SVG syntax.
///
/// `LayerBuilder` has already decided visibility, output profile, paint variants, and structural
/// clips. This backend owns only SVG serialization: it replays the shared plane order, preserves
/// nested clip scopes, and never reconstructs a `PageRenderTree` that could reinterpret those
/// decisions.
pub struct SvgLayerRenderer {
    renderer: SvgRenderer,
    next_generated_id: u32,
    generated_node_ids: std::collections::HashMap<usize, u32>,
    clip_enabled: bool,
}

fn project_debug_overlay(root: &LayerNode) -> LayerDebugOverlayProjection {
    fn extend_paragraph_bounds(
        projection: &mut LayerDebugOverlayProjection,
        section_index: usize,
        para_index: usize,
        bounds: crate::renderer::render_tree::BoundingBox,
    ) {
        let key = section_index * 100000 + para_index;
        let entry = projection
            .paragraph_bounds
            .entry(key)
            .or_insert(OverlayBounds {
                section_index,
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
            });
        let min_x = entry.x.min(bounds.x);
        let min_y = entry.y.min(bounds.y);
        let max_x = (entry.x + entry.width).max(bounds.x + bounds.width);
        let max_y = (entry.y + entry.height).max(bounds.y + bounds.height);
        entry.x = min_x;
        entry.y = min_y;
        entry.width = max_x - min_x;
        entry.height = max_y - min_y;
    }

    fn visit(projection: &mut LayerDebugOverlayProjection, node: &LayerNode, skip_depth: u32) {
        match &node.kind {
            LayerNodeKind::Group {
                children,
                group_kind,
                ..
            } => {
                let mut child_skip_depth = skip_depth;
                match group_kind {
                    crate::paint::GroupKind::TextLine(line) if skip_depth == 0 => {
                        if let (Some(para_index), Some(section_index)) =
                            (line.para_index, line.section_index)
                        {
                            if projection.page_section == -1 {
                                projection.page_section = section_index as i32;
                            }
                            if projection.page_section == section_index as i32 {
                                extend_paragraph_bounds(
                                    projection,
                                    section_index,
                                    para_index,
                                    node.bounds,
                                );
                                if let (Some(line_index), Some(vpos)) = (line.line_index, line.vpos)
                                {
                                    if line_index > 0 && vpos == 0 {
                                        projection.vpos_resets.push(OverlayVposReset {
                                            section_index,
                                            para_index,
                                            line_index,
                                            y: node.bounds.y,
                                            x: node.bounds.x,
                                            width: node.bounds.width,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    crate::paint::GroupKind::Table(table) => {
                        if skip_depth == 0 {
                            if let (Some(para_index), Some(control_index)) =
                                (table.para_index, table.control_index)
                            {
                                let section_index = table.section_index.unwrap_or(0);
                                if projection.page_section == -1 {
                                    projection.page_section = section_index as i32;
                                }
                                if projection.page_section == section_index as i32 {
                                    projection.table_bounds.push(OverlayTableInfo {
                                        section_index,
                                        para_index,
                                        control_index,
                                        x: node.bounds.x,
                                        y: node.bounds.y,
                                        width: node.bounds.width,
                                        height: node.bounds.height,
                                        row_count: table.row_count,
                                        col_count: table.col_count,
                                    });
                                    extend_paragraph_bounds(
                                        projection,
                                        section_index,
                                        para_index,
                                        node.bounds,
                                    );
                                }
                            }
                        }
                        child_skip_depth += 1;
                    }
                    crate::paint::GroupKind::Header
                    | crate::paint::GroupKind::Footer
                    | crate::paint::GroupKind::MasterPage
                    | crate::paint::GroupKind::FootnoteArea
                    | crate::paint::GroupKind::TextBox
                    | crate::paint::GroupKind::Group(_) => child_skip_depth += 1,
                    _ => {}
                }
                for child in children {
                    visit(projection, child, child_skip_depth);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => visit(projection, child, skip_depth),
            LayerNodeKind::Leaf { ops } if skip_depth == 0 => {
                for op in ops {
                    if let PaintOp::Image { bbox, image, .. } = op {
                        if let (Some(para_index), Some(control_index)) =
                            (image.para_index, image.control_index)
                        {
                            let section_index = image.section_index.unwrap_or(0);
                            if projection.page_section == -1 {
                                projection.page_section = section_index as i32;
                            }
                            if projection.page_section == section_index as i32 {
                                projection.image_bounds.push(OverlayImageInfo {
                                    section_index,
                                    para_index,
                                    control_index,
                                    x: bbox.x,
                                    y: bbox.y,
                                    width: bbox.width,
                                    height: bbox.height,
                                });
                            }
                        }
                    }
                }
            }
            LayerNodeKind::Leaf { .. } => {}
        }
    }

    let mut projection = LayerDebugOverlayProjection {
        paragraph_bounds: std::collections::HashMap::new(),
        table_bounds: Vec::new(),
        image_bounds: Vec::new(),
        vpos_resets: Vec::new(),
        page_section: -1,
    };
    visit(&mut projection, root, 0);
    projection
}

impl SvgLayerRenderer {
    pub fn new() -> Self {
        Self {
            renderer: SvgRenderer::new(),
            next_generated_id: 1_000_000,
            generated_node_ids: std::collections::HashMap::new(),
            clip_enabled: true,
        }
    }

    pub fn output(&self) -> &str {
        self.renderer.output()
    }

    /// Configures SVG-only serialization such as font embedding. Paint decisions must enter
    /// through `PageLayerTree`, not this escape hatch.
    pub fn inner_mut(&mut self) -> &mut SvgRenderer {
        &mut self.renderer
    }

    pub fn inner(&self) -> &SvgRenderer {
        &self.renderer
    }

    fn active_layer(
        node: &LayerNode,
        inherited_layer: Option<RenderLayerInfo>,
    ) -> Option<RenderLayerInfo> {
        // LayerBuilder publishes master-page provenance and every backend inherits exactly the
        // same metadata. SVG must not synthesize a private replay-plane decision from group kind.
        node.layer.or(inherited_layer)
    }

    fn svg_emits_paint_op(op: &PaintOp) -> bool {
        !matches!(op, PaintOp::GlyphRun { .. } | PaintOp::GlyphOutline { .. })
    }

    fn node_has_plane(
        node: &LayerNode,
        inherited_layer: Option<RenderLayerInfo>,
        plane: PaintReplayPlane,
    ) -> bool {
        let active_layer = Self::active_layer(node, inherited_layer);
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children
                .iter()
                .any(|child| Self::node_has_plane(child, active_layer, plane)),
            LayerNodeKind::ClipRect { child, .. } => {
                Self::node_has_plane(child, active_layer, plane)
            }
            LayerNodeKind::Leaf { ops } => ops.iter().any(|op| {
                Self::svg_emits_paint_op(op)
                    && paint_op_replay_plane_with_layer(op, active_layer) == plane
            }),
        }
    }

    fn render_node(
        &mut self,
        node: &LayerNode,
        inherited_layer: Option<RenderLayerInfo>,
        plane: PaintReplayPlane,
    ) -> LayerRenderResult<()> {
        if !Self::node_has_plane(node, inherited_layer, plane) {
            return Ok(());
        }
        let active_layer = Self::active_layer(node, inherited_layer);
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.render_node(child, active_layer, plane)?;
                }
            }
            LayerNodeKind::ClipRect {
                clip,
                child,
                clip_kind,
            } => {
                if !Self::node_has_plane(child, active_layer, plane) {
                    return Ok(());
                }
                if !self.clip_enabled {
                    return self.render_node(child, active_layer, plane);
                }
                let node_id = self.node_id(node);
                let prefix = match clip_kind {
                    ClipKind::Body => "body",
                    ClipKind::TableCell => "cell",
                    ClipKind::TextBox => "textbox",
                    ClipKind::Generic => "generic",
                };
                let pass_count = PaintReplayPlane::ORDERED
                    .iter()
                    .filter(|candidate| Self::node_has_plane(child, active_layer, **candidate))
                    .count();
                let clip_id = if pass_count > 1 {
                    format!("{prefix}-clip-{node_id}-{}", plane.as_str())
                } else {
                    format!("{prefix}-clip-{node_id}")
                };
                self.renderer.begin_layer_clip(*clip, &clip_id);
                self.render_node(child, active_layer, plane)?;
                self.renderer.end_layer_clip();
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if paint_op_replay_plane_with_layer(op, active_layer) != plane {
                        continue;
                    }
                    if text_visual_replay_role(op) == TextVisualReplayRole::SuppressedFallback {
                        // `CharOverlap` is the selected visual for this paint-order slot. Its
                        // paired TextRun is metadata/fallback only and must not paint base glyphs.
                        continue;
                    }
                    match self.renderer.render_layer_paint_op(op, node.source_node_id) {
                        SvgPaintDisposition::Rendered => {}
                        SvgPaintDisposition::UnsupportedTextVariant => {
                            // `validate_text_variant_scope` proves this equivalence group has a
                            // TextRun fallback in the same leaf. SVG chooses it explicitly while
                            // retaining glyph-native variants for capable CanvasKit/Skia backends.
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn node_id(&mut self, node: &LayerNode) -> u32 {
        if let Some(id) = node.source_node_id {
            return id;
        }
        let key = std::ptr::from_ref(node) as usize;
        if let Some(id) = self.generated_node_ids.get(&key) {
            return *id;
        }
        let id = self.next_generated_id;
        self.next_generated_id += 1;
        self.generated_node_ids.insert(key, id);
        id
    }
}

impl Default for SvgLayerRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Replays one in-memory layer tree without JSON or legacy render-tree reconstruction.
impl LayerRenderer for SvgLayerRenderer {
    fn render_page(&mut self, tree: &PageLayerTree) -> LayerRenderResult<()> {
        validate_text_variant_scope(tree).map_err(|error| {
            HwpError::RenderError(format!("invalid PageLayerTree text variants: {error}"))
        })?;
        self.next_generated_id = 1_000_000;
        self.generated_node_ids.clear();
        self.clip_enabled = tree.output_options.clip_enabled;
        self.renderer.show_paragraph_marks = tree.output_options.show_paragraph_marks;
        self.renderer.show_control_codes = tree.output_options.show_control_codes;
        self.renderer.debug_overlay = tree.output_options.debug_overlay;
        // Presence in the canonical tree is the visibility decision. The serializer must not
        // apply a second profile gate to missing-picture placeholders.
        self.renderer.show_missing_picture_placeholder = true;
        self.renderer.profile = tree.profile;
        self.renderer
            .begin_layer_page(tree.page_width, tree.page_height);
        if tree.output_options.debug_overlay {
            self.renderer
                .apply_layer_debug_overlay(project_debug_overlay(&tree.root));
        }
        for plane in PaintReplayPlane::ORDERED {
            if Self::node_has_plane(&tree.root, None, plane) {
                self.render_node(&tree.root, None, plane)?;
            }
        }
        self.renderer.end_layer_page();
        Ok(())
    }
}
