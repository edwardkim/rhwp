//! 저장된 행의 완전한 끝 조각이 소유하는 세로 정렬 프레임.

use super::LayoutEngine;

impl LayoutEngine {
    /// 원본의 이어받기 상자는 앞 조각이 소비하고 남은 저장 행 높이다.
    /// 남은 내용을 모두 담는 마지막 조각은 이 상자 안에서 셀 valign을 적용한다.
    /// 편집 후에는 해당 셀의 저장 프레임이 온전해야 남은 높이로 정렬할 수 있다.
    pub(super) fn stored_row_frame_owns_complete_tail(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
    ) -> bool {
        (self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().hwpx_stored_layout())
            && (!self.profile.get().session_edited()
                || !self
                    .render_normalization
                    .borrow()
                    .cell_frame_reflowed(table, cell))
            && !self
                .render_normalization
                .borrow()
                .cell_text_partition_reflowed(table, cell)
    }
}
