//! OOXML 차트 XML 파서
//!
//! DrawingML 차트 XML을 `OoxmlChart` 데이터 모델로 변환한다.
//! 의도적으로 관대한 파서: 알 수 없는 태그는 무시하고 지원 범위 데이터만 추출.
//!
//! ## 콤보/이중축 지원
//! - 여러 `<c:barChart>`, `<c:lineChart>`가 한 차트 안에 공존 가능
//! - 각 plot 블록의 `<c:axId val="...">`를 수집 → 시리즈에 복사
//! - `<c:valAx>`에서 `<c:axId>`와 `<c:axPos>` 수집 → axId→primary/secondary 매핑 생성
//! - 파싱 완료 시 시리즈의 axis_ids를 primary/secondary 집합과 비교해 axis_group 지정

use super::{
    BarGrouping, LegendPos, OfPieInfo, OfPieSplitType, OfPieType, OoxmlChart, OoxmlChartType,
    OoxmlSeries, ScatterStyle, SeriesMarker, View3D,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

use super::{AxisKind, AxisPos, LineSpec, OoxmlAxis, TickMark};

/// [#6624] 계열 밖 `c:spPr > a:ln` 의 주인.
#[derive(Clone, Copy)]
enum LineOwner {
    PlotArea,
    ChartSpace,
}

/// 지금 열린 `a:ln` 이 플롯 영역/차트 바깥 것인지. 계열·축·점별·표식·제목·범례·데이터표의
/// spPr 은 해당 없음.
fn line_owner(st: &ParseState) -> Option<LineOwner> {
    if !(st.in_sp_pr && st.in_ln)
        || st.cur_series.is_some()
        || st.cur_axis.is_some()
        || st.in_d_pt
        || st.in_marker
        || st.in_chart_title
        || st.in_d_table
    {
        return None;
    }
    if st.in_plot_area {
        Some(LineOwner::PlotArea)
    } else if !st.in_chart_body {
        Some(LineOwner::ChartSpace)
    } else {
        None
    }
}

fn set_line(chart: &mut OoxmlChart, owner: LineOwner, spec: LineSpec) {
    match owner {
        LineOwner::PlotArea => chart.plot_area_line = spec,
        LineOwner::ChartSpace => chart.chart_line = spec,
    }
}

/// [#6624] 축 선언 시작 — `chart.axes` 에 넣고 열린 축으로 표시한다.
fn open_axis(chart: &mut OoxmlChart, st: &mut ParseState, kind: AxisKind) {
    chart.axes.push(OoxmlAxis {
        kind,
        ..Default::default()
    });
    st.cur_axis = Some(chart.axes.len() - 1);
}

/// 파싱 진행 시 문맥(현재 어떤 태그 트리에 있는지) 추적
#[derive(Default)]
struct ParseState {
    cur_series: Option<OoxmlSeries>,
    cur_text_buf: String,
    in_tx: bool,
    in_cat: bool,
    collect_categories: bool,
    in_val: bool,
    in_x_val: bool, // c:xVal (분산형 X 값)
    in_y_val: bool, // c:yVal (분산형 Y 값)
    in_chart_title: bool,
    in_v: bool,
    in_a_t: bool,
    in_sp_pr: bool,                       // c:spPr — 계열/그림의 도형 속성
    in_solid_fill: bool,                  // a:solidFill 채우기 영역
    in_ln: bool,                          // a:ln 테두리 영역
    scheme_fill: Option<(u32, f64, f64)>, // a:schemeClr 와 lumMod/lumOff
    in_num_cache: bool,                   // c:numCache — formatCode 파싱
    // c:dPt(점별 속성) 블록 내부 — 점별 explosion 을 계열로 승격하지 않기 위한
    // 문맥 게이트 (PR #2500 후속)
    in_d_pt: bool,
    in_d_lbls: bool,
    // [#6624] 글꼴 크기·선 굵기 문맥. c:chart 밖의 c:txPr 만 차트 전체 기본 글꼴이고,
    // c:plotArea 안 c:title 은 축 제목, c:marker 안 spPr 은 표식 테두리라 제외한다.
    in_chart_body: bool,
    in_plot_area: bool,
    in_tx_pr: bool,
    in_marker: bool,
    // [#6624] 지금 열려 있는 축 선언의 `chart.axes` 색인 (catAx/valAx/dateAx/serAx 안).
    cur_axis: Option<usize>,
    // [#6624] 열린 `a:ln` 의 `w`(EMU) — 플롯 영역·차트 테두리 선 굵기용.
    cur_ln_w: Option<u32>,
    in_d_table: bool,
    bar_dir: Option<BarDir>,
    // 현재 파싱 중인 plot 블록 (barChart/lineChart/pieChart) 안에 있는지
    cur_plot_type: Option<OoxmlChartType>,
    // 현재 plot 블록에서 누적되는 axId (plot 종료 시 해당 plot의 모든 시리즈에 복사)
    cur_plot_ax_ids: Vec<String>,
    // 현재 plot이 시작된 시점의 chart.series.len() — plot 종료 시 이 시점 이후 시리즈에 axIds 할당
    cur_plot_series_start: usize,
    // c:valAx 블록 내에서 수집 중인 axId, axPos
    in_val_ax: bool,
    cur_val_ax_id: Option<String>,
    cur_val_ax_pos: Option<String>,
    // axId → axPos 매핑 (l/r/t/b)
    val_ax_map: HashMap<String, String>,
}

#[derive(Clone, Copy)]
enum BarDir {
    Bar,
    Col,
}

/// OOXML 차트 XML 파싱 진입점
pub fn parse_chart_xml(xml: &[u8]) -> Option<OoxmlChart> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut chart = OoxmlChart::default();
    let mut state = ParseState::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => handle_start(e, &mut chart, &mut state),
            Ok(Event::Empty(ref e)) => {
                handle_start(e, &mut chart, &mut state);
                handle_end(e.local_name().as_ref().as_bytes(), &mut chart, &mut state);
            }
            Ok(Event::End(ref e)) => {
                handle_end(e.local_name().as_ref().as_bytes(), &mut chart, &mut state)
            }
            Ok(Event::Text(t)) => {
                if state.in_v || state.in_a_t {
                    state.cur_text_buf.push_str(t.as_ref());
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }

    if chart.series.is_empty() && chart.title.is_none() {
        return None;
    }
    crate::drawing_style::resolve(xml, &mut chart);

    // 가로/세로 막대 최종 분기 (chart_type이 Column 상태면 barDir로 확정)
    if matches!(
        chart.chart_type,
        OoxmlChartType::Column | OoxmlChartType::Bar
    ) {
        if let Some(BarDir::Bar) = state.bar_dir {
            chart.chart_type = OoxmlChartType::Bar;
        } else {
            chart.chart_type = OoxmlChartType::Column;
        }
    }
    // 시리즈별 series_type이 Column인데 chart_type이 Bar인 경우도 동기화
    for s in chart.series.iter_mut() {
        if matches!(s.series_type, OoxmlChartType::Column | OoxmlChartType::Bar) {
            s.series_type = if matches!(state.bar_dir, Some(BarDir::Bar)) {
                OoxmlChartType::Bar
            } else {
                OoxmlChartType::Column
            };
        }
    }

    // 축 매핑 결정
    // primary: pos="l" (세로 막대/라인의 좌측 Y) 또는 pos="b"가 아닌 첫 valAx
    // secondary: primary가 아닌 나머지
    let mut primary_axid: Option<String> = None;
    let mut secondary_axid: Option<String> = None;
    // 순회 순서를 안정적으로 하기 위해 정렬
    let mut entries: Vec<(String, String)> = state
        .val_ax_map
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (axid, pos) in &entries {
        match pos.as_str() {
            "l" | "b" => {
                if primary_axid.is_none() {
                    primary_axid = Some(axid.clone());
                } else if secondary_axid.is_none() {
                    secondary_axid = Some(axid.clone());
                }
            }
            "r" | "t" => {
                if secondary_axid.is_none() {
                    secondary_axid = Some(axid.clone());
                } else if primary_axid.is_none() {
                    primary_axid = Some(axid.clone());
                }
            }
            _ => {
                if primary_axid.is_none() {
                    primary_axid = Some(axid.clone());
                } else if secondary_axid.is_none() {
                    secondary_axid = Some(axid.clone());
                }
            }
        }
    }

    // 시리즈 axis_group 지정.
    // 분산형(scatter)은 X·Y 모두 valAx(axPos b/l)라 위 primary/secondary 매핑이 두 축을
    // 오분류해 has_secondary_axis=true → 콤보 라우팅으로 새는 것을 차단한다. scatter는
    // axis_group=0, has_secondary_axis=false 기본값을 유지한다. (C1b #1660)
    if chart.chart_type != OoxmlChartType::Scatter {
        for s in chart.series.iter_mut() {
            let is_secondary = match (&secondary_axid, &primary_axid) {
                (Some(sec), _) if s.axis_ids.iter().any(|a| a == sec) => true,
                (_, Some(pri)) if s.axis_ids.iter().any(|a| a == pri) => false,
                _ => false,
            };
            s.axis_group = if is_secondary { 1 } else { 0 };
            if is_secondary {
                chart.has_secondary_axis = true;
            }
        }
    }

    Some(chart)
}

fn handle_start(e: &quick_xml::events::BytesStart, chart: &mut OoxmlChart, st: &mut ParseState) {
    let name = e.local_name();
    let name_bytes = name.as_ref().as_bytes();
    match name_bytes {
        b"hncChartStyle" => {
            chart.color_index = attr_val(e, "colorIndex").and_then(|v| v.parse().ok());
        }
        b"barChart" => {
            chart.chart_type = OoxmlChartType::Column; // barDir로 세분
            st.cur_plot_type = Some(OoxmlChartType::Column);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"lineChart" => {
            if chart.chart_type == OoxmlChartType::Unknown {
                chart.chart_type = OoxmlChartType::Line;
            }
            st.cur_plot_type = Some(OoxmlChartType::Line);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"pieChart" => {
            chart.chart_type = OoxmlChartType::Pie;
            st.cur_plot_type = Some(OoxmlChartType::Pie);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"bar3DChart" => {
            // 3D 막대 — barDir 핸들러가 col/bar를 그대로 채워 파싱 종료 후처리가
            // Column↔Bar를 확정한다. is_3d는 축 정책(C1c) + 투영 렌더(C2b v2).
            chart.chart_type = OoxmlChartType::Column;
            chart.is_3d = true;
            // view3D 요소 없는 3D 차트 폴백 — view3D는 plotArea보다 앞이라
            // 이미 파싱됐으면 덮어쓰지 않는다. (C2b #2278 v2)
            chart.view3d.get_or_insert_with(View3D::default);
            st.cur_plot_type = Some(OoxmlChartType::Column);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"pie3DChart" => {
            // 3D 원형 — 투영 렌더(C2b v2). 타원비는 view3d.rot_x에서 유도.
            chart.chart_type = OoxmlChartType::Pie;
            chart.is_3d = true;
            chart.view3d.get_or_insert_with(View3D::default);
            st.cur_plot_type = Some(OoxmlChartType::Pie);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"ofPieChart" => {
            // ofPie — 주 원 + 보조플롯(원형대원형의 2차 원, 원형대가로막대형의
            // 누적 막대) 렌더(C2b #2278 Stage 3). chart_type은 Pie 유지(#1453
            // 라우팅 앵커), of_pie 필드 유무로 render_of_pie 분기.
            chart.chart_type = OoxmlChartType::Pie;
            chart.of_pie = Some(OfPieInfo::default());
            st.cur_plot_type = Some(OoxmlChartType::Pie);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"scatterChart" => {
            // 분산형 — (x,y) 쌍, 2개 수치축(C1b #1660).
            chart.chart_type = OoxmlChartType::Scatter;
            st.cur_plot_type = Some(OoxmlChartType::Scatter);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"stockChart" => {
            // 주식형 (C2a #2277). 계열 역할은 XML 순서 규약(3계열=고/저/종,
            // 4계열=시/고/저/종 — 코퍼스 실측), 표현은 hiLowLines/upDownBars로 결정.
            chart.chart_type = OoxmlChartType::Stock;
            st.cur_plot_type = Some(OoxmlChartType::Stock);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"line3DChart" => {
            // 코퍼스 27종에 없음 — 방어적 라우팅(placeholder 방지, C1a bar3D/pie3D
            // 선례). 입체 표현은 C2b(#2278), 여기서는 2D 라인 근사 + is_3d(축 정책)만.
            // lineChart와 동일하게 콤보의 주 타입은 덮지 않음. (C2a #2277 stage5)
            if chart.chart_type == OoxmlChartType::Unknown {
                chart.chart_type = OoxmlChartType::Line;
            }
            chart.is_3d = true;
            chart.view3d.get_or_insert_with(View3D::default);
            st.cur_plot_type = Some(OoxmlChartType::Line);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"view3D" => {
            // c:plotArea보다 먼저 온다(ECMA CT_Chart 시퀀스, 코퍼스 바이트 실측
            // 908 < 1059) — 여기서 초기화해야 자식 rotX/rotY/…가 유실되지 않음.
            // (C2b #2278 v2 설계 리뷰: plot arm 초기화는 값 전량 폐기 은폐 버그)
            chart.view3d = Some(View3D::default());
        }
        b"rotX" => {
            if let (Some(v3), Some(v)) = (chart.view3d.as_mut(), attr_f64(e)) {
                v3.rot_x = v;
            }
        }
        b"rotY" => {
            if let (Some(v3), Some(v)) = (chart.view3d.as_mut(), attr_f64(e)) {
                v3.rot_y = v;
            }
        }
        b"perspective" => {
            if let (Some(v3), Some(v)) = (chart.view3d.as_mut(), attr_f64(e)) {
                v3.perspective = v;
            }
        }
        b"rAngAx" => {
            if let (Some(v3), Some(val)) = (chart.view3d.as_mut(), attr_val(e, "val")) {
                v3.r_ang_ax = !matches!(val.as_str(), "0" | "false");
            }
        }
        b"hPercent" => {
            if let (Some(v3), Some(v)) = (chart.view3d.as_mut(), attr_f64(e)) {
                v3.h_percent = v;
            }
        }
        b"depthPercent" => {
            if let (Some(v3), Some(v)) = (chart.view3d.as_mut(), attr_f64(e)) {
                v3.depth_percent = v;
            }
        }
        b"gapDepth" => {
            // bar3D 전용 — 방(씬) 깊이 = 막대 깊이×(1+gap/100). (C2b #2278 v2)
            if matches!(
                st.cur_plot_type,
                Some(OoxmlChartType::Column | OoxmlChartType::Bar)
            ) {
                if let Some(v) = attr_f64(e) {
                    chart.gap_depth = Some(v);
                }
            }
        }
        b"hiLowLines" => {
            // stock 고저선. lineChart에도 올 수 있는 요소라 Stock 게이트. (C2a #2277)
            if st.cur_plot_type == Some(OoxmlChartType::Stock) {
                chart.has_hi_low_lines = true;
            }
        }
        b"upDownBars" => {
            // stock 시가↔종가 캔들 (OHLC). (C2a #2277)
            if st.cur_plot_type == Some(OoxmlChartType::Stock) {
                chart.has_up_down_bars = true;
            }
        }
        b"gapWidth" => {
            // <c:upDownBars> 내부 캔들 폭(Stock 게이트) — C2a #2277.
            // bar/bar3D plot의 동명 요소는 3D 두께 규칙 slot/(n_eff+gap/100)용
            // 별도 필드로 저장(2D는 미사용) — C2b #2278 v2.
            if st.cur_plot_type == Some(OoxmlChartType::Stock) {
                if let Some(v) = attr_f64(e) {
                    chart.up_down_gap_width = Some(v);
                }
            } else if matches!(
                st.cur_plot_type,
                Some(OoxmlChartType::Column | OoxmlChartType::Bar)
            ) {
                if let Some(v) = attr_f64(e) {
                    chart.bar_gap_width = Some(v);
                }
            }
        }
        b"scatterStyle" => {
            if let Some(val) = attr_val(e, "val") {
                chart.scatter_style = match val.as_str() {
                    "line" => ScatterStyle::Line,
                    "lineMarker" => ScatterStyle::LineMarker,
                    "smooth" | "smoothMarker" => ScatterStyle::SmoothMarker,
                    _ => ScatterStyle::Marker, // "marker"/"none"/미상
                };
            }
        }
        b"ofPieType" => {
            if let (Some(of), Some(val)) = (chart.of_pie.as_mut(), attr_val(e, "val")) {
                if val == "bar" {
                    of.of_pie_type = OfPieType::Bar;
                }
            }
        }
        b"splitType" => {
            // pos 외 형태(val/percent/cust)는 splitPos 해석이 달라 count 적용을
            // 막는다 — 렌더에서 기본 정책 폴백. (PR #2500 후속)
            if let (Some(of), Some(val)) = (chart.of_pie.as_mut(), attr_val(e, "val")) {
                of.split_type = match val.as_str() {
                    "pos" => OfPieSplitType::Pos,
                    "val" => OfPieSplitType::Val,
                    "percent" => OfPieSplitType::Percent,
                    "cust" => OfPieSplitType::Cust,
                    _ => OfPieSplitType::Auto,
                };
            }
        }
        b"splitPos" => {
            if let (Some(of), Some(v)) = (chart.of_pie.as_mut(), attr_f64(e)) {
                of.split_pos = Some(v);
            }
        }
        b"secondPieSize" => {
            if let (Some(of), Some(v)) = (chart.of_pie.as_mut(), attr_f64(e)) {
                of.second_pie_size = v;
            }
        }
        b"serLines" => {
            // barChart(누적 계열선)에도 오는 요소 — Pie plot + of_pie 이중 게이트
            // (hiLowLines의 Stock 게이트 선례)
            if st.cur_plot_type == Some(OoxmlChartType::Pie) {
                if let Some(of) = chart.of_pie.as_mut() {
                    of.has_ser_lines = true;
                }
            }
        }
        b"barDir" => {
            if let Some(val) = attr_val(e, "val") {
                st.bar_dir = match val.as_str() {
                    "bar" => Some(BarDir::Bar),
                    "col" => Some(BarDir::Col),
                    _ => None,
                };
            }
        }
        b"grouping" => {
            // bar/bar3D → chart.grouping, line → chart.line_grouping (C1d #2129).
            // 콤보(bar+line 공존)에서 상호 오염 방지 위해 별도 필드에 분기 저장.
            if let Some(val) = attr_val(e, "val") {
                let g = match val.as_str() {
                    "stacked" => BarGrouping::Stacked,
                    "percentStacked" => BarGrouping::PercentStacked,
                    _ => BarGrouping::Clustered,
                };
                match st.cur_plot_type {
                    Some(OoxmlChartType::Column | OoxmlChartType::Bar) => chart.grouping = g,
                    Some(OoxmlChartType::Line) => chart.line_grouping = g,
                    _ => {}
                }
            }
        }
        b"marker" => {
            st.in_marker = true;
            if let Some(ser) = st.cur_series.as_mut() {
                // 계열 내부 <c:marker> 래퍼 — symbol 자식이 없으면 자동 표식
                // (stock 종가 실측: <c:marker><c:size val="7"/> 만). symbol이 오면
                // 아래 b"symbol" arm이 None/Named로 덮어씀. (C2a #2277)
                if ser.marker_symbol == SeriesMarker::NotSpecified {
                    ser.marker_symbol = SeriesMarker::Auto;
                }
            } else if st.cur_plot_type == Some(OoxmlChartType::Line) {
                // plot 레벨 <c:marker val="0|1"/> (lineChart 직계 자식, Empty 이벤트).
                // scatter는 scatterStyle이 담당하므로 Line 한정. 콤보의 lineChart에서도
                // 설정될 수 있으나 render_combo는 미참조 — 무해. (C1d #2129)
                if let Some(val) = attr_val(e, "val") {
                    chart.line_markers = matches!(val.as_str(), "1" | "true");
                }
            }
        }
        b"symbol" => {
            // 계열 내부 <c:marker><c:symbol val>. "none"=표식 억제(stock 시/고/저 실측),
            // 그 외 명시 심볼. (C2a #2277)
            if let Some(ser) = st.cur_series.as_mut() {
                if let Some(val) = attr_val(e, "val") {
                    ser.marker_symbol = if val == "none" {
                        SeriesMarker::None
                    } else {
                        SeriesMarker::Named(val)
                    };
                }
            }
        }
        b"explosion" => {
            // 계열 레벨 쪼개진원형 (%) — 렌더는 2D 원형 경로만 반영. (C2b #2278)
            // c:dPt 내부의 점별 explosion 은 계열 전체로 승격하지 않고 무시한다
            // (점별 렌더 미지원 — PR #2500 후속).
            if st.in_d_pt {
                return;
            }
            if let (Some(ser), Some(v)) = (st.cur_series.as_mut(), attr_f64(e)) {
                ser.explosion = Some(v);
            }
        }
        b"dPt" => st.in_d_pt = true,
        b"dLbls" => st.in_d_lbls = true,
        b"showVal" if st.in_d_lbls => {
            if let (Some(ser), Some(val)) = (st.cur_series.as_mut(), attr_val(e, "val")) {
                ser.show_values = matches!(val.as_str(), "1" | "true");
            }
        }
        b"dLblPos" if st.in_d_lbls => {
            if let Some(ser) = st.cur_series.as_mut() {
                ser.data_label_position = attr_val(e, "val");
            }
        }
        b"ser" => {
            let mut ser = OoxmlSeries::default();
            if let Some(t) = st.cur_plot_type {
                ser.series_type = t;
            }
            st.cur_series = Some(ser);
        }
        b"tx" => st.in_tx = true,
        b"cat" => {
            st.in_cat = true;
            // 첫 계열의 c:cat이 비어 있거나 생략된 차트도 뒤 계열의 범주를 사용한다.
            st.collect_categories = chart.categories.is_empty();
        }
        b"val" => st.in_val = true,
        b"xVal" => st.in_x_val = true,
        b"yVal" => st.in_y_val = true,
        b"title" => {
            st.in_chart_title = true;
            // C1c #1882 갭①: 요소 존재만 기록 (텍스트 유무는 chart.title이 담당 —
            // 한컴은 텍스트 없어도 autoTitleDeleted=0이면 자동 제목을 그림)
            chart.has_title_elem = true;
        }
        b"autoTitleDeleted" => {
            if let Some(val) = attr_val(e, "val") {
                chart.auto_title_deleted = matches!(val.as_str(), "1" | "true");
            }
        }
        b"legendPos" => {
            // C1c #1882 갭③: 한컴 코퍼스는 전 샘플 r(우측). legendPos는 c:legend
            // 안에서만 등장하므로 상태 플래그 불요.
            if let Some(val) = attr_val(e, "val") {
                chart.legend_pos = match val.as_str() {
                    "r" => LegendPos::Right,
                    "l" => LegendPos::Left,
                    "t" => LegendPos::Top,
                    _ => LegendPos::Bottom,
                };
            }
        }
        b"v" => {
            st.in_v = true;
            st.cur_text_buf.clear();
        }
        b"t" => {
            st.in_a_t = true;
            st.cur_text_buf.clear();
        }
        b"chart" => st.in_chart_body = true,
        b"plotArea" => st.in_plot_area = true,
        b"txPr" => st.in_tx_pr = true,
        // [#6624] 글꼴 크기(sz, 1/100pt). `c:chartSpace > c:txPr`(c:chart 밖)의 defRPr 는
        // 차트 전체 기본, `c:title` 안의 defRPr/rPr 는 제목. 축 제목(plotArea 안)은 제외.
        b"defRPr" | b"rPr" => {
            if let Some(pt) = attr_val(e, "sz").and_then(|v| v.parse::<f64>().ok()) {
                let pt = pt / 100.0;
                if st.in_d_lbls {
                    if let Some(ser) = st.cur_series.as_mut() {
                        ser.data_label_size_pt.get_or_insert(pt);
                    }
                } else if st.in_chart_title && !st.in_plot_area {
                    if chart.title_size_pt.is_none() {
                        chart.title_size_pt = Some(pt);
                    }
                } else if st.in_tx_pr && !st.in_chart_body && chart.text_size_pt.is_none() {
                    chart.text_size_pt = Some(pt);
                }
            }
        }
        b"spPr" => st.in_sp_pr = true,
        b"solidFill" => st.in_solid_fill = true,
        b"ln" => {
            st.in_ln = true;
            st.cur_ln_w = attr_val(e, "w").and_then(|v| v.parse::<u32>().ok());
            // [#6624] 계열 `c:spPr > a:ln w` (EMU) — 선 굵기. 점별(dPt)·표식(marker)의
            // spPr 은 계열 선이 아니라 제외.
            if st.in_sp_pr && !st.in_d_pt && !st.in_marker {
                if let Some(w) = st.cur_ln_w {
                    if let Some(ser) = st.cur_series.as_mut() {
                        if ser.line_width_emu.is_none() {
                            ser.line_width_emu = Some(w);
                        }
                    }
                }
            }
        }
        b"dTable" => st.in_d_table = true,
        // [#6053] `c:spPr > a:ln > a:noFill` — 선 없음. `a:ln` 안일 때만 본다
        // (`spPr > noFill` 은 면 채움 없음이라 뜻이 다르다). 주식형 렌더러가 소비한다.
        b"noFill" => {
            if st.in_sp_pr && st.in_ln {
                if let Some(ser) = st.cur_series.as_mut() {
                    ser.line_none = true;
                } else if let Some(owner) = line_owner(st) {
                    set_line(chart, owner, LineSpec::None);
                }
            }
        }
        b"srgbClr" => {
            if st.in_sp_pr && (st.in_solid_fill || st.in_ln) {
                if let Some(val) = attr_val(e, "val") {
                    if let Some(rgb) = parse_rgb_hex(&val) {
                        if let Some(ser) = st.cur_series.as_mut() {
                            if ser.color.is_none() {
                                ser.color = Some(rgb);
                            }
                        } else if let Some(owner) = line_owner(st) {
                            set_line(
                                chart,
                                owner,
                                LineSpec::Solid {
                                    rgb,
                                    width_emu: st.cur_ln_w,
                                },
                            );
                        }
                    }
                }
            }
        }
        b"sysClr" => {
            // 시스템 UI 색을 직접 조회할 수 없는 출력에서도 원문에 저장된
            // lastClr를 소비한다. 명도 변환 뒤 현재 선의 소유자에게 전달한다.
            if line_owner(st).is_some() {
                st.scheme_fill = attr_val(e, "lastClr")
                    .and_then(|v| parse_rgb_hex(&v))
                    .map(|rgb| (rgb, 1.0, 0.0));
            }
        }
        b"schemeClr" => {
            if st.in_sp_pr && (st.in_solid_fill || st.in_ln) {
                if let Some(val) = attr_val(e, "val") {
                    if let Some(rgb) = scheme_color(&val) {
                        st.scheme_fill = Some((rgb, 1.0, 0.0));
                    }
                }
            }
        }
        b"lumMod" => {
            if let (Some((_, factor, _)), Some(value)) = (st.scheme_fill.as_mut(), attr_f64(e)) {
                *factor = value / 100_000.0;
            }
        }
        b"lumOff" => {
            if let (Some((_, _, offset)), Some(value)) = (st.scheme_fill.as_mut(), attr_f64(e)) {
                *offset = value / 100_000.0;
            }
        }
        b"numCache" => st.in_num_cache = true,
        b"formatCode" => {
            // <c:formatCode>#,##0</c:formatCode> — 텍스트 노드로 옴
            st.cur_text_buf.clear();
            st.in_v = true; // 텍스트 누적 플래그 재활용 (handle_end에서 분기)
        }
        b"axId" => {
            if let Some(val) = attr_val(e, "val") {
                if st.in_val_ax {
                    st.cur_val_ax_id = Some(val.clone());
                } else if st.cur_plot_type.is_some() {
                    st.cur_plot_ax_ids.push(val);
                }
            }
        }
        b"axPos" => {
            if let Some(val) = attr_val(e, "val") {
                if let Some(ax) = st.cur_axis.and_then(|i| chart.axes.get_mut(i)) {
                    ax.pos = match val.as_str() {
                        "l" => AxisPos::Left,
                        "r" => AxisPos::Right,
                        "t" => AxisPos::Top,
                        _ => AxisPos::Bottom,
                    };
                }
                if st.in_val_ax {
                    st.cur_val_ax_pos = Some(val);
                }
            }
        }
        b"valAx" => {
            st.in_val_ax = true;
            st.cur_val_ax_id = None;
            st.cur_val_ax_pos = None;
            open_axis(chart, st, AxisKind::Value);
        }
        b"min" | b"max" if st.in_val_ax => {
            if let (Some(ax), Some(value)) = (
                st.cur_axis.and_then(|i| chart.axes.get_mut(i)),
                attr_f64(e).filter(|v| v.is_finite()),
            ) {
                if name_bytes == b"min" {
                    ax.minimum = Some(value);
                } else {
                    ax.maximum = Some(value);
                }
            }
        }
        b"numFmt" if st.in_val_ax => {
            if let Some(ax) = st.cur_axis.and_then(|i| chart.axes.get_mut(i)) {
                ax.format_code = attr_val(e, "formatCode");
            }
        }
        b"catAx" => open_axis(chart, st, AxisKind::Category),
        b"dateAx" => open_axis(chart, st, AxisKind::Date),
        b"serAx" => open_axis(chart, st, AxisKind::Series),
        // [#6624] 축 선언 안의 격자·눈금·숨김. 축 밖의 동명 요소(dLbls 의 delete 등)는 무시.
        b"majorGridlines" => {
            if let Some(ax) = st.cur_axis.and_then(|i| chart.axes.get_mut(i)) {
                ax.major_gridlines = true;
            }
        }
        b"majorTickMark" => {
            if let (Some(ax), Some(val)) = (
                st.cur_axis.and_then(|i| chart.axes.get_mut(i)),
                attr_val(e, "val"),
            ) {
                ax.major_tick_mark = match val.as_str() {
                    "in" => TickMark::In,
                    "cross" => TickMark::Cross,
                    "none" => TickMark::None,
                    _ => TickMark::Out,
                };
            }
        }
        b"delete" => {
            if let (Some(ax), Some(val)) = (
                st.cur_axis.and_then(|i| chart.axes.get_mut(i)),
                attr_val(e, "val"),
            ) {
                ax.deleted = matches!(val.as_str(), "1" | "true");
            }
        }
        _ => {}
    }
}

fn handle_end(name: &[u8], chart: &mut OoxmlChart, st: &mut ParseState) {
    match name {
        b"v" => {
            st.in_v = false;
            let text = std::mem::take(&mut st.cur_text_buf);
            if let Some(ser) = st.cur_series.as_mut() {
                if st.in_tx {
                    if ser.name.is_empty() {
                        ser.name = text;
                    }
                } else if st.in_cat {
                    if st.collect_categories {
                        chart.categories.push(text);
                    }
                } else if st.in_val || st.in_y_val {
                    if let Ok(v) = text.parse::<f64>() {
                        ser.values.push(v);
                    } else {
                        ser.values.push(0.0);
                    }
                } else if st.in_x_val {
                    ser.x_values.push(text.parse::<f64>().unwrap_or(0.0));
                }
            }
        }
        b"formatCode" => {
            st.in_v = false;
            let text = std::mem::take(&mut st.cur_text_buf);
            if !text.is_empty() {
                if let Some(ser) = st.cur_series.as_mut() {
                    if ser.format_code.is_none() {
                        ser.format_code = Some(text);
                    }
                }
            }
        }
        b"t" => {
            st.in_a_t = false;
            let text = std::mem::take(&mut st.cur_text_buf);
            if st.in_chart_title && !text.is_empty() {
                match chart.title.as_mut() {
                    Some(s) => s.push_str(&text),
                    None => chart.title = Some(text),
                }
            }
        }
        b"tx" => st.in_tx = false,
        b"cat" => {
            st.in_cat = false;
            st.collect_categories = false;
        }
        b"val" => st.in_val = false,
        b"dPt" => st.in_d_pt = false,
        b"dLbls" => st.in_d_lbls = false,
        b"chart" => st.in_chart_body = false,
        b"plotArea" => st.in_plot_area = false,
        b"txPr" => st.in_tx_pr = false,
        b"marker" => st.in_marker = false,
        b"xVal" => st.in_x_val = false,
        b"yVal" => st.in_y_val = false,
        b"title" => st.in_chart_title = false,
        b"ser" => {
            if let Some(ser) = st.cur_series.take() {
                // axIds는 plot 종료 시 일괄 할당 (XML 구조상 axId가 ser 뒤에 옴)
                chart.series.push(ser);
            }
        }
        b"spPr" => {
            st.in_sp_pr = false;
        }
        b"solidFill" => st.in_solid_fill = false,
        b"ln" => {
            st.in_ln = false;
            st.cur_ln_w = None;
        }
        b"schemeClr" | b"sysClr" => {
            if let Some((rgb, factor, offset)) = st.scheme_fill.take() {
                let color = apply_luminance(rgb, factor, offset);
                if let Some(ser) = st.cur_series.as_mut() {
                    if ser.color.is_none() {
                        ser.color = Some(color);
                    }
                } else if let Some(owner) = line_owner(st) {
                    set_line(
                        chart,
                        owner,
                        LineSpec::Solid {
                            rgb: color,
                            width_emu: st.cur_ln_w,
                        },
                    );
                }
            }
        }
        b"dTable" => st.in_d_table = false,
        b"numCache" => st.in_num_cache = false,
        b"catAx" | b"dateAx" | b"serAx" => st.cur_axis = None,
        b"valAx" => {
            st.in_val_ax = false;
            st.cur_axis = None;
            if let (Some(id), Some(pos)) = (st.cur_val_ax_id.take(), st.cur_val_ax_pos.take()) {
                st.val_ax_map.insert(id, pos);
            } else if let Some(id) = st.cur_val_ax_id.take() {
                st.val_ax_map.insert(id, String::new());
                st.cur_val_ax_pos = None;
            }
        }
        b"barChart" | b"lineChart" | b"pieChart" | b"bar3DChart" | b"pie3DChart"
        | b"ofPieChart" | b"scatterChart" | b"stockChart" | b"line3DChart" => {
            // plot 종료 — 이 plot에 속한 시리즈에 axIds 복사
            let start = st.cur_plot_series_start;
            for ser in chart.series.iter_mut().skip(start) {
                ser.axis_ids = st.cur_plot_ax_ids.clone();
            }
            st.cur_plot_type = None;
            st.cur_plot_ax_ids.clear();
        }
        _ => {}
    }
}

pub(super) fn attr_val(e: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref().as_bytes() == key.as_bytes() {
            return Some(attr.value.as_ref().to_string());
        }
    }
    None
}

/// `val` 속성을 f64로. view3D/gapDepth 등 수치 단일 속성 요소용. (C2b #2278 v2)
fn attr_f64(e: &quick_xml::events::BytesStart) -> Option<f64> {
    attr_val(e, "val").and_then(|s| s.parse().ok())
}

pub(super) fn parse_rgb_hex(s: &str) -> Option<u32> {
    let t = s.trim().trim_start_matches('#');
    if t.len() != 6 {
        return None;
    }
    u32::from_str_radix(t, 16).ok()
}

/// 테마 색상 이름 → RGB (Office 2016 기본 + HWP 스타일 102 근사)
/// accent1~6, dk1, lt1, dk2, lt2 등
pub(super) fn scheme_color(name: &str) -> Option<u32> {
    match name {
        "accent1" => Some(0x70AD47), // 녹색 (HWP style 102 차트의 1번 시리즈)
        "accent2" => Some(0x4472C4), // 파랑 (2번 시리즈)
        "accent3" => Some(0xED7D31), // 주황
        "accent4" => Some(0xFFC000), // 노랑
        "accent5" => Some(0x5B9BD5), // 하늘
        "accent6" => Some(0xA5A5A5), // 회색
        "dk1" | "tx1" => Some(0x000000),
        "lt1" | "bg1" => Some(0xFFFFFF),
        "dk2" | "tx2" => Some(0x44546A),
        "lt2" | "bg2" => Some(0xE7E6E6),
        _ => None,
    }
}

/// DrawingML `lumMod`/`lumOff`는 RGB 채널이 아니라 HSL 명도에 적용한다.
pub(super) fn apply_luminance(rgb: u32, factor: f64, offset: f64) -> u32 {
    let r = ((rgb >> 16) & 0xff) as f64 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f64 / 255.0;
    let b = (rgb & 0xff) as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let diff = max - min;
    let l = (max + min) / 2.0;
    let s = if diff == 0.0 {
        0.0
    } else {
        diff / (1.0 - (2.0 * l - 1.0).abs())
    };
    let h = if diff == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / diff).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / diff + 2.0)
    } else {
        60.0 * ((r - g) / diff + 4.0)
    };
    let l = (l * factor + offset).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let (rr, gg, bb) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let channel = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    (channel(rr) << 16) | (channel(gg) << 8) | channel(bb)
}

#[cfg(test)]
mod tests;
