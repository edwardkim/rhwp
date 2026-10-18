//! 이어받은 걸침 셀의 내용과 물리 종료 요구를 함께 산출한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::renderer::hwpunit_to_px;

#[derive(Debug, Clone, Copy)]
pub(crate) struct StraddleContinuationDemand {
    /// 이 조각이 받는 내용과 상하 안 여백의 점유.
    pub(crate) content: f64,
    /// 실제 앞 상자를 차감한 원본 물리 최소 높이.
    pub(crate) physical: f64,
}

impl StraddleContinuationDemand {
    pub(crate) fn required(self) -> f64 {
        self.content.max(self.physical)
    }
}

impl LayoutEngine {
    /// 시작 경계를 걸친 셀의 마지막 행을 온전히 배치할 때 필요한 조각 높이.
    /// 끝 컷이 있으면 남은 유닛 전부를 받지 않으므로 그 셀의 증분 예약은 제외한다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn straddle_continuation_demand(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_row: usize,
        start_cut: &[usize],
        start_cut_is_block: bool,
        start_row_height_override: Option<f64>,
        resolved_row_heights: &[f64],
        straddle_page_bands: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &ResolvedStyleSet,
        fragment_end: (usize, bool),
    ) -> Option<StraddleContinuationDemand> {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
                | crate::model::table::TablePageBreak::CellBreak
        ) {
            return None;
        }
        let (end_row, end_cut_is_empty) = fragment_end;
        self.spanning_cells(table)
            .filter(|cell| {
                let cell_row = cell.row as usize;
                let cell_end = cell_row + cell.row_span as usize;
                // 같은 행 재개도 남은 물리 띠를 소유한다. 예약과 paint는
                // 동일한 원본 셀·내용 창과 수용 전 prefix를 소비한다.
                let resumes_own_row = super::super::table_partial::resumes_inside_own_start_row(
                    table,
                    cell,
                    start_row,
                    start_cut,
                    start_cut_is_block,
                    start_row_height_override,
                );
                cell.row_span > 1
                    && (cell_row < start_row || resumes_own_row)
                    && cell_end > start_row
                    && cell_end == row + 1
                    && (cell_end < end_row || (cell_end == end_row && end_cut_is_empty))
            })
            .map(|cell| {
                let (su, eu) = self.rowbreak_straddle_cut_units(
                    table,
                    cell,
                    start_row,
                    end_row,
                    start_cut,
                    start_cut_is_block,
                    start_row_height_override,
                    // 이 소유자의 마지막 행은 온전히 받는다. 뒤 행에 있는
                    // 별도 끝 컷을 이 칸의 내용 컷으로 오인하지 않는다.
                    &[],
                    false,
                    None,
                    0.0,
                    resolved_row_heights,
                    straddle_page_bands,
                    styles,
                );
                let content = self.cell_cut_visible_height(cell, table, styles, su, eu);
                let carry = straddle_page_bands.iter().find(|carry| {
                    table
                        .cells
                        .get(carry.cell_index)
                        .is_some_and(|owner| std::ptr::eq(owner, cell))
                });
                let prefix = carry.and_then(|carry| carry.accepted_prefix_physical_height());
                let physical = prefix.map_or(0.0, |before| {
                    let declared = if cell.height < 0x8000_0000 {
                        hwpunit_to_px(cell.height as i32, self.dpi)
                    } else {
                        0.0
                    };
                    let whole = self.cell_cut_visible_height(
                        cell,
                        table,
                        styles,
                        0,
                        self.cell_units(cell, table, styles).len(),
                    );
                    (declared.max(whole) - before).max(0.0)
                });
                super::unit_diagnostics::trace_continuation_demand(
                    cell,
                    (content, physical),
                    (
                        prefix,
                        carry.and_then(|carry| carry.accepted_physical_height),
                    ),
                    carry.is_some_and(|carry| carry.fragment_cut.is_some()),
                );
                StraddleContinuationDemand { content, physical }
            })
            .reduce(|a, b| StraddleContinuationDemand {
                content: a.content.max(b.content),
                physical: a.physical.max(b.physical),
            })
    }
}
