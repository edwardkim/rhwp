//! 원본 일반 행의 저장 내용 컷과 실제 물리 조각 상자를 함께 조회한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    /// 원본 일반 칸의 선언 최소는 내용 컷과 별개의 물리 공간이다.
    /// 편집·재조판 또는 유효하지 않은 저장 줄은 같은 근거로 사용하지 않는다.
    fn stored_independent_row_minimum(
        &self,
        table: &Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        table
            .cells
            .iter()
            .filter(|cell| {
                usize::from(cell.row) == row
                    && cell.row_span == 1
                    && cell.height > 0
                    && cell.height <= i32::MAX as u32
                    && self.stored_plain_text_cell_frame(table, cell)
            })
            .filter_map(|cell| {
                let minimum = hwpunit_to_px(cell.height as i32, self.dpi);
                let end = self.cell_units(cell, table, styles).len();
                let content = self.cell_cut_visible_height(cell, table, styles, 0, end);
                (minimum > content + hwpunit_to_px(2, self.dpi)).then_some(minimum)
            })
            .reduce(f64::max)
    }

    /// 원본 일반 행의 물리 최소는 셀 선언이며, 여러 저장 프레임의 내용 합이 아니다.
    /// 병합 칸의 수용 원장이 따로 필요한 행은 이 일반 칸 계약에 포함하지 않는다.
    pub(crate) fn stored_ordinary_row_uses_declared_minimum(
        &self,
        table: &Table,
        row: usize,
    ) -> bool {
        let owners = Self::row_cut_cell_order(table, row);
        !owners.is_empty()
            && !table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && usize::from(cell.row) <= row
                    && row < usize::from(cell.row) + usize::from(cell.row_span)
            })
            && owners.iter().all(|&index| {
                let cell = &table.cells[index];
                usize::from(cell.row) == row
                    && cell.row_span == 1
                    && self.stored_plain_text_cell_frame(table, cell)
            })
    }

    fn original_ordinary_row_cells<'a>(
        &self,
        table: &'a Table,
        row: usize,
    ) -> Option<Vec<&'a Cell>> {
        if !self.stored_row_boxes_exceed_first_frame(table)
            || table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && usize::from(cell.row) <= row
                    && row < usize::from(cell.row) + usize::from(cell.row_span)
            })
        {
            return None;
        }
        let mut cells: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) == row)
            .collect();
        cells.sort_by_key(|cell| cell.col);
        (!cells.is_empty()
            && cells.iter().all(|cell| {
                cell.row_span == 1
                    && cell.height <= i32::MAX as u32
                    && self.stored_plain_text_cell_frame(table, cell)
            }))
        .then_some(cells)
    }

    /// 일반 칸 자신의 선언 최소는 다른 열의 병합 프레임과 별도다.
    /// 내용 컷을 정하는 저장 행 프레임과 혼동하지 않도록, 여기서는
    /// 모든 일반 내용이 끝나고도 최소 공간이 남는 소유자만 조회한다.
    fn completed_ordinary_minimum_cells<'a>(
        &self,
        table: &'a Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<Vec<&'a Cell>> {
        if !self.stored_row_boxes_exceed_first_frame(table) {
            return None;
        }
        let cells: Vec<_> = Self::row_cut_cell_order(table, row)
            .into_iter()
            .filter_map(|index| table.cells.get(index))
            .collect();
        if cells.is_empty()
            || cells.len() != cut.len()
            || !cells.iter().zip(cut).all(|(cell, &end)| {
                cell.height <= i32::MAX as u32
                    && self.stored_plain_text_cell_frame(table, cell)
                    && end == self.cell_units(cell, table, styles).len()
            })
        {
            return None;
        }
        let minimum = cells
            .iter()
            .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
            .fold(0.0_f64, f64::max);
        (minimum > self.row_complete_cut_content_height(table, row, styles)).then_some(cells)
    }

    /// 미완료 컷의 원본 프레임과 전내용을 소비한 뒤의 독립 최소를 같은 소비자에게 전달한다.
    fn ordinary_row_cells_for_cut<'a>(
        &self,
        table: &'a Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<Vec<&'a Cell>> {
        self.original_ordinary_row_cells(table, row)
            .or_else(|| self.completed_ordinary_minimum_cells(table, row, cut, styles))
    }

    /// 한 글줄은 쪼개지 않되, 원본 최소 높이의 빈 부분은 별도 물리 조각일 수 있다.
    pub(crate) fn stored_single_line_row_is_fragmentable(
        &self,
        table: &Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.original_ordinary_row_cells(table, row)
            .is_some_and(|cells| {
                let cut: Vec<_> = cells
                    .iter()
                    .map(|cell| self.cell_units(cell, table, styles).len())
                    .collect();
                self.stored_ordinary_row_cut_height(table, row, &[], &cut, styles)
                    .is_some_and(|content| {
                        cells.iter().all(|cell| {
                            let units = self.cell_units(cell, table, styles);
                            units.len() == 1
                                && units[0].vis_end == units[0].vis_start + 1
                                && hwpunit_to_px(cell.height as i32, self.dpi) > content
                        })
                    })
            })
    }

    /// 다음 줄로의 전진과 이 조각의 잉크 점유를 같은 원본 유닛에서 구분한다.
    pub(in crate::renderer::layout) fn stored_ordinary_cell_cut_content_height(
        &self,
        table: &Table,
        cell: &Cell,
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if !self.stored_plain_text_cell_frame(table, cell) {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let selected = units.get(start..end)?;
        let Some(last) = selected.last() else {
            return Some(0.0);
        };
        let line = cell.paragraphs.get(last.para_idx).and_then(|paragraph| {
            if last.empty_spacer {
                paragraph.line_segs.last()
            } else {
                paragraph.line_segs.get(last.vis_end.checked_sub(1)?)
            }
        })?;
        let advance =
            hwpunit_to_px(line.line_height, self.dpi) + hwpunit_to_px(line.line_spacing, self.dpi);
        // 유닛에 실제 들어 있는 저장 다음 줄 간격만 제외한다. 빈 spacer나
        // 문단 여백이 다른 높이를 소유하면 그 점유를 줄 간격으로 대체하지 않는다.
        let trailing = if (last.height - advance).abs() <= hwpunit_to_px(2, self.dpi) {
            hwpunit_to_px(line.line_spacing, self.dpi)
        } else {
            0.0
        };
        // 줄 전진에는 문단 앞 간격도 들어갈 수 있다. last.height와 줄간격의
        // 단순 비교로 저장 쪽 끝의 전진을 다시 세지 않고, 컷 선택과 같은
        // 생산자의 종료 간격·보조 spacer 범위를 정렬 높이에도 소비한다.
        let trim = self.physical_cut_trailing_trim(table, cell, &units, start, end, styles);
        let range = self.cell_cut_flow_range(cell, table, &units, start, end, styles);
        Some(
            (units[range].iter().map(|unit| unit.height).sum::<f64>() - trailing.max(trim))
                .max(0.0),
        )
    }

    pub(crate) fn stored_ordinary_row_cut_height(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.ordinary_row_cells_for_cut(table, row, end, styles)?;
        if end.len() != cells.len() {
            return None;
        }
        cells
            .iter()
            .enumerate()
            .try_fold(0.0_f64, |height, (index, cell)| {
                let from = start.get(index).copied().unwrap_or(0);
                let to = end[index];
                let content =
                    self.stored_ordinary_cell_cut_content_height(table, cell, from, to, styles)?;
                let padding = cell.effective_padding(&table.padding);
                let physical = if to > from {
                    content
                        + hwpunit_to_px(
                            i32::from(padding.top) + i32::from(padding.bottom),
                            self.dpi,
                        )
                } else {
                    0.0
                };
                Some(height.max(physical))
            })
    }

    // 문단 간 경계와 문단 내부의 원본 되감김·원점0 경계는 같은 유닛 컷을 소유한다.
    // 이미 출처를 확인한 생산 술어를 재사용하며 문단 간 리셋으로만 소비를 제한하지 않는다.
    fn stored_ordinary_row_cut_has_physical_frame(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_cut_ends_at_original_plain_text_reset(table, row, start, end, styles)
            || self.row_cut_ends_at_saved_zero_origin_frame(table, row, start, end, styles)
            // 실제 용량에서 나눈 진행 줄도 독립 선언 최소를 소유한다.
            // 내용이 앞뒤에 남는 컷만 해당하며, 완료한 원본에 빈 꼬리를 만들지 않는다.
            || (end
                .iter()
                .enumerate()
                .any(|(index, &to)| to > start.get(index).copied().unwrap_or(0))
                && !self.row_cut_consumes_all_ordinary_units(table, row, end, styles)
                && self
                    .stored_independent_row_minimum(table, row, styles)
                    .is_some())
    }

    /// 저장 쪽 경계에서 선택한 유닛을 실제 마지막 행 상자 안에 놓는다.
    pub(crate) fn stored_ordinary_row_end_frame(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.ordinary_row_cells_for_cut(table, row, end, styles)?;
        if !start.is_empty()
            || end.len() != cells.len()
            || !(self.stored_ordinary_row_cut_has_physical_frame(table, row, start, end, styles)
                || self.row_cut_consumes_all_ordinary_units(table, row, end, styles))
            || !self.row_cut_remaining_is_single_stored_frame(table, row, end, None, styles)
        {
            return None;
        }
        let full = cells
            .iter()
            .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
            .fold(0.0, f64::max);
        let content = self.stored_ordinary_row_cut_height(table, row, start, end, styles)?;
        // 내용이 든 첫 조각은 원본 최소가 들어가면 그 높이에서 닫는다.
        // 남은 쪽 공간이 선언보다 큰 경우에도 내용 높이로만 축소하지 않는다.
        (available.is_finite() && available >= content).then_some(full.min(available).max(content))
    }

    /// 내용 완료 행도 수용 상자가 독립 최소보다 작으면 같은 행의 물리 컷이다.
    /// 완전 소비 유닛을 명시해 다음 조각이 내용 대신 남은 빈 상자만 이어받는다.
    /// 줄 전진 또는 쪽 끝에서 흡수하는 작은 꼬리는 기존 잔여 계약이 제외한다.
    pub(crate) fn stored_complete_row_physical_tail_cut(
        &self,
        table: &Table,
        row: usize,
        accepted: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<Vec<usize>> {
        let cut: Vec<_> = Self::row_cut_cell_order(table, row)
            .into_iter()
            .map(|index| self.cell_units(&table.cells[index], table, styles).len())
            .collect();
        let remaining =
            self.stored_ordinary_row_remainder(table, row, &cut, accepted, None, styles)?;
        (!self.stored_complete_row_tail_is_line_advance(table, row, &cut, remaining, styles))
            .then_some(cut)
    }

    /// 같은 원본 또는 이어받은 상자에서 이미 수용한 공간을 한 번만 차감한다.
    pub(crate) fn stored_ordinary_row_remainder(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        accepted: f64,
        carried: Option<f64>,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.ordinary_row_cells_for_cut(table, row, cut, styles)?;
        if cut.len() != cells.len()
            || !self.row_cut_remaining_is_single_stored_frame(table, row, cut, None, styles)
            || !(self.stored_ordinary_row_cut_has_physical_frame(table, row, &[], cut, styles)
                || self.stored_single_line_row_is_fragmentable(table, row, styles)
                || carried.is_some()
                || self
                    .completed_ordinary_minimum_cells(table, row, cut, styles)
                    .is_some())
        {
            return None;
        }
        let declared = cells
            .iter()
            .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
            .fold(0.0, f64::max);
        let end: Vec<_> = cells
            .iter()
            .map(|cell| self.cell_units(cell, table, styles).len())
            .collect();
        if let Some(height) = self.stored_ordinary_closing_text_frame(table, row, cut, styles) {
            // 내용 합보다 큰 저장 높이도 앞 쪽에서 접힌 간격을 담을 수 있다.
            // 원본 Common 상자가 첫 내용 컷의 물리 공간을 함께 증언한
            // 경우에만 선언 최소의 잔여를 유지한다. 그 근거가 없는 후속
            // 원본 행의 마지막 프레임을 전체 전진 합의 잔여로 늘리지 않는다.
            let content = self.row_cut_content_height(table, row, &[], &end, styles);
            let source_minimum = cells
                .iter()
                .any(|cell| self.first_source_frame_owner(table, cell, styles).is_some());
            let minimum = if declared > content && source_minimum {
                (carried.unwrap_or(declared) - accepted).max(0.0)
            } else {
                0.0
            };
            #[cfg(not(target_arch = "wasm32"))]
            if std::env::var_os("RHWP_DIAG_ROW_TAIL").is_some() {
                eprintln!(
                    "DIAG_ROW_TAIL stored row={row} declared={declared:.6} content={content:.6} frame={height:.6} accepted={accepted:.6} carried={carried:?} minimum={minimum:.6} cut={cut:?}"
                );
            }
            return Some(height.max(minimum));
        }
        // 같은 행을 다시 나누면 이전 조각까지 차감한 커서의 물리 잔여를 쓴다.
        // 원본 전체 높이로 돌아가 이미 소비한 공간을 다시 예약하지 않는다.
        let full = carried.unwrap_or(declared);
        let content = self.stored_ordinary_row_cut_height(table, row, cut, &end, styles)?;
        let remaining = (full - accepted).max(content);
        // 저장 여부와 무관하게 이미 끝난 내용 뒤의 한 줄 이하 빈 꼬리는
        // 새 행 프레임을 열지 않는다. 큰 독립 최소와 미소비 내용은 보존한다.
        if self.row_cut_consumes_all_ordinary_units(table, row, cut, styles)
            && self.row_blank_tail_is_line_sliver(table, row, remaining, styles)
        {
            return None;
        }
        (accepted > 0.0 && accepted < full).then_some(remaining)
    }

    /// 미완료 칸들이 여는 마지막 원본 텍스트 프레임의 물리 끝이다.
    /// 전체 내용 전진 합에는 앞 쪽 경계에서 접힌 간격도 들어 있으므로,
    /// 같은 합의 잔여를 다시 예약하지 않고 저장 원점과 줄 끝을 소비한다.
    fn stored_ordinary_closing_text_frame(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.original_ordinary_row_cells(table, row)?;
        if cells.len() != cut.len() {
            return None;
        }
        let mut owners = 0;
        let mut height = 0.0_f64;
        for (cell, &start) in cells.iter().zip(cut) {
            let units = self.cell_units(cell, table, styles);
            if start == units.len() {
                continue;
            }
            let first = units.get(start)?;
            let paragraph = cell.paragraphs.get(first.para_idx)?;
            let paragraph_restart = first.vis_start == 0
                && first
                    .para_idx
                    .checked_sub(1)
                    .and_then(|index| cell.paragraphs.get(index))
                    .is_some_and(|previous| {
                        self.stored_plain_paragraph_restart(cell, table, previous, paragraph)
                    });
            // 다른 칸이 완료돼도 원본 문단 내부의 다음 쪽 줄은 같은
            // 마지막 프레임을 연다. 전체 줄 전진 합으로 그 상자를 늘리지 않는다.
            if !paragraph_restart
                && !Self::stored_in_paragraph_unit_is_original_restart(cell, first)
            {
                return None;
            }
            let range = self.cell_cut_flow_range(cell, table, &units, start, units.len(), styles);
            let mut previous_top = 0;
            let mut bottom = 0;
            for unit in &units[range] {
                let para = cell.paragraphs.get(unit.para_idx)?;
                for line in para.line_segs.get(unit.vis_start..unit.vis_end)? {
                    if line.vertical_pos < previous_top {
                        return None;
                    }
                    previous_top = line.vertical_pos;
                    bottom = bottom.max(line.vertical_pos.checked_add(line.line_height)?);
                }
            }
            let padding = cell.effective_padding(&table.padding);
            let physical = bottom
                .checked_add(i32::from(padding.top))?
                .checked_add(i32::from(padding.bottom))?;
            height = height.max(hwpunit_to_px(physical, self.dpi));
            owners += 1;
        }
        (owners > 0).then_some(height)
    }

    /// 종료 줄의 다음 줄 간격만 남은 컷은 새 내용·빈 밴드 소유자를 만들지 않는다.
    pub(crate) fn stored_complete_row_tail_is_line_advance(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        tail: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let Some(cells) = self.original_ordinary_row_cells(table, row) else {
            return false;
        };
        if cells.len() != cut.len() || tail <= 0.0 {
            return false;
        }
        cells.iter().zip(cut).all(|(cell, &end)| {
            let units = self.cell_units(cell, table, styles);
            let Some(last) = units.last().filter(|_| end == units.len()) else {
                return false;
            };
            cell.paragraphs
                .get(last.para_idx)
                .and_then(|paragraph| {
                    if last.empty_spacer {
                        paragraph.line_segs.last()
                    } else {
                        paragraph.line_segs.get(last.vis_end.checked_sub(1)?)
                    }
                })
                .is_some_and(|line| {
                    line.line_spacing > 0
                        && tail
                            <= hwpunit_to_px(line.line_spacing, self.dpi)
                                + hwpunit_to_px(2, self.dpi)
                })
        })
    }

    /// 실제 전달된 조각 높이 안에서 원본 정렬을 적용한다. 좌표를 다시 추측하지 않는다.
    pub(crate) fn stored_ordinary_row_frame_owns_alignment(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.ordinary_row_cells_for_cut(table, row, cut, styles)
            .is_some_and(|cells| {
                if cut.len() != cells.len() {
                    return false;
                }
                // 문단 내부의 실제 쪽 재시작은 새 쪽의 원점0을 이미 소유한다.
                // 내용 컷의 마지막 줄간격을 뺀 잉크 높이로 다시 가운데 정렬하면
                // 저장 원점을 밀어낸다. 모든 미완료 셀이 같은 재시작을 공유할 때
                // 해당 쪽 원점을 보존하고, 최소 높이만 이어받는 빈 상자는 유지한다.
                let mut has_page_owner = false;
                let page_origin_is_shared = cells.iter().zip(cut).all(|(cell, &start)| {
                    let units = self.cell_units(cell, table, styles);
                    units.get(start).is_none_or(|unit| {
                        let owns = Self::stored_in_paragraph_unit_is_original_restart(cell, unit);
                        has_page_owner |= owns;
                        owns
                    })
                });
                !(page_origin_is_shared && has_page_owner)
                    && (self.stored_ordinary_row_cut_has_physical_frame(
                        table,
                        row,
                        &[],
                        cut,
                        styles,
                    ) || self.stored_single_line_row_is_fragmentable(table, row, styles)
                        || self.row_cut_consumes_all_ordinary_units(table, row, cut, styles))
            })
    }
}
