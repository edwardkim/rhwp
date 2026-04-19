//! OOXML 차트 → SVG 네이티브 렌더러 (Task #195 단계 8)
//!
//! `OoxmlChart` 데이터 모델을 지정된 bbox 안에 SVG 문자열로 그린다.
//! 1차 범위: 세로/가로 막대, 꺾은선, 원형.

use super::{OoxmlChart, OoxmlChartType, OoxmlSeries};

/// 기본 시리즈 색상 팔레트 (시리즈 색상 미지정 시 순환 사용)
const DEFAULT_PALETTE: &[u32] = &[
    0xFF4A90E2, 0xFFF5A623, 0xFF7ED321, 0xFFBD10E0,
    0xFFD0021B, 0xFF417505, 0xFF9013FE, 0xFF50E3C2,
];

fn palette(i: usize) -> u32 {
    DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()]
}

fn color_hex(c: u32) -> String {
    format!("#{:06x}", c & 0xFFFFFF)
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 차트 전체를 SVG 조각으로 렌더
pub fn render_chart_svg(chart: &OoxmlChart, x: f64, y: f64, w: f64, h: f64) -> String {
    if chart.series.is_empty() || chart.chart_type == OoxmlChartType::Unknown {
        return render_fallback(chart, x, y, w, h);
    }

    let mut svg = String::new();
    svg.push_str(&format!(
        "<g class=\"hwp-ooxml-chart\"><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        x, y, w, h
    ));

    // 영역 분할: 제목 24px, 범례 24px, 나머지 플롯
    let title_h = if chart.title.is_some() { 22.0 } else { 4.0 };
    let legend_h = if chart.series.len() >= 1 { 20.0 } else { 0.0 };
    let plot_x = x + 48.0;
    let plot_y = y + title_h + 4.0;
    let plot_w = (w - 60.0).max(10.0);
    let plot_h = (h - title_h - legend_h - 28.0).max(10.0);

    // 타이틀
    if let Some(ref title) = chart.title {
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"13\" font-weight=\"600\" fill=\"#222\" text-anchor=\"middle\">{}</text>\n",
            x + w / 2.0,
            y + title_h - 4.0,
            xml_escape(title)
        ));
    }

    match chart.chart_type {
        OoxmlChartType::Column => render_bars(&mut svg, chart, plot_x, plot_y, plot_w, plot_h, false),
        OoxmlChartType::Bar => render_bars(&mut svg, chart, plot_x, plot_y, plot_w, plot_h, true),
        OoxmlChartType::Line => render_line(&mut svg, chart, plot_x, plot_y, plot_w, plot_h),
        OoxmlChartType::Pie => render_pie(&mut svg, chart, plot_x, plot_y, plot_w, plot_h),
        OoxmlChartType::Unknown => {}
    }

    // 범례
    render_legend(&mut svg, chart, x + 8.0, y + h - legend_h - 2.0, w - 16.0, legend_h);

    svg.push_str("</g>\n");
    svg
}

fn render_fallback(chart: &OoxmlChart, x: f64, y: f64, w: f64, h: f64) -> String {
    let label = format!("차트 ({})", chart.chart_type.label());
    format!(
        "<g class=\"hwp-ooxml-chart-fallback\"><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#f0f0f0\" stroke=\"#707070\" stroke-width=\"1\" stroke-dasharray=\"6 3\"/><text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#707070\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text></g>\n",
        x, y, w, h,
        x + w / 2.0, y + h / 2.0,
        xml_escape(&label)
    )
}

fn series_color(s: &OoxmlSeries, idx: usize) -> String {
    color_hex(s.color.unwrap_or_else(|| palette(idx)))
}

fn value_range(chart: &OoxmlChart) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for s in &chart.series {
        for &v in &s.values {
            if v < min { min = v; }
            if v > max { max = v; }
        }
    }
    if !min.is_finite() { min = 0.0; }
    if !max.is_finite() { max = 1.0; }
    // 0을 포함 (축 시작)
    if min > 0.0 { min = 0.0; }
    if max == min { max = min + 1.0; }
    (min, max)
}

