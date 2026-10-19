//! 논리 전진 예산과 실제 조각 끝의 내용 점유를 구분한다.

use super::{
    CellUnit, LayoutEngine, ResolvedStyleSet, RowCutResult, ROW_CUT_CAPACITY_FP_EPSILON_PX,
};
use crate::model::table::{Cell, Table};

#[derive(Clone, Copy)]
pub(super) enum RowCutCapacity<'a> {
    Content,
    Physical(&'a [f64]),
}

impl<'a> RowCutCapacity<'a> {
    pub(super) fn deltas(self) -> Option<&'a [f64]> {
        match self {
            Self::Content => None,
            Self::Physical(deltas) => Some(deltas),
        }
    }
}

impl LayoutEngine {
    /// 일반 셀이 없어 컷 슬롯이 비어도 시작 행의 잔여 물리 상자는 존재한다.
    /// 끝나는 병합 셀의 내용 요구는 기존 continuation demand가 별도로 예약한다.
    /// 블록 컷과 일반 셀 컷은 각자의 내용/물리 상자 계약을 유지한다.
    pub(crate) fn carried_span_row_physical_height(
        table: &Table,
        row: usize,
        start_cut: &[usize],
        start_cut_is_block: bool,
        carried: Option<f64>,
    ) -> Option<f64> {
        let height = carried.filter(|height| height.is_finite() && *height >= 0.0)?;
        if start_cut_is_block
            || !start_cut.is_empty()
            || table
                .cells
                .iter()
                .any(|cell| usize::from(cell.row) == row && cell.row_span == 1)
        {
            return None;
        }
        table
            .cells
            .iter()
            .any(|cell| {
                let first = usize::from(cell.row);
                cell.row_span > 1 && first <= row && row < first + usize::from(cell.row_span)
            })
            .then_some(height)
    }

    /// 선택한 문단의 다음 줄 전진은 다음 조각에 속한다. 문단 앞/뒤 간격과
    /// 빈 줄·개체의 점유를 추정하지 않고 유닛 생산자가 분리한 값만 소비한다.
    /// 기존 저장 컷의 종료 계약은 그대로 유지한다.
    pub(super) fn physical_cut_trailing_trim(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let stored =
            self.native_saved_reset_cut_trailing_trim(table, cell, units, start, end, styles);
        let following =
            if start < end && end < units.len() && units[end - 1].para_idx != units[end].para_idx {
                self.stored_span_line_tail_spacing(&units[end - 1])
            } else {
                0.0
            };
        stored.max(following)
    }

    /// 실제 칸별 안여백과 같은 컷의 내용 점유로 물리 밴드 안의 유닛을 고른다.
    /// 안여백이 같아도 논리 전진 예산으로 되돌아가지 않는다.
    pub(crate) fn advance_row_cut_for_physical_band(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        physical_budget: f64,
        styles: &ResolvedStyleSet,
    ) -> (RowCutResult, f64) {
        let padding = self.row_remaining_visible_cell_padding(table, row, start_cut, styles);
        let max_padding = padding.iter().copied().fold(0.0, f64::max);
        let mut physical_cut_budget = physical_budget;
        let mut previous_cut = None;
        loop {
            let budget = (physical_cut_budget - max_padding).max(0.0);
            let deltas: Vec<f64> = padding
                .iter()
                .map(|value| (physical_cut_budget - value).max(0.0) - budget)
                .collect();
            let cut = self.advance_row_cut_inner(
                table,
                row,
                start_cut,
                budget,
                styles,
                RowCutCapacity::Physical(&deltas),
            );
            let visible = self.row_cut_content_height(table, row, start_cut, &cut.end_cut, styles);
            let excess = (visible - physical_budget).max(0.0);
            if excess > ROW_CUT_CAPACITY_FP_EPSILON_PX && std::env::var("RHWP_DIAG_SCAN").is_ok() {
                let owners: Vec<_> = Self::row_cut_cell_order(table, row)
                    .iter()
                    .enumerate()
                    .map(|(slot, &index)| {
                        let cell = &table.cells[index];
                        let units = self.cell_units(cell, table, styles);
                        let start = start_cut.get(slot).copied().unwrap_or(0).min(units.len());
                        let end = cut.end_cut[slot].clamp(start, units.len());
                        let advance = units[start..end]
                            .iter()
                            .map(|unit| unit.height)
                            .sum::<f64>();
                        let trim = self
                            .physical_cut_trailing_trim(table, cell, &units, start, end, styles);
                        (cell.col, start, end, advance, trim)
                    })
                    .collect();
                eprintln!(
                    "DIAG_SCAN PHYSICAL_REFIT r={row} start={start_cut:?} end={:?} physical={physical_budget:.3} budget={budget:.3} visible={visible:.3} owners={owners:?}",
                    cut.end_cut,
                );
            }
            if excess <= ROW_CUT_CAPACITY_FP_EPSILON_PX
                || physical_cut_budget <= 0.0
                || previous_cut.as_ref() == Some(&cut.end_cut)
            {
                return (cut, budget);
            }
            previous_cut = Some(cut.end_cut);
            physical_cut_budget = (physical_cut_budget - excess).max(0.0);
        }
    }
}
