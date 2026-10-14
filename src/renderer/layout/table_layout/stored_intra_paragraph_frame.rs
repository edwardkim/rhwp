//! The opening table frame and a later source cell use different vertical origins.

use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 문단 안의 원본 재시작으로 잘린 텍스트는 계속되는 같은 글줄 흐름이다.
    /// 물리 프레임의 남는 공간을 현재 가시 접두부의 가운데 정렬 띠로 옮기지 않는다.
    pub(in crate::renderer::layout) fn original_in_paragraph_cut_owns_flow(
        &self,
        table: &Table,
        cell: &Cell,
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let profile = self.profile.get();
        if cell.row_span != 1
            || profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return false;
        }
        let units = self.cell_units(cell, table, styles);
        [start, end].into_iter().any(|index| {
            units
                .get(index)
                .is_some_and(|unit| Self::stored_in_paragraph_unit_is_original_restart(cell, unit))
        })
    }

    /// 원본 문단 안의 저장 재시작은 선언 높이가 있는 중첩 행도 분할한다.
    /// 문단마다 시작하는 로컬 좌표와 편집 후 만든 줄 정보는 이 근거가 아니다.
    pub(super) fn stored_nested_row_has_in_paragraph_cut(
        &self,
        table: &Table,
        cells: &[&Cell],
        units: &[std::sync::Arc<Vec<CellUnit>>],
    ) -> bool {
        let profile = self.profile.get();
        if profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return false;
        }
        cells.iter().zip(units).any(|(cell, units)| {
            units
                .iter()
                .any(|unit| Self::stored_in_paragraph_unit_is_original_restart(cell, unit))
        })
    }

    /// 행 분해와 상위 투영은 동일한 원본 줄의 재시작을 판별한다.
    pub(super) fn stored_in_paragraph_unit_is_original_restart(
        cell: &Cell,
        unit: &CellUnit,
    ) -> bool {
        if cell.dirty_flag
            || !unit.page_frame_reset_before
            || unit.vis_start == 0
            || unit.vis_end <= unit.vis_start
        {
            return false;
        }
        let Some(paragraph) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        if paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || !paragraph.controls.is_empty()
        {
            return false;
        }
        let Some(previous) = paragraph.line_segs.get(unit.vis_start - 1) else {
            return false;
        };
        let Some(current) = paragraph.line_segs.get(unit.vis_start) else {
            return false;
        };
        let synthetic = crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        previous.tag & synthetic == 0
            && current.tag & synthetic == 0
            && previous.vertical_pos > 0
            && current.vertical_pos == 0
            && previous.line_height > 0
            && current.line_height > 0
            && current.text_start > previous.text_start
    }

    /// Normalize the original opening box into the ordinary source row. This
    /// only supplies the existing saved-cut tail calculation; accepted physical
    /// rows and the table's occupied extent remain in their original ledger.
    pub(super) fn stored_first_cell_frame_height(&self, table: &Table, cell: &Cell) -> Option<f64> {
        (cell.row_span == 1)
            .then(|| self.source_first_cell_frame_height(table, cell))
            .flatten()
    }

    fn source_first_cell_frame_height(&self, table: &Table, cell: &Cell) -> Option<f64> {
        if cell.row == 0
            || !self.stored_plain_text_cell_frame(table, cell)
            || table.common.height == 0
            || table.common.height > i32::MAX as u32
        {
            return None;
        }
        let rows = table.get_raw_row_heights();
        let row = usize::from(cell.row);
        let prefix = rows.get(..row)?;
        let span_end = row.checked_add(usize::from(cell.row_span))?;
        let source_span = rows.get(row..span_end)?;
        if prefix
            .iter()
            .chain(source_span)
            .any(|&height| height == 0 || height > i32::MAX as u32)
        {
            return None;
        }
        let (origin, current) = self
            .stored_frame_row_plans
            .borrow()
            .get(&(table as *const Table as usize))
            .and_then(|rows| {
                Some((
                    rows.get(..row)?.iter().sum::<f64>(),
                    rows.get(row..span_end)?.iter().sum::<f64>(),
                ))
            })
            .unwrap_or_else(|| {
                (
                    prefix
                        .iter()
                        .map(|&height| hwpunit_to_px(height as i32, self.dpi))
                        .sum(),
                    source_span
                        .iter()
                        .map(|&height| hwpunit_to_px(height as i32, self.dpi))
                        .sum(),
                )
            });
        let origin = origin + hwpunit_to_px(table.cell_spacing as i32, self.dpi) * row as f64;
        let current = current
            + hwpunit_to_px(table.cell_spacing as i32, self.dpi)
                * usize::from(cell.row_span).saturating_sub(1) as f64;
        let end = hwpunit_to_px(table.common.height as i32, self.dpi);
        // A completed opening row or a later frame cannot authenticate this
        // partial source row. Unknown zero-height rows are not a physical origin.
        if !origin.is_finite() || !current.is_finite() || end <= origin || end >= origin + current {
            return None;
        }
        Some(end - origin)
    }

    pub(super) fn first_source_frame_owner(
        &self,
        table: &Table,
        cell: &Cell,
        styles: &ResolvedStyleSet,
    ) -> Option<(usize, f64)> {
        let opening = self.source_first_cell_frame_height(table, cell)?;
        let units = self.cell_units(cell, table, styles);
        // 저장 상자는 실제 컷과 같은 유효 안여백으로 인증한다. 선언 셀 높이로
        // 축소된 여백 대신 원본 여백을 더하면 현재 배치에 없는 물리 띠가 생긴다.
        let (_, _, top, bottom) = self.resolve_cell_padding(cell, table);
        let padding = top + bottom;
        let epsilon = hwpunit_to_px(2, self.dpi);
        let source_padding = cell.effective_padding(&table.padding);
        let source_padding = hwpunit_to_px(
            i32::from(source_padding.top) + i32::from(source_padding.bottom),
            self.dpi,
        );
        // 원본 Common 상자의 좌표는 저장 안여백을 포함한다. 실제 컷에서 그
        // 여백을 축소했다면 원본 상자를 같은 좌표의 물리 프레임으로 재사용할 수 없다.
        if (source_padding - padding).abs() > epsilon {
            return None;
        }
        let mut previous: Option<&crate::model::paragraph::LineSeg> = None;
        let mut advance = 0.0;
        for (index, unit) in units.iter().enumerate() {
            if unit.vis_end != unit.vis_start + 1 {
                return None;
            }
            let line = cell
                .paragraphs
                .get(unit.para_idx)?
                .line_segs
                .get(unit.vis_start)?;
            if let Some(before) = previous {
                if line.vertical_pos == 0 && before.vertical_pos > 0 {
                    let ink = hwpunit_to_px(
                        before.vertical_pos.checked_add(before.line_height)?,
                        self.dpi,
                    ) + padding;
                    // A stored empty line owns the same line box and following
                    // advance as text. The opening box ends in that final gap.
                    // A span can cross complete rows before the first partial
                    // row. Its source restart still owns the same Common frame
                    // when an independent ordinary cell closes that frame.
                    let independently_closed_span = cell.row_span > 1
                        && unit.stored_frame_break_before
                        && unit.page_frame_reset_before
                        && opening <= hwpunit_to_px(cell.height as i32, self.dpi)
                        && table.cells.iter().any(|peer| {
                            peer.row_span == 1
                                && peer.row >= cell.row
                                && usize::from(peer.row)
                                    < usize::from(cell.row) + usize::from(cell.row_span)
                                && peer.col != cell.col
                                && self.first_source_frame_owner(table, peer, styles).is_some()
                        });
                    // 같은 여백 좌표에서 인증한 Common 상자는 내용 전진 뒤의 빈
                    // 물리 공간도 소유할 수 있다. 같은 문단의 실제 쪽 프레임 재시작을
                    // 요구하며, 그 공간을 줄 전진의 상한으로 다시 잘라내지 않는다.
                    let ordinary_source_restart = cell.row_span == 1
                        && unit.page_frame_reset_before
                        && units[index - 1].para_idx == unit.para_idx;
                    let accepted = opening + epsilon >= ink
                        && (opening <= advance + padding + epsilon
                            || independently_closed_span
                            || ordinary_source_restart);
                    return accepted.then_some((index, opening));
                }
                if i64::from(line.vertical_pos)
                    != i64::from(before.vertical_pos)
                        + i64::from(before.line_height)
                        + i64::from(before.line_spacing)
                {
                    return None;
                }
            } else {
                // A real paragraph's leading spacing is part of its first
                // line origin. Authenticate it against the independent style
                // instead of treating every positive origin as stale alignment.
                let before = styles
                    .para_styles
                    .get(cell.paragraphs.get(unit.para_idx)?.para_shape_id as usize)?
                    .spacing_before;
                let origin = hwpunit_to_px(line.vertical_pos, self.dpi);
                if origin < 0.0 || (origin != 0.0 && (origin - before).abs() > epsilon) {
                    return None;
                }
            }
            let source_advance =
                hwpunit_to_px(line.line_height.checked_add(line.line_spacing)?, self.dpi)
                    + if previous.is_none() {
                        hwpunit_to_px(line.vertical_pos, self.dpi)
                    } else {
                        0.0
                    };
            if (unit.height - source_advance).abs() > epsilon {
                return None;
            }
            advance += unit.height;
            previous = Some(line);
        }
        None
    }

    /// A partial opening row has one physical height even when a sibling has
    /// finished. Every unfinished owner must restart at this same source cut.
    pub(crate) fn row_cut_first_source_frame_height(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if !start_cut.is_empty() || end_cut.is_empty() {
            return None;
        }
        let cells = Self::row_cut_cell_order(table, row);
        if cells.len() != end_cut.len() {
            return None;
        }
        let epsilon = hwpunit_to_px(2, self.dpi);
        let mut frame: Option<f64> = None;
        let mut completed_height = 0.0f64;
        for (&index, &end) in cells.iter().zip(end_cut) {
            let cell = &table.cells[index];
            if !self.stored_plain_text_cell_frame(table, cell) {
                return None;
            }
            let units = self.cell_units(cell, table, styles);
            if units.is_empty() || end == 0 || end > units.len() {
                return None;
            }
            if end < units.len() {
                let (owner, opening) = self.first_source_frame_owner(table, cell, styles)?;
                if owner != end || frame.is_some_and(|height| (height - opening).abs() > epsilon) {
                    return None;
                }
                frame = Some(opening);
            } else {
                let (_, _, top, bottom) = self.resolve_cell_padding(cell, table);
                let height = units.iter().map(|unit| unit.height).sum::<f64>() + top + bottom;
                completed_height = completed_height.max(height);
            }
        }
        frame.filter(|height| completed_height <= *height)
    }

    pub(super) fn stored_span_partial_first_frame_owner(
        &self,
        table: &Table,
        cell: &Cell,
        row: usize,
        cut: &[usize],
        measured_rows: &[f64],
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        if cell.row_span <= 1 || usize::from(cell.row) != row || cut.is_empty() {
            return None;
        }
        let (owner, opening) = self.first_source_frame_owner(table, cell, styles)?;
        let epsilon = hwpunit_to_px(2, self.dpi);
        let origin = measured_rows.get(..row)?.iter().sum::<f64>()
            + hwpunit_to_px(table.cell_spacing as i32, self.dpi) * row as f64;
        if (origin + opening - hwpunit_to_px(table.common.height as i32, self.dpi)).abs() > epsilon
            || (self.row_cut_content_height(table, row, &[], cut, styles) - opening).abs() > epsilon
        {
            return None;
        }
        table
            .cells
            .iter()
            .any(|peer| {
                peer.row == cell.row
                    && peer.col != cell.col
                    && self
                        .first_source_frame_owner(table, peer, styles)
                        .is_some_and(|(_, height)| (height - opening).abs() <= epsilon)
            })
            .then_some(owner)
    }

    /// A source cut consumes the opening physical box, not the row's entire
    /// trailing band. The next fragment reserves and paints this same remainder.
    pub(crate) fn stored_first_frame_row_remainder(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        measured_rows: &[f64],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let mut cells: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) == row && cell.row_span == 1)
            .collect();
        cells.sort_by_key(|cell| cell.col);
        if cells.len() != cut.len() || row >= measured_rows.len() {
            return None;
        }
        let epsilon = hwpunit_to_px(2, self.dpi);
        let mut opening: Option<f64> = None;
        for (cell, &end) in cells.into_iter().zip(cut) {
            let units = self.cell_units(cell, table, styles);
            if end >= units.len() {
                continue;
            }
            if end == 0 {
                return None;
            }
            let height = self.stored_first_cell_frame_height(table, cell)?;
            let before = units[..end - 1].iter().map(|unit| unit.height).sum();
            if self.native_intra_para_saved_reset_trailing_trim(
                table,
                cell,
                &units,
                end,
                before,
                units[end - 1].height,
                styles,
            ) <= 0.0
                || opening.is_some_and(|other| (other - height).abs() > epsilon)
            {
                return None;
            }
            opening = Some(height);
        }
        let opening = opening?;
        let origin = measured_rows[..row].iter().sum::<f64>()
            + hwpunit_to_px(table.cell_spacing as i32, self.dpi) * row as f64;
        let source_end = hwpunit_to_px(table.common.height as i32, self.dpi);
        let accepted = self.row_cut_content_height(table, row, &[], cut, styles);
        // Source and measured coordinates must identify the same opening box.
        // A grown prefix cannot be replaced with raw row minima to certify it.
        if (origin + opening - source_end).abs() > epsilon || (accepted - opening).abs() > epsilon {
            return None;
        }
        let remaining = measured_rows[row] - opening;
        let content = self.row_cut_content_height(table, row, cut, &[], styles);
        (remaining > content + epsilon).then_some(remaining)
    }
}
