//! OOXML 차트 → SVG 네이티브 렌더러
//!
//! `OoxmlChart` 데이터 모델을 지정된 bbox 안에 SVG 문자열로 그린다.
//! - 세로/가로 막대, 꺾은선, 원형
//! - **콤보 차트** (bar + line) 및 **이중 Y축** 지원

use super::{OoxmlChart, OoxmlChartType, OoxmlSeries};

/// 기본 시리즈 색상 팔레트 (시리즈 색상 미지정 시 순환 사용)
const DEFAULT_PALETTE: &[u32] = &[
    0xFF70AD47, 0xFF4472C4, 0xFFED7D31, 0xFFFFC000, 0xFF5B9BD5, 0xFFA5A5A5, 0xFF9013FE, 0xFF50E3C2,
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

/// 숫자 포맷 (#,##0 기본. 실수면 소수점 반올림)
fn format_num(v: f64, format_code: Option<&str>) -> String {
    let fc = format_code.unwrap_or("#,##0");
    let has_thousands = fc.contains(',');
    let _ = fc; // decimal handling 확장 여지
    let rounded = v.round() as i64;
    let abs = rounded.unsigned_abs();
    let sign = if rounded < 0 { "-" } else { "" };
    let s = abs.to_string();
    if !has_thousands {
        return format!("{}{}", sign, s);
    }
    // 콤마 구분
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    format!("{}{}", sign, out)
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

    // 영역 분할
    let title_h = if chart.title.is_some() { 22.0 } else { 4.0 };
    let legend_h = if chart.series.iter().any(|s| !s.name.is_empty()) {
        22.0
    } else {
        0.0
    };
    // 좌측 Y축 라벨용 여유: 실제 라벨 길이에 맞춰 조정
    let left_pad = estimate_axis_label_width(chart, 0);
    let right_pad = if chart.has_secondary_axis {
        estimate_axis_label_width(chart, 1)
    } else {
        16.0
    };
    let bottom_pad = 26.0;
    let plot_x = x + left_pad;
    let plot_y = y + title_h + 4.0;
    let plot_w = (w - left_pad - right_pad).max(10.0);
    let plot_h = (h - title_h - legend_h - bottom_pad).max(10.0);

    if let Some(ref title) = chart.title {
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"13\" font-weight=\"600\" fill=\"#222\" text-anchor=\"middle\">{}</text>\n",
            x + w / 2.0,
            y + title_h - 4.0,
            xml_escape(title)
        ));
    }

    // 파이 차트는 단독 경로
    if chart.chart_type == OoxmlChartType::Pie {
        render_pie(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
        render_legend(
            &mut svg,
            chart,
            x + 8.0,
            y + h - legend_h,
            w - 16.0,
            legend_h,
        );
        svg.push_str("</g>\n");
        return svg;
    }

    // 콤보 또는 이중축이면 조합 렌더
    if chart.is_combo() || chart.has_secondary_axis {
        render_combo(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
    } else {
        match chart.chart_type {
            OoxmlChartType::Column => {
                render_bars(&mut svg, chart, plot_x, plot_y, plot_w, plot_h, false)
            }
            OoxmlChartType::Bar => {
                render_bars(&mut svg, chart, plot_x, plot_y, plot_w, plot_h, true)
            }
            OoxmlChartType::Line => render_line(&mut svg, chart, plot_x, plot_y, plot_w, plot_h),
            _ => {}
        }
    }

    render_legend(
        &mut svg,
        chart,
        x + 8.0,
        y + h - legend_h,
        w - 16.0,
        legend_h,
    );
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

/// 지정한 axis_group의 최대 라벨 길이(문자 수) 기반으로 여백 추정
fn estimate_axis_label_width(chart: &OoxmlChart, axis_group: u8) -> f64 {
    let series: Vec<&OoxmlSeries> = chart
        .series
        .iter()
        .filter(|s| s.axis_group == axis_group)
        .collect();
    if series.is_empty() {
        return 16.0;
    }
    let (vmin, vmax) = value_range_for(series.iter().cloned());
    let fmt = series.first().and_then(|s| s.format_code.as_deref());
    let min_label = format_num(vmin, fmt);
    let max_label = format_num(vmax, fmt);
    let max_chars = min_label.chars().count().max(max_label.chars().count());
    // 숫자/콤마는 ~7px, 안전 여유 18px (좌우 플롯 영역 바깥 라벨 공간 확보)
    (max_chars as f64 * 7.0 + 18.0).max(28.0)
}

/// 시리즈 부분집합에 대한 값 범위
fn value_range_for<'a>(series: impl Iterator<Item = &'a OoxmlSeries>) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for s in series {
        for &v in &s.values {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
    }
    if !min.is_finite() {
        min = 0.0;
    }
    if !max.is_finite() {
        max = 1.0;
    }
    if min > 0.0 {
        min = 0.0;
    }
    if max == min {
        max = min + 1.0;
    }
    // Nice number 반올림 (눈금을 깔끔하게)
    let (min_n, max_n) = nice_range(min, max, 5);
    (min_n, max_n)
}

fn value_range(chart: &OoxmlChart) -> (f64, f64) {
    value_range_for(chart.series.iter())
}

/// min~max 구간을 "깔끔한" 눈금으로 확장
fn nice_range(min: f64, max: f64, target_ticks: usize) -> (f64, f64) {
    if max <= min {
        return (min, max);
    }
    let raw_step = (max - min) / target_ticks.max(1) as f64;
    let mag = 10f64.powf(raw_step.abs().log10().floor());
    let norm = raw_step / mag;
    let step = if norm < 1.5 {
        1.0
    } else if norm < 3.0 {
        2.0
    } else if norm < 7.0 {
        5.0
    } else {
        10.0
    };
    let step = step * mag;
    let new_min = (min / step).floor() * step;
    let new_max = (max / step).ceil() * step;
    (new_min, new_max)
}

// ---------------- Bar / Column (단일 축) ----------------

fn render_bars(
    svg: &mut String,
    chart: &OoxmlChart,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    horizontal: bool,
) {
    let (vmin, vmax) = value_range(chart);
    let cat_count = chart.categories.len().max(
        chart
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0),
    );
    if cat_count == 0 {
        return;
    }
    let ser_count = chart.series.len().max(1);

    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));

    render_value_grid(
        svg,
        px,
        py,
        pw,
        ph,
        vmin,
        vmax,
        chart.series.first().and_then(|s| s.format_code.as_deref()),
        horizontal,
        false,
    );

    let (cat_span, bar_span_total) = if horizontal {
        let span = ph / cat_count as f64;
        (span, span * 0.7)
    } else {
        let span = pw / cat_count as f64;
        (span, span * 0.7)
    };
    let bar_w = bar_span_total / ser_count as f64;

    for ci in 0..cat_count {
        for (si, ser) in chart.series.iter().enumerate() {
            let v = *ser.values.get(ci).unwrap_or(&0.0);
            let t = if vmax > vmin {
                (v - vmin) / (vmax - vmin)
            } else {
                0.0
            };
            let color = series_color(ser, si);
            if horizontal {
                let cy = py
                    + cat_span * ci as f64
                    + (cat_span - bar_span_total) / 2.0
                    + bar_w * si as f64;
                let bw = pw * t;
                svg.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                    px, cy, bw.max(0.0), bar_w * 0.95, color
                ));
            } else {
                let cx = px
                    + cat_span * ci as f64
                    + (cat_span - bar_span_total) / 2.0
                    + bar_w * si as f64;
                let bh = ph * t;
                let by = py + ph - bh;
                svg.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                    cx, by, bar_w * 0.95, bh.max(0.0), color
                ));
            }
        }
    }

    render_category_labels(svg, chart, px, py, pw, ph, cat_count, horizontal);
}

