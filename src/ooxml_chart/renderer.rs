//! OOXML 차트 → SVG 네이티브 렌더러
//!
//! `OoxmlChart` 데이터 모델을 지정된 bbox 안에 SVG 문자열로 그린다.
//! - 세로/가로 막대, 꺾은선, 원형
//! - **콤보 차트** (bar + line) 및 **이중 Y축** 지원

use super::{BarGrouping, LegendPos, OoxmlChart, OoxmlChartType, OoxmlSeries, ScatterStyle};

/// 기본 시리즈 색상 팔레트 (시리즈 색상 미지정 시 순환 사용)
///
/// 한컴 2022 기본 팔레트(`hncChartStyle colorIndex="0"`) — 앞 4색은 `pdf/chart/` 정답지
/// PDF 픽셀 실측(막대 3시리즈 + 원형 4슬라이스), 5번째 이후는 코퍼스에 4시리즈 초과
/// 샘플이 없어 미실측(Office 유사색 순서로 유추 배치).
const DEFAULT_PALETTE: &[u32] = &[
    0xFF6183D7, // 파랑 (실측)
    0xFFFE813B, // 주황 (실측)
    0xFFB0B0B0, // 회색 (실측)
    0xFFFCD801, // 노랑 (실측)
    0xFF5B9BD5, // 하늘 (유추)
    0xFF70AD47, // 초록 (유추)
    0xFF9013FE, 0xFF50E3C2,
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

/// 분산형 수치축 눈금용 소수 포맷. 정수면 소수점 없이, 아니면 소수 2자리 후 trailing 0 제거.
/// (`format_num`은 정수 반올림이라 0.5/2.6 등 소수 눈금을 손상시키므로 별도 헬퍼) — C1b #1660.
fn format_axis_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        return format!("{}", v.round() as i64);
    }
    let mut s = format!("{:.2}", v);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
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

    // C1c #1882 갭①: 명시 제목이 없어도 c:title 요소가 있고 autoTitleDeleted=0이면
    // 한컴처럼 자동 제목 placeholder "차트 제목"을 그린다 (정답지 PDF 실측).
    // 자동 제목 우선순위(#1882 v2): 명시 텍스트 → 단일 시리즈면 그 이름 → "차트 제목".
    // 한컴 실측: 원형 5종("판매")·단일 시리즈 가로막대("계열 1") 정답지가 시리즈
    // 이름을 제목으로 렌더 — 차트 종류가 아니라 시리즈 수 기준 (Excel 동작과 동일).
    let effective_title: Option<String> = chart.title.clone().or_else(|| {
        (chart.has_title_elem && !chart.auto_title_deleted).then(|| match &chart.series[..] {
            [only] if !only.name.is_empty() => only.name.clone(),
            _ => "차트 제목".to_string(),
        })
    });

    // 영역 분할
    let title_h = if effective_title.is_some() { 22.0 } else { 4.0 };
    let legend_visible = chart.series.iter().any(|s| !s.name.is_empty());
    // C1c #1882 갭③: legendPos=r(한컴 코퍼스 전 샘플)은 우측 세로 스택 — 하단 슬롯
    // 대신 우측 폭(legend_w)을 확보. 그 외 위치는 현행 하단 가로 유지.
    // `w * 0.30 >= 50.0` 가드: 폭이 좁으면(<167px) 하단 폴백 — 아래 clamp의
    // min(50)>max(w*0.30) 패닉 방지 (w는 문서 데이터가 결정). NaN도 false → 폴백.
    let legend_right = legend_visible && chart.legend_pos == LegendPos::Right && w * 0.30 >= 50.0;
    let legend_h = if legend_visible && !legend_right {
        22.0
    } else {
        0.0
    };
    let legend_w = if legend_right {
        let max_chars = legend_items(chart)
            .iter()
            .map(|(label, _, _)| label.chars().count())
            .max()
            .unwrap_or(0);
        // 스와치 10 + 간격 8 + CJK ~10px/자 (플롯 최소폭은 아래 .max(10.0)이 방어)
        (max_chars as f64 * 10.0 + 26.0).clamp(50.0, w * 0.30)
    } else {
        0.0
    };
    // 좌측 여유: 세로 차트는 값축 숫자 라벨, **가로 막대는 카테고리 라벨**("항목 1" 등)이
    // 좌측에 오므로 카테고리 폭 기준 — 숫자 폭(2자≈32px)으로 잡으면 라벨이 잘림.
    let horizontal_bars =
        chart.chart_type == OoxmlChartType::Bar && !chart.is_combo() && !chart.has_secondary_axis;
    let left_pad = if horizontal_bars {
        estimate_category_label_width(chart, w)
    } else {
        estimate_axis_label_width(chart, 0)
    };
    let right_pad = if chart.has_secondary_axis {
        estimate_axis_label_width(chart, 1)
    } else {
        16.0
    };
    let bottom_pad = 26.0;
    let plot_x = x + left_pad;
    let plot_y = y + title_h + 4.0;
    let plot_w = (w - left_pad - right_pad - legend_w).max(10.0);
    let plot_h = (h - title_h - legend_h - bottom_pad).max(10.0);

    if let Some(ref title) = effective_title {
        // 한컴 제목은 regular weight (정답지 PDF 실측 — C1c #1882 갭①)
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"13\" font-weight=\"400\" fill=\"#222\" text-anchor=\"middle\">{}</text>\n",
            x + w / 2.0,
            y + title_h - 4.0,
            xml_escape(title)
        ));
    }

    // 파이 차트는 단독 경로
    if chart.chart_type == OoxmlChartType::Pie {
        render_pie(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
        if legend_right {
            render_legend_right(&mut svg, chart, x + w - legend_w + 4.0, plot_y, plot_h);
        } else {
            render_legend(
                &mut svg,
                chart,
                x + 8.0,
                y + h - legend_h,
                w - 16.0,
                legend_h,
            );
        }
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
            OoxmlChartType::Scatter => {
                render_scatter(&mut svg, chart, plot_x, plot_y, plot_w, plot_h)
            }
            _ => {}
        }
    }

    if legend_right {
        render_legend_right(&mut svg, chart, x + w - legend_w + 4.0, plot_y, plot_h);
    } else {
        render_legend(
            &mut svg,
            chart,
            x + 8.0,
            y + h - legend_h,
            w - 16.0,
            legend_h,
        );
    }
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

/// 가로 막대 좌측 카테고리 라벨용 여백: 최장 카테고리 문자 수 기반 (CJK ~10px/자).
/// 상한은 차트 폭의 35%(플롯 최소폭은 호출부 `.max(10.0)`이 방어).
fn estimate_category_label_width(chart: &OoxmlChart, w: f64) -> f64 {
    let max_chars = chart
        .categories
        .iter()
        .map(|c| c.chars().count())
        .max()
        .unwrap_or(0);
    (max_chars as f64 * 10.0 + 14.0)
        .min((w * 0.35).max(28.0))
        .max(28.0)
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
    let (vmin, vmax, _) = value_range_for(series.iter().cloned(), VERTICAL_AXIS_TICKS);
    let fmt = series.first().and_then(|s| s.format_code.as_deref());
    let min_label = format_num(vmin, fmt);
    let max_label = format_num(vmax, fmt);
    let max_chars = min_label.chars().count().max(max_label.chars().count());
    // 숫자/콤마는 ~7px, 안전 여유 18px (좌우 플롯 영역 바깥 라벨 공간 확보)
    (max_chars as f64 * 7.0 + 18.0).max(28.0)
}

/// 시리즈 부분집합의 원시 값 범위 (0-baseline clamp + 퇴화 방어, nice 반올림 전)
fn raw_value_bounds<'a>(series: impl Iterator<Item = &'a OoxmlSeries>) -> (f64, f64) {
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
    (min, max)
}

/// 시리즈 부분집합에 대한 값 범위 `(min, max, step)`. `target_ticks`는 축 방향별
/// 눈금 밀도(`VERTICAL_AXIS_TICKS`/`HORIZONTAL_AXIS_TICKS`).
fn value_range_for<'a>(
    series: impl Iterator<Item = &'a OoxmlSeries>,
    target_ticks: f64,
) -> (f64, f64, f64) {
    let (min, max) = raw_value_bounds(series);
    // Nice number 반올림 (눈금을 깔끔하게, 경계 headroom 포함)
    nice_axis(min, max, target_ticks)
}

fn value_range(chart: &OoxmlChart, target_ticks: f64) -> (f64, f64, f64) {
    value_range_for(chart.series.iter(), target_ticks)
}

