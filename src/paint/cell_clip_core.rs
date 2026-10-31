// 셀 배경 소유와 물리·논리 clip을 공통으로 결합한다.
use super::layer_tree::{CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind};
use super::render_tree::{NodeId, RenderNode, TableCellNode};
use std::collections::HashSet;

pub(super) fn build(
    node: &RenderNode,
    cell: &TableCellNode,
    children: Vec<LayerNode>,
    cache_hint: CacheHint,
    background_ids: &HashSet<NodeId>,
) -> LayerNode {
    let owns_physical_clip = |child: &LayerNode| {
        child
            .source_node_id
            .is_some_and(|id| background_ids.contains(&id))
            && matches!(
                &child.kind,
                LayerNodeKind::ClipRect {
                    clip_kind: ClipKind::PrintBitmapFill,
                    ..
                }
            )
    };
    let clip_contents = |children, group_kind, source| {
        LayerNode::clip_rect(
            node.bbox,
            Some(node.id),
            node.bbox,
            LayerNode::group(node.bbox, source, children, cache_hint, group_kind),
            ClipKind::TableCell,
        )
        .with_layer(node.layer)
    };
    // 분할 셀의 경계는 본문 상자가 아니라 실제 쪽 컷이다. 배경에도 유지한다.
    // Screen이나 자체 물리 clip이 없는 채움도 기존 논리 경계를 그대로 소비한다.
    if cell.page_fragment || !children.iter().any(owns_physical_clip) {
        return clip_contents(children, GroupKind::TableCell(cell.clone()), Some(node.id));
    }

    let mut output = Vec::new();
    let mut contents = Vec::new();
    for child in children {
        if owns_physical_clip(&child) {
            if !contents.is_empty() {
                output.push(clip_contents(
                    std::mem::take(&mut contents),
                    GroupKind::Generic,
                    None,
                ));
            }
            output.push(child);
        } else {
            contents.push(child);
        }
    }
    if !contents.is_empty() {
        output.push(clip_contents(contents, GroupKind::Generic, None));
    }
    // 자식 순서를 바꾸지 않는다. 중첩 표와 본문은 원래 셀 clip 안에 남으며,
    // 직접 배경만 자신의 장치 프레임을 소비한다. 상위 셀·본문 clip은 유지된다.
    LayerNode::group(
        node.bbox,
        Some(node.id),
        output,
        cache_hint,
        GroupKind::TableCell(cell.clone()),
    )
    .with_layer(node.layer)
}
