//! 글자 겹침은 paint의 clip 안에 남는 글자 띠 사이에서 판정한다.

use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

pub(super) fn intersect(a: BoundingBox, b: BoundingBox) -> BoundingBox {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    BoundingBox::new(
        x,
        y,
        ((a.x + a.width).min(b.x + b.width) - x).max(0.0),
        ((a.y + a.height).min(b.y + b.height) - y).max(0.0),
    )
}

pub(super) fn for_node(node: &RenderNode, inherited: BoundingBox) -> BoundingBox {
    // paint::builder와 SVG의 기본 인쇄 clip 계약을 그대로 소비한다.
    // 일반 Body bbox 자체는 clip이 아니며, 명시한 clip_rect만 사용한다.
    let own = match &node.node_type {
        RenderNodeType::Body {
            clip_rect: Some(clip),
        } => Some(*clip),
        RenderNodeType::TableCell(cell) if cell.clip => Some(node.bbox),
        RenderNodeType::TextBox => Some(node.bbox),
        _ => None,
    };
    own.map_or(inherited, |own| intersect(inherited, own))
}
