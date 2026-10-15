//! 원본 일반 행의 저장 내용 컷과 실제 물리 조각 상자를 함께 조회한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
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
        Some(selected.iter().map(|unit| unit.height).sum::<f64>() - trailing)
    }

    pub(crate) fn stored_ordinary_row_cut_height(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.original_ordinary_row_cells(table, row)?;
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
    fn stored_ordinary_row_cut_has_boundary(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_cut_ends_at_original_plain_text_reset(table, row, start, end, styles)
            || self.row_cut_ends_at_saved_zero_origin_frame(table, row, start, end, styles)
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
        let cells = self.original_ordinary_row_cells(table, row)?;
        if !start.is_empty()
            || end.len() != cells.len()
            || !(self.stored_ordinary_row_cut_has_boundary(table, row, start, end, styles)
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
        (available.is_finite() && available >= content && available < full).then_some(available)
    }

    /// 스캔이 수용한 앞 상자를 차감하고 남은 내용도 포함한 물리 높이를 전달한다.
    pub(crate) fn stored_ordinary_row_remainder(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        opening: f64,
        carried: Option<f64>,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.original_ordinary_row_cells(table, row)?;
        if cut.len() != cells.len()
            || !self.row_cut_remaining_is_single_stored_frame(table, row, cut, None, styles)
            || !(self.stored_ordinary_row_cut_has_boundary(table, row, &[], cut, styles)
                || self.stored_single_line_row_is_fragmentable(table, row, styles)
                || carried.is_some())
        {
            return None;
        }
        // 같은 행을 다시 나누면 이전 조각까지 차감한 커서의 물리 잔여를 쓴다.
        // 원본 전체 높이로 돌아가 이미 소비한 공간을 다시 예약하지 않는다.
        let full = carried.unwrap_or_else(|| {
            cells
                .iter()
                .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
                .fold(0.0, f64::max)
        });
        let end: Vec<_> = cells
            .iter()
            .map(|cell| self.cell_units(cell, table, styles).len())
            .collect();
        let content = self.stored_ordinary_row_cut_height(table, row, cut, &end, styles)?;
        (opening > 0.0 && opening < full).then_some((full - opening).max(content))
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
        self.original_ordinary_row_cells(table, row)
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
                    && (self.stored_ordinary_row_cut_has_boundary(table, row, &[], cut, styles)
                        || self.stored_single_line_row_is_fragmentable(table, row, styles)
                        || self.row_cut_consumes_all_ordinary_units(table, row, cut, styles))
            })
    }
}
