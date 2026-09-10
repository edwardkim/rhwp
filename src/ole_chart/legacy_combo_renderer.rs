//! Semantic rendering of supported legacy line/column, dual-axis charts.

use std::fmt::Write;

use super::legacy_presentation::{Axis, Presentation};
use crate::ole_chart::OleChart;

fn color(value: u32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        value & 255,
        (value >> 8) & 255,
        (value >> 16) & 255
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn tick(value: f64) -> String {
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub(super) fn render(
    chart: &OleChart,
    presentation: &Presentation,
    bounds: [f64; 4],
) -> Option<String> {
    let [x, y, width, height] = bounds;
    if !bounds.iter().all(|v| v.is_finite())
        || width <= 0.0
        || height <= 0.0
        || chart.series.len() != presentation.series.len()
        || chart.categories.is_empty()
        || chart
            .series
            .iter()
            .any(|s| s.values.len() != chart.categories.len())
    {
        return None;
    }
    let [ex, ey, er, eb] = presentation.extent;
    let [px, py, pr, pb] = presentation.plot;
    let sx = width / (er - ex);
    let sy = height / (eb - ey);
    // Stored sections include labels. Reserve gutters separately; these font
    // metrics and inner margins are not yet a native-layout replica.
    let left = (px - ex) * sx + width * 0.065;
    let top = (py - ey) * sy + height * 0.065;
    let right = (pr - ex) * sx - width * 0.035;
    let bottom = (pb - ey) * sy - height * 0.065;
    if ![left, top, right, bottom].iter().all(|v| v.is_finite())
        || left < 0.0
        || top < 0.0
        || right > width
        || bottom > height
        || right <= left
        || bottom <= top
    {
        return None;
    }
    let plot_width = right - left;
    let plot_height = bottom - top;
    let font = (height * 0.032).max(1.0);
    let step = plot_width / chart.categories.len() as f64;
    let position =
        |value: f64, axis: Axis| bottom - (value - axis.min) / (axis.max - axis.min) * plot_height;
    let mut svg = format!(
        "<svg x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" overflow=\"hidden\" data-ole-chart-presentation=\"legacy-mixed\"><rect width=\"{width}\" height=\"{height}\" fill=\"white\"/><g font-size=\"{font}\" fill=\"black\">"
    );
    for (side, axis) in presentation.axes.iter().copied().enumerate() {
        for i in 0..=axis.divisions {
            let value = axis.min + (axis.max - axis.min) * f64::from(i) / f64::from(axis.divisions);
            let ty = position(value, axis);
            if side == 0 {
                let _ = write!(
                    svg,
                    "<path d=\"M{left} {ty}H{right}\" stroke=\"black\" stroke-width=\"0.5\"/>"
                );
            }
            let tx = if side == 0 {
                left - font * 0.45
            } else {
                right + font * 0.45
            };
            let anchor = if side == 0 { "end" } else { "start" };
            let baseline = ty + font * 0.35;
            let _ = write!(
                svg,
                "<text x=\"{tx}\" y=\"{baseline}\" text-anchor=\"{anchor}\">{}</text>",
                tick(value)
            );
        }
    }
    let _ = write!(svg, "<path d=\"M{left} {top}V{bottom}H{right}V{top}\" fill=\"none\" stroke=\"black\" stroke-width=\"0.5\"/>");
    for (index, label) in chart.categories.iter().enumerate() {
        if label.trim().is_empty() {
            continue;
        }
        let tx = left + (index as f64 + 0.5) * step;
        let ty = bottom + font * 1.7;
        let _ = write!(
            svg,
            "<text x=\"{tx}\" y=\"{ty}\" text-anchor=\"middle\">{}</text>",
            escape(label)
        );
    }
    // A nested viewport clips marks without document-global clip-path IDs.
    let _ = write!(svg, "<svg x=\"{left}\" y=\"{top}\" width=\"{plot_width}\" height=\"{plot_height}\" viewBox=\"{left} {top} {plot_width} {plot_height}\" overflow=\"hidden\">");
    let bars = presentation
        .series
        .iter()
        .filter(|s| s.selectors[9] == 1)
        .count();
    let bar_width = step * 0.65 / bars as f64;
    let mut slot = 0;
    // Columns first: a later series must not obscure the line.
    for (series, style) in chart.series.iter().zip(&presentation.series) {
        if style.selectors[9] != 1 {
            continue;
        }
        let axis = presentation.axes[usize::from(style.secondary)];
        let baseline = position(0.0_f64.clamp(axis.min, axis.max), axis);
        for (index, value) in series.values.iter().copied().enumerate() {
            if !value.is_finite() {
                continue;
            }
            let vy = position(value.clamp(axis.min, axis.max), axis);
            let bx = left + (index as f64 + 0.5) * step - step * 0.325 + slot as f64 * bar_width;
            let by = vy.min(baseline);
            let bh = (vy - baseline).abs();
            let _ = write!(
                svg,
                "<rect x=\"{bx}\" y=\"{by}\" width=\"{bar_width}\" height=\"{bh}\" fill=\"{}\"/>",
                color(style.fill)
            );
        }
        slot += 1;
    }
    for (series, style) in chart.series.iter().zip(&presentation.series) {
        if style.selectors[9] != 6 {
            continue;
        }
        let axis = presentation.axes[usize::from(style.secondary)];
        let mut path = String::new();
        let mut connected = false;
        for (index, value) in series.values.iter().copied().enumerate() {
            let vy = position(value, axis);
            if !value.is_finite() || !vy.is_finite() {
                connected = false;
                continue;
            }
            let vx = left + (index as f64 + 0.5) * step;
            let command = if connected { 'L' } else { 'M' };
            let _ = write!(path, "{command}{vx} {vy}");
            connected = true;
        }
        let stroke_width = (style.width * sx).clamp(0.5, width * 0.05 + 0.5);
        let _ = write!(svg, "<path d=\"{path}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke_width}\" stroke-linejoin=\"round\"/>", color(style.color));
    }
    svg.push_str("</svg></g></svg>");
    Some(svg)
}
