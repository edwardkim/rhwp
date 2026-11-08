//! 셀 생성용 기본 높이와 저장 병합 그리드의 미결 행을 구분한다.

use crate::model::{paragraph::LineSeg, table::Table};

use super::LayoutEngine;

impl LayoutEngine {
    /// 내용·선언 제약을 모두 적용한 뒤에도 높이가 없는 행의 양수 띠다.
    /// 저장 그리드에서 여러 행에 걸친 셀만 덮는 띠는 새 셀의 400HU 상자가
    /// 아니다. 독립 Print의 얇은 행 경계와 같은 1pt 최소 띠를 소비한다.
    /// 이미 측정한 양수 높이는 이 함수의 대상이 아니다.
    pub(crate) fn unresolved_row_default_hu(table: &Table, row: usize, stored: bool) -> i32 {
        if !stored || table.col_count == 0 || row >= usize::from(table.row_count) {
            return 400;
        }
        let mut columns = vec![false; usize::from(table.col_count)];
        for cell in &table.cells {
            let start = usize::from(cell.row);
            let end = start + usize::from(cell.row_span);
            if !(start <= row && row < end) {
                continue;
            }
            let col = usize::from(cell.col);
            let col_end = col + usize::from(cell.col_span);
            if cell.row_span <= 1
                || cell.dirty_flag
                || cell.text_direction != 0
                || cell.col_span == 0
                || end > usize::from(table.row_count)
                || col_end > columns.len()
                || cell.paragraphs.is_empty()
                || cell.paragraphs.iter().any(|para| {
                    para.stored_text_partition_is_dirty()
                        || para.cell_format_vpos_dirty
                        || !para.controls.is_empty()
                        || para.line_segs.is_empty()
                        || para.line_segs.iter().any(|line| {
                            line.line_height <= 0
                                || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        })
                })
                || columns[col..col_end].iter().any(|owned| *owned)
            {
                return 400;
            }
            columns[col..col_end].fill(true);
        }
        if columns.into_iter().all(|owned| owned) {
            100
        } else {
            400
        }
    }
}
