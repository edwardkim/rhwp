//! [#5652] B2 구조 편집 계약 — 스캐너의 구조 좌표(S1)와 패처의 삽입·삭제(S2).
//!
//! 합성 XML 과 공개 API(`rhwp::ooxml_chart::{data, patch}`)만 쓴다. 크레이트 안
//! `#[cfg(test)]` 에 테스트를 늘리는 것은 CI 단위 테스트 상한(`unit-test-tier-policy.json`)이
//! 막으므로 계약은 여기에 둔다. 코퍼스 실측은 `tests/issue_4100_chart_data_edit.rs` Stage 9.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::ooxml_chart::data::{scan_chart_values, ChartPoint, ChartSeries, PlotKind, SeriesAxis};
use std::ops::Range;

/// 2계열 × 2카테고리 막대 차트 — 코퍼스(한컴 단일 라인)와 같은 골격.
const TWO_SERIES_BAR: &str = concat!(
    r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:varyColors val="0"/>"#,
    r#"<c:ser><c:idx val="0"/><c:order val="0"/>"#,
    r#"<c:tx><c:strRef><c:f>Sheet1!$B$1</c:f><c:strCache><c:ptCount val="1"/>"#,
    r#"<c:pt idx="0"><c:v>계열 1</c:v></c:pt></c:strCache></c:strRef></c:tx>"#,
    r#"<c:cat><c:strRef><c:f>Sheet1!$A$2:$A$3</c:f><c:strCache><c:ptCount val="2"/>"#,
    r#"<c:pt idx="0"><c:v>항목 1</c:v></c:pt><c:pt idx="1"><c:v>항목 2</c:v></c:pt>"#,
    r#"</c:strCache></c:strRef></c:cat>"#,
    r#"<c:val><c:numRef><c:f>Sheet1!$B$2:$B$3</c:f><c:numCache>"#,
    r#"<c:formatCode>General</c:formatCode><c:ptCount val="2"/>"#,
    r#"<c:pt idx="0"><c:v>4.3</c:v></c:pt><c:pt idx="1"><c:v>2.5</c:v></c:pt>"#,
    r#"</c:numCache></c:numRef></c:val></c:ser>"#,
    r#"<c:ser><c:idx val="1"/><c:order val="1"/>"#,
    r#"<c:tx><c:strRef><c:f>Sheet1!$C$1</c:f><c:strCache><c:ptCount val="1"/>"#,
    r#"<c:pt idx="0"><c:v>계열 2</c:v></c:pt></c:strCache></c:strRef></c:tx>"#,
    r#"<c:cat><c:strRef><c:f>Sheet1!$A$2:$A$3</c:f><c:strCache><c:ptCount val="2"/>"#,
    r#"<c:pt idx="0"><c:v>항목 1</c:v></c:pt><c:pt idx="1"><c:v>항목 2</c:v></c:pt>"#,
    r#"</c:strCache></c:strRef></c:cat>"#,
    r#"<c:val><c:numRef><c:f>Sheet1!$C$2:$C$3</c:f><c:numCache>"#,
    r#"<c:formatCode>General</c:formatCode><c:ptCount val="2"/>"#,
    r#"<c:pt idx="0"><c:v>2.4</c:v></c:pt><c:pt idx="1"><c:v>4.4</c:v></c:pt>"#,
    r#"</c:numCache></c:numRef></c:val></c:ser>"#,
    r#"</c:barChart></c:plotArea></c:chart>"#,
    r#"<c:extLst><ho:hncChartStyle val="1"/></c:extLst></c:chartSpace>"#,
);

fn slice<'a>(xml: &'a str, span: &Range<usize>) -> &'a str {
    &xml[span.clone()]
}

fn points_of(series: &ChartSeries) -> impl Iterator<Item = &ChartPoint> {
    series.labels.iter().chain(&series.values)
}

// ---------------------------------------------------------------------------
// S1 — 스캐너 구조 좌표
// ---------------------------------------------------------------------------

/// 점마다 `<c:pt …>…</c:pt>` 요소 구간이 있고, 텍스트 구간은 그 안에 있다.
#[test]
fn pt_element_spans_wrap_each_point() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    for series in &data.series {
        for p in points_of(series) {
            let element = p.element_span.clone().expect("pt 요소 구간");
            let raw = slice(xml, &element);
            assert!(raw.starts_with("<c:pt "), "요소 시작이 아니다: {raw}");
            assert!(raw.ends_with("</c:pt>"), "요소 끝이 아니다: {raw}");
            let text = p.span.clone().expect("텍스트 구간");
            assert!(
                element.start < text.start && text.end < element.end,
                "텍스트 구간이 요소 구간 밖이다"
            );
        }
    }
}

