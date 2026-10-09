#![cfg(not(target_arch = "wasm32"))]
//! 스타일 모양 고치기(`updateStyleShapes`)는 문단 모양의 탭·테두리/배경·테두리 간격과 글자
//! 모양의 테두리/배경도 스타일에 넣는다. 그 스타일을 쓰는 문단도 같이 바뀌고, HWP·HWPX로
//! 저장해 다시 열어도 남는다.
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// 새 문서의 첫 문단(바탕글)에 글을 넣는다.
fn blank_with(text: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, text).unwrap();
    doc
}

fn caret_x(doc: &HwpDocument, offset: u32) -> f64 {
    parse(&doc.get_cursor_rect(0, 0, offset).unwrap())["x"]
        .as_f64()
        .unwrap()
}

/// 바탕글에 왼쪽 탭 하나를 둔다. 탭 위치는 문단 여백처럼 두 배 단위로 저장하므로 30000은
/// 15000HU, 곧 200px이다.
#[test]
fn a_style_tab_stop_moves_the_text_after_the_tab() {
    let mut doc = blank_with("가\t나");
    let stops = json!([{"position": 30000, "type": 0, "fill": 0}]);
    let para_mods = json!({ "tabStops": stops }).to_string();
    assert!(doc.update_style_shapes(0, "{}", &para_mods));

    let check = |doc: &HwpDocument, label: &str| {
        let gap = caret_x(doc, 2) - caret_x(doc, 0);
        assert!(
            (gap - 200.0).abs() < 0.5,
            "{label}: 탭 뒤 글자가 문단 시작에서 {gap}px 떨어져 있다"
        );
        let style = parse(&doc.get_style_detail(0));
        assert_eq!(style["paraProps"]["tabStops"], stops, "{label}: 스타일");
        let para = parse(&doc.get_para_properties_at(0, 0).unwrap());
        assert_eq!(para["tabStops"], stops, "{label}: 문단");
    };
    check(&doc, "편집 직후");
    check(
        &HwpDocument::new(&doc.export_hwp().unwrap()).unwrap(),
        "HWP 저장본",
    );
    check(
        &HwpDocument::new(&doc.export_hwpx().unwrap()).unwrap(),
        "HWPX 저장본",
    );
}

/// 바탕글의 글자·문단 모양에 이중선 테두리와 배경색을, 문단 모양에 테두리 간격을 준다.
#[test]
fn style_borders_fills_and_spacing_reach_the_style_and_its_paragraph() {
    let mut doc = blank_with("가나다");
    let double = json!({"type": 8, "width": 1, "color": "#0000ff"});
    let borders = |fill: &str| {
        json!({
            "borderLeft": double, "borderRight": double,
            "borderTop": double, "borderBottom": double,
            "fillType": "solid", "fillColor": fill,
        })
    };
    let char_mods = borders("#ffff00");
    let mut para_mods = borders("#00ff00");
    para_mods["borderSpacing"] = json!([100, 200, 300, 400]);
    assert!(doc.update_style_shapes(0, &char_mods.to_string(), &para_mods.to_string()));

    let check = |doc: &HwpDocument, label: &str| {
        let style = parse(&doc.get_style_detail(0));
        let para = parse(&doc.get_para_properties_at(0, 0).unwrap());
        let chars = parse(&doc.get_char_properties_at(0, 0, 1).unwrap());
        for (props, fill) in [
            (&style["charProps"], "#ffff00"),
            (&chars, "#ffff00"),
            (&style["paraProps"], "#00ff00"),
            (&para, "#00ff00"),
        ] {
            for side in ["borderLeft", "borderRight", "borderTop", "borderBottom"] {
                assert_eq!(props[side]["type"], 8, "{label}: {side}");
            }
            assert_eq!(props["fillColor"], fill, "{label}: 배경색");
        }
        for props in [&style["paraProps"], &para] {
            assert_eq!(
                props["borderSpacing"],
                json!([100, 200, 300, 400]),
                "{label}"
            );
        }
    };
    check(&doc, "편집 직후");
    check(
        &HwpDocument::new(&doc.export_hwp().unwrap()).unwrap(),
        "HWP 저장본",
    );
    check(
        &HwpDocument::new(&doc.export_hwpx().unwrap()).unwrap(),
        "HWPX 저장본",
    );
}
