//! 측정이 해석한 문단 프레임 소유 폭을 컷과 배치에도 전달한다.

use super::LayoutEngine;
use crate::model::table::{Cell, Table};

impl LayoutEngine {
    /// 닫히지 않은 행의 raw 폭은 보조값이며 표의 해석된 열이 문단 폭을 소유한다.
    /// 독립적인 행이나 명시적 편집 폭은 기존 물리 그리드 경로를 유지한다.
    pub(super) fn paragraph_frame_width_override(&self, cell: &Cell, table: &Table) -> Option<f64> {
        if cell.width > i32::MAX as u32 {
            return None;
        }
        let rows = self.table_rows_scope(table);
        let source_index = rows
            .starting(usize::from(cell.row))
            .iter()
            .copied()
            .find(|&index| std::ptr::eq(&table.cells[index], cell))
            .or_else(|| {
                // 조각은 높이·문단 범위만 자른 복사본일 수 있다. 원본 셀의 같은
                // 열·span·폭으로 소유를 확인하며 편집된 다른 폭은 승격하지 않는다.
                rows.starting(usize::from(cell.row))
                    .iter()
                    .copied()
                    .find(|&index| {
                        let source = &table.cells[index];
                        source.col == cell.col
                            && source.row_span == cell.row_span
                            && source.col_span == cell.col_span
                            && source.width == cell.width
                    })
            })?;
        let width = *rows.paragraph_widths().get(source_index)?;
        (width > 0 && u32::try_from(width).ok() != Some(cell.width)).then(|| {
            crate::renderer::hwpunit_to_px(width, self.dpi) * self.render_table_width_scale(table)
        })
    }
}
