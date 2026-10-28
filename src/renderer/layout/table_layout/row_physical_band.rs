//! 논리 전진 예산과 실제 조각 끝의 내용 점유를 구분한다.

use super::{
    CellUnit, LayoutEngine, ResolvedStyleSet, RowCutResult, ROW_CUT_CAPACITY_FP_EPSILON_PX,
};
use crate::model::{
    control::Control,
    paragraph::LineSeg,
    table::{Cell, Table},
};

/// 같은 내용 컷의 수용 전진과 실제 마지막 상자를 재귀 부모까지 구분한다.
#[derive(Clone, Copy)]
pub(super) enum CutHeight {
    Capacity,
    Occupied,
}

#[derive(Clone, Copy)]
pub(super) enum RowCutCapacity<'a> {
    Content,
    Physical {
        deltas: &'a [f64],
        allow_stored_tail: bool,
    },
}

impl<'a> RowCutCapacity<'a> {
    pub(super) fn deltas(self) -> Option<&'a [f64]> {
        match self {
            Self::Content => None,
            Self::Physical { deltas, .. } => Some(deltas),
        }
    }

    pub(super) fn allows_stored_tail(self) -> bool {
        match self {
            Self::Content => true,
            Self::Physical {
                allow_stored_tail, ..
            } => allow_stored_tail,
        }
    }
}

impl LayoutEngine {
    /// 독립 선언 최소 안의 마지막 저장 슬롯은 완결 수용량에 포함한다.
    /// 받아들인 컷의 잉크·안여백은 기존 Occupied 경로가 그대로 측정한다.
    /// 원본 유닛 높이를 늘려 이어받기 쪽에 종료 전진을 다시 더하지 않는다.
    pub(super) fn stored_complete_cell_slot_capacity(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
        end: usize,
    ) -> Option<f64> {
        if start != 0
            || end != units.len()
            || cell.row_span != 1
            || cell.vertical_align != crate::model::table::VerticalAlign::Top
            || cell.height > i32::MAX as u32
            || !self.stored_plain_fragment_table(table)
        {
            return None;
        }
        // 셀 전체의 연속 저장 좌표가 마지막 줄 상자를 증명하면 뒤 줄간격은
        // 다음 소유자의 전진이다. 선언 높이에 남은 빈 밴드는 내용 컷과
        // 독립적으로 나눌 수 있으므로 그 간격으로 마지막 줄을 밀지 않는다.
        // 문단별 좌표가 재시작해 전체 종점이 없는 경우에만 슬롯을 예약한다.
        if crate::renderer::float_placement::stored_plain_cell_ink_height(cell).is_some() {
            return None;
        }
        let last = units.last()?;
        let para = cell.paragraphs.get(last.para_idx)?;
        if last.empty_spacer
            || last.para_idx + 1 != cell.paragraphs.len()
            || last.vis_end != para.line_segs.len()
            || last.vis_start >= last.vis_end
            || last.following_line_advance != 0.0
        {
            return None;
        }
        let line = para.line_segs.last()?;
        let gap = crate::renderer::hwpunit_to_px(line.line_spacing, self.dpi);
        let (_, _, top, bottom) = self.resolve_cell_padding(cell, table);
        let complete = units.iter().map(|unit| unit.height).sum::<f64>() + gap + top + bottom;
        (gap > 0.0
            && crate::renderer::hwpunit_to_px(cell.height as i32, self.dpi)
                > complete + crate::renderer::hwpunit_to_px(2, self.dpi))
        .then_some(complete)
    }

    /// 실제 선택한 컷에 일반 글줄만 남는 재조판 행인지 확인한다.
    /// 바깥 프레임을 쓰는 개체 viewport를 일반 글줄과 혼동하지 않는다.
    pub(crate) fn row_cut_has_plain_reflow_units(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !self.row_uses_outer_fragment_frame(table, row) {
            return false;
        }
        let (cut, _) =
            self.advance_row_cut_for_physical_band(table, row, start_cut, available, styles);
        let mut has_reflow_line = false;
        for (slot, index) in Self::row_cut_cell_order(table, row).into_iter().enumerate() {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(slot).copied().unwrap_or(0).min(units.len());
            let end = cut
                .end_cut
                .get(slot)
                .copied()
                .unwrap_or(start)
                .clamp(start, units.len());
            for unit in &units[start..end] {
                if unit.nested_row.is_some()
                    || unit.nested_table_fragment.is_some()
                    || unit.mixed_nested_fragment
                    || unit.top_and_bottom_flow
                    || unit.non_inline_control_range.is_some()
                {
                    return false;
                }
                has_reflow_line |= cell
                    .paragraphs
                    .get(unit.para_idx)
                    .is_some_and(crate::renderer::para_has_no_stored_line_segs);
            }
        }
        has_reflow_line
    }

    /// 앞 쪽 꼬리의 무료 spacer는 바로 뒤 저장 프레임이 재시작함을 확인한다.
    /// 새 조각의 빈 줄은 양의 높이를 가진 상태로 같은 컷의 예약과 paint에 남긴다.
    pub(super) fn physical_start_after_leading_spacers(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
        styles: &ResolvedStyleSet,
    ) -> usize {
        if start == 0 {
            return start;
        }
        let mut next = start.min(units.len());
        while units
            .get(next)
            .is_some_and(|unit| self.is_free_leading_spacer(cell, table, unit, styles))
        {
            next += 1;
        }
        let restarts_after = units.get(next).is_some_and(|unit| {
            unit.hard_break_before || unit.stored_frame_break_before || unit.page_frame_reset_before
        });
        if next > start && restarts_after {
            next
        } else {
            start
        }
    }

