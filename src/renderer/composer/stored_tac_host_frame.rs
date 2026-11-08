//! 독립된 첫 개체 줄과 같은 문단의 뒤 본문 줄이 소유한 원본 프레임.

use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph};

pub(crate) struct StoredTacHostFrame {
    pub control: usize,
    pub object_advance_hu: i32,
    pub paragraph_advance_hu: i32,
}

/// 원본 UTF-16 슬롯과 연속된 저장 줄이 각 줄의 점유를 함께 인증한다.
/// 같은 줄의 본문, 여러 개체, 편집·합성 줄과 쪽 되감김은 일반 구성 경로에 남긴다.
pub(crate) fn stored_tac_host_frame(para: &Paragraph) -> Option<StoredTacHostFrame> {
    if para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || para.line_segs.len() < 2
        || para.char_offsets.len() != para.text.chars().count()
        || !para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && c > '\u{001f}')
        || para.line_segs.iter().any(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.line_height <= 0
                || line.line_spacing < 0
        })
        || para.line_segs.windows(2).any(|pair| {
            i64::from(pair[0].vertical_pos)
                + i64::from(pair[0].line_height)
                + i64::from(pair[0].line_spacing)
                != i64::from(pair[1].vertical_pos)
        })
        || para
            .source_line_seg_vertical_pos
            .as_ref()
            .is_some_and(|source| {
                source.len() != para.line_segs.len()
                    || source
                        .iter()
                        .zip(&para.line_segs)
                        .any(|(v, line)| *v != line.vertical_pos)
            })
    {
        return None;
    }
    let first = para.line_segs.first()?;
    let text_start = para.line_seg_text_start(1);
    if first.vertical_pos < 0 || para.char_offsets.iter().any(|offset| *offset < text_start) {
        return None;
    }
    let mut owner = None;
    for (index, control) in para.controls.iter().enumerate() {
        match control {
            Control::Table(table)
                if owner.is_none()
                    && table.common.treat_as_char
                    && table.common.flow_with_text
                    && !table.common.allow_overlap
                    && table.caption.is_none()
                    && table.common.height > 0
                    && table.common.height <= i32::MAX as u32
                    && !table.cells.iter().any(|cell| cell.dirty_flag)
                    && super::control_slot_line_index(para, index) == Some(0)
                    && i64::from(first.line_height)
                        == i64::from(table.common.height)
                            + i64::from(table.outer_margin_top)
                            + i64::from(table.outer_margin_bottom)
                    && first.text_height == first.line_height =>
            {
                owner = Some(index)
            }
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::Header(_)
            | Control::Footer(_)
            | Control::PageNumberPos(_)
            | Control::PageHide(_) => {}
            _ => return None,
        }
    }
    let last = para.line_segs.last()?;
    Some(StoredTacHostFrame {
        control: owner?,
        object_advance_hu: first.line_height.checked_add(first.line_spacing)?,
        paragraph_advance_hu: last
            .vertical_pos
            .checked_sub(first.vertical_pos)?
            .checked_add(last.line_height)?
            .checked_add(last.line_spacing)?,
    })
}
