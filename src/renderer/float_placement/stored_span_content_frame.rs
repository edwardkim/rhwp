//! Continuous source line boxes describe occupancy, not a saved page boundary.

use crate::model::shape::TextWrap;
use crate::model::table::{Cell, Table, TablePageBreak};

use super::stored_span_opening::original_plain_cell;

pub(crate) fn stored_continuous_span_prefixes_hu(
    table: &Table,
    cell: &Cell,
    original_stored_layout: bool,
    session_edited: bool,
    table_reflowed: bool,
) -> Option<Vec<(i32, i32)>> {
    if !original_stored_layout
        || session_edited
        || table_reflowed
        || table.common.treat_as_char
        || table.common.text_wrap != TextWrap::TopAndBottom
        || table.page_break != TablePageBreak::RowBreak
        || cell.row_span <= 1
        || cell.height == 0
        || cell.height > i32::MAX as u32
        || usize::from(cell.row) + usize::from(cell.row_span) > usize::from(table.row_count)
        || !original_plain_cell(cell)
    {
        return None;
    }
    let padding = cell.effective_padding(&table.padding);
    if padding.top < 0 || padding.bottom < 0 {
        return None;
    }
    let mut previous = None;
    let mut ink_end = 0i64;
    let mut prefixes = Vec::new();
    for paragraph in &cell.paragraphs {
        if paragraph.line_segs.first()?.text_start != 0 {
            return None;
        }
        for line in &paragraph.line_segs {
            // Equal or rewound origins do not authenticate a single frame.
            // Saved restarts and paragraph-local origins retain their own cuts.
            if previous.is_some_and(|pos| line.vertical_pos <= pos) {
                return None;
            }
            previous = Some(line.vertical_pos);
            ink_end = ink_end.max(i64::from(line.vertical_pos) + i64::from(line.line_height));
            let padded = ink_end + i64::from(padding.top) + i64::from(padding.bottom);
            prefixes.push((i32::try_from(ink_end).ok()?, i32::try_from(padded).ok()?));
        }
    }
    let height = ink_end + i64::from(padding.top) + i64::from(padding.bottom);
    // A contradictory declared box cannot validate its cached source ladder.
    (height <= i64::from(cell.height)).then_some(prefixes)
}
