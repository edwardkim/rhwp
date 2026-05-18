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

use super::{OoxmlChart, OoxmlChartType, OoxmlSeries};
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

    // 시리즈 axis_group 지정
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
        b"barDir" => {
            if let Some(val) = attr_val(e, "val") {
                st.bar_dir = match val.as_str() {
                    "bar" => Some(BarDir::Bar),
                    "col" => Some(BarDir::Col),
                    _ => None,
                };
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
        b"title" => st.in_chart_title = true,
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
                } else if st.in_val {
                    if let Ok(v) = text.parse::<f64>() {
                        ser.values.push(v);
                    } else {
                        ser.values.push(0.0);
                    }
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
        b"barChart" | b"lineChart" | b"pieChart" => {
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
        assert_eq!(c.series.len(), 1);
        assert_eq!(c.series[0].series_type, OoxmlChartType::Column);
        assert_eq!(c.series[0].values, vec![100.0, 80.0]);
        assert_eq!(c.categories, vec!["Seoul", "Busan"]);
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
}
