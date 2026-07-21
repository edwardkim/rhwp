//! OOXML 차트 → SVG 네이티브 렌더러
//!
//! `OoxmlChart` 데이터 모델을 지정된 bbox 안에 SVG 문자열로 그린다.
//! - 세로/가로 막대, 꺾은선, 원형
//! - **콤보 차트** (bar + line) 및 **이중 Y축** 지원

use super::{
    BarGrouping, LegendPos, OfPieInfo, OfPieType, OoxmlChart, OoxmlChartType, OoxmlSeries,
    ScatterStyle, SeriesMarker, View3D,
};

/// 기본 시리즈 색상 팔레트 (시리즈 색상 미지정 시 순환 사용)
///
/// 한컴 2022 기본 팔레트(`hncChartStyle colorIndex="0"`) — 앞 5색은 `pdf/chart/` 정답지
/// PDF 픽셀 실측(막대 3시리즈 + 원형 4슬라이스 + ofPie 결합 슬라이스), 6번째 이후는
/// 코퍼스에 초과 샘플이 없어 미실측(Office 유사색 순서로 유추 배치).
const DEFAULT_PALETTE: &[u32] = &[
    0xFF6183D7, // 파랑 (실측)
    0xFFFE813B, // 주황 (실측)
    0xFFB0B0B0, // 회색 (실측)
    0xFFFCD801, // 노랑 (실측)
    0xFF27A172, // 초록계 (실측 — ofPie 결합 슬라이스, 원형대원형·가로막대 교차 일치)
    0xFF5B9BD5, // 하늘 (유추 — [4]에서 강등)
    0xFF9013FE, 0xFF50E3C2,
];

fn palette(i: usize) -> u32 {
    DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()]
}

fn color_hex(c: u32) -> String {
    format!("#{:06x}", c & 0xFFFFFF)
}

/// RGB 음영 — factor>0 은 흰색 방향 lighten, factor<0 은 검정 방향 darken.
/// 채널별 선형 보간, 상위(알파) 바이트는 보존. 3D 면 음영용. (C2b #2278)
fn shade(rgb: u32, factor: f64) -> u32 {
    let f = factor.clamp(-1.0, 1.0);
    let ch = |c: u32| -> u32 {
        let c = c as f64;
        let v = if f >= 0.0 {
            c + (255.0 - c) * f
        } else {
            c * (1.0 + f)
        };
        v.round().clamp(0.0, 255.0) as u32
    };
    (rgb & 0xFF00_0000)
        | (ch((rgb >> 16) & 0xFF) << 16)
        | (ch((rgb >> 8) & 0xFF) << 8)
        | ch(rgb & 0xFF)
}

/// 3D 막대 면 음영 계수 — 정답지 4종 판독 근사(윗면 밝게/우측면 어둡게,
/// stage1 실측 기록), 시각판정 보정 대상. (C2b #2278)
const BAR3D_TOP_SHADE: f64 = 0.25;
const BAR3D_SIDE_SHADE: f64 = -0.25;

/// rAngAx=1(직각 축 — 한컴 차트 스펙 rev1.2 표 100 ProjectionType=1
/// "2.5차원: 회전·상승해도 XY 면 불변"과 동계) 시어 투영 사전계산.
/// 씬 = 앞면 플롯평면 × 깊이 [0..D], 화면 깊이 벡터 = (+sin(rotY), −sin(rotX))·D.
/// 투영 bbox를 플롯 rect에 비등방 fit — 앞면은 좌하 고정, 뒷벽이 우상으로.
/// rAngAx=0 막대는 코퍼스 부재 — 동일 시어 폴백(근사, 진짜 회전 투영은 Stage 2
/// 원형에서 rot_x 유도로 도입). (C2b #2278 v2)
struct ShearProj {
    /// fit 후 앞면(z=0) rect — 3D 배치 수식의 기준
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
    /// fit 후 z=D 화면 오프셋 (+우/+상)
    dxf: f64,
    dyf: f64,
}

fn shear_proj(view: &View3D, px: f64, py: f64, pw: f64, ph: f64, depth: f64) -> ShearProj {
    // 음수 시어 성분(rotX<0 하향, sin(rotY)<0 좌향)은 코퍼스 부재 + 페인트
    // 순서(아래→위/왼→오른쪽 은면 제거)와 상충 — 0 클램프 방어 근사.
    // 실샘플 확보 시 순회 방향 반전과 함께 확장. (v2 설계 리뷰 확정)
    let ox = (depth * view.rot_y.to_radians().sin()).max(0.0);
    let oy = (depth * view.rot_x.to_radians().sin()).max(0.0);
    let sx = pw / (pw + ox).max(1e-9);
    let sy = ph / (ph + oy).max(1e-9);
    ShearProj {
        fx: px,           // ox ≥ 0 → 앞면 좌측 고정
        fy: py + oy * sy, // oy ≥ 0 → 앞면 하단 고정 (화면 y-down: 상단 여백 = oy·sy)
        fw: pw * sx,
        fh: ph * sy,
        dxf: ox * sx,
        dyf: oy * sy,
    }
}

/// 3D 막대 1개(또는 누적 세그먼트 1개) — 압출 벡터 (+dx, −dy) 3면:
/// top 평행사변형(밝게) + right 평행사변형(어둡게) + front rect(원색).
/// dx, dy ≥ 0 전제(shear_proj 클램프 — 음수 성분은 페인트 순서 은면 제거와
/// 상충하여 정의역 밖). 은면 제거는 호출측 페인트 순서(누적: 아래→위/왼→오른쪽)가
/// 담당하므로 루프 순서 변경 금지. w/h ≤ 0(0값 세그먼트)이면 무방출 — 누적에서
/// 이웃 세그먼트의 캡 재도색 방지. 압출 퇴화(dx,dy < 0.01)면 front만. (C2b #2278 v2)
fn push_bar_3d(svg: &mut String, x: f64, y: f64, w: f64, h: f64, dx: f64, dy: f64, color: u32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    debug_assert!(dx >= 0.0 && dy >= 0.0, "시어 성분은 클램프로 비음수");
    if dx > 0.01 || dy > 0.01 {
        svg.push_str(&format!(
            "<polygon class=\"hwp-bar3d-top\" points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\"/>\n",
            x, y, x + dx, y - dy, x + w + dx, y - dy, x + w, y,
            color_hex(shade(color, BAR3D_TOP_SHADE))
        ));
        svg.push_str(&format!(
            "<polygon class=\"hwp-bar3d-side\" points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\"/>\n",
            x + w, y, x + w + dx, y - dy, x + w + dx, y + h - dy, x + w, y + h,
            color_hex(shade(color, BAR3D_SIDE_SHADE))
        ));
    }
    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
        x,
        y,
        w,
        h,
        color_hex(color)
    ));
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

    // 파이 차트는 단독 경로 (ofPie 보조플롯 / 3D 타원+측벽 — 2D 경로 무접촉)
    if chart.chart_type == OoxmlChartType::Pie {
        if let Some(of) = &chart.of_pie {
            render_of_pie(&mut svg, chart, of, plot_x, plot_y, plot_w, plot_h);
        } else if chart.is_3d {
            render_pie_3d(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
        } else {
            render_pie(&mut svg, chart, plot_x, plot_y, plot_w, plot_h);
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
            OoxmlChartType::Stock => render_stock(&mut svg, chart, plot_x, plot_y, plot_w, plot_h),
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
        let (mn, mx, st) = value_range(chart, ticks);
        // 특이케이스 실측(C1c v2 기록 → #2277 stage5): 가로막대 1카테고리 미니차트는
        // 축 범위 유지·step 절반 (4.3 → 0~5 step 0.5, 라벨 11개). 기존 가로축 앵커
        // (12.3→2 / 5.0→1 / 2.6→0.5)와 단일 규칙 불성립 — 단일 샘플 근거라
        // 가로·1카테고리(·이 분기 자체로 비누적·비3D)로 좁게 게이트. 세로 1카테고리는
        // 미실측이라 불변.
        if horizontal && cat_count == 1 {
            (mn, mx, st / 2.0)
        } else {
            (mn, mx, st)
        }
    };

    if chart.is_3d {
        // 3D는 시어 투영 기반 별도 경로 — 축 범위(vmin/vmax/vstep)는 위에서
        // 플롯 rect 기준으로 이미 확정(#1882 앵커 무접촉). 배치·방·압출만
        // 투영 좌표로 수행. (C2b #2278 v2)
        render_bars_3d(
            svg, chart, px, py, pw, ph, horizontal, stacked, percent, cat_count, ser_count, vmin,
            vmax, vstep,
        );
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
                    // 슬롯 내 세로 배치: 계열1이 맨 아래 (정답지 실측 — 위→아래 =
                    // 계열3→1, 범례 역순과 시각 일치. #2277 stage3)
                    let cy = py
                        + cat_slot(ci)
                        + (cat_span - bar_span_total) / 2.0
                        + bar_w * (ser_count - 1 - si) as f64;
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

// ---------------- Stock (주식형, hiLowLines/upDownBars) ----------------

/// stock (주식형). 계열 역할 = XML 순서 규약: 3계열=고/저/종, 4계열=시/고/저/종
/// (코퍼스 실측 — 그 외 계열 수는 render_line 폴백으로 placeholder 재발 방지).
/// 정답지 정합: 검정 고저선 + (OHLC) 시가↔종가 캔들(하락=진회색 채움/상승=흰
/// 채움+검정 테두리) + 종가 마커(마커 사이클·팔레트 폴백이 ▲회색/×노랑을 자동
/// 결정 — 시/고/저는 `c:symbol val="none"`이라 무마커). (C2a #2277)
fn render_stock(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let (hi_i, lo_i, close_i, open_i) = match chart.series.len() {
        3 => (0usize, 1usize, 2usize, None),
        4 => (1, 2, 3, Some(0usize)),
        _ => return render_line(svg, chart, px, py, pw, ph),
    };
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

    // 값축: stock 전용 무조건 +1 step 헤드룸 — 정답지 실측 max 59 → 0~80 step 20.
    // nice_axis의 경계 조건부 +1로는 0~60이라 부족. 3D 누적세로(+1 step)와 동형 패턴.
    let (raw_min, raw_max) = raw_value_bounds(chart.series.iter());
    let (vmin, mx, vstep) = nice_axis_no_headroom(raw_min, raw_max, VERTICAL_AXIS_TICKS);
    let vmax = mx + vstep;

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
        false,
        false,
    );

    let y_of = |v: f64| -> f64 {
        let t = if vmax > vmin {
            (v - vmin) / (vmax - vmin)
        } else {
            0.0
        };
        py + ph - ph * t
    };
    let val = |si: usize, ci: usize| -> Option<f64> {
        chart.series.get(si).and_then(|s| s.values.get(ci)).copied()
    };
    let cat_span = pw / cat_count as f64;
    // 캔들 폭 = cat_span / (1 + gapWidth/100) — 정답지 gapWidth=150 → 슬롯의 40%
    let gap = chart.up_down_gap_width.unwrap_or(150.0).max(0.0);
    let candle_w = cat_span / (1.0 + gap / 100.0);

    for ci in 0..cat_count {
        let x = px + cat_span * (ci as f64 + 0.5);
        if chart.has_hi_low_lines {
            if let (Some(hi), Some(lo)) = (val(hi_i, ci), val(lo_i, ci)) {
                svg.push_str(&format!(
                    "<line class=\"hwp-stock-hilow\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#000000\" stroke-width=\"1\"/>\n",
                    x,
                    y_of(hi),
                    x,
                    y_of(lo)
                ));
            }
        }
        if chart.has_up_down_bars {
            if let (Some(open), Some(close)) = (open_i.and_then(|oi| val(oi, ci)), val(close_i, ci))
            {
                let top = y_of(open.max(close));
                let bot = y_of(open.min(close));
                // 하락(종<시)=진회색 채움(#404040 근사 — 시각판정에서 픽셀 실측 확정),
                // 상승·동률=흰 채움+검정 테두리 (동률은 미실측 — 상승 처리 고정).
                let (fill, stroke) = if close < open {
                    ("#404040", "none")
                } else {
                    ("#ffffff", "#000000")
                };
                svg.push_str(&format!(
                    "<rect class=\"hwp-stock-candle\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>\n",
                    x - candle_w / 2.0,
                    top,
                    candle_w,
                    (bot - top).max(0.5),
                    fill,
                    stroke
                ));
            }
        }
    }

    // 마커: Auto/Named 계열만 (코퍼스 = 종가만 Auto). 고저선/캔들 위에 그린다.
    for (si, ser) in chart.series.iter().enumerate() {
        if !matches!(
            ser.marker_symbol,
            SeriesMarker::Auto | SeriesMarker::Named(_)
        ) {
            continue;
        }
        let color = series_color(ser, si);
        for (ci, &v) in ser.values.iter().enumerate() {
            let x = px + cat_span * (ci as f64 + 0.5);
            push_marker(svg, "hwp-chart-marker", si, x, y_of(v), 3.5, &color);
        }
    }

    render_category_labels(svg, chart, px, py, pw, ph, cat_count, false);
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
            // 계열 사이클 글리프 ◆■▲× — 정답지 실측(표식만있는분산형: 계열1 ◆/계열2 ■).
            // 반경 4.5 = 라인(3.5)보다 큰 실측 근사, 시각판정 조정 여지. (C2a #2277)
            for &(xp, yp) in &points {
                push_marker(svg, "hwp-chart-marker", si, xp, yp, 4.5, &color);
            }
        }
    }
}