// ---------------- Line (단일 축) ----------------

fn render_line(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let (vmin, vmax) = value_range(chart);
    let max_len = chart
        .series
        .iter()
        .map(|s| s.values.len())
        .max()
        .unwrap_or(0);
    if max_len < 2 {
        return;
    }

    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));
    render_value_grid(
        svg,
        px,
        py,
        pw,
        ph,
        vmin,
        vmax,
        chart.series.first().and_then(|s| s.format_code.as_deref()),
        false,
        false,
    );

    let step = pw / (max_len - 1).max(1) as f64;
    for (si, ser) in chart.series.iter().enumerate() {
        let color = series_color(ser, si);
        let mut d = String::new();
        for (i, &v) in ser.values.iter().enumerate() {
            let t = if vmax > vmin {
                (v - vmin) / (vmax - vmin)
            } else {
                0.0
            };
            let xp = px + step * i as f64;
            let yp = py + ph - ph * t;
            d.push_str(&format!(
                "{}{:.2},{:.2} ",
                if i == 0 { "M" } else { "L" },
                xp,
                yp
            ));
        }
        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>\n",
            d.trim(),
            color
        ));
    }

    render_category_labels(svg, chart, px, py, pw, ph, max_len, false);
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

    let mut start_angle = -std::f64::consts::FRAC_PI_2;
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

// ---------------- Combo + Dual Axis ----------------

