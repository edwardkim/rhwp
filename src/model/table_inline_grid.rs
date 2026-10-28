//! 완결된 TAC 표의 행 선언에서 흐름과 실제 그리드가 공유하는 폭을 구한다.

use super::table::Cell;
use super::Table;

impl Table {
    /// 선언 폭과 별개로 셀의 내용·안 여백이 소유하는 최소 물리 폭.
    ///
    /// 한컴 2020의 동일 입력 Print·저장 대조에서 내용 폭 하한은 1mm(283 HU)다.
    /// 글자 크기·바깥 여백·TAC·noAdjust와 무관하며 병합 셀에도 한 번 적용된다.
    /// 폭 0은 아직 풀리지 않은 열 제약이므로 기존 해결 경로에 남긴다.
    /// 원시 셀 폭과 직렬화·유효성 판정은 바꾸지 않는다.
    pub(crate) fn physical_cell_width_hu(&self, cell: &Cell) -> u32 {
        if cell.width == 0 {
            return 0;
        }
        const MIN_CONTENT_WIDTH_HU: i32 = 283;
        let padding = cell.effective_padding(&self.padding);
        let minimum = (i32::from(padding.left) + i32::from(padding.right) + MIN_CONTENT_WIDTH_HU)
            .max(0) as u32;
        cell.width.max(minimum)
    }

    /// 모든 행이 겹침·빈 열 없이 닫히고 같은 물리 폭을 소유할 때의 표 폭.
    ///
    /// 개체의 `sz`만 바꾼 독립 한컴 Print 대조군에서도 셀 그리드 폭은 유지된다.
    /// 반대로 행마다 구획·합이 다른 표는 열별 최대값을 더할 수 없으므로 기존
    /// 개체 폭 계약을 유지한다. 직렬화 모델과 원시 유효성 검사는 바꾸지 않는다.
    pub(crate) fn closed_inline_grid_width_hu(&self) -> Option<u32> {
        if !self.common.treat_as_char
            || self.row_count == 0
            || self.col_count == 0
            || self.cell_spacing < 0
            || self.cells.iter().any(|cell| {
                cell.row >= self.row_count
                    || cell.row_span != 1
                    || cell.col_span == 0
                    || cell.width == 0
            })
        {
            return None;
        }
        let mut rows = vec![Vec::new(); usize::from(self.row_count)];
        for cell in &self.cells {
            rows[usize::from(cell.row)].push(cell);
        }
        let mut consensus = None;
        for cells in &mut rows {
            cells.sort_unstable_by_key(|cell| cell.col);
            let mut next_col = 0u16;
            let mut width = 0u64;
            for cell in cells.iter() {
                if cell.col != next_col {
                    return None;
                }
                next_col = next_col.checked_add(cell.col_span)?;
                if next_col > self.col_count {
                    return None;
                }
                width = width.checked_add(u64::from(self.physical_cell_width_hu(cell)))?;
            }
            if next_col != self.col_count {
                return None;
            }
            // 병합 셀 안의 간격은 cell.width에 포함된다. 물리 셀 사이만 더한다.
            let spacing =
                i64::from(self.outer_cell_spacing_hu()) + self.inner_cell_spacing_hu(cells.len());
            width = width.checked_add(u64::try_from(spacing).ok()?)?;
            let width = u32::try_from(width).ok()?;
            if width > i32::MAX as u32 || consensus.is_some_and(|prior| prior != width) {
                return None;
            }
            consensus = Some(width);
        }
        consensus
    }
}
