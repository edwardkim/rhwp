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

// ---------------------------------------------------------------------------
// S2 — 패처: 구간→바이트열 splice, 점·계열 꼬리 삽입·삭제, 계열명·라벨 치환
// ---------------------------------------------------------------------------

use rhwp::ooxml_chart::patch::{
    apply_chart_edits, apply_value_edits, ChartEdit, EditTarget, PatchError, ValueEdit,
};

fn value(series: usize, point: usize, text: &str) -> ChartEdit {
    ChartEdit::Value(ValueEdit {
        series,
        point,
        target: EditTarget::Value,
        text: text.to_string(),
    })
}

fn label(series: usize, point: usize, text: &str) -> ChartEdit {
    ChartEdit::Value(ValueEdit {
        series,
        point,
        target: EditTarget::Label,
        text: text.to_string(),
    })
}

fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn apply(xml: &str, edits: &[ChartEdit]) -> Result<String, PatchError> {
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    apply_chart_edits(xml.as_bytes(), &data, edits).map(|b| String::from_utf8(b).expect("UTF-8"))
}

/// 카테고리 라벨도 캐시 텍스트 치환 대상이다 — B1 의 `LabelNotEditable` 은 사라진다.
#[test]
fn category_label_edit_patches_the_cache_text() {
    let out = apply(TWO_SERIES_BAR, &[label(0, 0, "바뀐항목")]).expect("패치");
    assert!(out.contains("<c:v>바뀐항목</c:v>"), "{out}");
    assert!(out.contains("Sheet1!$A$2:$A$3"), "c:f 가 사라졌다");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series[0].labels[0].text, "바뀐항목");
    assert_eq!(
        data.series[1].labels[0].text, "항목 1",
        "다른 계열 라벨이 바뀌었다"
    );
}

/// 계열명은 `c:tx` 캐시 텍스트만 바뀌고 `c:f` 는 그대로다.
#[test]
fn series_name_edit_replaces_cache_text_only() {
    let out = apply(
        TWO_SERIES_BAR,
        &[ChartEdit::SeriesName {
            series: 0,
            text: "이름바뀐계열".to_string(),
        }],
    )
    .expect("패치");
    assert!(out.contains("<c:v>이름바뀐계열</c:v>"), "{out}");
    assert!(out.contains("Sheet1!$B$1"), "계열명 c:f 가 사라졌다");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series[0].name.as_deref(), Some("이름바뀐계열"));
    assert_eq!(data.series[1].name.as_deref(), Some("계열 2"));
}

/// `c:tx` 가 없는 계열의 이름은 제자리 치환 대상이 아니다.
#[test]
fn series_name_edit_without_tx_is_refused() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    assert_eq!(
        apply(
            xml,
            &[ChartEdit::SeriesName {
                series: 0,
                text: "이름".to_string()
            }]
        ),
        Err(PatchError::SeriesNameNotPatchable { series: 0 })
    );
}

