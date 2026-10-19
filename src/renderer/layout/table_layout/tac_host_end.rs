//! 실제 줄과 표가 소비한 물리 끝에서 TAC 호스트의 종료 간격을 한 번 확정한다.

use crate::renderer::render_tree::{RenderNode, RenderNodeType};

pub(super) fn close(children: &[RenderNode], current_end: f64, trailing_spacing: f64) -> f64 {
    // 자식은 이 호스트를 배치한 범위뿐이다. 글자가 없는 TextLine도 물리 줄이다.
    // 줄의 실제 높이에는 TAC 바깥여백이 포함되며 표가 더 크면 그 끝도 보존한다.
    let occupied_end = children
        .iter()
        .filter(|node| {
            matches!(
                node.node_type,
                RenderNodeType::TextLine(_) | RenderNodeType::Table(_)
            )
        })
        .map(|node| node.bbox.y + node.bbox.height)
        .reduce(f64::max);
    occupied_end.map_or(current_end + trailing_spacing, |end| {
        current_end.max(end + trailing_spacing)
    })
}
