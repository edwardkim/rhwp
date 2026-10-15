//! Derived recording geometry, deliberately separate from layout/hit-test bounds.

use std::collections::HashMap;

use super::paint_op::{
    ColorGlyphFormat, ColorPaintGraphNodeKind, ColorPaintGraphPayload, GlyphOutlinePayloadKind,
    LayerAffineTransform, LayerGlyphOutlinePaint, PaintBounds, PaintOp, TextDecorationKind,
};
use super::{LayerNode, LayerNodeKind, PageLayerTree, MAX_POSITIONED_CONTROL_MARKS_PER_RUN};
use crate::model::style::UnderlineType;
use crate::renderer::composer::expand_pua_display_text;
use crate::renderer::render_tree::{BoundingBox, ShapeTransform, TextRunNode};
use crate::renderer::{
    clamp_tab_leader_end_x, svg_arc_to_beziers, LineRenderType, LineStyle, PathCommand, ShadowStyle,
};

impl PaintOp {
    /// Returns logical bounds unchanged and an estimate of visible page-space ink.
    ///
    /// This is derived on every call: public mutable PaintOps cannot leave a stale
    /// recording rectangle behind. Text padding preserves the old Skia estimate;
    /// it does not replace font measurement, source geometry or authored clips.
    pub fn paint_bounds(&self) -> PaintBounds {
        let logical = self.bounds();
        let visual = match self {
            Self::PageBackground { background, .. } => {
                expand(logical, background.border_width * 0.5)
            }
            Self::TextRun { run, .. } => text_bounds(logical, run),
            Self::GlyphRun { run, .. } => expand(
                logical,
                run.paint_style
                    .font_size
                    .max(1.0)
                    .max(run.paint_style.shadow_offset_x.abs())
                    .max(run.paint_style.shadow_offset_y.abs())
                    * 0.2,
            ),
            Self::GlyphOutline { outline, .. } => {
                let padding = outline
                    .paint_style
                    .font_size
                    .max(1.0)
                    .max(outline.paint_style.shadow_offset_x.abs())
                    .max(outline.paint_style.shadow_offset_y.abs())
                    * 0.2;
                let mut bounds = expand(
                    logical,
                    padding.max(
                        outline
                            .stroke
                            .as_ref()
                            .map_or(0.0, |stroke| stroke.width * 0.5),
                    ),
                );
                if let Some(paths) = glyph_outline_bounds(outline) {
                    bounds = union(bounds, paths);
                }
                // Bitmap placement and fontEmBox are already page-space. Applying
                // run_to_page here would translate a positioned font glyph twice.
                if let Some(bitmap) = &outline.bitmap_glyph {
                    bounds = union(
                        bounds,
                        affine_bounds(bitmap.placement, bitmap.transform_to_run),
                    );
                }
                if let Some(svg) = &outline.svg_glyph {
                    bounds = union(
                        bounds,
                        affine_bounds(svg.font_em_box.unwrap_or(logical), svg.transform_to_run),
                    );
                }
                bounds
            }
            Self::CharOverlap { run, .. } => {
                let padding = run
                    .style
                    .font_size
                    .max(1.0)
                    .max(run.style.shadow_offset_x.abs())
                    .max(run.style.shadow_offset_y.abs())
                    * 0.15;
                rotate_text(expand(logical, padding), logical, run)
            }
            Self::TextControlMark { run, .. } => control_mark_bounds(logical, run),
            Self::TabLeader { run, .. } => tab_leader_bounds(logical, run),
            Self::TextDecoration {
                run,
                kind,
                trim_trailing_spaces,
                ..
            } => decoration_bounds(logical, run, *kind, *trim_trailing_spaces),
            Self::FootnoteMarker { marker, .. } => expand(logical, marker.base_font_size * 0.15),
            Self::Line { line, .. } => {
                let base = arrow_bounds(
                    union(logical, line.ink_bbox()),
                    crate::renderer::arrow::line_heads(
                        (line.x1, line.y1, line.x2, line.y2),
                        &line.style,
                    ),
                    line.style.width,
                );
                let bounds = with_shadow(
                    expand(base, line_padding(&line.style)),
                    line.style.shadow.as_ref(),
                );
                shape_bounds(bounds, logical, line.transform)
            }
            Self::Rectangle { rect, .. } => shape_bounds(
                with_shadow(
                    expand(logical, rect.style.stroke_width * 0.5),
                    rect.style.shadow.as_ref(),
                ),
                logical,
                rect.transform,
            ),
            Self::Ellipse { ellipse, .. } => shape_bounds(
                with_shadow(
                    expand(logical, ellipse.style.stroke_width * 0.5),
                    ellipse.style.shadow.as_ref(),
                ),
                logical,
                ellipse.transform,
            ),
            Self::Path { path, .. } => {
                let padding = (path.style.stroke_width * 0.5)
                    .max(path.line_style.as_ref().map_or(0.0, line_padding));
                let base = arrow_bounds(
                    logical,
                    crate::renderer::arrow::connector_heads(path),
                    path.line_style.as_ref().map_or(0.0, |style| style.width),
                );
                let bounds = with_shadow(expand(base, padding), path.style.shadow.as_ref());
                let bounds = with_shadow(
                    bounds,
                    path.line_style
                        .as_ref()
                        .and_then(|style| style.shadow.as_ref()),
                );
                shape_bounds(bounds, logical, path.transform)
            }
            Self::Image { image, .. } => {
                let effective = image
                    .transform
                    .effective_image_bbox(&image.paint_bbox(&logical));
                shape_bounds(effective, effective, image.transform)
            }
            Self::ControlLabel { .. }
            | Self::Equation { .. }
            | Self::FormObject { .. }
            | Self::Placeholder { .. }
            | Self::RawSvg { .. } => logical,
        };
        PaintBounds { logical, visual }
    }