/// 요소 구간은 서로 겹치지 않고 문서 순서다 — 꼬리 삭제가 구간 산술로 성립하는 전제.
#[test]
fn pt_element_spans_are_disjoint_and_ordered() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    for series in &data.series {
        for block in [&series.labels, &series.values] {
            let spans: Vec<Range<usize>> = block
                .iter()
                .map(|p| p.element_span.clone().expect("요소 구간"))
                .collect();
            for pair in spans.windows(2) {
                assert!(
                    pair[0].end <= pair[1].start,
                    "요소 구간이 겹치거나 역순이다: {pair:?}"
                );
            }
        }
    }
}

/// 빈 값 `<c:v/>` 는 텍스트 구간이 없어도 요소 구간은 있다 — 지울 수는 있고 고칠 수만 없다.
#[test]
fn empty_point_element_still_has_an_element_span() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numLit><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v/></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt>"#,
        r#"</c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    let blank = &data.series[0].values[0];
    assert_eq!(blank.span, None);
    assert_eq!(
        slice(xml, blank.element_span.as_ref().expect("요소 구간")),
        r#"<c:pt idx="0"><c:v/></c:pt>"#
    );
}

/// 계열마다 `<c:ser>…</c:ser>` 요소 구간과 접두어가 있다.
#[test]
fn series_element_spans_wrap_each_ser_and_carry_prefix() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    assert_eq!(data.series.len(), 2);
    for series in &data.series {
        let raw = slice(xml, &series.element_span);
        assert!(raw.starts_with("<c:ser>"), "{raw}");
        assert!(raw.ends_with("</c:ser>"), "{raw}");
        assert_eq!(series.prefix, "c:");
    }
    assert!(
        data.series[0].element_span.end <= data.series[1].element_span.start,
        "계열 구간이 겹친다"
    );

    let other = xml.replace("c:", "chart:");
    let data = scan_chart_values(other.as_bytes()).expect("접두어가 달라도 스캔");
    assert_eq!(data.series[0].prefix, "chart:");
    assert!(slice(&other, &data.series[0].element_span).starts_with("<chart:ser>"));
}

/// 라벨·값 블록의 `ptCount` 속성값 구간이 선언값으로 되읽히고, `c:tx` 의 `ptCount val="1"` 은 섞이지 않는다.
#[test]
fn pt_count_span_reads_back_the_declared_count() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    for series in &data.series {
        for shape in [
            series.labels_shape.as_ref().expect("라벨 블록"),
            series.values_shape.as_ref().expect("값 블록"),
        ] {
            let pt_count = shape.pt_count.as_ref().expect("ptCount");
            assert_eq!(pt_count.value, 2, "c:tx 의 ptCount(1) 가 섞였다");
            assert_eq!(slice(xml, &pt_count.span), "2");
            assert!(
                shape.element_span.start < pt_count.span.start
                    && pt_count.span.end < shape.element_span.end,
                "ptCount 구간이 블록 밖이다"
            );
        }
    }
}

/// 블록 요소 구간은 `<c:cat>`/`<c:val>` 전체를 감싼다.
#[test]
fn block_element_spans_wrap_the_section() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    let s = &data.series[0];
    let labels = slice(xml, &s.labels_shape.as_ref().unwrap().element_span);
    assert!(
        labels.starts_with("<c:cat>") && labels.ends_with("</c:cat>"),
        "{labels}"
    );
    let values = slice(xml, &s.values_shape.as_ref().unwrap().element_span);
    assert!(
        values.starts_with("<c:val>") && values.ends_with("</c:val>"),
        "{values}"
    );
}

