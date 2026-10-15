//! 저장된 중첩 프레임의 열린 컷은 부모 셀의 종료 안 여백을 소유하지 않는다.

use crate::model::table::{Cell, Table, TablePageBreak};

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    pub(super) fn nested_source_frame_bottom_padding(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.page_break != TablePageBreak::RowBreak
            || cell.row_span != 1
            || cell.dirty_flag
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || end <= start
            || end >= units.len()
        {
            return 0.0;
        }
        let closing = &units[end - 1];
        let next = &units[end];
        // 재귀 셀 유닛은 자식의 실제 저장 reset을 부모 컷에 전달한다.
        // 임의 용량 컷·텍스트 줄·전체 셀 종료에는 이 여백을 회수하지 않는다.
        if !closing.mixed_nested_recursive
            || !next.mixed_nested_recursive
            || !next.stored_frame_break_before
            || closing.para_idx != next.para_idx
            || closing.mixed_nested_source_para_idx != next.mixed_nested_source_para_idx
        {
            return 0.0;
        }
        let (_, _, _, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
        bottom.min(closing.height)
    }
}
