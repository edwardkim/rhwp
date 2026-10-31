//! 확정한 첫 줄 원점과 앞 간격을 예약·배치에 함께 전달한다.

#[derive(Debug, Clone, Copy)]
pub struct FloatPushedFirstLine {
    /// 단 상대 문단 원점. 앞 간격은 이 원점에서 한 번 소비한다.
    pub origin: Option<f64>,
    pub spacing_before: f64,
}

impl FloatPushedFirstLine {
    pub(crate) fn spacing(spacing_before: f64) -> Self {
        Self {
            origin: None,
            spacing_before,
        }
    }
}

crate::storage::clone_copy!(FloatPushedFirstLine);