/// raw 간격에 가장 가까운 "깔끔한" 눈금 간격 (1/2/5/10 × 10^n, 반올림 임계 1.5/3/7)
fn floor_nice_step(raw: f64) -> f64 {
    let mag = 10f64.powf(raw.abs().log10().floor());
    let norm = raw / mag;
    let step = if norm < 1.5 {
        1.0
    } else if norm < 3.0 {
        2.0
    } else if norm < 7.0 {
        5.0
    } else {
        10.0
    };
    step * mag
}

/// 세로 값축 눈금 목표 칸수 (한컴 2022 실측: 세로막대/선의 값축은 ~3칸 — 5.0→0~6
/// step 2, 누적 12.3→0~15 step 5)
const VERTICAL_AXIS_TICKS: f64 = 3.0;
/// 가로 값축·scatter 양축 눈금 목표 칸수 (실측: 가로 누적 12.3→0~14 step 2,
/// 가로 묶은 5.0→0~6 step 1, scatter X 2.6→0~3 step 0.5)
const HORIZONTAL_AXIS_TICKS: f64 = 5.0;

/// min~max 구간을 "깔끔한" 눈금으로 확장하고 `(min', max', step)`을 반환.
///
/// 한컴 정합(C1c #1882 갭④, 시각판정 실측 보강): 데이터 max가 step 경계에 정확히
/// 걸리면 **+1 step headroom**(step은 유지 — 가로 묶은막대 5.0→0~6 step 1 실측).
/// 눈금 밀도는 축 방향별 target_ticks로 제어(세로 3칸/가로·scatter 5칸) — 같은
/// 데이터(합 12.3)가 세로 누적 0~15 step 5, 가로 누적 0~14 step 2로 실측됨.
/// 3차원 계열의 고유 축(묶은 0~5 무헤드룸/누적 0~20 과헤드룸)은 2D 근사 범위 밖(C2).
fn nice_axis(min: f64, max: f64, target_ticks: f64) -> (f64, f64, f64) {
    let (new_min, mut new_max, step) = nice_axis_no_headroom(min, max, target_ticks);
    if (new_max - max).abs() < step * 1e-6 {
        new_max += step; // 경계 headroom +1 step (step 유지)
    }
    (new_min, new_max, step)
}

/// `nice_axis`의 경계 headroom 없는 변형 — 한컴 3D 묶은막대 실측(세로·가로 모두
/// 0~5: 데이터 max 5.0이 step 1 경계에 걸려도 확장하지 않음)용.
fn nice_axis_no_headroom(min: f64, max: f64, target_ticks: f64) -> (f64, f64, f64) {
    if max <= min {
        return (min, max, 1.0);
    }
    let step = floor_nice_step((max - min) / target_ticks);
    let new_min = (min / step).floor() * step;
    let new_max = (max / step).ceil() * step;
    (new_min, new_max, step)
}

/// 분산형 수치축 범위 `(min, max, step)`. 양수 데이터는 **0 기준선으로 clamp**한다 —
/// 한컴 분산형 PDF 정합(정답지 X·Y 모두 0부터: 표식만있는분산형 X 0~3·Y 0~5).
/// 막대/선 축(`value_range_for`)과 동일한 0-baseline 동작이라 차트 종류 간 일관성도
/// 확보. nice_axis로 눈금 정리(경계 headroom 포함, C1c #1882 갭④). — C1b #1660.
fn scatter_range(vals: impl Iterator<Item = f64>) -> (f64, f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in vals {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }
    if !min.is_finite() {
        min = 0.0;
    }
    if !max.is_finite() {
        max = 1.0;
    }
    if min > 0.0 {
        min = 0.0; // 양수 데이터는 0 기준선 (한컴 분산형 정합)
    }
    if (max - min).abs() < 1e-9 {
        max = min + 1.0;
    }
    nice_axis(min, max, HORIZONTAL_AXIS_TICKS)
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
    let stacked = matches!(
        chart.grouping,
        BarGrouping::Stacked | BarGrouping::PercentStacked
    );
    let percent = chart.grouping == BarGrouping::PercentStacked;

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

    // 값축 범위: clustered=개별값, stacked=카테고리 합의 최대, percent=0~100%
    // (percent는 step 20 고정 = 종전 5등분 라벨 0/20/…/100%와 동일)
    // 눈금 밀도는 값축 방향 기준: 세로막대=세로 값축(3칸), 가로막대=가로 값축(5칸)
    let ticks = if horizontal {
        HORIZONTAL_AXIS_TICKS
    } else {
        VERTICAL_AXIS_TICKS
    };
    let (vmin, vmax, vstep) = if percent {
        (0.0, 100.0, 20.0)
    } else if stacked {
        let max_sum = (0..cat_count)
            .map(|ci| category_positive_sum(chart, ci))
            .fold(0.0_f64, f64::max);
        let (mn, mx, st) = nice_axis(0.0, max_sum.max(1.0), ticks);
        if chart.is_3d && !horizontal {
            // 한컴 3D 누적'세로' 실측: 2D(0~15) + 1 step = 0~20. 가로는 2D와 동일(0~14).
            (mn, mx + st, st)
        } else {
            (mn, mx, st)
        }
    } else if chart.is_3d {
        // 한컴 3D 묶은막대 실측: 세로·가로 모두 촘촘 눈금(5칸) + 경계 headroom 없음
        // (max 5.0 → 0~5 step 1; 2D의 0~6과 다름)
        let (mn, mx) = raw_value_bounds(chart.series.iter());
        nice_axis_no_headroom(mn, mx, HORIZONTAL_AXIS_TICKS)
    } else {
        value_range(chart, ticks)
    };

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
        vstep,
        chart.series.first().and_then(|s| s.format_code.as_deref()),
        horizontal,
        false,
        percent,
        false,
    );

    let (cat_span, bar_span_total) = if horizontal {
        let span = ph / cat_count as f64;
        (span, span * 0.7)
    } else {
        let span = pw / cat_count as f64;
        (span, span * 0.7)
    };

    // 가로 막대는 카테고리를 아래→위로 배치 (한컴 실측: 항목 1이 맨 아래).
    // 세로는 왼→오른쪽 그대로.
    let cat_slot = |ci: usize| -> f64 {
        let idx = if horizontal { cat_count - 1 - ci } else { ci };
        cat_span * idx as f64
    };

    if stacked {
        // 누적: 카테고리당 단일 막대, 시리즈를 아래/왼쪽부터 쌓음.
        // percent → 카테고리 합으로 정규화(전체 길이 = 100%), stacked → vmax로 정규화.
        for ci in 0..cat_count {
            let denom = if percent {
                let s = category_positive_sum(chart, ci);
                if s > 0.0 {
                    s
                } else {
                    1.0
                }
            } else {
                (vmax - vmin).max(1e-9)
            };
            let mut acc = 0.0_f64; // 지금까지 쌓인 픽셀 길이
            for (si, ser) in chart.series.iter().enumerate() {
                let v = ser.values.get(ci).copied().unwrap_or(0.0).max(0.0);
                let color = series_color(ser, si);
                let base = px;
                // 셀 시작: 가로=세로축(py) 기준, 세로=가로축(px) 기준
                let cell = if horizontal { py } else { px }
                    + cat_slot(ci)
                    + (cat_span - bar_span_total) / 2.0;
                if horizontal {
                    let seg = pw * (v / denom);
                    svg.push_str(&format!(
                        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                        base + acc, cell, seg.max(0.0), bar_span_total, color
                    ));
                    acc += seg;
                } else {
                    let seg = ph * (v / denom);
                    let by = py + ph - acc - seg;
                    svg.push_str(&format!(
                        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                        cell, by, bar_span_total, seg.max(0.0), color
                    ));
                    acc += seg;
                }
            }
        }
    } else {
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
                    let cy =
                        py + cat_slot(ci) + (cat_span - bar_span_total) / 2.0 + bar_w * si as f64;
                    let bw = pw * t;
                    svg.push_str(&format!(
                        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                        px, cy, bw.max(0.0), bar_w * 0.95, color
                    ));
                } else {
                    let cx =
                        px + cat_slot(ci) + (cat_span - bar_span_total) / 2.0 + bar_w * si as f64;
                    let bh = ph * t;
                    let by = py + ph - bh;
                    svg.push_str(&format!(
                        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                        cx, by, bar_w * 0.95, bh.max(0.0), color
                    ));
                }
            }
        }
    }

    render_category_labels(svg, chart, px, py, pw, ph, cat_count, horizontal);
}

