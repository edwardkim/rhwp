//! #5797 회귀 가드 — HWPX 도형의 **자기닫힘 자식 요소**에서 파서가 폭주하지 않는다.
//!
//! `<hp:fillBrush/>`·`<hp:renderingInfo/>`·`<hp:drawText/>`·`<hp:p/>` 처럼 자식이
//! 없는 표기는 종료 태그도 없다. 종전 파서는 이들을 여는 태그와 같은 갈래에서
//! 처리해 하위 파서(`parse_shape_fill_brush`·`parse_rendering_info`·
//! `parse_draw_text`·`parse_paragraph`)를 태웠고, 그 하위 파서가 존재하지 않는
//! 종료 태그를 찾아 **그 도형의 남은 자식과 뒤 형제 도형까지** 통째로 소비했다.
//!
//! 겉으로는 "도형 테두리는 그려지는데 글상자 글자만 통째로 없다"로 보인다 —
//! `<hp:lineShape>` 는 이 자식들보다 앞에 있어 이미 읽혔고, `<hp:drawText>` 는
//! 폭주 구간에 삼켜져 `text_box` 가 비기 때문이다.

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::hwpx::section::parse_hwpx_section;

/// 도형 하나(`<hp:rect>`)를 만든다. `quirk` 로 지정한 자식만 자기닫힘 표기로 낸다.
fn rect(instid: u32, text: &str, quirk: &str) -> String {
    let rendering = if quirk == "renderingInfo" {
        "<hp:renderingInfo/>".to_string()
    } else {
        "<hp:renderingInfo>\
<hc:transMatrix e1=\"1\" e2=\"0\" e3=\"0\" e4=\"0\" e5=\"1\" e6=\"0\"/>\
<hc:scaMatrix e1=\"1\" e2=\"0\" e3=\"0\" e4=\"0\" e5=\"1\" e6=\"0\"/>\
<hc:rotMatrix e1=\"1\" e2=\"0\" e3=\"0\" e4=\"0\" e5=\"1\" e6=\"0\"/>\
</hp:renderingInfo>"
            .to_string()
    };
    // 한컴 저장본은 채우기 없는 도형에서 `<hp:fillBrush>` 를 아예 생략하지만,
    // 다른 생성계는 빈 요소로 낸다. 두 표기 모두 "채우기 없음"이다.
    let fill = if quirk == "fillBrush" {
        "<hp:fillBrush/>"
    } else {
        ""
    };
    let draw = if quirk == "drawText" {
        "<hp:drawText lastWidth=\"5000\" name=\"\" editable=\"0\"/>".to_string()
    } else {
        let inner_p = if quirk == "emptyPara" {
            format!(
                "<hp:p paraPrIDRef=\"0\" styleIDRef=\"0\"/>\
<hp:p paraPrIDRef=\"0\" styleIDRef=\"0\"><hp:run charPrIDRef=\"0\"><hp:t>{text}</hp:t></hp:run></hp:p>"
            )
        } else {
            format!(
                "<hp:p paraPrIDRef=\"0\" styleIDRef=\"0\"><hp:run charPrIDRef=\"0\"><hp:t>{text}</hp:t></hp:run></hp:p>"
            )
        };
        format!(
            "<hp:drawText lastWidth=\"5000\" name=\"\" editable=\"0\">\
<hp:subList vertAlign=\"CENTER\">{inner_p}</hp:subList>\
<hp:textMargin left=\"0\" right=\"0\" top=\"0\" bottom=\"0\"/>\
</hp:drawText>"
        )
    };
    format!(
        "<hp:rect id=\"0\" zOrder=\"0\" groupLevel=\"1\" instid=\"{instid}\" ratio=\"0\">\
<hp:offset x=\"0\" y=\"0\"/><hp:orgSz width=\"5000\" height=\"2000\"/>\
<hp:curSz width=\"0\" height=\"0\"/><hp:flip horizontal=\"0\" vertical=\"0\"/>\
<hp:rotationInfo angle=\"0\" centerX=\"0\" centerY=\"0\" rotateimage=\"1\"/>\
{rendering}\
<hp:lineShape color=\"#000000\" width=\"33\" style=\"SOLID\"/>{fill}\
<hp:shadow type=\"NONE\" color=\"#B2B2B2\" offsetX=\"0\" offsetY=\"0\" alpha=\"0\"/>\
{draw}\
<hc:pt0 x=\"0\" y=\"0\"/><hc:pt1 x=\"5000\" y=\"0\"/>\
<hc:pt2 x=\"5000\" y=\"2000\"/><hc:pt3 x=\"0\" y=\"2000\"/></hp:rect>"
    )
}

