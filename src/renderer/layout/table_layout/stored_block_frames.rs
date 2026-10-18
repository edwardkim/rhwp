//! 저장 쪽 경계에서 나뉜 블록의 내용 컷과 물리 행 상자를 구분한다.

use crate::model::table::{Table, TablePageBreak};
use crate::renderer::hwpunit_to_px;

use super::{LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 실제 원본 줄·단일행 선언·걸침 셀의 저장 쪽 재시작이 함께 증명하는 행 상자.
    /// 편집/재조판과 중첩 개체는 `stored_plain_text_cell_frame`에서 제외한다.
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
                .any(|cell| !self.stored_plain_text_cell_frame(table, cell))
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
