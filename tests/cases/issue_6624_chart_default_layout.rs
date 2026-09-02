//! [#6624] OOXML 차트 자동 레이아웃·기본 서식 — 한/글 정답지(`pdf/chart/**`) 실측 계약.
//!
//! 차트 XML 에는 글꼴 크기가 `c:chartSpace/c:txPr` 의 10pt 하나뿐이고 제목 크기·계열 선
//! 굵기·배치(`c:manualLayout`)는 없다. 한/글은 그 자리를 기본 서식으로 채운다: 제목 14pt
//! 검정, 라벨 10pt 검정, 격자·축선 #8c8c8c 1px, 계열 선 2.25pt. 배치는 글꼴 배수(제목
//! baseline 1.9T, 플롯 상단 3.26T, 플롯 하단 3.1L, 원형은 왼쪽 여백 0·아래 1.1L).
//! 이 파일은 표본 문서의 1쪽 SVG 에서 그 서식이 실제로 나오는지 본다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::ooxml_chart::renderer::render_chart_svg;
use rhwp::ooxml_chart::{BarGrouping, LegendPos, OoxmlChart, OoxmlChartType, OoxmlSeries};

fn page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {rel}: {e:?}"))
}

fn chart_group(svg: &str) -> &str {
    let i = svg
        .find("<g class=\"hwp-ooxml-chart\">")
        .expect("OOXML 차트 그룹");
    &svg[i..]
}

/// `pat` 바로 뒤의 숫자.
fn num_after(s: &str, pat: &str) -> f64 {
    let i = s.find(pat).unwrap_or_else(|| panic!("{pat} 없음")) + pat.len();
    let rest = &s[i..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end]
        .parse()
        .unwrap_or_else(|_| panic!("{pat} 뒤 숫자 아님: {rest:.20}"))
}

#[test]
fn clustered_column_uses_hancom_default_typography_and_axis() {
    let svg = page0_svg("samples/chart/세로막대형/묶은세로막대형.hwp");
    let g = chart_group(&svg);
    // 차트 XML 기본 글꼴 sz=1000 → 라벨 13.33px 검정. 제목은 sz 없음 → 14pt.
    assert!(g.contains("class=\"hwp-chart-title\""), "제목");
    assert!(g.contains("font-size=\"18.67\""), "제목 14pt");
    assert!(
        g.contains("font-size=\"13.33\" fill=\"#000000\""),
        "라벨 10pt 검정"
    );
    assert!(
        !g.contains("fill=\"#666\"") && !g.contains("fill=\"#333\""),
        "옛 회색 글자 잔존"
    );
    assert!(g.contains("class=\"hwp-chart-axis\""), "축선");
    assert!(
        g.contains("stroke=\"#8c8c8c\" stroke-width=\"1\""),
        "격자·축선 #8c8c8c 1px"
    );
    assert!(!g.contains("stroke=\"#e8e8e8\""), "옛 연회색 격자 잔존");

    // 배치: 제목 baseline → 플롯 상단(축선 y1) = 1.36T, 축선 → 프레임 하단 = 3.1L
    let frame_y = num_after(g, " y=\"");
    let frame_h = num_after(g, " height=\"");
    let title = &g[g.find("class=\"hwp-chart-title\"").unwrap()..];
    let title_y = num_after(title, " y=\"");
    let axis = &g[g.find("class=\"hwp-chart-axis\"").unwrap()..];
    let plot_top = num_after(axis, " y1=\"");
    let plot_bottom = num_after(axis, " y2=\"");
    let t = 14.0 * 96.0 / 72.0;
    let l = 10.0 * 96.0 / 72.0;
    assert!(
        (title_y - frame_y - 1.9 * t).abs() < 0.1,
        "제목 baseline 1.9T: {}",
        title_y - frame_y
    );
    assert!(
        (plot_top - title_y - 1.36 * t).abs() < 0.1,
        "제목→플롯 1.36T: {}",
        plot_top - title_y
    );
    assert!(
        (frame_y + frame_h - plot_bottom - 3.1 * l).abs() < 0.1,
        "플롯 하단 여백 3.1L: {}",
        frame_y + frame_h - plot_bottom
    );
}

