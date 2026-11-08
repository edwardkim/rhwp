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

    /// RowBreak(«나눔») 표의 보호 rowspan 묶음을 첫 행 경계에서 자를 수 있는가.
    ///
    /// 첫 행 상자(`row_h`)가 남은 예산에 들고, 그 행에서 시작하는 rowspan 칸의 **내용
    /// 전체**와 안쪽 여백도 그 행 안에 들면 첫 행 뒤의 컷은 어떤 칸의 글자도 자르지
    /// 않는다. 한/글은 그때 첫 행 뒤에서 자른다(1480000-201900698 표 9·17: `Name` 한 줄
    /// 17.1px ≤ 0행 47.0px). rowspan 칸 내용이 첫 행을 넘으면 묶음을 통째로 넘긴다
    /// (#1145 제목 18.7px > 0행 17.3px, task1753 pi=51 `적용|수량|단위` 3줄 > 0행 28.3px).
    /// 판정은 이월 게이트가 한 번 하고, 행 스캔은 그 결과를 넘겨받는다.
    pub(crate) fn rowbreak_block_first_row_holds_opening(
        &self,
        row: usize,
        row_h: f64,
        budget: f64,
    ) -> bool {
        self.allows_row_break_split()
            && row_h > 0.0
            && row_h <= budget
            && self
                .cells
                .iter()
                .filter(|c| c.row == row && c.row_span > 1)
                .all(|c| c.total_content_height + c.padding_top + c.padding_bottom <= row_h + 0.5)
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