/// 꼬리 점 추가 — `c:pt` 요소가 붙고 `ptCount` 가 재계산되며, 산출을 다시 읽을 수 있다.
#[test]
fn append_points_adds_pt_elements_and_recounts() {
    let edits = [
        ChartEdit::AppendPoints {
            series: 0,
            target: EditTarget::Label,
            texts: strs(&["추가항목"]),
        },
        ChartEdit::AppendPoints {
            series: 0,
            target: EditTarget::Value,
            texts: strs(&["9"]),
        },
        ChartEdit::AppendPoints {
            series: 1,
            target: EditTarget::Label,
            texts: strs(&["추가항목"]),
        },
        ChartEdit::AppendPoints {
            series: 1,
            target: EditTarget::Value,
            texts: strs(&["8"]),
        },
    ];
    let out = apply(TWO_SERIES_BAR, &edits).expect("패치");
    assert!(
        out.contains(r#"<c:pt idx="2"><c:v>9</c:v></c:pt></c:numCache>"#),
        "새 점이 캐시 끝에 없다: {out}"
    );
    assert!(
        out.contains(r#"<c:pt idx="2"><c:v>추가항목</c:v></c:pt></c:strCache>"#),
        "새 라벨이 캐시 끝에 없다: {out}"
    );
    assert_eq!(
        out.matches(r#"<c:ptCount val="3"/>"#).count(),
        4,
        "라벨·값 블록 4개의 ptCount 가 3 이어야 한다"
    );
    assert_eq!(
        out.matches(r#"<c:ptCount val="1"/>"#).count(),
        2,
        "c:tx 의 ptCount 는 그대로다"
    );
    let data =
        scan_chart_values(out.as_bytes()).expect("산출 재스캔 — 비순차 idx 면 여기서 죽는다");
    for s in &data.series {
        assert_eq!(s.labels.len(), 3);
        assert_eq!(s.values.len(), 3);
        assert_eq!(
            s.labels_shape
                .as_ref()
                .unwrap()
                .pt_count
                .as_ref()
                .unwrap()
                .value,
            3
        );
        assert_eq!(
            s.values_shape
                .as_ref()
                .unwrap()
                .pt_count
                .as_ref()
                .unwrap()
                .value,
            3
        );
    }
    assert_eq!(data.series[0].values[2].text, "9");
    assert_eq!(data.series[1].labels[2].text, "추가항목");
}

/// 꼬리 점 삭제 — `keep` 개만 남고 `ptCount` 가 줄며, 지운 텍스트는 산출에 없다.
#[test]
fn truncate_points_removes_the_tail_and_recounts() {
    let edits = [
        ChartEdit::TruncatePoints {
            series: 0,
            target: EditTarget::Label,
            keep: 1,
        },
        ChartEdit::TruncatePoints {
            series: 0,
            target: EditTarget::Value,
            keep: 1,
        },
    ];
    let out = apply(TWO_SERIES_BAR, &edits).expect("패치");
    assert!(!out.contains("<c:v>2.5</c:v>"), "지운 값이 남았다");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series[0].labels.len(), 1);
    assert_eq!(data.series[0].values.len(), 1);
    assert_eq!(
        data.series[0]
            .values_shape
            .as_ref()
            .unwrap()
            .pt_count
            .as_ref()
            .unwrap()
            .value,
        1
    );
    assert_eq!(data.series[1].values.len(), 2, "다른 계열이 줄었다");
}

/// 빈 점 `<c:v/>` 는 치환할 수 없지만 꼬리 삭제는 된다.
#[test]
fn blank_point_can_be_truncated_but_not_replaced() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numLit><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v/></c:pt>"#,
        r#"</c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    assert_eq!(
        apply(xml, &[value(0, 1, "5")]),
        Err(PatchError::ValueNotPatchable {
            series: 0,
            point: 1
        })
    );
    let out = apply(
        xml,
        &[ChartEdit::TruncatePoints {
            series: 0,
            target: EditTarget::Value,
            keep: 1,
        }],
    )
    .expect("빈 점도 지운다");
    assert!(!out.contains("<c:v/>"));
    assert!(out.contains(r#"<c:ptCount val="1"/>"#));
}

/// 마지막 1점·1계열은 패처 층에서도 지우지 못한다 — 산출을 스캐너가 못 읽게 되는 하한.
#[test]
fn truncate_to_zero_is_refused() {
    assert_eq!(
        apply(
            TWO_SERIES_BAR,
            &[ChartEdit::TruncatePoints {
                series: 0,
                target: EditTarget::Value,
                keep: 0
            }]
        ),
        Err(PatchError::EmptyBlockRefused {
            series: 0,
            target: EditTarget::Value
        })
    );
    assert_eq!(
        apply(TWO_SERIES_BAR, &[ChartEdit::TruncateSeries { keep: 0 }]),
        Err(PatchError::EmptySeriesRefused)
    );
}

/// 앵커·ptCount 가 없는 블록(다층 캐시)은 개수를 바꿀 수 없다.
#[test]
fn block_without_anchor_cannot_be_resized() {
    let xml = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:cat><c:multiLvlStrRef><c:multiLvlStrCache><c:ptCount val="1"/>"#,
        r#"<c:lvl><c:pt idx="0"><c:v>상반기</c:v></c:pt></c:lvl>"#,
        r#"</c:multiLvlStrCache></c:multiLvlStrRef></c:cat>"#,
        r#"<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    assert_eq!(
        apply(
            xml,
            &[ChartEdit::AppendPoints {
                series: 0,
                target: EditTarget::Label,
                texts: strs(&["하반기"])
            }]
        ),
        Err(PatchError::BlockNotResizable {
            series: 0,
            target: EditTarget::Label
        })
    );
}

/// 계열 신설 — 마지막 계열을 복제해 `c:idx`/`c:order` 채번, 이름·값 교체. `c:f` 는 복제분 그대로.
#[test]
fn append_series_clones_the_last_series_with_new_idx_order_name_values() {
    let out = apply(
        TWO_SERIES_BAR,
        &[ChartEdit::AppendSeries {
            name: Some("추가계열".to_string()),
            labels: None,
            values: strs(&["6", "7"]),
        }],
    )
    .expect("패치");
    assert!(
        out.contains(r#"<c:idx val="2"/><c:order val="2"/>"#),
        "채번이 안 됐다: {out}"
    );
    assert_eq!(
        out.matches("Sheet1!$C$2:$C$3").count(),
        2,
        "복제 계열의 c:f 는 원본 그대로 둔다"
    );
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series.len(), 3);
    let added = &data.series[2];
    assert_eq!(added.name.as_deref(), Some("추가계열"));
    assert_eq!(
        added
            .values
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>(),
        ["6", "7"]
    );
    assert_eq!(
        added
            .labels
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>(),
        ["항목 1", "항목 2"]
    );
    assert_eq!(
        data.series[1].name.as_deref(),
        Some("계열 2"),
        "템플릿이 바뀌었다"
    );
    assert_eq!(data.series[1].values[0].text, "2.4");
}

/// 행 수가 같은 호출에서 바뀌면 신설 계열도 **최종** 행 수로 만들어진다.
#[test]
fn append_series_with_row_change_uses_the_final_row_count() {
    let mut edits = Vec::new();
    for s in 0..2 {
        edits.push(ChartEdit::AppendPoints {
            series: s,
            target: EditTarget::Label,
            texts: strs(&["항목 3"]),
        });
        edits.push(ChartEdit::AppendPoints {
            series: s,
            target: EditTarget::Value,
            texts: strs(&["1"]),
        });
    }
    edits.push(ChartEdit::AppendSeries {
        name: Some("계열 3".to_string()),
        labels: Some(strs(&["항목 1", "항목 2", "항목 3"])),
        values: strs(&["3", "3", "3"]),
    });
    let out = apply(TWO_SERIES_BAR, &edits).expect("패치");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series.len(), 3);
    for s in &data.series {
        assert_eq!(s.labels.len(), 3, "{:?}", s.name);
        assert_eq!(s.values.len(), 3, "{:?}", s.name);
        assert_eq!(
            s.values_shape
                .as_ref()
                .unwrap()
                .pt_count
                .as_ref()
                .unwrap()
                .value,
            3
        );
        assert_eq!(
            s.labels_shape
                .as_ref()
                .unwrap()
                .pt_count
                .as_ref()
                .unwrap()
                .value,
            3
        );
    }
    assert_eq!(data.series[2].values[2].text, "3");
    assert_eq!(data.series[2].labels[2].text, "항목 3");
}

/// 신설 계열의 값 개수가 최종 행 수와 다르면 거부한다.
#[test]
fn append_series_length_mismatch_is_refused() {
    assert_eq!(
        apply(
            TWO_SERIES_BAR,
            &[ChartEdit::AppendSeries {
                name: None,
                labels: None,
                values: strs(&["1", "2", "3", "4", "5"]),
            }]
        ),
        Err(PatchError::LengthMismatch {
            series: 2,
            expected: 2,
            actual: 5
        })
    );
}

/// 꼬리 계열 삭제 — `keep` 개만 남는다.
#[test]
fn truncate_series_removes_trailing_ser_elements() {
    let out = apply(TWO_SERIES_BAR, &[ChartEdit::TruncateSeries { keep: 1 }]).expect("패치");
    assert!(!out.contains("계열 2"), "지운 계열이 남았다");
    assert!(out.contains("ho:hncChartStyle"), "모델 밖 요소가 사라졌다");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series.len(), 1);
    assert_eq!(data.series[0].name.as_deref(), Some("계열 1"));
}

