//! 저장 첫 조각과 실제 본체의 종료식으로 중첩 표 뒤 호스트 줄을 인증한다.

use super::{hwpunit_to_px, is_para_topbottom_float};
use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph, table::Table};

/// 개체 높이가 첫 조각을 기록하고 빈 호스트 줄이 나머지 본체와 바깥 여백을
/// 정확히 닫으면, 그 줄은 자식 표의 시작이나 콘텐츠가 아니라 부모의 종료 공간이다.
pub(crate) fn stored_nested_terminal_host_line_height_hu(
    para: &Paragraph,
    child: &Table,
    body_height: f64,
    dpi: f64,
) -> Option<i32> {
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    if !dpi.is_finite()
        || dpi <= 0.0
        || !body_height.is_finite()
        || !para.text.trim().is_empty()
        || !matches!(para.controls.as_slice(), [Control::Table(_)])
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos <= 0
        || line.line_height <= 0
        || child.common.treat_as_char
        || !is_para_topbottom_float(&child.common)
        || !child.common.flow_with_text
        || child.caption.is_some()
        || child.common.height == 0
        || child.common.height > i32::MAX as u32
    {
        return None;
    }
    let opening = hwpunit_to_px(child.common.height as i32, dpi);
    if body_height <= opening {
        return None;
    }
    let margins = hwpunit_to_px(
        i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom),
        dpi,
    );
    let closing_origin = body_height - opening + margins;
    let saved_origin = hwpunit_to_px(line.vertical_pos, dpi);
    let epsilon = hwpunit_to_px(2, dpi)
        + f64::EPSILON * closing_origin.abs().max(saved_origin.abs()).max(1.0) * 4.0;
    ((closing_origin - saved_origin).abs() <= epsilon).then_some(line.line_height)
}