    pub fn visual_bounds(&self) -> BoundingBox {
        self.paint_bounds().visual
    }
}

impl LayerNode {
    /// Unions descendant paint estimates without changing this node's layout box.
    /// `clip_enabled` applies the producer's exact clips, with no inferred slop.
    /// An empty/fully clipped subtree has a zero-size visual extent.
    pub fn paint_bounds(&self, clip_enabled: bool) -> PaintBounds {
        PaintBounds {
            logical: self.bounds,
            visual: self.visual_bounds(clip_enabled),
        }
    }

    pub fn visual_bounds(&self, clip_enabled: bool) -> BoundingBox {
        collect_visual_bounds(self, clip_enabled, &mut |_, _| {}).unwrap_or_default()
    }
}

impl PageLayerTree {
    /// Uses this export's clip option, not an implicit backend clip policy.
    pub fn paint_bounds(&self) -> PaintBounds {
        self.root.paint_bounds(self.output_options.clip_enabled)
    }

    pub fn visual_bounds(&self) -> BoundingBox {
        self.paint_bounds().visual
    }

    /// Build once per export. Recomputing each group recursively while writing
    /// its children would be quadratic for nested groups. Pointer keys are valid
    /// only for this immutable borrow and are never serialized or retained.
    pub(super) fn visual_bounds_index(&self) -> HashMap<*const LayerNode, BoundingBox> {
        let mut bounds = HashMap::new();
        collect_visual_bounds(
            &self.root,
            self.output_options.clip_enabled,
            &mut |node, bbox| {
                if let Some(bbox) = bbox.filter(|bbox| bbox.width > 0.0 && bbox.height > 0.0) {
                    bounds.insert(node, bbox);
                }
            },
        );
        bounds
    }
}

fn collect_visual_bounds(
    node: &LayerNode,
    clip_enabled: bool,
    visit: &mut impl FnMut(*const LayerNode, Option<BoundingBox>),
) -> Option<BoundingBox> {
    let bounds = match &node.kind {
        LayerNodeKind::Leaf { ops } => ops
            .iter()
            .map(PaintOp::visual_bounds)
            .filter(|bounds| valid(*bounds))
            .reduce(union),
        LayerNodeKind::Group { children, .. } => children
            .iter()
            .filter_map(|child| collect_visual_bounds(child, clip_enabled, visit))
            .reduce(union),
        LayerNodeKind::ClipRect { clip, child, .. } => {
            let child = collect_visual_bounds(child, clip_enabled, visit);
            if clip_enabled {
                child.and_then(|child| intersection(child, *clip))
            } else {
                child
            }
        }
    };
    visit(node, bounds);
    bounds
}