fn render_combo(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let cat_count = chart.categories.len().max(
        chart
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0),
    );
    if cat_count == 0 {
        return;
    }

    // 기본축/보조축 시리즈 분리
    let pri: Vec<&OoxmlSeries> = chart.series.iter().filter(|s| s.axis_group == 0).collect();
    let sec: Vec<&OoxmlSeries> = chart.series.iter().filter(|s| s.axis_group == 1).collect();

    let (pri_min, pri_max) = if pri.is_empty() {
        value_range(chart)
    } else {
        value_range_for(pri.iter().cloned())
    };
    let (sec_min, sec_max) = if sec.is_empty() {
        (0.0, 1.0)
    } else {
        value_range_for(sec.iter().cloned())
    };

    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));

    // 기본축 격자 (좌측)
    let pri_fmt = pri.first().and_then(|s| s.format_code.as_deref());
    render_value_grid(svg, px, py, pw, ph, pri_min, pri_max, pri_fmt, false, false);

    // 보조축 격자 (우측, 눈금만)
    if !sec.is_empty() {
        let sec_fmt = sec.first().and_then(|s| s.format_code.as_deref());
        render_value_grid(svg, px, py, pw, ph, sec_min, sec_max, sec_fmt, false, true);
    }

    // 막대 시리즈만 추려서 그룹화 렌더 (카테고리별 여러 바는 나란히)
    let bar_series: Vec<(usize, &OoxmlSeries)> = chart
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s.series_type, OoxmlChartType::Column | OoxmlChartType::Bar))
        .collect();
    let line_series: Vec<(usize, &OoxmlSeries)> = chart
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.series_type == OoxmlChartType::Line)
        .collect();

    let cat_span = pw / cat_count as f64;
    // 막대 그룹 너비를 더 좁혀 라인이 바 양옆으로 가려지지 않게 함
    let bar_group_w = cat_span * 0.55;
    let bar_w = if bar_series.is_empty() {
        0.0
    } else {
        bar_group_w / bar_series.len() as f64
    };

    // 막대 렌더 (각 시리즈 축 기준)
    for ci in 0..cat_count {
        for (bi, (si, ser)) in bar_series.iter().enumerate() {
            let v = *ser.values.get(ci).unwrap_or(&0.0);
            let (vmin, vmax) = if ser.axis_group == 1 {
                (sec_min, sec_max)
            } else {
                (pri_min, pri_max)
            };
            let t = if vmax > vmin {
                (v - vmin) / (vmax - vmin)
            } else {
                0.0
            };
            let color = series_color(ser, *si);
            let cx = px + cat_span * ci as f64 + (cat_span - bar_group_w) / 2.0 + bar_w * bi as f64;
            let bh = ph * t;
            let by = py + ph - bh;
            svg.push_str(&format!(
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                cx,
                by,
                (bar_w * 0.95).max(0.0),
                bh.max(0.0),
                color
            ));
        }
    }

    // 라인 렌더 (각자 축 기준) — 바보다 항상 위에 그려지고, 데이터 포인트 마커까지 표시
    let step = if cat_count > 1 {
        pw / (cat_count - 1) as f64
    } else {
        pw
    };
    let line_x_offset = cat_span / 2.0;
    for (si, ser) in &line_series {
        let (vmin, vmax) = if ser.axis_group == 1 {
            (sec_min, sec_max)
        } else {
            (pri_min, pri_max)
        };
        let color = series_color(ser, *si);
        let mut d = String::new();
        let mut points: Vec<(f64, f64)> = Vec::new();
        for (i, &v) in ser.values.iter().enumerate() {
            let t = if vmax > vmin {
                (v - vmin) / (vmax - vmin)
            } else {
                0.0
            };
            let xp = if !bar_series.is_empty() {
                px + cat_span * i as f64 + line_x_offset
            } else {
                px + step * i as f64
            };
            let yp = py + ph - ph * t;
            d.push_str(&format!(
                "{}{:.2},{:.2} ",
                if i == 0 { "M" } else { "L" },
                xp,
                yp
            ));
            points.push((xp, yp));
        }
        // 라인: 3px + 흰색 외곽 1px (바와 겹쳐도 선명하게)
        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"4\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>\n",
            d.trim()
        ));
        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2.5\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>\n",
            d.trim(), color
        ));
        // 데이터 포인트 마커
        for (xp, yp) in &points {
            svg.push_str(&format!(
                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"2.5\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
                xp, yp, color
            ));
        }
    }

    render_category_labels(svg, chart, px, py, pw, ph, cat_count, false);
}

// ---------------- 공통: 값 격자/라벨 ----------------

