//! TAC 표 문단의 실제 배치에서 얻은 흐름 끝과 물리 점유 끝.

/// 후행 간격을 포함한 전진과, 표·글줄이 점유하는 끝을 구분한다.
/// 페인트 원점을 이동해도 논리 흐름만 원래 문단 원점으로 되돌린다.
pub(crate) struct InlineTableParagraphGeometry {
    pub(crate) flow_bottom: f64,
    pub(crate) content_bottom: f64,
    pub(crate) line_box_bottom: f64,
}

impl InlineTableParagraphGeometry {
    pub(crate) fn at_flow_origin(mut self, paint_origin: f64, flow_origin: f64) -> Self {
        // 공백 캐리어의 기존 전진 계약을 유지한다. 내용 경계는 페인트 좌표다.
        self.flow_bottom = flow_origin + (self.flow_bottom - paint_origin).max(0.0);
        self.line_box_bottom += flow_origin - paint_origin;
        self
    }
}
