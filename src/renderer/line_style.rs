//! HWP 선 종류를 모든 출력 경로에서 사용하는 선 스타일로 변환한다.

use super::{LineRenderType, StrokeDash};

/// 도형의 HWP5 선 코드. HWPX lineShape의 DASH/DOT 이름과 구분한다.
pub(crate) fn hwp_shape_line_type(raw: u8) -> (StrokeDash, LineRenderType) {
    match raw {
        2 => (StrokeDash::HwpDash, LineRenderType::Single),
        3 => (StrokeDash::HwpDot, LineRenderType::Single),
        4 => (StrokeDash::HwpDashDot, LineRenderType::Single),
        5 => (StrokeDash::HwpDashDotDot, LineRenderType::Single),
        6 => (StrokeDash::HwpLongDash, LineRenderType::Single),
        7 => (StrokeDash::HwpCircle, LineRenderType::Single),
        _ => hwp_line_type(raw),
    }
}

pub(crate) fn hwp_line_type(raw: u8) -> (StrokeDash, LineRenderType) {
    match raw {
        2 | 6 => (StrokeDash::Dash, LineRenderType::Single),
        3 | 7 => (StrokeDash::Dot, LineRenderType::Single),
        4 => (StrokeDash::DashDot, LineRenderType::Single),
        5 => (StrokeDash::DashDotDot, LineRenderType::Single),
        8 => (StrokeDash::Solid, LineRenderType::Double),
        9 => (StrokeDash::Solid, LineRenderType::ThinThickDouble),
        10 => (StrokeDash::Solid, LineRenderType::ThickThinDouble),
        11 => (StrokeDash::Solid, LineRenderType::ThinThickThinTriple),
        _ => (StrokeDash::Solid, LineRenderType::Single),
    }
}