/// 삽입 앵커는 마지막 점 요소 끝이자 캐시 닫는 태그 직전이다.
#[test]
fn insert_anchor_sits_after_the_last_point_and_before_the_cache_close() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    let s = &data.series[0];
    let labels = s.labels_shape.as_ref().unwrap();
    let at = labels.insert_at.expect("라벨 앵커");
    assert_eq!(
        at,
        s.labels.last().unwrap().element_span.as_ref().unwrap().end
    );
    assert!(
        xml[at..].starts_with("</c:strCache>"),
        "{}",
        &xml[at..at + 20]
    );
    let values = s.values_shape.as_ref().unwrap();
    let at = values.insert_at.expect("값 앵커");
    assert_eq!(
        at,
        s.values.last().unwrap().element_span.as_ref().unwrap().end
    );
    assert!(
        xml[at..].starts_with("</c:numCache>"),
        "{}",
        &xml[at..at + 20]
    );
}

/// 점이 하나도 없는 캐시(`ptCount val="0"`)도 닫는 태그 직전을 앵커로 준다.
#[test]
fn empty_cache_anchors_before_its_close_tag() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numRef><c:numCache><c:ptCount val="0"/></c:numCache></c:numRef></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    let shape = data.series[0].values_shape.as_ref().expect("값 블록");
    assert_eq!(shape.pt_count.as_ref().map(|c| c.value), Some(0));
    let at = shape.insert_at.expect("앵커");
    assert!(
        xml[at..].starts_with("</c:numCache>"),
        "{}",
        &xml[at..at + 20]
    );
}

/// 계열명 캐시 텍스트 구간 — 참조형(`strCache`)과 리터럴형(`<c:tx><c:v>`) 둘 다.
#[test]
fn series_name_span_reads_back_the_name() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    assert_eq!(
        slice(xml, data.series[0].name_span.as_ref().expect("이름 구간")),
        "계열 1"
    );
    assert_eq!(
        slice(xml, data.series[1].name_span.as_ref().expect("이름 구간")),
        "계열 2"
    );

    let literal = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:tx><c:v>리터럴 이름</c:v></c:tx>"#,
        r#"<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(literal.as_bytes()).expect("스캔");
    assert_eq!(data.series[0].name.as_deref(), Some("리터럴 이름"));
    assert_eq!(
        slice(
            literal,
            data.series[0].name_span.as_ref().expect("이름 구간")
        ),
        "리터럴 이름"
    );
    // 리터럴형 `c:v` 는 `c:pt` 없이 오므로 점 요소 구간은 없다 — 계열명은 점이 아니다.

    let unnamed = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(unnamed.as_bytes()).expect("스캔");
    assert_eq!(data.series[0].name, None);
    assert_eq!(data.series[0].name_span, None);
}

/// `c:idx`/`c:order` 속성값 구간 — 계열 복제 시 채번 자리.
#[test]
fn idx_and_order_spans_read_back() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    for (i, series) in data.series.iter().enumerate() {
        assert_eq!(
            slice(xml, series.idx_span.as_ref().expect("idx")),
            i.to_string()
        );
        assert_eq!(
            slice(xml, series.order_span.as_ref().expect("order")),
            i.to_string()
        );
    }
}

/// `c:dPt` 안의 `c:idx` 는 계열 idx 가 아니다 — 서브트리째 건너뛴다.
#[test]
fn dpt_idx_does_not_override_series_idx() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:idx val="0"/><c:order val="0"/>"#,
        r#"<c:dPt><c:idx val="7"/><c:bubble3D val="0"/></c:dPt>"#,
        r#"<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    assert_eq!(
        slice(xml, data.series[0].idx_span.as_ref().expect("idx")),
        "0"
    );
}

