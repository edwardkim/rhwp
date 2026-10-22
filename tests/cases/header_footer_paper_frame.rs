#![cfg(not(target_arch = "wasm32"))]
//! 머리말·꼬리말에 놓인 용지·쪽 기준 표와 글상자는 그 머리말·꼬리말 틀에서 잰다(그림은
//! #6608). 틀은 본문 왼쪽 끝·본문 폭, 세로는 머리말이면 위 여백에서 머리말 여백까지,
//! 꼬리말이면 본문 아래끝에서 꼬리말 여백까지다.
//!
//! 표본은 A4(위 21mm·머리말 12mm·꼬리말 5mm·아래 52mm) 한 쪽이다(한글 2024 저장, 글은
//! 채움 글자). 머리말에는 용지 기준으로 위 여백만큼 위로 올린 1칸 표(교재의 장 머리말과
//! 같은 배치), 꼬리말 문단에는 아래·오른쪽 정렬 1칸 표와 아래·왼쪽 정렬 사각형이 있다.
//! 한글 2024 Print PDF: 머리말 표 (본문 왼쪽, 0), 꼬리말 표와 사각형의 아래끝은 본문
//! 아래끝 + 꼬리말 여백. 고치기 전 rhwp 는 머리말 표를 용지 원점(0, −79.4)에, 꼬리말 표를
//! 용지 오른쪽 아래 구석에, 사각형을 아래 여백 끝에 그렸다.
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/header-footer-paper-frame.hwpx";

fn bbox(node: &Value) -> (f64, f64, f64, f64) {
    let b = &node["bbox"];
    (
        b["x"].as_f64().unwrap(),
        b["y"].as_f64().unwrap(),
        b["w"].as_f64().unwrap(),
        b["h"].as_f64().unwrap(),
    )
}

fn first<'a>(node: &'a Value, kind: &str) -> Option<&'a Value> {
    if node["type"] == kind {
        return Some(node);
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| first(child, kind))
}

fn area<'a>(root: &'a Value, kind: &str) -> &'a Value {
    root["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|child| child["type"] == kind)
        .unwrap_or_else(|| panic!("{kind} 영역"))
}

fn px(hwpunit: u32) -> f64 {
    f64::from(hwpunit) * 96.0 / 7200.0
}

#[test]
fn header_and_footer_floats_are_measured_from_their_frames() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(doc.page_count(), 1, "독립 한글 출력과 같은 쪽 수");
    let page_def = &doc.document().sections[0].section_def.page_def;

    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let body = bbox(area(&tree, "Body"));
    let body_bottom = body.1 + body.3;

    // 머리말 표: 용지 기준 세로 오프셋이 −위 여백이라 틀(위 여백) 원점에서 재면 종이 위끝.
    let header = bbox(first(area(&tree, "Header"), "Table").expect("머리말 표"));
    assert!(
        (header.0 - body.0).abs() < 0.5,
        "머리말 표 왼쪽 {:.1} 이 본문 왼쪽 {:.1} 이다",
        header.0,
        body.0
    );
    assert!(
        header.1.abs() < 0.5,
        "머리말 표 위끝 {:.1} 이 종이 위끝 0 이다",
        header.1
    );

    // 꼬리말 틀의 아래끝은 꼬리말 여백까지다(아래 여백 끝이 아니다).
    let band_bottom = body_bottom + px(page_def.margin_footer);
    let footer = area(&tree, "Footer");
    let table = bbox(first(footer, "Table").expect("꼬리말 표"));
    assert!(
        (table.0 + table.2 - (body.0 + body.2)).abs() < 0.5,
        "꼬리말 표 오른쪽 {:.1} 이 본문 오른쪽 {:.1} 이다",
        table.0 + table.2,
        body.0 + body.2
    );
    let table_declared_bottom = table.1 + px(1200);
    assert!(
        (table_declared_bottom - band_bottom).abs() < 0.5,
        "꼬리말 표(선언 높이 1200) 아래끝 {table_declared_bottom:.1} 이 꼬리말 틀 아래끝 {band_bottom:.1} 이다"
    );

    let rect = bbox(first(footer, "Rect").expect("꼬리말 사각형"));
    assert!(
        (rect.0 - body.0).abs() < 0.5,
        "꼬리말 사각형 왼쪽 {:.1} 이 본문 왼쪽 {:.1} 이다",
        rect.0,
        body.0
    );
    assert!(
        (rect.1 + rect.3 - band_bottom).abs() < 0.5,
        "꼬리말 사각형 아래끝 {:.1} 이 꼬리말 틀 아래끝 {band_bottom:.1} 이다",
        rect.1 + rect.3
    );
}
