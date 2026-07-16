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
    BarGrouping, LegendPos, OoxmlChart, OoxmlChartType, OoxmlSeries, ScatterStyle, SeriesMarker,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// 파싱 진행 시 문맥(현재 어떤 태그 트리에 있는지) 추적
#[derive(Default)]
struct ParseState {
    cur_series: Option<OoxmlSeries>,
    cur_text_buf: String,
    in_tx: bool,
    in_cat: bool,
    in_val: bool,
    in_x_val: bool, // c:xVal (분산형 X 값)
    in_y_val: bool, // c:yVal (분산형 Y 값)
    in_chart_title: bool,
    in_v: bool,
    in_a_t: bool,
    in_sp_pr: bool,      // c:spPr — 시리즈/figure의 shape properties
    in_solid_fill: bool, // a:solidFill
    in_ln: bool,         // a:ln (stroke)
    in_num_cache: bool,  // c:numCache — formatCode 파싱
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
                handle_end(e.local_name().as_ref(), &mut chart, &mut state);
            }
            Ok(Event::End(ref e)) => handle_end(e.local_name().as_ref(), &mut chart, &mut state),
            Ok(Event::Text(t)) => {
                if state.in_v || state.in_a_t {
                    let s = t.decode().unwrap_or_default();
                    state.cur_text_buf.push_str(&s);
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
    let name_bytes = name.as_ref();
    match name_bytes {
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
            // 3D 막대 — 2D 근사(C1a #1453). barDir 핸들러가 col/bar를 그대로 채워
            // 파싱 종료 후처리가 Column↔Bar를 확정한다. is_3d는 축 정책용(C1c).
            chart.chart_type = OoxmlChartType::Column;
            chart.is_3d = true;
            st.cur_plot_type = Some(OoxmlChartType::Column);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"pie3DChart" => {
            // 3D 원형 — 단일 원형으로 2D 근사(C1a #1453). 입체감은 후속(C2).
            chart.chart_type = OoxmlChartType::Pie;
            chart.is_3d = true;
            st.cur_plot_type = Some(OoxmlChartType::Pie);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
        }
        b"ofPieChart" => {
            // ofPie — 단일 원형으로 2D 근사(C1a #1453).
            // 보조플롯(원형대원형의 2차 원, 원형대막대의 막대)은 후속(C2).
            chart.chart_type = OoxmlChartType::Pie;
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
            st.cur_plot_type = Some(OoxmlChartType::Line);
            st.cur_plot_ax_ids.clear();
            st.cur_plot_series_start = chart.series.len();
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
            // <c:upDownBars> 내부 캔들 폭. barChart의 동명 요소(막대 간격)는 미사용이라
            // Stock 게이트로 격리. (C2a #2277)
            if st.cur_plot_type == Some(OoxmlChartType::Stock) {
                if let Some(v) = attr_val(e, "val").and_then(|s| s.parse::<f64>().ok()) {
                    chart.up_down_gap_width = Some(v);
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
        b"ser" => {
            let mut ser = OoxmlSeries::default();
            if let Some(t) = st.cur_plot_type {
                ser.series_type = t;
            }
            st.cur_series = Some(ser);
        }
        b"tx" => st.in_tx = true,
        b"cat" => st.in_cat = true,
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
        b"spPr" => st.in_sp_pr = true,
        b"solidFill" => st.in_solid_fill = true,
        b"ln" => st.in_ln = true,
        b"srgbClr" => {
            if st.in_sp_pr && (st.in_solid_fill || st.in_ln) {
                if let Some(val) = attr_val(e, "val") {
                    if let Some(rgb) = parse_rgb_hex(&val) {
                        if let Some(ser) = st.cur_series.as_mut() {
                            if ser.color.is_none() {
                                ser.color = Some(rgb);
                            }
                        }
                    }
                }
            }
        }
        b"schemeClr" => {
            if st.in_sp_pr && (st.in_solid_fill || st.in_ln) {
                if let Some(val) = attr_val(e, "val") {
                    if let Some(rgb) = scheme_color(&val) {
                        if let Some(ser) = st.cur_series.as_mut() {
                            if ser.color.is_none() {
                                ser.color = Some(rgb);
                            }
                        }
                    }
                }
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
            if st.in_val_ax {
                if let Some(val) = attr_val(e, "val") {
                    st.cur_val_ax_pos = Some(val);
                }
            }
        }
        b"valAx" => {
            st.in_val_ax = true;
            st.cur_val_ax_id = None;
            st.cur_val_ax_pos = None;
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
                    if chart.series.is_empty() {
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
        b"cat" => st.in_cat = false,
        b"val" => st.in_val = false,
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
        b"ln" => st.in_ln = false,
        b"numCache" => st.in_num_cache = false,
        b"valAx" => {
            st.in_val_ax = false;
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

fn attr_val(e: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == key.as_bytes() {
            return Some(String::from_utf8_lossy(attr.value.as_ref()).to_string());
        }
    }
    None
}

fn parse_rgb_hex(s: &str) -> Option<u32> {
    let t = s.trim().trim_start_matches('#');
    if t.len() != 6 {
        return None;
    }
    u32::from_str_radix(t, 16).ok()
}

/// 테마 색상 이름 → RGB (Office 2016 기본 + HWP 스타일 102 근사)
/// accent1~6, dk1, lt1, dk2, lt2 등
fn scheme_color(name: &str) -> Option<u32> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const BAR_XML: &str = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="x" xmlns:a="y">
<c:chart>
  <c:title><c:tx><c:rich><a:p><a:r><a:t>Title A</a:t></a:r></a:p></c:rich></c:tx></c:title>
  <c:plotArea>
    <c:barChart>
      <c:barDir val="col"/>
      <c:ser>
        <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Q1</c:v></c:pt></c:strCache></c:strRef></c:tx>
        <c:cat><c:strRef><c:strCache>
          <c:pt idx="0"><c:v>Seoul</c:v></c:pt>
          <c:pt idx="1"><c:v>Busan</c:v></c:pt>
        </c:strCache></c:strRef></c:cat>
        <c:val><c:numRef><c:numCache>
          <c:pt idx="0"><c:v>100</c:v></c:pt>
          <c:pt idx="1"><c:v>80</c:v></c:pt>
        </c:numCache></c:numRef></c:val>
      </c:ser>
    </c:barChart>
  </c:plotArea>
</c:chart>
</c:chartSpace>"#;

    #[test]
    fn test_parse_bar_chart() {
        let c = parse_chart_xml(BAR_XML.as_bytes()).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Column);
        assert_eq!(c.title.as_deref(), Some("Title A"));
        assert!(c.has_title_elem, "명시 제목도 c:title 요소 존재로 기록");
        assert_eq!(c.series.len(), 1);
        assert_eq!(c.series[0].series_type, OoxmlChartType::Column);
        assert_eq!(c.series[0].values, vec![100.0, 80.0]);
        assert_eq!(c.categories, vec!["Seoul", "Busan"]);
    }

    // --- C1c (#1882) 갭①: 자동 제목 플래그 ---

    /// 한컴 코퍼스 실태 미러: c:title 요소는 있으나 텍스트(a:t) 없음 + autoTitleDeleted.
    fn titleless_bar_xml(auto_title_deleted: &str) -> String {
        format!(
            r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:title><c:layout/><c:overlay val="0"/></c:title>
<c:autoTitleDeleted val="{auto_title_deleted}"/>
<c:plotArea><c:barChart><c:barDir val="col"/><c:ser>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#
        )
    }

    #[test]
    fn test_parse_title_elem_without_text() {
        // 텍스트 없는 c:title → title=None 유지(빈 차트 가드 불변) + 요소 존재 플래그.
        let c = parse_chart_xml(titleless_bar_xml("0").as_bytes()).expect("parse OK");
        assert_eq!(c.title, None, "명시 텍스트 없으면 title은 None 유지");
        assert!(c.has_title_elem);
        assert!(!c.auto_title_deleted);
    }

    #[test]
    fn test_parse_auto_title_deleted() {
        let c = parse_chart_xml(titleless_bar_xml("1").as_bytes()).expect("parse OK");
        assert!(c.has_title_elem);
        assert!(c.auto_title_deleted, "val=1 → 자동 제목 억제");
    }

    // --- C1c (#1882) 갭③: 범례 위치 ---

    #[test]
    fn test_parse_legend_pos_right() {
        // 한컴 코퍼스 실태: <c:legend><c:legendPos val="r"/></c:legend> (plotArea 뒤)
        let xml = r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:plotArea><c:barChart><c:barDir val="col"/><c:ser>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:barChart></c:plotArea>
<c:legend><c:legendPos val="r"/><c:overlay val="0"/></c:legend>
</c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml.as_bytes()).expect("parse OK");
        assert_eq!(c.legend_pos, LegendPos::Right);
    }

    #[test]
    fn test_parse_is_3d_flag() {
        // bar3DChart → is_3d (축 정책용). 2D barChart는 false.
        let xml3d = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:bar3DChart><c:barDir val="col"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:bar3DChart></c:plotArea></c:chart></c:chartSpace>"#;
        assert!(parse_chart_xml(xml3d).expect("parse OK").is_3d);
        assert!(!parse_chart_xml(BAR_XML.as_bytes()).expect("parse OK").is_3d);
    }

    #[test]
    fn test_parse_legend_pos_default_bottom() {
        // legend/legendPos 미존재 → 기본 Bottom (현행 하단 배치 유지)
        let c = parse_chart_xml(titleless_bar_xml("0").as_bytes()).expect("parse OK");
        assert_eq!(c.legend_pos, LegendPos::Bottom);
    }

    // --- C2a (#2277): stock (주식형) 파싱 ---

    /// 코퍼스 stock XML 미러: 시/고/저 계열은 `<c:marker><c:symbol val="none"/>`,
    /// 종가만 marker 래퍼(size만, symbol 부재). HLC=hiLowLines만,
    /// OHLC=+upDownBars(gapWidth 150).
    fn stock_xml(with_up_down_bars: bool) -> String {
        let open_ser = if with_up_down_bars {
            r#"<c:ser><c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>시가</c:v></c:pt></c:strCache></c:strRef></c:tx>
<c:marker><c:symbol val="none"/></c:marker>
<c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>44</c:v></c:pt><c:pt idx="1"><c:v>32</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#
        } else {
            ""
        };
        let up_down = if with_up_down_bars {
            r#"<c:upDownBars><c:gapWidth val="150"/><c:upBars/><c:downBars/></c:upDownBars>"#
        } else {
            ""
        };
        format!(
            r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea>
<c:stockChart>
{open_ser}
<c:ser><c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>고가</c:v></c:pt></c:strCache></c:strRef></c:tx>
<c:marker><c:symbol val="none"/></c:marker>
<c:cat><c:strRef><c:strCache><c:pt idx="0"><c:v>1월</c:v></c:pt><c:pt idx="1"><c:v>2월</c:v></c:pt></c:strCache></c:strRef></c:cat>
<c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>55</c:v></c:pt><c:pt idx="1"><c:v>57</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>
<c:ser><c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>저가</c:v></c:pt></c:strCache></c:strRef></c:tx>
<c:marker><c:symbol val="none"/></c:marker>
<c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>11</c:v></c:pt><c:pt idx="1"><c:v>12</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>
<c:ser><c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>종가</c:v></c:pt></c:strCache></c:strRef></c:tx>
<c:marker><c:size val="7"/></c:marker>
<c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>32</c:v></c:pt><c:pt idx="1"><c:v>35</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>
<c:hiLowLines/>
{up_down}
<c:axId val="AX1"/><c:axId val="AX2"/>
</c:stockChart>
</c:plotArea></c:chart></c:chartSpace>"#
        )
    }

    #[test]
    fn test_parse_stock_hlc() {
        let c = parse_chart_xml(stock_xml(false).as_bytes()).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Stock);
        assert!(c.has_hi_low_lines);
        assert!(!c.has_up_down_bars);
        assert_eq!(c.up_down_gap_width, None);
        assert_eq!(c.series.len(), 3);
        assert_eq!(c.series[0].name, "고가");
        assert_eq!(c.series[0].series_type, OoxmlChartType::Stock);
        assert_eq!(c.series[0].marker_symbol, SeriesMarker::None);
        assert_eq!(c.series[1].marker_symbol, SeriesMarker::None);
        assert_eq!(
            c.series[2].marker_symbol,
            SeriesMarker::Auto,
            "종가 = marker 래퍼만(symbol 부재) → Auto"
        );
        assert_eq!(c.categories, vec!["1월", "2월"]);
    }

    #[test]
    fn test_parse_stock_ohlc_up_down_bars() {
        let c = parse_chart_xml(stock_xml(true).as_bytes()).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Stock);
        assert!(c.has_hi_low_lines);
        assert!(c.has_up_down_bars);
        assert_eq!(c.up_down_gap_width, Some(150.0));
        assert_eq!(c.series.len(), 4);
        assert_eq!(c.series[0].name, "시가");
        assert_eq!(c.series[0].marker_symbol, SeriesMarker::None);
        assert_eq!(c.series[3].marker_symbol, SeriesMarker::Auto);
    }

    #[test]
    fn test_parse_line3d_routing() {
        // line3DChart — 코퍼스 27종에 없음. 방어 라우팅(placeholder 방지, C1a
        // bar3D/pie3D 선례): Line + is_3d(축 정책). 입체 표현은 C2b·2D 근사.
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:line3DChart><c:grouping val="stacked"/><c:ser><c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt><c:pt idx="1"><c:v>4</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser><c:axId val="A1"/></c:line3DChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Line);
        assert!(c.is_3d);
        assert_eq!(c.series.len(), 1);
        assert_eq!(c.series[0].series_type, OoxmlChartType::Line);
        assert_eq!(
            c.line_grouping,
            BarGrouping::Stacked,
            "line3D의 grouping도 line_grouping 채택"
        );
        assert_eq!(
            c.series[0].axis_ids,
            vec!["A1".to_string()],
            "plot 종료 시 axId 복사"
        );
    }

    #[test]
    fn test_parse_stock_no_line_marker_cross_talk() {
        // 계열 내부 <c:marker>가 plot 레벨 line_markers를 오염하지 않음
        let c = parse_chart_xml(stock_xml(true).as_bytes()).expect("parse OK");
        assert!(!c.line_markers);
    }

    #[test]
    fn test_parse_bar_gap_width_not_captured() {
        // barChart의 <c:gapWidth>는 stock 캔들 폭(up_down_gap_width)과 무관 — Stock 게이트
        let xml = r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:plotArea><c:barChart><c:barDir val="col"/><c:gapWidth val="150"/><c:ser>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml.as_bytes()).expect("parse OK");
        assert_eq!(c.up_down_gap_width, None);
    }

    #[test]
    fn test_parse_combo_dual_axis() {
        let xml = r#"<c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea>
<c:barChart><c:barDir val="col"/><c:ser>
  <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>금액</c:v></c:pt></c:strCache></c:strRef></c:tx>
  <c:spPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></c:spPr>
  <c:val><c:numRef><c:numCache><c:formatCode>#,##0</c:formatCode>
    <c:pt idx="0"><c:v>1000</c:v></c:pt><c:pt idx="1"><c:v>2000</c:v></c:pt>
  </c:numCache></c:numRef></c:val>
</c:ser><c:axId val="AX1"/><c:axId val="AX2"/></c:barChart>
<c:lineChart><c:ser>
  <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>건수</c:v></c:pt></c:strCache></c:strRef></c:tx>
  <c:spPr><a:ln><a:solidFill><a:schemeClr val="accent2"/></a:solidFill></a:ln></c:spPr>
  <c:val><c:numRef><c:numCache>
    <c:pt idx="0"><c:v>10</c:v></c:pt><c:pt idx="1"><c:v>20</c:v></c:pt>
  </c:numCache></c:numRef></c:val>
</c:ser><c:axId val="AX3"/><c:axId val="AX4"/></c:lineChart>
<c:valAx><c:axId val="AX2"/><c:axPos val="l"/></c:valAx>
<c:valAx><c:axId val="AX4"/><c:axPos val="r"/></c:valAx>
</c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml.as_bytes()).expect("parse OK");
        assert_eq!(c.series.len(), 2);
        assert_eq!(c.series[0].name, "금액");
        assert_eq!(c.series[0].series_type, OoxmlChartType::Column);
        assert_eq!(c.series[0].color, Some(0x70AD47));
        assert_eq!(c.series[0].axis_group, 0);
        assert_eq!(c.series[0].format_code.as_deref(), Some("#,##0"));
        assert_eq!(c.series[1].name, "건수");
        assert_eq!(c.series[1].series_type, OoxmlChartType::Line);
        assert_eq!(c.series[1].color, Some(0x4472C4));
        assert_eq!(c.series[1].axis_group, 1);
        assert!(c.has_secondary_axis);
        assert!(c.is_combo());
    }

    #[test]
    fn test_parse_horizontal_bar() {
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:barChart><c:barDir val="bar"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>5</c:v></c:pt></c:numCache></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Bar);
    }

    #[test]
    fn test_parse_pie_chart() {
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:pieChart><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>30</c:v></c:pt><c:pt idx="1"><c:v>70</c:v></c:pt></c:numCache></c:val></c:ser></c:pieChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Pie);
        assert_eq!(c.series[0].values, vec![30.0, 70.0]);
    }

    #[test]
    fn test_parse_malformed() {
        assert!(parse_chart_xml(b"not xml").is_none());
    }

    // --- C1a (#1453): 3D막대·3D원형·ofPie 라우팅 ---

    #[test]
    fn test_parse_bar3d_col() {
        // bar3DChart + barDir=col → Column (세로 3D 막대 2D 근사)
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:bar3DChart><c:barDir val="col"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>100</c:v></c:pt><c:pt idx="1"><c:v>80</c:v></c:pt></c:numCache></c:val></c:ser></c:bar3DChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Column);
        assert_eq!(c.series[0].values, vec![100.0, 80.0]);
    }

    #[test]
    fn test_parse_bar3d_bar() {
        // bar3DChart + barDir=bar → Bar (가로 3D 막대 2D 근사)
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:bar3DChart><c:barDir val="bar"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>5</c:v></c:pt></c:numCache></c:val></c:ser></c:bar3DChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Bar);
    }

    #[test]
    fn test_parse_pie3d() {
        // pie3DChart → Pie (3D 원형 2D 근사)
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:pie3DChart><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>30</c:v></c:pt><c:pt idx="1"><c:v>70</c:v></c:pt></c:numCache></c:val></c:ser></c:pie3DChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Pie);
        assert_eq!(c.series[0].values, vec![30.0, 70.0]);
    }

    #[test]
    fn test_parse_ofpie() {
        // ofPieChart → Pie (원형대원형/원형대막대 → 단일 원형 근사)
        let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:ofPieChart><c:ofPieType val="pie"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>40</c:v></c:pt><c:pt idx="1"><c:v>25</c:v></c:pt><c:pt idx="2"><c:v>35</c:v></c:pt></c:numCache></c:val></c:ser></c:ofPieChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Pie);
        assert_eq!(c.series[0].values, vec![40.0, 25.0, 35.0]);
    }

    // --- C1a Part B (#1453): 막대 누적 grouping 파싱 ---

    fn bar_xml_with_grouping(plot: &str, grouping: &str) -> String {
        format!(
            r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:{plot}><c:barDir val="col"/><c:grouping val="{grouping}"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:{plot}></c:plotArea></c:chart></c:chartSpace>"#
        )
    }

    #[test]
    fn test_parse_grouping_stacked() {
        let c = parse_chart_xml(bar_xml_with_grouping("barChart", "stacked").as_bytes())
            .expect("parse OK");
        assert_eq!(c.grouping, BarGrouping::Stacked);
    }

    #[test]
    fn test_parse_grouping_percent_stacked() {
        // bar3DChart 경로에서도 grouping 파싱
        let c = parse_chart_xml(bar_xml_with_grouping("bar3DChart", "percentStacked").as_bytes())
            .expect("parse OK");
        assert_eq!(c.grouping, BarGrouping::PercentStacked);
    }

    #[test]
    fn test_parse_grouping_clustered_default() {
        // clustered 명시 → Clustered. grouping 없는 차트도 기본 Clustered.
        let c = parse_chart_xml(bar_xml_with_grouping("barChart", "clustered").as_bytes())
            .expect("parse OK");
        assert_eq!(c.grouping, BarGrouping::Clustered);
        let c2 = parse_chart_xml(BAR_XML.as_bytes()).expect("parse OK");
        assert_eq!(c2.grouping, BarGrouping::Clustered);
    }

    // --- C1d (#2129): 라인 누적 grouping + plot 레벨 marker 파싱 ---

    /// 실제 샘플(누적꺽은선형.hwpx) 구조를 본뜬 라인 XML — 계열 내부 `<c:marker>`
    /// 래퍼(val 없음) + plot 레벨 `<c:marker val>` 공존.
    fn line_xml(grouping: &str, marker_val: Option<&str>) -> String {
        let plot_marker = marker_val
            .map(|v| format!(r#"<c:marker val="{v}"/>"#))
            .unwrap_or_default();
        format!(
            r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:lineChart><c:grouping val="{grouping}"/><c:ser><c:marker><c:symbol val="none"/><c:size val="7"/></c:marker><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser>{plot_marker}</c:lineChart></c:plotArea></c:chart></c:chartSpace>"#
        )
    }

    #[test]
    fn test_parse_line_grouping_stacked() {
        // line stacked → line_grouping 채택, 막대 grouping은 불변 (별도 필드)
        let c = parse_chart_xml(line_xml("stacked", Some("0")).as_bytes()).expect("parse OK");
        assert_eq!(c.line_grouping, BarGrouping::Stacked);
        assert_eq!(c.grouping, BarGrouping::Clustered);
    }

    #[test]
    fn test_parse_line_grouping_percent_stacked() {
        let c =
            parse_chart_xml(line_xml("percentStacked", Some("0")).as_bytes()).expect("parse OK");
        assert_eq!(c.line_grouping, BarGrouping::PercentStacked);
    }

    #[test]
    fn test_parse_line_grouping_standard() {
        // standard → Clustered 흡수 (꺽은선형/표식이있는꺽은선형)
        let c = parse_chart_xml(line_xml("standard", Some("0")).as_bytes()).expect("parse OK");
        assert_eq!(c.line_grouping, BarGrouping::Clustered);
    }

    #[test]
    fn test_parse_combo_grouping_no_cross_contamination() {
        // 콤보(bar stacked + line standard) — 양방향 무오염
        let xml = r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:barChart><c:barDir val="col"/><c:grouping val="stacked"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:barChart><c:lineChart><c:grouping val="standard"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>5</c:v></c:pt></c:numCache></c:val></c:ser></c:lineChart></c:plotArea></c:chart></c:chartSpace>"#;
        let c = parse_chart_xml(xml.as_bytes()).expect("parse OK");
        assert_eq!(c.grouping, BarGrouping::Stacked, "bar grouping 유지");
        assert_eq!(
            c.line_grouping,
            BarGrouping::Clustered,
            "line standard가 bar에 오염되지 않음"
        );
    }

    #[test]
    fn test_parse_line_marker_flag() {
        // plot 레벨 <c:marker val="1"/> → true / "0"·부재 → false
        let c1 = parse_chart_xml(line_xml("standard", Some("1")).as_bytes()).expect("parse OK");
        assert!(c1.line_markers);
        let c0 = parse_chart_xml(line_xml("standard", Some("0")).as_bytes()).expect("parse OK");
        assert!(!c0.line_markers);
        let cn = parse_chart_xml(line_xml("standard", None).as_bytes()).expect("parse OK");
        assert!(!cn.line_markers);
    }

    #[test]
    fn test_parse_series_marker_ignored() {
        // 계열 내부 <c:marker>(val 없음, symbol/size 래퍼)는 line_markers에 무영향.
        // line_xml은 계열 내부 marker를 항상 포함 — plot 레벨 부재 시 false 유지.
        let c = parse_chart_xml(line_xml("stacked", None).as_bytes()).expect("parse OK");
        assert!(!c.line_markers);
        // scatterChart 내부의 <c:marker val="1"/>도 무영향 (Line plot 게이트)
        let xml = r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:scatterChart><c:scatterStyle val="marker"/><c:ser><c:xVal><c:numCache><c:pt idx="0"><c:v>1</c:v></c:pt></c:numCache></c:xVal><c:yVal><c:numCache><c:pt idx="0"><c:v>2</c:v></c:pt></c:numCache></c:yVal></c:ser><c:marker val="1"/></c:scatterChart></c:plotArea></c:chart></c:chartSpace>"#;
        let s = parse_chart_xml(xml.as_bytes()).expect("parse OK");
        assert!(!s.line_markers);
    }

    // --- C1b (#1660): 분산형(scatter) xVal/yVal 파싱 ---

    /// 실제 샘플 구조를 본뜬 분산형 XML (2 시리즈, X·Y 수치축 2개).
    /// 시리즈명은 `c:tx`, 값은 `c:xVal`/`c:yVal`의 numCache `c:v`.
    fn scatter_xml(style: &str) -> String {
        format!(
            r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea>
<c:scatterChart>
  <c:scatterStyle val="{style}"/>
  <c:ser>
    <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Y1 값</c:v></c:pt></c:strCache></c:strRef></c:tx>
    <c:xVal><c:numRef><c:numCache><c:formatCode>General</c:formatCode>
      <c:pt idx="0"><c:v>0.7</c:v></c:pt><c:pt idx="1"><c:v>1.8</c:v></c:pt><c:pt idx="2"><c:v>2.6</c:v></c:pt>
    </c:numCache></c:numRef></c:xVal>
    <c:yVal><c:numRef><c:numCache>
      <c:pt idx="0"><c:v>2.7</c:v></c:pt><c:pt idx="1"><c:v>3.2</c:v></c:pt><c:pt idx="2"><c:v>0.8</c:v></c:pt>
    </c:numCache></c:numRef></c:yVal>
  </c:ser>
  <c:ser>
    <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Y2 값</c:v></c:pt></c:strCache></c:strRef></c:tx>
    <c:xVal><c:numRef><c:numCache>
      <c:pt idx="0"><c:v>0.7</c:v></c:pt><c:pt idx="1"><c:v>1.8</c:v></c:pt><c:pt idx="2"><c:v>2.6</c:v></c:pt>
    </c:numCache></c:numRef></c:xVal>
    <c:yVal><c:numRef><c:numCache>
      <c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt><c:pt idx="2"><c:v>4</c:v></c:pt>
    </c:numCache></c:numRef></c:yVal>
  </c:ser>
  <c:axId val="111"/><c:axId val="222"/>
</c:scatterChart>
<c:valAx><c:axId val="111"/><c:axPos val="b"/></c:valAx>
<c:valAx><c:axId val="222"/><c:axPos val="l"/></c:valAx>
</c:plotArea></c:chart></c:chartSpace>"#
        )
    }

    #[test]
    fn test_parse_scatter_marker() {
        let c = parse_chart_xml(scatter_xml("marker").as_bytes()).expect("parse OK");
        assert_eq!(c.chart_type, OoxmlChartType::Scatter);
        assert_eq!(c.scatter_style, ScatterStyle::Marker);
        assert_eq!(c.series.len(), 2);
        // 시리즈명은 x_values로 새지 않는다 (in_tx 우선)
        assert_eq!(c.series[0].name, "Y1 값");
        assert_eq!(c.series[0].x_values, vec![0.7, 1.8, 2.6]);
        assert_eq!(c.series[0].values, vec![2.7, 3.2, 0.8]);
        assert_eq!(c.series[1].x_values, vec![0.7, 1.8, 2.6]);
        assert_eq!(c.series[1].values, vec![1.0, 2.0, 4.0]);
        // 분산형은 카테고리를 쓰지 않는다
        assert!(c.categories.is_empty());
    }

    #[test]
    fn test_parse_scatter_style_line() {
        let c = parse_chart_xml(scatter_xml("line").as_bytes()).expect("parse OK");
        assert_eq!(c.scatter_style, ScatterStyle::Line);
    }

    #[test]
    fn test_parse_scatter_style_line_marker() {
        let c = parse_chart_xml(scatter_xml("lineMarker").as_bytes()).expect("parse OK");
        assert_eq!(c.scatter_style, ScatterStyle::LineMarker);
    }

    #[test]
    fn test_parse_scatter_style_smooth_marker() {
        let c = parse_chart_xml(scatter_xml("smoothMarker").as_bytes()).expect("parse OK");
        assert_eq!(c.scatter_style, ScatterStyle::SmoothMarker);
    }

    #[test]
    fn test_scatter_no_secondary_axis() {
        // 회귀 가드: scatter의 2개 valAx(b/l)가 primary/secondary로 오분류되어
        // 콤보 라우팅으로 새면 안 된다. (parser.rs 축 분류 가드)
        let c = parse_chart_xml(scatter_xml("marker").as_bytes()).expect("parse OK");
        assert!(!c.has_secondary_axis, "scatter는 보조축이 없어야 함");
        assert!(!c.is_combo(), "scatter는 콤보가 아니어야 함");
        assert!(
            c.series.iter().all(|s| s.axis_group == 0),
            "scatter 시리즈는 모두 기본축(axis_group=0)",
        );
    }
}
