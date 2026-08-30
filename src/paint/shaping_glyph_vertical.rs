//! Q4-D3-A read-only bridge from a D2 vertical line sidecar to leaf-scoped
//! text sources.
//!
//! Product publication remains closed in D3-A. The builder does not call this
//! module until D3-B has separately proved atomic resource and subtree commit.

#![allow(dead_code)]

use std::sync::Arc;

use crate::paint::{LayerNode, LayerNodeKind, PaintOp};
use crate::renderer::shaping_vertical::{
    prepare_bounded_vertical_glyph_publication_shadow, BoundedVerticalHwp5TableCellSidecar,
    VerticalGlyphPublicationLeafInput, VerticalGlyphPublicationShadow,
    VerticalGlyphPublicationShadowRejectReason, VerticalRect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerticalGlyphLayerShadowRejectReason {
    WrongLineNode,
    UnsupportedLineSurface,
    TextSourceIdOverflow,
    Semantic(VerticalGlyphPublicationShadowRejectReason),
}

/// Audit one bounded line subtree without mutating layer ops or resources.
pub(crate) fn prepare_vertical_shaping_line_shadow(
    line: &LayerNode,
    first_text_source_id: u32,
    sidecar: &Arc<BoundedVerticalHwp5TableCellSidecar>,
) -> Result<VerticalGlyphPublicationShadow, VerticalGlyphLayerShadowRejectReason> {
    if line.source_node_id != Some(sidecar.line_node_id()) {
        return Err(VerticalGlyphLayerShadowRejectReason::WrongLineNode);
    }
    let LayerNodeKind::Group { children, .. } = &line.kind else {
        return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
    };
    let mut leaves = Vec::with_capacity(children.len());
    for (index, child) in children.iter().enumerate() {
        let Some(source_node_id) = child.source_node_id else {
            return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
        };
        let LayerNodeKind::Leaf { ops } = &child.kind else {
            return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
        };
        let [PaintOp::TextRun { bbox, run }] = ops.as_slice() else {
            return Err(VerticalGlyphLayerShadowRejectReason::UnsupportedLineSurface);
        };
        let text_source_id = first_text_source_id
            .checked_add(u32::try_from(index).unwrap_or(u32::MAX))
            .ok_or(VerticalGlyphLayerShadowRejectReason::TextSourceIdOverflow)?;
        leaves.push(VerticalGlyphPublicationLeafInput {
            source_node_id,
            text_source_id,
            text: &run.text,
            is_vertical: run.is_vertical,
            bbox: VerticalRect {
                x: bbox.x,
                y: bbox.y,
                width: bbox.width,
                height: bbox.height,
            },
        });
    }
    prepare_bounded_vertical_glyph_publication_shadow(sidecar, &leaves)
        .map_err(VerticalGlyphLayerShadowRejectReason::Semantic)
}
