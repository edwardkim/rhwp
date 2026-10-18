//! 원본 격자의 물리 프레임과 말미 편집 줄의 논리 상자를 구분한다.

use crate::model::{
    paragraph::{ColumnBreakType, LineSeg},
    table::{Cell, Table, TablePageBreak},
};

#[derive(Clone, Copy)]
pub(crate) struct StoredEditorTailFrame {
    pub(crate) content_end_hu: i32,
    pub(crate) empty_from: usize,
}

/// 다른 열의 원본 행 선언도 같은 끝을 소유하고 실제 본문 줄이 그 안에 들어오면,
/// PARA_TEXT 없는 말미 편집 줄이 원본 표 상자를 다시 키우지 않는다. 논리 문단/줄
/// 상자는 그대로 남는다. 중간 빈 줄과 공백 문자는 이 말미 범위로 재분류하지 않는다.
pub(crate) fn stored_editor_tail_frame(
    cell: &Cell,
    table: &Table,
) -> Option<StoredEditorTailFrame> {
    if cell.dirty_flag
        || cell.text_direction != 0
        || table.common.treat_as_char
        || table.page_break != TablePageBreak::RowBreak
        || table.cell_spacing != 0
        || cell.height == 0
        || cell.height > i32::MAX as u32
    {
        return None;
    }
    let is_editor_line = |p: &crate::model::paragraph::Paragraph| {
        p.text.is_empty() && !p.has_para_text && p.controls.is_empty()
    };
    let empty_from = cell.paragraphs.iter().rposition(|p| !is_editor_line(p))? + 1;
    if empty_from >= cell.paragraphs.len()
        || !cell.paragraphs[..empty_from]
            .iter()
            .any(|p| !p.text.is_empty())
    {
        return None;
    }
    let pad = cell.effective_padding(&table.padding);
    if pad.top < 0 || pad.bottom < 0 {
        return None;
    }
    let content_end = i64::from(cell.height) - i64::from(pad.top) - i64::from(pad.bottom);
    let first = usize::from(cell.row);
    let end = first + usize::from(cell.row_span);
    if first >= end || end > usize::from(table.row_count) {
        return None;
    }
    let grid = (first..end).try_fold(0_u64, |sum, row| {
        let height = table
            .cells
            .iter()
            .filter(|owner| {
                (owner.row != cell.row || owner.col != cell.col)
                    && usize::from(owner.row) == row
                    && owner.row_span == 1
                    && !owner.dirty_flag
                    && owner.height > 0
                    && owner.height <= i32::MAX as u32
            })
            .map(|owner| owner.height)
            .max()?;
        sum.checked_add(u64::from(height))
    })?;
    if grid != u64::from(cell.height) {
        return None;
    }
    let mut previous_end = 0_i64;
    let mut previous_slot = 0_i64;
    for (index, para) in cell.paragraphs.iter().enumerate() {
        if para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.column_type != ColumnBreakType::None
            || !para.controls.is_empty()
            || para.line_segs.is_empty()
        {
            return None;
        }
        for line in &para.line_segs {
            let top = i64::from(line.vertical_pos);
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.line_height <= 0
                || line.line_spacing < 0
                || top != previous_slot
            {
                return None;
            }
            previous_end = top.checked_add(i64::from(line.line_height))?;
            previous_slot = previous_end.checked_add(i64::from(line.line_spacing))?;
            if index < empty_from && previous_end > content_end {
                return None;
            }
        }
    }
    (previous_end > content_end).then_some(StoredEditorTailFrame {
        content_end_hu: i32::try_from(content_end).ok()?,
        empty_from,
    })
}
