//! 분할 셀의 TAC 그림 보완 경로가 실제 문단 원점과 저장 줄의 점유 간격을 소비한다.

use super::{Paragraph, RenderNode, RenderNodeType};

pub(super) fn rendered_picture_paragraph_origin(
    cell_node: &RenderNode,
    para: &Paragraph,
    section_index: usize,
    para_index: usize,
) -> Option<f64> {
    if crate::renderer::para_has_no_stored_line_segs(para) {
        return None;
    }
    // 빈 안내 글줄의 작은 상자는 TAC 그림이 차지하는 저장 줄 높이가 아니다.
    // 첫 저장 줄에서 문단 앞 간격을 반영한 시작점만 얻는다. 이후 그림의 줄 간격은
    // 기존 LINE_SEG 점유 슬롯을 유지한다. 첫 줄이 없는 이어받기 조각은 원점을 추측하지 않는다.
    cell_node.children.iter().find_map(|node| {
        let RenderNodeType::TextLine(line) = &node.node_type else {
            return None;
        };
        (line.section_index == Some(section_index)
            && line.para_index == Some(para_index)
            && line.line_index == Some(0))
        .then_some(node.bbox.y)
    })
}
