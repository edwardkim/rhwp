//! 일반 ATOP의 항 프레임·템플릿 슬롯과 첫 항 기준선 계약.
//!
//! 독립 한컴 2020의 숫자·단어·복합항·분수 및 빈 항 대조에 근거한다.
//! CHOOSE/BINOM의 가운데 정렬 프레임은 별도 소비 경로다.
use super::{EqLayout, EqNode, LayoutBox, LayoutKind};

impl EqLayout {
    pub(super) fn layout_source_atop(&self, top: &EqNode, bottom: &EqNode, fs: f64) -> LayoutBox {
        let mut top = self.matrix_row_box(self.layout_node(top, fs), fs);
        let mut bottom = self.matrix_row_box(self.layout_node(bottom, fs), fs);
        let column = top.width.max(bottom.width).max(fs * 0.6);
        let pad = fs * 0.25;
        let gap = fs * 0.1;
        let width = column + pad * 2.0;
        top.x = pad + (column - top.width) / 2.0;
        bottom.x = pad + (column - bottom.width) / 2.0;
        bottom.y = top.height + gap;
        let height = bottom.y + bottom.height;
        let (line_height, line_baseline) =
            super::super::font_frame_metrics::line_at_dpi(fs, self.dpi).unwrap_or((fs, fs * 0.86));
        let axis = line_baseline - line_height / 2.0;
        let percent = ((top.height + gap / 2.0 + axis) / height * 100.0).round();
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline: height * percent / 100.0,
            kind: LayoutKind::Atop {
                top: Box::new(top),
                bottom: Box::new(bottom),
            },
        }
    }
}