// ---------------- Bar / Column ----------------

fn render_bars(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64, horizontal: bool) {
    let (vmin, vmax) = value_range(chart);

    // 카테고리 수: 첫 시리즈의 값 개수 또는 chart.categories
    let cat_count = chart.categories.len().max(
        chart.series.iter().map(|s| s.values.len()).max().unwrap_or(0)
    );
    if cat_count == 0 {
        return;
    }
    let ser_count = chart.series.len().max(1);

    // 플롯 배경
    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#fafafa\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));

    // 격자 (수평 또는 수직 4분할)
    let grid_lines = 4;
    for i in 0..=grid_lines {
        let t = i as f64 / grid_lines as f64;
        if horizontal {
            let gx = px + pw * t;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#e8e8e8\" stroke-width=\"0.5\"/>\n",
                gx, py, gx, py + ph
            ));
            let v = vmin + (vmax - vmin) * t;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"middle\">{:.0}</text>\n",
                gx, py + ph + 12.0, v
            ));
        } else {
            let gy = py + ph - ph * t;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#e8e8e8\" stroke-width=\"0.5\"/>\n",
                px, gy, px + pw, gy
            ));
            let v = vmin + (vmax - vmin) * t;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"end\">{:.0}</text>\n",
                px - 4.0, gy + 3.0, v
            ));
        }
    }

    // 각 카테고리 슬롯의 너비/높이
    let (cat_span, bar_span_total) = if horizontal {
        // 가로 막대: 세로로 카테고리 배열
        let span = ph / cat_count as f64;
        (span, span * 0.7)
    } else {
        // 세로 막대: 가로로 카테고리 배열
        let span = pw / cat_count as f64;
        (span, span * 0.7)
    };
    let bar_w = bar_span_total / ser_count as f64;

    for (ci, _cat) in (0..cat_count).zip(chart.categories.iter().chain(std::iter::repeat(&String::new())).take(cat_count)) {
        for (si, ser) in chart.series.iter().enumerate() {
            let v = *ser.values.get(ci).unwrap_or(&0.0);
            let t = if vmax > vmin { (v - vmin) / (vmax - vmin) } else { 0.0 };
            let color = series_color(ser, si);

            if horizontal {
                let cy = py + cat_span * ci as f64 + (cat_span - bar_span_total) / 2.0 + bar_w * si as f64;
                let bw = pw * t;
                svg.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                    px, cy, bw.max(0.0), bar_w * 0.95, color
                ));
            } else {
                let cx = px + cat_span * ci as f64 + (cat_span - bar_span_total) / 2.0 + bar_w * si as f64;
                let bh = ph * t;
                let by = py + ph - bh;
                svg.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                    cx, by, bar_w * 0.95, bh.max(0.0), color
                ));
            }
        }
    }

    // 카테고리 레이블
    for (ci, cat) in chart.categories.iter().enumerate() {
        if ci >= cat_count { break; }
        if horizontal {
            let cy = py + cat_span * ci as f64 + cat_span / 2.0 + 3.0;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\" text-anchor=\"end\">{}</text>\n",
                px - 4.0, cy, xml_escape(cat)
            ));
        } else {
            let cx = px + cat_span * ci as f64 + cat_span / 2.0;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\" text-anchor=\"middle\">{}</text>\n",
                cx, py + ph + 12.0, xml_escape(cat)
            ));
        }
    }
}

// ---------------- Line ----------------

fn render_line(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let (vmin, vmax) = value_range(chart);
    let max_len = chart.series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    if max_len < 2 {
        return;
    }

    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#fafafa\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));

    let step = pw / (max_len - 1) as f64;
    for (si, ser) in chart.series.iter().enumerate() {
        let color = series_color(ser, si);
        let mut d = String::new();
        for (i, &v) in ser.values.iter().enumerate() {
            let t = if vmax > vmin { (v - vmin) / (vmax - vmin) } else { 0.0 };
            let x = px + step * i as f64;
            let y = py + ph - ph * t;
            d.push_str(&format!("{}{:.2},{:.2} ", if i == 0 { "M" } else { "L" }, x, y));
        }
        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
            d.trim(), color
        ));
    }

    // 카테고리 레이블 (x축)
    for (ci, cat) in chart.categories.iter().enumerate() {
        if ci >= max_len { break; }
        let x = px + step * ci as f64;
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\" text-anchor=\"middle\">{}</text>\n",
            x, py + ph + 12.0, xml_escape(cat)
        ));
    }
}

