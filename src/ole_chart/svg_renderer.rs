//! Canonical Rust SVG renderer for parsed OLE charts.

use super::ir::{OLE_CHART_IR_SCHEMA, OLE_CHART_IR_VERSION};
use super::legacy_presentation::{Font, Presentation};
use super::{OleChart, OleChartType};

pub fn render_ole_chart_svg_fragment(
    chart: &OleChart,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    bin_data_id: u32,
) -> String {
    render_fragment(chart, None, [x, y, width, height], bin_data_id, 96.0)
}

pub(super) fn render_legacy_column_fragment(
    chart: &OleChart,
    presentation: &Presentation,
    bounds: [f64; 4],
    bin_data_id: u32,
    dpi: f64,
) -> String {
    if let Some(body) =
        super::legacy_column_renderer::render(chart, presentation, bounds[2], bounds[3], dpi)
    {
        return format!("<g class=\"hwp-ole-chart hwp-ole-chart-rust-svg\" data-rhwp-ole-chart-renderer=\"rust-svg\" data-rhwp-ole-chart-schema=\"{}\" data-rhwp-ole-chart-version=\"{}\" data-rhwp-bin-data-id=\"{}\" transform=\"translate({:.2} {:.2})\">{body}</g>", OLE_CHART_IR_SCHEMA, OLE_CHART_IR_VERSION, bin_data_id, bounds[0], bounds[1]);
    }
    render_fragment(chart, Some(presentation), bounds, bin_data_id, dpi)
}

fn render_fragment(
    chart: &OleChart,
    presentation: Option<&Presentation>,
    [x, y, width, height]: [f64; 4],
    bin_data_id: u32,
    dpi: f64,
) -> String {
    let safe_x = finite_or(x, 0.0);
    let safe_y = finite_or(y, 0.0);
    let safe_w = positive_finite_or(width, 1.0);
    let safe_h = positive_finite_or(height, 1.0);
    let body = render_body(chart, presentation, safe_w, safe_h, dpi);

    format!(
        "<g class=\"hwp-ole-chart hwp-ole-chart-rust-svg\" \
         data-rhwp-ole-chart-renderer=\"rust-svg\" \
         data-rhwp-ole-chart-schema=\"{}\" data-rhwp-ole-chart-version=\"{}\" \
         data-rhwp-bin-data-id=\"{}\" transform=\"translate({:.2} {:.2})\">{}</g>",
        OLE_CHART_IR_SCHEMA, OLE_CHART_IR_VERSION, bin_data_id, safe_x, safe_y, body,
    )
}

pub fn render_ole_chart_standalone_svg(chart: &OleChart, width: u32, height: u32) -> String {
    let width = width.max(1);
    let height = height.max(1);
    let body = render_ole_chart_svg_body(chart, width as f64, height as f64);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">{}</svg>",
        width, height, width, height, body
    )
}

pub fn render_ole_chart_svg_body(chart: &OleChart, width: f64, height: f64) -> String {
    render_body(chart, None, width, height, 96.0)
}

