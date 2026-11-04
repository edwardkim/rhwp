//! 저장 텍스트 줄의 내용 상자와 실효 안 여백을 같은 행 최소값으로 측정한다.

use crate::model::table::{Cell, Table};

use super::LayoutEngine;

impl LayoutEngine {
    /// 선언 셀 높이는 내용 높이가 아니다. 실제 텍스트 줄(공백 포함)이 있는
    /// 텍스트·인라인 수식 저장 셀은 줄 끝과 활성 안 여백을 함께 수용해야 한다.
    ///
    /// 빈 저장 문단의 lh는 셀 전체 물리 프레임을 나타낼 수 있다. 그림/표의
    /// 저장 lh도 개체 높이를 이미 소유하므로 여기서 텍스트 줄로 재분류하지
    /// 않는다. 이 경로들은 기존 프레임/개체 점유 계약을 소비한다.
    pub(super) fn stored_plain_cell_content_minimum(
        &self,
        cell: &Cell,
        table: &Table,
        line_based: f64,
        effective_pad_v: f64,
    ) -> Option<f64> {
        let profile = self.profile.get();
        // 다른 문단의 편집은 이 셀의 저장 줄과 활성 여백을 무효화하지 않는다.
        // 측정과 동일하게 이 셀의 실제 재조판 여부를 확인하고, 공통 생산자가
        // 셀/문단 dirty 및 합성 줄을 거절하도록 한다.
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || self
                .render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell)
        {
            return None;
        }
        crate::renderer::float_placement::stored_text_line_cell_minimum(
            cell,
            line_based,
            effective_pad_v,
        )
    }
}
