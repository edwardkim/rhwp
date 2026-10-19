//! 문단 시작과 위 간격을 이미 포함한 첫 글줄 시작을 구분한다.

#[derive(Clone, Copy, Debug)]
pub(crate) enum FlowOrigin {
    Paragraph(f64),
    Line(f64),
}

impl FlowOrigin {
    pub(crate) fn y(self) -> f64 {
        match self {
            Self::Paragraph(y) | Self::Line(y) => y,
        }
    }

    pub(crate) fn first_line_y(self, spacing_before: f64) -> f64 {
        match self {
            Self::Paragraph(y) => y + spacing_before,
            Self::Line(y) => y,
        }
    }
}
