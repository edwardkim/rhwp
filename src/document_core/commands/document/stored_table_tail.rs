//! 원본 줄로 닫힌 표 프레임의 점유 끝을 문단 재구성에도 전달한다.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    shape::{TextWrap, VertAlign, VertRelTo},
};

/// 글줄과 표 프레임이 같은 원본 좌표계를 소유할 때만 잉크 끝과 전진 끝을 반환한다.
/// 셀의 선언 최소 높이 대신 실제 저장 줄의 끝과 활성 안 여백으로 프레임을 입증한다.
pub(super) fn source_frame_tail(host: &Paragraph) -> Option<(i32, i32)> {
    let [Control::Table(table)] = host.controls.as_slice() else {
        return None;
    };
    let [anchor] = host.line_segs.as_slice() else {
        return None;
    };
    let [cell] = table.cells.as_slice() else {
        return None;
    };
    if !host.text.trim().is_empty()
        || host.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || anchor.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || anchor.vertical_pos < 0
        || anchor.line_height <= 0
        || table.common.treat_as_char
        || table.common.text_wrap != TextWrap::TopAndBottom
        || table.common.vert_rel_to != VertRelTo::Para
        || table.common.vert_align != VertAlign::Top
        || table.caption.is_some()
        || !table.raw_ctrl_data.is_empty()
        || table.row_count != 1
        || table.col_count != 1
        || cell.row_span != 1
        || cell.col_span != 1
        || cell.dirty_flag
        || !crate::renderer::cell_vpos_ladder_is_intact(&cell.paragraphs)
        || cell.paragraphs.iter().any(|paragraph| {
            paragraph.stored_text_partition_is_dirty()
                || paragraph.cell_format_vpos_dirty
                || !paragraph.controls.is_empty()
                || paragraph.line_segs.is_empty()
                || paragraph.line_segs.iter().any(|line| {
                    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        || line.vertical_pos < 0
                        || line.line_height <= 0
                })
        })
    {
        return None;
    }
    let frame_height = i32::try_from(table.common.height)
        .ok()
        .filter(|height| *height > 0)?;
    let lines: Vec<_> = cell
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.line_segs)
        .collect();
    // 여러 쪽에서 되감긴 줄이나 문단 로컬 원점은 하나의 완결 프레임이 아니다.
    if lines.first()?.vertical_pos != 0
        || lines
            .windows(2)
            .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
    {
        return None;
    }
    let ink_end = lines
        .iter()
        .map(|line| line.vertical_pos.checked_add(line.line_height))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max()?;
    let padding = cell.effective_padding(&table.padding);
    let occupied = ink_end
        .checked_add(i32::from(padding.top))?
        .checked_add(i32::from(padding.bottom))?;
    if occupied != frame_height {
        return None;
    }
    let top = anchor
        .vertical_pos
        .checked_add(table.common.vertical_offset as i32)?
        .checked_add(i32::from(table.outer_margin_top))?;
    if top < 0 {
        return None;
    }
    let painted_end = top.checked_add(frame_height)?;
    let advance_end = painted_end.checked_add(i32::from(table.outer_margin_bottom))?;
    Some((painted_end, advance_end))
}
