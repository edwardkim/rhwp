use serde::Serialize;

use crate::model::shape::TextWrap;
use crate::paint::layer_tree::{LayerNode, LayerNodeKind};
use crate::paint::paint_op::PaintOp;
use crate::renderer::render_tree::RenderLayerInfo;

/// Logical replay planes for PageLayerTree direct paint backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PaintReplayPlane {
    Background,
    BehindText,
    Flow,
    InFrontOfText,
}

impl PaintReplayPlane {
    pub const ORDERED: [Self; 4] = [
        Self::Background,
        Self::BehindText,
        Self::Flow,
        Self::InFrontOfText,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::BehindText => "behindText",
            Self::Flow => "flow",
            Self::InFrontOfText => "inFrontOfText",
        }
    }
}

pub fn paint_op_replay_plane(op: &PaintOp) -> PaintReplayPlane {
    paint_op_replay_plane_with_layer(op, None)
}

pub fn paint_op_replay_plane_with_layer(
    op: &PaintOp,
    layer: Option<RenderLayerInfo>,
) -> PaintReplayPlane {
    if matches!(op, PaintOp::PageBackground { .. }) {
        return PaintReplayPlane::Background;
    }
    if layer.and_then(|layer| layer.text_wrap).is_some() {
        return render_layer_replay_plane(layer);
    }

    match op {
        PaintOp::Image { image, .. } => match image.text_wrap {
            Some(TextWrap::BehindText) => PaintReplayPlane::BehindText,
            Some(TextWrap::InFrontOfText) => PaintReplayPlane::InFrontOfText,
            _ => PaintReplayPlane::Flow,
        },
        _ => PaintReplayPlane::Flow,
    }
}

pub fn render_layer_replay_plane(layer: Option<RenderLayerInfo>) -> PaintReplayPlane {
    match layer.and_then(|layer| layer.text_wrap) {
        Some(TextWrap::BehindText) => PaintReplayPlane::BehindText,
        Some(TextWrap::InFrontOfText) => PaintReplayPlane::InFrontOfText,
        _ => PaintReplayPlane::Flow,
    }
}

pub(crate) fn layer_node_has_replay_plane(node: &LayerNode, target: PaintReplayPlane) -> bool {
    layer_node_has_replay_plane_with_layer(node, target, None)
}

fn layer_node_has_replay_plane_with_layer(
    node: &LayerNode,
    target: PaintReplayPlane,
    inherited_layer: Option<RenderLayerInfo>,
) -> bool {
    let active_layer = node.layer.or(inherited_layer);
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children
            .iter()
            .any(|child| layer_node_has_replay_plane_with_layer(child, target, active_layer)),
        LayerNodeKind::ClipRect { child, .. } => {
            layer_node_has_replay_plane_with_layer(child, target, active_layer)
        }
        LayerNodeKind::Leaf { ops } => ops
            .iter()
            .any(|op| paint_op_replay_plane_with_layer(op, active_layer) == target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{CacheHint, GroupKind, LayerNode};
    use crate::renderer::render_tree::{
        BoundingBox, ImageNode, PageBackgroundNode, RectangleNode, RenderLayerInfo,
    };
    use crate::renderer::ShapeStyle;

    fn bbox() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 10.0, 10.0)
    }

    fn image_with_wrap(wrap: Option<TextWrap>) -> PaintOp {
        let mut image = ImageNode::new(1, Some(vec![1, 2, 3]));
        image.text_wrap = wrap;
        PaintOp::Image {
            bbox: bbox(),
            image,
            resolved: None,
        }
    }

    #[test]
    fn ordered_planes_match_hwp_z_order_contract() {
        assert_eq!(
            PaintReplayPlane::ORDERED.map(PaintReplayPlane::as_str),
            ["background", "behindText", "flow", "inFrontOfText"]
        );
    }

    #[test]
    fn page_background_replays_on_background_plane() {
        let op = PaintOp::PageBackground {
            bbox: bbox(),
            background: PageBackgroundNode {
                background_color: None,
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            },
        };

        assert_eq!(paint_op_replay_plane(&op), PaintReplayPlane::Background);
    }

    #[test]
    fn behind_text_image_replays_before_flow() {
        let op = image_with_wrap(Some(TextWrap::BehindText));

        assert_eq!(paint_op_replay_plane(&op), PaintReplayPlane::BehindText);
    }

    #[test]
    fn in_front_of_text_image_replays_after_flow() {
        let op = image_with_wrap(Some(TextWrap::InFrontOfText));

        assert_eq!(paint_op_replay_plane(&op), PaintReplayPlane::InFrontOfText);
    }

    #[test]
    fn non_layered_ops_replay_on_flow_plane() {
        let plain_image = image_with_wrap(None);
        let top_and_bottom_image = image_with_wrap(Some(TextWrap::TopAndBottom));
        let vector = PaintOp::Rectangle {
            bbox: bbox(),
            rect: RectangleNode::new(0.0, ShapeStyle::default(), None),
        };

        assert_eq!(paint_op_replay_plane(&plain_image), PaintReplayPlane::Flow);
        assert_eq!(
            paint_op_replay_plane(&top_and_bottom_image),
            PaintReplayPlane::Flow
        );
        assert_eq!(paint_op_replay_plane(&vector), PaintReplayPlane::Flow);
    }

    #[test]
    fn render_layer_metadata_overrides_non_image_paint_ops() {
        let vector = PaintOp::Rectangle {
            bbox: bbox(),
            rect: RectangleNode::new(0.0, ShapeStyle::default(), None),
        };
        let behind_layer = RenderLayerInfo::new(Some(TextWrap::BehindText), 1, 1);
        let front_layer = RenderLayerInfo::new(Some(TextWrap::InFrontOfText), 2, 2);

        assert_eq!(
            paint_op_replay_plane_with_layer(&vector, Some(behind_layer)),
            PaintReplayPlane::BehindText
        );
        assert_eq!(
            paint_op_replay_plane_with_layer(&vector, Some(front_layer)),
            PaintReplayPlane::InFrontOfText
        );
    }

    #[test]
    fn layer_node_replay_plane_scan_descends_groups() {
        let child = LayerNode::leaf(
            bbox(),
            None,
            vec![image_with_wrap(Some(TextWrap::InFrontOfText))],
        );
        let group = LayerNode::group(
            bbox(),
            None,
            vec![child],
            CacheHint::None,
            GroupKind::Generic,
        );

        assert!(layer_node_has_replay_plane(
            &group,
            PaintReplayPlane::InFrontOfText
        ));
        assert!(!layer_node_has_replay_plane(
            &group,
            PaintReplayPlane::BehindText
        ));
    }

    #[test]
    fn layer_node_replay_plane_scan_honors_inherited_layer_metadata() {
        let child = LayerNode::leaf(
            bbox(),
            None,
            vec![PaintOp::Rectangle {
                bbox: bbox(),
                rect: RectangleNode::new(0.0, ShapeStyle::default(), None),
            }],
        );
        let group = LayerNode::group(
            bbox(),
            None,
            vec![child],
            CacheHint::None,
            GroupKind::Generic,
        )
        .with_layer(Some(RenderLayerInfo::new(Some(TextWrap::BehindText), 1, 1)));

        assert!(layer_node_has_replay_plane(
            &group,
            PaintReplayPlane::BehindText
        ));
        assert!(!layer_node_has_replay_plane(&group, PaintReplayPlane::Flow));
    }
}
