//! 행 오프셋 컷의 유닛 소유와 물리 상자.

use super::{CellUnit, LayoutEngine, RowCut, RowCutResult};
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 같은 유닛 범위의 내용 전진과 paint의 물리 점유 높이를 구분한다.
/// 컷 후보와 paint는 저장 줄 끝의 trim과 안여백을 같은 결과에서 받는다.
pub(super) struct CellCutFrame<'a> {
    engine: &'a LayoutEngine,
    cell: &'a crate::model::table::Cell,
    table: &'a crate::model::table::Table,
    styles: &'a ResolvedStyleSet,
    units: &'a [CellUnit],
    start: usize,
    padding: f64,
}

impl<'a> CellCutFrame<'a> {
    pub(super) fn new(
        engine: &'a LayoutEngine,
        cell: &'a crate::model::table::Cell,
        table: &'a crate::model::table::Table,
        styles: &'a ResolvedStyleSet,
        units: &'a [CellUnit],
        start: usize,
    ) -> Self {
        let (_, _, top, bottom) = engine.resolve_stored_nested_cell_padding(cell, table, styles);
        Self {
            engine,
            cell,
            table,
            styles,
            units,
            start,
            padding: top + bottom,
        }
    }

    pub(super) fn content_budget(&self, physical_budget: f64) -> f64 {
        (physical_budget - self.padding - self.header_height(self.start + 1)).max(0.0)
    }

    fn header_height(&self, end: usize) -> f64 {
        if end > self.start {
            self.engine.nested_continuation_header_height(
                self.cell,
                self.units,
                self.start,
                self.styles,
            )
        } else {
            0.0
        }
    }

    pub(super) fn height(&self, end: usize, content_advance: f64) -> f64 {
        let trim = self.engine.native_saved_reset_cut_trailing_trim(
            self.table,
            self.cell,
            self.units,
            self.start,
            end,
            self.styles,
        );
        let content = (content_advance - trim).max(0.0) + self.header_height(end);
        let physical = if content > 0.0 {
            content + self.padding
        } else {
            0.0
        };
        if self.start == 0 {
            if let Some((owner, opening)) =
                self.engine
                    .first_source_frame_owner(self.table, self.cell, self.styles)
            {
                if end == owner {
                    return physical.max(opening);
                }
            }
        }
        physical
    }
}