/// 업무체계도처럼 `<hp:container>` 안에 글상자 도형 3개를 나란히 둔 구역 XML.
/// 첫 도형에만 `quirk` 표기를 준다.
fn section_xml(quirk: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0">
    <hp:container id="10" zOrder="0" groupLevel="0" instid="100">
      <hp:offset x="0" y="0"/><hp:orgSz width="20000" height="10000"/>
      <hp:curSz width="0" height="0"/><hp:flip horizontal="0" vertical="0"/>
      <hp:rotationInfo angle="0" centerX="0" centerY="0" rotateimage="1"/>
      <hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/><hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/><hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo>
      {first}{second}{third}
    </hp:container>
    <hp:t/>
  </hp:run></hp:p>
</hs:sec>"#,
        first = rect(101, "검토/승인", quirk),
        second = rect(102, "개발/양산", "none"),
        third = rect(103, "CSRP정보", "none"),
    )
}

/// 구역을 파싱해 묶음 자식 도형의 (instid, 글상자 문단 텍스트) 목록을 낸다.
fn group_children(quirk: &str) -> Vec<(u32, Option<String>)> {
    let section = parse_hwpx_section(&section_xml(quirk))
        .unwrap_or_else(|e| panic!("[{quirk}] HWPX 구역 파싱 실패: {e:?}"));
    let control = section.paragraphs[0]
        .controls
        .first()
        .unwrap_or_else(|| panic!("[{quirk}] 묶음 개체가 파싱되지 않았다"));
    let Control::Shape(shape) = control else {
        panic!("[{quirk}] 첫 컨트롤이 도형이 아니다");
    };
    let ShapeObject::Group(group) = shape.as_ref() else {
        panic!("[{quirk}] 첫 컨트롤이 묶음(container) 이 아니다");
    };
    group
        .children
        .iter()
        .map(|child| {
            let text = child.drawing().and_then(|drawing| {
                drawing.text_box.as_ref().map(|tb| {
                    tb.paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
            });
            (child.common().instance_id, text)
        })
        .collect()
}

/// 정상 표기 기준선 — 도형 3개가 각자의 글을 그대로 갖는다.
#[test]
fn issue5797_baseline_keeps_every_textbox() {
    assert_eq!(
        group_children("none"),
        vec![
            (101, Some("검토/승인".to_string())),
            (102, Some("개발/양산".to_string())),
            (103, Some("CSRP정보".to_string())),
        ]
    );
}

/// `<hp:fillBrush/>` — 종전엔 첫 도형이 글상자를 잃고 뒤 형제 도형 2개를 삼켰다.
#[test]
fn issue5797_self_closing_fill_brush_keeps_siblings_and_text() {
    assert_eq!(
        group_children("fillBrush"),
        vec![
            (101, Some("검토/승인".to_string())),
            (102, Some("개발/양산".to_string())),
            (103, Some("CSRP정보".to_string())),
        ],
        "자기닫힘 <hp:fillBrush/> 뒤의 <hp:drawText> 와 형제 도형이 삼켜졌다"
    );
}

/// `<hp:renderingInfo/>` — 종전엔 첫 도형이 형제의 글을 훔치고 그 형제가 사라졌다.
#[test]
fn issue5797_self_closing_rendering_info_keeps_siblings_and_text() {
    assert_eq!(
        group_children("renderingInfo"),
        vec![
            (101, Some("검토/승인".to_string())),
            (102, Some("개발/양산".to_string())),
            (103, Some("CSRP정보".to_string())),
        ],
        "자기닫힘 <hp:renderingInfo/> 가 뒤 형제 도형을 삼켰다"
    );
}

/// `<hp:drawText/>` — 글 없는 빈 글상자다. 형제의 글을 끌어오면 안 된다.
#[test]
fn issue5797_self_closing_draw_text_is_an_empty_text_box() {
    assert_eq!(
        group_children("drawText"),
        vec![
            (101, Some(String::new())),
            (102, Some("개발/양산".to_string())),
            (103, Some("CSRP정보".to_string())),
        ],
        "자기닫힘 <hp:drawText/> 가 뒤 형제 도형의 글을 끌어왔다"
    );
}

/// `<hp:p/>` — 내용 없는 문단이다. 뒤 문단의 글을 삼키면 안 된다.
#[test]
fn issue5797_self_closing_paragraph_does_not_swallow_next_paragraph() {
    assert_eq!(
        group_children("emptyPara"),
        vec![
            (101, Some("검토/승인".to_string())),
            (102, Some("개발/양산".to_string())),
            (103, Some("CSRP정보".to_string())),
        ],
        "자기닫힘 <hp:p/> 가 뒤 문단을 삼켰다"
    );
}
