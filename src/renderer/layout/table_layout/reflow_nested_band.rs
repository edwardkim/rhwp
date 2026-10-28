//! 재조판 중첩 행의 내용 컷과 쪽 끝의 빈 물리 띠를 함께 결정한다.

use super::{LayoutEngine, ResolvedStyleSet, RowCutResult};
use crate::model::{control::Control, table::Table};
use crate::renderer::hwpunit_to_px;

/// 부모 칸의 물리 예산을 공유하는 재조판 RowBreak 자식 프레임이다.
pub(super) fn owns_constrained_row_frame(
    para: &crate::model::paragraph::Paragraph,
    child: &Table,
) -> bool {
    super::reflow_nested_table_has_outer_frame(para, child)
        && child.page_break == crate::model::table::TablePageBreak::RowBreak
        && !crate::renderer::float_placement::unrestricted_cell_float(&child.common)
}

pub(in crate::renderer::layout) struct ReflowNestedBand {
    pub parent_height: f64,
    pub last_row_height: f64,
}

impl LayoutEngine {
    /// 배치는 이미 수용한 부모 상자를 입력으로 같은 종료 계획을 다시 조회한다.
    pub(in crate::renderer::layout) fn reflow_nested_band_at_height(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        height: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<ReflowNestedBand> {
        for (slot, index) in self
            .indexed_row_cut_cell_order(table, row)
            .into_iter()
            .enumerate()
        {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            let first = start.get(slot).copied().unwrap_or(0).min(units.len());
            if end.get(slot).copied().unwrap_or(first) <= first {
                continue;
            }
            let para = cell.paragraphs.get(units.get(first)?.para_idx)?;
            let child = para.controls.iter().find_map(|control| match control {
                Control::Table(child) => Some(child.as_ref()),
                _ => None,
            })?;
            return self.reflow_nested_closing_band(
                table,
                row,
                start,
                end,
                height + hwpunit_to_px(i32::from(child.outer_margin_bottom), self.dpi),
                styles,
            );
        }
        None
    }

    /// 내용은 모두 수용하되 선언 최소의 빈 끝만 쪽 예산으로 닫는다.
    /// 저장 줄·셀 내부 컷·걸침 행은 각각의 기존 물리 원장을 유지한다.
    pub(in crate::renderer::layout) fn reflow_nested_closing_band(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<ReflowNestedBand> {
        if !crate::renderer::float_placement::paragraph_following_overlay_rowbreak_frame(table)
            || !self.row_uses_outer_fragment_frame(table, row)
        {
            return None;
        }
        let mut owner = None;
        for (slot, index) in self
            .indexed_row_cut_cell_order(table, row)
            .into_iter()
            .enumerate()
        {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            let first = start.get(slot).copied().unwrap_or(0).min(units.len());
            let last = end
                .get(slot)
                .copied()
                .unwrap_or(first)
                .clamp(first, units.len());
            if first == last {
                continue;
            }
            if cell.row_span != 1 || owner.is_some() {
                return None;
            }
            let selected = &units[first..last];
            let cut = Self::nested_row_projection_cut(selected)?;
            if !cut.start_cut.is_empty() || !cut.end_cut.is_empty() {
                return None;
            }
            let pi = selected.first()?.para_idx;
            if selected.iter().any(|unit| unit.para_idx != pi) {
                return None;
            }
            let para = cell.paragraphs.get(pi)?;
            if !crate::renderer::para_has_no_stored_line_segs(para) || !para.text.trim().is_empty()
            {
                return None;
            }
            let mut children = para.controls.iter().filter_map(|control| match control {
                Control::Table(child) => Some(child.as_ref()),
                _ => None,
            });
            let child = children.next()?;
            if children.next().is_some()
                || !owns_constrained_row_frame(para, child)
                || cut.end_row >= usize::from(child.row_count)
                || child.cells.iter().any(|cell| cell.row_span != 1)
            {
                return None;
            }
            let last_row = cut.end_row.checked_sub(1)?;
            let child_cells = self.indexed_row_cut_cell_order(child, last_row);
            if child_cells.is_empty()
                || child_cells.iter().any(|&i| {
                    child.cells[i].paragraphs.iter().any(|para| {
                        !crate::renderer::para_has_no_stored_line_segs(para)
                            || !para.controls.is_empty()
                    })
                })
            {
                return None;
            }
            let last_end: Vec<_> = child_cells
                .iter()
                .map(|&i| self.cell_units(&child.cells[i], child, styles).len())
                .collect();
            // 명시적인 전체 내용 컷은 선언된 빈 최소를 강제하지 않는다.
            let content_minimum =
                self.row_cut_content_height(child, last_row, &[], &last_end, styles);
            let rows = self.resolve_row_heights(
                child,
                usize::from(child.col_count),
                usize::from(child.row_count),
                None,
                styles,
                true,
            );
            let declared = *rows.get(last_row)?;
            let trailing_outer = hwpunit_to_px(i32::from(child.outer_margin_bottom), self.dpi);
            // 부모의 흐름 종료 띠와 자식 viewport의 닫는 띠를 행의 빈 최소와
            // 구분한다. 같은 수용 예산에서 정한 두 경계를 배치도 그대로 쓴다.
            let parent_height = available - trailing_outer;
            let full = self.row_cut_content_height(table, row, start, end, styles);
            let last_row_height = declared + parent_height - full - trailing_outer;
            if last_row_height + super::ROW_CUT_CAPACITY_FP_EPSILON_PX < content_minimum
                || last_row_height >= declared
                || last_row_height <= 0.0
            {
                return None;
            }
            owner = Some(ReflowNestedBand {
                parent_height,
                last_row_height,
            });
        }
        owner
    }

    /// 기존 컷 또는 바로 다음 원자 행의 빈 최소만 줄여 같은 쪽에 수용한다.
    pub(super) fn advance_reflow_nested_closing_band(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        mut cut: RowCutResult,
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<RowCutResult> {
        if cut.hit_hard_break || cut.fully_consumed {
            return None;
        }
        let mut accepted =
            self.reflow_nested_closing_band(table, row, start, &cut.end_cut, available, styles);
        for (slot, index) in self
            .indexed_row_cut_cell_order(table, row)
            .into_iter()
            .enumerate()
        {
            let units = self.cell_units(&table.cells[index], table, styles);
            if cut.end_cut.get(slot).is_none_or(|&end| end >= units.len()) {
                continue;
            }
            let mut next = cut.end_cut.clone();
            next[slot] += 1;
            if let Some(band) =
                self.reflow_nested_closing_band(table, row, start, &next, available, styles)
            {
                cut.end_cut = next;
                accepted = Some(band);
            }
        }
        let band = accepted?;
        let padding = self
            .row_remaining_visible_cell_padding(table, row, start, styles)
            .into_iter()
            .fold(0.0_f64, f64::max);
        cut.consumed_height = (band.parent_height - padding).max(0.0);
        cut.physical_height = Some(band.parent_height);
        Some(cut)
    }
}