/// 마커 경로. 계열 인덱스 사이클 ◆■▲× — 한컴 기본 (정답지 실측: 라인 ◆■▲
/// C1d #2129, OHLC 종가 ×·scatter ◆■ C2a #2277). 반환 `(d, stroke 기반 여부)` —
/// ×는 채움 없는 열린 경로라 stroke=계열색으로 그린다. `r`=명목 반경, ■/×는
/// 하프폭 `r-0.5` (종전 ◆3.5/■3.0 비율 유지 — 출력 바이트 보존).
fn marker_path(si: usize, cx: f64, cy: f64, r: f64) -> (String, bool) {
    let h = r - 0.5;
    match si % 4 {
        0 => (
            // ◆ 다이아몬드
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
            ),
            false,
        ),
        1 => (
            // ■ 정사각형
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
            ),
            false,
        ),
        2 => (
            // ▲ 삼각형
            format!(
                "M{:.2},{:.2} L{:.2},{:.2} L{:.2},{:.2} Z",
                cx,
                cy - r,
                cx + r,
                cy + r * 0.8,
                cx - r,
                cy + r * 0.8
            ),
            false,
        ),
        _ => (
            // × 두 대각선 열린 경로 — OHLC 종가 정답지 실측 (C2a #2277, 종전 원 폴백 교체)
            format!(
                "M{:.2},{:.2} L{:.2},{:.2} M{:.2},{:.2} L{:.2},{:.2}",
                cx - h,
                cy - h,
                cx + h,
                cy + h,
                cx + h,
                cy - h,
                cx - h,
                cy + h
            ),
            true,
        ),
    }
}

/// 마커 1개 방출. 채움형(◆■▲)=fill 계열색+흰 테두리, stroke 기반(×)=fill 없이
/// stroke 계열색. `class`로 데이터 마커("hwp-chart-marker")와 범례 글리프
/// ("hwp-legend-glyph", 4단계)를 구분 — issue_2129 마커 카운트 오염 방지. (C2a #2277)
fn push_marker(svg: &mut String, class: &str, si: usize, cx: f64, cy: f64, r: f64, color: &str) {
    let (d, stroke_based) = marker_path(si, cx, cy, r);
    if stroke_based {
        svg.push_str(&format!(
            "<path class=\"{}\" d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
            class, d, color
        ));
    } else {
        svg.push_str(&format!(
            "<path class=\"{}\" d=\"{}\" fill=\"{}\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n",
            class, d, color
        ));
    }
}

/// 라인 차트 표식(마커) — `marker_path` 사이클, 반경 3.5(정답지 근사, 시각판정
/// 조정 여지). (C1d #2129)
fn push_line_marker(svg: &mut String, si: usize, cx: f64, cy: f64, color: &str) {
    push_marker(svg, "hwp-chart-marker", si, cx, cy, 3.5, color);
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
    // 쪼개진원형: 계열 explosion(%)만큼 슬라이스를 중심각 방향으로 이동 —
    // 벌어진 extent(r×(1+e))가 기존 fit과 같도록 반지름 축소. explosion 부재
    // (e=0) 시 기존 산식·출력 그대로. (C2b #2278 Stage 3 v3, 정답지 쪼개진원형)
    let explode = first.explosion.unwrap_or(0.0).max(0.0) / 100.0;
    let r = (pw.min(ph) / 2.0) * 0.9 / (1.0 + explode);

    let mut start_angle = -std::f64::consts::FRAC_PI_2;
    for (i, &v) in first.values.iter().enumerate() {
        let sweep = v / total * std::f64::consts::TAU;
        let end_angle = start_angle + sweep;
        let mid = start_angle + sweep / 2.0;
        let (ox, oy) = (cx + r * explode * mid.cos(), cy + r * explode * mid.sin());
        let (x1, y1) = (ox + r * start_angle.cos(), oy + r * start_angle.sin());
        let (x2, y2) = (ox + r * end_angle.cos(), oy + r * end_angle.sin());
        let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
        let color = color_hex(first.color.unwrap_or_else(|| palette(i)));
        svg.push_str(&format!(
            "<path d=\"M{:.2},{:.2} L{:.2},{:.2} A{:.2},{:.2} 0 {} 1 {:.2},{:.2} Z\" fill=\"{}\"/>\n",
            ox, oy, x1, y1, r, r, large, x2, y2, color
        ));
        start_angle = end_angle;
    }
}

/// ofPie — 주 원(앞 n−k 카테고리 + 결합 슬라이스) + 보조 플롯(pie|bar) + serLines.
/// k = split_pos(반올림, 1..=n−1 클램프) 없으면 2. n < 3 → 일반 원형 폴백.
/// 주 원은 결합 슬라이스 중앙이 보조 플롯(3시 방향)을 향하도록 회전. 결합 슬라이스
/// 색 = palette(n)(정답지 실측 초록계 [4]), 보조 플롯 색 = palette(n−k+j)
/// (범례의 카테고리 정순 색과 일치). (C2b #2278 Stage 3)
fn render_of_pie(
    svg: &mut String,
    chart: &OoxmlChart,
    of: &OfPieInfo,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
) {
    use std::f64::consts::TAU;
    let first = match chart.series.first() {
        Some(s) => s,
        None => return,
    };
    let n = first.values.len();
    if n < 3 {
        render_pie(svg, chart, px, py, pw, ph);
        return;
    }
    let total: f64 = first.values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let k = of
        .split_pos
        .map(|v| (v.round() as usize).clamp(1, n - 1))
        .unwrap_or(2)
        .min(n - 1);
    let combined: f64 = first.values[n - k..].iter().sum();

    // 레이아웃 — 정답지(원형대원형-2022 임베드 2702×1577) 픽셀 실측 캘리브레이션:
    // 주 원 중심 x≈0.23·플롯폭, 보조 중심 x≈0.80, r1≈0.38·플롯높이(가로는 캡),
    // r2/r1 실측 0.754 = secondPieSize(75)/100 ✓ (스키마 의미 그대로)
    let cx1 = px + pw * 0.23;
    let cy = py + ph / 2.0;
    let r1 = (pw * 0.46).min(ph * 0.76) / 2.0;
    let cx2 = px + pw * 0.80;
    let r2 = r1 * (of.second_pie_size.max(0.0) / 100.0);

    // 주 원 — 값 시퀀스 = values[..n−k] + [combined]. 결합 슬라이스 중앙이 3시(θ=0)
    let sweep_c = combined / total * TAU;
    let start_main = -sweep_c / 2.0 - (total - combined) / total * TAU;
    let main_vals: Vec<(f64, u32)> = first.values[..n - k]
        .iter()
        .enumerate()
        .map(|(ci, &v)| (v, first.color.unwrap_or_else(|| palette(ci))))
        .chain(std::iter::once((
            combined,
            first.color.unwrap_or_else(|| palette(n)),
        )))
        .collect();
    let mut start_angle = start_main;
    for (v, rgb) in &main_vals {
        let sweep = v / total * TAU;
        let end_angle = start_angle + sweep;
        let (x1, y1) = (cx1 + r1 * start_angle.cos(), cy + r1 * start_angle.sin());
        let (x2, y2) = (cx1 + r1 * end_angle.cos(), cy + r1 * end_angle.sin());
        let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
        svg.push_str(&format!(
            "<path class=\"hwp-ofpie-main\" d=\"M{:.2},{:.2} L{:.2},{:.2} A{:.2},{:.2} 0 {} 1 {:.2},{:.2} Z\" fill=\"{}\"/>\n",
            cx1, cy, x1, y1, r1, r1, large, x2, y2, color_hex(*rgb)
        ));
        start_angle = end_angle;
    }

    if combined > 0.0 {
        match of.of_pie_type {
            OfPieType::Pie => {
                // 보조 원 — 시작각 = +sweep_c/2 (결합 슬라이스 아래 모서리 각도와
                // 정렬, 정답지 실측 경계 26°≈유도 30°; 12시 시작 아님)
                let mut s = sweep_c / 2.0;
                for (j, &v) in first.values[n - k..].iter().enumerate() {
                    let sweep = v / combined * TAU;
                    let e = s + sweep;
                    let (x1, y1) = (cx2 + r2 * s.cos(), cy + r2 * s.sin());
                    let (x2, y2) = (cx2 + r2 * e.cos(), cy + r2 * e.sin());
                    let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
                    let rgb = first.color.unwrap_or_else(|| palette(n - k + j));
                    svg.push_str(&format!(
                        "<path class=\"hwp-ofpie-second\" d=\"M{:.2},{:.2} L{:.2},{:.2} A{:.2},{:.2} 0 {} 1 {:.2},{:.2} Z\" fill=\"{}\"/>\n",
                        cx2, cy, x1, y1, r2, r2, large, x2, y2, color_hex(rgb)
                    ));
                    s = e;
                }
            }
            OfPieType::Bar => {
                // 보조 누적 막대 — 첫 분할 카테고리가 맨 위, 위→아래 누적
                let bar_h = 2.0 * r2;
                let bar_w = bar_h * 0.45;
                let bx = cx2 - bar_w / 2.0;
                let top = cy - bar_h / 2.0;
                let mut acc = 0.0;
                for (j, &v) in first.values[n - k..].iter().enumerate() {
                    let seg = v / combined * bar_h;
                    let rgb = first.color.unwrap_or_else(|| palette(n - k + j));
                    svg.push_str(&format!(
                        "<rect class=\"hwp-ofpie-second\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                        bx,
                        top + acc,
                        bar_w,
                        seg,
                        color_hex(rgb)
                    ));
                    acc += seg;
                }
            }
        }

        // serLines — 결합 슬라이스 양 모서리 → 보조 플롯. 색은 검정(정답지 실측
        // 코어 (8,8,8)). Pie는 보조 원 **접선점**(실측: 원 상/하단 아님 —
        // 모서리에서 원에 접하는 선), Bar는 막대 좌변 상/하단.
        if of.has_ser_lines {
            let (ux, uy) = (
                cx1 + r1 * (-sweep_c / 2.0).cos(),
                cy + r1 * (-sweep_c / 2.0).sin(),
            );
            let (lx, ly) = (
                cx1 + r1 * (sweep_c / 2.0).cos(),
                cy + r1 * (sweep_c / 2.0).sin(),
            );
            // 외부점 P → 원(cx2, cy, r2) 접점: 중심→P 각 α, β=acos(r2/d), α±β 중
            // 위 연결선은 위쪽(작은 y) 접점 선택. d ≤ r2 퇴화 시 원 좌측점 폴백.
            let tangent = |pxp: f64, pyp: f64, upper: bool| -> (f64, f64) {
                let (dx, dy) = (pxp - cx2, pyp - cy);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= r2 {
                    return (cx2 - r2, cy);
                }
                let alpha = dy.atan2(dx);
                let beta = (r2 / d).acos();
                let p1 = (
                    cx2 + r2 * (alpha + beta).cos(),
                    cy + r2 * (alpha + beta).sin(),
                );
                let p2 = (
                    cx2 + r2 * (alpha - beta).cos(),
                    cy + r2 * (alpha - beta).sin(),
                );
                if (p1.1 < p2.1) == upper {
                    p1
                } else {
                    p2
                }
            };
            let ((tx, ty), (bx2, by2)) = match of.of_pie_type {
                OfPieType::Pie => (tangent(ux, uy, true), tangent(lx, ly, false)),
                OfPieType::Bar => {
                    let bar_h = 2.0 * r2;
                    let bar_w = bar_h * 0.45;
                    let bx = cx2 - bar_w / 2.0;
                    ((bx, cy - r2), (bx, cy + r2))
                }
            };
            for ((x1, y1), (x2, y2)) in [((ux, uy), (tx, ty)), ((lx, ly), (bx2, by2))] {
                svg.push_str(&format!(
                    "<line class=\"hwp-ofpie-serline\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#000000\" stroke-width=\"0.75\"/>\n",
                    x1, y1, x2, y2
                ));
            }
        }
    }
}

/// 3D 원형 측벽 높이 / rx — 정답지(3차원원형-2022, rotX=30/persp=30) 픽셀 실측
/// 175px/846.5px. 한컴 스펙 표 43 Pie.ThicknessRatio(반지름의 백분율) 구조 채택,
/// 비율값은 실측 캘리브레이션. hPercent(기본 100)로 스케일.
const PIE3D_WALL_RATIO: f64 = 0.207;

