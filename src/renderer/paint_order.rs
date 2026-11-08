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
    ordered.sort_by_key(|child| key(child));
    Children::Paint(ordered.into_iter())
}
