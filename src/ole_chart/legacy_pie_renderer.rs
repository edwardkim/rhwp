//! 원형 차트의 계열별 저장 레이블과 슬라이스를 같은 순서로 그린다.

use std::{f64::consts::TAU, fmt::Write};

use super::legacy_presentation::{PointLabel, Presentation, Text};
use super::svg_renderer::{escape_xml_text, font_attributes, series_fill, stored_chart_backdrop};
use super::OleChart;

const INSIDE_LABEL_DISTANCE: f64 = 0.6;

struct Plot {
    left: f64,
    top: f64,
    sx: f64,
    sy: f64,
}

impl Plot {
    fn point(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        [(x - self.left) * self.sx, (self.top - y) * self.sy]
    }
}

struct Slice<'a> {
    angle: f64,
    sweep: f64,
    label: &'a PointLabel,
}

impl Slice<'_> {
    fn direction(&self) -> [f64; 2] {
        let (y, x) = (self.angle + self.sweep / 2.0).sin_cos();
        [x, y]
    }
}

pub(super) fn render(
    chart: &OleChart,
    p: &Presentation,
    width: f64,
    height: f64,
    dpi: f64,
) -> Option<String> {
    let settings = p.pie?;
    let [series] = chart.series.as_slice() else {
        return None;
    };
    if p.chart_type != 14
        || series.values.len() != p.series.len()
        || series.values.len() != chart.categories.len()
        || series.values.is_empty()
        || p.series.iter().any(|series| series.secondary)
        || ![width, height, dpi]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        || series.values.iter().any(|v| !v.is_finite() || *v < 0.0)
    {
        return None;
    }
    let total: f64 = series.values.iter().sum();
    if !total.is_finite() || total <= 0.0 {
        return None;
    }
    let [left, bottom, right, top] = p.extent;
    let plot = Plot {
        left,
        top,
        sx: width / (right - left),
        sy: height / (top - bottom),
    };
    let caption = p.title.as_ref().or(p.labels[0].as_ref())?;
    let [l, b, r, t] = p.plot;
    let [caption_l, caption_b, caption_r, caption_t] = caption.metrics.paint_rect.map(f64::from);
    let caption_height = caption_t - caption_b;
    if !caption.metrics.cache_valid || caption_height <= 0.0 || caption_l >= caption_r {
        return None;
    }
    let caption_clearance = caption_height + f64::from(caption.metrics.margin) + 40.0;
    let nominal = [(l + r) / 2.0, (b + t - caption_clearance) / 2.0];
    let mut angle = settings.start_degrees.to_radians();
    let direction = if settings.clockwise { -1.0 } else { 1.0 };
    let mut slices = Vec::with_capacity(p.series.len());
    for (value, style) in series.values.iter().zip(&p.series) {
        let sweep = value / total * TAU * direction;
        let label = style.point_label.as_deref()?;
        if !matches!(label.position, 0 | 5 | 6) || !label.text.metrics.cache_valid {
            return None;
        }
        slices.push(Slice {
            angle,
            sweep,
            label,
        });
        angle += sweep;
    }

    // 위치 5인 레이블의 저장 중심은 조각의 이등분선과 반지름의 60%가 만나는 점이다.
    // 서로 다른 이등분선 두 개 이상으로 공통 중심과 반지름을 복원한다.
    let inside = slices
        .iter()
        .filter(|s| s.label.position == 5 && s.sweep.abs() > 0.0)
        .collect::<Vec<_>>();
    let [cx, cy, radius] = if inside.len() >= 2 {
        solve_inside(&inside)?
    } else {
        let radius = slices
            .iter()
            .filter(|s| s.label.position == 6 && s.sweep.abs() > 0.0)
            .map(|s| {
                let [x, y, w, h] = label_box(&s.label.text);
                let [vx, vy] = s.direction();
                (x - nominal[0]) * vx + (y - nominal[1]) * vy
                    - (vx.abs() * w + vy.abs() * h) / 2.0
                    - f64::from(p.plot_section.internal)
            })
            .fold(f64::INFINITY, f64::min);
        [nominal[0], nominal[1], radius]
    };
    if [cx, cy, radius].iter().any(|v| !v.is_finite())
        || radius <= 0.0
        || cx - radius < l
        || cx + radius > r
        || cy - radius < b
        || cy + radius > t
    {
        return None;
    }

    let mut svg = String::from("<g data-ole-chart-presentation=\"legacy-pie\">");
    svg.push_str(&stored_chart_backdrop(p, width, height, dpi)?);
    let [x, y] = plot.point([cx, cy]);
    let rx = radius * plot.sx;
    let ry = radius * plot.sy;
    for (index, slice) in slices.iter().enumerate() {
        let fill = series_fill(Some(p), index);
        if slice.sweep.abs() >= TAU - 1e-9 {
            write!(
                svg,
                "<ellipse cx=\"{x}\" cy=\"{y}\" rx=\"{rx}\" ry=\"{ry}\" fill=\"{fill}\"/>"
            )
            .ok()?;
        } else if slice.sweep != 0.0 {
            let point = |a: f64| [x + rx * a.cos(), y - ry * a.sin()];
            let [x1, y1] = point(slice.angle);
            let [x2, y2] = point(slice.angle + slice.sweep);
            let long = u8::from(slice.sweep.abs() > std::f64::consts::PI);
            let clockwise = u8::from(slice.sweep < 0.0);
            write!(svg, "<path d=\"M{x} {y} L{x1} {y1} A{rx} {ry} 0 {long} {clockwise} {x2} {y2} Z\" fill=\"{fill}\"/>").ok()?;
        }
    }
    for slice in slices.iter().filter(|s| s.sweep != 0.0) {
        let text = &slice.label.text;
        let [x, y, w, h] = label_box(text);
        let anchor = if x >= left && x <= right && y >= bottom && y <= top {
            [x, y]
        } else {
            let [vx, vy] = slice.direction();
            let label_extent = (vx.abs() * w + vy.abs() * h) / 2.0;
            let distance = (radius - label_extent).max(radius * INSIDE_LABEL_DISTANCE);
            [cx + vx * distance, cy + vy * distance]
        };
        let [x, mut y] = plot.point(anchor);
        if slice.label.position == 6 {
            // 바깥 레이블은 저장 상자의 위쪽 여유와 첫 줄의 baseline을 분리한다.
            y -= f64::from(text.metrics.margin) * plot.sy / 2.0;
        }
        draw_text(&mut svg, text, [x, y], dpi, "middle")?;
    }
    let title_y = if caption_b < t && caption_t <= top {
        (caption_b + caption_t) / 2.0
    } else {
        top - 1.5 * caption_height
    };
    draw_text(&mut svg, caption, plot.point([cx, title_y]), dpi, "middle")?;
    if let Some(legend) = p.legend_section.as_ref().filter(|legend| legend.visible) {
        let [l, b, _, t] = legend.rect;
        let rows = p.series.len() as f64;
        let step = (t - b) / rows;
        let square = f64::from(legend.internal) * 2.0;
        for (index, s) in p.series.iter().enumerate() {
            let text = s.legend.as_ref()?;
            let centre = t - (index as f64 + 0.5) * step + square / 8.0;
            let left = l + f64::from(legend.internal) / 2.0;
            let [x, y] = plot.point([left, centre + square / 2.0]);
            let w = square * plot.sx;
            let h = square * plot.sy;
            let fill = series_fill(Some(p), index);
            write!(
                svg,
                "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"{fill}\"/>"
            )
            .ok()?;
            draw_text(
                &mut svg,
                text,
                plot.point([
                    left + square + f64::from(legend.internal) / 2.0,
                    centre + square / 5.0,
                ]),
                dpi,
                "start",
            )?;
        }
    }
    svg.push_str("</g>");
    Some(svg)
}

