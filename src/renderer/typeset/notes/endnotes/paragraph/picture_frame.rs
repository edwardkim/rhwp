//! 앞 줄 끝·그림 프레임·뒤 줄 원점으로 닫히는 미주 그림의 공통 예약.

use crate::renderer::float_placement::{ParagraphFloatFlow, ParagraphFloatPlacement};
use crate::renderer::typeset::{Paragraph, ResolvedStyleSet};

/// 저장 절대 vpos를 단의 절대 원점으로 사용하지 않는다. 호스트를 원점으로
/// 옮긴 상대 프레임을 먼저 검증하고, 단 이동이 끝난 뒤 실제 흐름 원점에 붙인다.
pub(super) fn relative_frame(
    paragraphs: &[Paragraph],
    index: usize,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let previous = paragraphs.get(index.checked_sub(1)?)?;
    let host = paragraphs.get(index)?;
    let successor = paragraphs.get(index.checked_add(1)?)?;
    let successor_spacing = styles
        .para_styles
        .get(successor.para_shape_id as usize)?
        .spacing_before;
    crate::renderer::float_placement::stored_picture_empty_host_placement(
        previous,
        host,
        successor,
        successor_spacing,
        host.line_segs.first()?.vertical_pos,
        0.0,
        dpi,
    )
}

pub(super) fn at_origin(
    mut frame: ParagraphFloatPlacement,
    origin: f64,
) -> ParagraphFloatPlacement {
    frame.anchor_y += origin;
    frame.stored_host_origin = frame.stored_host_origin.map(|y| y + origin);
    frame.stored_successor_line_origin = frame.stored_successor_line_origin.map(|y| y + origin);
    frame.table_top += origin;
    frame.occupied_bottom += origin;
    if let ParagraphFloatFlow::StoredPicture { next_flow_y } = &mut frame.flow {
        *next_flow_y += origin;
    }
    frame
}