fn render_value_grid(
    svg: &mut String,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    vmin: f64,
    vmax: f64,
    format_code: Option<&str>,
    horizontal: bool,
    secondary: bool,
) {
    let grid_lines = 5;
    for i in 0..=grid_lines {
        let t = i as f64 / grid_lines as f64;
        if horizontal {
            let gx = px + pw * t;
            // 보조축일 때는 격자선 중복 방지, 라벨만
            if !secondary {
                svg.push_str(&format!(
                    "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#e8e8e8\" stroke-width=\"0.5\"/>\n",
                    gx, py, gx, py + ph
                ));
            }
            let v = vmin + (vmax - vmin) * t;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"middle\">{}</text>\n",
                gx, py + ph + 12.0, xml_escape(&format_num(v, format_code))
            ));
        } else {
            let gy = py + ph - ph * t;
            if !secondary {
                svg.push_str(&format!(
                    "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#e8e8e8\" stroke-width=\"0.5\"/>\n",
                    px, gy, px + pw, gy
                ));
            }
            let v = vmin + (vmax - vmin) * t;
            let (tx, anchor) = if secondary {
                (px + pw + 4.0, "start")
            } else {
                (px - 4.0, "end")
            };
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"{}\">{}</text>\n",
                tx, gy + 3.0, anchor, xml_escape(&format_num(v, format_code))
            ));
        }
    }
}

fn render_category_labels(
    svg: &mut String,
    chart: &OoxmlChart,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    cat_count: usize,
    horizontal: bool,
) {
    let cat_span = if horizontal {
        ph / cat_count as f64
    } else {
        pw / cat_count as f64
    };
    for (ci, cat) in chart.categories.iter().enumerate() {
        if ci >= cat_count {
            break;
        }
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
                cx, py + ph + 14.0, xml_escape(cat)
            ));
        }
    }
}

// ---------------- Legend ----------------

fn render_legend(svg: &mut String, chart: &OoxmlChart, x: f64, y: f64, w: f64, _h: f64) {
    let n = chart.series.len();
    if n == 0 {
        return;
    }
    let items: Vec<(String, u32, OoxmlChartType)> = match chart.chart_type {
        OoxmlChartType::Pie => {
            let first = chart.series.first();
            first
                .map(|s| {
                    s.values
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let label = chart
                                .categories
                                .get(i)
                                .cloned()
                                .unwrap_or_else(|| format!("항목 {}", i + 1));
                            let color = s.color.unwrap_or_else(|| palette(i));
                            (label, color, OoxmlChartType::Pie)
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        _ => chart
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let label = if s.name.is_empty() {
                    format!("시리즈 {}", i + 1)
                } else {
                    s.name.clone()
                };
                let color = s.color.unwrap_or_else(|| palette(i));
                (label, color, s.series_type)
            })
            .collect(),
    };

    // 가운데 정렬: 항목 개수로 총 너비 계산
    let item_w = 100.0_f64.min((w / items.len().max(1) as f64).max(60.0));
    let total_w = item_w * items.len() as f64;
    let start_x = x + (w - total_w) / 2.0;
    for (i, (label, color, stype)) in items.iter().enumerate() {
        let ix = start_x + item_w * i as f64;
        let cy = y + 11.0;
        // 라인 시리즈는 작은 선 + 점, 막대/파이는 사각형
        if *stype == OoxmlChartType::Line {
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"2\"/>\n",
                ix, cy, ix + 14.0, cy, color_hex(*color)
            ));
        } else {
            svg.push_str(&format!(
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" fill=\"{}\"/>\n",
                ix,
                y + 5.0,
                color_hex(*color)
            ));
        }
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\">{}</text>\n",
            ix + 18.0, y + 14.0, xml_escape(label)
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
                series_type: OoxmlChartType::Column,
                ..Default::default()
            }],
            categories: vec!["x".to_string(), "y".to_string(), "z".to_string()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("test"));
    }

    #[test]
    fn test_render_combo_dual_axis() {
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Column,
            has_secondary_axis: true,
            series: vec![
                OoxmlSeries {
                    name: "금액".into(),
                    values: vec![100.0, 200.0],
                    series_type: OoxmlChartType::Column,
                    axis_group: 0,
                    color: Some(0x70AD47),
                    ..Default::default()
                },
                OoxmlSeries {
                    name: "건수".into(),
                    values: vec![5.0, 10.0],
                    series_type: OoxmlChartType::Line,
                    axis_group: 1,
                    color: Some(0x4472C4),
                    ..Default::default()
                },
            ],
            categories: vec!["1월".into(), "2월".into()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 500.0, 300.0);
        assert!(svg.contains("<rect")); // 막대
        assert!(svg.contains("<path")); // 라인
        assert!(svg.contains("금액"));
        assert!(svg.contains("건수"));
    }

    #[test]
    fn test_format_num() {
        assert_eq!(format_num(1234.0, Some("#,##0")), "1,234");
        assert_eq!(format_num(-1234567.0, Some("#,##0")), "-1,234,567");
        assert_eq!(format_num(0.0, Some("#,##0")), "0");
        assert_eq!(format_num(123.0, None), "123");
    }

    #[test]
    fn test_color_hex() {
        assert_eq!(color_hex(0xFFFF00FF), "#ff00ff");
    }
}
