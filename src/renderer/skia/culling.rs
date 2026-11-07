//! Proven bounds only. Estimated text/SVG/effect bounds cannot justify culling.

use std::collections::HashMap;

use crate::paint::{GroupKind, LayerNode, LayerNodeKind, PaintOp};
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::{ShapeStyle, StrokeDash};
use skia_safe::Canvas;

const MAX_PROOF_ITEMS: usize = 65_536;

#[derive(Clone, Copy)]
struct ProvenBounds {
    edges: [f32; 4],
    stroke: f32,
}

impl ProvenBounds {
    fn union(self, other: Self) -> Self {
        Self {
            edges: [
                self.edges[0].min(other.edges[0]),
                self.edges[1].min(other.edges[1]),
                self.edges[2].max(other.edges[2]),
                self.edges[3].max(other.edges[3]),
            ],
            stroke: self.stroke.max(other.stroke),
        }
    }
}

#[derive(Default)]
pub(super) struct CullingIndex {
    known: HashMap<*const LayerNode, ProvenBounds>,
}

impl CullingIndex {
    /// One bounded post-order walk per replay. No retained cache or recursion.
    pub fn new(root: &LayerNode, enabled: bool) -> Self {
        let mut index = Self::default();
        if !enabled {
            return index;
        }
        let mut work = 0usize;
        let mut stack = vec![(root, false)];
        while let Some((node, visited)) = stack.pop() {
            if !visited {
                work += 1;
                if work > MAX_PROOF_ITEMS {
                    return Self::default();
                }
                stack.push((node, true));
                match &node.kind {
                    LayerNodeKind::Group { children, .. } => {
                        if children.len().saturating_add(stack.len()) > MAX_PROOF_ITEMS {
                            return Self::default();
                        }
                        stack.extend(children.iter().rev().map(|child| (child, false)));
                    }
                    LayerNodeKind::ClipRect { child, .. }
                    | LayerNodeKind::ClipPath { child, .. } => stack.push((child, false)),
                    LayerNodeKind::Leaf { .. } => {}
                }
                continue;
            }
            let proof = match &node.kind {
                LayerNodeKind::Leaf { ops } => {
                    work = work.saturating_add(ops.len());
                    if work > MAX_PROOF_ITEMS {
                        return Self::default();
                    }
                    ops.iter()
                        .map(prove_op)
                        .try_fold(None, |bounds, next| {
                            let next = next?;
                            Some(Some(
                                bounds.map_or(next, |old: ProvenBounds| old.union(next)),
                            ))
                        })
                        .flatten()
                }
                LayerNodeKind::Group {
                    children,
                    group_kind,
                    ..
                } => {
                    if matches!(group_kind, GroupKind::Group(group) if group.paint_transform.is_some())
                    {
                        None
                    } else {
                        children
                            .iter()
                            .try_fold(None, |bounds, child| {
                                let next = *index.known.get(&(child as *const LayerNode))?;
                                Some(Some(
                                    bounds.map_or(next, |old: ProvenBounds| old.union(next)),
                                ))
                            })
                            .flatten()
                    }
                }
                // Ignoring a clip only overestimates ink. An unknown child
                // remains unknown even if its estimated bounds miss the clip.
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                    index.known.get(&(&**child as *const LayerNode)).copied()
                }
            };
            if let Some(proof) = proof {
                index.known.insert(node, proof);
            }
        }
        index
    }

    pub fn is_outside(&self, node: &LayerNode, canvas: &Canvas) -> bool {
        let Some(bounds) = self.known.get(&(node as *const LayerNode)) else {
            return false;
        };
        let matrix = canvas.local_to_device_as_3x3();
        // The first subset proves device padding for axis-aligned transforms.
        // Rotations, skew, perspective and degenerate matrices retain replay.
        if matrix.has_perspective() || matrix.skew_x() != 0.0 || matrix.skew_y() != 0.0 {
            return false;
        }
        let unit = (1.0 / matrix.scale_x().abs()).max(1.0 / matrix.scale_y().abs());
        if !unit.is_finite() || unit <= 0.0 {
            return false;
        }
        let Some(clip) = canvas.local_clip_bounds() else {
            return true;
        };
        if !clip.is_finite() {
            return false;
        }
        // Cover the raster's one-device-pixel stroke floor, miter extent,
        // antialiasing and Print rectangle fill snapping. Use f32 geometry,
        // matching the actual draw, rather than rounding an f64 estimate inward.
        let padding = bounds.stroke.max(unit) * 2.0 + unit * 2.0;
        let [left, top, right, bottom] = bounds.edges;
        let expanded = [
            left - padding,
            top - padding,
            right + padding,
            bottom + padding,
        ];
        if !expanded.iter().all(|value| value.is_finite()) {
            return false;
        }
        expanded[2] < clip.left
            || expanded[3] < clip.top
            || expanded[0] > clip.right
            || expanded[1] > clip.bottom
    }
}

fn prove_op(op: &PaintOp) -> Option<ProvenBounds> {
    let (bbox, style) = match op {
        PaintOp::Rectangle { bbox, rect }
            if !rect.transform.has_transform()
                && rect.gradient.is_none()
                && !rect.picture_border
                && rect.print_picture_pen.is_none()
                && rect.corner_radius.is_finite()
                && (rect.corner_radius as f32).is_finite()
                && rect.corner_radius >= 0.0 =>
        {
            (bbox, &rect.style)
        }
        PaintOp::Ellipse { bbox, ellipse }
            if !ellipse.transform.has_transform() && ellipse.gradient.is_none() =>
        {
            (bbox, &ellipse.style)
        }
        _ => return None,
    };
    prove_shape(*bbox, style)
}

fn prove_shape(bbox: BoundingBox, style: &ShapeStyle) -> Option<ProvenBounds> {
    if style.shadow.is_some()
        || style.pattern.is_some()
        || style.stroke_dash != StrokeDash::Solid
        || !style.opacity.is_finite()
        || !(0.0..=1.0).contains(&style.opacity)
        || !style.stroke_width.is_finite()
        || style.stroke_width < 0.0
    {
        return None;
    }
    let left = bbox.x as f32;
    let top = bbox.y as f32;
    let width = bbox.width as f32;
    let height = bbox.height as f32;
    let edges = [left, top, left + width, top + height];
    if ![bbox.x, bbox.y, bbox.width, bbox.height]
        .iter()
        .all(|value| value.is_finite())
        || !edges.iter().all(|value| value.is_finite())
        || width <= 0.0
        || height <= 0.0
        || edges[2] <= left
        || edges[3] <= top
    {
        return None;
    }
    let stroke = if style.stroke_color.is_some() {
        if style.stroke_width > 0.0 {
            style.stroke_width as f32
        } else {
            1.0
        }
    } else {
        0.0
    };
    stroke.is_finite().then_some(ProvenBounds { edges, stroke })
}