fn render_body(
    chart: &OleChart,
    presentation: Option<&Presentation>,
    width: f64,
    height: f64,
    dpi: f64,
) -> String {
    let width = positive_finite_or(width, 1.0);
    let height = positive_finite_or(height, 1.0);

    if chart.categories.is_empty() || chart.series.is_empty() {
        return format!(
            "<rect x=\"0\" y=\"0\" width=\"{width:.2}\" height=\"{height:.2}\" fill=\"#fff7ed\" \
             stroke=\"#b45309\" stroke-width=\"1\" stroke-dasharray=\"6 3\"/>\
             <text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"12\" \
             fill=\"#92400e\" text-anchor=\"middle\" dominant-baseline=\"central\">OLE chart</text>",
            width / 2.0,
            height / 2.0,
        );
    }

    // 레거시 제목은 소유 객체의 원문과 표시 상태를 따른다.
    // 그리드 탐색의 문자열 정규화로 전각 공백을 소실하지 않는다.
    let title = match presentation {
        Some(p) => p.title.as_ref().map(|title| title.value.as_str()),
        None => chart.title.as_deref(),
    };
    let title_h = if title.is_some() { 24.0 } else { 8.0 };
    let legend_h = if chart.series.len() > 1 { 24.0 } else { 8.0 };
    let left = 42.0f64.min(width * 0.18).max(28.0);
    let right = 16.0f64.min(width * 0.08).max(8.0);
    let top = title_h;
    let bottom = 30.0 + legend_h;
    let plot_w = (width - left - right).max(1.0);
    let plot_h = (height - top - bottom).max(1.0);
    let data_max = chart
        .series
        .iter()
        .flat_map(|series| series.values.iter())
        .copied()
        .filter(|value| value.is_finite() && *value > 0.0)
        .fold(0.0, f64::max)
        .max(1.0);
    let min_value = presentation.map_or(0.0, |p| p.axes[0].min);
    let max_value = presentation.map_or(data_max, |p| p.axes[0].max);
    let divisions = presentation.map_or(4, |p| usize::from(p.axes[0].divisions));

    let mut svg = String::new();
    svg.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"{width:.2}\" height=\"{height:.2}\" fill=\"#ffffff\"/>\
         <rect x=\"0.5\" y=\"0.5\" width=\"{:.2}\" height=\"{:.2}\" fill=\"none\" stroke=\"#cbd5e1\" stroke-width=\"1\"/>",
        (width - 1.0).max(0.0),
        (height - 1.0).max(0.0),
    ));

    if let Some(title) = title {
        let attrs = font_attributes(
            presentation.and_then(|p| p.title.as_ref()).map(|t| &t.font),
            dpi,
            "font-family=\"sans-serif\" font-size=\"13\" font-weight=\"600\" fill=\"#111827\"",
        );
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"16\" {attrs} text-anchor=\"middle\" xml:space=\"preserve\">{}</text>",
            width / 2.0,
            escape_xml_text(title),
        ));
    }

    let value_font = font_attributes(
        presentation.and_then(|p| p.label_fonts[1].as_ref()),
        dpi,
        "font-family=\"sans-serif\" font-size=\"9\" fill=\"#64748b\"",
    );
    for i in 0..=divisions {
        let ratio = i as f64 / divisions as f64;
        let y = top + plot_h - plot_h * ratio;
        let value = min_value + (max_value - min_value) * ratio;
        svg.push_str(&format!(
            "<line x1=\"{left:.2}\" y1=\"{y:.2}\" x2=\"{:.2}\" y2=\"{y:.2}\" stroke=\"#e5e7eb\" stroke-width=\"1\"/>\
             <text x=\"{:.2}\" y=\"{:.2}\" {value_font} \
             text-anchor=\"end\">{}</text>",
            left + plot_w,
            left - 4.0,
            y + 3.0,
            format_axis_value(value),
        ));
    }

    svg.push_str(&format!(
        "<line x1=\"{left:.2}\" y1=\"{top:.2}\" x2=\"{left:.2}\" y2=\"{:.2}\" stroke=\"#94a3b8\" stroke-width=\"1\"/>\
         <line x1=\"{left:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#94a3b8\" stroke-width=\"1\"/>",
        top + plot_h,
        top + plot_h,
        left + plot_w,
        top + plot_h,
    ));

    match chart.chart_type {
        OleChartType::Line => {
            render_line_series(&mut svg, chart, left, top, plot_w, plot_h, max_value)
        }
        _ => render_bar_series(
            &mut svg,
            chart,
            [left, top, plot_w, plot_h],
            max_value,
            presentation,
        ),
    }

    render_category_labels(
        &mut svg,
        chart,
        [left, top, plot_w, plot_h],
        presentation,
        dpi,
    );
    render_legend(
        &mut svg,
        chart,
        left,
        top + plot_h + 28.0,
        plot_w,
        presentation,
        dpi,
    );
    svg
}