fn valid(bounds: BoundingBox) -> bool {
    [
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        bounds.x + bounds.width,
        bounds.y + bounds.height,
    ]
    .into_iter()
    .all(f64::is_finite)
        && bounds.width >= 0.0
        && bounds.height >= 0.0
}

fn expand(bounds: BoundingBox, amount: f64) -> BoundingBox {
    if !amount.is_finite() || amount <= 0.0 {
        return bounds;
    }
    let expanded = BoundingBox::new(
        bounds.x - amount,
        bounds.y - amount,
        bounds.width + amount * 2.0,
        bounds.height + amount * 2.0,
    );
    if valid(expanded) {
        expanded
    } else {
        bounds
    }
}

fn union(a: BoundingBox, b: BoundingBox) -> BoundingBox {
    if !valid(a) {
        return b;
    }
    if !valid(b) {
        return a;
    }
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let result = BoundingBox::new(
        x,
        y,
        (a.x + a.width).max(b.x + b.width) - x,
        (a.y + a.height).max(b.y + b.height) - y,
    );
    if valid(result) {
        result
    } else {
        a
    }
}

fn intersection(a: BoundingBox, b: BoundingBox) -> Option<BoundingBox> {
    if !valid(a) || !valid(b) {
        return None;
    }
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > x && bottom > y).then(|| BoundingBox::new(x, y, right - x, bottom - y))
}

fn with_shadow(bounds: BoundingBox, shadow: Option<&ShadowStyle>) -> BoundingBox {
    let Some(shadow) = shadow else {
        return bounds;
    };
    let offset = BoundingBox::new(
        bounds.x + shadow.offset_x,
        bounds.y + shadow.offset_y,
        bounds.width,
        bounds.height,
    );
    union(
        bounds,
        expand(offset, shadow.offset_x.abs().max(shadow.offset_y.abs())),
    )
}

fn line_padding(style: &LineStyle) -> f64 {
    let mut padding = style.width.max(0.0) * 0.5;
    if style.line_type != LineRenderType::Single {
        padding = padding.max(style.width.max(1.0) * 2.0);
    }
    padding
}

// Control points contain the cubic outline's convex hull. Reuse the same
// positioned arrow geometry as each backend rather than estimating by size code.
fn arrow_bounds(
    mut bounds: BoundingBox,
    heads: Vec<crate::renderer::arrow::ArrowHead>,
    stroke: f64,
) -> BoundingBox {
    for head in heads {
        for command in head.commands {
            let mut include = |x, y| {
                bounds = union(
                    bounds,
                    expand(BoundingBox::new(x, y, 0.0, 0.0), stroke.max(0.5) * 0.5),
                );
            };
            match command {
                PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => include(x, y),
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    include(x1, y1);
                    include(x2, y2);
                    include(x, y);
                }
                PathCommand::ClosePath => {}
                PathCommand::ArcTo(..) => unreachable!("shared arrows use cubic outlines"),
            }
        }
    }
    bounds
}

fn affine_bounds(bounds: BoundingBox, transform: Option<LayerAffineTransform>) -> BoundingBox {
    let Some(t) = transform else {
        return bounds;
    };
    checked_affine_bounds(bounds, t).unwrap_or(bounds)
}

fn checked_affine_bounds(bounds: BoundingBox, t: LayerAffineTransform) -> Option<BoundingBox> {
    let (mut left, mut top) = (f64::INFINITY, f64::INFINITY);
    let (mut right, mut bottom) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (x, y) in [
        (bounds.x, bounds.y),
        (bounds.x + bounds.width, bounds.y),
        (bounds.x, bounds.y + bounds.height),
        (bounds.x + bounds.width, bounds.y + bounds.height),
    ] {
        let point = BoundingBox::new(t.a * x + t.c * y + t.e, t.b * x + t.d * y + t.f, 0.0, 0.0);
        if !valid(point) {
            return None;
        }
        left = left.min(point.x);
        top = top.min(point.y);
        right = right.max(point.x);
        bottom = bottom.max(point.y);
    }
    let result = BoundingBox::new(left, top, right - left, bottom - top);
    valid(result).then_some(result)
}

