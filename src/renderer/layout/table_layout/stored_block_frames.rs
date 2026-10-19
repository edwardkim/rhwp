//! 저장 쪽 경계에서 나뉜 블록의 내용 컷과 물리 행 상자를 구분한다.

use crate::model::table::{Table, TablePageBreak};
use crate::renderer::hwpunit_to_px;

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 현재 소유한 원본 컷이 독립된 저장 쪽 프레임을 여닫는가.
    /// 앞뒤 조각의 중첩 개체를 현재 컷의 소유로 추정하지 않는다.
    pub(crate) fn row_block_cut_is_original_source_page(
        &self,
        table: &Table,
        range: (usize, usize),
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        use crate::model::{control::Control, paragraph::LineSeg};

        let profile = self.profile.get();
        if !profile.hwp5_stored_pagination_layout()
            || profile.session_edited()
            || table.common.treat_as_char
            || table.cell_spacing != 0
            || table.page_break != TablePageBreak::RowBreak
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return false;
        }
        let mut cells = Self::row_block_cells(table, range.0, range.1);
        cells.sort_by_key(|cell| (cell.row, cell.col));
        if cells.is_empty() || cells.len() != start_cut.len() || cells.len() != end_cut.len() {
            return false;
        }
        let mut opens_next_frame = false;
        for ((cell, &start), &end) in cells.iter().zip(start_cut).zip(end_cut) {
            let units = self.cell_units(cell, table, styles);
            if cell.dirty_flag || start > end || end > units.len() {
                return false;
            }
            if start == end {
                if end != units.len() {
                    return false;
                }
                continue;
            }
            if start == 0 || !units[start].page_frame_reset_before {
                return false;
            }
            if end < units.len() {
                if !units[end].page_frame_reset_before {
                    return false;
                }
                opens_next_frame = true;
            }
            let mut index = start;
            while index < end {
                let unit = &units[index];
                let para = &cell.paragraphs[unit.para_idx];
                if matches!(para.controls.as_slice(), [Control::Table(_)]) {
                    let stop = units[index..end]
                        .iter()
                        .position(|next| next.para_idx != unit.para_idx)
                        .map_or(end, |length| index + length);
                    if !self.stored_nested_group_matches_source_advance(
                        cell,
                        unit.para_idx,
                        &units[index..stop],
                    ) {
                        return false;
                    }
                    index = stop;
                    continue;
                }
                if unit.reflowed_line_slot
                    || unit.vis_end <= unit.vis_start
                    || para.stored_text_partition_is_dirty()
                    || para.cell_format_vpos_dirty
                    || para
                        .controls
                        .iter()
                        .any(|control| !matches!(control, Control::Header(_) | Control::Footer(_)))
                    || !para
                        .line_segs
                        .get(unit.vis_start..unit.vis_end)
                        .is_some_and(|lines| {
                            lines.iter().all(|line| {
                                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                                    && line.vertical_pos >= 0
                                    && line.line_height > 0
                                    && line.line_spacing >= 0
                            })
                        })
                {
                    return false;
                }
                index += 1;
            }
        }
        opens_next_frame
    }

    /// 온전한 중첩 표의 원본 전진과 선언 상자, 현재 행 유닛의 점유가 같은가.
    /// 개체를 포함한 문단 그룹에서 한 번 확인하며 부분 행 컷은 인증하지 않는다.
    fn stored_nested_group_matches_source_advance(
        &self,
        cell: &crate::model::table::Cell,
        para_index: usize,
        units: &[CellUnit],
    ) -> bool {
        use crate::model::{control::Control, paragraph::LineSeg, shape::VertAlign};
        let para = &cell.paragraphs[para_index];
        let [Control::Table(child)] = para.controls.as_slice() else {
            return false;
        };
        let [line] = para.line_segs.as_slice() else {
            return false;
        };
        let Some(next) = cell.paragraphs.get(para_index + 1) else {
            return false;
        };
        let Some(next_line) = next.line_segs.first() else {
            return false;
        };
        let source_line = |line: &LineSeg| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
                && line.line_spacing >= 0
        };
        if !para.text.trim().is_empty()
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || next.stored_text_partition_is_dirty()
            || next.cell_format_vpos_dirty
            || !source_line(line)
            || !source_line(next_line)
            || child.common.treat_as_char
            || !child.common.flow_with_text
            || child.common.allow_overlap
            || child.common.vert_align != VertAlign::Top
            || !crate::renderer::float_placement::is_para_topbottom_float(&child.common)
            || child.caption.is_some()
            || child.common.height == 0
            || child.common.height > i32::MAX as u32
            || child.outer_margin_top < 0
            || child.outer_margin_bottom < 0
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(child)
            || child.cells.is_empty()
            || child.cells.iter().any(|cell| {
                cell.dirty_flag
                    || cell.paragraphs.iter().any(|para| {
                        para.stored_text_partition_is_dirty()
                            || para.cell_format_vpos_dirty
                            || para.line_segs.is_empty()
                            || !para.line_segs.iter().all(source_line)
                            || para.controls.iter().any(|control| {
                                !matches!(control, Control::Header(_) | Control::Footer(_))
                            })
                    })
            })
            || units.len() != usize::from(child.row_count)
            || units.iter().enumerate().any(|(row, unit)| {
                unit.nested_row != Some(row)
                    || unit.nested_table_fragment.is_some()
                    || unit.mixed_nested_fragment
                    || unit.reflowed_line_slot
            })
        {
            return false;
        }
        let declared = i64::from(child.common.height)
            + i64::from(child.outer_margin_top)
            + i64::from(child.outer_margin_bottom);
        let source_advance = i64::from(next_line.vertical_pos) - i64::from(line.vertical_pos);
        if source_advance != declared || source_advance <= 0 || source_advance > i64::from(i32::MAX)
        {
            return false;
        }
        let measured = units.iter().map(|unit| unit.height).sum::<f64>();
        // 원본 정수 HU로 선언한 점유와 같은지 확인한다. 화면 좌표를 보정하지 않는다.
        (measured - hwpunit_to_px(source_advance as i32, self.dpi)).abs()
            <= hwpunit_to_px(1, self.dpi)
    }

    /// 실제 원본 줄·단일행 선언·걸침 셀의 저장 쪽 재시작이 함께 증명하는 행 상자.
    /// 편집/재조판과 중첩 개체는 `stored_body_text_cell_frame`에서 제외한다.
    /// 분리 괘선의 간격은 행과 별도 소유자이므로 이 연속 행 상자 계약을 쓰지 않는다.
    pub(crate) fn stored_block_source_rows(
        &self,
        table: &Table,
        range: (usize, usize),
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<Vec<f64>> {
        let (first, end) = range;
        if end <= first + 1
            || table.cell_spacing != 0
            || table.page_break != TablePageBreak::RowBreak
        {
            return None;
        }
        let mut cells = Self::row_block_cells(table, first, end);
        cells.sort_by_key(|cell| (cell.row, cell.col));
        if cells.is_empty()
            || end_cut.len() != cells.len()
            || cells
                .iter()
                .any(|cell| !self.stored_body_text_cell_frame(table, cell))
            || !cells.iter().zip(end_cut).any(|(cell, &unit)| {
                cell.row_span > 1
                    && unit > 0
                    && self.cell_unit_opens_stored_page_frame(cell, table, styles, unit)
            })
        {
            return None;
        }
        (first..end)
            .map(|row| {
                let owners: Vec<_> = cells
                    .iter()
                    .filter(|cell| usize::from(cell.row) == row && cell.row_span == 1)
                    .collect();
                if owners.is_empty() {
                    return None;
                }
                let mut declared = 0_u32;
                for cell in owners {
                    if cell.height == 0 || cell.height > i32::MAX as u32 {
                        return None;
                    }
                    let padding = cell.effective_padding(&table.padding);
                    let mut previous_end = None;
                    for line in cell.paragraphs.iter().flat_map(|para| &para.line_segs) {
                        let top = i64::from(line.vertical_pos);
                        if previous_end.is_some_and(|end| top < end) {
                            return None;
                        }
                        previous_end = Some(top + i64::from(line.line_height));
                    }
                    let ink_end =
                        previous_end? + i64::from(padding.top) + i64::from(padding.bottom);
                    if ink_end > i64::from(cell.height) {
                        return None;
                    }
                    declared = declared.max(cell.height);
                }
                Some(hwpunit_to_px(declared as i32, self.dpi))
            })
            .collect()
    }

    /// 앞 조각이 실제 수용한 높이를 원본 행 순서로 차감한다. 남은 내용/안여백은
    /// 최소 높이보다 우선하며 걸침이 끝나는 행에 부족분을 예약한다.
    pub(crate) fn stored_block_remaining_rows(
        &self,
        table: &Table,
        range: (usize, usize),
        source_rows: &[f64],
        accepted: f64,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Vec<f64> {
        let (first, end) = range;
        let mut debit = accepted;
        let mut rows: Vec<_> = source_rows
            .iter()
            .enumerate()
            .map(|(offset, &source)| {
                let used = debit.min(source).max(0.0);
                debit -= used;
                (source - used).max(self.row_block_cut_row_content_height(
                    table,
                    first,
                    end,
                    first + offset,
                    start_cut,
                    &[],
                    styles,
                ))
            })
            .collect();
        let mut cells = Self::row_block_cells(table, first, end);
        cells.sort_by_key(|cell| (cell.row, cell.col));
        for (index, cell) in cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.row_span > 1)
        {
            let start = usize::from(cell.row) - first;
            let stop = (start + usize::from(cell.row_span)).min(rows.len());
            if start >= stop {
                continue;
            }
            let unit = start_cut.get(index).copied().unwrap_or(0);
            let need = self.cell_cut_visible_height(cell, table, styles, unit, usize::MAX);
            let have = rows[start..stop].iter().sum::<f64>();
            if need > have {
                rows[stop - 1] += need - have;
            }
        }
        rows
    }
}
