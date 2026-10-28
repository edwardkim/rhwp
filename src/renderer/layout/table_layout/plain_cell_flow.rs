//! 셀의 단 설정 메타데이터와 실제 줄·개체 점유를 구분한다.

use crate::model::paragraph::Paragraph;

impl super::LayoutEngine {
    /// 내용이 끝난 종료 병합 칸의 미소비 최소 공간만 보존한다.
    /// 남은 글줄은 기존 유닛 원장과 이어받기 요구가 소유한다. 저장 줄 칸도 같다 —
    /// 한/글 2020 1342000 61·62쪽 첫 빈 띠(51.6·84.1px)는 일반 행 꼬리가 아니라 그 행에서
    /// 끝나는 병합 칸(행 8–17 등)의 남은 선언이다. 저장 줄 칸은 끝 행 내용이 이 조각에서
    /// 끝난(`complete_end_row`) 경우만 본다 — 블록 컷의 부분 행(1341000 29쪽)은 저장 프레임이 소유한다.
    pub(crate) fn ending_span_minimum_remainder(
        &self,
        table: &crate::model::table::Table,
        end_row: usize,
        accepted: &[crate::renderer::pagination::StraddleCellCarry],
        complete_end_row: bool,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> Option<f64> {
        table
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| {
                cell.row_span > 1
                    && usize::from(cell.row) + usize::from(cell.row_span) == end_row
                    && (complete_end_row || self.cell_uses_plain_reflow_units(cell, table))
            })
            .filter_map(|(index, cell)| {
                let owner = accepted.iter().find(|owner| owner.cell_index == index)?;
                let physical = owner.accepted_physical_height?;
                let units = self.cell_units(cell, table, styles);
                let end = owner.fragment_cut?.1.min(units.len());
                if end != units.len() {
                    return None;
                }
                let minimum = if cell.height < 0x8000_0000 {
                    super::hwpunit_to_px(cell.height as i32, self.dpi)
                } else {
                    0.0
                };
                let remaining = minimum - physical;
                // 종료 병합 칸도 자기 내용이 모두 끝난 작은 빈 꼬리는 흡수한다.
                // 일반 칸에서 흡수한 공간을 이 원장에서 다시 만들지 않는다.
                if complete_row_blank_tail_is_absorbed(remaining, self.dpi) {
                    return None;
                }
                (remaining > 0.5).then_some(remaining)
            })
            .reduce(f64::max)
    }

