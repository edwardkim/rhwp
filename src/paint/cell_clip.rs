//! 셀 본문의 논리 clip과 이미 확정한 배경의 물리 Print clip을 분리한다.

use super::layer_tree::{CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind};
use crate::renderer::render_tree::{RenderNode, TableCellNode};
use std::collections::HashSet;

pub(super) fn build(
    node: &RenderNode,
    cell: &TableCellNode,
    children: Vec<LayerNode>,
    cache_hint: CacheHint,
) -> LayerNode {
    let background_ids: HashSet<_> = node
        .children
        .iter()
        .filter(|child| child.table_background_paint)
        .map(|child| child.id)
        .collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::render_tree::{BoundingBox, ImageNode, RenderNodeType};

    fn fixture(fragment: bool) -> (RenderNode, TableCellNode) {
        let cell = TableCellNode {
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            border_fill_id: 0,
            text_direction: 0,
            clip: true,
            page_fragment: fragment,
            model_cell_index: None,
        };
        let mut node = RenderNode::new(
            1,
            RenderNodeType::TableCell(cell.clone()),
            BoundingBox::new(10.0, 20.0, 40.0, 50.0),
        );
        for id in [2, 4] {
            let mut background = RenderNode::new(
                id,
                RenderNodeType::Image(ImageNode::new(id as u16, None)),
                node.bbox,
            );
            background.table_background_paint = true;
            node.children.push(background);
        }
        (node, cell)
    }

    fn leaf(id: u32) -> LayerNode {
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 100.0, 100.0), Some(id), vec![])
    }

    fn physical(id: u32) -> LayerNode {
        let clip = BoundingBox::new(9.9, 19.9, 40.0, 50.0);
        LayerNode::clip_rect(clip, Some(id), clip, leaf(id), ClipKind::PrintBitmapFill)
    }

    fn leaves(node: &LayerNode, clips: &mut Vec<ClipKind>, out: &mut Vec<(u32, Vec<ClipKind>)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    leaves(child, clips, out);
                }
            }
            LayerNodeKind::ClipRect {
                clip_kind, child, ..
            } => {
                clips.push(*clip_kind);
                leaves(child, clips, out);
                clips.pop();
            }
            LayerNodeKind::Leaf { .. } => out.push((node.source_node_id.unwrap(), clips.clone())),
            _ => panic!("예상하지 않은 시험 노드"),
        }
    }

    #[test]
    fn direct_background_keeps_physical_clip_and_interleaved_content_keeps_cell_clip() {
        let (node, cell) = fixture(false);
        let children = vec![leaf(3), physical(2), leaf(5), physical(4), physical(6)];
        let result = build(&node, &cell, children, CacheHint::None);
        assert_eq!(result.source_node_id, Some(node.id));
        assert!(matches!(
            result.kind,
            LayerNodeKind::Group {
                group_kind: GroupKind::TableCell(_),
                ..
            }
        ));
        let mut actual = vec![];
        leaves(&result, &mut vec![], &mut actual);
        assert_eq!(
            actual,
            vec![
                (3, vec![ClipKind::TableCell]),
                (2, vec![ClipKind::PrintBitmapFill]),
                (5, vec![ClipKind::TableCell]),
                (4, vec![ClipKind::PrintBitmapFill]),
                (6, vec![ClipKind::TableCell, ClipKind::PrintBitmapFill]),
            ]
        );
    }

    #[test]
    fn page_cut_and_background_without_own_print_clip_keep_original_cell_clip() {
        for fragment in [false, true] {
            let (node, cell) = fixture(fragment);
            let children = if fragment {
                vec![physical(2), leaf(3)]
            } else {
                vec![leaf(2), leaf(3)]
            };
            let result = build(&node, &cell, children, CacheHint::None);
            assert!(matches!(
                result.kind,
                LayerNodeKind::ClipRect {
                    clip_kind: ClipKind::TableCell,
                    ..
                }
            ));
            let mut actual = vec![];
            leaves(&result, &mut vec![], &mut actual);
            assert!(actual
                .iter()
                .all(|(_, clips)| clips.first() == Some(&ClipKind::TableCell)));
            assert_eq!(
                actual.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
                vec![2, 3]
            );
        }
    }

    #[test]
    fn nested_background_keeps_outer_cell_clip() {
        let (node, cell) = fixture(false);
        let inner = build(&node, &cell, vec![physical(2), leaf(3)], CacheHint::None);
        let mut parent = node.clone();
        parent.children.clear();
        let result = build(&parent, &cell, vec![inner], CacheHint::None);
        let mut actual = vec![];
        leaves(&result, &mut vec![], &mut actual);
        assert_eq!(
            actual,
            vec![
                (2, vec![ClipKind::TableCell, ClipKind::PrintBitmapFill]),
                (3, vec![ClipKind::TableCell, ClipKind::TableCell]),
            ]
        );
    }
}