/// 둘러싼 `*Chart` 요소가 계열의 plot 종류다 — 콤보는 계열마다 다르다.
#[test]
fn plot_kind_follows_the_enclosing_plot_element() {
    fn single(plot_tag: &str) -> PlotKind {
        let xml = format!(
            concat!(
                r#"<c:chartSpace><c:chart><c:plotArea><c:{tag}><c:ser>"#,
                r#"<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
                r#"</c:ser></c:{tag}></c:plotArea></c:chart></c:chartSpace>"#,
            ),
            tag = plot_tag
        );
        scan_chart_values(xml.as_bytes()).expect("스캔").series[0].plot
    }
    assert_eq!(single("barChart"), PlotKind::Bar);
    assert_eq!(single("bar3DChart"), PlotKind::Bar);
    assert_eq!(single("lineChart"), PlotKind::Line);
    assert_eq!(single("pieChart"), PlotKind::Pie);
    assert_eq!(single("pie3DChart"), PlotKind::Pie);
    assert_eq!(single("ofPieChart"), PlotKind::OfPie);
    assert_eq!(single("doughnutChart"), PlotKind::Doughnut);
    assert_eq!(single("stockChart"), PlotKind::Stock);
    assert_eq!(single("scatterChart"), PlotKind::Scatter);
    assert_eq!(single("radarChart"), PlotKind::Radar);
    assert_eq!(single("areaChart"), PlotKind::Area);
    assert_eq!(single("bubbleChart"), PlotKind::Bubble);
    assert_eq!(single("fooChart"), PlotKind::Other);

    let combo = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea>"#,
        r#"<c:barChart><c:ser><c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val></c:ser></c:barChart>"#,
        r#"<c:lineChart><c:ser><c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser></c:lineChart>"#,
        r#"</c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(combo.as_bytes()).expect("스캔");
    assert_eq!(
        data.series.iter().map(|s| s.plot).collect::<Vec<_>>(),
        [PlotKind::Bar, PlotKind::Line]
    );
}

/// `ptCount` 가 없는 리터럴 블록은 `pt_count` 만 None 이고 앵커는 있다; `c:cat` 없는 계열은 라벨 블록이 없다.
#[test]
fn blocks_without_cache_or_ptcount_expose_none() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    let s = &data.series[0];
    assert!(s.labels_shape.is_none(), "c:cat 이 없는데 라벨 블록이 있다");
    let values = s.values_shape.as_ref().expect("값 블록");
    assert!(values.pt_count.is_none(), "ptCount 가 없는데 구간이 있다");
    let at = values.insert_at.expect("앵커");
    assert!(xml[at..].starts_with("</c:numLit>"));
}

/// 다층 카테고리(`multiLvlStrCache`)는 캐시로 보지 않으므로 ptCount·앵커를 싣지 않는다 — 구조 편집 거부의 근거.
#[test]
fn multi_level_cache_is_not_an_insert_anchor() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:cat><c:multiLvlStrRef><c:multiLvlStrCache><c:ptCount val="2"/>"#,
        r#"<c:lvl><c:pt idx="0"><c:v>상반기</c:v></c:pt><c:pt idx="1"><c:v>하반기</c:v></c:pt></c:lvl>"#,
        r#"</c:multiLvlStrCache></c:multiLvlStrRef></c:cat>"#,
        r#"<c:val><c:numRef><c:numCache><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(xml.as_bytes()).expect("다층 라벨 문서는 읽힌다");
    let s = &data.series[0];
    assert!(s.labels_multi_level);
    let labels = s.labels_shape.as_ref().expect("c:cat 블록 자체는 있다");
    assert!(labels.pt_count.is_none(), "다층 캐시의 ptCount 가 실렸다");
    assert!(labels.insert_at.is_none(), "다층 캐시에 앵커가 실렸다");
    assert_eq!(
        s.values_shape
            .as_ref()
            .unwrap()
            .pt_count
            .as_ref()
            .unwrap()
            .value,
        2
    );
}

/// 분산형은 라벨 블록이 `c:xVal`, 값 블록이 `c:yVal` 이다.
#[test]
fn scatter_blocks_are_x_and_y() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:scatterChart><c:ser>"#,
        r#"<c:xVal><c:numRef><c:numCache><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>0.7</c:v></c:pt><c:pt idx="1"><c:v>1.8</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:xVal>"#,
        r#"<c:yVal><c:numRef><c:numCache><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>2.7</c:v></c:pt><c:pt idx="1"><c:v>3.2</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:yVal>"#,
        r#"</c:ser></c:scatterChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    let s = &data.series[0];
    assert_eq!(s.axis, SeriesAxis::Scatter);
    assert_eq!(s.plot, PlotKind::Scatter);
    assert!(slice(xml, &s.labels_shape.as_ref().unwrap().element_span).starts_with("<c:xVal>"));
    assert!(slice(xml, &s.values_shape.as_ref().unwrap().element_span).starts_with("<c:yVal>"));
}

/// 기존 계약 — `&xml[span] == text` 는 그대로다.
#[test]
fn legacy_spans_still_slice_back_to_their_text() {
    let xml = TWO_SERIES_BAR;
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    for series in &data.series {
        for p in points_of(series) {
            assert_eq!(slice(xml, p.span.as_ref().unwrap()), p.text);
        }
    }
}
