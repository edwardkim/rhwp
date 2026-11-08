//! 수식의 내용 폭과 좌우 바깥 여백을 분리해 같은 점유 폭을 공유한다.

use crate::model::control::Equation;

#[derive(Clone, Copy)]
pub(crate) struct InlineEquationHorizontalMetrics {
    pub content_width_hwp: i32,
    pub margin_left_hwp: i32,
    pub margin_right_hwp: i32,
}

impl InlineEquationHorizontalMetrics {
    pub fn occupied_width_hwp(self) -> i32 {
        self.content_width_hwp
            .saturating_add(self.margin_left_hwp)
            .saturating_add(self.margin_right_hwp)
    }

    pub fn content_origin_x(self, flow_origin_x: f64, dpi: f64) -> f64 {
        flow_origin_x + crate::renderer::hwpunit_to_px(self.margin_left_hwp, dpi)
    }
}

pub(crate) fn horizontal_metrics(equation: &Equation) -> InlineEquationHorizontalMetrics {
    let margin = &equation.common.margin;
    InlineEquationHorizontalMetrics {
        content_width_hwp: super::flow_width_hwp(equation) as i32,
        margin_left_hwp: if equation.common.treat_as_char {
            i32::from(margin.left)
        } else {
            0
        },
        margin_right_hwp: if equation.common.treat_as_char {
            i32::from(margin.right)
        } else {
            0
        },
    }
}
