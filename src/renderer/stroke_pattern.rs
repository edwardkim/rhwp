//! 도형의 한컴 Print 반복 비율을 선언 선 굵기로 변환한다.

use super::StrokeDash;

mod border;
pub(crate) use border::border_dash;

// 독립 한컴 Print의 pen 해상도. HWPUNIT에서 반올림한 뒤 출력 좌표로 변환한다.
const HANCOM_PRINT_PEN_DPI: f64 = 600.0;
const STYLE_DPI: f64 = 96.0;

pub(crate) fn shape_pen_width_to_px(width_hu: i32, line_type: u8) -> f64 {
    if line_type == 0 {
        return 0.0;
    }
    if !(1..=7).contains(&line_type) {
        return f64::from(width_hu.max(0)) * STYLE_DPI / 7200.0;
    }
    let quantum_hu = 7200.0 / HANCOM_PRINT_PEN_DPI;
    let fill_hu = (f64::from(width_hu.max(0)) / quantum_hu).round().max(1.0) * quantum_hu;
    let outline_hu = if line_type > 1 { quantum_hu } else { 0.0 };
    (fill_hu + outline_hu) * STYLE_DPI / 7200.0
}

/// 글자 외곽선(1 실선, 2 점선, 3 굵은 실선, 4 파선, 5 일점쇄선, 6 이점쇄선)의 획 굵기와
/// 반복 구간(px). 0이면 외곽선이 없다.
///
/// 한/글 Mac 12.30 Print PDF는 글자 크기와 관계없이 1/600 inch pen으로 윤곽만 긋고
/// (굵은 실선만 2칸), 반복 구간도 같은 격자 칸 수다. 끝과 꺾임은 둥글고 안은 채우지 않는다.
pub(crate) fn char_outline_stroke(outline_type: u8) -> Option<(f64, Vec<f64>)> {
    let (width, intervals): (f64, &[f64]) = match outline_type {
        0 => return None,
        2 => (1.0, &[3.0, 3.0]),
        3 => (2.0, &[]),
        4 => (1.0, &[14.0, 6.0]),
        5 => (1.0, &[10.0, 6.0, 3.0, 6.0]),
        6 => (1.0, &[11.0, 3.0, 3.0, 3.0, 3.0, 3.0]),
        _ => (1.0, &[]),
    };
    let unit = STYLE_DPI / HANCOM_PRINT_PEN_DPI;
    Some((width * unit, intervals.iter().map(|v| v * unit).collect()))
}

impl StrokeDash {
    pub(crate) fn is_absolute_pattern(self) -> bool {
        self.is_hwp_pattern() || matches!(self, Self::Explicit(_))
    }

    pub(crate) fn is_hwp_pattern(self) -> bool {
        matches!(
            self,
            Self::HwpDash
                | Self::HwpDot
                | Self::HwpDashDot
                | Self::HwpDashDotDot
                | Self::HwpLongDash
                | Self::HwpCircle
        )
    }

    pub(crate) fn uses_round_cap(self) -> bool {
        self == Self::HwpCircle
    }

    /// 기존 범용 선은 종전 구간을 유지한다. 한컴 도형은 선 굵기의 배수다.
    pub(crate) fn intervals(self, width: f64) -> Vec<f64> {
        if let Self::Explicit(values) = self {
            return values
                .into_iter()
                .take_while(|value| *value > 0.0)
                .collect();
        }
        let values: &[f64] = match self {
            Self::Explicit(_) => unreachable!(),
            Self::Solid => &[],
            Self::Dash => &[6.0, 3.0],
            Self::Dot => &[2.0, 2.0],
            Self::DashDot => &[6.0, 3.0, 2.0, 3.0],
            Self::DashDotDot => &[6.0, 3.0, 2.0, 3.0, 2.0, 3.0],
            Self::HwpDash => &[7.5, 4.5],
            Self::HwpDot => &[1.5, 2.25],
            Self::HwpDashDot => &[15.0, 4.5, 1.5, 4.5],
            Self::HwpDashDotDot => &[15.0, 4.5, 1.5, 4.5, 1.5, 4.5],
            Self::HwpLongDash => &[15.0, 4.5],
            // 길이 0인 round cap의 지름은 선 굵기다.
            Self::HwpCircle => &[0.0, 2.25],
        };
        let outline = STYLE_DPI / HANCOM_PRINT_PEN_DPI;
        let scale = if self.is_hwp_pattern() {
            (width - outline).max(outline)
        } else {
            1.0
        };
        values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let edge = if self.is_hwp_pattern() && !self.uses_round_cap() {
                    if index % 2 == 0 {
                        outline
                    } else {
                        -outline
                    }
                } else {
                    0.0
                };
                value * scale + edge
            })
            .collect()
    }

    pub(crate) fn canvas_width(self, width: f64) -> f64 {
        if self.is_absolute_pattern() && width > 0.0 {
            width
        } else {
            width.max(0.5)
        }
    }
}