impl LayoutEngine {
    /// RowBreak rowspan 블록에서 셀의 행 시작 y를 반영해 컷을 전진시킨다.
    ///
    /// 일반 `advance_row_block_cut`은 블록 안의 모든 셀에 같은 예산을 주기 때문에,
    /// 위쪽 큰 셀이 페이지 경계에서 잘릴 때 아래 행의 짧은 셀까지 먼저 소비할 수 있다.
    /// 이 함수는 행별 top offset을 빼고 남은 예산으로 셀을 전진시켜 같은 블록 안의
    /// 아래 행 내용이 한컴처럼 다음 조각에 남도록 한다.
    pub(crate) fn advance_row_block_cut_with_row_offsets(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        avail_height: f64,
        row_offsets: &[f64],
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        cells.sort_by_key(|c| (c.row, c.col));

        let mut end_cut: RowCut = Vec::with_capacity(cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        let mut physical_height = 0.0f64;
        // [#2291] plain 블록 walk(advance_row_block_cut)와 동일한 relaxed_hard_break
        // 의미론 — 한글 2022 는 재개방 시 저장 vpos-reset(원저작 쪽나눔 흔적)을
        // 무시하고 fresh 재배치로 쪽을 만충한다(연결맵 p26 = 81줄 실측, #2291).
        // 종전 이 walk 는 hard-break 에서 무조건 정지해 예산 잔여(≤52px)를 버리고
        // 조각 경계가 한글과 어긋났다.
        const HARD_BREAK_REMAINING_TOLERANCE_PX: f64 = 32.0;
        let block_has_top_and_bottom_flow = cells
            .iter()
            .any(|cell| self.cell_has_top_and_bottom_non_inline_flow(cell));
        let relaxed_hard_break = matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) && (table.col_count <= 2 || table.row_count > 5)
            && !block_has_top_and_bottom_flow;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let cell_row = cell.row as usize;
            let row_offset = cell_row
                .checked_sub(b_start)
                .and_then(|idx| row_offsets.get(idx))
                .copied()
                .unwrap_or(0.0);
            let cell_budget = (avail_height - row_offset).max(0.0);
            let frame = CellCutFrame::new(self, cell, table, styles, &units, start);
            let content_budget = frame.content_budget(cell_budget);
            let allow_force_progress = row_offset <= 0.5;
            let mut j = start;
            let mut h = 0.0f64;
            // [#2287/PR #2290 P1] 연속 조각(start>0)이 시작 직후(start+1) 저장
            // hard-break 를 만나면, start 유닛은 직전 조각의 orphan-rewind 가
            // 이월시킨 고아다 — 여기서 hard 를 쪽 경계로 존중하면 고아 혼자
            // 한 쪽(교육부 47×9 p26: 유닛 1개 17.3px sliver)이 되어 rewind 의
            // 의도(고아 방지)와 정반대가 된다. 극소 소비(h ≤ 한 줄급) 한정으로
            // 그 hard 는 이미 소비된 경계로 보고 통과한다.
            const REWIND_ORPHAN_CONT_PX: f64 = 48.0;
            // [#2291] relaxed 통과는 거대 셀(원저작 쪽나눔 흔적이 촘촘한 다문단
            // 셀)에 한정한다 — 소형 셀의 저장 hard-break 는 실제 행/조각 경계라
            // 통과시키면 조각이 비대해져 밴드 컷이 어긋난다 (21217935 8→9쪽
            // 회귀 실측). 연결맵 결함 셀은 유닛 37~123개.
            const GIANT_CELL_RELAXED_MIN_UNITS: usize = 24;
            let cell_relaxed = relaxed_hard_break && units.len() >= GIANT_CELL_RELAXED_MIN_UNITS;
            while j < units.len() {
                let u = &units[j];
                let next_physical_height = frame.height(j + 1, h + u.height);
                if j > start
                    && u.hard_break_before
                    && (!cell_relaxed
                        || (!u.empty_spacer
                            && (next_physical_height > cell_budget
                                || cell_budget - frame.height(j, h)
                                    <= HARD_BREAK_REMAINING_TOLERANCE_PX)))
                {
                    if start > 0 && j == start + 1 && h <= REWIND_ORPHAN_CONT_PX {
                        h += u.height;
                        j += 1;
                        continue;
                    }
                    if !self.stored_paragraph_allows_orphan_split(table, cell, &units, j, styles) {
                        Self::rewind_rowbreak_orphan_before_hard_break(
                            table,
                            &units,
                            start,
                            content_budget,
                            true,
                            &mut j,
                            &mut h,
                        );
                    }
                    hit_hard_break = true;
                    break;
                }
                if j > start && next_physical_height > cell_budget {
                    if Self::rewind_rowbreak_mixed_nested_table_tail_for_fresh_page(
                        table,
                        &units,
                        start,
                        self.current_body_area.get().3.max(content_budget),
                        &mut j,
                        &mut h,
                    ) {
                        break;
                    }
                    // [#1921] sliver 흡수 — advance_row_block_cut 의 예산 정지와 동일.
                    // 직후 tolerance 안의 저장 hard-break(한글 실제 페이지 경계)까지
                    // 흡수해, 다음 fragment 가 극소 잔여 sliver 페이지가 되는 것을 막는다.
                    if let Some((absorbed_h, absorbed_j)) =
                        Self::absorb_tail_before_stored_hard_break(&units, j, h, content_budget)
                    {
                        h = absorbed_h;
                        j = absorbed_j;
                        hit_hard_break = true;
                        break;
                    }
                    break;
                }
                if j == start && !allow_force_progress && next_physical_height > cell_budget {
                    break;
                }
                if self.should_defer_overflowing_top_and_bottom_entry(
                    u,
                    j,
                    start,
                    h,
                    content_budget,
                ) {
                    break;
                }
                h += u.height;
                j += 1;
            }
            // 후보의 컷 소유를 확정한 뒤 같은 물리 상자를 예약한다.
            // 내용 전진 h는 orphan/hard-break 정책을 위해 따로 유지한다.
            let cell_physical_height = frame.height(j, h);
            // 기존 저장 restart의 수용 계약은 물리 첫 프레임 높이를 쓴다.
            // 그 밖의 부분 컷 최소 높이는 안여백을 더하기 전의 내용 전진이다.
            let minimum_cut_height = if self
                .native_first_line_restart_cut_trailing_trim(table, cell, &units, start, j, styles)
                > 0.0
            {
                cell_physical_height
            } else {
                h + frame.header_height(j)
            };
            if j < units.len() {
                fully_consumed = false;
            }
            if minimum_cut_height > 0.0 {
                consumed_height = consumed_height.max(row_offset + minimum_cut_height);
            }
            if cell_physical_height > 0.0 {
                physical_height = physical_height.max(row_offset + cell_physical_height);
            }
            // [#2097 진단] 오프셋 walk 셀별 결과 — 동작 불변.
            if std::env::var("RHWP_DIAG_BLKCUT").is_ok() {
                let stop = if j >= units.len() {
                    "end"
                } else if units[j].hard_break_before {
                    "hard"
                } else {
                    "budget"
                };
                eprintln!(
                    "DIAG_BLKCUT(ofs) cell[{}] r={} c={} units={} start={} j={} h={:.1} physical_h={:.1} row_off={:.1} cell_budget={:.1} stop={} next_h={:.1}",
                    i,
                    cell.row,
                    cell.col,
                    units.len(),
                    start,
                    j,
                    h,
                    cell_physical_height,
                    row_offset,
                    cell_budget,
                    stop,
                    units.get(j).map(|u| u.height).unwrap_or(0.0)
                );
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
            physical_height: Some(physical_height),
        }
    }
}