/// 한 카테고리의 (양수) 시리즈 값 합. 누적 막대 축/정규화에 사용.
fn category_positive_sum(chart: &OoxmlChart, ci: usize) -> f64 {
    chart
        .series
        .iter()
        .map(|s| s.values.get(ci).copied().unwrap_or(0.0).max(0.0))
        .sum()
}

// ---------------- Line (단일 축) ----------------

fn render_line(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let stacked = matches!(
        chart.line_grouping,
        BarGrouping::Stacked | BarGrouping::PercentStacked
    );
    let percent = chart.line_grouping == BarGrouping::PercentStacked;

    let max_len = chart
        .series
        .iter()
        .map(|s| s.values.len())
        .max()
        .unwrap_or(0);
    if max_len < 2 {
        return;
    }

    // 값축: 비누적=개별값, 누적=카테고리 합의 최대, 백프로=0~100% step 20
    // (render_bars 누적 정책 미러 — 정답지 실측 누적 0~15 step 5. C1d #2129)
    let (vmin, vmax, vstep) = if percent {
        (0.0, 100.0, 20.0)
    } else if stacked {
        let max_sum = (0..max_len)
            .map(|ci| category_positive_sum(chart, ci))
            .fold(0.0_f64, f64::max);
        nice_axis(0.0, max_sum.max(1.0), VERTICAL_AXIS_TICKS)
    } else {
        value_range(chart, VERTICAL_AXIS_TICKS)
    };

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
        vstep,
        chart.series.first().and_then(|s| s.format_code.as_deref()),
        false,
        false,
        percent,
        false,
    );

    // x 배치: 카테고리 슬롯 중앙 (한컴 정합, XML crossBetween=between —
    // 첫/끝 점이 플롯 가장자리가 아닌 반 슬롯 안쪽. 카테고리 라벨과 동일 공식.
    // 작업지시자 시각판정 반영, C1d #2129)
    let cat_span = pw / max_len as f64;
    let mut cum = vec![0.0_f64; max_len]; // 카테고리별 누적값 (값공간)
    for (si, ser) in chart.series.iter().enumerate() {
        let color = series_color(ser, si);
        let mut points: Vec<(f64, f64)> = Vec::with_capacity(ser.values.len());
        for (i, &v) in ser.values.iter().enumerate() {
            let val = if stacked {
                cum[i] += v.max(0.0); // 음수 clamp — render_bars 누적과 동일 정책
                if percent {
                    let sum = category_positive_sum(chart, i);
                    if sum > 0.0 {
                        cum[i] / sum * 100.0
                    } else {
                        0.0 // 합 0 카테고리 → 0% (막대 denom=1.0 가드와 동등)
                    }
                } else {
                    cum[i]
                }
            } else {
                v
            };
            let t = if vmax > vmin {
                (val - vmin) / (vmax - vmin)
            } else {
                0.0
            };
            points.push((px + cat_span * (i as f64 + 0.5), py + ph - ph * t));
        }
        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>\n",
            polyline_path(&points),
            color
        ));
        if chart.line_markers {
            for &(mx, my) in &points {
                push_line_marker(svg, si, mx, my, &color);
            }
        }
    }

    render_category_labels(svg, chart, px, py, pw, ph, max_len, false);
}

// ---------------- Scatter (분산형, 2 수치축) ----------------

fn render_scatter(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    // 전 시리즈가 (x,y) 쌍을 못 만들면 격자도 의미 없음 → 조기 종료.
    // (상위 <g class="hwp-ooxml-chart">는 이미 출력되어 placeholder는 안 뜸)
    if chart
        .series
        .iter()
        .all(|s| s.x_values.is_empty() || s.values.is_empty())
    {
        return;
    }

    let (xmin, xmax, xstep) =
        scatter_range(chart.series.iter().flat_map(|s| s.x_values.iter().copied()));
    let (ymin, ymax, ystep) =
        scatter_range(chart.series.iter().flat_map(|s| s.values.iter().copied()));
    let xspan = (xmax - xmin).max(1e-9);
    let yspan = (ymax - ymin).max(1e-9);

    // 플롯 배경
    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));
    // X축(하단, 수직 격자선) + Y축(좌측, 수평 격자선) — 둘 다 수치축, 소수 라벨
    render_value_grid(
        svg, px, py, pw, ph, xmin, xmax, xstep, None, true, false, false, true,
    );
    render_value_grid(
        svg, px, py, pw, ph, ymin, ymax, ystep, None, false, false, false, true,
    );

    let (show_line, smooth, show_markers) = chart.scatter_style.flags();

    for (si, ser) in chart.series.iter().enumerate() {
        let color = series_color(ser, si);
        // (x,y) 픽셀 좌표. 데이터 순서 유지(x 정렬 안 함), 길이 불일치 시 짧은 쪽으로 절단.
        let points: Vec<(f64, f64)> = ser
            .x_values
            .iter()
            .zip(ser.values.iter())
            .map(|(&x, &y)| {
                (
                    px + pw * (x - xmin) / xspan,
                    py + ph - ph * (y - ymin) / yspan,
                )
            })
            .collect();
        if points.is_empty() {
            continue;
        }

        if show_line && points.len() >= 2 {
            let d = if smooth {
                smooth_path(&points)
            } else {
                polyline_path(&points)
            };
            svg.push_str(&format!(
                "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>\n",
                d, color
            ));
        }
        if show_markers {
            for (xp, yp) in &points {
                svg.push_str(&format!(
                    "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"3\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
                    xp, yp, color
                ));
            }
        }
    }
}

/// 라인 차트 표식(마커). 계열 인덱스별 한컴 기본 사이클 ◆■▲(+원 폴백) —
/// 정답지 PDF 실측(표식이있는누적꺽은선형: 계열1 ◆/계열2 ■/계열3 ▲).
/// 크기 상수는 근사값으로 시각판정에서 조정 여지. (C1d #2129)
fn push_line_marker(svg: &mut String, si: usize, cx: f64, cy: f64, color: &str) {
    let d = match si % 4 {
        0 => {
            // ◆ 다이아몬드
            let r = 3.5;
            format!(
                "M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx,
                cy - r,
                cx + r,
                cy,
                cx,
                cy + r,
                cx - r,
                cy
            )
        }
        1 => {
            // ■ 정사각형
            let h = 3.0;
            format!(
                "M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx - h,
                cy - h,
                cx + h,
                cy - h,
                cx + h,
                cy + h,
                cx - h,
                cy + h
            )
        }
        2 => {
            // ▲ 삼각형
            let r = 3.5;
            format!(
                "M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx,
                cy - r,
                cx + r,
                cy + r * 0.8,
                cx - r,
                cy + r * 0.8
            )
        }
        _ => {
            // 원 폴백 (계열 4+ — 코퍼스 밖, scatter 마커와 동일 반경 3)
            format!("M{:.2},{:.2} a3,3 0 1,0 6,0 a3,3 0 1,0 -6,0", cx - 3.0, cy)
        }
    };
    svg.push_str(&format!(
        "<path class=\"hwp-chart-marker\" d=\"{}\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
        d, color
    ));
}

/// 직선 폴리라인 path (`M…L…`).
fn polyline_path(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        d.push_str(&format!(
            "{}{:.2},{:.2} ",
            if i == 0 { "M" } else { "L" },
            x,
            y
        ));
    }
    d.trim().to_string()
}