/// 3D 원형 — 타원(top) 슬라이스 + 하반부 측벽 밴드 (rAngAx=0 회전+원근).
///
/// 타원비 `ry/rx = sin(rotX)·cos(perspective/2°)` — 정답지 실측 0.480과 유도
/// 0.483이 0.5% 이내 정합(캘리브레이션 1점). 앞/뒤 반타원이 실측 대칭(407.5px
/// 동일)이라 원근 나눗셈(비대칭) 모델은 기각 — 대칭 타원 유지. rotY는 시작각
/// 오프셋. 벽 전체를 먼저, top을 나중에 그린다(은면 제거 — 페인트 순서).
/// 2D `render_pie`는 무접촉(바이트 불변). (C2b #2278 Stage 2)
fn render_pie_3d(svg: &mut String, chart: &OoxmlChart, px: f64, py: f64, pw: f64, ph: f64) {
    let first = match chart.series.first() {
        Some(s) => s,
        None => return,
    };
    let total: f64 = first.values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let view = chart.view3d.clone().unwrap_or_default();
    // 정의역 방어: rotX 5..90 클램프(0° 타원 퇴화 차단), perspective 0..90 클램프
    // (perspective/2 > 90° → cos 음수 차단) — 코퍼스(30/30) 밖 카메라 방어
    let tilt = view.rot_x.clamp(5.0, 90.0).to_radians().sin();
    let persp = (view.perspective.clamp(0.0, 90.0) / 2.0).to_radians().cos();
    let ratio = tilt * persp;
    let wall_k = PIE3D_WALL_RATIO * (view.h_percent.max(0.0) / 100.0);
    // 반경: 타원+벽 블록(가로 2rx, 세로 (2·타원비+벽비)·rx)이 플롯에 맞는 최대 —
    // 2D의 원 fit(min(pw,ph))과 달리 납작한 타원의 세로 여유를 사용 (한컴 실측:
    // rx=846.5 ≈ 플롯 절반폭 × 0.85, 세로는 비구속)
    let rx = (pw / 2.0).min(ph / (2.0 * ratio + wall_k).max(1e-6)) * 0.9;
    let ry = rx * ratio;
    let wall_h = rx * wall_k;
    let cx = px + pw / 2.0;
    let cy = py + (ph - wall_h) / 2.0;

    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    let start0 = -FRAC_PI_2 + view.rot_y.to_radians();

    // 1차 루프 — 하반부(θ∈(0,π), SVG y-down에서 벽 노출면) 측벽. rotY 오프셋으로
    // 각도가 τ를 넘을 수 있어 (0,π)와 (τ,τ+π) 두 윈도우 클립 — 랩어라운드 불요.
    let mut s = start0;
    for (i, &v) in first.values.iter().enumerate() {
        let e = s + v / total * TAU;
        let rgb = first.color.unwrap_or_else(|| palette(i));
        for (w0, w1) in [(0.0, PI), (TAU, TAU + PI)] {
            let a = s.max(w0);
            let b = e.min(w1);
            if b - a > 1e-6 {
                let (xa, ya) = (cx + rx * a.cos(), cy + ry * a.sin());
                let (xb, yb) = (cx + rx * b.cos(), cy + ry * b.sin());
                svg.push_str(&format!(
                    "<path class=\"hwp-pie3d-wall\" d=\"M{:.2},{:.2} A{:.2},{:.2} 0 0 1 {:.2},{:.2} L{:.2},{:.2} A{:.2},{:.2} 0 0 0 {:.2},{:.2} Z\" fill=\"{}\"/>\n",
                    xa,
                    ya,
                    rx,
                    ry,
                    xb,
                    yb,
                    xb,
                    yb + wall_h,
                    rx,
                    ry,
                    xa,
                    ya + wall_h,
                    color_hex(shade(rgb, BAR3D_SIDE_SHADE))
                ));
            }
        }
        s = e;
    }

    // 2차 루프 — top 타원 슬라이스 (2D render_pie 로직의 타원호 버전)
    let mut start_angle = start0;
    for (i, &v) in first.values.iter().enumerate() {
        let sweep = v / total * TAU;
        let end_angle = start_angle + sweep;
        let (x1, y1) = (cx + rx * start_angle.cos(), cy + ry * start_angle.sin());
        let (x2, y2) = (cx + rx * end_angle.cos(), cy + ry * end_angle.sin());
        let large = if sweep > PI { 1 } else { 0 };
        let color = color_hex(first.color.unwrap_or_else(|| palette(i)));
        svg.push_str(&format!(
            "<path class=\"hwp-pie3d-top\" d=\"M{:.2},{:.2} L{:.2},{:.2} A{:.2},{:.2} 0 {} 1 {:.2},{:.2} Z\" fill=\"{}\"/>\n",
            cx, cy, x1, y1, rx, ry, large, x2, y2, color
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

/// 3D 방 선 스타일 — 정답지 임베드(2702px) 픽셀 실측: 축선·조그·격자·틱 전부
/// #808080 균일(gray 126~148)·0.72pt≈0.75. 2D의 축/격자 명암 구분(#e8e8e8)과
/// 다른 3D 전용 실측 규칙 (시각판정 피드백 2026-07-19).
const ROOM_LINE_STYLE: &str = "stroke=\"#808080\" stroke-width=\"0.75\"";
/// 값·카테고리 좌측 틱 길이(pt) — 실측 44/38px ÷ 8.34px/pt ≈ 5.3/4.6
const ROOM_TICK_LEFT: f64 = 5.0;
/// 하단 틱 길이(pt) — 실측 31~37px ≈ 3.7~4.4
const ROOM_TICK_DOWN: f64 = 4.0;

/// 3D 방 + 값축 격자·라벨 — 시어 투영 기반: 뒷벽(z=D, 격자 포함) + 눈금별
/// 바닥 조그 + 바닥 평행사변형 + 값·카테고리 틱. 격자는 `<line>` 2개(조그+뒷벽)로
/// 방출(`<polyline>` 미사용 — room 테스트 어휘 유지). 라벨 문자열·포맷은
/// render_value_grid와 동일(#1882 라벨 앵커) — 위치는 fit 후 앞면 rect 기준.
/// 한컴 실측(2026-07-19): 벽 테두리·바닥 채움 없음, 바닥 외곽선·틱은 격자와
/// 동일 스타일. 2D 경로는 이 함수를 쓰지 않는다. (C2b #2278 v2)
#[allow(clippy::too_many_arguments)]
fn render_value_grid_3d(
    svg: &mut String,
    proj: &ShearProj,
    vmin: f64,
    vmax: f64,
    step: f64,
    format_code: Option<&str>,
    horizontal: bool,
    percent: bool,
    cat_count: usize,
) {
    let (fx, fy, fw, fh) = (proj.fx, proj.fy, proj.fw, proj.fh);
    let (dxf, dyf) = (proj.dxf, proj.dyf);
    // 방 표면: 뒷벽(흰 면, 무테두리 — 한컴: 벽 모서리선 없음) → 바닥(흰 면 +
    // #808080 외곽선; 뒷모서리·좌측 대각은 0 눈금 격자/조그와 겹침) → 앞면 좌측 축선
    svg.push_str(&format!(
        "<g class=\"hwp-bar3d-room\">\n<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\"/>\n",
        fx + dxf,
        fy - dyf,
        fw,
        fh
    ));
    svg.push_str(&format!(
        "<polygon points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"#ffffff\" {ROOM_LINE_STYLE}/>\n",
        fx,
        fy + fh,
        fx + dxf,
        fy + fh - dyf,
        fx + fw + dxf,
        fy + fh - dyf,
        fx + fw,
        fy + fh
    ));
    svg.push_str(&format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
        fx,
        fy,
        fx,
        fy + fh
    ));

    // 라벨 문자열 규칙은 render_value_grid와 동일 (#1882 라벨 앵커 — 위치는
    // 앞면 rect 기준)
    let decimal = (step - step.round()).abs() > 1e-9;
    let label = |v: f64| -> String {
        if percent {
            format!("{}%", v.round() as i64)
        } else if decimal {
            format_axis_num(v)
        } else {
            format_num(v, format_code)
        }
    };
    let span = (vmax - vmin).max(1e-9);
    let step = if step > 0.0 { step } else { span / 5.0 };
    let grid_lines = (span / step).round().max(1.0) as usize;
    for i in 0..=grid_lines {
        let t = (step * i as f64) / span;
        let v = vmin + step * i as f64;
        if horizontal {
            let gx = fx + fw * t;
            // 바닥 조그(앞 눈금 → 뒷벽, 깊이 D) + 뒷벽 세로 격자 + 하단 값 틱
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                gx,
                fy + fh,
                gx + dxf,
                fy + fh - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                gx + dxf,
                fy + fh - dyf,
                gx + dxf,
                fy - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                gx,
                fy + fh,
                gx,
                fy + fh + ROOM_TICK_DOWN
            ));
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"middle\">{}</text>\n",
                gx,
                fy + fh + 12.0,
                xml_escape(&label(v))
            ));
        } else {
            let gy = fy + fh - fh * t;
            // 바닥 조그(앞 좌측 눈금 → 뒷벽) + 뒷벽 수평 격자 + 좌측 값 틱
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx,
                gy,
                fx + dxf,
                gy - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx + dxf,
                gy - dyf,
                fx + fw + dxf,
                gy - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx - ROOM_TICK_LEFT,
                gy,
                fx,
                gy
            ));
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#666\" text-anchor=\"end\">{}</text>\n",
                fx - ROOM_TICK_LEFT - 1.0,
                gy + 3.0,
                xml_escape(&label(v))
            ));
        }
    }
    // 카테고리 경계 틱 — 경계+양끝 = cat_count+1개 (한컴 실측: 세로형은 바닥
    // 앞모서리 아래로, 가로형은 축선 왼쪽으로; k=0 틱이 축선 연장 역할)
    for k in 0..=cat_count {
        if horizontal {
            let by = fy + fh * k as f64 / cat_count.max(1) as f64;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx - ROOM_TICK_LEFT,
                by,
                fx,
                by
            ));
        } else {
            let bx = fx + fw * k as f64 / cat_count.max(1) as f64;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                bx,
                fy + fh,
                bx,
                fy + fh + ROOM_TICK_DOWN
            ));
        }
    }
    svg.push_str("</g>\n");
}

/// 3D 막대 렌더 — 시어 투영(rAngAx=1 관례) 기반 별도 경로. 축 범위
/// (vmin/vmax/vstep)는 호출측 render_bars가 플롯 rect 기준으로 확정(#1882
/// 앵커 무접촉), 여기서는 방·배치·압출만 담당. 배치는 fit 후 앞면 rect 기준 —
/// 2D 루프와 완전 분리(2D 출력 바이트 불변). (C2b #2278 v2)
#[allow(clippy::too_many_arguments)]
fn render_bars_3d(
    svg: &mut String,
    chart: &OoxmlChart,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    horizontal: bool,
    stacked: bool,
    percent: bool,
    cat_count: usize,
    ser_count: usize,
    vmin: f64,
    vmax: f64,
    vstep: f64,
) {
    let view = chart.view3d.clone().unwrap_or_default();
    // 두께 규칙 slot/(n_eff + gapWidth/100) — Excel gapWidth 의미(슬롯 내 여백을
    // 막대 폭 %로). 코퍼스 150: 누적 1/2.5=0.4, 묶은 3계열 3/4.5≈0.667 —
    // v1~v3 눈대중 상수의 유도 원형. 2D는 0.7 휴리스틱 동결.
    let gap_w = chart.bar_gap_width.unwrap_or(150.0).max(0.0);
    let n_eff = if stacked { 1.0 } else { ser_count as f64 };

    // fit 전 좌표로 깊이 산출 (fit이 깊이에 의존 — 역방향 순환 없음).
    // depthPercent = 막대 깊이/폭 % — ECMA "차트 폭 대비"와 다른 의도적 편차
    // (mod.rs View3D 주석 참조).
    let slot0 = (if horizontal { ph } else { pw }) / cat_count as f64;
    let bar_w0 = slot0 / (n_eff + gap_w / 100.0);
    let b_depth = bar_w0 * view.depth_percent.max(0.0) / 100.0;
    let d_scene = b_depth * (1.0 + chart.gap_depth.unwrap_or(150.0).max(0.0) / 100.0);

    let proj = shear_proj(&view, px, py, pw, ph, d_scene);

    // 막대 깊이 센터링: z ∈ [z0, z0+b], z0 = (D−b)/2 — gapDepth 여백을 앞뒤로
    // 분할(Excel/한컴 관례, v2 설계 리뷰). d_scene ≤ ε 퇴화(depthPercent=0)는
    // 0/0 NaN 가드.
    let (bdx0, bdy0, bdx, bdy) = if d_scene > 1e-9 {
        let z0 = (d_scene - b_depth) / 2.0 / d_scene;
        let zb = b_depth / d_scene;
        (proj.dxf * z0, proj.dyf * z0, proj.dxf * zb, proj.dyf * zb)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    render_value_grid_3d(
        svg,
        &proj,
        vmin,
        vmax,
        vstep,
        chart.series.first().and_then(|s| s.format_code.as_deref()),
        horizontal,
        percent,
        cat_count,
    );

    // 배치는 fit 후 앞면 rect 기준
    let (fx, fy, fw, fh) = (proj.fx, proj.fy, proj.fw, proj.fh);
    let cat_span = (if horizontal { fh } else { fw }) / cat_count as f64;
    let bar_w = cat_span / (n_eff + gap_w / 100.0);
    let bar_span_total = bar_w * n_eff;

    // 가로 막대는 카테고리를 아래→위로 배치 (2D와 동일 규칙)
    let cat_slot = |ci: usize| -> f64 {
        let idx = if horizontal { cat_count - 1 - ci } else { ci };
        cat_span * idx as f64
    };

    if stacked {
        // 페인트 순서(세로: 아래→위, 가로: 왼→오른쪽)가 은면 제거 담당 —
        // 시어 성분 ≥0 클램프(shear_proj)가 이 순서의 유효 전제. 순서 변경 금지.
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
            let mut acc = 0.0_f64;
            for (si, ser) in chart.series.iter().enumerate() {
                let v = ser.values.get(ci).copied().unwrap_or(0.0).max(0.0);
                let rgb = ser.color.unwrap_or_else(|| palette(si));
                let cell = (if horizontal { fy } else { fx })
                    + cat_slot(ci)
                    + (cat_span - bar_span_total) / 2.0;
                if horizontal {
                    let seg = fw * (v / denom);
                    push_bar_3d(
                        svg,
                        fx + acc + bdx0,
                        cell - bdy0,
                        seg.max(0.0),
                        bar_span_total,
                        bdx,
                        bdy,
                        rgb,
                    );
                    acc += seg;
                } else {
                    let seg = fh * (v / denom);
                    let by = fy + fh - acc - seg;
                    push_bar_3d(
                        svg,
                        cell + bdx0,
                        by - bdy0,
                        bar_span_total,
                        seg.max(0.0),
                        bdx,
                        bdy,
                        rgb,
                    );
                    acc += seg;
                }
            }
        }
    } else {
        for ci in 0..cat_count {
            for (si, ser) in chart.series.iter().enumerate() {
                let v = *ser.values.get(ci).unwrap_or(&0.0);
                let t = if vmax > vmin {
                    (v - vmin) / (vmax - vmin)
                } else {
                    0.0
                };
                let rgb = ser.color.unwrap_or_else(|| palette(si));
                if horizontal {
                    let cy = fy
                        + cat_slot(ci)
                        + (cat_span - bar_span_total) / 2.0
                        + bar_w * (ser_count - 1 - si) as f64;
                    let bw = fw * t;
                    // 3D는 gapWidth 규칙이 간격 담당 — 2D의 0.95 계수 미적용
                    push_bar_3d(svg, fx + bdx0, cy - bdy0, bw.max(0.0), bar_w, bdx, bdy, rgb);
                } else {
                    let cx =
                        fx + cat_slot(ci) + (cat_span - bar_span_total) / 2.0 + bar_w * si as f64;
                    let bh = fh * t;
                    let by = fy + fh - bh;
                    push_bar_3d(svg, cx + bdx0, by - bdy0, bar_w, bh.max(0.0), bdx, bdy, rgb);
                }
            }
        }
    }

    // 가로형 라벨은 좌측 카테고리 틱(5pt) 바깥 — 한컴 실측 간격(2D는 4.0 유지)
    render_category_labels_at(svg, chart, fx, fy, fw, fh, cat_count, horizontal, 6.0);
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
    render_category_labels_at(svg, chart, px, py, pw, ph, cat_count, horizontal, 4.0);
}

