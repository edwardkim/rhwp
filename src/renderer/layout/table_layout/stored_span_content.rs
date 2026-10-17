//! Stored text spans distinguish a line's occupied box from its next-line advance.

use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, TablePageBreak};
use crate::renderer::hwpunit_to_px;

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 온전한 행 경계의 수용 상자가 이미 소유한 빈 하단 띠다.
    /// 내용·안여백·앞 조각의 띠를 반영한 자연 행 높이와 비교하므로
    /// 병합 셀의 남은 내용이나 패딩을 빈 공간으로 다시 차감하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserved_straddle_end_band(
        &self,
        table: &Table,
        resolved_row_heights: &[f64],
        start_row: usize,
        end_row: usize,
        start_cut: &[usize],
        is_continuation: bool,
        start_row_height_override: Option<f64>,
        end_row_height_override: Option<f64>,
        straddle_row_relief: &[(usize, f64)],
        carried: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let Some(accepted) = end_row_height_override else {
            return 0.0;
        };
        let Some(last) = end_row.checked_sub(1).filter(|last| *last >= start_row) else {
            return 0.0;
        };
        if !table.cells.iter().any(|cell| {
            usize::from(cell.row) < end_row
                && end_row < usize::from(cell.row) + usize::from(cell.row_span)
        }) {
            return 0.0;
        }
        // 배치와 같은 행 소유 결과에서 마지막 상자 확장만 제외한다.
        let natural = self.ordinary_rowbreak_fragment_heights(
            table,
            resolved_row_heights,
            start_row,
            end_row,
            start_cut,
            &[],
            is_continuation,
            start_row_height_override,
            None,
            straddle_row_relief,
            carried,
            true,
            styles,
        );
        natural
            .get(last)
            .map_or(0.0, |height| (accepted - height).max(0.0))
    }

    /// Physical prefix at a RowCut, measured from this spanning owner's origin.
    pub(in crate::renderer::layout) fn rowbreak_cut_prefix_height(
        &self,
        table: &Table,
        cell: &Cell,
        boundary_row: usize,
        cut: &[usize],
        resolved_row_heights: &[f64],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let stored_physical_span = self.stored_plain_text_span_frame(table, cell);
        let spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let mut height = 0.0;
        for row in cell.row as usize..boundary_row {
            height += self.rowbreak_prior_row_height(
                table,
                row,
                resolved_row_heights,
                styles,
                stored_physical_span,
            ) + spacing;
        }
        height + self.row_cut_content_height(table, boundary_row, &[], cut, styles)
    }

    pub(in crate::renderer::layout) fn rowbreak_prior_row_height(
        &self,
        table: &Table,
        row: usize,
        resolved_row_heights: &[f64],
        styles: &ResolvedStyleSet,
        stored_physical_span: bool,
    ) -> f64 {
        let physical = resolved_row_heights.get(row).copied().unwrap_or(0.0);
        // The previous fragment accepted this physical box. Re-measuring its
        // single-row siblings discards the trailing band and repeats span units.
        if stored_physical_span {
            return physical;
        }
        let has_single_row_cells = table
            .cells
            .iter()
            .any(|cell| cell.row as usize == row && cell.row_span == 1);
        let content = if has_single_row_cells {
            self.row_cut_content_height(table, row, &[], &[], styles)
        } else {
            0.0
        };
        if content > 0.0 {
            content
        } else {
            physical
        }
    }

    pub(in crate::renderer::layout) fn stored_plain_text_span_frame(
        &self,
        table: &Table,
        cell: &Cell,
    ) -> bool {
        cell.row_span > 1 && self.stored_plain_text_cell_frame(table, cell)
    }

    pub(super) fn stored_plain_text_cell_frame(&self, table: &Table, cell: &Cell) -> bool {
        let profile = self.profile.get();
        (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            && !profile.session_edited()
            && !self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            && !table.common.treat_as_char
            && table.page_break == TablePageBreak::RowBreak
            && !cell.dirty_flag
            && !cell.paragraphs.is_empty()
            && cell.paragraphs.iter().all(|para| {
                !para.stored_text_partition_is_dirty()
                    && !para.cell_format_vpos_dirty
                    && para.controls.is_empty()
                    && !para.line_segs.is_empty()
                    && para.line_segs.iter().all(|line| {
                        line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                            && line.vertical_pos >= 0
                            && line.line_height > 0
                            && line.line_spacing >= 0
                    })
            })
    }

    pub(super) fn stored_span_line_tail_spacing(&self, cell: &Cell, unit: &CellUnit) -> f64 {
        if unit.empty_spacer
            || unit.vis_start >= unit.vis_end
            || unit.nested_row.is_some()
            || unit.mixed_nested_fragment
            || unit.non_inline_control_range.is_some()
        {
            return 0.0;
        }
        cell.paragraphs
            .get(unit.para_idx)
            .and_then(|para| para.line_segs.get(unit.vis_end - 1))
            .map(|line| hwpunit_to_px(line.line_spacing, self.dpi).min(unit.height.max(0.0)))
            .unwrap_or(0.0)
    }
}