fn label_box(text: &Text) -> [f64; 4] {
    let [l, b, r, t] = text.metrics.paint_rect.map(f64::from);
    [(l + r) / 2.0, (b + t) / 2.0, r - l, t - b]
}

fn solve_inside(slices: &[&Slice<'_>]) -> Option<[f64; 3]> {
    let mut points = Vec::with_capacity(slices.len());
    let mut midpoint = [0.0; 2];
    let mut average = [0.0; 2];
    let count = slices.len() as f64;
    for slice in slices {
        let [x, y, _, _] = label_box(&slice.label.text);
        let unit = slice.direction();
        points.push(([x, y], unit));
        for (i, center) in [x, y].iter().enumerate() {
            midpoint[i] += center / count;
            average[i] += unit[i] / count;
        }
    }
    let mut covariance = 0.0;
    let mut variance = 0.0;
    for (point, unit) in &points {
        for i in 0..2 {
            let offset = unit[i] - average[i];
            covariance += (point[i] - midpoint[i]) * offset;
            variance += offset * offset;
        }
    }
    if variance <= 1e-6 {
        return None;
    }
    let distance = covariance / variance;
    let center = [
        midpoint[0] - average[0] * distance,
        midpoint[1] - average[1] * distance,
    ];
    // 캐시 사각형의 정수 절사 오차를 넘는 레이블로는 플롯을 복원하지 않는다.
    let roundoff = |p: [f64; 2], unit: [f64; 2]| {
        (p[0] - center[0] - unit[0] * distance).abs() <= 2.0
            && (p[1] - center[1] - unit[1] * distance).abs() <= 2.0
    };
    points
        .iter()
        .all(|(p, unit)| roundoff(*p, *unit))
        .then_some([center[0], center[1], distance / INSIDE_LABEL_DISTANCE])
}

fn draw_text(
    svg: &mut String,
    text: &Text,
    [x, y]: [f64; 2],
    dpi: f64,
    anchor: &str,
) -> Option<()> {
    if text.font.style != 0 || text.font.effects != 0 {
        return None;
    }
    let attrs = font_attributes(Some(&text.font), dpi, "");
    let lines = text.value.lines().collect::<Vec<_>>();
    let em = text.font.size * dpi / 72.0;
    let line_advance = em * 1.25;
    let first_baseline = y + em * 0.3 - line_advance * (lines.len().saturating_sub(1)) as f64 / 2.0;
    for (index, line) in lines.iter().enumerate() {
        let baseline = first_baseline + index as f64 * line_advance;
        write!(svg, "<text x=\"{x}\" y=\"{baseline}\" {attrs} text-anchor=\"{anchor}\" xml:space=\"preserve\">{}</text>", escape_xml_text(line)).ok()?;
    }
    Some(())
}