// Canvas concatenation applies the child matrix before its enclosing matrix.
fn concat_affine(
    parent: LayerAffineTransform,
    child: LayerAffineTransform,
) -> LayerAffineTransform {
    LayerAffineTransform {
        a: parent.a * child.a + parent.c * child.b,
        b: parent.b * child.a + parent.d * child.b,
        c: parent.a * child.c + parent.c * child.d,
        d: parent.b * child.c + parent.d * child.d,
        e: parent.a * child.e + parent.c * child.f + parent.e,
        f: parent.b * child.e + parent.d * child.f + parent.f,
    }
}

/// Outward-rounded f32 arithmetic also encloses fused multiply/add and wider
/// intermediates. Keep ranges through concatenation: rounding only the final
/// point misses cancellation between large, individually rounded matrices.
#[derive(Clone, Copy)]
struct F32Range {
    min: f64,
    max: f64,
}

impl F32Range {
    fn scalar(value: f64) -> Self {
        let value = f64::from(value as f32);
        Self {
            min: value,
            max: value,
        }
    }

    fn rounded(min: f64, max: f64) -> Self {
        let mut lower = min as f32;
        let mut upper = max as f32;
        if !min.is_finite() || !max.is_finite() || !lower.is_finite() || !upper.is_finite() {
            return Self {
                min: f64::NAN,
                max: f64::NAN,
            };
        }
        if f64::from(lower) > min {
            lower = lower.next_down();
        }
        if f64::from(upper) < max {
            upper = upper.next_up();
        }
        Self {
            min: f64::from(lower),
            max: f64::from(upper),
        }
    }

    fn add(self, other: Self) -> Self {
        Self::rounded(self.min + other.min, self.max + other.max)
    }

    fn mul(self, other: Self) -> Self {
        let products = [
            self.min * other.min,
            self.min * other.max,
            self.max * other.min,
            self.max * other.max,
        ];
        if products.iter().any(|value| !value.is_finite()) {
            return Self {
                min: f64::NAN,
                max: f64::NAN,
            };
        }
        Self::rounded(
            products.into_iter().fold(f64::INFINITY, f64::min),
            products.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
}

type F32Affine = [[F32Range; 3]; 2];

fn f32_affine(t: LayerAffineTransform) -> F32Affine {
    [[t.a, t.c, t.e], [t.b, t.d, t.f]].map(|row| row.map(F32Range::scalar))
}

fn concat_f32_affine(parent: F32Affine, child: F32Affine) -> F32Affine {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            let value = parent[row][0]
                .mul(child[0][column])
                .add(parent[row][1].mul(child[1][column]));
            if column == 2 {
                value.add(parent[row][2])
            } else {
                value
            }
        })
    })
}

fn f32_affine_bounds(bounds: BoundingBox, transform: F32Affine) -> Option<BoundingBox> {
    let x = F32Range::rounded(bounds.x, bounds.x + bounds.width);
    let y = F32Range::rounded(bounds.y, bounds.y + bounds.height);
    let [x, y] = transform.map(|row| row[0].mul(x).add(row[1].mul(y)).add(row[2]));
    let result = BoundingBox::new(x.min, y.min, x.max - x.min, y.max - y.min);
    valid(result).then_some(result)
}

fn glyph_transformed_bounds(
    bounds: BoundingBox,
    transforms: &[LayerAffineTransform],
    clip_path: bool,
) -> Option<BoundingBox> {
    let (&first, rest) = transforms.split_first()?;
    let exact = rest.iter().copied().fold(first, concat_affine);
    let native = if clip_path {
        // PreparedGlyph clips are transformed paths, not a concatenated canvas
        // matrix. Enclose rounding after each reversed path transformation too.
        transforms
            .iter()
            .rev()
            .try_fold(bounds, |bounds, &transform| {
                f32_affine_bounds(bounds, f32_affine(transform))
            })?
    } else {
        let transform = rest
            .iter()
            .copied()
            .map(f32_affine)
            .fold(f32_affine(first), concat_f32_affine);
        f32_affine_bounds(bounds, transform)?
    };
    Some(union(checked_affine_bounds(bounds, exact)?, native))
}

