//! 앞 조각의 실제 물리 공간과 현재 조각의 행 제약을 한 계획으로 전달한다.

use super::{hwpunit_to_px, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 앞 조각의 실제 걸침 칸 상자를 소비한 뒤 끝 행에서 덜어낼 양 `(행, px)`.
    /// 음수이면 남은 요구를 그리기 전에 스캔 예산에도 추가 예약한다.
    ///
    /// 측정기는 걸침 칸의 부족분(칸 요구 − 걸친 행 합)을 걸침의 마지막 행에 몰아 둔다.
    /// 칸이 쪽을 넘으면 한/글은 앞 쪽 마지막 행 상자를 본문 바닥까지 늘리고, 그 띠도 칸
    /// 높이로 센다. 이어받는 조각의 끝 행은 남은 부족분만 받는다(1342000 구역 5 `pi=11`:
    /// 칸 `(8,1)` rs=10 선언 636.1 ↔ 정본 147쪽 행 8~16 상자 580.7 + 148쪽 행 17 53.8,
    /// rhwp 는 행 17 에 부족분 18.5 를 더해 72.9).
    ///
    /// `carried_bands` 는 내용 컷과 실제 수용 상자 합·빈 띠를 칸 색인별로 전달한다.
    /// 실제 상자 원장이 있으면 그 합과 이 조각의 보정된 행을 소비한다. 원장이 없는
    /// 기존 경로는 측정 행 합과 빈 띠를 사용한다. 행의 `row_span==1` 칸
    /// 선언·내용 높이 아래로는 줄이지 않는다. 조판 scan 과 `table_partial` 그리기가 같은
    /// 값을 쓰도록 결과는 조각 항목에 실어 넘긴다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn straddle_page_band_relief(
        &self,
        table: &crate::model::table::Table,
        row_heights: &[f64],
        cursor_row: usize,
        start_cut: &[usize],
        start_row_height_override: Option<f64>,
        carried_bands: &[crate::renderer::pagination::StraddleCellCarry],
        styles: &ResolvedStyleSet,
    ) -> Vec<(usize, f64)> {
        if carried_bands.is_empty()
            || !matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
        {
            return Vec::new();
        }
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let mut relief = Vec::new();
        // 시작 부분 행도 실제 그리기와 같은 상자로 잰다. 뒤의 끝 행은 이 상자와
        // 앞서 보정한 온전한 행을 소비하며 원래 측정 높이를 다시 합산하지 않는다.
        let mut planned = if !start_cut.is_empty() || start_row_height_override.is_some() {
            self.ordinary_rowbreak_fragment_heights(
                table,
                row_heights,
                cursor_row,
                (cursor_row + 1).min(row_heights.len()),
                start_cut,
                &[],
                true,
                start_row_height_override,
                None,
                &[],
                carried_bands,
                true,
                styles,
            )
        } else {
            row_heights.to_vec()
        };
        for row in cursor_row..row_heights.len() {
            // 부분 시작 행은 컷/override의 상자를 유지한다. 뒤 행 요구에도 그
            // 실제 상자를 사용하므로 호출자에서 제거할 보정을 여기에 선반영하지 않는다.
            if row == cursor_row && (!start_cut.is_empty() || start_row_height_override.is_some()) {
                continue;
            }
            // 이 행에서 끝나는 걸침 칸마다 끝 행에 요구하는 높이를 다시 잰다. 앞 조각에서
            // 시작한 칸은 앞 쪽 띠를 이미 차지했으므로 그만큼 덜 요구한다. 이 조각에서
            // 시작한 칸은 종전 요구를 그대로 둔다.
            let mut demand = 0.0f64;
            let mut original = 0.0f64;
            let mut any_crossing = false;
            let mut any_ending = false;
            let mut has_actual_owner = false;
            let mut adjusted_preceding_rows = false;
            for (index, cell) in table.cells.iter().enumerate() {
                if cell.row_span <= 1 || cell.row as usize + cell.row_span as usize != row + 1 {
                    continue;
                }
                any_ending = true;
                let start = cell.row as usize;
                let declared = if cell.height < 0x8000_0000 {
                    hwpunit_to_px(cell.height as i32, self.dpi)
                } else {
                    0.0
                };
                let units = self.cell_units(cell, table, styles).len();
                let required =
                    declared.max(self.cell_cut_visible_height(cell, table, styles, 0, units));
                let others: f64 = row_heights[start..row].iter().sum::<f64>()
                    + cell_spacing * (row - start) as f64;
                let previous = carried_bands.iter().find(|carry| carry.cell_index == index);
                let accepted = previous.and_then(|carry| carry.accepted_physical_height);
                let band = if start < cursor_row {
                    previous.map_or(0.0, |carry| carry.physical_height)
                } else {
                    0.0
                };
                let actual_others = planned[start.max(cursor_row)..row].iter().sum::<f64>()
                    + cell_spacing * (row - start.max(cursor_row)) as f64;
                // 앞 행의 변경은 새 쪽에서 시작한 칸의 종료 요구에도 영향을 준다.
                // 원래 측정 합으로 되돌리지 않고 같은 planned 행을 이어 소비한다.
                adjusted_preceding_rows |= relief
                    .iter()
                    .any(|&(affected, _)| start.max(cursor_row) <= affected && affected < row);
                let remaining = if let Some(before) = accepted {
                    has_actual_owner = true;
                    any_crossing |= before > 0.5;
                    required - before - actual_others
                } else if start >= cursor_row {
                    required - actual_others
                } else {
                    any_crossing |= band > 0.5;
                    required - others - band
                };
                unit_diagnostics::trace_span_remainder(
                    cell,
                    cursor_row,
                    accepted,
                    (required, others, actual_others, band, remaining),
                );
                demand = demand.max(remaining);
                original = original.max(required - others);
            }
            if !any_ending || !(any_crossing || adjusted_preceding_rows) {
                continue;
            }
            let declared = table
                .cells
                .iter()
                .filter(|cell| cell.row as usize == row && cell.row_span == 1)
                .filter(|cell| cell.height < 0x8000_0000)
                .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
                .fold(0.0, f64::max);
            let natural = declared.max(self.row_complete_cut_content_height(table, row, styles));
            // 덜 수 있는 양은 걸침 칸들이 이 행에 실제로 더한 몫 안이다. 행이 다른 이유로
            // 커진 몫(측정 내용 등)은 덜지 않는다.
            let amount = (row_heights[row].min(natural.max(original))) - natural.max(demand);
            if amount > 0.5 {
                relief.push((row, amount));
                planned[row] = (row_heights[row] - amount).max(0.0);
            } else if (has_actual_owner || adjusted_preceding_rows)
                && demand > row_heights[row] + 0.5
            {
                // 실제 앞 상자가 작았다면 남은 선언 공간도 스캔 예산에 먼저 예약한다.
                // 음수 덜기는 같은 값을 소비하는 그리기에도 필요한 증가분을 전달한다.
                relief.push((row, row_heights[row] - demand));
                planned[row] = demand;
            }
        }
        relief
    }
}
