//! OOXML 차트 XML 파서 (Task #195 단계 8)
//!
//! DrawingML 차트 XML을 `OoxmlChart` 데이터 모델로 변환한다.
//! 의도적으로 관대한 파서: 알 수 없는 태그는 무시하고 1차 범위 데이터만 추출.

use super::{OoxmlChart, OoxmlChartType, OoxmlSeries};
use quick_xml::events::Event;
use quick_xml::Reader;

/// 파싱 진행 시 문맥(현재 어떤 태그 트리에 있는지) 추적
#[derive(Default)]
struct ParseState {
    // 지금 처리 중인 시리즈
    cur_series: Option<OoxmlSeries>,
    // 파싱 중 임시 값
    cur_text_buf: String,
    in_tx: bool,          // c:tx > c:strRef > c:strCache > c:pt > c:v → 시리즈명
    in_cat: bool,         // c:cat > ... > c:v → 카테고리 (첫 시리즈만 채움)
    in_val: bool,         // c:val > ... > c:v → 값
    in_chart_title: bool, // c:title > c:tx > c:rich > a:p > a:r > a:t
    in_v: bool,           // c:v 텍스트 수집 중
    in_a_t: bool,         // a:t 텍스트 수집 중
    // bar direction은 <c:barDir val="bar"|"col"/>
    bar_dir: Option<BarDir>,
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
                // in_v 또는 in_a_t일 때 텍스트 누적
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

    // 가로/세로 막대 최종 분기
    if matches!(chart.chart_type, OoxmlChartType::Column | OoxmlChartType::Bar) {
        if let Some(BarDir::Bar) = state.bar_dir {
            chart.chart_type = OoxmlChartType::Bar;
        } else {
            chart.chart_type = OoxmlChartType::Column;
        }
    }
    Some(chart)
}

fn handle_start(e: &quick_xml::events::BytesStart, chart: &mut OoxmlChart, st: &mut ParseState) {
    let name = e.local_name();
    let name_bytes = name.as_ref();
    match name_bytes {
        b"barChart" => chart.chart_type = OoxmlChartType::Column, // barDir로 세분
        b"lineChart" => chart.chart_type = OoxmlChartType::Line,
        b"pieChart" => chart.chart_type = OoxmlChartType::Pie,
        b"barDir" => {
            // <c:barDir val="bar" or "col"/>
            if let Some(val) = attr_val(e, "val") {
                st.bar_dir = match val.as_str() {
                    "bar" => Some(BarDir::Bar),
                    "col" => Some(BarDir::Col),
                    _ => None,
                };
            }
        }
        b"ser" => {
            st.cur_series = Some(OoxmlSeries::default());
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
            // a:t (DrawingML 텍스트 런)
            st.in_a_t = true;
            st.cur_text_buf.clear();
        }
        _ => {}
    }
}

fn handle_end(name: &[u8], chart: &mut OoxmlChart, st: &mut ParseState) {
    match name {
        b"v" => {
            st.in_v = false;
            let text = std::mem::take(&mut st.cur_text_buf);
            // tx 안에 있으면 시리즈 이름, cat이면 카테고리, val이면 값
            if let Some(ser) = st.cur_series.as_mut() {
                if st.in_tx {
                    // 시리즈명은 첫 문자열만 (중복 방지)
                    if ser.name.is_empty() {
                        ser.name = text;
                    }
                } else if st.in_cat {
                    // 첫 시리즈일 때만 chart.categories에 추가
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
        b"t" => {
            st.in_a_t = false;
            let text = std::mem::take(&mut st.cur_text_buf);
            if st.in_chart_title && !text.is_empty() {
                // 타이틀: 여러 런이 있을 수 있으므로 누적
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
                chart.series.push(ser);
            }
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
      <c:ser>
        <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Q2</c:v></c:pt></c:strCache></c:strRef></c:tx>
        <c:val><c:numRef><c:numCache>
          <c:pt idx="0"><c:v>120</c:v></c:pt>
          <c:pt idx="1"><c:v>90</c:v></c:pt>
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
        assert_eq!(c.series.len(), 2);
        assert_eq!(c.series[0].name, "Q1");
        assert_eq!(c.series[0].values, vec![100.0, 80.0]);
        assert_eq!(c.series[1].name, "Q2");
        assert_eq!(c.series[1].values, vec![120.0, 90.0]);
        assert_eq!(c.categories, vec!["Seoul", "Busan"]);
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