/// 같은 앵커의 삽입 여럿은 계획 순서대로 놓인다.
#[test]
fn same_offset_inserts_follow_plan_order() {
    let out = apply(
        TWO_SERIES_BAR,
        &[
            ChartEdit::AppendSeries {
                name: Some("셋째".to_string()),
                labels: None,
                values: strs(&["1", "1"]),
            },
            ChartEdit::AppendSeries {
                name: Some("넷째".to_string()),
                labels: None,
                values: strs(&["2", "2"]),
            },
        ],
    )
    .expect("패치");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(
        data.series
            .iter()
            .map(|s| s.name.clone().unwrap())
            .collect::<Vec<_>>(),
        ["계열 1", "계열 2", "셋째", "넷째"]
    );
    assert!(out.contains(r#"<c:idx val="2"/>"#) && out.contains(r#"<c:idx val="3"/>"#));
}

/// 지워질 점을 동시에 치환하려 하면 구간이 겹쳐 거부한다 — 한 바이트도 쓰지 않는다.
#[test]
fn overlapping_structure_edits_are_refused() {
    let result = apply(
        TWO_SERIES_BAR,
        &[
            ChartEdit::TruncatePoints {
                series: 0,
                target: EditTarget::Value,
                keep: 1,
            },
            value(0, 1, "9"),
        ],
    );
    assert!(
        matches!(
            result,
            Err(PatchError::OverlappingSpans { series: 0, .. })
                | Err(PatchError::OverlappingStructureEdits { series: 0, .. })
        ),
        "{result:?}"
    );
}

/// 이름·라벨·새 점의 XML 특수문자는 이스케이프하지 않고 거부한다.
#[test]
fn unsafe_texts_are_refused_for_names_labels_and_new_points() {
    assert_eq!(
        apply(
            TWO_SERIES_BAR,
            &[ChartEdit::SeriesName {
                series: 0,
                text: "R&D".to_string()
            }]
        ),
        Err(PatchError::UnsafeName { series: 0 })
    );
    assert_eq!(
        apply(TWO_SERIES_BAR, &[label(0, 0, "a<b")]),
        Err(PatchError::UnsafeText {
            series: 0,
            point: 0
        })
    );
    assert_eq!(
        apply(
            TWO_SERIES_BAR,
            &[ChartEdit::AppendPoints {
                series: 0,
                target: EditTarget::Value,
                texts: strs(&["1", "x>y"])
            }]
        ),
        Err(PatchError::UnsafeText {
            series: 0,
            point: 3
        })
    );
}

/// 구조 편집 산출을 다시 스캔해 모든 값을 그대로 되쓰면 바이트 동일이다 — 산출이 자기정합이다.
#[test]
fn structure_output_rescans_and_identity_reapplies_byte_identical() {
    let out = apply(
        TWO_SERIES_BAR,
        &[
            ChartEdit::AppendPoints {
                series: 0,
                target: EditTarget::Label,
                texts: strs(&["항목 3"]),
            },
            ChartEdit::AppendPoints {
                series: 0,
                target: EditTarget::Value,
                texts: strs(&["1"]),
            },
            ChartEdit::TruncatePoints {
                series: 1,
                target: EditTarget::Label,
                keep: 1,
            },
            ChartEdit::TruncatePoints {
                series: 1,
                target: EditTarget::Value,
                keep: 1,
            },
            ChartEdit::SeriesName {
                series: 1,
                text: "둘째".to_string(),
            },
            // 템플릿(마지막 계열)은 같은 목록에서 1행으로 줄었으므로 신설 계열도 1값이다.
            ChartEdit::AppendSeries {
                name: Some("셋째".to_string()),
                labels: None,
                values: strs(&["5"]),
            },
        ],
    )
    .expect("패치");
    let data = scan_chart_values(out.as_bytes()).expect("재스캔");
    assert_eq!(data.series.len(), 3);
    assert_eq!(data.series[2].values.len(), 1);
    assert_eq!(
        data.series[2].labels.len(),
        1,
        "신설 계열은 템플릿의 라벨 편집(꼬리 삭제)을 물려받는다"
    );
    let mut identity = Vec::new();
    for (si, s) in data.series.iter().enumerate() {
        for (pi, p) in s.labels.iter().enumerate() {
            identity.push(label(si, pi, &p.text));
        }
        for (pi, p) in s.values.iter().enumerate() {
            identity.push(value(si, pi, &p.text));
        }
        if let Some(name) = &s.name {
            identity.push(ChartEdit::SeriesName {
                series: si,
                text: name.clone(),
            });
        }
    }
    let again = apply_chart_edits(out.as_bytes(), &data, &identity).expect("항등 재적용");
    assert_eq!(again, out.as_bytes());
}

/// `apply_value_edits` 는 `ChartEdit::Value` 로 감싼 호출과 같다 — B1 호출부는 그대로 돈다.
#[test]
fn apply_value_edits_is_an_unchanged_wrapper() {
    let data = scan_chart_values(TWO_SERIES_BAR.as_bytes()).expect("스캔");
    let edits = [ValueEdit {
        series: 1,
        point: 0,
        target: EditTarget::Value,
        text: "99".to_string(),
    }];
    let via_values = apply_value_edits(TWO_SERIES_BAR.as_bytes(), &data, &edits).expect("패치");
    let via_chart = apply_chart_edits(
        TWO_SERIES_BAR.as_bytes(),
        &data,
        &[ChartEdit::Value(edits[0].clone())],
    )
    .expect("패치");
    assert_eq!(via_values, via_chart);
    assert!(String::from_utf8_lossy(&via_values).contains("<c:v>99</c:v>"));
}