#[test]
fn line_series_default_width_is_office_2_25pt() {
    // 계열 spPr 에 a:ln w 가 없다 → Office 기본 2.25pt = 3px (한/글 실측 3px).
    let svg = page0_svg("samples/chart/라인/꺽은선형.hwp");
    let g = chart_group(&svg);
    let series_paths: Vec<&str> = g
        .split("<path d=\"M")
        .skip(1)
        .map(|p| &p[..p.find("/>").expect("path 닫힘")])
        .filter(|p| p.contains("fill=\"none\""))
        .collect();
    assert_eq!(series_paths.len(), 3, "계열 3");
    for p in series_paths {
        assert!(
            p.contains("stroke-width=\"3.00\""),
            "계열 선 3px 아님: {p:.80}"
        );
    }
}

#[test]
fn pie_fills_plot_region_like_hancom() {
    // 한/글 실측 2차원원형: 프레임 252.5 에 지름 183(0.725), 중심 y 146(0.578).
    let svg = page0_svg("samples/chart/원형/2차원원형.hwp");
    let g = chart_group(&svg);
    let frame_y = num_after(g, " y=\"");
    let frame_h = num_after(g, " height=\"");
    let r = num_after(g, " A");
    let first = &g[g.find("<path d=\"M").unwrap()..];
    let cy = num_after(&first[first.find(',').unwrap()..], ",");
    let d_ratio = 2.0 * r / frame_h;
    let cy_ratio = (cy - frame_y) / frame_h;
    assert!(
        (d_ratio - 0.725).abs() < 0.02,
        "지름/프레임 높이 0.725: {d_ratio:.3}"
    );
    assert!(
        (cy_ratio - 0.578).abs() < 0.02,
        "중심 y/프레임 높이 0.578: {cy_ratio:.3}"
    );
}

// ---- 파서: 차트 XML 에서 읽는 값 ----

#[test]
fn parse_text_size_from_chart_space_tx_pr_only() {
    // 차트 기본 글꼴은 c:chartSpace 직계 c:txPr 의 defRPr sz. c:chart 안의 txPr(범례 800·축
    // 제목 900)은 그 요소 것이라 기본 글꼴도 제목 글꼴도 아니다.
    let xml = r#"<c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:title><c:txPr><a:p><a:pPr><a:defRPr b="0"/></a:pPr></a:p></c:txPr></c:title>
<c:plotArea><c:barChart><c:barDir val="col"/><c:ser>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:barChart>
<c:valAx><c:title><c:txPr><a:p><a:pPr><a:defRPr sz="900"/></a:pPr></a:p></c:txPr></c:title></c:valAx>
</c:plotArea>
<c:legend><c:txPr><a:p><a:pPr><a:defRPr sz="800"/></a:pPr></a:p></c:txPr></c:legend>
</c:chart>
<c:txPr><a:p><a:pPr><a:defRPr sz="1000"/></a:pPr></a:p></c:txPr>
</c:chartSpace>"#;
    let c = OoxmlChart::parse(xml.as_bytes()).expect("parse OK");
    assert_eq!(c.text_size_pt, Some(10.0));
    assert_eq!(
        c.title_size_pt, None,
        "제목에 sz 없음 → None (축 제목 900 은 제목이 아님)"
    );
}

#[test]
fn parse_title_size_from_title_run() {
    // 제목 run 의 rPr sz 가 제목 글꼴. 차트 기본 글꼴은 없으면 None.
    let xml = r#"<c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart>
<c:title><c:tx><c:rich><a:p><a:pPr><a:defRPr/></a:pPr><a:r><a:rPr sz="1600"/><a:t>제목</a:t></a:r></a:p></c:rich></c:tx></c:title>
<c:plotArea><c:barChart><c:barDir val="col"/><c:ser>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = OoxmlChart::parse(xml.as_bytes()).expect("parse OK");
    assert_eq!(c.title_size_pt, Some(16.0));
    assert_eq!(c.text_size_pt, None);
}

