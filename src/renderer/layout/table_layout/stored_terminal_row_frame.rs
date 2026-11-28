//! Complete original rows retain their source padding and terminal ownership.

use crate::model::table::Table;
use crate::renderer::hwpunit_to_px;

use super::{LayoutEngine, ResolvedStyleSet, ROW_CUT_CAPACITY_FP_EPSILON_PX};

impl LayoutEngine {
    /// 원본 0 원점에서 시작해 마지막 빈 줄 하단까지 닫힌 셀의 내용 높이.
    /// 호출자가 후속 문단의 저장 좌표와 실제 물리 상자를 함께 확인한다.
    pub(crate) fn stored_terminal_blank_cell_content_hu(
        &self,
        table: &Table,
        start: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<i32> {
        use crate::model::paragraph::LineSeg;
        let [cell] = table.cells.as_slice() else {
            return None;
        };
        if !self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().session_edited()
            || table.common.treat_as_char
            || table.layout_page_break() != crate::model::table::TablePageBreak::RowBreak
            || cell.dirty_flag
            || self
                .render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell)
            || self.cell_unit_stored_page_frame_origin(cell, table, styles, start) != Some(0)
        {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let first = units.get(start)?;
        let suffix = cell.paragraphs.get(first.para_idx..)?;
        let last = suffix.last()?;
        let [closing] = last.line_segs.as_slice() else {
            return None;
        };
        if first.vis_start != 0
            || !last.text.is_empty()
            || !last.controls.is_empty()
            || suffix.iter().any(|para| {
                !para.controls.is_empty()
                    || para.line_segs.is_empty()
                    || para.stored_text_partition_is_dirty()
                    || para.cell_format_vpos_dirty
            })
        {
            return None;
        }
        let mut end = 0i32;
        for line in suffix.iter().flat_map(|para| &para.line_segs) {
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.vertical_pos < end
                || line.line_height <= 0
            {
                return None;
            }
            end = line.vertical_pos.checked_add(line.line_height)?;
        }
        (closing.vertical_pos > 0).then_some(end)
    }

    /// 현재 조각에서 시작해 이 행에서 끝나는 병합 칸은 수용한 행 상자에서 완결한다.
    /// 다른 칸이 뒤 행으로 이어진다는 사실은 이 칸의 빈 꼬리를 다시 여는 근거가 아니다.
    /// 앞 쪽에서 시작한 칸과 개체·저장 재시작은 이 원본 전체 내용 검사로 추정하지 않는다.
    /// 이 행에서 끝나는 병합 칸이 없으면 추가로 완결해야 할 소유자도 없다.
    /// 일반 칸의 원본·내용 완결 여부는 호출자가 같은 실제 띠로 확인한다.
    pub(crate) fn stored_closing_span_content_fits(
        &self,
        table: &Table,
        row: usize,
        first_row: usize,
        row_heights: &[f64],
        physical: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !physical.is_finite() || physical <= 0.0 || row >= row_heights.len() {
            return false;
        }
        let spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        table
            .cells
            .iter()
            .filter(|cell| {
                cell.row_span > 1 && cell.row as usize + cell.row_span as usize == row + 1
            })
            .all(|cell| {
                let first = cell.row as usize;
                if first < first_row
                    || !self.stored_plain_text_cell_frame(table, cell)
                    || !crate::renderer::cell_vpos_ladder_is_intact(&cell.paragraphs)
                {
                    return false;
                }
                let units = self.cell_units(cell, table, styles);
                let accepted = row_heights[first..row].iter().sum::<f64>()
                    + spacing * (row - first) as f64
                    + physical;
                !units.is_empty()
                    && !units
                        .iter()
                        .any(|unit| unit.hard_break_before || unit.page_frame_reset_before)
                    && self.cell_cut_visible_height(cell, table, styles, 0, units.len()) <= accepted
            })
    }

    /// A certified source fragment can finish its row before a merged owner's
    /// physical tail ends. Its whole source content must fit the same box;
    /// later rows retain ownership of the merged owner's remaining space.
    pub(crate) fn stored_complete_source_row_content_fits(
        &self,
        table: &Table,
        row: usize,
        first_row: usize,
        row_heights: &[f64],
        remaining: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if row >= row_heights.len()
            || row < first_row
            || !remaining.is_finite()
            || remaining <= 0.0
            || !table
                .cells
                .iter()
                .any(|cell| cell.row as usize == row && cell.row_span == 1)
            || self.row_complete_cut_content_height(table, row, styles) > remaining
        {
            return false;
        }
        let spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        table
            .cells
            .iter()
            .filter(|cell| {
                cell.row as usize <= row && row < cell.row as usize + cell.row_span as usize
            })
            .all(|cell| {
                let first = usize::from(cell.row);
                if first < first_row
                    || !self.stored_inline_content_cell_frame(table, cell)
                    || !crate::renderer::cell_vpos_ladder_is_intact(&cell.paragraphs)
                {
                    return false;
                }
                let units = self.cell_units(cell, table, styles);
                let physical = row_heights[first..row].iter().sum::<f64>()
                    + spacing * (row - first) as f64
                    + remaining;
                !units.is_empty()
                    && !units.iter().any(|unit| {
                        unit.hard_break_before
                            || unit.page_frame_reset_before
                            || unit.non_inline_control_range.is_some()
                            || unit.nested_row.is_some()
                            || unit.nested_table_fragment.is_some()
                            || unit.mixed_nested_fragment
                    })
                    && self.cell_cut_visible_height(cell, table, styles, 0, units.len()) <= physical
            })
    }

    /// A content-complete original RowCut also owns its accepted blank band.
    /// This is a physical result, shared by scan, paint and the next prefix.
    pub(crate) fn stored_complete_row_cut_band(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        physical: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if row + 1 >= usize::from(table.row_count)
            || cut.is_empty()
            || !physical.is_finite()
            || physical <= 0.0
            || self.row_cut_content_height(table, row, &[], cut, styles) > physical
        {
            return None;
        }
        let cells = Self::row_cut_cell_order(table, row);
        if cells.len() != cut.len()
            || !cells.iter().zip(cut).all(|(&index, &end)| {
                let cell = &table.cells[index];
                self.stored_plain_text_cell_frame(table, cell)
                    && crate::renderer::cell_vpos_ladder_is_intact(&cell.paragraphs)
                    && end == self.cell_units(cell, table, styles).len()
            })
        {
            return None;
        }
        Some(physical)
    }

    /// A whole source row loses only padding when its small cellSz is used
    /// for a content cut. Its actual grown frame belongs to both scan and paint.
    pub(crate) fn stored_whole_row_padding_frame(
        &self,
        table: &Table,
        row: usize,
        cut: f64,
        painted: f64,
    ) -> bool {
        self.whole_fragment_row_uses_measured_height(table, row)
            && self.whole_row_height_diff_is_padding_reduction(table, row, cut, painted)
            && table
                .cells
                .iter()
                .filter(|cell| cell.row as usize == row)
                .all(|cell| {
                    cell.row_span == 1
                        && self.stored_plain_text_cell_frame(table, cell)
                        && crate::renderer::cell_vpos_ladder_is_intact(&cell.paragraphs)
                })
    }

    /// The last ordinary source row can fit while a spanning cell's cached
    /// blank minimum raises only that row. Complete every closing owner in
    /// its accepted physical box before using the ordinary source boundary.
    /// Interior rows and spans already cut on an earlier page are excluded.
    pub(crate) fn stored_complete_terminal_row_frame(
        &self,
        table: &Table,
        row: usize,
        first_row: usize,
        row_heights: &[f64],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if row + 1 != usize::from(table.row_count)
            || row <= first_row
            || row >= row_heights.len()
            || !available.is_finite()
            || available <= 0.0
        {
            return None;
        }
        let ordinary: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        if ordinary.is_empty()
            || ordinary
                .iter()
                .any(|cell| cell.height == 0 || cell.height > i32::MAX as u32)
        {
            return None;
        }
        let source = ordinary
            .iter()
            .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
            .fold(0.0_f64, f64::max);
        if source > available + ROW_CUT_CAPACITY_FP_EPSILON_PX
            || self.row_complete_cut_content_height(table, row, styles) > available
        {
            return None;
        }
        // 병합 칸이 키운 끝 행은 쪽에 남은 물리 높이까지 그린다. 남는 빈 띠는 다음 쪽으로 넘기지
        // 않는다 — 한/글은 이 행을 쪽 바닥에서 닫는다(입법예고문 docs#333 11쪽: 칸 선언 27.3px,
        // 병합 칸으로 38.6px, 남은 높이 34.0px 를 그린다). 남은 높이가 원본 칸 선언 이하이면
        // 종전과 같다.
        let closing = row_heights[row].max(source).min(available);
        let spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let mut has_closing_span = false;
        for cell in table.cells.iter().filter(|cell| {
            cell.row as usize <= row && row < cell.row as usize + cell.row_span as usize
        }) {
            let first = usize::from(cell.row);
            if first < first_row
                || !self.stored_plain_text_cell_frame(table, cell)
                || !crate::renderer::cell_vpos_ladder_is_intact(&cell.paragraphs)
            {
                return None;
            }
            let units = self.cell_units(cell, table, styles);
            if units.is_empty()
                || units
                    .iter()
                    .any(|unit| unit.hard_break_before || unit.page_frame_reset_before)
            {
                return None;
            }
            let physical = row_heights[first..row].iter().sum::<f64>()
                + spacing * (row - first) as f64
                + closing;
            if self.cell_cut_visible_height(cell, table, styles, 0, units.len()) > physical {
                return None;
            }
            has_closing_span |= cell.row_span > 1;
        }
        has_closing_span.then_some(closing)
    }
}
