//! 논리 전진 예산과 실제 조각 끝의 내용 점유를 구분한다.

use super::{
    CellUnit, LayoutEngine, ResolvedStyleSet, RowCutResult, ROW_CUT_CAPACITY_FP_EPSILON_PX,
};
use crate::model::table::{Cell, Table};

/// 같은 내용 컷의 수용 전진과 실제 마지막 상자를 재귀 부모까지 구분한다.
#[derive(Clone, Copy)]
pub(super) enum CutHeight {
    Capacity,
    Occupied,
}

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
    /// 내용 커서는 무높이 보조 spacer도 소비한다. 이어받은 조각의 실제
    /// 전진 범위는 이 공통 소유 판정으로 컷 선택과 물리 높이 측정에 전달한다.
    /// 다른 컷 경로의 원본 빈 문단 소유까지 이 전진 범위로 대체하지 않는다.
    /// 저장 빈 줄과 재조판 빈 문단의 유효 줄 상자는 제외하지 않는다.
    pub(super) fn cell_cut_flow_range(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> std::ops::Range<usize> {
        let mut first = start.min(units.len());
        let mut last = end.clamp(first, units.len());
        if first > 0 {
            while first < last && self.is_free_leading_spacer(cell, table, &units[first], styles) {
                first += 1;
            }
            if last == units.len() {
                while last > first
                    && self.is_free_leading_spacer(cell, table, &units[last - 1], styles)
                {
                    last -= 1;
                }
            }
        }
        first..last
    }

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

    /// 수용 용량과 실제 마지막 줄 상자를 같은 유닛의 서로 다른 값으로 구분한다.
    /// 저장 슬롯이 없는 새 줄은 다음 줄 전진까지 들어가야 수용할 수 있다.
    /// 수용한 뒤 다음 조각으로 넘어가는 전진은 실제 끝 상자에 다시 포함하지 않는다.
    fn cut_trailing_advances(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> (f64, f64) {
        let stored =
            self.native_saved_reset_cut_trailing_trim(table, cell, units, start, end, styles);
        let Some(last) = end.checked_sub(1).and_then(|index| units.get(index)) else {
            return (stored, stored);
        };
        if start >= end || end >= units.len() {
            return (stored, stored);
        }
        let original_slot = !last.reflowed_line_slot
            && cell.paragraphs.get(last.para_idx).is_some_and(|para| {
                !para.stored_text_partition_is_dirty()
                    && !para.cell_format_vpos_dirty
                    && last.vis_end.checked_sub(1).is_some_and(|line| {
                        para.line_segs.get(line).is_some_and(|seg| {
                            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                == 0
                        })
                    })
            });
        let paragraph_end = last.para_idx != units[end].para_idx;
        let following = self.stored_span_line_tail_spacing(last);
        // 문단 끝 유닛에는 줄간격과 문단 뒤 간격이 함께 들어 있다.
        // 다음 문단이 다른 조각에 속하면 두 간격 모두 그 소유자에게 넘긴다.
        // 마지막 글줄의 뒤 간격을 확인한 일반 텍스트 유닛에만 적용한다.
        let following = if paragraph_end && following > 0.0 {
            following
                + cell
                    .paragraphs
                    .get(last.para_idx)
                    .and_then(|para| styles.para_styles.get(para.para_shape_id as usize))
                    .map_or(0.0, |style| style.spacing_after.max(0.0))
        } else {
            following
        };
        let capacity = if original_slot && paragraph_end {
            following
        } else {
            0.0
        };
        let occupied = if paragraph_end || !original_slot {
            following
        } else {
            0.0
        };
        (stored.max(capacity), stored.max(occupied))
    }

    /// 저장 줄이 증언한 컷의 수용량만 완화한다. 새 줄의 잉크 높이로 용량을 늘리지 않는다.
    pub(super) fn row_cut_capacity_trailing_trim(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        self.cut_trailing_advances(table, cell, units, start, end, styles)
            .0
    }

    /// 수용된 컷의 실제 끝 상자. 문단 간격·빈 줄·개체의 점유는 그대로 유지한다.
    pub(super) fn physical_cut_trailing_trim(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        self.cut_trailing_advances(table, cell, units, start, end, styles)
            .1
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
            let required = self.row_cut_height(
                table,
                row,
                start_cut,
                &cut.end_cut,
                styles,
                CutHeight::Capacity,
            );
            let excess = (required - physical_budget).max(0.0);
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
                    "DIAG_SCAN PHYSICAL_REFIT r={row} start={start_cut:?} end={:?} physical={physical_budget:.3} budget={budget:.3} required={required:.3} owners={owners:?}",
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
            // 논리 컷이 아직 예약하지 않은 자식 상자·전진 전체를 다음 예산에 반영한다.
            // 초과분만 빼면 남은 여유가 그 예약을 상쇄해 같은 컷을 다시 고를 수 있다.
            let additional_reserve = (required - cut.consumed_height - max_padding).max(0.0);
            physical_cut_budget = (physical_cut_budget - excess)
                .min(physical_budget - additional_reserve)
                .max(0.0);
        }
    }
}
