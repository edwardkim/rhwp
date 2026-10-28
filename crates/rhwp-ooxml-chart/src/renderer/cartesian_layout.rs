//! 차트의 축 범위·글줄 예약을 측정과 표시가 같은 값으로 소비한다.

use super::*;

/// 차트의 기본 안쪽 여백은 기본 글자 크기의 물리 단위다.
/// 축 글자를 확대해도 프레임과 마지막 범주 글줄 사이의 여백은 변하지 않는다.
pub(super) const FRAME_PAD: f64 = hancom_default::LABEL_PT * PX_PER_PT;
pub(super) const TICK_GAP: f64 = FRAME_PAD * 0.25;

pub(super) fn title_height(chart: &OoxmlChart, has_title: bool) -> f64 {
    let horizontal = chart.chart_type == OoxmlChartType::Bar && !chart.is_combo() && !chart.is_3d;
    if !has_title {
        // 제목이 없으면 제목 글줄을 만들지 않는다. 프레임 안쪽 여백과
        // 세로 값축의 최상단 라벨 반 줄만 남긴다.
        return FRAME_PAD
            + if horizontal {
                0.0
            } else {
                label_font_px(chart) * 0.5
            };
    }
    let text = title_font_px(chart) * 1.2 + if horizontal { 0.0 } else { TICK_GAP };
    // 세로 값축의 최상단 숫자는 격자 밖으로 반 줄을 차지한다.
    // 가로 차트의 범주 라벨은 각 밴드 안에 있으므로 같은 공간을 예약하지 않는다.
    FRAME_PAD * 2.0
        + text.max(FRAME_PAD * 1.2)
        + if horizontal {
            0.0
        } else {
            label_font_px(chart) * 0.5
        }
}

pub(super) fn category_text_bottom(font: f64) -> f64 {
    font * 2.0 + TICK_GAP
}

#[derive(Clone, Copy)]
pub(super) struct ValueScale {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl ValueScale {
    fn from_tuple((min, max, step): (f64, f64, f64)) -> Self {
        Self { min, max, step }
    }

    pub fn tuple(self) -> (f64, f64, f64) {
        (self.min, self.max, self.step)
    }

    /// 원점 밖 값의 포함 여부와 축선 위치는 같은 범위에서 결정한다.
    pub fn zero_fraction(self) -> f64 {
        if self.max > self.min {
            (0.0_f64.clamp(self.min, self.max) - self.min) / (self.max - self.min)
        } else {
            0.0
        }
    }