/// 카테고리 라벨 — `left_gap`: 가로형 라벨의 축선~라벨 간격(2D 4.0 불변,
/// 3D는 좌측 틱 밖 6.0). 세로형 위치는 공통.
#[allow(clippy::too_many_arguments)]
fn render_category_labels_at(
    svg: &mut String,
    chart: &OoxmlChart,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    cat_count: usize,
    horizontal: bool,
    left_gap: f64,
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
                px - left_gap, cy, xml_escape(cat)
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

/// 범례 항목 역순 여부 — 정답지 PDF 28종 전수 실측 규칙 (#2277 stage3 보고서 표, 예외 0).
///
/// 한컴은 계열이 플롯에서 **세로 방향으로 배열되는 차트**의 우측 세로 범례를 시각적
/// 상→하 순서와 일치시키기 위해 역순으로 나열한다 (C1c의 "관찰 상충"은 이 규칙):
/// - 세로 값축 누적(막대·라인 stacked/percentStacked): 스택 맨 위 = 마지막 계열
/// - 가로막대 묶음(clustered): 슬롯 맨 위 = 마지막 계열 (슬롯 배치 반전과 세트)
///
/// 3D는 2D와 동일 규칙(실측 4종 일치). pie(카테고리 범례)/scatter/stock/콤보/이중축
/// = 정순. 하단 가로 범례는 코퍼스 미실측(전 샘플 legendPos=r) — 현행 정순 유지.
fn legend_order_reversed(chart: &OoxmlChart) -> bool {
    if chart.legend_pos != LegendPos::Right || chart.is_combo() || chart.has_secondary_axis {
        return false;
    }
    match chart.chart_type {
        OoxmlChartType::Column => matches!(
            chart.grouping,
            BarGrouping::Stacked | BarGrouping::PercentStacked
        ),
        OoxmlChartType::Bar => chart.grouping == BarGrouping::Clustered,
        OoxmlChartType::Line => matches!(
            chart.line_grouping,
            BarGrouping::Stacked | BarGrouping::PercentStacked
        ),
        _ => false,
    }
}

/// 범례 스와치 형태 — 정답지 28종 전수 실측 (#2277 stage3 표). 글리프 인덱스는
/// **원 계열 인덱스**(역순 나열과 무관하게 플롯 마커·팔레트와 동일 형상/색 유지).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwatchKind {
    /// 막대/원형: 10×10 색 사각형 (현행 유지 — issue_1882 필터 문자열 보호)
    Square,
    /// 무표식 라인·콤보 라인: 14px 색 선분 (현행 유지)
    LineOnly,
    /// 표식 라인·선+표식 분산형: 선분 + 중앙 마커 글리프 (—◆—)
    LineGlyph(usize),
    /// 표식만 분산형·stock 종가: 마커 글리프만
    GlyphOnly(usize),
    /// stock 시/고/저 (`c:symbol val="none"`): 스와치 없음 — 텍스트 정렬은 유지
    Blank,
}

/// 계열별 스와치 형태 결정. 실측 근거: 표식 라인 = 선+글리프 / 분산형 = 스타일
/// flags 따라 선·글리프 조합 / stock = 종가만 글리프·나머지 빈 스와치. (C2a #2277)
fn swatch_kind(chart: &OoxmlChart, s: &OoxmlSeries, i: usize) -> SwatchKind {
    match s.series_type {
        // 순수 라인 차트 + plot 레벨 표식 → 선+글리프. 콤보의 라인 계열은
        // render_combo가 마커를 그리지 않으므로 선만 (현행 유지).
        OoxmlChartType::Line if chart.chart_type == OoxmlChartType::Line && chart.line_markers => {
            SwatchKind::LineGlyph(i)
        }
        OoxmlChartType::Line => SwatchKind::LineOnly,
        OoxmlChartType::Scatter => {
            let (line, _, marker) = chart.scatter_style.flags();
            match (line, marker) {
                (true, true) => SwatchKind::LineGlyph(i),
                (false, true) => SwatchKind::GlyphOnly(i),
                _ => SwatchKind::LineOnly,
            }
        }
        OoxmlChartType::Stock => {
            if matches!(s.marker_symbol, SeriesMarker::Auto | SeriesMarker::Named(_)) {
                SwatchKind::GlyphOnly(i)
            } else {
                SwatchKind::Blank
            }
        }
        _ => SwatchKind::Square,
    }
}

/// 범례 항목 목록 `(라벨, 색상, 스와치 형태)`. pie는 카테고리별, 그 외는 시리즈별
/// (색·글리프 매핑 후 `legend_order_reversed`면 역순 나열).
fn legend_items(chart: &OoxmlChart) -> Vec<(String, u32, SwatchKind)> {
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
                            (label, color, SwatchKind::Square)
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        _ => {
            let mut items: Vec<(String, u32, SwatchKind)> = chart
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
                    (label, color, swatch_kind(chart, s, i))
                })
                .collect();
            if legend_order_reversed(chart) {
                items.reverse();
            }
            items
        }
    }
}

