//! 저장 글줄과 병합 셀의 물리 높이 제약으로 닫힌 표의 행.

use crate::model::table::{Cell, Table};

use super::{complete_declared_column_grid, stored_plain_cell_ink_height};

fn saved_cell_floor(table: &Table, cell: &Cell) -> Option<i32> {
    let declared = i32::try_from(cell.height).ok()?;
    let padding = cell.effective_padding(&table.padding);
    if padding.top < 0 || padding.bottom < 0 {
        return None;
    }
    let ink_end = stored_plain_cell_ink_height(cell)?;
    let content = ink_end
        .checked_add(i32::from(padding.top))?
        .checked_add(i32::from(padding.bottom))?;
    Some(declared.max(content))
}

/// 단일 행 셀의 높이 하한 합이 저장 개체를 정확히 닫고,
/// 같은 행들을 걸치는 셀의 글줄과 선언도 각각 그 안에 들어야 한다.
pub(super) fn closed_row_heights(table: &Table) -> Option<Vec<i32>> {
    let row_count = usize::from(table.row_count);
    if !table.common.treat_as_char
        || row_count < 2
        || row_count > table.cells.len()
        || table.cell_spacing < 0
    {
        return None;
    }
    let mut rows = vec![0_i32; row_count];
    for cell in table.cells.iter().filter(|cell| cell.row_span == 1) {
        let row = rows.get_mut(usize::from(cell.row))?;
        *row = (*row).max(saved_cell_floor(table, cell)?);
    }
    let physical_height = rows.iter().map(|&height| i64::from(height)).sum::<i64>()
        + table.inner_cell_spacing_hu(row_count)
        + i64::from(table.outer_cell_spacing_hu());
    if rows.contains(&0)
        || physical_height != i64::from(table.common.height)
        || !complete_declared_column_grid(table)
    {
        return None;
    }
    for cell in table.cells.iter().filter(|cell| cell.row_span > 1) {
        let start = usize::from(cell.row);
        let span = usize::from(cell.row_span);
        let supported_height = rows
            .get(start..start + span)?
            .iter()
            .map(|&height| i64::from(height))
            .sum::<i64>()
            + table.inner_cell_spacing_hu(span);
        if supported_height < i64::from(saved_cell_floor(table, cell)?) {
            return None;
        }
    }
    Some(rows)
}
