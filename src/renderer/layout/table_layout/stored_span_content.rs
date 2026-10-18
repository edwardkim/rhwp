//! Stored text spans distinguish a line's occupied box from its next-line advance.

use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, TablePageBreak};
use crate::renderer::hwpunit_to_px;

use super::{unit_diagnostics, CellUnit, LayoutEngine, ResolvedStyleSet};

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

    /// RowBreak/CellBreak의 경계 rowspan 셀이 소유하는 유닛 범위.
    /// 높이 예약과 실제 셀 배치가 시작 컷 및 native 저장 문단 owner를 함께 사용한다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn rowbreak_straddle_cut_units(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        start_row: usize,
        end_row: usize,
        start_cut: &[usize],
        start_cut_is_block: bool,
        start_row_height_override: Option<f64>,
        end_cut: &[usize],
        end_cut_is_block: bool,
        end_row_height_override: Option<f64>,
        cell_height: f64,
        resolved_row_heights: &[f64],
        straddle_page_bands: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &ResolvedStyleSet,
    ) -> (usize, usize) {
        let cell_row = cell.row as usize;
        let cell_end = cell_row + cell.row_span as usize;
        // [#7226] 앞 행에서 걸쳐 온 칸뿐 아니라, 앞 조각이 **이 행 안에서** 멈춰
        // 빈 컷으로 재개한 걸침 전용 행의 칸도 이미 일부 소비된 상태다.
        let straddles_start = (cell_row < start_row && cell_end > start_row)
            || crate::renderer::layout::table_partial::resumes_inside_own_start_row(
                table,
                cell,
                start_row,
                start_cut,
                start_cut_is_block,
                start_row_height_override,
            );
        let straddles_end = crate::renderer::layout::table_partial::cell_crosses_fragment_end(
            cell, end_row, end_cut,
        )
                || (cell_row < end_row && cell_row + usize::from(cell.row_span) == end_row
                    && end_row_height_override.is_some()
                    && self.cell_uses_plain_reflow_units(cell, table));
        // HWP5 저장 pagination의 2행/2문단 계약에서는 문단 하나가 행 하나의 owner다.
        if start_cut.is_empty()
            && start_row_height_override.is_none()
            && end_cut.is_empty()
            && ((straddles_start && start_row == cell_row + 1)
                || (straddles_end && end_row == cell_row + 1))
            && self.native_two_row_rowspan_paragraph_owner_boundary(cell, table, styles)
        {
            return (
                usize::from(straddles_start),
                if straddles_end { 1 } else { usize::MAX },
            );
        }
        let stored_opening_owner = self
            .stored_span_opening_frame(table, cell_row)
            .is_some_and(|frame| frame.restart_columns.contains(&cell.col));
        // 시작 컷은 start_row가 소유한다. 그보다 앞 행의 이어받기는
        // 같은 조각의 뒤 행에서 시작하는 이 셀의 내용을 소비하지 않는다.
        if stored_opening_owner
            && straddles_end
            && end_row == cell_row + 1
            && end_cut.is_empty()
            && (start_row < cell_row
                || (start_row == cell_row
                    && start_cut.is_empty()
                    && start_row_height_override.is_none()))
        {
            return (0, 1);
        }
        let resumes_stored_opening = stored_opening_owner
            && straddles_start
            && start_row == cell_row + 1
            && start_cut.is_empty()
            && start_row_height_override.is_none();
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let padding = cell.effective_padding(&table.padding);
        let pad_top = hwpunit_to_px(padding.top as i32, self.dpi);
        let mut prior_h = 0.0;
        if straddles_start {
            let stored_physical_span = self.stored_plain_text_span_frame(table, cell);
            for r in cell_row..start_row {
                prior_h += self.rowbreak_prior_row_height(
                    table,
                    r,
                    resolved_row_heights,
                    styles,
                    stored_physical_span,
                );
                prior_h += cell_spacing;
            }
            if let Some(remaining_band) = start_row_height_override {
                // 내용 컷으로는 이미 소비한 물리 빈 밴드를 알 수 없다. 앞 조각이
                // 남긴 정확한 행 높이로 소비 구간을 복원해 앞 조각 eu와 이어준다.
                let physical_prefix = (resolved_row_heights.get(start_row).copied().unwrap_or(0.0)
                    - remaining_band)
                    .max(0.0);
                // 저장 행 높이는 전체가 아닌 첫 프레임만 나타낼 수 있다.
                // 수용한 RowCut의 내용·여백과 그보다 큰 빈 물리 띠를 보존한다.
                let accepted_cut_prefix = if !start_cut_is_block && !start_cut.is_empty() {
                    self.row_cut_content_height(table, start_row, &[], start_cut, styles)
                } else {
                    0.0
                };
                prior_h += physical_prefix.max(accepted_cut_prefix);
            } else if !start_cut_is_block && !start_cut.is_empty() {
                prior_h += self.row_cut_content_height(table, start_row, &[], start_cut, styles);
            }
        }
        let whole_source_rows =
            start_cut.is_empty() && start_row_height_override.is_none() && end_cut.is_empty();
        let saved_start_owner = whole_source_rows
            .then(|| self.stored_span_row_boundary_owner(table, cell, start_row, styles))
            .flatten()
            .or_else(|| {
                (!start_cut_is_block)
                    .then(|| {
                        self.stored_span_partial_first_frame_owner(
                            table,
                            cell,
                            start_row,
                            start_cut,
                            resolved_row_heights,
                            styles,
                        )
                    })
                    .flatten()
            })
            .or_else(|| {
                (!start_cut_is_block)
                    .then(|| {
                        self.stored_shared_span_cut_owner(table, cell, start_row, start_cut, styles)
                    })
                    .flatten()
            });
        if straddles_start {
            if let Some(index) = table
                .cells
                .iter()
                .position(|owner| std::ptr::eq(owner, cell))
            {
                prior_h += straddle_page_bands
                    .iter()
                    .find(|carry| carry.cell_index == index)
                    .map_or(0.0, |carry| carry.physical_height);
            }
        }
        let accepted_start = table
            .cells
            .iter()
            .position(|owner| std::ptr::eq(owner, cell))
            .and_then(|index| {
                straddle_page_bands
                    .iter()
                    .find(|carry| carry.cell_index == index)
            })
            .and_then(|carry| carry.next_unit);
        let su = accepted_start.unwrap_or_else(|| {
            if resumes_stored_opening {
                // 앞 조각이 소비한 첫 원본 유닛을 이어받는다. 실제 carry 컷이 우선한다.
                1
            } else if let Some(owner) = saved_start_owner {
                owner
            } else if prior_h > 0.0 {
                let fit = self.cell_units_fitting_height(cell, table, styles, prior_h - pad_top);
                self.snap_units_to_stored_cell_reset(cell, table, styles, 0, fit)
            } else {
                0
            }
        });
        let eu = if straddles_end {
            // 종료 RowCut은 경계 행의 일반 셀만 소유한다. 걸침 셀은 이미 수용한
            // 자기 물리 상자로 유닛을 선택하고, 그 컷을 다음 조각에 그대로 전달한다.
            let owner = whole_source_rows
                .then(|| self.stored_span_row_boundary_owner(table, cell, end_row, styles))
                .flatten()
                .or_else(|| {
                    (!end_cut_is_block)
                        .then(|| {
                            self.stored_span_partial_first_frame_owner(
                                table,
                                cell,
                                end_row.saturating_sub(1),
                                end_cut,
                                resolved_row_heights,
                                styles,
                            )
                        })
                        .flatten()
                })
                .or_else(|| {
                    (!end_cut_is_block)
                        .then(|| {
                            self.stored_shared_span_cut_owner(
                                table,
                                cell,
                                end_row.saturating_sub(1),
                                end_cut,
                                styles,
                            )
                        })
                        .flatten()
                });
            owner
                .unwrap_or_else(|| {
                    let accepted_height = cell_height.max(0.0);
                    // 실제 수용 커서에서 예산을 다시 시작한다. 저장 되감김으로
                    // 커서가 바뀌면 원래 행의 누적 높이는 이 유닛들의 높이가 아니다.
                    let units = self.cell_units(cell, table, styles);
                    let mut fit = su.min(units.len());
                    let frame = super::row_block_cut::CellCutFrame::new(
                        self, cell, table, styles, &units, fit,
                    );
                    let mut advance = 0.0;
                    while fit < units.len() {
                        let next = advance + units[fit].height;
                        if frame.fitting_height(fit + 1, next) > accepted_height + 0.1 {
                            break;
                        }
                        advance = next;
                        fit += 1;
                    }
                    // 수용 상자에 들어가는 유닛은 그대로 소비한다. 원점0 근처라고
                    // 한 유닛 되감으면 들어가는 줄을 다음 조각으로 잘못 미룬다.
                    // 독립적으로 인증된 저장 owner는 위 경로에서 이미 선택했다.
                    fit
                })
                .max(su)
        } else {
            usize::MAX
        };
        unit_diagnostics::trace_span_cut(cell, start_row, end_row, prior_h, cell_height, su, eu);
        (su, eu)
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

    /// 높이로 고른 원본 병합 칸의 가운데/아래 정렬은 전체 줄 슬롯을
    /// 수용한 뒤, 같은 컷의 가시 내용으로 실제 물리 상자 안에서 정렬한다.
    pub(in crate::renderer::layout) fn stored_span_cut_preserves_line_slots(
        &self,
        table: &Table,
        cell: &Cell,
    ) -> bool {
        cell.vertical_align != crate::model::table::VerticalAlign::Top
            && self.stored_plain_text_span_frame(table, cell)
    }

    pub(super) fn stored_plain_text_cell_frame(&self, table: &Table, cell: &Cell) -> bool {
        self.stored_body_text_cell_frame(table, cell)
            && cell.paragraphs.iter().all(|para| para.controls.is_empty())
    }

    /// 머리말·꼬리말 정의는 별도 쪽 영역의 소유자이며 셀 본문의 개체가 아니다.
    /// 저장 줄의 유효성은 같은 계약으로 확인하되, 기존 순수 글줄 소비자는 위의
    /// 더 엄격한 컨트롤 없음 조건을 유지한다.
    pub(super) fn stored_body_text_cell_frame(&self, table: &Table, cell: &Cell) -> bool {
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
                    && para.controls.iter().all(|control| {
                        matches!(
                            control,
                            crate::model::control::Control::Header(_)
                                | crate::model::control::Control::Footer(_)
                        )
                    })
                    && !para.line_segs.is_empty()
                    && para.line_segs.iter().all(|line| {
                        line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                            && line.vertical_pos >= 0
                            && line.line_height > 0
                            && line.line_spacing >= 0
                    })
            })
    }

    pub(super) fn stored_span_line_tail_spacing(&self, unit: &CellUnit) -> f64 {
        if unit.empty_spacer
            || unit.vis_start >= unit.vis_end
            || unit.nested_row.is_some()
            || unit.mixed_nested_fragment
            || unit.non_inline_control_range.is_some()
        {
            return 0.0;
        }
        unit.following_line_advance.min(unit.height.max(0.0))
    }
}