/// 범례 스와치 1개. `cy` = 행 세로 중심. Square/LineOnly는 종전 출력 바이트 유지,
/// 글리프는 별도 클래스 `hwp-legend-glyph`(플롯 마커 `hwp-chart-marker` 카운트
/// 오염 방지 — issue_2129 보호). (C2a #2277)
fn push_legend_swatch(svg: &mut String, ix: f64, cy: f64, color: u32, kind: SwatchKind) {
    let swatch_line = |svg: &mut String| {
        svg.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"2\"/>\n",
            ix, cy, ix + 14.0, cy, color_hex(color)
        ));
    };
    match kind {
        SwatchKind::Square => {
            svg.push_str(&format!(
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" fill=\"{}\"/>\n",
                ix,
                cy - 6.0,
                color_hex(color)
            ));
        }
        SwatchKind::LineOnly => swatch_line(svg),
        SwatchKind::LineGlyph(si) => {
            swatch_line(svg);
            push_marker(
                svg,
                "hwp-legend-glyph",
                si,
                ix + 7.0,
                cy,
                3.0,
                &color_hex(color),
            );
        }
        SwatchKind::GlyphOnly(si) => {
            push_marker(
                svg,
                "hwp-legend-glyph",
                si,
                ix + 7.0,
                cy,
                3.5,
                &color_hex(color),
            );
        }
        SwatchKind::Blank => {}
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
    for (i, (label, color, kind)) in items.iter().enumerate() {
        let ix = start_x + item_w * i as f64;
        push_legend_swatch(svg, ix, y + 11.0, *color, *kind);
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
    for (i, (label, color, kind)) in items.iter().enumerate() {
        let cy = start_y + row_h * i as f64 + row_h / 2.0;
        push_legend_swatch(svg, x, cy, *color, *kind);
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
    fn test_line_marker_x_series4() {
        // 사이클 4번째는 × — OHLC 종가 정답지 실측 (C2a #2277, 종전 원 폴백 교체)
        let mut chart = line_chart(BarGrouping::Clustered);
        chart.line_markers = true;
        chart.series.push(OoxmlSeries {
            values: vec![1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        });
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let ds = marker_ds(&svg);
        assert_eq!(ds.len(), 16);
        let skel: String = ds[12].chars().filter(|c| c.is_ascii_alphabetic()).collect();
        assert_eq!(skel, "MLML", "계열4는 × (두 대각선 열린 경로): {}", ds[12]);
        // × 는 stroke 기반 — 채움이면 안 보임 (열린 경로)
        assert!(
            svg.contains(&format!("d=\"{}\" fill=\"none\"", ds[12])),
            "× 마커는 fill=none + stroke=계열색"
        );
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
        // marker: 점만(사이클 글리프 — C2a #2277, 종전 circle 교체), 연결선 없음.
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Marker), 0.0, 0.0, 400.0, 300.0);
        assert!(
            !svg.contains("<circle"),
            "표식은 circle이 아니라 사이클 글리프"
        );
        assert_eq!(marker_ds(&svg).len(), 3, "1계열×3점 마커");
        assert!(data_line_paths(&svg).is_empty(), "marker는 연결선 없어야");
        assert!(!svg.contains("차트 (미지원)"));
        assert!(svg.contains("hwp-ooxml-chart\""));
    }

    #[test]
    fn test_render_scatter_line_only() {
        // line: 직선만, 표식 없음.
        let svg = render_chart_svg(&scatter_chart(ScatterStyle::Line), 0.0, 0.0, 400.0, 300.0);
        assert_eq!(data_line_paths(&svg).len(), 1, "line은 연결선 있어야");
        assert!(marker_ds(&svg).is_empty(), "line은 표식 없어야");
        assert!(!svg.contains("<circle"));
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
        assert_eq!(data_line_paths(&svg).len(), 1);
        assert_eq!(marker_ds(&svg).len(), 3);
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
        assert_eq!(marker_ds(&svg).len(), 3);
        assert!(svg.contains(" C"), "smooth는 cubic Bézier(C) 곡선");
    }

    #[test]
    fn test_scatter_markers_use_cycle() {
        // scatter 마커 = 라인과 동일 계열 사이클 (정답지 실측: 계열1 ◆ / 계열2 ■ —
        // 표식만있는분산형-2022.pdf. C2a #2277)
        let mut chart = scatter_chart(ScatterStyle::Marker);
        chart.series.push(OoxmlSeries {
            name: "Y2".into(),
            x_values: vec![0.7, 1.8, 2.6],
            values: vec![1.0, 2.0, 4.0],
            series_type: OoxmlChartType::Scatter,
            ..Default::default()
        });
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let ds = marker_ds(&svg);
        assert_eq!(ds.len(), 6, "2계열×3점");
        let skel = |d: &str| {
            d.chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
        };
        // 계열1=◆, 계열2=■ (둘 다 4각형 path — 첫 세그먼트 대각/수평으로 구분)
        assert_eq!(skel(&ds[0]), "MLLLZ", "계열1 ◆");
        assert_eq!(skel(&ds[3]), "MLLLZ", "계열2 ■");
        let dia = path_points(&ds[0]);
        assert!((dia[0].1 - dia[1].1).abs() > 1e-6, "◆ 첫 세그먼트 대각");
        let sq = path_points(&ds[3]);
        assert!((sq[0].1 - sq[1].1).abs() < 1e-6, "■ 첫 세그먼트 수평");
    }

    // --- C2a (#2277): stock (주식형) 렌더 ---

    /// 코퍼스 실측 스케일 미러: 고가 max 59 → stock 전용 축 0~80 step 20.
    /// n=3: 고/저/종(HLC), n=4: 시/고/저/종(OHLC — 1월만 하락(시44>종32), 나머지 상승).
    fn stock_chart(n: usize) -> OoxmlChart {
        let ser = |name: &str, values: Vec<f64>, marker: SeriesMarker| OoxmlSeries {
            name: name.into(),
            values,
            marker_symbol: marker,
            series_type: OoxmlChartType::Stock,
            ..Default::default()
        };
        let mut series = Vec::new();
        if n == 4 {
            series.push(ser(
                "시가",
                vec![44.0, 32.0, 33.0, 34.0],
                SeriesMarker::None,
            ));
        }
        series.push(ser(
            "고가",
            vec![55.0, 57.0, 57.0, 59.0],
            SeriesMarker::None,
        ));
        series.push(ser(
            "저가",
            vec![11.0, 12.0, 13.0, 21.0],
            SeriesMarker::None,
        ));
        series.push(ser(
            "종가",
            vec![32.0, 35.0, 34.0, 35.0],
            SeriesMarker::Auto,
        ));
        OoxmlChart {
            chart_type: OoxmlChartType::Stock,
            has_hi_low_lines: true,
            has_up_down_bars: n == 4,
            up_down_gap_width: (n == 4).then_some(150.0),
            categories: vec!["1월".into(), "2월".into(), "3월".into(), "4월".into()],
            series,
            ..Default::default()
        }
    }

    #[test]
    fn test_stock_axis_unconditional_headroom() {
        // 정답지 실측: max 59 → 0~80 step 20. 경계 조건부 headroom(nice_axis)이면 0~60.
        let svg = render_chart_svg(&stock_chart(3), 0.0, 0.0, 400.0, 300.0);
        assert!(
            svg.contains(">80<"),
            "stock 전용 +1 step 헤드룸 → 축 max 80"
        );
        assert!(svg.contains(">20<"), "step 20");
        assert!(!svg.contains(">100<"), "과확장 금지");
    }

    #[test]
    fn test_stock_hilow_lines_per_category() {
        let svg = render_chart_svg(&stock_chart(3), 0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            svg.matches("hwp-stock-hilow").count(),
            4,
            "카테고리당 고저선 1"
        );
        assert_eq!(
            svg.matches("hwp-stock-candle").count(),
            0,
            "HLC는 캔들 없음"
        );
    }

    #[test]
    fn test_stock_ohlc_candles() {
        let svg = render_chart_svg(&stock_chart(4), 0.0, 0.0, 400.0, 300.0);
        let candles: Vec<&str> = svg
            .split("<rect ")
            .skip(1)
            .filter(|c| c[..c.find("/>").unwrap_or(c.len())].contains("hwp-stock-candle"))
            .collect();
        assert_eq!(candles.len(), 4, "카테고리당 캔들 1");
        let down = candles.iter().filter(|c| c.contains("#404040")).count();
        let up = candles
            .iter()
            .filter(|c| c.contains("fill=\"#ffffff\"") && c.contains("stroke=\"#000000\""))
            .count();
        assert_eq!(down, 1, "1월(시44>종32)만 하락 = 진회색 채움");
        assert_eq!(up, 3, "상승 = 흰 채움 + 검정 테두리");
    }

    #[test]
    fn test_stock_close_marker_only() {
        // 종가(Auto)만 마커 — HLC 종가는 3번째 계열(si=2 → ▲), OHLC는 4번째(si=3 → ×)
        let skel = |d: &str| {
            d.chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
        };
        let svg3 = render_chart_svg(&stock_chart(3), 0.0, 0.0, 400.0, 300.0);
        let ds3 = marker_ds(&svg3);
        assert_eq!(ds3.len(), 4, "HLC: 종가 4점만 (고/저 무마커)");
        assert_eq!(skel(&ds3[0]), "MLLZ", "HLC 종가 ▲");
        let svg4 = render_chart_svg(&stock_chart(4), 0.0, 0.0, 400.0, 300.0);
        let ds4 = marker_ds(&svg4);
        assert_eq!(ds4.len(), 4, "OHLC: 종가 4점만");
        assert_eq!(skel(&ds4[0]), "MLML", "OHLC 종가 ×");
    }

    #[test]
    fn test_stock_unusual_series_count_line_fallback() {
        // 계열 수 3/4 외 → render_line 폴백 (placeholder 재발 방지)
        let mut chart = stock_chart(3);
        chart.series.truncate(2);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(svg.matches("hwp-stock-hilow").count(), 0);
        assert!(!data_line_paths(&svg).is_empty(), "라인 폴백으로 렌더");
        assert!(!svg.contains("hwp-ooxml-chart-fallback"));
    }

    // --- C2a (#2277) stage3: 범례 순서 규칙 (정답지 28종 전수 실측 — 예외 0) ---

    /// 이름 있는 3계열 차트 (범례 순서 검증용). 우측 범례 = 코퍼스 전 샘플 legendPos=r.
    fn named3(chart_type: OoxmlChartType, grouping: BarGrouping) -> OoxmlChart {
        let ser = |i: usize| OoxmlSeries {
            name: format!("계열 {}", i + 1),
            values: vec![4.3, 2.5, 3.5, 4.5],
            series_type: chart_type,
            ..Default::default()
        };
        let mut c = OoxmlChart {
            chart_type,
            legend_pos: LegendPos::Right,
            series: vec![ser(0), ser(1), ser(2)],
            categories: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            ..Default::default()
        };
        match chart_type {
            OoxmlChartType::Line => c.line_grouping = grouping,
            _ => c.grouping = grouping,
        }
        c
    }

    fn first_legend_label(chart: &OoxmlChart) -> String {
        legend_items(chart)
            .first()
            .map(|(l, _, _)| l.clone())
            .unwrap_or_default()
    }

    #[test]
    fn test_legend_order_rule_table() {
        use BarGrouping::*;
        use OoxmlChartType::*;
        // 역순 = (세로 값축 && 누적/백프로) || (가로막대 && 묶음) — 실측 28종 예외 0
        let cases: &[(OoxmlChartType, BarGrouping, bool)] = &[
            (Column, Stacked, true),
            (Column, PercentStacked, true),
            (Column, Clustered, false),
            (Bar, Clustered, true),
            (Bar, Stacked, false),
            (Bar, PercentStacked, false),
            (Line, Stacked, true),
            (Line, PercentStacked, true),
            (Line, Clustered, false), // standard 라인
        ];
        for &(t, g, reversed) in cases {
            let expect = if reversed { "계열 3" } else { "계열 1" };
            assert_eq!(
                first_legend_label(&named3(t, g)),
                expect,
                "{:?}/{:?} → 역순={}",
                t,
                g,
                reversed
            );
        }
    }

    #[test]
    fn test_legend_order_3d_same_as_2d() {
        // 실측: 3D누적세로·3D묶은가로=역순, 3D묶은세로·3D누적가로=정순 — 2D와 동일 규칙
        let mut c = named3(OoxmlChartType::Column, BarGrouping::Stacked);
        c.is_3d = true;
        assert_eq!(first_legend_label(&c), "계열 3", "3D 누적세로 역순");
        let mut c = named3(OoxmlChartType::Bar, BarGrouping::Clustered);
        c.is_3d = true;
        assert_eq!(first_legend_label(&c), "계열 3", "3D 묶은가로 역순");
    }

    #[test]
    fn test_legend_order_forward_for_stock_and_bottom_legend() {
        // stock = 정순 (실측: 고가→저가→종가)
        let mut c = stock_chart(3);
        c.legend_pos = LegendPos::Right;
        assert_eq!(first_legend_label(&c), "고가");
        // 하단 가로 범례는 코퍼스 미실측 — 역순 규칙 미적용 (현행 정순 유지)
        let mut c2 = named3(OoxmlChartType::Column, BarGrouping::Stacked);
        c2.legend_pos = LegendPos::Bottom;
        assert_eq!(first_legend_label(&c2), "계열 1");
    }

    #[test]
    fn test_legend_order_combo_forward() {
        // 콤보(막대+라인)는 정순 고정 — 역순 규칙에서 명시 제외
        let mut c = named3(OoxmlChartType::Column, BarGrouping::Stacked);
        c.series[2].series_type = OoxmlChartType::Line;
        assert_eq!(first_legend_label(&c), "계열 1");
    }

    #[test]
    fn test_hbar_clustered_slot_series1_at_bottom() {
        // 묶은가로 실측: 슬롯 내 위→아래 = 계열3→2→1 (계열1이 맨 아래 = y 최대).
        // 범례 역순과 세트로 시각 일치 (#2277 stage3).
        let c = named3(OoxmlChartType::Bar, BarGrouping::Clustered);
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let rect_y = |color: &str| -> f64 {
            let tag = svg
                .split("<rect ")
                .skip(1)
                .map(|ch| &ch[..ch.find("/>").unwrap_or(ch.len())])
                .find(|t| t.contains(&format!("fill=\"{}\"", color)))
                .unwrap_or_else(|| panic!("{color} 막대 없음"));
            let p = tag.find("y=\"").unwrap() + 3;
            let e = tag[p..].find('"').unwrap();
            tag[p..p + e].parse().unwrap()
        };
        assert!(
            rect_y("#6183d7") > rect_y("#b0b0b0"),
            "계열1(파랑)이 슬롯 맨 아래 (y가 계열3(회색)보다 커야)"
        );
    }

    // --- C2a (#2277) stage5: 특이케이스 1카테고리 미니차트 0.5축 ---

    #[test]
    fn test_hbar_single_category_half_step() {
        // 특이케이스 실측(C1c v2 기록 → #2277 반영): 가로막대 1카테고리 미니차트는
        // 축 범위 유지·step 절반 (4.3 → 0~5 step 0.5, 라벨 11개). 단일 샘플 근거라
        // 가로·1카테고리·비누적·비3D로 좁게 게이트 — 코퍼스 나머지 27종(전부
        // 4카테고리) 무영향.
        let mut c = named3(OoxmlChartType::Bar, BarGrouping::Clustered);
        c.series.truncate(1);
        c.series[0].values = vec![4.3];
        c.categories = vec!["항목 1".into()];
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        for want in [">0.5<", ">4.5<", ">5<"] {
            assert!(svg.contains(want), "미니차트 0.5 step 라벨 {want} 없음");
        }
        // 다카테고리 무회귀 핀: step 1 유지
        let svg4 = render_chart_svg(
            &named3(OoxmlChartType::Bar, BarGrouping::Clustered),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert!(!svg4.contains(">0.5<"), "다카테고리는 step 절반 미적용");
    }

    // --- C2a (#2277) stage4: 범례 스와치 글리프 (SwatchKind) ---

    /// 범례 그룹(`hwp-chart-legend`) 조각만 잘라 반환.
    fn legend_fragment(svg: &str) -> &str {
        let start = svg
            .find("<g class=\"hwp-chart-legend\">")
            .expect("범례 그룹 없음");
        let end = svg[start..].find("</g>").expect("범례 그룹 종료") + start;
        &svg[start..end]
    }

    #[test]
    fn test_legend_swatch_marker_line_has_glyph() {
        // 실측(표식이있는꺽은선형): 스와치 = 선분 + 플롯 마커와 동일 글리프 (—◆—)
        let mut c = named3(OoxmlChartType::Line, BarGrouping::Clustered);
        c.line_markers = true;
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let legend = legend_fragment(&svg);
        assert_eq!(
            legend.matches("hwp-legend-glyph").count(),
            3,
            "계열별 글리프 1개"
        );
        assert_eq!(
            legend.matches("stroke-width=\"2\"").count(),
            3,
            "선분 스와치 유지"
        );
        // 플롯 마커 카운트 무오염 (issue_2129 보호 — 별도 클래스)
        assert_eq!(
            svg.matches("hwp-chart-marker").count(),
            12,
            "플롯 마커 12개 불변"
        );
    }

    #[test]
    fn test_legend_swatch_plain_line_no_glyph() {
        // 실측(꺽은선형): 무표식 라인 스와치 = 선분만 (글리프 없음, 현행 유지)
        let c = named3(OoxmlChartType::Line, BarGrouping::Clustered);
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let legend = legend_fragment(&svg);
        assert_eq!(legend.matches("hwp-legend-glyph").count(), 0);
        assert_eq!(legend.matches("stroke-width=\"2\"").count(), 3);
    }

    #[test]
    fn test_legend_swatch_scatter_marker_only_glyph_only() {
        // 실측(표식만있는분산형): 스와치 = 마커 글리프만 (선분 없음)
        let mut c = scatter_chart(ScatterStyle::Marker);
        c.legend_pos = LegendPos::Right;
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let legend = legend_fragment(&svg);
        assert_eq!(
            legend.matches("hwp-legend-glyph").count(),
            1,
            "1계열 글리프"
        );
        assert_eq!(
            legend.matches("stroke-width=\"2\"").count(),
            0,
            "표식만은 선분 스와치 없음"
        );
    }

    #[test]
    fn test_legend_swatch_scatter_line_marker_line_glyph() {
        // 실측(직선및표식/곡선및표식): 스와치 = 선분 + 글리프
        let mut c = scatter_chart(ScatterStyle::LineMarker);
        c.legend_pos = LegendPos::Right;
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let legend = legend_fragment(&svg);
        assert_eq!(legend.matches("hwp-legend-glyph").count(), 1);
        assert_eq!(
            legend.matches("stroke-width=\"2\"").count(),
            1,
            "선분 스와치 동반"
        );
    }

    #[test]
    fn test_legend_swatch_stock_blank_except_close() {
        // 실측(stock 2종): 시/고/저 스와치 없음(라벨 정렬 유지), 종가만 글리프
        let mut c = stock_chart(4);
        c.legend_pos = LegendPos::Right;
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let legend = legend_fragment(&svg);
        assert_eq!(
            legend.matches("hwp-legend-glyph").count(),
            1,
            "종가(Auto)만 글리프"
        );
        assert_eq!(
            legend.matches("<rect ").count(),
            0,
            "stock 범례에 색 사각형 스와치 없음"
        );
        assert_eq!(
            legend.matches("stroke-width=\"2\"").count(),
            0,
            "stock 범례에 선분 스와치 없음"
        );
        for name in ["시가", "고가", "저가", "종가"] {
            assert!(
                legend.contains(&format!(">{name}</text>")),
                "{name} 라벨 유지"
            );
        }
    }

    #[test]
    fn test_legend_swatch_square_unchanged_for_bars() {
        // issue_1882 보호: 막대 범례 스와치 = 10×10 색 사각형 문자열 불변
        let c = named3(OoxmlChartType::Column, BarGrouping::Clustered);
        let svg = render_chart_svg(&c, 0.0, 0.0, 400.0, 300.0);
        let legend = legend_fragment(&svg);
        assert_eq!(legend.matches("width=\"10\" height=\"10\"").count(), 3);
        assert_eq!(legend.matches("hwp-legend-glyph").count(), 0);
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

    // --- C2b (#2278) Stage 1: 3D 막대 압출 ---

    fn bars3d_chart(chart_type: OoxmlChartType, grouping: BarGrouping) -> OoxmlChart {
        let mut chart = bars_chart(grouping);
        chart.chart_type = chart_type;
        chart.is_3d = true;
        chart
    }

    #[test]
    fn test_shade_lighten_darken() {
        // 채널별 선형 보간: +0.25 = 흰색 방향 25%, -0.25 = 검정 방향 25%
        assert_eq!(shade(0x006183D7, 0.25), 0x0089A2E1);
        assert_eq!(shade(0x006183D7, -0.25), 0x004962A1);
        // 극단값 클램프
        assert_eq!(shade(0x00123456, 1.0), 0x00FFFFFF);
        assert_eq!(shade(0x00123456, -1.0), 0x00000000);
        // factor 0 항등 + 상위(알파) 바이트 보존
        assert_eq!(shade(0xFF6183D7, 0.0), 0xFF6183D7);
        assert_eq!(shade(0xFF6183D7, 0.25) >> 24, 0xFF);
    }

    #[test]
    fn test_bar3d_clustered_faces_both_orientations() {
        // 3D 묶은: 막대(2cat×3ser=6)마다 top/side 면 1쌍 (정답지: 윗면 밝게 +
        // 우측면 어둡게 사선 압출). 2D는 면 없음.
        for chart_type in [OoxmlChartType::Column, OoxmlChartType::Bar] {
            let chart = bars3d_chart(chart_type, BarGrouping::Clustered);
            let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
            assert_eq!(
                svg.matches("hwp-bar3d-top").count(),
                6,
                "{chart_type:?}: top 면 6개"
            );
            assert_eq!(
                svg.matches("hwp-bar3d-side").count(),
                6,
                "{chart_type:?}: side 면 6개"
            );
        }
        let svg2d = render_chart_svg(&bars_chart(BarGrouping::Clustered), 0.0, 0.0, 400.0, 300.0);
        assert!(!svg2d.contains("hwp-bar3d-"), "2D 묶은막대에 3D 면 없어야");
    }

    #[test]
    fn test_bar3d_stacked_all_segments_extrude() {
        // 3D 누적: 모든 세그먼트가 자기 색 top/side를 그림 (2cat×3ser=6쌍).
        // 은면 제거는 페인트 순서(세로: 아래→위 = 계열1 먼저)가 담당 — 순서 핀.
        let chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Stacked);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(svg.matches("hwp-bar3d-top").count(), 6, "top 면 6개");
        assert_eq!(svg.matches("hwp-bar3d-side").count(), 6, "side 면 6개");
        let s1 = color_hex(shade(palette(0), BAR3D_SIDE_SHADE));
        let s3 = color_hex(shade(palette(2), BAR3D_SIDE_SHADE));
        assert!(
            svg.find(&s1).expect("계열1 side 색") < svg.find(&s3).expect("계열3 side 색"),
            "누적 페인트 순서: 계열1(아래) 먼저 → 계열3(위) 나중"
        );
    }

    #[test]
    fn test_bar3d_zero_segment_skipped() {
        // 0값 세그먼트는 면 무방출 — 이웃 세그먼트의 캡(top) 재도색 방지.
        let mut chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Stacked);
        chart.series[1].values = vec![0.0, 1.0]; // 카테고리 a의 계열2 = 0
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            svg.matches("hwp-bar3d-top").count(),
            5,
            "0값 세그먼트는 스킵 (6-1)"
        );
    }

    // --- C2b (#2278) v2: 투영 기하 헬퍼 ---

    /// room 그룹 조각 (첫 </g>까지)
    fn room_slice(svg: &str) -> &str {
        let s = svg.find("hwp-bar3d-room").expect("room");
        let room = &svg[s..];
        &room[..room.find("</g>").expect("room 닫힘")]
    }

    /// polygon points="..." → (x,y) 목록
    fn poly_points(chunk: &str) -> Vec<(f64, f64)> {
        let pts = &chunk[chunk.find("points=\"").expect("points") + 8..];
        let pts = &pts[..pts.find('"').expect("닫는 따옴표")];
        pts.split_whitespace()
            .map(|p| {
                let mut it = p.split(',');
                (
                    it.next().unwrap().parse().unwrap(),
                    it.next().unwrap().parse().unwrap(),
                )
            })
            .collect()
    }

    /// 방 바닥 폴리곤(room 첫 polygon) 4점: p1=(fx,fyb) p2=(fx+dxf,fyb−dyf)
    /// p3=(fx+fw+dxf,·) p4=(fx+fw,fyb) — 씬 파라미터를 SVG에서 역산하는 기준
    fn floor_points(svg: &str) -> Vec<(f64, f64)> {
        let room = room_slice(svg);
        let start = room.find("<polygon").expect("바닥 폴리곤");
        poly_points(&room[start..])
    }

    /// 첫 top 면 폴리곤 → 막대 압출 벡터 (bdx, bdy)
    fn bar_extrusion(svg: &str) -> (f64, f64) {
        let chunk = svg.split("hwp-bar3d-top").nth(1).expect("top 폴리곤");
        let pts = poly_points(chunk);
        (pts[1].0 - pts[0].0, pts[0].1 - pts[1].1)
    }

    /// 파랑(계열1) front rect (x, y, w, h) 목록 — 범례 swatch(10×10) 제외
    fn blue_fronts(svg: &str) -> Vec<(f64, f64, f64, f64)> {
        let parts: Vec<&str> = svg.split("fill=\"#6183d7\"").collect();
        parts[..parts.len() - 1]
            .iter()
            .filter_map(|chunk| {
                let tag = &chunk[chunk.rfind("<rect ")?..];
                if tag.contains("width=\"10\" height=\"10\"") {
                    return None;
                }
                Some((
                    attr_f64_of(tag, "x=\"")?,
                    attr_f64_of(tag, "y=\"")?,
                    attr_f64_of(tag, "width=\"")?,
                    attr_f64_of(tag, "height=\"")?,
                ))
            })
            .collect()
    }

    #[test]
    fn test_bar3d_shear_direction() {
        // 시어 방향: fit 역산으로 pre-fit 성분비 oy/ox == sin(rotX)/sin(rotY)
        // (기본 카메라 15/20). 비등방 fit(sx≠sy) 때문에 화면 비율은 순수 sin비가
        // 아님 — pw=fw+dxf, ph=fh+dyf 복원으로 역산 (v2 설계 리뷰 반영).
        let chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let fl = floor_points(&svg);
        let dxf = fl[1].0 - fl[0].0;
        let dyf = fl[0].1 - fl[1].1;
        let fw = fl[3].0 - fl[0].0;
        // 뒷벽 rect height = fh
        let room = room_slice(&svg);
        let wall = &room[room.find("<rect").expect("뒷벽")..];
        let fh = attr_f64_of(wall, "height=\"").expect("fh");
        let ox = dxf * (fw + dxf) / fw;
        let oy = dyf * (fh + dyf) / fh;
        let expected = 15.0_f64.to_radians().sin() / 20.0_f64.to_radians().sin();
        assert!(
            (oy / ox - expected).abs() < 2e-3,
            "pre-fit 시어 성분비 sin15/sin20={expected}, 실제 {}",
            oy / ox
        );
        // 막대 압출은 방 깊이 벡터와 평행
        let (bdx, bdy) = bar_extrusion(&svg);
        assert!(
            (bdy / bdx - dyf / dxf).abs() < 2e-3,
            "막대 압출({},{})이 방 깊이({dxf},{dyf})와 평행해야",
            bdx,
            bdy
        );
    }

    #[test]
    fn test_bar3d_room_depth_ratio() {
        // 방 깊이 / 막대 깊이 = 1 + gapDepth/100 (기본 150 → 2.5) — 센터링과
        // 무관하게 성립(dxf/bdx = D/b).
        let chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let fl = floor_points(&svg);
        let dxf = fl[1].0 - fl[0].0;
        let (bdx, _) = bar_extrusion(&svg);
        assert!(
            (dxf / bdx - 2.5).abs() < 1e-2,
            "기본 gapDepth 150 → D/b = 2.5, 실제 {}",
            dxf / bdx
        );
        // gapDepth=300 → 4.0
        let mut chart2 = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
        chart2.gap_depth = Some(300.0);
        let svg2 = render_chart_svg(&chart2, 0.0, 0.0, 400.0, 300.0);
        let fl2 = floor_points(&svg2);
        let dxf2 = fl2[1].0 - fl2[0].0;
        let (bdx2, _) = bar_extrusion(&svg2);
        assert!(
            (dxf2 / bdx2 - 4.0).abs() < 1e-2,
            "gapDepth 300 → 4.0, 실제 {}",
            dxf2 / bdx2
        );
    }

    #[test]
    fn test_bar3d_thickness_from_gap_width() {
        // 두께 규칙 slot/(n_eff+gapWidth/100) — v3 눈대중 상수(누적 0.4)의 유도
        // 원형. 기본 150: 누적 1/2.5=0.4, 묶은 3계열 bar_w = slot/4.5.
        let cat_span_of = |fronts: &[(f64, f64, f64, f64)]| fronts[1].0 - fronts[0].0;

        let stacked3d = bars3d_chart(OoxmlChartType::Column, BarGrouping::Stacked);
        let svg = render_chart_svg(&stacked3d, 0.0, 0.0, 400.0, 300.0);
        let f = blue_fronts(&svg);
        assert_eq!(f.len(), 2, "2 카테고리 파랑 front");
        let ratio = f[0].2 / cat_span_of(&f);
        assert!(
            (ratio - 0.4).abs() < 1e-3,
            "누적 두께/슬롯 0.4, 실제 {ratio}"
        );

        let clustered3d = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
        let svg_c = render_chart_svg(&clustered3d, 0.0, 0.0, 400.0, 300.0);
        let fc = blue_fronts(&svg_c);
        let ratio_c = fc[0].2 / cat_span_of(&fc);
        assert!(
            (ratio_c - 1.0 / 4.5).abs() < 1e-3,
            "묶은 3계열 bar_w/슬롯 = 1/4.5, 실제 {ratio_c}"
        );

        // gapWidth=300 누적 → 1/4 = 0.25
        let mut wide_gap = bars3d_chart(OoxmlChartType::Column, BarGrouping::Stacked);
        wide_gap.bar_gap_width = Some(300.0);
        let svg_g = render_chart_svg(&wide_gap, 0.0, 0.0, 400.0, 300.0);
        let fg = blue_fronts(&svg_g);
        let ratio_g = fg[0].2 / cat_span_of(&fg);
        assert!(
            (ratio_g - 0.25).abs() < 1e-3,
            "gapWidth 300 누적 → 0.25, 실제 {ratio_g}"
        );

        // 2D 대조군: 0.7 유지 (바이트 불변 가드)
        let svg2d = render_chart_svg(&bars_chart(BarGrouping::Stacked), 0.0, 0.0, 400.0, 300.0);
        let f2 = blue_fronts(&svg2d);
        let ratio2 = f2[0].2 / cat_span_of(&f2);
        assert!(
            (ratio2 - 0.7).abs() < 1e-3,
            "2D 누적 0.7 유지, 실제 {ratio2}"
        );
    }

    #[test]
    fn test_bar3d_bars_depth_centered() {
        // 막대 깊이 센터링: 세로 막대 하단 y = 앞면 하단(fyb) − bdy0,
        // bdy0 = (dyf − bdy)/2 (z0 = (D−b)/2).
        let chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let fl = floor_points(&svg);
        let fyb = fl[0].1;
        let dyf = fl[0].1 - fl[1].1;
        let (_, bdy) = bar_extrusion(&svg);
        let f = blue_fronts(&svg);
        let bottom = f[0].1 + f[0].3;
        let expected_off = (dyf - bdy) / 2.0;
        assert!(
            ((fyb - bottom) - expected_off).abs() < 2e-2,
            "센터링 오프셋 (dyf−bdy)/2 = {expected_off}, 실제 {}",
            fyb - bottom
        );
    }

    #[test]
    fn test_bar3d_faces_within_plot() {
        // fit 스모크: 모든 3D 면 좌표가 차트 bbox(0..400, 0..300) 안.
        for grouping in [BarGrouping::Clustered, BarGrouping::Stacked] {
            for chart_type in [OoxmlChartType::Column, OoxmlChartType::Bar] {
                let chart = bars3d_chart(chart_type, grouping);
                let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
                for class in ["hwp-bar3d-top", "hwp-bar3d-side"] {
                    for chunk in svg.split(class).skip(1) {
                        for (x, y) in poly_points(chunk) {
                            assert!(
                                (-0.5..=400.5).contains(&x) && (-0.5..=300.5).contains(&y),
                                "{chart_type:?}/{grouping:?}: 면 좌표({x},{y}) 이탈"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_bar3d_degenerate_cameras() {
        // 퇴화·경계 카메라 무패닉 + NaN 무방출 + front 존재.
        // 음수/역방향 성분은 shear_proj 클램프(정의역 방어)로 0 처리.
        let cases: &[(f64, f64, f64)] = &[
            (0.0, 0.0, 100.0),    // 시어 없음 → front만
            (90.0, 20.0, 100.0),  // rotX 최대
            (-15.0, 20.0, 100.0), // rotX<0 → 수직 성분 클램프
            (15.0, 200.0, 100.0), // sin(rotY)<0 → 수평 성분 클램프
            (15.0, 20.0, 0.0),    // depthPercent=0 → d_scene=0 NaN 가드
        ];
        for &(rx, ry, dp) in cases {
            let mut chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
            chart.view3d = Some(View3D {
                rot_x: rx,
                rot_y: ry,
                depth_percent: dp,
                ..Default::default()
            });
            let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
            assert!(!svg.contains("NaN"), "({rx},{ry},{dp}): NaN 방출");
            assert!(
                svg.contains("fill=\"#6183d7\""),
                "({rx},{ry},{dp}): front 미방출"
            );
        }
    }

    #[test]
    fn test_bar3d_room_only_when_3d() {
        // 3D 방(뒷벽+바닥+커넥터)은 is_3d 막대에만 1회 — 2D는 부재 (시각판정 보정
        // 2026-07-16: 방 표현 추가, 정답지 4종 공통).
        for chart_type in [OoxmlChartType::Column, OoxmlChartType::Bar] {
            for grouping in [BarGrouping::Clustered, BarGrouping::Stacked] {
                let chart = bars3d_chart(chart_type, grouping);
                let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
                assert_eq!(
                    svg.matches("hwp-bar3d-room").count(),
                    1,
                    "{chart_type:?}/{grouping:?}: 방 1회"
                );
                // 방 그룹 안: 바닥 평행사변형(첫 polygon) + 커넥터/뒷벽 격자 라인
                let room = &svg[svg.find("hwp-bar3d-room").unwrap()..];
                let room = &room[..room.find("</g>").expect("방 그룹 닫힘")];
                assert!(room.contains("<polygon"), "바닥 평행사변형");
                assert!(room.matches("<line").count() >= 5, "커넥터+뒷벽 격자");
            }
        }
        let svg2d = render_chart_svg(&bars_chart(BarGrouping::Clustered), 0.0, 0.0, 400.0, 300.0);
        assert!(!svg2d.contains("hwp-bar3d-room"), "2D에 방 없음");
    }

    #[test]
    fn test_bar3d_room_grid_on_back_wall() {
        // 뒷벽 격자선은 (+d,-d) 오프셋 — 세로 차트의 격자 y가 앞면 눈금보다 d만큼
        // 위(작음). 라벨 텍스트/위치는 2D와 동일(#1882 — test_axis_3d_*가 문자열 핀).
        let chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Clustered);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let room = &svg[svg.find("hwp-bar3d-room").unwrap()..];
        let room = &room[..room.find("</g>").unwrap()];
        // 뒷벽 수평 격자(x1≠x2, y1==y2)의 x1 = px+d — 앞면(px)보다 오른쪽
        let grid_line = room
            .split("<line ")
            .skip(1)
            .find(|l| {
                let y1 = attr_f64_of(l, "y1=\"");
                let y2 = attr_f64_of(l, "y2=\"");
                let x1 = attr_f64_of(l, "x1=\"");
                let x2 = attr_f64_of(l, "x2=\"");
                y1.is_some() && y1 == y2 && x1 != x2
            })
            .expect("뒷벽 수평 격자선");
        let gx1 = attr_f64_of(grid_line, "x1=\"").unwrap();
        // 방 뒷벽 rect의 x와 격자 x1 일치 (= px+d)
        let wall_x = attr_f64_of(&room[room.find("<rect").expect("뒷벽")..], "x=\"").unwrap();
        assert!(
            (gx1 - wall_x).abs() < 1e-6,
            "뒷벽 격자 x1({gx1}) == 뒷벽 x({wall_x})"
        );
    }

    // --- Stage 1R v2: 방 선 처리 한컴 정합 (시각판정 피드백 2026-07-19) ---

    /// room 내 `<line>`들의 (x1,y1,x2,y2) 목록
    fn room_lines(room: &str) -> Vec<(f64, f64, f64, f64)> {
        room.split("<line ")
            .skip(1)
            .map(|l| {
                (
                    attr_f64_of(l, "x1=\"").unwrap(),
                    attr_f64_of(l, "y1=\"").unwrap(),
                    attr_f64_of(l, "x2=\"").unwrap(),
                    attr_f64_of(l, "y2=\"").unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn test_bar3d_room_hancom_line_style() {
        // 정답지 임베드(2702px) 픽셀 실측: 축선·조그·격자·틱 전부 #808080 균일
        // (실측 gray 126~148)·0.72pt≈0.75 — 2D의 축/격자 명암 구분과 다름.
        // 뒷벽 테두리·바닥 채움 없음(흰 면 + #808080 외곽선만).
        for chart_type in [OoxmlChartType::Column, OoxmlChartType::Bar] {
            let chart = bars3d_chart(chart_type, BarGrouping::Stacked);
            let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
            let room = room_slice(&svg);
            for stale in ["#cccccc", "#e8e8e8", "#f2f2f2"] {
                assert!(
                    !room.contains(stale),
                    "{chart_type:?}: 연회색 어휘 잔존 {stale}"
                );
            }
            for l in room.split("<line ").skip(1) {
                let l = &l[..l.find("/>").expect("line 닫힘")];
                assert!(
                    l.contains("stroke=\"#808080\"") && l.contains("stroke-width=\"0.75\""),
                    "{chart_type:?}: 균일 선 스타일 아님: {l}"
                );
            }
            let wall = &room[room.find("<rect").expect("뒷벽")..];
            let wall = &wall[..wall.find("/>").unwrap()];
            assert!(!wall.contains("stroke"), "{chart_type:?}: 뒷벽 무테두리");
            let floor = &room[room.find("<polygon").expect("바닥")..];
            let floor = &floor[..floor.find("/>").unwrap()];
            assert!(
                floor.contains("fill=\"#ffffff\"") && floor.contains("stroke=\"#808080\""),
                "{chart_type:?}: 바닥 흰 면 + #808080 외곽선"
            );
        }
    }

    #[test]
    fn test_bar3d_axis_ticks_vertical() {
        // 세로형: 값 눈금마다 좌측 틱(fx−5→fx, 길이 실측 44px≈5.3pt) + 카테고리
        // 경계 하단 틱(fyb→fyb+4, 실측 31px≈3.7pt) — 경계+양끝 = cat_count+1개.
        let chart = bars3d_chart(OoxmlChartType::Column, BarGrouping::Stacked);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let room = room_slice(&svg);
        let fl = floor_points(&svg);
        let (fx, fyb) = fl[0];
        let lines = room_lines(room);
        let left_ticks = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| {
                (y1 - y2).abs() < 1e-6 && (x2 - fx).abs() < 1e-6 && (fx - x1 - 5.0).abs() < 1e-6
            })
            .count();
        let back_grids = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| (y1 - y2).abs() < 1e-6 && (x2 - x1) > 10.0)
            .count();
        assert!(back_grids >= 2, "뒷벽 수평 격자 존재");
        assert_eq!(left_ticks, back_grids, "값 눈금마다 좌측 틱");
        let down_ticks = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| {
                (x1 - x2).abs() < 1e-6 && (y1 - fyb).abs() < 1e-6 && (y2 - fyb - 4.0).abs() < 1e-6
            })
            .count();
        assert_eq!(down_ticks, 3, "카테고리 경계 하단 틱 = cat_count(2)+1");
    }

    #[test]
    fn test_bar3d_axis_ticks_horizontal() {
        // 가로형: 값 눈금마다 하단 틱 + 카테고리 경계 좌측 틱(cat_count+1) —
        // 한컴 실측(누적가로: 하단 값틱 8개 등간격 225px, 좌측 경계틱 5개).
        let chart = bars3d_chart(OoxmlChartType::Bar, BarGrouping::Stacked);
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        let room = room_slice(&svg);
        let fl = floor_points(&svg);
        let (fx, fyb) = fl[0];
        let lines = room_lines(room);
        let down_ticks = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| {
                (x1 - x2).abs() < 1e-6 && (y1 - fyb).abs() < 1e-6 && (y2 - fyb - 4.0).abs() < 1e-6
            })
            .count();
        // 뒷벽 세로 격자(x1==x2, 앞면 축선(x==fx)보다 오른쪽, 길이 > 틱)
        let back_grids = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| {
                (x1 - x2).abs() < 1e-6 && *x1 > fx + 1e-6 && (y1 - y2).abs() > 10.0
            })
            .count();
        assert!(back_grids >= 2, "뒷벽 세로 격자 존재");
        assert_eq!(down_ticks, back_grids, "값 눈금마다 하단 틱");
        let left_ticks = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| {
                (y1 - y2).abs() < 1e-6 && (x2 - fx).abs() < 1e-6 && (fx - x1 - 5.0).abs() < 1e-6
            })
            .count();
        assert_eq!(left_ticks, 3, "카테고리 경계 좌측 틱 = cat_count(2)+1");
    }

    // --- C2b (#2278) Stage 2: 3D 원형 (rAngAx=0 회전+원근) ---

    fn pie3d_chart(values: Vec<f64>, rot_x: f64, perspective: f64) -> OoxmlChart {
        OoxmlChart {
            chart_type: OoxmlChartType::Pie,
            is_3d: true,
            view3d: Some(View3D {
                rot_x,
                rot_y: 0.0,
                perspective,
                r_ang_ax: false,
                ..View3D::default()
            }),
            series: vec![OoxmlSeries {
                values,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// 첫 top 타원호의 (cx, cy, rx, ry) — "M{cx},{cy} … A{rx},{ry}" 파싱
    fn pie3d_top_geom(svg: &str) -> (f64, f64, f64, f64) {
        let chunk = svg.split("hwp-pie3d-top").nth(1).expect("top 경로");
        let d = &chunk[chunk.find("d=\"M").expect("M") + 4..];
        let (cx, rest) = d.split_once(',').unwrap();
        let (cy, _) = rest.split_once(' ').unwrap();
        let a = &chunk[chunk.find(" A").expect("타원호") + 2..];
        let (rx, rest) = a.split_once(',').unwrap();
        let (ry, _) = rest.split_once(' ').unwrap();
        (
            cx.parse().unwrap(),
            cy.parse().unwrap(),
            rx.parse().unwrap(),
            ry.parse().unwrap(),
        )
    }

    /// 벽 경로의 좌표쌍 목록 — 순서: M점, A반지름, 호끝, L점, A반지름, 호끝
    /// (인덱스 0=시작점, 2=1차 호 끝, 3=벽 하단, 5=복귀 호 끝)
    fn wall_pairs(chunk: &str) -> Vec<(f64, f64)> {
        let d = &chunk[chunk.find("d=\"").unwrap() + 3..];
        let d = &d[..d.find('"').unwrap()];
        d.split_whitespace()
            .filter_map(|t| {
                let t = t.trim_start_matches(['M', 'A', 'L', 'Z']);
                let (x, y) = t.split_once(',')?;
                Some((x.parse().ok()?, y.parse().ok()?))
            })
            .collect()
    }

    #[test]
    fn test_pie3d_ellipse_ratio_follows_rotx() {
        // 타원비 = sin(rotX)·cos(perspective/2°) — 정답지(rotX=30/persp=30) 실측
        // ry/rx=0.480, 유도 0.483 (0.5% 이내). 앞뒤 반타원 대칭(원근 비대칭 부재).
        let svg = render_chart_svg(
            &pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        let (_, _, rx, ry) = pie3d_top_geom(&svg);
        let expected = 30f64.to_radians().sin() * 15f64.to_radians().cos();
        assert!(
            (ry / rx - expected).abs() < 2e-3,
            "rotX=30/persp=30 → ry/rx≈{expected:.4}, 실제 {}",
            ry / rx
        );
        let svg = render_chart_svg(
            &pie3d_chart(vec![25.0, 25.0, 50.0], 60.0, 30.0),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        let (_, _, rx, ry) = pie3d_top_geom(&svg);
        let expected = 60f64.to_radians().sin() * 15f64.to_radians().cos();
        assert!(
            (ry / rx - expected).abs() < 2e-3,
            "rotX=60 → ry/rx≈{expected:.4}, 실제 {}",
            ry / rx
        );
    }

    #[test]
    fn test_pie3d_wall_height_measured() {
        // 측벽 높이 = rx × 0.207 × hPercent/100 — 정답지 실측 175px/846.5px
        let svg = render_chart_svg(
            &pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        let (_, _, rx, _) = pie3d_top_geom(&svg);
        let wall = svg.split("hwp-pie3d-wall").nth(1).expect("벽");
        let pts = wall_pairs(wall);
        // 복귀 호 끝(xa, ya+wall) − 시작점(xa, ya)
        let wall_h = pts[5].1 - pts[0].1;
        assert!(
            (wall_h / rx - 0.207).abs() < 5e-3,
            "벽 높이/rx ≈ 0.207, 실제 {}",
            wall_h / rx
        );
    }

    #[test]
    fn test_pie3d_wall_lower_half_only() {
        // 하반부(θ∈(0,π))만 벽: [25,25,50] → 슬라이스1(우상) 벽 없음, 2·3만 —
        // 벽 색 = shade(팔레트, SIDE) (윗면은 원색)
        let svg = render_chart_svg(
            &pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert_eq!(svg.matches("hwp-pie3d-wall").count(), 2, "벽 2개");
        assert_eq!(svg.matches("hwp-pie3d-top").count(), 3, "top 3개");
        let w1 = svg.split("hwp-pie3d-wall").nth(1).unwrap();
        let w2 = svg.split("hwp-pie3d-wall").nth(2).unwrap();
        assert!(
            w1.contains(&color_hex(shade(palette(1), BAR3D_SIDE_SHADE))),
            "벽1 = 팔레트1 음영"
        );
        assert!(
            w2.contains(&color_hex(shade(palette(2), BAR3D_SIDE_SHADE))),
            "벽2 = 팔레트2 음영"
        );
    }

    #[test]
    fn test_pie3d_wall_clipped_at_boundaries() {
        // 첫 벽 시작 = θ=0 클립(cx+rx, cy), 마지막 벽 호 끝 = θ=π 클립(cx−rx, cy)
        let svg = render_chart_svg(
            &pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        let (cx, cy, rx, _) = pie3d_top_geom(&svg);
        let w1 = wall_pairs(svg.split("hwp-pie3d-wall").nth(1).unwrap());
        assert!(
            (w1[0].0 - (cx + rx)).abs() < 0.05 && (w1[0].1 - cy).abs() < 0.05,
            "첫 벽 시작 (cx+rx, cy), 실제 {:?}",
            w1[0]
        );
        let w2 = wall_pairs(svg.split("hwp-pie3d-wall").nth(2).unwrap());
        assert!(
            (w2[2].0 - (cx - rx)).abs() < 0.05 && (w2[2].1 - cy).abs() < 0.05,
            "마지막 벽 호 끝 (cx−rx, cy), 실제 {:?}",
            w2[2]
        );
    }

    #[test]
    fn test_pie3d_walls_before_tops() {
        // 페인트 순서: 벽 전체 → top 전체 (은면 제거)
        let svg = render_chart_svg(
            &pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert!(
            svg.rfind("hwp-pie3d-wall").unwrap() < svg.find("hwp-pie3d-top").unwrap(),
            "벽이 top보다 선행"
        );
    }

    #[test]
    fn test_pie_2d_no_pie3d_vocab() {
        // 2D 원형 가드: is_3d=false → 3D 어휘 부재 (2D 바이트 불변의 방증)
        let mut chart = pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0);
        chart.is_3d = false;
        chart.view3d = None;
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(!svg.contains("hwp-pie3d"), "2D에 3D 어휘 없음");
    }

    // --- C2b (#2278) Stage 3: ofPie 보조플롯 + 팔레트 #5 ---

    fn ofpie_chart(of: OfPieInfo) -> OoxmlChart {
        OoxmlChart {
            chart_type: OoxmlChartType::Pie,
            of_pie: Some(of),
            series: vec![OoxmlSeries {
                values: vec![10.0, 3.5, 1.5, 1.2],
                ..Default::default()
            }],
            categories: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            ..Default::default()
        }
    }

    #[test]
    fn test_palette_index4_measured() {
        // [4] = ofPie 결합 슬라이스 실측 초록계 #27A172 (원형대원형·원형대가로막대형
        // 정답지 임베드 픽셀 히스토그램 최빈값 — 두 파일 교차 일치)
        assert_eq!(palette(4), 0xFF27A172, "팔레트 [4] 실측 고정");
    }

    #[test]
    fn test_ofpie_pie_secondary_and_serlines() {
        // 주 원 3(= n−k+1 = 4−2+1) + 보조 원 2(= k) + serLines 2
        let svg = render_chart_svg(
            &ofpie_chart(OfPieInfo {
                has_ser_lines: true,
                ..Default::default()
            }),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert_eq!(svg.matches("hwp-ofpie-main").count(), 3, "주 원 슬라이스 3");
        assert_eq!(
            svg.matches("hwp-ofpie-second").count(),
            2,
            "보조 원 슬라이스 2"
        );
        assert_eq!(svg.matches("hwp-ofpie-serline").count(), 2, "serLines 2");
        // has_ser_lines=false → serline 0
        let svg2 = render_chart_svg(&ofpie_chart(OfPieInfo::default()), 0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            svg2.matches("hwp-ofpie-serline").count(),
            0,
            "serLines 부재"
        );
    }

    #[test]
    fn test_ofpie_combined_slice_uses_palette4() {
        // 결합 슬라이스 = palette(n) (n=4 → 실측 초록계) — hex 하드코딩 대신 참조
        let svg = render_chart_svg(&ofpie_chart(OfPieInfo::default()), 0.0, 0.0, 400.0, 300.0);
        let main = svg.split("hwp-ofpie-main").nth(3).expect("결합 슬라이스");
        let main = &main[..main.find("/>").unwrap()];
        assert!(
            main.contains(&color_hex(palette(4))),
            "결합 슬라이스 fill = palette(4)"
        );
    }

    #[test]
    fn test_ofpie_bar_secondary_first_split_cat_on_top() {
        // Bar형 보조: rect 2개, 첫 분할 카테고리(palette(2))가 맨 위
        let svg = render_chart_svg(
            &ofpie_chart(OfPieInfo {
                of_pie_type: OfPieType::Bar,
                ..Default::default()
            }),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        let rects: Vec<&str> = svg.split("hwp-ofpie-second").skip(1).collect();
        assert_eq!(rects.len(), 2, "보조 rect 2");
        let y_of = |chunk: &str| attr_f64_of(&chunk[..chunk.find("/>").unwrap()], "y=\"").unwrap();
        let c_of =
            |chunk: &str, rgb: u32| chunk[..chunk.find("/>").unwrap()].contains(&color_hex(rgb));
        assert!(
            c_of(rects[0], palette(2)) && c_of(rects[1], palette(3)),
            "보조 색 [2],[3]"
        );
        assert!(y_of(rects[0]) < y_of(rects[1]), "첫 분할 카테고리가 맨 위");
    }

    #[test]
    fn test_ofpie_split_pos_respected() {
        // split_pos=3 → 주 원 2(= 4−3+1) + 보조 3
        let svg = render_chart_svg(
            &ofpie_chart(OfPieInfo {
                split_pos: Some(3.0),
                ..Default::default()
            }),
            0.0,
            0.0,
            400.0,
            300.0,
        );
        assert_eq!(svg.matches("hwp-ofpie-main").count(), 2, "주 원 2");
        assert_eq!(svg.matches("hwp-ofpie-second").count(), 3, "보조 3");
    }

    #[test]
    fn test_ofpie_legend_categories_in_order_no_combined() {
        // 범례: 카테고리 4개 정순(palette 0..3), 결합 슬라이스(palette 4) 부재
        let svg = render_chart_svg(&ofpie_chart(OfPieInfo::default()), 0.0, 0.0, 400.0, 300.0);
        let legend = &svg[svg.find("hwp-chart-legend").expect("범례")..];
        let mut last = 0usize;
        for i in 0..4 {
            let p = legend
                .find(&color_hex(palette(i)))
                .unwrap_or_else(|| panic!("범례 스와치 {i}"));
            assert!(p >= last, "범례 정순 위반 ({i})");
            last = p;
        }
        assert!(
            !legend.contains(&color_hex(palette(4))),
            "범례에 결합 슬라이스 없음"
        );
    }

    #[test]
    fn test_ofpie_two_values_plain_pie_fallback() {
        // n=2 < 3 → 일반 원형 폴백 (ofpie 어휘·serline 부재)
        let mut chart = ofpie_chart(OfPieInfo {
            has_ser_lines: true,
            ..Default::default()
        });
        chart.series[0].values = vec![7.0, 3.0];
        chart.categories = vec!["a".into(), "b".into()];
        let svg = render_chart_svg(&chart, 0.0, 0.0, 400.0, 300.0);
        assert!(!svg.contains("hwp-ofpie"), "n<3은 일반 원형 폴백");
    }

    #[test]
    fn test_pie_exploded_slices_offset() {
        // 쪼개진원형(계열 explosion 25): 각 슬라이스 꼭짓점이 중심에서 중심각
        // 방향으로 r×0.25 이동, 반지름은 1/(1+0.25)로 축소(벌어진 만큼 fit).
        // 정답지: 한컴 쪼개진원형-2022 — 전 슬라이스 균일 벌어짐.
        let mut plain = OoxmlChart {
            chart_type: OoxmlChartType::Pie,
            series: vec![OoxmlSeries {
                values: vec![4.0, 3.0, 2.0],
                ..Default::default()
            }],
            categories: vec!["a".into(), "b".into(), "c".into()],
            ..Default::default()
        };
        let svg_plain = render_chart_svg(&plain, 0.0, 0.0, 400.0, 300.0);
        // 2D 원형 슬라이스 path: "M{cx},{cy} L..." — 전 슬라이스 동일 꼭짓점 = 중심
        let apex = |chunk: &str| -> (f64, f64) {
            let d = &chunk[chunk.find("d=\"M").unwrap() + 4..];
            let (x, rest) = d.split_once(',').unwrap();
            let (y, _) = rest.split_once(' ').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        };
        let arc_r = |chunk: &str| -> f64 {
            let a = &chunk[chunk.find(" A").unwrap() + 2..];
            a.split_once(',').unwrap().0.parse().unwrap()
        };
        // plain 중심/반지름
        let plain_slices: Vec<String> = svg_plain
            .split("<path ")
            .skip(1)
            .filter(|c| c.starts_with("d=\"M"))
            .map(|c| c[..c.find("/>").unwrap()].to_string())
            .collect();
        assert_eq!(plain_slices.len(), 3, "2D 원형 3슬라이스");
        let (cx, cy) = apex(&plain_slices[0]);
        let r_plain = arc_r(&plain_slices[0]);

        plain.series[0].explosion = Some(25.0);
        let svg_ex = render_chart_svg(&plain, 0.0, 0.0, 400.0, 300.0);
        let ex_slices: Vec<String> = svg_ex
            .split("<path ")
            .skip(1)
            .filter(|c| c.starts_with("d=\"M"))
            .map(|c| c[..c.find("/>").unwrap()].to_string())
            .collect();
        assert_eq!(ex_slices.len(), 3);
        let r_ex = arc_r(&ex_slices[0]);
        assert!(
            (r_ex - r_plain / 1.25).abs() < 0.05,
            "반지름 fit 축소: {r_ex} vs {}",
            r_plain / 1.25
        );
        let off = r_ex * 0.25;
        for (i, s) in ex_slices.iter().enumerate() {
            let (ax, ay) = apex(s);
            let d = ((ax - cx).powi(2) + (ay - cy).powi(2)).sqrt();
            assert!(
                (d - off).abs() < 0.05,
                "슬라이스 {i} 꼭짓점 오프셋 {d} ≠ {off}"
            );
        }
        // 서로 다른 방향으로 벌어짐 (꼭짓점 전부 상이)
        let a0 = apex(&ex_slices[0]);
        let a1 = apex(&ex_slices[1]);
        let a2 = apex(&ex_slices[2]);
        assert!(a0 != a1 && a1 != a2 && a0 != a2, "슬라이스별 방향 분리");
    }

    #[test]
    fn test_pie_slices_butt_joined_no_white_border() {
        // 시각판정 확정(2026-07-19): 한컴 원형 계열은 슬라이스 밀착 — 2D/3D/ofPie
        // 정답지 원주 전수 스캔 흰 run 0건 → 흰 테두리 미방출 (마커/라인 할로 무관)
        let pie2d = OoxmlChart {
            chart_type: OoxmlChartType::Pie,
            series: vec![OoxmlSeries {
                values: vec![4.0, 3.0, 2.0],
                ..Default::default()
            }],
            categories: vec!["a".into(), "b".into(), "c".into()],
            ..Default::default()
        };
        let charts = [
            pie2d,
            pie3d_chart(vec![25.0, 25.0, 50.0], 30.0, 30.0),
            ofpie_chart(OfPieInfo {
                has_ser_lines: true,
                ..Default::default()
            }),
            ofpie_chart(OfPieInfo {
                of_pie_type: OfPieType::Bar,
                ..Default::default()
            }),
        ];
        for (i, chart) in charts.iter().enumerate() {
            let svg = render_chart_svg(chart, 0.0, 0.0, 400.0, 300.0);
            assert!(
                !svg.contains("stroke=\"#ffffff\""),
                "원형 계열 {i}: 슬라이스 흰 테두리 잔존"
            );
        }
    }

    #[test]
    fn test_pie3d_degenerate_cameras() {
        // 정의역 방어: rotX=0(타원 퇴화)·90(정원)·perspective=240(cos 음수 위험)
        for (rx_deg, persp) in [(0.0, 30.0), (90.0, 30.0), (30.0, 240.0), (-15.0, 0.0)] {
            let svg = render_chart_svg(
                &pie3d_chart(vec![25.0, 25.0, 50.0], rx_deg, persp),
                0.0,
                0.0,
                400.0,
                300.0,
            );
            assert!(
                !svg.contains("NaN"),
                "rotX={rx_deg}/persp={persp}: NaN 없음"
            );
            let (_, _, rx, ry) = pie3d_top_geom(&svg);
            assert!(
                ry > 0.0 && ry <= rx + 1e-6,
                "타원비 (0,1] 유지: {}",
                ry / rx
            );
        }
    }
}
