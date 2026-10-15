//! 병합 셀의 소유 단위와 물리 페이지 예산을 구분한다.

use super::MeasuredTable;
use crate::model::table::TablePageBreak;

impl MeasuredTable {
    /// CellBreak는 병합 셀을 포함한 행 그룹을 하나의 단위로 보존한다.
    /// RowBreak의 작은 제목/라벨 그룹과 None의 기존 분류는 유지한다.
    /// 단위가 실제 페이지보다 큰 경우의 컷은 소비자가 페이지 예산으로 결정한다.
    pub(crate) fn rowspan_block_uses_whole_unit(&self, start: usize, end: usize) -> bool {
        let size = end.saturating_sub(start);
        size >= 2
            && (matches!(self.page_break, TablePageBreak::CellBreak)
                || size <= super::BLOCK_UNIT_MAX_ROWS)
    }

    pub(crate) fn rowspan_block_fits_whole_unit(
        &self,
        start: usize,
        end: usize,
        fresh_row_budget: f64,
    ) -> bool {
        self.rowspan_block_uses_whole_unit(start, end)
            && (!matches!(self.page_break, TablePageBreak::CellBreak)
                || self.range_height(start, end) <= fresh_row_budget)
    }

    /// Paginator의 행 기반 fallback은 온전한 다음 밴드에 들어가는 단위만 snap한다.
    /// 실제 page-larger 그룹은 행/내용 컷으로 전진하여 통째 overflow를 피한다.
    pub(crate) fn snap_to_block_boundary_for_budget(
        &self,
        end_row: usize,
        fresh_row_budget: f64,
    ) -> usize {
        if self.allows_row_break_split() || end_row >= self.row_heights.len() {
            return end_row.min(self.row_heights.len());
        }
        let (start, end, _) = self.row_block_for(end_row);
        if self.rowspan_block_fits_whole_unit(start, end, fresh_row_budget) {
            start
        } else {
            end_row
        }
    }
}
