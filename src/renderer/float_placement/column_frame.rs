//! 수용한 저장 줄의 기준값과 물리 단 앵커를 한 쌍으로 전달한다.

/// `column_anchor_y`는 InlineBoxPlacement의 y와 같은 단 상대 px 좌표다.
/// 실제 쪽 좌표를 소비하는 배치는 단의 원점을 한 번 더한다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredColumnFrame {
    pub source_origin_hu: i32,
    pub column_anchor_y: f64,
}

impl StoredColumnFrame {
    /// 같은 원문/물리 축이 실제 수용한 줄 원점을 설명할 때만 인증을 이어 쓴다.
    pub(crate) fn accepts_line_origin(self, source_hu: i32, origin: f64, dpi: f64) -> bool {
        dpi.is_finite()
            && dpi > 0.0
            && origin.is_finite()
            && self.column_anchor_y.is_finite()
            && (self.column_anchor_y
                + crate::renderer::hwpunit_to_px(
                    source_hu.saturating_sub(self.source_origin_hu),
                    dpi,
                )
                - origin)
                .abs()
                <= crate::renderer::hwpunit_to_px(1, dpi)
    }
}