fn glyph_outline_bounds(outline: &LayerGlyphOutlinePaint) -> Option<BoundingBox> {
    let placement = outline.placement.run_to_page;
    match outline.payload_kind {
        GlyphOutlinePayloadKind::MonochromeFill | GlyphOutlinePayloadKind::MonochromeFillStroke => {
            if outline.paths.len() > 4096 {
                return None;
            }
            let stroke_padding =
                if outline.payload_kind == GlyphOutlinePayloadKind::MonochromeFillStroke {
                    let stroke = outline.stroke.as_ref()?;
                    if !stroke.is_strict_subset() {
                        return None;
                    }
                    // Miter length is bounded in path space, before scale/skew.
                    (stroke.width * 0.5 * stroke.miter_limit).max(
                        F32Range::scalar(stroke.width)
                            .mul(F32Range::scalar(0.5))
                            .mul(F32Range::scalar(stroke.miter_limit))
                            .max,
                    )
                } else {
                    0.0
                };
            let mut bounds = None;
            for path in &outline.paths {
                let local = expand(glyph_path_bounds(&path.commands)?, stroke_padding);
                let path = glyph_transformed_bounds(local, &[placement], false)?;
                bounds = Some(bounds.map_or(path, |prior| union(prior, path)));
            }
            bounds
        }
        GlyphOutlinePayloadKind::ColorLayers => {
            let payload = outline.color_layers.as_ref()?;
            if payload.color_format == ColorGlyphFormat::ColrV0
                && payload.has_colrv0_resolved_layer_contract()
            {
                let mut bounds = None;
                for layer in &payload.layers {
                    let mut transforms = vec![placement];
                    transforms.extend(layer.transform_to_run);
                    let path = glyph_transformed_bounds(
                        glyph_path_bounds(layer.commands.as_deref()?)?,
                        &transforms,
                        false,
                    )?;
                    bounds = Some(bounds.map_or(path, |prior| union(prior, path)));
                }
                bounds
            } else if payload.has_colrv1_supported_graph_contract() {
                color_graph_bounds(payload.paint_graph.as_ref()?, placement)
            } else {
                None
            }
        }
        // These payloads carry page-space placements and are handled separately.
        GlyphOutlinePayloadKind::BitmapGlyph | GlyphOutlinePayloadKind::SvgGlyph => None,
    }
}

fn color_graph_bounds(
    graph: &ColorPaintGraphPayload,
    placement: LayerAffineTransform,
) -> Option<BoundingBox> {
    // The caller checks the graph's bounded, acyclic replay contract. Do not
    // deduplicate shared leaves: each parent can supply a different transform.
    let mut pending = vec![(graph.root_node_id, vec![placement], None::<BoundingBox>)];
    let mut bounds = None;
    while let Some((id, mut transforms, clip)) = pending.pop() {
        let node = graph.nodes.iter().find(|node| node.node_id == id)?;
        let commands = match node.kind {
            ColorPaintGraphNodeKind::Transform => {
                let child = node.transform.as_ref()?;
                transforms.push(child.transform);
                pending.push((child.child_node_id, transforms, clip));
                continue;
            }
            ColorPaintGraphNodeKind::Composite => {
                let composite = node.composite.as_ref()?;
                pending.push((composite.source_node_id, transforms.clone(), clip));
                pending.push((composite.backdrop_node_id, transforms, clip));
                continue;
            }
            ColorPaintGraphNodeKind::Clip => {
                let child = node.clip.as_ref()?;
                let own_clip = glyph_transformed_bounds(
                    glyph_path_bounds(&child.clip_commands)?,
                    &transforms,
                    true,
                )?;
                let clipped = match clip {
                    Some(clip) => intersection(clip, own_clip),
                    None => Some(own_clip),
                };
                if let Some(clipped) = clipped {
                    // A later child transform must not move this enclosing clip.
                    pending.push((child.child_node_id, transforms, Some(clipped)));
                }
                continue;
            }
            ColorPaintGraphNodeKind::SolidPath => &node.solid_path.as_ref()?.commands,
            ColorPaintGraphNodeKind::LinearGradientPath => {
                &node.linear_gradient_path.as_ref()?.commands
            }
            ColorPaintGraphNodeKind::RadialGradientPath => {
                &node.radial_gradient_path.as_ref()?.commands
            }
            ColorPaintGraphNodeKind::SweepGradientPath => {
                &node.sweep_gradient_path.as_ref()?.commands
            }
        };
        let path = glyph_transformed_bounds(glyph_path_bounds(commands)?, &transforms, false)?;
        let path = match clip {
            Some(clip) => intersection(path, clip),
            None => Some(path),
        };
        if let Some(path) = path {
            bounds = Some(bounds.map_or(path, |prior| union(prior, path)));
        }
    }
    bounds
}

