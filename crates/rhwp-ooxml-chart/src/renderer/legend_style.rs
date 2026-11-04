//! 범례의 논리 슬롯과 그 안에 표시하는 한컴 기본 사각 키의 잉크를 구분한다.

use super::PX_PER_PT;

#[derive(Clone, Copy)]
pub(super) enum LegendFlow {
    Horizontal,
    Vertical,
}

pub(super) struct SquareLegendInk {
    pub left: f64,
    pub top: f64,
    pub side: f64,
    pub text: f64,
    pub advance: f64,
}

impl SquareLegendInk {
    /// 명시·상속 글자 크기 모두 같은 기본 키 서식을 소비한다.
    /// 한컴 Print의 16pt/16.01pt 경계와 5~24pt·서로 다른 face에서
    /// 확인한 잉크 상자다. 가로 셀의 전진과 세로 열의 정렬 슬롯을 구분하며,
    /// 실제 잉크 폭을 항목의 전진 폭으로 대신하지 않는다.
    pub fn new(font: f64, flow: LegendFlow) -> Self {
        let large = font > 16.0 * PX_PER_PT;
        let (top, side) = if large { (-0.375, 0.42) } else { (-0.25, 0.6) };
        let (left, text, advance) = match (flow, large) {
            (LegendFlow::Horizontal, false) => (0.25, 1.15, 1.1),
            (LegendFlow::Horizontal, true) => (0.15, 0.8, 0.75),
            (LegendFlow::Vertical, false) => (-0.1, 0.8, 1.0),
            (LegendFlow::Vertical, true) => (0.225, 0.85, 1.0),
        };
        Self {
            left: font * left,
            top: font * top,
            side: font * side,
            text: font * text,
            advance: font * advance,
        }
    }
}

pub(super) fn square_row_height(font: f64) -> f64 {
    font * if font > 16.0 * PX_PER_PT { 1.3 } else { 1.35 }
}
