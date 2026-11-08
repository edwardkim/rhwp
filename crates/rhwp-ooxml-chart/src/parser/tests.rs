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

// --- C2b (#2278) v2: view3D / gapWidth / gapDepth 파싱 ---

#[test]
fn test_parse_view3d_fields() {
    // 문서 순서 미러: c:view3D는 c:plotArea보다 **앞** (코퍼스 바이트 실측
    // 908 < 1059). 값은 기본값과 다른 원형 코퍼스(rAngAx=0/rotX=30/rotY=0) —
    // 막대 값(15/20)은 View3D::default()와 같아 초기화 순서 버그가 공허
    // 통과한다 (v2 설계 리뷰 확정).
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:view3D><c:rAngAx val="0"/><c:rotX val="30"/><c:rotY val="0"/><c:perspective val="30"/><c:hPercent val="100"/><c:depthPercent val="100"/></c:view3D>
<c:plotArea><c:pie3DChart><c:ser>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:pie3DChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    let v = c.view3d.expect("view3D 파싱");
    assert!(!v.r_ang_ax, "rAngAx=0 → false");
    assert_eq!(v.rot_x, 30.0);
    assert_eq!(v.rot_y, 0.0);
    assert_eq!(v.perspective, 30.0);
    assert_eq!(v.h_percent, 100.0);
    assert_eq!(v.depth_percent, 100.0);
}

#[test]
fn test_parse_view3d_defaults_when_absent() {
    // view3D 요소 없는 bar3D → plot arm 폴백(get_or_insert)으로 Office 관례
    // 기본값(MS-OE376: 15/20/30/rAngAx=1 — XSD 기본 아님).
    let xml3d = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:bar3DChart><c:barDir val="col"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:bar3DChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml3d).expect("parse OK");
    let v = c.view3d.expect("3D plot이면 view3d Some");
    assert!(v.r_ang_ax);
    assert_eq!(v.rot_x, 15.0);
    assert_eq!(v.rot_y, 20.0);
    assert_eq!(v.perspective, 30.0);
    // 2D 차트는 view3d 없음
    let c2d = parse_chart_xml(BAR_XML.as_bytes()).expect("parse OK");
    assert!(c2d.view3d.is_none(), "2D는 view3d None");
}

#[test]
fn test_parse_bar_gap_width_and_depth() {
    // bar3DChart의 gapWidth/gapDepth → bar_gap_width/gap_depth.
    // stock의 upDownBars>gapWidth(up_down_gap_width)와 필드 분리 — 무오염은
    // 기존 test_parse_stock_ohlc(150 저장)와 본 테스트의 None 단언이 상호 가드.
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:view3D><c:rotX val="15"/><c:rotY val="20"/><c:perspective val="30"/><c:rAngAx val="1"/></c:view3D>
<c:plotArea><c:bar3DChart><c:barDir val="col"/>
<c:ser><c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>
<c:gapWidth val="150"/><c:gapDepth val="150"/>
</c:bar3DChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert_eq!(c.bar_gap_width, Some(150.0));
    assert_eq!(c.gap_depth, Some(150.0));
    assert_eq!(c.up_down_gap_width, None, "stock 필드 무오염");
    assert!(c.view3d.expect("view3d").r_ang_ax);
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
    // ofPieChart → Pie (chart_type 앵커 유지 — #1453 라우팅)
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:ofPieChart><c:ofPieType val="pie"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>40</c:v></c:pt><c:pt idx="1"><c:v>25</c:v></c:pt><c:pt idx="2"><c:v>35</c:v></c:pt></c:numCache></c:val></c:ser></c:ofPieChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert_eq!(c.chart_type, OoxmlChartType::Pie);
    assert_eq!(c.series[0].values, vec![40.0, 25.0, 35.0]);
}

// --- C2b Stage 3 (#2278): ofPie 보조플롯 파라미터 ---

#[test]
fn test_parse_ofpie_info() {
    // secondPieSize/serLines 포함 → of_pie Some + chart_type Pie 재확인
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:ofPieChart><c:ofPieType val="pie"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>40</c:v></c:pt><c:pt idx="1"><c:v>25</c:v></c:pt><c:pt idx="2"><c:v>35</c:v></c:pt></c:numCache></c:val></c:ser><c:serLines/><c:secondPieSize val="75"/></c:ofPieChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert_eq!(c.chart_type, OoxmlChartType::Pie);
    let of = c.of_pie.expect("of_pie Some");
    assert_eq!(of.of_pie_type, OfPieType::Pie);
    assert_eq!(of.split_pos, None);
    assert_eq!(of.second_pie_size, 75.0);
    assert!(of.has_ser_lines);
}