    /// 같은 저장 줄의 TAC 표 본체와 위·아래 여백이 그 줄 상자를 실제로 소유한다.
    fn unit_owns_stored_inline_table_frame(&self, cell: &Cell, unit: &CellUnit) -> bool {
        if !(self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().hwpx_stored_layout())
        {
            return false;
        }
        let Some(para) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        if !para.text.trim().is_empty()
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || crate::renderer::para_has_no_stored_line_segs(para)
        {
            return false;
        }
        para.controls.iter().enumerate().any(|(index, control)| {
            let Control::Table(child) = control else {
                return false;
            };
            if !child.common.treat_as_char
                || child.common.height == 0
                || self
                    .render_normalization
                    .borrow()
                    .table_text_reflowed(child)
            {
                return false;
            }
            let Some(line_index) = super::super::control_line_seg_index(para, index) else {
                return false;
            };
            let height = i64::from(child.common.height)
                + i64::from(child.outer_margin_top)
                + i64::from(child.outer_margin_bottom);
            unit.vis_start <= line_index
                && line_index < unit.vis_end
                && para.line_segs.get(line_index).is_some_and(|line| {
                    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                        && i64::from(line.line_height) == height
                })
        })
    }

    /// 바깥 조각의 예산은 행 전체가 아닌 표의 실제 프레임 소유로 정한다.
    /// 부모가 크기를 정하는 중첩 viewport는 자신의 예산을 유지한다.
    pub(crate) fn row_uses_outer_fragment_frame(&self, table: &Table, row: usize) -> bool {
        !self
            .nested_table_ptrs
            .borrow()
            .contains(&(table as *const Table as usize))
            && !Self::row_cut_cell_order(table, row).is_empty()
    }

    /// 현재 밴드가 실제로 소비하는 일반 글줄의 바깥 프레임이다.
    /// 먼 개체의 존재나 표의 중첩 깊이만으로 현재 컷의 소유자를 추정하지 않는다.
    pub(crate) fn row_cut_uses_outer_line_frame(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !self.row_uses_outer_fragment_frame(table, row) {
            return false;
        }
        if self.row_has_plain_line_units(table, row, styles) {
            return true;
        }
        // 내용 선택과 같은 물리 컷 생산자를 사용한다. 개체 viewport가 예약한
        // 몫까지 일반 글줄의 바깥 예산으로 다시 차감하지 않는다.
        let (cut, _) =
            self.advance_row_cut_for_physical_band(table, row, start_cut, available, styles);
        let mut has_content = false;
        for (slot, index) in Self::row_cut_cell_order(table, row).into_iter().enumerate() {
            let units = self.cell_units(&table.cells[index], table, styles);
            let start = start_cut.get(slot).copied().unwrap_or(0).min(units.len());
            let end = cut
                .end_cut
                .get(slot)
                .copied()
                .unwrap_or(start)
                .clamp(start, units.len());
            for unit in &units[start..end] {
                if self.unit_owns_stored_inline_table_frame(&table.cells[index], unit) {
                    return false;
                }
                if unit.nested_row.is_some()
                    || unit.nested_table_fragment.is_some()
                    || unit.mixed_nested_fragment
                    || unit.top_and_bottom_flow
                    || unit.non_inline_control_range.is_some()
                {
                    // 저장 부모 프레임이 없는 재구성 사다리는 이 viewport도
                    // 현재 바깥 조각 안에서 예약해야 한다. 자식 reset을 부모
                    // 쪽 프레임의 증거로 대신하지 않는다.
                    return !self.table_uses_stored_fragment_coordinates(table);
                }
                has_content = true;
            }
        }
        has_content
    }

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
            first = self
                .physical_start_after_leading_spacers(cell, table, units, first, styles)
                .min(last);
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
        // 외부 공동 앵커의 컷은 부모 패딩까지 포함한 물리 프레임과 비교했다.
        // 일반 칸 예산으로 바꾸며 같은 패딩을 다시 빼지 않는다.
        if let Some(cut) =
            self.advance_reflow_external_row_cut(table, row, start_cut, physical_budget, styles)
        {
            return (cut, physical_budget);
        }
        let padding = self.row_remaining_visible_cell_padding(table, row, start_cut, styles);
        let max_padding = padding.iter().copied().fold(0.0, f64::max);
        let mut physical_cut_budget = physical_budget;
        let mut previous_cut = None;
        let mut allow_stored_tail = true;
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
                RowCutCapacity::Physical {
                    deltas: &deltas,
                    allow_stored_tail,
                },
            );
            if let Some(fitted) = self.advance_reflow_nested_closing_band(
                table,
                row,
                start_cut,
                cut.clone(),
                physical_budget,
                styles,
            ) {
                return (fitted, budget);
            }
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
            // 실제 상자가 예산을 넘으면 옛 저장 reset까지의 꼬리 흡수는
            // 현재 컷의 수용 근거가 아니다. 원래 물리 예산에서 다시 선택해
            // 초과를 일으킨 허용량 때문에 정상 앞부분까지 물러나지 않는다.
            if excess > ROW_CUT_CAPACITY_FP_EPSILON_PX && allow_stored_tail {
                allow_stored_tail = false;
                physical_cut_budget = physical_budget;
                previous_cut = None;
                continue;
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