    /// 내용이 끝난 뒤 한 줄도 되지 않는 최소 공간만 남으면 빈 조각을 만들지 않는다.
    /// 줄이 남은 컷에는 적용하지 않으며, 큰 빈 최소는 기존 물리 원장으로 이어진다.
    pub(crate) fn row_blank_tail_is_line_sliver(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        tail: f64,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> bool {
        let line = Self::row_cut_cell_order(table, row)
            .into_iter()
            .filter_map(|index| {
                let cell = &table.cells[index];
                let stored = self.stored_plain_text_cell_frame(table, cell);
                if !stored && !self.cell_uses_plain_reflow_units(cell, table) {
                    return None;
                }
                let units = self.cell_units(cell, table, styles);
                let last = units.last()?;
                // 마지막 잉크 상자에서는 다음 줄 간격을 이미 접었을 수 있다.
                // 완료 뒤의 한 줄 경계는 유효한 원본 슬롯을 소비하며,
                // 새로 조판한 줄에는 그 줄 생산자가 준 현재 높이를 쓴다.
                let source_slot = (stored && !last.reflowed_line_slot)
                    .then(|| {
                        let paragraph = cell.paragraphs.get(last.para_idx)?;
                        let line = paragraph.line_segs.get(last.vis_end.checked_sub(1)?)?;
                        line.line_height.checked_add(line.line_spacing)
                    })
                    .flatten()
                    .map_or(0.0, |height| super::hwpunit_to_px(height, self.dpi));
                Some(last.height.max(source_slot))
            })
            .fold(0.0, f64::max);
        blank_tail_is_line_sliver(tail, line, self.dpi)
    }

    /// 내용이 끝난 행의 빈 꼬리가 쪽 경계에서 흡수되는가. 저장 줄 행도 같은 상한을 쓴다.
    pub(crate) fn complete_row_blank_tail_is_absorbed(&self, tail: f64) -> bool {
        complete_row_blank_tail_is_absorbed(tail, self.dpi)
    }

    /// 현재 행을 넘어 계속되는 병합 칸의 독립적인 빈 최소 공간.
    /// 글줄이 재조판된다는 사실만으로 쪽 끝의 빈 띠를 소유하지 않는다.
    pub(crate) fn row_has_continuing_reflow_minimum(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> bool {
        table.cells.iter().any(|cell| {
            if usize::from(cell.row) > row
                || usize::from(cell.row) + usize::from(cell.row_span) <= row + 1
                || cell.height >= 0x8000_0000
                || !self.cell_uses_plain_reflow_units(cell, table)
            {
                return false;
            }
            let end = self.cell_units(cell, table, styles).len();
            super::hwpunit_to_px(cell.height as i32, self.dpi)
                > self.cell_cut_visible_height(cell, table, styles, 0, end) + 0.5
        })
    }

    /// 실제 재조판 요구가 선언을 넘으면 미지 행의 옛 배분을 버리고 같은 제약을 다시 푼다.
    /// 저장 프레임의 요구는 기존 순서대로 마지막 걸침 행에서 계속 보존한다.
    pub(crate) fn resolve_reflow_span_requirements(
        table: &crate::model::table::Table,
        requirements: &[(usize, f64)],
        initial_rows: &[f64],
        content_floors: &[f64],
        declared_shrink_hu: &[u32],
        dpi: f64,
        render_normalization: &crate::renderer::render_normalization::RenderNormalizationOverlay,
    ) -> Option<Vec<f64>> {
        let is_plain_reflow = |cell: &crate::model::table::Cell| {
            // 이웃 셀의 줄 구성이 바뀌어도 이 셀의 저장 선언 제약은 유지한다.
            // 컷 유닛과 같은 출처로 실제 재조판 셀의 내용 요구만 먼저 배분한다.
            cell.text_direction == 0
                && Self::cell_has_plain_reflow_paragraphs(
                    cell,
                    render_normalization.cell_text_partition_reflowed(table, cell),
                )
        };
        if !requirements.iter().any(|&(index, height)| {
            let cell = &table.cells[index];
            cell.height < 0x8000_0000
                && is_plain_reflow(cell)
                && height > super::hwpunit_to_px(cell.height as i32, dpi)
        }) {
            return None;
        }
        let mut required = vec![0.0f64; table.cells.len()];
        for &(index, height) in requirements {
            required[index] = height;
        }
        let mut constraints: Vec<(usize, usize, f64)> = Vec::new();
        for (index, cell) in table.cells.iter().enumerate() {
            let row = usize::from(cell.row);
            let span = usize::from(cell.row_span);
            if span <= 1 || row + span > initial_rows.len() || cell.height >= 0x8000_0000 {
                continue;
            }
            let declared = super::hwpunit_to_px(cell.height as i32, dpi);
            let total = if is_plain_reflow(cell) {
                declared.max(required[index])
            } else {
                declared
            } - super::hwpunit_to_px(1, dpi) * table.inner_cell_spacing_hu(span) as f64;
            if let Some(prior) = constraints
                .iter_mut()
                .find(|value| value.0 == row && value.1 == span)
            {
                prior.2 = prior.2.max(total);
            } else {
                constraints.push((row, span, total));
            }
        }
        constraints.sort_by_key(|&(_, span, _)| span);
        let mut heights = initial_rows.to_vec();
        crate::renderer::height_measurer::solve_rowspan_row_heights(&constraints, &mut heights);
        for (row, &shrink) in declared_shrink_hu.iter().enumerate().take(heights.len()) {
            if shrink != 0 {
                heights[row] = (heights[row] - super::hwpunit_to_px(shrink as i32, dpi))
                    .max(content_floors[row]);
            }
        }
        for &(index, required) in requirements {
            let cell = &table.cells[index];
            let row = usize::from(cell.row);
            let span = usize::from(cell.row_span);
            let combined = heights[row..row + span].iter().sum::<f64>()
                + super::hwpunit_to_px(1, dpi) * table.inner_cell_spacing_hu(span) as f64;
            if required > combined {
                heights[row + span - 1] += required - combined;
            }
        }
        Some(heights)
    }

    /// 저장 프레임을 높이 컷으로 다시 해석하지 않는 일반 재조판 출처.
    pub(crate) fn cell_has_plain_reflow_paragraphs(
        cell: &crate::model::table::Cell,
        table_reflowed: bool,
    ) -> bool {
        cell.paragraphs.iter().all(|paragraph| {
            paragraph_has_plain_flow(paragraph)
                && (table_reflowed || crate::renderer::para_has_no_stored_line_segs(paragraph))
        })
    }

    pub(crate) fn cell_uses_plain_reflow_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> bool {
        Self::cell_has_plain_reflow_paragraphs(
            cell,
            self.render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell),
        )
    }

    /// 일반 칸이 없는 행의 종료 병합 칸이 실제 수용 공간에서 모두 끝났는지 확인한다.
    /// 계속되는 다른 병합 칸의 높이와 내용 커서는 별도 원장에 남긴다.
    pub(crate) fn row_band_completes_ending_reflow_spans(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        accepted: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> bool {
        if table.page_break != crate::model::table::TablePageBreak::RowBreak
            || !Self::row_cut_cell_order(table, row).is_empty()
        {
            return false;
        }
        let mut has_ending_owner = false;
        for (index, cell) in table.cells.iter().enumerate().filter(|(_, cell)| {
            cell.row_span > 1 && usize::from(cell.row) + usize::from(cell.row_span) == row + 1
        }) {
            has_ending_owner = true;
            if !self.cell_uses_plain_reflow_units(cell, table) {
                return false;
            }
            let Some(carry) = accepted.iter().find(|carry| carry.cell_index == index) else {
                return false;
            };
            let units = self.cell_units(cell, table, styles);
            if !carry
                .fragment_cut
                .is_some_and(|(_, end)| end == units.len())
            {
                return false;
            }
            let Some(physical) = carry.accepted_physical_height else {
                return false;
            };
            // 끝 행 소속만으로 미소비 줄이 모두 들어갔다고 추정하지 않는다.
            let content = self.cell_cut_visible_height(cell, table, styles, 0, units.len());
            let roundoff = f64::EPSILON * content.abs().max(physical.abs()).max(1.0) * 8.0;
            if content > physical + roundoff {
                return false;
            }
        }
        has_ending_owner
    }
}

impl super::CellUnit {
    /// 빈 간격 유닛을 제외하고 실제 가시 줄 범위의 존재만 공유한다.
    pub(in crate::renderer::layout) fn has_visible_line(&self) -> bool {
        !self.empty_spacer && self.vis_start < self.vis_end
    }
}

/// 내용이 모두 끝난 행에서 그린 뒤 남은 빈 꼬리를 쪽 경계에서 흡수하는 상한(HU).
///
/// 한/글 2024 합성 사다리(#8105 `probes/make_declared_row_band.py`): 남은 선언이 1275HU(17.0px)
/// 이면 흡수하고 1290HU(17.2px) 이면 다음 쪽 첫머리 빈 띠로 넘긴다. 시험 행 칸 여백 283·500,
/// 표 기본 칸 여백 141·400, 칸 글자 10·16pt, 병합 칸·저장 줄 유무와 무관했다 — 칸의 줄 높이가
/// 기준이 아니다. 한/글 2020 #7642 stage3197 의 13.33px 흡수 · 21.33px 이월과도 맞는다.
/// 1282HU 는 기본 10pt 줄(1000)과 기본 칸 위아래 여백(141×2)의 합이다.
const COMPLETE_ROW_BLANK_TAIL_ABSORB_HU: i32 = 1282;

fn complete_row_blank_tail_is_absorbed(tail: f64, dpi: f64) -> bool {
    tail <= super::hwpunit_to_px(COMPLETE_ROW_BLANK_TAIL_ABSORB_HU, dpi)
}

/// 흡수 대상은 내용 유닛을 가진 소유 칸의 꼬리다.
fn blank_tail_is_line_sliver(tail: f64, line: f64, dpi: f64) -> bool {
    line > 0.0 && complete_row_blank_tail_is_absorbed(tail, dpi)
}

/// 한 단 설정은 이미 선택한 셀 프레임을 바꾸거나 내용 상자를 추가하지 않는다.
/// 다단 설정이나 독립 개체의 흐름 계약까지 이 판정으로 확대하지 않는다.
pub(super) fn paragraph_has_plain_flow(paragraph: &Paragraph) -> bool {
    crate::renderer::cell_paragraph_has_only_single_column_metadata(paragraph)
}