#[test]
fn parse_series_line_width_skips_marker_and_dpt() {
    // 계열 선 굵기는 계열 직계 spPr 의 a:ln w. 표식(marker)·점별(dPt) spPr 은 제외.
    let xml = r#"<c:chartSpace xmlns:c="x" xmlns:a="y"><c:chart><c:plotArea>
<c:lineChart><c:ser>
  <c:marker><c:symbol val="circle"/><c:spPr><a:ln w="9525"/></c:spPr></c:marker>
  <c:dPt><c:idx val="0"/><c:spPr><a:ln w="12700"/></c:spPr></c:dPt>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser><c:ser>
  <c:spPr><a:ln w="28575" cap="rnd"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln></c:spPr>
  <c:val><c:numRef><c:numCache><c:pt idx="0"><c:v>3</c:v></c:pt></c:numCache></c:numRef></c:val>
</c:ser></c:lineChart></c:plotArea></c:chart></c:chartSpace>"#;
    let c = OoxmlChart::parse(xml.as_bytes()).expect("parse OK");
    assert_eq!(c.series.len(), 2);
    assert_eq!(
        c.series[0].line_width_emu, None,
        "표식·점별 spPr 의 ln w 는 계열 선 굵기가 아니다"
    );
    assert_eq!(c.series[1].line_width_emu, Some(28575));
}

// ---- 렌더러: 합성 차트로 기하 핀 (코퍼스 프레임 430×250) ----

