//! 원본 완결 행의 실제 끝점과 저장 셀 폭으로 물리 구획을 함께 해석한다.

use super::{Table, MAX_TABLE_GRID_CELLS};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RowCellFrame {
    pub cell_index: usize,
    pub start_hu: i64,
    pub width_hu: i64,
}

impl Table {
    /// 가상 열 내부 보간을 실제 경계로 승격하지 않는다. 표 외곽을 닫는
    /// 원본 행의 실제 셀 끝점을 재사용하고 그 사이에서 저장 폭을 소비한다.
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
        let mut coverage = 0usize;
        for (index, cell) in self.cells.iter().enumerate() {
            let start = usize::from(cell.row);
            let end = start + usize::from(cell.row_span);
            coverage = coverage.saturating_add(usize::from(cell.row_span));
            if cell.row_span == 0
                || cell.col_span == 0
                || end > rows
                || cell.width == 0
                || usize::from(cell.col) + usize::from(cell.col_span) > cols
                || coverage > MAX_TABLE_GRID_CELLS
            {
                return Ok(result);
            }
            for row in &mut covering[start..end] {
                storage.push(row, index)?;
            }
        }
        let spacing = i64::from(self.cell_spacing);
        let target = i64::from(self.common.width);
        let mut closed = storage.filled(rows, false)?;
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
                // 병합 셀은 열 코어뿐 아니라 그 사이의 원본 간격도 점유한다.
                // 이 간격을 마지막 셀에 다시 몰면 같은 걸침 셀의 시작이 행마다 달라진다.
                let width_hu =
                    i64::from(cell.width) + self.inner_cell_spacing_hu(usize::from(cell.col_span));
                frames.push(RowCellFrame {
                    cell_index: index,
                    start_hu: cursor,
                    width_hu,
                });
                cursor += width_hu + spacing;
                col = end_col;
            }
            if col != cols || frames.len() != indices.len() || cursor > target {
                continue;
            }
            if !frames.is_empty() {
                closed[row] = cursor == target;
                result[row] = Some(frames);
            }
        }
        let mut boundaries = storage.filled(cols + 1, None)?;
        let mut conflicting = storage.filled(cols + 1, false)?;
        let mut observe = |col: usize, position: i64| {
            if conflicting[col] {
                return;
            }
            match boundaries[col] {
                Some(old) if old != position => {
                    conflicting[col] = true;
                    boundaries[col] = None;
                }
                None => boundaries[col] = Some(position),
                _ => {}
            }
        };
        for (row, frames) in result.iter().enumerate() {
            if !closed[row] {
                continue;
            }
            for frame in frames.iter().flatten() {
                let cell = &self.cells[frame.cell_index];
                observe(usize::from(cell.col), frame.start_hu);
                observe(
                    usize::from(cell.col) + usize::from(cell.col_span),
                    frame.start_hu + frame.width_hu + spacing,
                );
            }
        }
        boundaries[0] = Some(spacing);
        boundaries[cols] = Some(target);
        for (row, owned) in result.iter_mut().enumerate() {
            if closed[row] {
                continue;
            }
            let Some(frames) = owned else {
                continue;
            };
            let mut cursor = spacing;
            let mut valid = true;
            for frame in frames {
                let cell = &self.cells[frame.cell_index];
                let end = usize::from(cell.col) + usize::from(cell.col_span);
                let width = boundaries[end].map_or(frame.width_hu, |end| end - cursor - spacing);
                if width <= 0 {
                    valid = false;
                    break;
                }
                frame.start_hu = cursor;
                frame.width_hu = width;
                cursor += width + spacing;
            }
            if !valid || cursor != target {
                *owned = None;
            }
        }
        Ok(result)
    }
}
