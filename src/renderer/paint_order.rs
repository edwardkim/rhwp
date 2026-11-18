//! 같은 단의 글줄 흐름과 문단 앵커 개체를 그리는 공통 순서.
//!
//! 문서/조판 tree의 자식 순서는 유지한다. paint 소비자만 같은 단에서 흐름을 먼저,
//! 앵커를 원본 z-order와 문서 주소 순으로 소비하며 소유 clip을 넘어 이동하지 않는다.

use super::render_tree::{doc_path_for_node, DocPath, RenderNode, RenderNodeType};
use crate::model::shape::TextWrap;

pub(crate) enum Children<'a> {
    Document(std::slice::Iter<'a, RenderNode>),
    Paint(std::vec::IntoIter<&'a RenderNode>),
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a RenderNode;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Document(children) => children.next(),
            Self::Paint(children) => children.next(),
        }
    }
}

fn key(node: &RenderNode) -> (u8, i32, DocPath) {
    let wrap = node.layer.and_then(|layer| layer.text_wrap);
    let plane = match wrap {
        Some(TextWrap::BehindText) => 0,
        Some(TextWrap::InFrontOfText) => 3,
        Some(_) => 2,
        None => match &node.node_type {
            // 명시적 layer 이전의 Image plane도 Native/SVG 계약대로 유지한다.
            RenderNodeType::Image(image) => match image.text_wrap {
                Some(TextWrap::BehindText) => 0,
                Some(TextWrap::InFrontOfText) => 3,
                _ => 1,
            },
            _ => 1,
        },
    };
    if plane == 1 {
        // 일반 흐름끼리는 안정 정렬로 기존 문서 순서를 유지한다.
        return (1, 0, Vec::new());
    }
    (
        plane,
        node.layer.map_or(0, |layer| layer.z_order),
        if node.layer.is_some() {
            doc_path_for_node(node).unwrap_or_default()
        } else {
            Vec::new()
        },
    )
}

pub(crate) fn children(node: &RenderNode) -> Children<'_> {
    if !matches!(node.node_type, RenderNodeType::Column(_))
        || !node.children.iter().any(|child| {
            child.layer.and_then(|layer| layer.text_wrap).is_some()
                || matches!(&child.node_type, RenderNodeType::Image(image)
                    if matches!(image.text_wrap, Some(TextWrap::BehindText | TextWrap::InFrontOfText)))
        })
    {
        return Children::Document(node.children.iter());
    }
    let mut ordered: Vec<_> = node.children.iter().collect();
    let keys = sibling_keys(&node.children, key);
    ordered.sort_by_key(|child| keys[&child.id].clone());
    Children::Paint(ordered.into_iter())
}

/// 단 내부 paint와 종이 기준 개체 조립이 같은 그림 소유 키를 소비한다.
pub(crate) fn sibling_keys<K: Clone>(
    nodes: &[RenderNode],
    key: impl Fn(&RenderNode) -> K,
) -> std::collections::HashMap<super::render_tree::NodeId, K> {
    // 그림 외곽은 owner와 하나의 paint 개체다. layer가 없는 외곽을 일반
    // 흐름으로 먼저 보내면 뒤의 그림이 자기 선을 덮는다. 실제 같은 부모의
    // owner 키를 공유하며 안정 정렬로 원래 image → border 순서를 보존한다.
    let images: std::collections::HashMap<_, _> = nodes
        .iter()
        .filter(|child| matches!(child.node_type, RenderNodeType::Image(_)))
        .map(|child| (child.id, child))
        .collect();
    nodes
        .iter()
        .map(|child| {
            let owner = match &child.node_type {
                RenderNodeType::Rectangle(rect) if rect.picture_border => rect
                    .image_frame_owner
                    .and_then(|id| images.get(&id).copied()),
                _ => None,
            };
            (child.id, key(owner.unwrap_or(child)))
        })
        .collect()
}
