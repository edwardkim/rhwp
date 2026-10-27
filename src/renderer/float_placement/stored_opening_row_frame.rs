//! 첫 개체 상자가 완결 행의 빈 띠에서 닫히는 원본 그리드.

use super::{is_para_topbottom_float, stored_complete_column_frame, stored_plain_cell_ink_height};
use crate::model::table::{Table, TablePageBreak};

pub(crate) struct StoredOpeningRowFrame {
    /// 첫 조각에서 내용과 여백을 인증한 행만 포함한다.
    pub row_heights: Vec<i32>,
    pub end_row: usize,
}

impl StoredOpeningRowFrame {
    /// 인증하지 않은 이후 행은 실제 내용으로 측정한 높이를 유지한다.
    pub fn apply_to(&self, rows: &mut [f64], dpi: f64) {
        for (target, source) in rows.iter_mut().zip(&self.row_heights) {
            *target = crate::renderer::hwpunit_to_px(*source, dpi);
        }
    }
}

/// 개체 높이는 첫 조각, 칸 높이는 보조 행 상자일 수 있다. 선택 행의 저장 줄과
/// 활성 여백이 실제 개체 안에 들어갈 때만 마지막 빈 띠를 그 경계에서 닫는다.
/// 호출자는 원본 저장 프로필과 해당 표의 재조판 여부를 별도로 확인한다.
pub(crate) fn stored_plain_opening_row_frame(table: &Table) -> Option<StoredOpeningRowFrame> {
    let count = usize::from(table.row_count);
    if count < 2
        || table.common.treat_as_char
        || table.common.size_protect
        || table.page_break != TablePageBreak::RowBreak
        || !is_para_topbottom_float(&table.common)
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || table.cells.iter().any(|cell| cell.row_span != 1)
        || !stored_complete_column_frame(table)
    {
        return None;
    }
    let mut rows = table
        .get_raw_row_heights()
        .into_iter()
        .map(|height| i32::try_from(height).ok().filter(|height| *height > 0))
        .collect::<Option<Vec<_>>>()?;
    if rows.len() != count {
        return None;
    }
    let mut prefix = i64::from(table.outer_cell_spacing_hu());
    for last in 0..count - 1 {
        prefix += i64::from(rows[last]);
        if last > 0 {
            prefix += i64::from(table.cell_spacing);
        }
        let surplus = prefix - i64::from(table.common.height);
        if surplus == 0 {
            // 원래 행 경계에서 정확히 닫히는 기존 계약은 보조 높이를 바꾸지 않는다.
            return None;
        }
        if surplus < 0 {
            continue;
        }
        let closing = i32::try_from(i64::from(rows[last]) - surplus).ok()?;
        if closing <= 0 {
            return None;
        }
        rows[last] = closing;
        for cell in table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) <= last)
        {
            let ink = stored_plain_cell_ink_height(cell)?;
            let padding = cell.effective_padding(&table.padding);
            if padding.top < 0
                || padding.bottom < 0
                || ink
                    .checked_add(i32::from(padding.top))?
                    .checked_add(i32::from(padding.bottom))?
                    > rows[usize::from(cell.row)]
            {
                return None;
            }
        }
        rows.truncate(last + 1);
        return Some(StoredOpeningRowFrame {
            row_heights: rows,
            end_row: last + 1,
        });
    }
    None
}
