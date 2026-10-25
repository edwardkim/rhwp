//! 원본 셀의 물리 구획과 표 외곽에서 닫히는 마지막 셀을 함께 해석한다.

use super::{Table, MAX_TABLE_GRID_CELLS};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RowCellFrame {
    pub cell_index: usize,
    pub start_hu: i64,
    pub width_hu: i64,
}

impl Table {
    /// 가상 열 번호를 실제 폭으로 취급하지 않는다. 완전한 행의 앞 셀은
    /// 저장 폭을 소유하고 마지막 셀이 표 외곽까지 남은 공간을 소유한다.
    pub(crate) fn closing_row_frames(&self) -> Vec<Option<Vec<RowCellFrame>>> {
        match self.closing_row_frames_with_storage(&crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(crate) fn closing_row_frames_with_storage<S: crate::storage::Storage>(
        &self,
        storage: &S,
    ) -> Result<Vec<Option<Vec<RowCellFrame>>>, S::Error> {
        let rows = usize::from(self.row_count);
        let cols = usize::from(self.col_count);
        let mut result = storage.vec(rows)?;
        result.resize_with(rows, || None);
        if rows == 0
            || cols == 0
            || rows.saturating_mul(cols) > MAX_TABLE_GRID_CELLS
            || self.common.width == 0
            || self.cell_spacing < 0
        {
            return Ok(result);
        }
        let mut covering = storage.vec(rows)?;
        covering.resize_with(rows, Vec::new);
        for (index, cell) in self.cells.iter().enumerate() {
            let start = usize::from(cell.row);
            let end = start + usize::from(cell.row_span);
            if cell.row_span == 0 || cell.col_span == 0 || end > rows || cell.width == 0 {
                continue;
            }
            for row in &mut covering[start..end] {
                storage.push(row, index)?;
            }
        }
        let spacing = i64::from(self.cell_spacing);
        let target = i64::from(self.common.width);
        for (row, indices) in covering.iter_mut().enumerate() {
            storage.bytes(std::mem::size_of_val(indices.as_slice()))?;
            indices.sort_by_key(|index| self.cells[*index].col);
            let mut cursor = spacing;
            let mut col = 0;
            let mut frames = storage.vec(indices.len())?;
            for &index in indices.iter() {
                let cell = &self.cells[index];
                let end_col = usize::from(cell.col) + usize::from(cell.col_span);
                if usize::from(cell.col) != col || end_col > cols {
                    break;
                }
                frames.push(RowCellFrame {
                    cell_index: index,
                    start_hu: cursor,
                    width_hu: i64::from(cell.width),
                });
                cursor += i64::from(cell.width) + spacing;
                col = end_col;
            }
            if col != cols || frames.len() != indices.len() || cursor > target {
                continue;
            }
            if let Some(last) = frames.last_mut() {
                last.width_hu += target - cursor;
                result[row] = Some(frames);
            }
        }
        Ok(result)
    }
}
