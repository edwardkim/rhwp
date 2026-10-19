//! 원래 저장 글줄의 점유와 다음 쪽 앞 공간으로 물리 프레임 경계를 판정한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};

pub(crate) fn saved_positive_frame_reset(
    previous: Option<(i32, i32)>,
    first: &LineSeg,
    body_height: i32,
) -> bool {
    previous.is_some_and(|(previous_first, previous_end)| {
        let next_bottom = first.vertical_pos.saturating_add(first.line_height);
        first.vertical_pos > 0
            && first.line_height > 0
            && first.vertical_pos < previous_first
            && next_bottom <= body_height
            // 새 쪽의 앞 공간도 그 줄의 실제 점유다. 이전 축에 이어 넣을 때
            // 줄 높이만 더하면 표 뒤의 앞 간격을 버려 본문 밖까지 그릴 수 있다.
            && previous_end.saturating_add(next_bottom) > body_height
    })
}

pub(crate) fn owns_saved_object_line(para: &Paragraph) -> bool {
    let [line] = para.line_segs.as_slice() else {
        return false;
    };
    let mut owners = para
        .controls
        .iter()
        .filter(|control| !crate::renderer::control_is_body_flow_neutral_marker(control));
    let Some(control) = owners.next() else {
        return false;
    };
    if owners.next().is_some() {
        return false;
    }
    if para.stored_text_partition_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && c > '\u{001f}' && c != '\u{fffc}')
    {
        return false;
    }
    let common = match control {
        Control::Shape(shape)
            if shape
                .drawing()
                .is_some_and(|drawing| drawing.caption.is_none()) =>
        {
            shape.common()
        }
        Control::Picture(picture) if picture.caption.is_none() => &picture.common,
        Control::Table(table)
            if table.caption.is_none()
                && table.cells.iter().all(|cell| {
                    !cell.dirty_flag
                        && cell.paragraphs.iter().all(|paragraph| {
                            !paragraph.stored_text_partition_is_dirty()
                                && !paragraph.cell_format_vpos_dirty
                                && !crate::renderer::para_has_no_stored_line_segs(paragraph)
                        })
                }) =>
        {
            &table.common
        }
        _ => return false,
    };
    common.treat_as_char
        && common.height > 0
        && i64::from(line.line_height)
            == i64::from(common.height)
                + i64::from(common.margin.top)
                + i64::from(common.margin.bottom)
}