#[test]
fn test_parse_ofpie_bar_type() {
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:ofPieChart><c:ofPieType val="bar"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt></c:numCache></c:val></c:ser></c:ofPieChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert_eq!(c.of_pie.expect("of_pie").of_pie_type, OfPieType::Bar);
}

#[test]
fn test_parse_ofpie_split_pos() {
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:ofPieChart><c:ofPieType val="pie"/><c:splitPos val="3"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt></c:numCache></c:val></c:ser></c:ofPieChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert_eq!(c.of_pie.expect("of_pie").split_pos, Some(3.0));
}

#[test]
fn test_parse_pie_explosion() {
    // 쪼개진원형: 계열 레벨 c:explosion val="25" (코퍼스 dPt 0개)
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:pieChart><c:ser><c:explosion val="25"/><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt><c:pt idx="1"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:pieChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert_eq!(c.series[0].explosion, Some(25.0));
    // 부재 시 None (2차원원형 등)
    let xml2 = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:pieChart><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt></c:numCache></c:val></c:ser></c:pieChart></c:plotArea></c:chart></c:chartSpace>"#;
    assert_eq!(
        parse_chart_xml(xml2).expect("parse OK").series[0].explosion,
        None
    );
}

// --- PR #2500 후속: dPt 점별 explosion 비승격 + splitType 해석 ---

#[test]
fn test_dpt_explosion_not_promoted_to_series() {
    // c:dPt 내부 점별 explosion — 점별 렌더 미지원 동안 계열 승격 금지
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:pieChart><c:ser><c:dPt><c:idx val="1"/><c:explosion val="25"/></c:dPt><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt><c:pt idx="1"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:pieChart></c:plotArea></c:chart></c:chartSpace>"#;
    assert_eq!(
        parse_chart_xml(xml).expect("parse OK").series[0].explosion,
        None,
        "dPt explosion 이 계열 전체로 승격되면 안 됨"
    );
    // 계열 레벨 explosion 은 dPt 공존 시에도 유지
    let xml2 = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:pieChart><c:ser><c:explosion val="10"/><c:dPt><c:idx val="1"/><c:explosion val="25"/></c:dPt><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt></c:numCache></c:val></c:ser></c:pieChart></c:plotArea></c:chart></c:chartSpace>"#;
    assert_eq!(
        parse_chart_xml(xml2).expect("parse OK").series[0].explosion,
        Some(10.0),
        "계열 레벨 explosion 은 유지"
    );
}

#[test]
fn test_ofpie_split_type_parsed() {
    let make = |ty: &str| {
        let xml = format!(
            r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:ofPieChart><c:ofPieType val="pie"/><c:splitType val="{ty}"/><c:splitPos val="3"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>4</c:v></c:pt></c:numCache></c:val></c:ser></c:ofPieChart></c:plotArea></c:chart></c:chartSpace>"#
        );
        parse_chart_xml(xml.as_bytes())
            .expect("parse OK")
            .of_pie
            .expect("of_pie")
    };
    assert_eq!(make("pos").split_type, OfPieSplitType::Pos);
    assert_eq!(make("val").split_type, OfPieSplitType::Val);
    assert_eq!(make("percent").split_type, OfPieSplitType::Percent);
    assert_eq!(make("cust").split_type, OfPieSplitType::Cust);
    // splitPos 값 자체는 형태와 무관하게 IR 에 보존 (해석은 렌더에서 게이트)
    assert_eq!(make("val").split_pos, Some(3.0));
}

#[test]
fn test_parse_serlines_not_leaked_to_barchart() {
    // barChart의 c:serLines(누적 계열선)는 of_pie로 새지 않음 (이중 게이트)
    let xml = br#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:barChart><c:barDir val="col"/><c:serLines/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = parse_chart_xml(xml).expect("parse OK");
    assert!(c.of_pie.is_none(), "barChart serLines가 of_pie 생성 금지");
}

// --- C1a Part B (#1453): 막대 누적 grouping 파싱 ---

fn bar_xml_with_grouping(plot: &str, grouping: &str) -> String {
    format!(
        r#"<?xml version="1.0"?><c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea><c:{plot}><c:barDir val="col"/><c:grouping val="{grouping}"/><c:ser><c:val><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:val></c:ser></c:{plot}></c:plotArea></c:chart></c:chartSpace>"#
    )
}

#[test]
fn test_parse_grouping_stacked() {
    let c =
        parse_chart_xml(bar_xml_with_grouping("barChart", "stacked").as_bytes()).expect("parse OK");
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
    let c = parse_chart_xml(line_xml("percentStacked", Some("0")).as_bytes()).expect("parse OK");
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