fn bars_chart() -> OoxmlChart {
    OoxmlChart {
        chart_type: OoxmlChartType::Column,
        grouping: BarGrouping::Clustered,
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

fn line_chart() -> OoxmlChart {
    OoxmlChart {
        chart_type: OoxmlChartType::Line,
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

/// `pat` 가 있는 태그의 속성 문자열(`pat` 부터 `>` 앞까지).
fn tag_after<'a>(svg: &'a str, pat: &str) -> &'a str {
    let i = svg.find(pat).unwrap_or_else(|| panic!("{pat} 없음"));
    let rest = &svg[i..];
    &rest[..rest.find('>').expect("태그 닫힘")]
}

/// 한/글 기본 서식 실측 핀 — 제목 14pt 검정 baseline 35.5, 첫 격자선 61, 축선 209,
/// 카테고리 라벨 10pt 검정, 격자·축선 #8c8c8c 1px.
#[test]
fn hancom_default_layout_pins_430x250() {
    let mut chart = bars_chart();
    chart.title = Some("차트 제목".into());
    let svg = render_chart_svg(&chart, 0.0, 0.0, 430.0, 250.0);

    let title = tag_after(&svg, "class=\"hwp-chart-title\"");
    assert!(title.contains("font-size=\"18.67\""), "제목 14pt: {title}");
    assert!(title.contains("fill=\"#000000\""), "제목 검정: {title}");
    let ty = num_after(title, " y=\"");
    assert!((ty - 35.47).abs() < 0.05, "제목 baseline 35.5: {ty}");

    let axis = tag_after(&svg, "class=\"hwp-chart-axis\"");
    assert!(
        axis.contains("stroke=\"#8c8c8c\"") && axis.contains("stroke-width=\"1\""),
        "축선 스타일: {axis}"
    );
    let top = num_after(axis, " y1=\"");
    let bottom = num_after(axis, " y2=\"");
    assert!((top - 60.85).abs() < 0.05, "플롯 상단(첫 격자선) 61: {top}");
    assert!((bottom - 208.67).abs() < 0.05, "축선 209: {bottom}");

    let cat = svg
        .split("<text ")
        .find(|t| t.contains(">a<"))
        .expect("카테고리 라벨");
    assert!(
        cat.contains("font-size=\"13.33\"") && cat.contains("fill=\"#000000\""),
        "라벨 10pt 검정: {cat}"
    );
    let cy = num_after(cat, " y=\"");
    assert!(
        (cy - (208.67 + 13.33 * 2.3)).abs() < 0.1,
        "라벨 baseline = 축선 + 2.3L: {cy}"
    );

    for l in svg.split("<line ").skip(1) {
        let l = &l[..l.find("/>").expect("line 닫힘")];
        assert!(
            l.contains("stroke=\"#8c8c8c\"") && l.contains("stroke-width=\"1\""),
            "격자·축선 #8c8c8c 1px: {l}"
        );
    }
}

#[test]
fn explicit_text_and_title_sizes_are_honored() {
    // 차트 XML 이 준 글꼴 크기(pt)를 쓰고, 여백은 그 배수로 따라간다.
    let mut chart = bars_chart();
    chart.title = Some("T".into());
    chart.text_size_pt = Some(12.0);
    chart.title_size_pt = Some(21.0);
    let svg = render_chart_svg(&chart, 0.0, 0.0, 430.0, 250.0);
    let title = tag_after(&svg, "class=\"hwp-chart-title\"");
    assert!(title.contains("font-size=\"28.00\""), "제목 21pt: {title}");
    let ty = num_after(title, " y=\"");
    assert!((ty - 28.0 * 1.9).abs() < 0.05, "제목 baseline 1.9T: {ty}");
    let cat = svg
        .split("<text ")
        .find(|t| t.contains(">a<"))
        .expect("카테고리 라벨");
    assert!(cat.contains("font-size=\"16.00\""), "라벨 12pt: {cat}");
    let axis = tag_after(&svg, "class=\"hwp-chart-axis\"");
    let top = num_after(axis, " y1=\"");
    assert!((top - 28.0 * 3.26).abs() < 0.05, "플롯 상단 3.26T: {top}");
}

#[test]
fn pie_fills_plot_region_430x250() {
    // 원형: 왼쪽 여백 0, 제목 baseline + 1.0T 아래부터 프레임 하단 1.1L 위까지가 플롯이고
    // 원은 거기에 꽉 찬다 (한/글 실측 2차원원형 지름 183, 중심 y 146).
    let chart = OoxmlChart {
        chart_type: OoxmlChartType::Pie,
        title: Some("판매".into()),
        legend_pos: LegendPos::Right,
        series: vec![OoxmlSeries {
            name: "판매".into(),
            values: vec![8.0, 3.0, 1.0, 1.0],
            series_type: OoxmlChartType::Pie,
            ..Default::default()
        }],
        categories: vec![
            "1 분기".into(),
            "2 분기".into(),
            "3 분기".into(),
            "4 분기".into(),
        ],
        ..Default::default()
    };
    let svg = render_chart_svg(&chart, 0.0, 0.0, 430.0, 250.0);
    let r = num_after(&svg, " A");
    let top = 18.6667 * 2.9;
    let ph = 250.0 - top - 13.3333 * 1.1;
    assert!(
        (r - ph / 2.0).abs() < 0.1,
        "반지름 = 플롯 높이/2: r={r} ph={ph}"
    );
    let first = &svg[svg.find("<path d=\"M").unwrap()..];
    let cy = num_after(&first[first.find(',').unwrap()..], ",");
    assert!(
        (cy - (top + ph / 2.0)).abs() < 0.1,
        "중심 y = 플롯 세로 중앙: {cy}"
    );
}

#[test]
fn series_line_width_default_and_explicit() {
    // a:ln w 없음 → Office 기본 2.25pt(3px); 12700 EMU(1pt) → 1.33px.
    let mut chart = line_chart();
    chart.series[1].line_width_emu = Some(12700);
    let svg = render_chart_svg(&chart, 0.0, 0.0, 430.0, 250.0);
    let widths: Vec<&str> = svg
        .split("<path d=\"M")
        .skip(1)
        .filter(|p| p.contains("fill=\"none\""))
        .map(|p| {
            let i = p.find("stroke-width=\"").expect("stroke-width") + 14;
            &p[i..i + 4]
        })
        .collect();
    assert_eq!(widths, vec!["3.00", "1.33", "3.00"]);
}
