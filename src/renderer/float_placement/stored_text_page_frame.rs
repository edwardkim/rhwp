//! 독립 셀이 함께 확인하는 저장 쪽 재시작 이후의 내용 프레임.

use crate::model::paragraph::{ColumnBreakType, LineSeg};
use crate::model::table::{Cell, Table, TablePageBreak};

#[derive(Clone, Copy)]
pub(crate) struct StoredTextPageFrame {
    pub(crate) end_paragraph: usize,
    pub(crate) content_height_hu: i32,
}

struct SourceFrame {
    first_paragraph: usize,
    end_paragraph: usize,
    ink_end: i32,
    lines: usize,
}

/// 문단 내부 재시작은 줄 소속을 추가로 해석해야 하므로 이 계약에 포함하지 않는다.
fn source_frames(cell: &Cell, table: &Table) -> Option<Vec<SourceFrame>> {
    if cell.dirty_flag
        || cell.text_direction != 0
        || cell.row_span == 0
        || cell.col_span == 0
        || u32::from(cell.row) + u32::from(cell.row_span) > u32::from(table.row_count)
        || u32::from(cell.col) + u32::from(cell.col_span) > u32::from(table.col_count)
        || cell.height == 0
        || cell.height > i32::MAX as u32
        || cell.paragraphs.is_empty()
    {
        return None;
    }
    let padding = cell.effective_padding(&table.padding);
    if padding.top < 0 || padding.bottom < 0 {
        return None;
    }
    let origin = cell.paragraphs.first()?.line_segs.first()?.vertical_pos;
    let mut frames = Vec::new();
    let mut frame = SourceFrame {
        first_paragraph: 0,
        end_paragraph: 0,
        ink_end: origin,
        lines: 0,
    };
    let mut previous_top = None;
    for (index, para) in cell.paragraphs.iter().enumerate() {
        if para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.column_type != ColumnBreakType::None
            || !para.controls.is_empty()
            || !para.title_marks.is_empty()
            || para.line_segs.first()?.text_start != 0
        {
            return None;
        }
        let first = para.line_segs.first()?;
        if first.vertical_pos == origin && previous_top.is_some_and(|top| top > origin) {
            frames.push(frame);
            frame = SourceFrame {
                first_paragraph: index,
                end_paragraph: index,
                ink_end: origin,
                lines: 0,
            };
        }
        for line in &para.line_segs {
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.vertical_pos < 0
                || line.vertical_pos < frame.ink_end
                || line.line_height <= 0
                || line.line_spacing < 0
                || line.segment_width <= 0
            {
                return None;
            }
            frame.ink_end = line.vertical_pos.checked_add(line.line_height)?;
            previous_top = Some(line.vertical_pos);
            frame.lines += 1;
        }
        frame.end_paragraph = index + 1;
    }
    frames.push(frame);
    Some(frames)
}

/// 같은 물리 걸침의 독립 셀이 앞 쪽의 종료와 다음 쪽의 재시작을 확인한다.
/// 첫 쪽의 Common 프레임 인증과 구분하며, 문단 로컬0 하나를 쪽으로 승격하지 않는다.
pub(crate) fn stored_text_page_frame(
    cell: &Cell,
    table: &Table,
    first_paragraph: usize,
) -> Option<StoredTextPageFrame> {
    if table.common.treat_as_char
        || table.page_break != TablePageBreak::RowBreak
        || cell.row_span <= 1
    {
        return None;
    }
    let prior = cell.paragraphs.get(first_paragraph.checked_sub(1)?)?;
    let origin = cell.paragraphs.first()?.line_segs.first()?.vertical_pos;
    if cell
        .paragraphs
        .get(first_paragraph)?
        .line_segs
        .first()?
        .vertical_pos
        != origin
        || prior.line_segs.last()?.vertical_pos <= origin
    {
        return None;
    }
    let frames = source_frames(cell, table)?;
    let index = frames
        .iter()
        .position(|frame| frame.first_paragraph == first_paragraph)?;
    let previous = frames.get(index.checked_sub(1)?)?;
    let frame = &frames[index];
    let padding = cell.effective_vertical_padding_hu(&table.padding);
    if previous.lines < 2 || i64::from(frame.ink_end) + i64::from(padding) > i64::from(cell.height)
    {
        return None;
    }
    let shared = table.cells.iter().any(|peer| {
        let disjoint = u32::from(peer.col) >= u32::from(cell.col) + u32::from(cell.col_span)
            || u32::from(cell.col) >= u32::from(peer.col) + u32::from(peer.col_span);
        if !disjoint
            || peer.row != cell.row
            || peer.row_span != cell.row_span
            || peer.height != cell.height
        {
            return false;
        }
        let Some(other) = source_frames(peer, table) else {
            return false;
        };
        let Some(before) = other.get(index - 1) else {
            return false;
        };
        before.lines > 1
            && other.get(index).is_some()
            && i64::from(before.ink_end)
                + i64::from(peer.effective_vertical_padding_hu(&table.padding))
                == i64::from(previous.ink_end) + i64::from(padding)
    });
    shared.then_some(StoredTextPageFrame {
        end_paragraph: frame.end_paragraph,
        content_height_hu: frame.ink_end,
    })
}