fn glyph_path_bounds(commands: &[PathCommand]) -> Option<BoundingBox> {
    if !super::paint_op::path_commands_are_finite(commands) {
        return None;
    }
    let mut bounds = None;
    let mut include = |(x, y)| {
        // PathBuilder receives f32 endpoints/control points, including the
        // cubic points produced by the shared f64 arc conversion.
        for (x, y) in [(x, y), (f64::from(x as f32), f64::from(y as f32))] {
            let point = BoundingBox::new(x, y, 0.0, 0.0);
            if !valid(point) {
                return None;
            }
            bounds = Some(bounds.map_or(point, |prior| union(prior, point)));
        }
        Some(())
    };
    let mut current = (0.0, 0.0);
    let mut origin = current;
    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                include((x, y))?;
                origin = (x, y);
                current = origin;
            }
            PathCommand::LineTo(x, y) => {
                include(current)?;
                include((x, y))?;
                current = (x, y);
            }
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                // A cubic lies within its endpoints/control-point convex hull.
                for point in [current, (x1, y1), (x2, y2), (x, y)] {
                    include(point)?;
                }
                current = (x, y);
            }
            PathCommand::ArcTo(rx, ry, rotation, large, sweep, x, y) => {
                include(current)?;
                include((x, y))?;
                if rx != 0.0 && ry != 0.0 {
                    // Native replay draws these same cubic approximations.
                    for segment in svg_arc_to_beziers(
                        current.0, current.1, rx, ry, rotation, large, sweep, x, y,
                    ) {
                        if let PathCommand::CurveTo(x1, y1, x2, y2, x, y) = segment {
                            for point in [(x1, y1), (x2, y2), (x, y)] {
                                include(point)?;
                            }
                        }
                    }
                }
                current = (x, y);
            }
            PathCommand::ClosePath => current = origin,
        }
    }
    bounds
}

fn shape_bounds(
    bounds: BoundingBox,
    logical: BoundingBox,
    transform: ShapeTransform,
) -> BoundingBox {
    let radians = transform.rotation.to_radians();
    let (sin, cos) = radians.sin_cos();
    let sx = if transform.horz_flip { -1.0 } else { 1.0 };
    let sy = if transform.vert_flip { -1.0 } else { 1.0 };
    let (a, b, c, d) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    let cx = logical.x + logical.width * 0.5;
    let cy = logical.y + logical.height * 0.5;
    affine_bounds(
        bounds,
        Some(LayerAffineTransform {
            a,
            b,
            c,
            d,
            e: cx - a * cx - c * cy,
            f: cy - b * cx - d * cy,
        }),
    )
}

fn rotate_text(bounds: BoundingBox, logical: BoundingBox, run: &TextRunNode) -> BoundingBox {
    shape_bounds(
        bounds,
        logical,
        ShapeTransform {
            rotation: run.rotation,
            ..Default::default()
        },
    )
}

fn text_bounds(logical: BoundingBox, run: &TextRunNode) -> BoundingBox {
    let (font_size, _) = effective_text_font_size_and_baseline(run);
    let mut padding: f64 = 0.0;
    if run.style.underline != UnderlineType::None || run.style.strikethrough {
        padding = font_size * 0.2;
    }
    if run.style.outline_type != 0 || run.style.emboss || run.style.engrave {
        padding = padding.max(font_size * 0.15);
    }
    if run.style.emphasis_dot != 0 {
        padding = padding.max(font_size * 0.35);
    }
    let mut bounds = expand(logical, padding);
    if run.style.shadow_type != 0 {
        let offset = BoundingBox::new(
            logical.x + run.style.shadow_offset_x,
            logical.y + run.style.shadow_offset_y,
            logical.width,
            logical.height,
        );
        bounds = union(
            bounds,
            expand(
                offset,
                run.style
                    .shadow_offset_x
                    .abs()
                    .max(run.style.shadow_offset_y.abs())
                    .max(font_size * 0.1),
            ),
        );
    }
    rotate_text(bounds, logical, run)
}