/// Catmull-Rom → cubic Bézier 곡선 path. 데이터 순서, 끝점 clamp(P₋₁=P₀, Pₙ=Pₙ₋₁). — C1b #1660.
fn smooth_path(points: &[(f64, f64)]) -> String {
    let n = points.len();
    if n < 2 {
        return polyline_path(points);
    }
    let mut d = format!("M{:.2},{:.2}", points[0].0, points[0].1);
    for i in 0..n - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 >= n {
            points[n - 1]
        } else {
            points[i + 2]
        };
        let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        d.push_str(&format!(
            " C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
            c1.0, c1.1, c2.0, c2.1, p2.0, p2.1
        ));
    }
    d
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

    let (pri_min, pri_max, pri_step) = if pri.is_empty() {
        value_range(chart, VERTICAL_AXIS_TICKS)
    } else {
        value_range_for(pri.iter().cloned(), VERTICAL_AXIS_TICKS)
    };
    let (sec_min, sec_max, sec_step) = if sec.is_empty() {
        (0.0, 1.0, 0.2)
    } else {
        value_range_for(sec.iter().cloned(), VERTICAL_AXIS_TICKS)
    };

    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\" stroke=\"#cccccc\" stroke-width=\"0.5\"/>\n",
        px, py, pw, ph
    ));

    // 기본축 격자 (좌측)
    let pri_fmt = pri.first().and_then(|s| s.format_code.as_deref());
    render_value_grid(
        svg, px, py, pw, ph, pri_min, pri_max, pri_step, pri_fmt, false, false, false, false,
    );

    // 보조축 격자 (우측, 눈금만) — step 기반이라 기본축과 눈금 수가 다를 수 있음
    // (보조축은 라벨만 출력하므로 격자선 불일치 없음)
    if !sec.is_empty() {
        let sec_fmt = sec.first().and_then(|s| s.format_code.as_deref());
        render_value_grid(
            svg, px, py, pw, ph, sec_min, sec_max, sec_step, sec_fmt, false, true, false, false,
        );
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

#[allow(clippy::too_many_arguments)]
fn render_value_grid(
    svg: &mut String,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    vmin: f64,
    vmax: f64,
    step: f64,
    format_code: Option<&str>,
    horizontal: bool,
    secondary: bool,
    percent: bool,
    decimal: bool,
) {
    // 비정수 step은 소수 라벨 강제 — format_num의 정수 반올림이 0.5 간격 라벨을
    // "0,1,1,2…"로 손상시키는 것 차단 (C1c #1882 갭④)
    let decimal = decimal || (step - step.round()).abs() > 1e-9;
    let label = |v: f64| -> String {
        if percent {
            format!("{}%", v.round() as i64)
        } else if decimal {
            format_axis_num(v)
        } else {
            format_num(v, format_code)
        }
    };
    // step 기반 눈금: v = vmin + step*i (정수 루프 — 부동소수 누적 드리프트 방지)
    let span = (vmax - vmin).max(1e-9);
    let step = if step > 0.0 { step } else { span / 5.0 };
    let grid_lines = (span / step).round().max(1.0) as usize;
    for i in 0..=grid_lines {
        let t = (step * i as f64) / span;
        if horizontal {
            let gx = px + pw * t;
            // 보조축일 때는 격자선 중복 방지, 라벨만
            if !secondary {
                svg.push_str(&format!(
                    "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#e8e8e8\" stroke-width=\"0.5\"/>\n",
                    gx, py, gx, py + ph
                ));
            }
            let v = vmin + step * i as f64;
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"middle\">{}</text>\n",
                gx, py + ph + 12.0, xml_escape(&label(v))
            ));
        } else {
            let gy = py + ph - ph * t;
            if !secondary {
                svg.push_str(&format!(
                    "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#e8e8e8\" stroke-width=\"0.5\"/>\n",
                    px, gy, px + pw, gy
                ));
            }
            let v = vmin + step * i as f64;
            let (tx, anchor) = if secondary {
                (px + pw + 4.0, "start")
            } else {
                (px - 4.0, "end")
            };
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"{}\">{}</text>\n",
                tx, gy + 3.0, anchor, xml_escape(&label(v))
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
            // 가로 막대: 카테고리 아래→위 (한컴 실측 — 막대 배치와 동일 순서)
            let row = cat_count - 1 - ci;
            let cy = py + cat_span * row as f64 + cat_span / 2.0 + 3.0;
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

/// 범례 항목 목록 `(라벨, 색상, 시리즈 타입)`. pie는 카테고리별, 그 외는 시리즈별.
fn legend_items(chart: &OoxmlChart) -> Vec<(String, u32, OoxmlChartType)> {
    match chart.chart_type {
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
    }
}

/// 범례 스와치 1개: 라인 시리즈는 선, 그 외 10×10 사각형. `cy` = 행 세로 중심.
fn push_legend_swatch(svg: &mut String, ix: f64, cy: f64, color: u32, stype: OoxmlChartType) {
    if stype == OoxmlChartType::Line {
        svg.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"2\"/>\n",
            ix, cy, ix + 14.0, cy, color_hex(color)
        ));
    } else {
        svg.push_str(&format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" fill=\"{}\"/>\n",
            ix,
            cy - 6.0,
            color_hex(color)
        ));
    }
}

/// 하단 가로 범례 (legendPos=b 및 기본값)
fn render_legend(svg: &mut String, chart: &OoxmlChart, x: f64, y: f64, w: f64, _h: f64) {
    if chart.series.is_empty() {
        return;
    }
    let items = legend_items(chart);

    svg.push_str("<g class=\"hwp-chart-legend\">\n");
    // 가운데 정렬: 항목 개수로 총 너비 계산
    let item_w = 100.0_f64.min((w / items.len().max(1) as f64).max(60.0));
    let total_w = item_w * items.len() as f64;
    let start_x = x + (w - total_w) / 2.0;
    for (i, (label, color, stype)) in items.iter().enumerate() {
        let ix = start_x + item_w * i as f64;
        push_legend_swatch(svg, ix, y + 11.0, *color, *stype);
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\">{}</text>\n",
            ix + 18.0, y + 14.0, xml_escape(label)
        ));
    }
    svg.push_str("</g>\n");
}

