//! 줄 수용에서 확정한 ASCII 공백 압축. 문서 속성과 글자 자간은 바꾸지 않는다.

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceCompression {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) reduction_px: f64,
}

impl SpaceCompression {
    pub(crate) fn reduction_at(&self, scalar: usize) -> f64 {
        if (self.start..self.end).contains(&scalar) {
            self.reduction_px
        } else {
            0.0
        }
    }

    pub(crate) fn for_fragment(&self, start: usize) -> Self {
        Self {
            start: self.start.saturating_sub(start),
            end: self.end.saturating_sub(start),
            reduction_px: self.reduction_px,
        }
    }
}
