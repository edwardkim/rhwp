//! 실제 줄과 표가 소비한 물리 끝에서 TAC 호스트의 종료 간격을 한 번 확정한다.

use crate::{
    model::control::Control,
    renderer::{
        hwpunit_to_px,
        render_tree::{RenderNode, RenderNodeType},
    },
};

pub(super) fn close(
    children: &[RenderNode],
    current_end: f64,
    trailing_spacing: f64,
    controls: &[Control],
    dpi: f64,
) -> f64 {
    // 자식은 이 호스트를 배치한 범위뿐이다. 글자가 없는 TextLine도 물리 줄이다.
    // 낡은 저장 줄보다 표가 자랐으면 줄 상자에는 실제 바깥여백이 들어 있지 않다.
    // 실제로 배치한 표의 소유 control에서 아래 바깥여백을 읽어 물리 끝을 복원한다.
    let occupied_end = children
        .iter()
        .filter(|node| {
            matches!(
                node.node_type,
                RenderNodeType::TextLine(_) | RenderNodeType::Table(_)
            )
        })
        .map(|node| {
            let bottom_margin = match &node.node_type {
                RenderNodeType::Table(table) => table
                    .control_index
                    .and_then(|index| controls.get(index))
                    .and_then(|control| match control {
                        Control::Table(source) if source.common.treat_as_char => {
                            Some(hwpunit_to_px(i32::from(source.outer_margin_bottom), dpi))
                        }
                        _ => None,
                    })
                    .unwrap_or(0.0),
                _ => 0.0,
            };
            node.bbox.y + node.bbox.height + bottom_margin
        })
        .reduce(f64::max);
    occupied_end.map_or(current_end + trailing_spacing, |end| {
        current_end.max(end + trailing_spacing)
    })
}