fn render_bar_series(
    svg: &mut String,
    chart: &OleChart,
    [left, top, plot_w, plot_h]: [f64; 4],
    max_value: f64,
    presentation: Option<&Presentation>,
) {
    let cat_count = chart.categories.len().max(1);
    let series_count = chart.series.len().max(1);
    let cat_w = plot_w / cat_count as f64;
    // 한 범주의 가용 폭을 실제 막대들과 저장된 계열/범주 간격이 나눠 쓴다.
    // gap0인 이웃 계열에는 출력 단계에서 별도의1px 틈을 만들지 않는다.
    let (bar_w, series_gap, group_pad, paint_width) = match presentation {
        Some(p) => {
            let slots = series_count as f64 + (series_count - 1) as f64 * p.bar_gap + p.x_gap;
            let bar_w = cat_w / slots;
            (bar_w, bar_w * p.bar_gap, bar_w * p.x_gap / 2.0, bar_w)
        }
        None => {
            let bar_w = (cat_w * 0.72 / series_count as f64).max(1.0);
            (
                bar_w,
                0.0,
                (cat_w - bar_w * series_count as f64) / 2.0,
                (bar_w - 1.0).max(1.0),
            )
        }
    };

    for (cat_idx, _) in chart.categories.iter().enumerate() {
        for (series_idx, series) in chart.series.iter().enumerate() {
            let raw_value = series.values.get(cat_idx).copied().unwrap_or(0.0);
            let min_value = presentation.map_or(0.0, |p| p.axes[0].min);
            let value = if presentation.is_some() {
                raw_value
            } else {
                raw_value.max(0.0)
            };
            if !value.is_finite() {
                continue;
            }
            let baseline =
                (0.0_f64.clamp(min_value, max_value) - min_value) / (max_value - min_value);
            let value_ratio =
                (value.clamp(min_value, max_value) - min_value) / (max_value - min_value);
            let bar_h = plot_h * (value_ratio - baseline).abs();
            let x = left
                + cat_w * cat_idx as f64
                + group_pad
                + (bar_w + series_gap) * series_idx as f64;
            let y = top + plot_h * (1.0 - value_ratio.max(baseline));
            svg.push_str(&format!(
                "<rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{:.2}\" height=\"{bar_h:.2}\" \
                 fill=\"{}\"/>",
                paint_width,
                series_fill(presentation, series_idx),
            ));
        }
    }
}

fn render_line_series(
    svg: &mut String,
    chart: &OleChart,
    left: f64,
    top: f64,
    plot_w: f64,
    plot_h: f64,
    max_value: f64,
) {
    let cat_count = chart.categories.len().max(1);
    let step = if cat_count > 1 {
        plot_w / (cat_count - 1) as f64
    } else {
        0.0
    };

    for (series_idx, series) in chart.series.iter().enumerate() {
        let mut points = Vec::new();
        for cat_idx in 0..cat_count {
            let value = series.values.get(cat_idx).copied().unwrap_or(0.0).max(0.0);
            let x = left + step * cat_idx as f64;
            let y = top + plot_h - plot_h * (value / max_value).clamp(0.0, 1.0);
            points.push((x, y));
        }
        if points.is_empty() {
            continue;
        }
        let mut d = format!("M {:.2} {:.2}", points[0].0, points[0].1);
        for (x, y) in points.iter().skip(1) {
            d.push_str(&format!(" L {x:.2} {y:.2}"));
        }
        let color = chart_color(series_idx);
        svg.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\"/>"
        ));
        for (x, y) in points {
            svg.push_str(&format!(
                "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"2.4\" fill=\"{color}\"/>"
            ));
        }
    }
}

fn render_category_labels(
    svg: &mut String,
    chart: &OleChart,
    [left, top, plot_w, plot_h]: [f64; 4],
    presentation: Option<&Presentation>,
    dpi: f64,
) {
    let attrs = font_attributes(
        presentation.and_then(|p| p.label_fonts[0].as_ref()),
        dpi,
        "font-family=\"sans-serif\" font-size=\"9\" fill=\"#334155\"",
    );
    let cat_count = chart.categories.len().max(1);
    let cat_w = plot_w / cat_count as f64;
    for (idx, label) in chart.categories.iter().enumerate() {
        let x = left + cat_w * (idx as f64 + 0.5);
        svg.push_str(&format!(
            "<text x=\"{x:.2}\" y=\"{:.2}\" {attrs} text-anchor=\"middle\">{}</text>",
            top + plot_h + 14.0,
            escape_xml_text(label),
        ));
    }
}

