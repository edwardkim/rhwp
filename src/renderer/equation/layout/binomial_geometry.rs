//! CHOOSE/BINOM의 두 행과 괄호를 같은 논리 프레임에서 배치한다.
//!
//! 한컴 2020의 숫자·단어·복합항·분수 대조군에서 일반 ATOP 괄호와
//! 다른 피치를 확인했다. 측정·SVG·Canvas가 아래 행/구분자 원점을 공유한다.
use super::{EqLayout, EqNode, LayoutBox, LayoutKind};

impl EqLayout {
    pub(super) fn layout_binomial(&self, top: &EqNode, bottom: &EqNode, fs: f64) -> LayoutBox {
        if !self.hancom_math_face {
            return self.layout_paren(
                "(",
                ")",
                &EqNode::Atop {
                    top: Box::new(top.clone()),
                    bottom: Box::new(bottom.clone()),
                },
                fs,
            );
        }
        let mut top = self.matrix_row_box(self.layout_node(top, fs), fs);
        let mut bottom = self.matrix_row_box(self.layout_node(bottom, fs), fs);
        let width = top.width.max(bottom.width).max(fs * 0.6);
        top.x = (width - top.width) / 2.0;
        bottom.x = (width - bottom.width) / 2.0;
        bottom.y = top.height + fs * 0.15;
        let height = bottom.y + bottom.height;
        let (line_height, line_baseline) =
            super::super::font_frame_metrics::line_at_dpi(fs, self.dpi).unwrap_or((fs, fs * 0.86));
        let baseline = super::ladder_geometry::centered_baseline(
            height,
            line_height,
            line_baseline,
            fs,
            self.dpi,
        );
        let slot = super::source_font_text::logical_em_advance(0.4, fs, self.dpi);
        let body = LayoutBox {
            font_size: fs,
            x: slot,
            y: 0.0,
            width,
            height,
            baseline,
            kind: LayoutKind::Atop {
                top: Box::new(top),
                bottom: Box::new(bottom),
            },
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: width + slot * 2.0,
            height,
            baseline,
            kind: LayoutKind::Paren {
                left: "(".into(),
                right: ")".into(),
                left_width: slot,
                right_width: slot,
                delimiter_height: height,
                glyph_delimiters: false,
                left_outline: self.delimiter_outline("(", slot, height, baseline, fs, false),
                right_outline: self.delimiter_outline(")", slot, height, baseline, fs, false),
                body: Box::new(body),
            },
        }
    }
}
