//! 이어받은 걸침 셀의 내용과 물리 종료 요구를 함께 산출한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::renderer::hwpunit_to_px;

#[derive(Debug, Clone, Copy)]
pub(crate) struct StraddleContinuationDemand {
    /// 이 조각이 받는 내용과 상하 안 여백의 점유.
    pub(crate) content: f64,
    /// 실제 앞 상자를 차감한 원본 물리 최소 높이.
    pub(crate) physical: f64,
}

impl StraddleContinuationDemand {
    pub(crate) fn required(self) -> f64 {
        self.content.max(self.physical)
    }
}

impl LayoutEngine {
    /// 시작 경계를 걸친 셀의 마지막 행을 온전히 배치할 때 필요한 조각 높이.
    /// 끝 컷이 있으면 남은 유닛 전부를 받지 않으므로 그 셀의 증분 예약은 제외한다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn straddle_continuation_demand(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_row: usize,
        start_cut: &[usize],
        start_cut_is_block: bool,
        start_row_height_override: Option<f64>,
        resolved_row_heights: &[f64],
        straddle_page_bands: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &ResolvedStyleSet,
        fragment_end: (usize, bool),
    ) -> Option<StraddleContinuationDemand> {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
                | crate::model::table::TablePageBreak::CellBreak
        ) {
            return None;
        }
        let (end_row, end_cut_is_empty) = fragment_end;
        self.spanning_cells(table)
            .filter(|cell| {
                let cell_row = cell.row as usize;
                let cell_end = cell_row + cell.row_span as usize;
                // 같은 행 재개도 남은 물리 띠를 소유한다. 예약과 paint는
                // 동일한 원본 셀·내용 창과 수용 전 prefix를 소비한다.
                // 표 전체 셀을 찾는 재개 판정은 이 행에서 닫히는 후보에만 한다.
                let resumes_own_row = || {
                    super::super::table_partial::resumes_inside_own_start_row(
                        table,
                        cell,
                        start_row,
                        start_cut,
                        start_cut_is_block,
                        start_row_height_override,
                        straddle_page_bands,
                    )
                };
                cell.row_span > 1
                    && cell_end > start_row
                    && cell_end == row + 1
                    && (cell_end < end_row || (cell_end == end_row && end_cut_is_empty))
                    && (cell_row < start_row || resumes_own_row())
            })
            .map(|cell| {
                let (su, eu) = self.rowbreak_straddle_cut_units(
                    table,
                    cell,
                    start_row,
                    end_row,
                    start_cut,
                    start_cut_is_block,
                    start_row_height_override,
                    // 이 소유자의 마지막 행은 온전히 받는다. 뒤 행에 있는
                    // 별도 끝 컷을 이 칸의 내용 컷으로 오인하지 않는다.
                    &[],
                    false,
                    None,
                    0.0,
                    resolved_row_heights,
                    straddle_page_bands,
                    styles,
                );
                let content = self.cell_cut_visible_height(cell, table, styles, su, eu);
                let carry = straddle_page_bands.iter().find(|carry| {
                    table
                        .cells
                        .get(carry.cell_index)
                        .is_some_and(|owner| std::ptr::eq(owner, cell))
                });
                let prefix = carry.and_then(|carry| carry.accepted_prefix_physical_height());
                let physical = prefix.map_or(0.0, |before| {
                    let declared = if cell.height < 0x8000_0000 {
                        hwpunit_to_px(cell.height as i32, self.dpi)
                    } else {
                        0.0
                    };
                    let whole = self.cell_cut_visible_height(
                        cell,
                        table,
                        styles,
                        0,
                        self.cell_units(cell, table, styles).len(),
                    );
                    (declared.max(whole) - before).max(0.0)
                });
                super::unit_diagnostics::trace_continuation_demand(
                    cell,
                    (content, physical),
                    (
                        prefix,
                        carry.and_then(|carry| carry.accepted_physical_height),
                    ),
                    carry.is_some_and(|carry| carry.fragment_cut.is_some()),
                );
                StraddleContinuationDemand { content, physical }
            })
            .reduce(|a, b| StraddleContinuationDemand {
                content: a.content.max(b.content),
                physical: a.physical.max(b.physical),
            })
    }

    /// 시작 블록 뒤에서 새로 시작한 일반 행 셀은 실제 수용 상자를 소유한다.
    /// 블록 안에서 시작한 셀은 아직 계상하지 않은 블록 공간을 추정하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn accepted_straddle_after_block(
        &self,
        table: &crate::model::table::Table,
        resolved_row_heights: &[f64],
        trailing_start: Option<usize>,
        end_row: usize,
        end_cut: &[usize],
        is_continuation: bool,
        end_row_height_override: Option<f64>,
        straddle_row_relief: &[(usize, f64)],
        carried: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &ResolvedStyleSet,
    ) -> Vec<crate::renderer::pagination::StraddleCellCarry> {
        let mut accepted: Vec<_> = carried
            .iter()
            .cloned()
            .map(|mut carry| {
                carry.accepted_physical_height = None;
                carry.owns_blank_band = false;
                carry
            })
            .collect();
        if let Some(start) = trailing_start {
            let ordinary = self.accepted_straddle_fragment_units(
                table,
                resolved_row_heights,
                start,
                end_row,
                &[],
                end_cut,
                is_continuation,
                None,
                end_row_height_override,
                straddle_row_relief,
                carried,
                styles,
            );
            for owner in ordinary.into_iter().filter(|carry| {
                table
                    .cells
                    .get(carry.cell_index)
                    .is_some_and(|cell| usize::from(cell.row) >= start)
            }) {
                accepted.retain(|carry| carry.cell_index != owner.cell_index);
                accepted.push(owner);
            }
        }
        accepted
    }

    /// 수용한 조각의 쪽 끝 빈 띠를 경계를 넘는 걸침 칸마다 누적한다.
    /// 내용 컷과 실제 상자 원장도 보존하고, 경계를 다 지난 칸은 버린다.
    pub(crate) fn straddle_page_bands_after_fragment(
        table: &crate::model::table::Table,
        carried_bands: &[crate::renderer::pagination::StraddleCellCarry],
        next_row: usize,
        band: f64,
        owns_blank_band: bool,
    ) -> Vec<crate::renderer::pagination::StraddleCellCarry> {
        table
            .cells
            .iter()
            .enumerate()
            .filter(|(index, cell)| {
                // 같은 행에서 재개할 때 내용 컷이 0→0이어도 앞 조각이
                // 실제 공간을 사용했을 수 있다. 내용과 물리 prefix를 함께
                // 유지해야 새 첫 상자로 오인해 선언 높이를 다시 예약하지 않는다.
                // 미접촉 미래 셀은 앞 조각의 수용 목록에 없다.
                let started_on_own_row = cell.row as usize == next_row
                    && carried_bands.iter().any(|carry| {
                        carry.cell_index == *index
                            && (carry.next_unit.is_some_and(|unit| unit > 0)
                                || carry.fragment_cut.is_some_and(|(start, end)| end > start)
                                || carry
                                    .accepted_physical_height
                                    .is_some_and(|height| height.is_finite() && height > 0.0))
                    });
                cell.row_span > 1
                    && ((cell.row as usize) < next_row || started_on_own_row)
                    && next_row < cell.row as usize + cell.row_span as usize
            })
            .map(|(index, _)| {
                let before = carried_bands
                    .iter()
                    .find(|carry| carry.cell_index == index)
                    .map_or(0.0, |carry| carry.physical_height);
                let accepted = carried_bands.iter().find(|carry| carry.cell_index == index);
                crate::renderer::pagination::StraddleCellCarry {
                    cell_index: index,
                    physical_height: before + band.max(0.0),
                    owns_blank_band: owns_blank_band
                        || accepted.is_some_and(|carry| carry.owns_blank_band),
                    accepted_physical_height: accepted
                        .and_then(|carry| carry.accepted_physical_height),
                    accepted_before_fragment: None,
                    next_unit: accepted.and_then(|carry| {
                        carry.fragment_cut.map(|(_, end)| end).or(carry.next_unit)
                    }),
                    fragment_cut: None,
                }
            })
            .collect()
    }
}
