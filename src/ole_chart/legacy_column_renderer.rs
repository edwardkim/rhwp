//! 저장된 소유 객체의 여백·레이블 점유·선 속성을 소비하는 2D 막대 차트.

use std::fmt::Write;

use super::legacy_column_layout::{tick_sides, ColumnLayout};
use super::legacy_presentation::{Backdrop, Font, Pen, Presentation};
use super::legacy_text_metrics::cached_size;
use super::svg_renderer::{escape_xml_text, font_attributes, format_axis_value, series_fill};
use super::OleChart;

fn color(value: u32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        value & 255,
        (value >> 8) & 255,
        (value >> 16) & 255
    )
}

fn plain(font: &Font) -> bool {
    font.style == 0 && font.effects == 0
}

fn supported(backdrop: Option<&Backdrop>) -> bool {
    backdrop.is_none_or(|b| !b.shadow_enabled && matches!(b.fill_kind, 0 | 1))
}

// 구형 Frame은 정수 참조 펜을 600dpi Print 장치에 전달한다.
// cosmetic 선은 참조 장치 한 픽셀의 coverage를 유지한다. 플롯의 예약 폭과는 별개다.
fn frame_stroke_width(twips: f64, dpi: f64) -> f64 {
    const FRAME_PRINT_DPI: f64 = 600.0;
    let reference_dpi = crate::renderer::DEFAULT_DPI;
    let pen = (twips.trunc() * reference_dpi / 1440.0).round().max(1.0);
    (pen * dpi / FRAME_PRINT_DPI).max(dpi / reference_dpi)
}

fn line(svg: &mut String, bounds: [f64; 4], pen: Pen, unit: f64) {
    if !pen.enabled {
        return;
    }
    let [x1, y1, x2, y2] = bounds;
    let _ = write!(svg, "<path d=\"M{x1:.4} {y1:.4}L{x2:.4} {y2:.4}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.4}\"/>", color(pen.color), pen.width * unit);
}

