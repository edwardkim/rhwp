//! A shared saved restart owns an opening row band and a separate next-frame cut.

use std::collections::BTreeSet;

use crate::model::paragraph::LineSeg;
use crate::model::shape::TextWrap;
use crate::model::table::{Cell, Table, TablePageBreak};

pub(crate) struct StoredSpanOpeningFrame {
    pub height_hu: i32,
    pub restart_columns: BTreeSet<u16>,
}

pub(super) fn original_plain_cell(cell: &Cell) -> bool {
    !cell.dirty_flag
        && cell.text_direction == 0
        && !cell.paragraphs.is_empty()
        && cell.paragraphs.iter().all(|para| {
            !para.stored_text_partition_is_dirty()
                && !para.cell_format_vpos_dirty
                && para.controls.is_empty()
                && !para.line_segs.is_empty()
                && para.line_segs.iter().all(|line| {
                    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                        && line.vertical_pos >= 0
                        && line.line_height > 0
                        && line.line_spacing >= 0
                })
        })
}

fn restarts_after_first_line(cell: &Cell) -> bool {
    let Some(para) = cell.paragraphs.first() else {
        return false;
    };
    let [first, next, ..] = para.line_segs.as_slice() else {
        return false;
    };
    cell.row_span > 1
        && original_plain_cell(cell)
        && !para.text.trim().is_empty()
        && first.vertical_pos == 0
        && next.vertical_pos == 0
        && first.text_start == 0
        && next.text_start > first.text_start
        && first.column_start == next.column_start
        && first.segment_width > 0
        && first.segment_width == next.segment_width
}

pub(crate) fn stored_span_opening_frame(
    table: &Table,
    row: usize,
    original_stored_layout: bool,
    session_edited: bool,
    table_reflowed: bool,
) -> Option<StoredSpanOpeningFrame> {
    if !original_stored_layout
        || session_edited
        || table_reflowed
        || table.common.treat_as_char
        || table.common.text_wrap != TextWrap::TopAndBottom
        || table.page_break != TablePageBreak::RowBreak
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || row >= usize::from(table.row_count)
    {
        return None;
    }
    let cells: Vec<_> = table
        .cells
        .iter()
        .filter(|cell| usize::from(cell.row) == row)
        .collect();
    let restart_columns: BTreeSet<_> = cells
        .iter()
        .filter(|cell| restarts_after_first_line(cell))
        .map(|cell| cell.col)
        .collect();
    // Two independent source owners identify the common row boundary. A
    // paragraph-local zero or a single cell's loose frame is insufficient.
    if restart_columns.len() < 2
        || cells.iter().any(|cell| {
            !original_plain_cell(cell)
                || cell.row_span == 0
                || usize::from(cell.row) + usize::from(cell.row_span) > usize::from(table.row_count)
        })
    {
        return None;
    }
    let lower_bound: i64 = table
        .get_raw_row_heights()
        .iter()
        .filter(|&&height| height < 0x8000_0000)
        .map(|&height| i64::from(height))
        .sum();
    if lower_bound + i64::from(table.cell_spacing) * i64::from(table.row_count.saturating_sub(1))
        <= i64::from(table.common.height)
    {
        return None;
    }
    let mut required = 0i64;
    for cell in cells {
        let pad = cell.effective_padding(&table.padding);
        if pad.top < 0 || pad.bottom < 0 {
            return None;
        }
        let mut ink_end = 0i64;
        if restart_columns.contains(&cell.col) {
            ink_end = i64::from(cell.paragraphs[0].line_segs[0].line_height);
        } else {
            let mut previous: Option<&LineSeg> = None;
            'prefix: for para in &cell.paragraphs {
                for (index, line) in para.line_segs.iter().enumerate() {
                    if let Some(before) = previous {
                        if line.vertical_pos < before.vertical_pos {
                            // Only an in-paragraph reset identifies the end of
                            // this source prefix; writer-local paragraph origins
                            // cannot authenticate its physical occupancy.
                            if index == 0 || line.vertical_pos != 0 {
                                return None;
                            }
                            break 'prefix;
                        }
                    }
                    ink_end =
                        ink_end.max(i64::from(line.vertical_pos) + i64::from(line.line_height));
                    previous = Some(line);
                }
            }
        }
        required = required.max(ink_end + i64::from(pad.top) + i64::from(pad.bottom));
    }
    Some(StoredSpanOpeningFrame {
        height_hu: i32::try_from(required).ok()?,
        restart_columns,
    })
}
