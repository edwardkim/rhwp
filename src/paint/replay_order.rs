use serde::Serialize;

use crate::model::shape::TextWrap;
use crate::paint::layer_tree::{LayerNode, LayerNodeKind};
use crate::paint::paint_op::PaintOp;
use crate::renderer::render_tree::{BoundingBox, RenderLayerInfo};

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

    let plane = match op {
        PaintOp::Image { image, .. } => match image.text_wrap {
            Some(TextWrap::BehindText) => PaintReplayPlane::BehindText,
            Some(TextWrap::InFrontOfText) => PaintReplayPlane::InFrontOfText,
            _ => PaintReplayPlane::Flow,
        },
        _ => PaintReplayPlane::Flow,
    };
    cap_master_page_plane(plane, layer)
}

pub fn render_layer_replay_plane(layer: Option<RenderLayerInfo>) -> PaintReplayPlane {
    let plane = match layer.and_then(|layer| layer.text_wrap) {
        Some(TextWrap::BehindText) => PaintReplayPlane::BehindText,
        Some(TextWrap::InFrontOfText) => PaintReplayPlane::InFrontOfText,
        _ => PaintReplayPlane::Flow,
    };
    cap_master_page_plane(plane, layer)
}

/// 바탕쪽 유래 op 의 replay plane 상한 (#2318).
///
/// 한컴 의미론: 바탕쪽 개체의 text_wrap 은 바탕쪽 **내부** 개체 간 순서에만
/// 적용되고, 바탕쪽 전체는 항상 본문 뒤에 깔린다. SVG 의 `node_z_plane` 계약
/// (페이지 배경 → 바탕쪽 → BehindText → Flow → InFrontOfText, #1167)과 동일
/// 의미를 plane 재생 backend(web_canvas/skia/canvaskit)에 적용한다.
/// BehindText plane 내에서 바탕쪽 그룹은 트리 순서상 본문 개체보다 먼저
/// 재생되므로 더 깊게 깔린다.
fn cap_master_page_plane(
    plane: PaintReplayPlane,
    layer: Option<RenderLayerInfo>,
) -> PaintReplayPlane {
    if plane != PaintReplayPlane::Background && layer.is_some_and(|layer| layer.master_page) {
        PaintReplayPlane::BehindText
    } else {
        plane
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

/// [#5763] flow 그림을 canvas 아래 별도 평면으로 분리해도 되는지 paint 순서대로 누적 판정한다.
///
/// `flow-static` 분리(#516·#3315)는 flow plane 의 `Image`/`RawSvg` 만 canvas **아래**
/// 평면(studio 의 DOM `<img>` layer, 또는 flow-static canvas)으로 내리고, 나머지 본문은
/// `flow-dynamic` 으로 그 **위에** 그린다. 그림 밑에 깔린 불투명 채우기(그림을 담은 표 칸의
/// 흰 배경 등)는 `flow-dynamic` 에 남으므로, 그대로 분리하면 그 채우기가 그림을 덮어 그림이
/// 통째로 사라진다.
///
/// 그래서 **paint 순서상 앞선 불투명 flow 채우기와 겹치는 flow 그림**이 하나라도 있으면 그
/// 페이지는 분리 대상이 아니다. 호출부는 layer tree 를 paint 순서대로 훑으며 `observe` 를
/// 먹이고 `occluded()` 로 판정한다.
#[derive(Debug, Default)]
pub struct FlowStaticOcclusion {
    opaque_fills: Vec<BoundingBox>,
    occluded: bool,
}

impl FlowStaticOcclusion {
    /// paint 순서대로 op 하나를 관찰한다.
    pub fn observe(&mut self, plane: PaintReplayPlane, op: &PaintOp) {
        if plane != PaintReplayPlane::Flow {
            return;
        }
        match op {
            PaintOp::Image { bbox, .. } | PaintOp::RawSvg { bbox, .. } => {
                if self.opaque_fills.iter().any(|prior| prior.intersects(bbox)) {
                    self.occluded = true;
                }
            }
            _ => {
                if let Some(bbox) = opaque_flow_fill_bbox(op) {
                    self.opaque_fills.push(bbox);
                }
            }
        }
    }

    /// 분리하면 flow 그림이 가려지는 페이지인가.
    pub fn occluded(&self) -> bool {
        self.occluded
    }
}

/// 뒤에 오는 그림을 덮을 수 있는 불투명 채우기면 그 bbox 를 준다.
///
/// 채우기 색·패턴·그라데이션이 없으면 덮지 않는다(테두리만 있는 도형 포함). `opacity` 가 1
/// 미만이면 밑이 비치므로 덮는 것으로 보지 않는다.
fn opaque_flow_fill_bbox(op: &PaintOp) -> Option<BoundingBox> {
    let (bbox, style, has_gradient) = match op {
        PaintOp::Rectangle { bbox, rect } => (bbox, &rect.style, rect.gradient.is_some()),
        PaintOp::Ellipse { bbox, ellipse } => (bbox, &ellipse.style, ellipse.gradient.is_some()),
        PaintOp::Path { bbox, path } => (bbox, &path.style, path.gradient.is_some()),
        _ => return None,
    };
    if style.opacity < 1.0 {
        return None;
    }
    if style.fill_color.is_some() || style.pattern.is_some() || has_gradient {
        Some(*bbox)
    } else {
        None
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
        PaintOp::image(bbox(), image, None)
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
        let op = PaintOp::page_background(
            bbox(),
            PageBackgroundNode {
                background_color: None,
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            },
        );

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
        let vector =
            PaintOp::rectangle(bbox(), RectangleNode::new(0.0, ShapeStyle::default(), None));

        assert_eq!(paint_op_replay_plane(&plain_image), PaintReplayPlane::Flow);
        assert_eq!(
            paint_op_replay_plane(&top_and_bottom_image),
            PaintReplayPlane::Flow
        );
        assert_eq!(paint_op_replay_plane(&vector), PaintReplayPlane::Flow);
    }

    #[test]
    fn render_layer_metadata_overrides_non_image_paint_ops() {
        let vector =
            PaintOp::rectangle(bbox(), RectangleNode::new(0.0, ShapeStyle::default(), None));
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
            vec![PaintOp::rectangle(
                bbox(),
                RectangleNode::new(0.0, ShapeStyle::default(), None),
            )],
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
