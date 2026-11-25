//! 저장 VtChart의 백분율 누적 가로 막대를 범주 단위로 그린다.

use std::fmt::Write;

use super::legacy_presentation::{Pen, Presentation};
use super::svg_renderer::{escape_xml_text, font_attributes, series_fill, stored_chart_backdrop};
use super::OleChart;

pub(super) fn render(
    chart: &OleChart,
    p: &Presentation,
    width: f64,
    height: f64,
    dpi: f64,
) -> Option<String> {
    if p.chart_type != 11
        || p.axis_styles[1].percent_basis != 4
        || chart.series.len() != p.series.len()
        || chart.categories.is_empty()
        || p.series.iter().any(|series| series.secondary)
        || chart.series.iter().any(|s| {
            s.values.len() != chart.categories.len()
                || s.values
                    .iter()
                    .any(|value| !value.is_finite() || *value < 0.0)
        })
        || ![width, height, dpi]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        || p.legend_section.as_ref().is_some_and(|s| s.visible)
        || p.title.is_some()
    {
        return None;
    }
    let [el, eb, er, et] = p.extent;
    let [pl, pb, pr, pt] = p.plot;
    let (sx, sy) = (width / (er - el), height / (et - eb));
    let cat = p.labels[0].as_ref()?;
    let tick = p.labels[1].as_ref()?;
    if !cat.metrics.cache_valid || !tick.metrics.cache_valid {
        return None;
    }
    let inset = 40.0;
    let left = (pl - el - f64::from(cat.metrics.paint_rect[0]) + 2.0 * inset) * sx;
    let right = (pr - el - f64::from(p.plot_section.external)) * sx;
    let top = (et - pt + inset) * sy;
    let bottom = (et - pb + f64::from(tick.metrics.paint_rect[1]) - inset) * sy;
    if left < 0.0 || left >= right || right > width || top < 0.0 || top >= bottom || bottom > height
    {
        return None;
    }

    let count = chart.categories.len();
    let slot = (bottom - top) / count as f64;
    let plot_width = right - left;
    let bar_height = slot / (1.0 + p.x_gap);
    let category_font = font_attributes(Some(&cat.font), dpi, "");
    let tick_font = font_attributes(Some(&tick.font), dpi, "");
    let [category_left, _, category_right, _] = cat.metrics.paint_rect.map(f64::from);
    let (category_offset, category_anchor) = match cat.metrics.horizontal {
        0 => (category_left, "start"),
        1 => (category_right, "end"),
        2 => ((category_left + category_right) / 2.0, "middle"),
        _ => return None,
    };
    let category_x = left + category_offset * sx;
    let tick_y =
        bottom - f64::from(tick.metrics.paint_rect[3]) * sy + tick.font.size * dpi / 72.0 * 0.7;
    let mut svg = String::from("<g data-ole-chart-presentation=\"legacy-percent-bar\">");
    svg.push_str(&stored_chart_backdrop(p, width, height, dpi)?);
    for (index, category) in chart.categories.iter().enumerate() {
        let centre = top + index as f64 * slot + slot / 2.0;
        let bar_top = centre - bar_height / 2.0;
        let total: f64 = chart.series.iter().map(|series| series.values[index]).sum();
        if !total.is_finite() {
            return None;
        }
        let mut start = left;
        if total > 0.0 {
            for (series_index, series) in chart.series.iter().enumerate().rev() {
                let bar_width = series.values[index] / total * plot_width;
                if bar_width > 0.0 {
                    let fill = series_fill(Some(p), series_index);
                    write!(svg, "<rect x=\"{start}\" y=\"{bar_top}\" width=\"{bar_width}\" height=\"{bar_height}\" fill=\"{fill}\"/>").ok()?;
                    start += bar_width;
                }
            }
        }
        if p.axis_styles[0].labels_visible {
            let baseline = centre + cat.font.size * dpi / 72.0 * 0.3;
            write!(svg, "<text x=\"{category_x}\" y=\"{baseline}\" {category_font} text-anchor=\"{category_anchor}\">{}</text>", escape_xml_text(category)).ok()?;
        }
    }
    axis_line(
        &mut svg,
        p.axis_styles[0].pen,
        [left, top, left, bottom],
        dpi,
    )?;
    axis_line(
        &mut svg,
        p.axis_styles[1].pen,
        [left, bottom, right, bottom],
        dpi,
    )?;
    for index in 0..=p.axes[0].divisions {
        let fraction = f64::from(index) / f64::from(p.axes[0].divisions);
        let x = left + fraction * plot_width;
        if p.axis_styles[1].tick_style == 1 {
            let half_tick = p.axis_styles[1].tick_length * sy / 2.0;
            axis_line(
                &mut svg,
                p.axis_styles[1].pen,
                [x, bottom - half_tick, x, bottom + half_tick],
                dpi,
            )?;
        }
        if p.axis_styles[1].labels_visible {
            write!(svg, "<text x=\"{x}\" y=\"{tick_y}\" {tick_font} text-anchor=\"middle\">{fraction:.1}</text>").ok()?;
        }
    }
    svg.push_str("</g>");
    Some(svg)
}

fn axis_line(svg: &mut String, pen: Pen, [x1, y1, x2, y2]: [f64; 4], dpi: f64) -> Option<()> {
    if pen.enabled {
        let color = pen.color;
        let width = pen.width * dpi / 1440.0;
        write!(svg, "<path d=\"M{x1} {y1} L{x2} {y2}\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"{width}\"/>",
            color & 255, (color >> 8) & 255, (color >> 16) & 255).ok()?;
    }
    Some(())
}
