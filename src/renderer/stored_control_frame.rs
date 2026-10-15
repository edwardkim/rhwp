//! 원본 TAC 글줄 뒤의 물리 프레임 경계와 앞 개체 소유자를 함께 판별한다.

use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{composer::stored_first_tac_line, layout::control_line_seg_index};

#[derive(Clone, Copy)]
pub(crate) struct StoredControlFrame {
    pub before_control: usize,
    pub line: usize,
}

/// 첫 TAC 상자는 현재 본문 안에 닫히지만 다음 저장 줄은 들어가지 않는 경계다.
/// 줄 소속과 원본 좌표가 입증되지 않은 생성기 원점은 쪽나눔으로 해석하지 않는다.
pub(crate) fn following_first_tac_frame(
    para: &Paragraph,
    body_height_hu: i32,
) -> Option<StoredControlFrame> {
    if para.cell_format_vpos_dirty || body_height_hu <= 0 {
        return None;
    }
    let first = stored_first_tac_line(para)?;
    let next = para.line_segs.get(1)?;
    let source = match &para.source_line_seg_vertical_pos {
        Some(source) if source.len() == para.line_segs.len() => Some(source),
        Some(_) => return None,
        None => None,
    };
    let first_vpos = source.map_or(first.vertical_pos, |positions| positions[0]);
    let next_vpos = source.map_or(next.vertical_pos, |positions| positions[1]);
    if first_vpos < 0
        || next_vpos != 0
        || next.vertical_pos != 0
        || first.line_height <= 0
        || first.line_spacing < 0
        || next.line_height <= 0
        || next.text_start <= first.text_start
        || first.column_start != next.column_start
        || first.segment_width <= 0
        || first.segment_width != next.segment_width
    {
        return None;
    }
    let painted_end = i64::from(first_vpos) + i64::from(first.line_height);
    let next_end = painted_end + i64::from(first.line_spacing) + i64::from(next.line_height);
    if painted_end > i64::from(body_height_hu) || next_end <= i64::from(body_height_hu) {
        return None;
    }
    let before_control = para.controls.iter().enumerate().find_map(|(ci, control)| {
        let Control::Table(table) = control else {
            return None;
        };
        (table.common.treat_as_char
            && table.caption.is_none()
            && control_line_seg_index(para, ci) == Some(0)
            && i64::from(table.common.height)
                + i64::from(table.outer_margin_top)
                + i64::from(table.outer_margin_bottom)
                == i64::from(first.line_height))
        .then_some(ci)
    })?;
    Some(StoredControlFrame {
        before_control,
        line: 1,
    })
}