/// 제목·축·범례는 공통 배치 결과를 소비하며 글꼴과 펜의 물리 크기를 보존한다.
pub(super) fn render(
    chart: &OleChart,
    p: &Presentation,
    width: f64,
    height: f64,
    dpi: f64,
) -> Option<String> {
    if p.chart_type != 1
        || ![width, height, dpi]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        || chart.categories.is_empty()
        || chart.series.is_empty()
        || !supported(p.backdrop.as_ref())
        || !supported(p.plot_section.backdrop.as_ref())
        || !p.plot_section.visible
        || p.plot_section.location != 1
        || p.axis_styles.iter().any(|s| s.minor_grid.enabled)
    {
        return None;
    }
    let unit = dpi / 1440.0;
    if let Some(title) = p.title.as_ref() {
        let section = p.title_section.as_ref()?;
        if !plain(&title.font)
            || title.metrics.horizontal != 2
            || section.location != 1
            || !supported(section.backdrop.as_ref())
        {
            return None;
        }
        cached_size(title)?;
    }

    if let Some(font) = p.legend_font.as_ref() {
        let section = p.legend_section.as_ref()?;
        if !plain(font) || section.location != 6 || !supported(section.backdrop.as_ref()) {
            return None;
        }
    }

    let category = p.labels[0].as_ref()?;
    let value = p.labels[1].as_ref()?;
    if !plain(&category.font) || category.metrics.horizontal != 2 || !plain(&value.font) {
        return None;
    }
    let layout = ColumnLayout::new(p, width, height, dpi)?;
    let [left, top, right, bottom] = layout.plot;
    let plot_w = right - left;
    let plot_h = bottom - top;
    let axis = p.axes[0];
    let mut svg = String::new();
    if let Some(b) = p.backdrop.as_ref() {
        if b.fill_kind == 1 {
            let _ = write!(
                svg,
                "<rect width=\"{width:.4}\" height=\"{height:.4}\" fill=\"{}\"/>",
                color(b.fill_color)
            );
        }
        if b.frame_enabled {
            let stroke = frame_stroke_width(b.frame_width, dpi);
            let _ = write!(svg, "<rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke:.4}\"/>", stroke / 2.0, stroke / 2.0, width - stroke, height - stroke, color(b.frame_color));
        }
    }
    if let (Some(title), Some([x, y])) = (p.title.as_ref(), layout.title) {
        let attrs = font_attributes(Some(&title.font), dpi, "");
        let _ = write!(svg, "<text x=\"{:.4}\" y=\"{:.4}\" {attrs} text-anchor=\"middle\" dominant-baseline=\"central\" xml:space=\"preserve\">{}</text>", x, y, escape_xml_text(&title.value));
    }
    let value_attrs = font_attributes(Some(&value.font), dpi, "");
    let vertical = p.axis_styles[1];
    let horizontal = p.axis_styles[0];
    if layout.axes_visible {
        for i in 0..=axis.divisions {
            let ratio = f64::from(i) / f64::from(axis.divisions);
            let y = bottom - plot_h * ratio;
            line(&mut svg, [left, y, right, y], vertical.major_grid, unit);
            let length = vertical.tick_length * unit;
            let (inside, outside) = tick_sides(vertical.tick_style, length)?;
            line(
                &mut svg,
                [left - outside, y, left + inside, y],
                vertical.pen,
                unit,
            );
            if vertical.labels_visible {
                let number = axis.min + (axis.max - axis.min) * ratio;
                let label_right = left - layout.value_gap;
                let (label_x, anchor) = match value.metrics.horizontal {
                    0 => (label_right - layout.value_width, "start"),
                    1 => (label_right, "end"),
                    _ => (label_right - layout.value_width / 2.0, "middle"),
                };
                let _ = write!(svg, "<text x=\"{label_x:.4}\" y=\"{y:.4}\" {value_attrs} text-anchor=\"{anchor}\" dominant-baseline=\"central\">{}</text>", format_axis_value(number));
            }
        }
        line(&mut svg, [left, top, left, bottom], vertical.pen, unit);
        line(
            &mut svg,
            [left, bottom, right, bottom],
            horizontal.pen,
            unit,
        );
    }
    let category_attrs = font_attributes(Some(&category.font), dpi, "");
    let cat_w = plot_w / chart.categories.len() as f64;
    let (inside, outside) = tick_sides(horizontal.tick_style, horizontal.tick_length * unit)?;
    if layout.axes_visible {
        for index in 0..=chart.categories.len() {
            let x = left + index as f64 * cat_w;
            line(
                &mut svg,
                [x, bottom - inside, x, bottom + outside],
                horizontal.pen,
                unit,
            );
        }
        for (index, label) in chart.categories.iter().enumerate() {
            let x = left + (index as f64 + 0.5) * cat_w;
            let y = bottom + layout.category_offset;
            if horizontal.labels_visible {
                let _ = write!(svg, "<text x=\"{x:.4}\" y=\"{y:.4}\" {category_attrs} text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>", escape_xml_text(label));
            }
        }
    }
    let count = chart.series.len();
    let bar_w = cat_w / (count as f64 + (count - 1) as f64 * p.bar_gap + p.x_gap);
    let position = |number: f64| bottom - (number - axis.min) / (axis.max - axis.min) * plot_h;
    let zero = position(0.0_f64.clamp(axis.min, axis.max));
    for (series_index, series) in chart.series.iter().enumerate() {
        for (index, value) in series.values.iter().copied().enumerate() {
            if !value.is_finite() {
                continue;
            }
            let x = left
                + index as f64 * cat_w
                + bar_w * p.x_gap / 2.0
                + series_index as f64 * bar_w * (1.0 + p.bar_gap);
            let end = position(value.clamp(axis.min, axis.max));
            let _ = write!(svg, "<rect x=\"{x:.4}\" y=\"{:.4}\" width=\"{bar_w:.4}\" height=\"{:.4}\" fill=\"{}\"/>", end.min(zero), (end - zero).abs(), series_fill(Some(p), series_index));
        }
    }
    if let Some(legend) = layout.legend {
        let attrs = font_attributes(p.legend_font.as_ref(), dpi, "");
        for (index, _) in chart.series.iter().enumerate() {
            let [x, y] = legend.origin(index);
            let size = legend.marker;
            let label = p.series[index].legend.as_ref()?;
            let name = &label.value;
            let [label_x, label_y] = legend.label_origin(index);
            let _ = write!(svg, "<rect x=\"{x:.4}\" y=\"{y:.4}\" width=\"{size:.4}\" height=\"{size:.4}\" fill=\"{}\"/><text x=\"{label_x:.4}\" y=\"{label_y:.4}\" {attrs} dominant-baseline=\"central\">{}</text>", series_fill(Some(p), index), escape_xml_text(name));
        }
    }
    Some(svg)
}