fn control_mark_bounds(logical: BoundingBox, run: &TextRunNode) -> BoundingBox {
    let (text, _) = bounded_text_prefix(&run.text);
    let positions = run.replay_positions_prefix_for(&run.text, &text);
    let font_size = if run.style.font_size > 0.0 {
        run.style.font_size
    } else {
        12.0
    };
    let mut bounds = logical;
    for x in positions
        .iter()
        .copied()
        .chain((run.is_para_end || run.is_line_break_end).then_some(logical.width))
    {
        bounds = union(
            bounds,
            BoundingBox::new(
                logical.x + x - font_size * 0.25,
                logical.y + run.baseline - font_size,
                font_size * 1.5,
                font_size * 1.2,
            ),
        );
    }
    rotate_text(bounds, logical, run)
}

fn tab_leader_bounds(logical: BoundingBox, run: &TextRunNode) -> BoundingBox {
    let (text, _) = bounded_display_text_for_run(run);
    let positions = run.replay_positions_prefix_for(run.display_or_text(), &text);
    let (font_size, baseline) = effective_text_font_size_and_baseline(run);
    let mut bounds = logical;
    for leader in run
        .style
        .tab_leaders
        .iter()
        .take(MAX_POSITIONED_CONTROL_MARKS_PER_RUN)
    {
        let end = clamp_tab_leader_end_x(&text, &positions, leader, font_size);
        bounds = union(
            bounds,
            BoundingBox::new(
                logical.x + leader.start_x.min(end),
                logical.y + baseline - font_size * 0.4,
                (end - leader.start_x).abs(),
                font_size * 0.5,
            ),
        );
    }
    rotate_text(bounds, logical, run)
}

fn decoration_bounds(
    logical: BoundingBox,
    run: &TextRunNode,
    kind: TextDecorationKind,
    trim: usize,
) -> BoundingBox {
    let (text, _) = bounded_display_text_for_run(run);
    let positions = run.replay_positions_prefix_for(run.display_or_text(), &text);
    let trailing = text.chars().rev().take_while(|ch| *ch == ' ').count();
    let end = positions.len().saturating_sub(trim.min(trailing));
    let width = positions.get(end.saturating_sub(1)).copied().unwrap_or(0.0);
    let (font_size, baseline) = effective_text_font_size_and_baseline(run);
    let (y, height) = match kind {
        TextDecorationKind::Underline => (
            if run.style.underline == UnderlineType::Top {
                baseline - font_size + 1.0
            } else {
                baseline + 2.0
            },
            1.0,
        ),
        TextDecorationKind::Strikethrough => (baseline - font_size * 0.3, 1.0),
        TextDecorationKind::EmphasisDot => (baseline - font_size * 1.2, font_size * 0.3),
    };
    rotate_text(
        union(
            logical,
            expand(
                BoundingBox::new(logical.x, logical.y + y, width, height),
                2.0,
            ),
        ),
        logical,
        run,
    )
}

// The JSON writer and recording estimates must use the same bounded display
// projection, script metrics and producer positions for explicit text visuals.
pub(super) fn bounded_text_prefix(text: &str) -> (String, bool) {
    let mut chars = text.chars();
    let prefix = chars
        .by_ref()
        .take(MAX_POSITIONED_CONTROL_MARKS_PER_RUN)
        .collect();
    (prefix, chars.next().is_none())
}

pub(super) fn bounded_display_text_for_run(run: &TextRunNode) -> (String, bool) {
    let (source_prefix, source_complete) = bounded_text_prefix(run.display_or_text());
    let display_text = expand_pua_display_text(&source_prefix);
    let (display_prefix, display_complete) = bounded_text_prefix(&display_text);
    (display_prefix, source_complete && display_complete)
}

pub(super) fn effective_text_font_size_and_baseline(run: &TextRunNode) -> (f64, f64) {
    let base_font_size = if run.style.font_size > 0.0 {
        run.style.font_size
    } else {
        12.0
    };
    run.style.script_draw_metrics(base_font_size, run.baseline)
}