// ---------------- Pie ----------------

fn render_pie(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let first = match chart.series.first() {
        Some(s) => s,
        None => return,
    };
    let total: f64 = first.values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let cx = px + pw / 2.0;
    let cy = py + ph / 2.0;
    let r = (pw.min(ph) / 2.0) * 0.9;

    let mut start_angle = -std::f64::consts::FRAC_PI_2; // 12시 방향 시작
    for (i, &v) in first.values.iter().enumerate() {
        let sweep = v / total * std::f64::consts::TAU;
        let end_angle = start_angle + sweep;
        let (x1, y1) = (cx + r * start_angle.cos(), cy + r * start_angle.sin());
        let (x2, y2) = (cx + r * end_angle.cos(), cy + r * end_angle.sin());
        let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
        let color = color_hex(first.color.unwrap_or_else(|| palette(i)));
        svg.push_str(&format!(
            "<path d=\"M{:.2},{:.2} L{:.2},{:.2} A{:.2},{:.2} 0 {} 1 {:.2},{:.2} Z\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
            cx, cy, x1, y1, r, r, large, x2, y2, color
        ));
        start_angle = end_angle;
    }
}

// ---------------- Legend ----------------

fn render_legend(svg: &mut String, chart: &OoxmlChart, x: f64, y: f64, w: f64, _h: f64) {
    let n = chart.series.len();
    if n == 0 {
        return;
    }
    // 파이 차트는 카테고리가 범례 역할
    let items: Vec<(String, u32)> = match chart.chart_type {
        OoxmlChartType::Pie => {
            let first = chart.series.first();
            first.map(|s| {
                s.values.iter().enumerate().map(|(i, _)| {
                    let label = chart.categories.get(i).cloned().unwrap_or_else(|| format!("항목 {}", i + 1));
                    let color = s.color.unwrap_or_else(|| palette(i));
                    (label, color)
                }).collect()
            }).unwrap_or_default()
        }
        _ => chart.series.iter().enumerate().map(|(i, s)| {
            let label = if s.name.is_empty() { format!("시리즈 {}", i + 1) } else { s.name.clone() };
            let color = s.color.unwrap_or_else(|| palette(i));
            (label, color)
        }).collect()
    };

    let item_w = if items.is_empty() { w } else { (w / items.len() as f64).min(120.0) };
    for (i, (label, color)) in items.iter().enumerate() {
        let ix = x + item_w * i as f64;
        svg.push_str(&format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" fill=\"{}\"/>\n",
            ix, y + 3.0, color_hex(*color)
        ));
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\">{}</text>\n",
            ix + 14.0, y + 12.0, xml_escape(label)
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_chart() {
        let chart = OoxmlChart::default();
        let svg = render_chart_svg(&chart, 0.0, 0.0, 100.0, 100.0);
        assert!(svg.contains("fallback"));
    }

    #[test]
    fn test_render_column() {
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Column,
            title: Some("test".to_string()),
            series: vec![OoxmlSeries {
                name: "A".to_string(),
                values: vec![1.0, 2.0, 3.0],
                color: None,
            }],
            categories: vec!["x".to_string(), "y".to_string(), "z".to_string()],
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("test"));
    }

    #[test]
    fn test_render_pie() {
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Pie,
            series: vec![OoxmlSeries {
                values: vec![30.0, 70.0],
                ..Default::default()
            }],
            categories: vec!["A".to_string(), "B".to_string()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 200.0, 200.0);
        assert!(svg.contains("<path"));
    }

    #[test]
    fn test_color_hex() {
        assert_eq!(color_hex(0xFFFF00FF), "#ff00ff");
    }
}