    pub fn label_width(
        self,
        chart: &OoxmlChart,
        group: u8,
        measure: &dyn Fn(&str, f64, &str) -> f64,
    ) -> f64 {
        let horizontal = chart.chart_type == OoxmlChartType::Bar && !chart.is_combo();
        let axis = axis_at(
            chart,
            if group == 1 {
                AxisPos::Right
            } else if horizontal {
                AxisPos::Bottom
            } else {
                AxisPos::Left
            },
        );
        let fmt = axis.format_code.as_deref().or_else(|| {
            chart
                .series
                .iter()
                .find(|s| s.axis_group == group)
                .and_then(|s| s.format_code.as_deref())
        });
        let fmt = if chart.chart_type == OoxmlChartType::Scatter {
            None
        } else {
            fmt
        };
        let font = label_font_px(chart);
        let family = chart.text_font_family.as_deref().unwrap_or("sans-serif");
        let steps = if self.step > 0.0 {
            (((self.max - self.min) / self.step) + 1e-9)
                .floor()
                .clamp(0.0, 2000.0) as usize
        } else {
            0
        };
        let percent = chart.grouping == BarGrouping::PercentStacked
            || (chart.chart_type == OoxmlChartType::Line
                && chart.line_grouping == BarGrouping::PercentStacked);
        (0..=steps)
            .map(|i| {
                let value = self.min + i as f64 * self.step;
                let decimal = chart.chart_type == OoxmlChartType::Scatter
                    || (self.step - self.step.round()).abs() > 1e-9;
                let text = format_grid_value(value, fmt, percent, decimal);
                measure(&text, font, family)
            })
            .fold(0.0, f64::max)
    }
}

/// 두 글줄 사이에 들어가는 눈금 수와 양 끝을 포함해 원 데이터에 최대 열 칸을 배정한다.
/// 10% 범위 여유는 표시 범위에 포함하며 원 데이터의 최대값을 축 끝에 붙이지 않는다.
fn automatic_scale(min: f64, max: f64, length: f64, font: f64) -> ValueScale {
    if length.abs() <= f64::EPSILON {
        return ValueScale::from_tuple((0.0, 0.0, 1.0));
    }
    let min = min.min(0.0);
    let max = max.max(0.0);
    let span = (max - min).max(f64::EPSILON);
    let line_height = axis_line_height(font);
    let budget = (1.0 + (length.abs() / (line_height * 2.0)).round()).clamp(3.0, 10.0);
    let raw_step = span / budget;
    let magnitude = 10.0_f64.powf(raw_step.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|n| n * magnitude)
        .find(|n| *n > raw_step * (1.0 + 1e-9))
        .unwrap_or(magnitude * 20.0);
    let padding = span * 0.1;
    ValueScale {
        min: if min < 0.0 {
            ((min - padding) / step).floor() * step
        } else {
            0.0
        },
        max: if max > 0.0 {
            ((max + padding) / step).ceil() * step
        } else {
            0.0
        },
        step,
    }
}

/// 3D 세로 축은 최대 다섯 칸을 두고 큰 글줄에는 더 성긴 단위를 고른다.
/// 묶은 막대의 원래 최댓값이 눈금에 닿으면 그 범위를 유지하되 두 칸은 확보한다.
fn automatic_3d_vertical(min: f64, max: f64, height: f64, font: f64) -> ValueScale {
    let min = min.min(0.0);
    let max = max.max(0.0);
    if max <= min {
        return ValueScale::from_tuple((min, max, 1.0));
    }
    let line_height = font * crate::legend_layout::TEXT_LINE_EM;
    let budget = (1.0 + (height.abs() / (line_height * 2.0)).round()).clamp(2.0, 5.0);
    let raw_step = (max - min) / budget;
    let magnitude = 10.0_f64.powf(raw_step.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|n| n * magnitude)
        .find(|step| *step >= raw_step * (1.0 - 1e-9))
        .unwrap_or(magnitude * 10.0);
    let mut scaled_min = (min / step).floor() * step;
    let mut scaled_max = (max / step).ceil() * step;
    // 큰 글자의 두 칸 예산에서도 단위 선택 후 한 칸만 남길 수 있다.
    // 한컴의 자동 축은 같은 단위를 유지하며 데이터 바깥으로 두 칸을 확보한다.
    if scaled_max - scaled_min < step * 2.0 {
        if max > 0.0 {
            scaled_max = scaled_min + step * 2.0;
        } else {
            scaled_min = scaled_max - step * 2.0;
        }
    }
    ValueScale {
        min: scaled_min,
        max: scaled_max,
        step,
    }
}

/// 범위의 양 끝을 유지하며 기존 눈금의 부분집합으로 글줄 겹침을 피한다.
fn fit_3d_vertical_ticks(scale: &mut ValueScale, height: f64, font: f64) -> bool {
    let span = scale.max - scale.min;
    if span <= 0.0 || scale.step <= 0.0 {
        return false;
    }
    let count = span / scale.step;
    let intervals = count.round();
    if (count - intervals).abs() > 1e-7 || !(2.0..=2000.0).contains(&intervals) {
        return false;
    }
    let intervals = intervals as usize;
    let capacity = (height / (font * crate::legend_layout::TEXT_LINE_EM))
        .floor()
        .max(1.0) as usize;
    if intervals <= capacity {
        return false;
    }
    let factor = (2..=intervals)
        .find(|factor| intervals.is_multiple_of(*factor) && intervals / factor <= capacity)
        .unwrap_or(intervals);
    scale.step *= factor as f64;
    true
}

#[derive(Clone, Copy)]
pub(super) struct ChartScales {
    pub primary: ValueScale,
    pub secondary: ValueScale,
}

impl ChartScales {
    pub fn new(chart: &OoxmlChart, width: f64, height: f64) -> Self {
        let font = label_font_px(chart);
        let (min, max) = raw_value_bounds(chart.series.iter());
        if chart.chart_type == OoxmlChartType::Scatter {
            return Self {
                primary: ValueScale::from_tuple(scatter_range(
                    chart.series.iter().flat_map(|s| s.values.iter().copied()),
                )),
                secondary: ValueScale::from_tuple(scatter_range(
                    chart.series.iter().flat_map(|s| s.x_values.iter().copied()),
                )),
            };
        }
        let mut primary = automatic_scale(min, max, height, font);
        let secondary = if chart.is_combo() || chart.has_secondary_axis {
            let pri: Vec<_> = chart.series.iter().filter(|s| s.axis_group == 0).collect();
            let sec: Vec<_> = chart.series.iter().filter(|s| s.axis_group == 1).collect();
            if !pri.is_empty() {
                let (min, max) = raw_value_bounds(pri.into_iter());
                primary = automatic_scale(min, max, height, font);
            }
            if sec.is_empty() {
                ValueScale::from_tuple((0.0, 1.0, 0.2))
            } else {
                let (min, max) = raw_value_bounds(sec.into_iter());
                automatic_scale(min, max, height, font)
            }
        } else {
            let bars = matches!(
                chart.chart_type,
                OoxmlChartType::Column | OoxmlChartType::Bar
            );
            let grouping = if chart.chart_type == OoxmlChartType::Line {
                chart.line_grouping
            } else {
                chart.grouping
            };
            let horizontal = chart.chart_type == OoxmlChartType::Bar;
            let ticks = if horizontal {
                HORIZONTAL_AXIS_TICKS
            } else {
                VERTICAL_AXIS_TICKS
            };
            let length = if horizontal { width } else { height };
            if grouping == BarGrouping::PercentStacked {
                primary = ValueScale::from_tuple((0.0, 100.0, 20.0));
            } else if grouping == BarGrouping::Stacked {
                let count = chart
                    .series
                    .iter()
                    .map(|s| s.values.len())
                    .max()
                    .unwrap_or(0);
                let positive = (0..count)
                    .map(|ci| category_positive_sum(chart, ci))
                    .fold(0.0, f64::max);
                let negative = (0..count)
                    .map(|ci| {
                        chart
                            .series
                            .iter()
                            .map(|s| s.values.get(ci).copied().unwrap_or(0.0).min(0.0))
                            .sum::<f64>()
                    })
                    .fold(0.0, f64::min);
                primary = if chart.is_3d && bars {
                    let (min, max, step) = nice_axis(0.0, positive.max(1.0), ticks);
                    ValueScale::from_tuple((min, max + if horizontal { 0.0 } else { step }, step))
                } else {
                    automatic_scale(negative, positive.max(f64::EPSILON), length, font)
                };
            } else if chart.is_3d && bars {
                primary = if horizontal {
                    ValueScale::from_tuple(nice_axis_no_headroom(min, max, HORIZONTAL_AXIS_TICKS))
                } else {
                    automatic_3d_vertical(min, max, height, font)
                };
            } else if chart.chart_type == OoxmlChartType::Stock {
                let (min, max, step) = nice_axis_no_headroom(min, max, VERTICAL_AXIS_TICKS);
                primary = ValueScale::from_tuple((min, max + step, step));
            } else if horizontal {
                primary = automatic_scale(min, max, length, font);
            }
            // 저장 범위는 측정 뒤 paint에서 덮어쓰지 않고 이 결과에서 확정한다.
            if bars {
                let axis = axis_at(
                    chart,
                    if horizontal {
                        AxisPos::Bottom
                    } else {
                        AxisPos::Left
                    },
                );
                if let Some(min) = axis.minimum {
                    primary.min = min;
                }
                if let Some(max) = axis.maximum.filter(|max| *max > primary.min) {
                    primary.max = max;
                }
                if axis.minimum.is_some() || axis.maximum.is_some() {
                    let intervals = if axis.format_code.as_deref().is_some_and(|s| s.contains('%'))
                    {
                        5.0
                    } else {
                        ticks
                    };
                    primary.step = if chart.is_3d && !horizontal {
                        automatic_3d_vertical(primary.min, primary.max, height, font).step
                    } else {
                        (primary.max - primary.min) / intervals
                    };
                }
            }
            ValueScale::from_tuple((0.0, 1.0, 0.2))
        };
        Self { primary, secondary }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct CategoryLabelLayout {
    pub rotation_degrees: f64,
    pub maximum_advance: f64,
    pub axis_fraction: f64,
    pub label_stride: usize,
}

impl CategoryLabelLayout {
    pub fn new(chart: &OoxmlChart, width: f64, measure: &dyn Fn(&str, f64, &str) -> f64) -> Self {
        let font = label_font_px(chart);
        let family = chart.text_font_family.as_deref().unwrap_or("sans-serif");
        let maximum_advance = chart
            .categories
            .iter()
            .map(|s| measure(s, font, family))
            .fold(0.0, f64::max);
        let count = chart.categories.len().max(
            chart
                .series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0),
        );
        let needs_rotation = matches!(
            chart.chart_type,
            OoxmlChartType::Column | OoxmlChartType::Line | OoxmlChartType::Stock
        ) && count > 0
            && maximum_advance + TICK_GAP * 2.0 > width / count as f64;
        let rotation_degrees = if !needs_rotation {
            0.0
        } else if (maximum_advance + font) * std::f64::consts::FRAC_1_SQRT_2 <= width / count as f64
        {
            45.0
        } else {
            90.0
        };
        Self {
            rotation_degrees,
            maximum_advance,
            axis_fraction: 0.0,
            label_stride: 1,
        }
    }

    /// 세로 글줄은 축 방향으로 한 em을 점유한다. 3D도 투영이 끝난 축 너비를
    /// 사용하며, 표시만 건너뛰고 원래 범주·막대·눈금의 소속과 간격은 유지한다.
    fn fit_vertical_labels(&mut self, chart: &OoxmlChart, axis_width: f64) {
        if self.rotation_degrees != 90.0 {
            return;
        }
        let count = chart.categories.len().max(
            chart
                .series
                .iter()
                .map(|series| series.values.len())
                .max()
                .unwrap_or(0),
        );
        if count == 0 {
            return;
        }
        let pitch = axis_width / count as f64;
        self.label_stride = if pitch > 0.0 {
            (label_font_px(chart) / pitch)
                .ceil()
                .clamp(1.0, count as f64) as usize
        } else {
            count
        };
    }

    pub fn bottom_reservation(self, chart: &OoxmlChart, crosses_inside: bool) -> f64 {
        let font = label_font_px(chart);
        if crosses_inside {
            FRAME_PAD + font * 0.5
        } else if self.rotation_degrees > 0.0 {
            let radians = self.rotation_degrees.to_radians();
            FRAME_PAD + font + self.maximum_advance * radians.sin() + font * radians.cos()
        } else {
            FRAME_PAD + category_text_bottom(font)
        }
    }

    pub fn axis_y(self, y: f64, height: f64) -> f64 {
        y + height * (1.0 - self.axis_fraction)
    }
}

/// 반복 계산에서 예약한 여백과 최종 축·그리기 프레임을 함께 반환한다.
pub(super) struct CartesianLayout {
    pub left: f64,
    pub width: f64,
    pub height: f64,
    pub scales: ChartScales,
    pub category: CategoryLabelLayout,
    pub scene: Option<super::chart_3d::SceneFrame>,
}

impl CartesianLayout {
    pub fn new(
        chart: &OoxmlChart,
        width: f64,
        height: f64,
        title_height: f64,
        legend_height: f64,
        legend_width: f64,
        has_title: bool,
        has_side_legend: bool,
        measure: &dyn Fn(&str, f64, &str) -> f64,
    ) -> Self {
        let font = label_font_px(chart);
        let horizontal = chart.chart_type == OoxmlChartType::Bar
            && !chart.is_combo()
            && !chart.has_secondary_axis;
        let pie = chart.chart_type == OoxmlChartType::Pie;
        let raw_min = raw_value_bounds(chart.series.iter()).0;
        let crosses_inside = !horizontal
            && !chart.is_3d
            && axis_at(chart, AxisPos::Left).minimum.unwrap_or(raw_min) < 0.0;
        let mut left: f64 = 0.0;
        let mut right: f64 = 0.0;
        let mut rotation_degrees: f64 = 0.0;
        loop {
            let plot_width = (width - left - right - legend_width).max(0.0);
            let mut category = CategoryLabelLayout::new(chart, plot_width, measure);
            if category.rotation_degrees > rotation_degrees {
                // 방향이 바뀌면 이전 방향의 숫자 라벨 예약은 다시 계산한다.
                // 이 이후의 너비 감소로 이미 세운 글줄을 수평으로 되돌리지는 않는다.
                rotation_degrees = category.rotation_degrees;
                left = 0.0;
                right = 0.0;
                continue;
            }
            category.rotation_degrees = rotation_degrees;
            let bottom = if pie {
                font * hancom_default::PIE_BOTTOM_PAD_EM
            } else {
                category.bottom_reservation(chart, crosses_inside)
            };
            let remaining_height = height - title_height - legend_height - bottom;
            let plot_height = if pie || chart.is_3d {
                remaining_height.max(0.0)
            } else {
                remaining_height
            };
            let mut scales = ChartScales::new(chart, plot_width, plot_height);
            if crosses_inside || (horizontal && !chart.is_3d) {
                category.axis_fraction = scales.primary.zero_fraction();
            }
            let needed_left = if pie {
                0.0
            } else if horizontal {
                category.maximum_advance + FRAME_PAD * 2.05
            } else {
                scales.primary.label_width(chart, 0, measure) + FRAME_PAD * 1.8
            };
            let needed_right = if chart.has_secondary_axis {
                scales.secondary.label_width(chart, 1, measure) + FRAME_PAD * 1.8
            } else if horizontal {
                // 마지막 수치 라벨은 축 끝을 중심으로 놓인다.
                // 중간의 소수 눈금이 정수 끝점보다 넓어도 같은 숫자열로 예약한다.
                scales.primary.label_width(chart, 0, measure) * 0.5 + FRAME_PAD * 1.3
            } else if chart.chart_type == OoxmlChartType::Scatter {
                let family = chart.text_font_family.as_deref().unwrap_or("sans-serif");
                measure(&format_num(scales.secondary.max, None), font, family) * 0.5
                    + FRAME_PAD * 1.3
            } else if pie {
                font * hancom_default::PIE_RIGHT_PAD_EM
            } else if has_side_legend {
                FRAME_PAD
            } else {
                // 옆 범례와의 분리 여백은 범례가 있을 때만 필요하다.
                // 하단 범례·범례 없음은 기본 바깥 여백을 소비한다.
                FRAME_PAD * 0.6
            };
            // 같은 숫자 예산과 라벨 방향에는 같은 글자들이 생긴다. 필요한 여백만
            // 단조롭게 예약하여 축 간격 변경이 앞선 측정 결과를 무효화하지 않게 한다.
            if needed_left <= left + 1e-7 && needed_right <= right + 1e-7 {
                let scene = loop {
                    let scene = super::chart_3d::SceneFrame::new(
                        chart,
                        [width, height],
                        [plot_width, plot_height],
                        [title_height, legend_height, legend_width],
                        has_title,
                        category,
                        scales,
                        measure,
                    );
                    if let Some(frame) = &scene {
                        let [projected_width, projected_height] = frame.projected_size(chart);
                        if category.rotation_degrees == 0.0 && !horizontal {
                            let projected =
                                CategoryLabelLayout::new(chart, projected_width, measure);
                            if projected.rotation_degrees > 0.0 {
                                rotation_degrees = projected.rotation_degrees;
                                break None;
                            }
                        }
                        if !horizontal
                            && fit_3d_vertical_ticks(&mut scales.primary, projected_height, font)
                        {
                            continue;
                        }
                    }
                    break scene;
                };
                if category.rotation_degrees < rotation_degrees {
                    left = 0.0;
                    right = 0.0;
                    continue;
                }
                if let Some(frame) = &scene {
                    category.fit_vertical_labels(chart, frame.projected_size(chart)[0]);
                } else if !chart.is_3d {
                    category.fit_vertical_labels(chart, plot_width);
                }
                return Self {
                    left,
                    width: plot_width,
                    height: plot_height,
                    scales,
                    category,
                    scene,
                };
            }
            left = left.max(needed_left);
            right = right.max(needed_right);
        }
    }
}