fn render_legend(
    svg: &mut String,
    chart: &OleChart,
    left: f64,
    y: f64,
    plot_w: f64,
    presentation: Option<&Presentation>,
    dpi: f64,
) {
    if chart.series.is_empty() || presentation.is_some_and(|p| p.legend_font.is_none()) {
        return;
    }
    let attrs = font_attributes(
        presentation.and_then(|p| p.legend_font.as_ref()),
        dpi,
        "font-family=\"sans-serif\" font-size=\"9\" fill=\"#334155\"",
    );
    let mut x = left + 4.0;
    let max_x = left + plot_w;
    for (idx, series) in chart.series.iter().enumerate() {
        let name = series.name.as_deref().unwrap_or("Series");
        if x > max_x - 38.0 {
            break;
        }
        svg.push_str(&format!(
            "<rect x=\"{x:.2}\" y=\"{:.2}\" width=\"9\" height=\"9\" fill=\"{}\"/>\
             <text x=\"{:.2}\" y=\"{:.2}\" {attrs}>{}</text>",
            y - 8.0,
            series_fill(presentation, idx),
            x + 13.0,
            y,
            escape_xml_text(name),
        ));
        x += 58.0;
    }
}

pub(super) fn font_attributes(font: Option<&Font>, dpi: f64, fallback: &str) -> String {
    // 미검증 스타일 플래그 해석으로 굵은꼴이나 기울임꼴을 추정하지 않는다.
    let Some(font) = font.filter(|font| font.style == 0 && font.effects == 0) else {
        return fallback.to_owned();
    };
    let size = font.size * positive_finite_or(dpi, 96.0) / 72.0;
    let color = font.color;
    format!(
        "font-family=\"{}\" font-size=\"{size:.4}\" font-weight=\"normal\" fill=\"#{:02x}{:02x}{:02x}\"",
        escape_xml_text(&font.family),
        color & 255,
        (color >> 8) & 255,
        (color >> 16) & 255,
    )
}

pub(super) fn series_fill(presentation: Option<&Presentation>, index: usize) -> String {
    match presentation.and_then(|p| p.series.get(index)) {
        Some(series) => format!(
            "#{:02x}{:02x}{:02x}",
            series.fill & 255,
            (series.fill >> 8) & 255,
            (series.fill >> 16) & 255,
        ),
        None => chart_color(index).to_string(),
    }
}

fn chart_color(idx: usize) -> &'static str {
    const COLORS: &[&str] = &[
        "#2563eb", "#dc2626", "#16a34a", "#9333ea", "#ea580c", "#0891b2",
    ];
    COLORS[idx % COLORS.len()]
}

pub(super) fn format_axis_value(value: f64) -> String {
    if value >= 1000.0 {
        format!("{value:.0}")
    } else if value.fract().abs() < 1e-9 {
        format!("{}", value as i64)
    } else {
        format!("{value:.1}")
    }
}

pub(super) fn escape_xml_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            // XML 1.0 허용 문자: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
            // 그 외(제어문자, U+FFFE, U+FFFF 등)는 제거 — 방출된 SVG 가 불법 XML 이 되지 않도록 (#3382)
            '\u{09}' | '\u{0A}' | '\u{0D}' => escaped.push(ch),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                escaped.push(ch)
            }
            _ => {} // XML 무효 문자 제거
        }
    }
    escaped
}

fn positive_finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ole_chart::{OleChartSeries, OleChartType, SeriesAxis, SeriesAxisEvidence};

    fn sample_chart() -> OleChart {
        OleChart {
            chart_type: OleChartType::Unknown,
            title: Some("연금 재정 전망".to_string()),
            categories: vec!["2010년".to_string(), "2020년".to_string()],
            series: vec![OleChartSeries {
                name: Some("적립금".to_string()),
                values: vec![328.0, 812.0],
            }],
            series_axis: SeriesAxis::Columns,
            series_axis_evidence: SeriesAxisEvidence::ColumnLabelsEchoed,
        }
    }

    #[test]
    fn renders_svg_fragment() {
        let svg = render_ole_chart_svg_fragment(&sample_chart(), 10.0, 20.0, 300.0, 200.0, 2);

        assert!(svg.contains("hwp-ole-chart-rust-svg"));
        assert!(svg.contains("data-rhwp-ole-chart-renderer=\"rust-svg\""));
        assert!(svg.contains("transform=\"translate(10.00 20.00)\""));
        assert!(svg.contains("연금 재정 전망"));
        assert!(svg.contains("적립금"));
    }

    #[test]
    fn renders_standalone_svg() {
        let svg = render_ole_chart_standalone_svg(&sample_chart(), 300, 200);

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("연금 재정 전망"));
        assert!(svg.contains("적립금"));
    }
}