/// 우측 세로 범례 (legendPos=r — 한컴 코퍼스 전 샘플). 플롯 세로 중앙 정렬.
/// C1c #1882 갭③.
fn render_legend_right(svg: &mut String, chart: &OoxmlChart, x: f64, y: f64, h: f64) {
    if chart.series.is_empty() {
        return;
    }
    let items = legend_items(chart);
    let row_h = 16.0;
    let total_h = row_h * items.len() as f64;
    let start_y = y + ((h - total_h) / 2.0).max(0.0);

    svg.push_str("<g class=\"hwp-chart-legend\">\n");
    for (i, (label, color, stype)) in items.iter().enumerate() {
        let cy = start_y + row_h * i as f64 + row_h / 2.0;
        push_legend_swatch(svg, x, cy, *color, *stype);
        svg.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#333\">{}</text>\n",
            x + 18.0,
            cy + 3.0,
            xml_escape(label)
        ));
    }
    svg.push_str("</g>\n");
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

    // --- C1c (#1882) 갭②: 한컴 2022 기본 팔레트 ---

    #[test]
    fn test_default_palette_hancom_order() {
        // 색 미지정 3시리즈 → 팔레트 순환: 파랑 → 주황 → 회색 (한컴 2022 실측)
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Column,
            series: (0..3)
                .map(|i| OoxmlSeries {
                    values: vec![1.0 + i as f64, 2.0],
                    series_type: OoxmlChartType::Column,
                    ..Default::default()
                })
                .collect(),
            categories: vec!["a".into(), "b".into()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let i_blue = svg.find("#6183d7").expect("시리즈1 파랑");
        let i_orange = svg.find("#fe813b").expect("시리즈2 주황");
        let i_gray = svg.find("#b0b0b0").expect("시리즈3 회색");
        assert!(
            i_blue < i_orange && i_orange < i_gray,
            "팔레트 순서: 파랑→주황→회색"
        );
        assert!(!svg.contains("#70ad47"), "구 녹색-우선 팔레트 미사용");
    }

    // --- C1a Part B (#1453): 막대 누적 기하 ---

    /// 데이터 막대(fill="#...", stroke 없음)의 x 좌표 목록. 배경/플롯 rect 제외.
    /// (시리즈 name 비움 → 범례 미렌더 → 데이터 막대만 남음)
    fn data_bar_xs(svg: &str) -> Vec<i64> {
        let mut xs = Vec::new();
        for chunk in svg.split("<rect ").skip(1) {
            let end = chunk.find('>').unwrap_or(chunk.len());
            let tag = &chunk[..end];
            // 배경/플롯 rect(stroke) + 범례 swatch(10×10) 제외 → 데이터 막대만.
            if tag.contains("stroke")
                || !tag.contains("fill=\"#")
                || tag.contains("width=\"10\" height=\"10\"")
            {
                continue;
            }
            if let Some(p) = tag.find("x=\"") {
                let s = p + 3;
                if let Some(e) = tag[s..].find('"') {
                    if let Ok(v) = tag[s..s + e].parse::<f64>() {
                        xs.push((v * 10.0).round() as i64); // 0.1 단위 라운드
                    }
                }
            }
        }
        xs
    }

    fn distinct(mut v: Vec<i64>) -> usize {
        v.sort_unstable();
        v.dedup();
        v.len()
    }

    fn bars_chart(grouping: BarGrouping) -> OoxmlChart {
        OoxmlChart {
            chart_type: OoxmlChartType::Column,
            grouping,
            // name 비움 → 범례 미렌더
            series: vec![
                OoxmlSeries {
                    values: vec![4.0, 3.0],
                    ..Default::default()
                },
                OoxmlSeries {
                    values: vec![2.0, 1.0],
                    ..Default::default()
                },
                OoxmlSeries {
                    values: vec![2.0, 4.0],
                    ..Default::default()
                },
            ],
            categories: vec!["a".into(), "b".into()],
            ..Default::default()
        }
    }

    #[test]
    fn test_stacked_bars_share_x_per_category() {
        // 누적: 카테고리(2)당 단일 컬럼 → 서로 다른 x = 2개 (시리즈가 같은 x 공유)
        let svg = render_chart_svg(&bars_chart(BarGrouping::Stacked), 0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            distinct(data_bar_xs(&svg)),
            2,
            "stacked는 카테고리당 단일 x"
        );
    }

    #[test]
    fn test_clustered_bars_distinct_x() {
        // 묶은: 카테고리(2) × 시리즈(3) = 6개 서로 다른 x (무회귀 가드)
        let svg = render_chart_svg(&bars_chart(BarGrouping::Clustered), 0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            distinct(data_bar_xs(&svg)),
            6,
            "clustered는 시리즈별 x 분리"
        );
    }

    #[test]
    fn test_percent_stacked_axis_and_single_column() {
        // 백프로: % 축 라벨 + 카테고리당 단일 컬럼
        let svg = render_chart_svg(
            &bars_chart(BarGrouping::PercentStacked),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert!(svg.contains("100%"), "percentStacked는 % 축 라벨");
        assert!(svg.contains("0%"));
        assert_eq!(
            distinct(data_bar_xs(&svg)),
            2,
            "percent도 카테고리당 단일 x"
        );
    }

    // --- C1d (#2129): 라인 누적/백프로 기하 ---

    /// 데이터 라인 path(fill="none" stroke-width="2")의 d 문자열 목록 (시리즈 순서).
    /// 마커 path(fill=색)·격자선(line)·배경(rect)은 제외됨.
    fn data_line_paths(svg: &str) -> Vec<String> {
        let mut out = Vec::new();
        for chunk in svg.split("<path ").skip(1) {
            let end = chunk.find("/>").unwrap_or(chunk.len());
            let tag = &chunk[..end];
            if !tag.contains("fill=\"none\"") || !tag.contains("stroke-width=\"2\"") {
                continue;
            }
            if let Some(p) = tag.find("d=\"") {
                let s = p + 3;
                if let Some(e) = tag[s..].find('"') {
                    out.push(tag[s..s + e].to_string());
                }
            }
        }
        out
    }

    /// path d의 (x,y) 점 목록 (`M`/`L` 접두 제거).
    fn path_points(d: &str) -> Vec<(f64, f64)> {
        d.split_whitespace()
            .filter_map(|tok| {
                let t = tok.trim_start_matches(['M', 'L']);
                let (x, y) = t.split_once(',')?;
                Some((x.parse().ok()?, y.parse().ok()?))
            })
            .collect()
    }

    /// 3계열×4카테고리, 카테고리 합 최대 12.3 (합: 8.7/8.9/8.3/12.3 — 코퍼스 라인
    /// 샘플과 동일 스케일). 개별값 최대 5.0 → 비누적 축 0~6, 누적 축 0~15로 구분됨.
    fn line_chart(line_grouping: BarGrouping) -> OoxmlChart {
        OoxmlChart {
            chart_type: OoxmlChartType::Line,
            line_grouping,
            // name 비움 → 범례 미렌더
            series: vec![
                OoxmlSeries {
                    values: vec![4.3, 2.5, 3.5, 4.5],
                    ..Default::default()
                },
                OoxmlSeries {
                    values: vec![2.4, 4.4, 1.8, 2.8],
                    ..Default::default()
                },
                OoxmlSeries {
                    values: vec![2.0, 2.0, 3.0, 5.0],
                    ..Default::default()
                },
            ],
            categories: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            ..Default::default()
        }
    }

    #[test]
    fn test_line_stacked_axis_from_category_sum() {
        // 누적 축 = 카테고리 합 최대(12.3) 기반 0~15 step 5 — 정답지 실측.
        // 개별값 최대(5.0) 기반 0~6이 아님.
        let svg = render_chart_svg(&line_chart(BarGrouping::Stacked), 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains(">15<"), "누적 축 max 15");
        assert!(!svg.contains(">6<"), "개별값 축(0~6) 미사용");
        assert!(!svg.contains(">14<"), "step 5 유지 (경계 headroom 미발동)");
    }

    #[test]
    fn test_line_stacked_series_order() {
        // 누적: 시리즈2 첫 점(누적 6.7)이 시리즈1 첫 점(4.3) 위 (화면 y 작음)
        let svg = render_chart_svg(&line_chart(BarGrouping::Stacked), 0.0, 0.0, 400.0, 300.0);
        let paths = data_line_paths(&svg);
        assert_eq!(paths.len(), 3, "데이터 라인 3개");
        let y0 = path_points(&paths[0])[0].1;
        let y1 = path_points(&paths[1])[0].1;
        assert!(y1 < y0, "누적이면 시리즈2(y={y1})가 시리즈1(y={y0})보다 위");
    }

    #[test]
    fn test_line_percent_axis_labels() {
        // 백프로: 축 0%~100% step 20% — 정답지 실측 (막대 percent와 동일 정책)
        let svg = render_chart_svg(
            &line_chart(BarGrouping::PercentStacked),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert!(svg.contains("100%"), "percent 축 100% 라벨");
        assert!(svg.contains("20%"), "step 20%");
    }

    #[test]
    fn test_line_percent_top_series_flat() {
        // 최상위 시리즈 누적 = 카테고리 합 = 100% → 수평선 (정답지: 계열3이 100% 평행선)
        let svg = render_chart_svg(
            &line_chart(BarGrouping::PercentStacked),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        let paths = data_line_paths(&svg);
        let pts = path_points(&paths[2]);
        assert_eq!(pts.len(), 4);
        assert!(
            pts.windows(2).all(|w| (w[0].1 - w[1].1).abs() < 1e-6),
            "최상위 시리즈 y 전부 동일해야: {pts:?}"
        );
    }

    #[test]
    fn test_line_percent_zero_sum_category_no_nan() {
        // 합 0 카테고리 → cum/0 NaN 방지 가드 (0%로 렌더)
        let mut chart = line_chart(BarGrouping::PercentStacked);
        chart.series = vec![
            OoxmlSeries {
                values: vec![1.0, 0.0],
                ..Default::default()
            },
            OoxmlSeries {
                values: vec![1.0, 0.0],
                ..Default::default()
            },
        ];
        chart.categories = vec!["a".into(), "b".into()];
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(!svg.contains("NaN"), "합 0 카테고리 NaN 가드");
    }

    /// `>{label}<` 텍스트 요소의 x 좌표.
    fn text_label_x(svg: &str, label: &str) -> f64 {
        let i = svg
            .find(&format!(">{label}<"))
            .unwrap_or_else(|| panic!("라벨 {label} 없음"));
        let start = svg[..i].rfind("<text ").expect("text 태그");
        let tag = &svg[start..i];
        let p = tag.find("x=\"").expect("x 속성") + 3;
        let e = p + tag[p..].find('"').expect("닫는 따옴표");
        tag[p..e].parse().expect("x 파싱")
    }

    #[test]
    fn test_line_points_at_category_slot_centers() {
        // 한컴 정합(작업지시자 시각판정 2026-07-10): 라인 점은 카테고리 슬롯 중앙 —
        // 첫/끝 점이 플롯 가장자리에 붙지 않고 반 슬롯 안쪽 (XML crossBetween=between).
        // 카테고리 라벨(슬롯 중앙, text-anchor=middle)과 x가 일치해야 한다.
        let svg = render_chart_svg(&line_chart(BarGrouping::Clustered), 0.0, 0.0, 400.0, 300.0);
        let pts = path_points(&data_line_paths(&svg)[0]);
        assert!(
            (pts[0].0 - text_label_x(&svg, "a")).abs() < 0.5,
            "첫 점 x={} ≠ 첫 카테고리 라벨 x={} (슬롯 중앙 아님)",
            pts[0].0,
            text_label_x(&svg, "a")
        );
        assert!(
            (pts[3].0 - text_label_x(&svg, "d")).abs() < 0.5,
            "끝 점 x={} ≠ 끝 카테고리 라벨 x={} (슬롯 중앙 아님)",
            pts[3].0,
            text_label_x(&svg, "d")
        );
    }

    /// `hwp-chart-marker` path의 d 문자열 목록 (시리즈×점 순서).
    fn marker_ds(svg: &str) -> Vec<String> {
        let mut out = Vec::new();
        for chunk in svg.split("<path ").skip(1) {
            let end = chunk.find("/>").unwrap_or(chunk.len());
            let tag = &chunk[..end];
            if !tag.contains("hwp-chart-marker") {
                continue;
            }
            if let Some(p) = tag.find("d=\"") {
                let s = p + 3;
                if let Some(e) = tag[s..].find('"') {
                    out.push(tag[s..s + e].to_string());
                }
            }
        }
        out
    }

    #[test]
    fn test_line_markers_rendered() {
        // line_markers=true → 마커 수 = 계열(3) × 점(4) = 12 (누적에서도 동일)
        let mut chart = line_chart(BarGrouping::Stacked);
        chart.line_markers = true;
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(marker_ds(&svg).len(), 12, "3계열×4점 마커");
    }

    #[test]
    fn test_line_marker_shape_cycle() {
        // 계열별 기본 표식 사이클 ◆■▲ (정답지 실측 — 표식이있는누적꺽은선형)
        let mut chart = line_chart(BarGrouping::Clustered);
        chart.line_markers = true;
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let ds = marker_ds(&svg);
        assert_eq!(ds.len(), 12);
        let skel = |d: &str| {
            d.chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
        };
        // 시리즈별 첫 마커: [0]=◆, [4]=■, [8]=▲
        assert_eq!(skel(&ds[0]), "MLLLZ", "◆ 4각형");
        assert_eq!(skel(&ds[4]), "MLLLZ", "■ 4각형");
        assert_eq!(skel(&ds[8]), "MLLZ", "▲ 3각형");
        // ◆ vs ■ 구분: 첫 세그먼트가 ◆는 대각(y 변화), ■는 수평(y 동일)
        let dia = path_points(&ds[0]);
        assert!((dia[0].1 - dia[1].1).abs() > 1e-6, "◆ 첫 세그먼트 대각");
        let sq = path_points(&ds[4]);
        assert!((sq[0].1 - sq[1].1).abs() < 1e-6, "■ 첫 세그먼트 수평");
    }

    #[test]
    fn test_line_marker_circle_fallback_series4() {
        // 계열 4+ 는 원 폴백 (코퍼스 밖 — 사이클 재시작 대신 원)
        let mut chart = line_chart(BarGrouping::Clustered);
        chart.line_markers = true;
        chart.series.push(OoxmlSeries {
            values: vec![1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        });
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let ds = marker_ds(&svg);
        assert_eq!(ds.len(), 16);
        assert!(ds[12].contains('a'), "계열4는 원(arc) 폴백: {}", ds[12]);
    }

    #[test]
    fn test_line_no_markers_by_default() {
        // 기본값(line_markers=false) → 마커 없음 (꺽은선형/누적꺽은선형 무회귀)
        let svg = render_chart_svg(&line_chart(BarGrouping::Stacked), 0.0, 0.0, 400.0, 300.0);
        assert!(!svg.contains("hwp-chart-marker"), "기본은 무마커");
    }

    #[test]
    fn test_line_clustered_unchanged() {
        // 비누적(기본, 꺽은선형 무회귀 핀): 개별값 축 0~6 + 시리즈1(4.3)이 시리즈2(2.4) 위
        let svg = render_chart_svg(&line_chart(BarGrouping::Clustered), 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains(">6<"), "개별값 축 max 6");
        assert!(!svg.contains(">15<"), "누적 축 미사용");
        let paths = data_line_paths(&svg);
        assert_eq!(paths.len(), 3);
        let y0 = path_points(&paths[0])[0].1;
        let y1 = path_points(&paths[1])[0].1;
        assert!(y0 < y1, "비누적: 개별값 기준 시리즈1이 위");
    }

    // --- C1b (#1660): 분산형(scatter) 렌더 ---

    fn scatter_chart(style: ScatterStyle) -> OoxmlChart {
        OoxmlChart {
            chart_type: OoxmlChartType::Scatter,
            scatter_style: style,
            series: vec![OoxmlSeries {
                name: "Y1".into(),
                x_values: vec![0.7, 1.8, 2.6],
                values: vec![2.7, 3.2, 0.8],
                series_type: OoxmlChartType::Scatter,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_render_scatter_marker_only() {
        // marker: 점만, 연결선 없음.
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Marker), 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains("<circle"), "marker는 표식(circle) 있어야");
        assert!(!svg.contains("<path"), "marker는 연결선(path) 없어야");
        assert!(!svg.contains("차트 (미지원)"));
        assert!(svg.contains("hwp-ooxml-chart\""));
    }

    #[test]
    fn test_render_scatter_line_only() {
        // line: 직선만, 표식 없음.
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Line), 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains("<path"), "line은 연결선(path) 있어야");
        assert!(!svg.contains("<circle"), "line은 표식(circle) 없어야");
        assert!(!svg.contains(" C"), "line은 직선(C 베지어 없음)");
    }

    #[test]
    fn test_render_scatter_line_marker() {
        // lineMarker: 직선 + 표식.
        let svg = render_chart_svg(
            &scatter_chart(ScatterStyle::LineMarker),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
        assert!(!svg.contains(" C"), "lineMarker는 직선");
    }

    #[test]
    fn test_render_scatter_smooth() {
        // smoothMarker: 곡선(cubic Bézier C) + 표식.
        let svg = render_chart_svg(
            &scatter_chart(ScatterStyle::SmoothMarker),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains(" C"), "smooth는 cubic Bézier(C) 곡선");
    }

    #[test]
    fn test_render_scatter_decimal_axis_labels() {
        // 소수 데이터 → 소수 축 라벨 (format_num 정수 반올림이 아니라 format_axis_num).
        // 0-baseline clamp 후 X 0~3(step 0.5) → 눈금 0.5/1.5/2.5 등 (소수 라벨). — C1c 갭④
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Marker), 0.0, 0.0, 400.0, 300.0);
        assert!(
            svg.contains(">2.5<"),
            "분산형 축은 소수 라벨이어야 (정수 반올림 시 '2'로 손상)",
        );
        assert!(!svg.contains("차트 (미지원)"));
    }

    #[test]
    fn test_render_scatter_zero_baseline() {
        // 양수 데이터 → 축이 0부터 (한컴 분산형 PDF 정합). 0 라벨이 X·Y에 존재.
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Marker), 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains(">0<"), "분산형 축은 0 기준선이어야");
    }

    // --- C1c (#1882) 갭①: 자동 제목 ---

    #[test]
    fn test_render_auto_title_placeholder() {
        // c:title 요소 존재 + autoTitleDeleted=0 + 명시 텍스트 없음 →
        // 한컴처럼 자동 제목 "차트 제목" 렌더 (regular weight).
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Column,
            has_title_elem: true,
            series: vec![OoxmlSeries {
                values: vec![1.0, 2.0],
                series_type: OoxmlChartType::Column,
                ..Default::default()
            }],
            categories: vec!["a".into(), "b".into()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains("차트 제목"), "자동 제목 placeholder 렌더");
        assert!(
            !svg.contains("font-weight=\"600\""),
            "한컴 제목은 regular weight (600 아님)"
        );
    }

    #[test]
    fn test_render_no_auto_title_when_deleted_or_absent() {
        // autoTitleDeleted=1 또는 c:title 요소 자체가 없으면 자동 제목 없음.
        let base = OoxmlChart {
            chart_type: OoxmlChartType::Column,
            series: vec![OoxmlSeries {
                values: vec![1.0, 2.0],
                series_type: OoxmlChartType::Column,
                ..Default::default()
            }],
            categories: vec!["a".into(), "b".into()],
            ..Default::default()
        };
        let deleted = OoxmlChart {
            has_title_elem: true,
            auto_title_deleted: true,
            ..base.clone()
        };
        assert!(!render_chart_svg(&deleted, 0.0, 0.0, 400.0, 300.0).contains("차트 제목"));
        // has_title_elem=false (기본값) → 자동 제목 없음
        assert!(!render_chart_svg(&base, 0.0, 0.0, 400.0, 300.0).contains("차트 제목"));
    }

    // --- #1882 v2: 단일 시리즈 이름 자동 제목 fallback ---

    /// 제목 텍스트(font-size 13 — 범례/축 라벨(10px)과 구분)만 추출
    fn title_text(svg: &str) -> Option<String> {
        let chunk = svg.split("font-size=\"13\"").nth(1)?;
        let s = chunk.find('>')? + 1;
        let e = s + chunk[s..].find('<')?;
        Some(chunk[s..e].to_string())
    }

    fn single_series_chart(name: &str, chart_type: OoxmlChartType) -> OoxmlChart {
        OoxmlChart {
            chart_type,
            has_title_elem: true,
            series: vec![OoxmlSeries {
                name: name.into(),
                values: vec![4.3, 2.5],
                series_type: chart_type,
                ..Default::default()
            }],
            categories: vec!["a".into(), "b".into()],
            ..Default::default()
        }
    }

    #[test]
    fn test_render_auto_title_single_series_uses_name() {
        // 한컴 실측: 단일 시리즈면 자동 제목 = 시리즈 이름 (원형 5종 "판매",
        // 단일 시리즈 가로막대 "계열 1" — 차트 종류 불문 시리즈 수 기준 규칙).
        for chart_type in [
            OoxmlChartType::Pie,
            OoxmlChartType::Bar,
            OoxmlChartType::Column,
        ] {
            let svg = render_chart_svg(
                &single_series_chart("판매", chart_type),
                0.0,
                0.0,
                400.0,
                300.0,
            );
            assert_eq!(
                title_text(&svg).as_deref(),
                Some("판매"),
                "{chart_type:?}: 단일 시리즈 이름이 제목이어야"
            );
        }
    }

    #[test]
    fn test_render_auto_title_single_series_fallbacks() {
        // 단일 시리즈여도 이름이 비면 placeholder 유지.
        let unnamed = single_series_chart("", OoxmlChartType::Column);
        let svg = render_chart_svg(&unnamed, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(title_text(&svg).as_deref(), Some("차트 제목"));

        // 명시 제목이 있으면 시리즈 이름보다 우선.
        let mut explicit = single_series_chart("판매", OoxmlChartType::Column);
        explicit.title = Some("명시 제목".into());
        let svg = render_chart_svg(&explicit, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(title_text(&svg).as_deref(), Some("명시 제목"));

        // autoTitleDeleted=1이면 시리즈 이름 fallback도 억제 (제목 요소 없음).
        let mut suppressed = single_series_chart("판매", OoxmlChartType::Column);
        suppressed.auto_title_deleted = true;
        let svg = render_chart_svg(&suppressed, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(title_text(&svg), None);

        // 다계열이면 종전대로 placeholder (이름 있는 2계열).
        let mut multi = single_series_chart("판매", OoxmlChartType::Column);
        multi.series.push(OoxmlSeries {
            name: "재고".into(),
            values: vec![1.0, 2.0],
            series_type: OoxmlChartType::Column,
            ..Default::default()
        });
        let svg = render_chart_svg(&multi, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(title_text(&svg).as_deref(), Some("차트 제목"));
    }

    // --- C1c (#1882) 갭③: 범례 우측 배치 ---

    /// `hwp-chart-legend` 그룹 안 첫 `<text>`의 지정 속성 값
    fn legend_first_text_attr(svg: &str, attr: &str) -> f64 {
        let g = svg
            .split("class=\"hwp-chart-legend\"")
            .nth(1)
            .expect("범례 그룹");
        let text = g.split("<text ").nth(1).expect("범례 텍스트");
        let pat = format!("{attr}=\"");
        let s = text.find(&pat).expect("attr") + pat.len();
        let e = s + text[s..].find('"').expect("attr close");
        text[s..e].parse().expect("f64")
    }

    fn named_chart(legend_pos: LegendPos) -> OoxmlChart {
        OoxmlChart {
            chart_type: OoxmlChartType::Column,
            legend_pos,
            series: vec![
                OoxmlSeries {
                    name: "계열 1".into(),
                    values: vec![1.0, 2.0],
                    series_type: OoxmlChartType::Column,
                    ..Default::default()
                },
                OoxmlSeries {
                    name: "계열 2".into(),
                    values: vec![3.0, 4.0],
                    series_type: OoxmlChartType::Column,
                    ..Default::default()
                },
            ],
            categories: vec!["a".into(), "b".into()],
            ..Default::default()
        }
    }

    #[test]
    fn test_render_legend_right_vertical() {
        // legendPos=Right → 범례가 플롯 우측(x > 차트 폭 65%)에 세로 스택.
        let svg = render_chart_svg(&named_chart(LegendPos::Right), 0.0, 0.0, 400.0, 300.0);
        let tx = legend_first_text_attr(&svg, "x");
        assert!(tx > 260.0, "우측 범례 텍스트 x={tx} > 260 이어야");
        let ty = legend_first_text_attr(&svg, "y");
        assert!(ty < 250.0, "우측 범례는 플롯 세로 중앙부(y={ty} < 250)여야");
    }

    #[test]
    fn test_render_legend_bottom_default_unchanged() {
        // 기본(Bottom) → 종전 하단 가로 배치 유지.
        let svg = render_chart_svg(&named_chart(LegendPos::Bottom), 0.0, 0.0, 400.0, 300.0);
        let ty = legend_first_text_attr(&svg, "y");
        assert!(ty > 270.0, "하단 범례 텍스트 y={ty} > 270 이어야");
    }

    #[test]
    fn test_horizontal_bar_category_labels_not_clipped() {
        // 가로 막대: 좌측은 숫자 값축이 아니라 카테고리 라벨("항목 1" 등) —
        // left_pad를 값축 숫자 폭(2자≈32px)으로 잡으면 라벨이 차트 왼쪽 밖으로 잘림.
        // 카테고리 라벨 anchor x(= plot_x - 4)가 라벨 폭 이상 확보돼야 한다.
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Bar,
            series: vec![OoxmlSeries {
                values: vec![4.3, 2.5, 3.5, 4.5],
                series_type: OoxmlChartType::Bar,
                ..Default::default()
            }],
            categories: vec![
                "항목 1".into(),
                "항목 2".into(),
                "항목 3".into(),
                "항목 4".into(),
            ],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let chunk = svg.split(">항목 1<").next().expect("카테고리 라벨");
        let tag_start = chunk.rfind("<text ").expect("text 태그");
        let x = attr_f64_of(&chunk[tag_start..], "x=\"").expect("x 속성");
        assert!(
            x >= 45.0,
            "카테고리 라벨 anchor x={x} — 라벨 폭(≈40px)만큼 왼쪽 여백 필요"
        );
    }

    fn attr_f64_of(tag: &str, pat: &str) -> Option<f64> {
        let s = tag.find(pat)? + pat.len();
        let e = s + tag[s..].find('"')?;
        tag[s..e].parse().ok()
    }

    #[test]
    fn test_render_legend_right_narrow_chart_no_panic() {
        // 폭이 좁으면(w*0.30 < 50) clamp(50, w*0.30)이 min>max로 패닉하던 결함 가드 —
        // 하단 폴백으로 렌더되고 패닉하지 않아야 한다. NaN 폭도 패닉 금지.
        let svg = render_chart_svg(&named_chart(LegendPos::Right), 0.0, 0.0, 100.0, 80.0);
        assert!(
            svg.contains("hwp-chart-legend"),
            "좁은 차트는 하단 폴백 범례"
        );
        let _ = render_chart_svg(&named_chart(LegendPos::Right), 0.0, 0.0, f64::NAN, 80.0);
    }

    // --- C1c (#1882) 갭④: Y축 headroom + step 기반 눈금 (한컴 실측 앵커 3점) ---

    #[test]
    fn test_axis_headroom_bar_max_on_boundary() {
        // 한컴 실측 앵커: 세로막대 max 5.0 → 축 0~6, 세로 값축 3칸 정책으로
        // step 2 → 성긴 라벨 0,2,4,6 (묶은세로막대형-2022.pdf).
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Column,
            series: vec![
                OoxmlSeries {
                    values: vec![4.3, 2.5, 3.5, 4.5],
                    series_type: OoxmlChartType::Column,
                    ..Default::default()
                },
                OoxmlSeries {
                    values: vec![2.0, 2.0, 3.0, 5.0],
                    series_type: OoxmlChartType::Column,
                    ..Default::default()
                },
            ],
            categories: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        for want in [">0<", ">2<", ">4<", ">6<"] {
            assert!(svg.contains(want), "라벨 {want} 있어야 (0~6, step 2)");
        }
        for absent in [">1<", ">3<", ">5<"] {
            assert!(!svg.contains(absent), "라벨 {absent} 없어야 (성긴 라벨)");
        }
    }

    #[test]
    fn test_axis_vertical_stacked_coarse_ticks() {
        // 한컴 실측: 누적'세로'막대(합 max 12.3) → 축 0~15 step 5 (세로 값축은 ~3칸).
        // 같은 데이터의 누적'가로'막대는 0~14 step 2 — 방향별 눈금 밀도가 다름.
        let mut chart = bars_chart(BarGrouping::Stacked);
        chart.series[0].values = vec![4.3, 2.5, 3.5, 4.5];
        chart.series[1].values = vec![2.4, 4.4, 1.8, 2.8];
        chart.series[2].values = vec![2.0, 2.0, 3.0, 5.0];
        chart.categories = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        for want in [">5<", ">10<", ">15<"] {
            assert!(
                svg.contains(want),
                "세로 누적 라벨 {want} 있어야 (0~15 step 5)"
            );
        }
        for absent in [">14<", ">2<", ">4<"] {
            assert!(!svg.contains(absent), "세로 누적 라벨 {absent} 없어야");
        }
    }

    #[test]
    fn test_axis_horizontal_stacked_fine_ticks() {
        // 한컴 실측: 누적'가로'막대(합 max 12.3) → 축 0~14 step 2 (가로 값축은 ~5칸).
        let mut chart = bars_chart(BarGrouping::Stacked);
        chart.chart_type = OoxmlChartType::Bar;
        chart.series[0].values = vec![4.3, 2.5, 3.5, 4.5];
        chart.series[1].values = vec![2.4, 4.4, 1.8, 2.8];
        chart.series[2].values = vec![2.0, 2.0, 3.0, 5.0];
        for s in &mut chart.series {
            s.series_type = OoxmlChartType::Bar;
        }
        chart.categories = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        for want in [">2<", ">14<"] {
            assert!(
                svg.contains(want),
                "가로 누적 라벨 {want} 있어야 (0~14 step 2)"
            );
        }
        assert!(!svg.contains(">15<"), "가로 누적은 0~14 (15 아님)");
    }

    #[test]
    fn test_axis_horizontal_clustered_headroom_keeps_step() {
        // 한컴 실측: 묶은'가로'막대(max 5.0, step 1 경계) → 0~6 **step 1 유지**
        // (라벨 0~6 전부 — 경계 headroom 후 step 재계산하지 않음).
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Bar,
            series: vec![
                OoxmlSeries {
                    values: vec![4.3, 2.5, 3.5, 4.5],
                    series_type: OoxmlChartType::Bar,
                    ..Default::default()
                },
                OoxmlSeries {
                    values: vec![2.0, 2.0, 3.0, 5.0],
                    series_type: OoxmlChartType::Bar,
                    ..Default::default()
                },
            ],
            categories: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        for want in [">1<", ">3<", ">5<", ">6<"] {
            assert!(
                svg.contains(want),
                "가로 묶은 라벨 {want} 있어야 (0~6 step 1)"
            );
        }
    }

    #[test]
    fn test_axis_3d_clustered_no_headroom() {
        // 한컴 실측: 3D 묶은막대는 세로·가로 모두 0~5 step 1 — 촘촘 눈금 + 경계
        // headroom 없음 (2D 묶은세로 0~6 step 2 / 2D 묶은가로 0~6 step 1과 다름).
        for chart_type in [OoxmlChartType::Column, OoxmlChartType::Bar] {
            let chart = OoxmlChart {
                chart_type,
                is_3d: true,
                series: vec![OoxmlSeries {
                    values: vec![4.3, 2.5, 3.5, 5.0],
                    series_type: chart_type,
                    ..Default::default()
                }],
                categories: vec!["a".into(), "b".into(), "c".into(), "d".into()],
                ..Default::default()
            };
            let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
            for want in [">1<", ">4<", ">5<"] {
                assert!(
                    svg.contains(want),
                    "{chart_type:?}: 3D 묶은 라벨 {want} (0~5 step 1)"
                );
            }
            assert!(
                !svg.contains(">6<"),
                "{chart_type:?}: 3D 묶은은 headroom 없음 (0~5)"
            );
        }
    }

    #[test]
    fn test_axis_3d_stacked_vertical_extra_headroom() {
        // 한컴 실측: 3D 누적'세로'(합 max 12.3) → 0~20 step 5 (2D 15 + 1 step).
        let mut chart = bars_chart(BarGrouping::Stacked);
        chart.is_3d = true;
        chart.series[0].values = vec![4.3, 2.5, 3.5, 4.5];
        chart.series[1].values = vec![2.4, 4.4, 1.8, 2.8];
        chart.series[2].values = vec![2.0, 2.0, 3.0, 5.0];
        chart.categories = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains(">20<"), "3D 누적세로는 0~20 (2D 15 + 1 step)");
        assert!(!svg.contains(">14<"));

        // 3D 누적'가로'는 2D 가로와 동일 (0~14 step 2, 실측).
        let mut hchart = chart.clone();
        hchart.chart_type = OoxmlChartType::Bar;
        for s in &mut hchart.series {
            s.series_type = OoxmlChartType::Bar;
        }
        let hsvg = render_chart_svg(&hchart, 0.0, 0.0, 400.0, 300.0);
        assert!(hsvg.contains(">14<"), "3D 누적가로는 2D와 동일 0~14");
        assert!(!hsvg.contains(">16<") && !hsvg.contains(">20<"));
    }

    #[test]
    fn test_horizontal_bar_categories_bottom_up() {
        // 한컴 실측: 가로막대는 카테고리를 아래→위로 배치 (항목 1이 맨 아래).
        let chart = OoxmlChart {
            chart_type: OoxmlChartType::Bar,
            series: vec![OoxmlSeries {
                values: vec![1.0, 2.0],
                series_type: OoxmlChartType::Bar,
                ..Default::default()
            }],
            categories: vec!["catA".into(), "catB".into()],
            ..Default::default()
        };
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let y_of = |label: &str| -> f64 {
            let chunk = svg.split(&format!(">{label}<")).next().expect("라벨");
            let tag = &chunk[chunk.rfind("<text ").expect("text")..];
            attr_f64_of(tag, "y=\"").expect("y")
        };
        assert!(
            y_of("catA") > y_of("catB"),
            "첫 카테고리(catA)가 아래쪽(y 큼)이어야: catA={} catB={}",
            y_of("catA"),
            y_of("catB"),
        );
    }

    #[test]
    fn test_stacked_vertical_bars_align_with_category_labels() {
        // 누적 세로 막대 x가 plot_y 기반으로 계산되던 결함 가드 — 막대 중심이
        // 카테고리 라벨 중심과 일치해야 한다 (y 오프셋이 있는 배치에서 검증).
        let svg = render_chart_svg(&bars_chart(BarGrouping::Stacked), 0.0, 100.0, 400.0, 300.0);
        let label_chunk = svg.split(">a<").next().expect("라벨 a");
        let label_x = attr_f64_of(
            &label_chunk[label_chunk.rfind("<text ").expect("text")..],
            "x=\"",
        )
        .expect("라벨 x");
        let bar_chunk = svg.split("fill=\"#6183d7\"").next().expect("첫 파랑 막대");
        let bar_tag = &bar_chunk[bar_chunk.rfind("<rect ").expect("rect")..];
        let bar_center = attr_f64_of(bar_tag, "x=\"").expect("x")
            + attr_f64_of(bar_tag, "width=\"").expect("w") / 2.0;
        assert!(
            (bar_center - label_x).abs() < 2.0,
            "누적 막대 중심({bar_center})과 라벨 중심({label_x}) 불일치",
        );
    }

    #[test]
    fn test_axis_headroom_scatter_y_on_boundary() {
        // 한컴 실측 앵커: scatter Y max 4.0(step 1 경계) → 축 0~5, 라벨 1 간격
        // (표식만있는분산형-2022.pdf).
        let mut chart = scatter_chart(ScatterStyle::Marker);
        chart.series[0].values = vec![2.7, 3.2, 4.0];
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(svg.contains(">5<"), "Y축 headroom: max 4.0 → 축 0~5");
        assert!(svg.contains(">4<"), "step 1 라벨 유지");
    }

    #[test]
    fn test_axis_no_headroom_when_max_off_boundary() {
        // 한컴 실측 앵커: scatter X max 2.6(경계 아님) → 축 0~3, step 0.5 유지
        // (무조건 step 재계산 시 1.0으로 승격되는 회귀 방지).
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Marker), 0.0, 0.0, 400.0, 300.0);
        for want in [">0.5<", ">2.5<", ">3<"] {
            assert!(svg.contains(want), "X축 {want} 있어야 (0~3, step 0.5)");
        }
    }
}
