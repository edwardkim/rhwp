//! PR #7406: 편집 가능한 OOXML 차트의 축·범주·데이터 라벨과
//! OLE 미리보기 색상표를 공개 API에서 검증한다.

use std::io::Read;

use rhwp::ole_chart::apply_preview_palette;
use rhwp::ooxml_chart::parser::parse_chart_xml;
use rhwp::ooxml_chart::{AxisKind, BarGrouping, OoxmlChart, OoxmlChartType, OoxmlSeries};
use rhwp::parser::ole_container::parse_ole_container;

#[test]
fn stacked_chart_uses_explicit_percent_value_axis() {
    let xml = r#"<c:chartSpace xmlns:c="x"><c:chart><c:plotArea>
      <c:barChart><c:barDir val="col"/><c:grouping val="stacked"/>
        <c:ser><c:tx><c:v>A</c:v></c:tx><c:dLbls><c:txPr><a:p><a:pPr><a:defRPr sz="1000"/></a:pPr></a:p></c:txPr><c:dLblPos val="outEnd"/><c:showVal val="1"/></c:dLbls><c:val><c:numRef><c:numCache><c:formatCode>0%</c:formatCode>
          <c:pt idx="0"><c:v>0.7</c:v></c:pt>
        </c:numCache></c:numRef></c:val></c:ser>
      </c:barChart>
      <c:catAx><c:axPos val="b"/></c:catAx>
      <c:valAx><c:scaling><c:min val="0"/><c:max val="1"/></c:scaling>
        <c:axPos val="l"/><c:numFmt formatCode="0%" sourceLinked="0"/>
      </c:valAx>
    </c:plotArea></c:chart></c:chartSpace>"#;
    let chart = parse_chart_xml(xml.as_bytes()).expect("chart");
    let axis = chart
        .axes
        .iter()
        .find(|a| a.kind == AxisKind::Value)
        .unwrap();
    assert_eq!((axis.minimum, axis.maximum), (Some(0.0), Some(1.0)));
    assert_eq!(axis.format_code.as_deref(), Some("0%"));
    assert!(chart.series[0].show_values);
    assert_eq!(chart.series[0].data_label_size_pt, Some(10.0));
    assert_eq!(
        chart.series[0].data_label_position.as_deref(),
        Some("outEnd")
    );
    let svg = chart.render_svg(0.0, 0.0, 430.0, 250.0);
    assert!(svg.contains(">100%</text>"), "{svg}");
    assert!(svg.contains(">20%</text>"), "{svg}");
    assert!(!svg.contains(">1.5</text>"), "{svg}");
    assert!(svg.contains(">70%</text>"), "{svg}");
}

#[test]
fn scheme_luminance_is_applied_in_hsl_space() {
    let xml = r#"<c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea>
      <c:barChart><c:ser><c:spPr><a:solidFill>
        <a:schemeClr val="accent4"><a:lumMod val="40000"/><a:lumOff val="60000"/></a:schemeClr>
      </a:solidFill></c:spPr><c:val><c:numRef><c:numCache>
        <c:pt idx="0"><c:v>1</c:v></c:pt>
      </c:numCache></c:numRef></c:val></c:ser></c:barChart>
    </c:plotArea></c:chart></c:chartSpace>"#;
    let chart = parse_chart_xml(xml.as_bytes()).unwrap();
    assert_eq!(chart.series[0].color, Some(0xffe699));
}

#[test]
fn categories_may_begin_in_a_later_series() {
    let xml = r#"<c:chartSpace xmlns:c="x"><c:chart><c:plotArea><c:barChart>
      <c:ser><c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>0.7</c:v></c:pt>
      </c:numCache></c:numRef></c:val></c:ser>
      <c:ser><c:cat><c:strRef><c:strCache>
        <c:pt idx="0"><c:v>20-29</c:v></c:pt><c:pt idx="1"><c:v>30-39</c:v></c:pt>
      </c:strCache></c:strRef></c:cat><c:val><c:numRef><c:numCache>
        <c:pt idx="0"><c:v>0.3</c:v></c:pt>
      </c:numCache></c:numRef></c:val></c:ser>
    </c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
    let chart = parse_chart_xml(xml.as_bytes()).unwrap();
    assert_eq!(chart.categories, ["20-29", "30-39"]);
    assert!(chart
        .render_svg(0.0, 0.0, 430.0, 250.0)
        .contains(">20-29</text>"));
}

#[test]
fn stacked_out_end_label_is_readable_on_dark_next_segment() {
    let chart = OoxmlChart {
        chart_type: OoxmlChartType::Column,
        grouping: BarGrouping::Stacked,
        categories: vec!["20-29".into()],
        series: vec![
            OoxmlSeries {
                values: vec![0.58],
                color: Some(0x289b6e),
                format_code: Some("0%".into()),
                show_values: true,
                data_label_size_pt: Some(10.0),
                data_label_position: Some("outEnd".into()),
                ..Default::default()
            },
            OoxmlSeries {
                values: vec![0.22],
                color: Some(0x000000),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let svg = chart.render_svg(0.0, 0.0, 430.0, 250.0);
    assert!(
        svg.contains("fill=\"#ffffff\" text-anchor=\"middle\">58%</text>"),
        "{svg}"
    );
}

#[test]
fn issue_7406_ole_preview_supplies_custom_series_colors() {
    let bytes = std::fs::read("samples/issue7406/7406_OLE__CHART.hwpx").unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut xml = Vec::new();
    zip.by_name("Chart/chart1.xml")
        .unwrap()
        .read_to_end(&mut xml)
        .unwrap();
    let mut ole = Vec::new();
    zip.by_name("BinData/ole1.ole")
        .unwrap()
        .read_to_end(&mut ole)
        .unwrap();
    let container = parse_ole_container(&ole).unwrap();
    let mut chart = OoxmlChart::parse(&xml).unwrap();
    chart.series[0].values[0] = 0.4; // 편집된 OOXML 값은 미리보기의 원래 값과 달라도 된다.
    assert!(apply_preview_palette(
        &mut chart,
        container.raw_contents.as_deref().unwrap(),
        container.preview_emf.as_deref().unwrap(),
        643.28,
        294.87,
    ));
    assert_eq!(chart.series[0].color, Some(0x289b6e));
    assert_eq!(chart.series[1].color, Some(0x000000));
    assert_eq!(chart.series[2].color, Some(0xffef99));
}
