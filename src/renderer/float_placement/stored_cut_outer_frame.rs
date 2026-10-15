//! 작은 원본 호스트 줄과 실제 표 프레임의 바깥 여백 소유를 구분한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::{TextWrap, VertAlign, VertRelTo};
use crate::model::table::{Table, TablePageBreak};

/// 직접 저장 HWPX의 작은 개체 앵커는 큰 표 상자의 바깥 여백을 대신 점유하지 않는다.
/// 전체 개체를 담은 저장 줄, 본문 텍스트, 편집된 호스트는 기존 앵커 계약을 유지한다.
pub(crate) fn stored_hwpx_cut_owns_outer_frame(
    original_hwpx: bool,
    host: &Paragraph,
    table: &Table,
) -> bool {
    let [line] = host.line_segs.as_slice() else {
        return false;
    };
    original_hwpx
        && !table.common.treat_as_char
        && table.common.flow_with_text
        && table.common.text_wrap == TextWrap::TopAndBottom
        && table.common.vert_rel_to == VertRelTo::Para
        && table.common.vert_align == VertAlign::Top
        && table.page_break == TablePageBreak::RowBreak
        && host.text.trim().is_empty()
        && !host.stored_text_partition_is_dirty()
        && !host.cell_format_vpos_dirty
        && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && line.vertical_pos >= 0
        && line.line_height > 0
        && u32::try_from(line.line_height).is_ok_and(|height| height < table.common.height)
        && table.common.height <= i32::MAX as u32
        && host
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Table(_)))
            .count()
            == 1
        && host.controls.iter().all(|control| {
            // 쪽번호 위치는 본문 줄의 개체가 아니라 구역의 인쇄 메타데이터다.
            matches!(
                control,
                Control::Table(_)
                    | Control::SectionDef(_)
                    | Control::ColumnDef(_)
                    | Control::PageNumberPos(_)
            )
        })
        && table.cells.iter().all(|cell| !cell.dirty_flag)
}
