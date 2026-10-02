//! [Issue #5820] 부분 문단으로 시작하는 둘째 쪽에서 본문과 로고 글상자가
//! 같은 쪽에 남고, 글상자가 본문 뒤·바닥글 앞에 놓이는지 검사한다.
//! 저장 사다리에서 추정한 절대 y 값은 한컴 2022 PDF의 실제 위치와 달랐다.
//! 두 쪽의 Native 시각 비교가 각각 96.60%, 98.18%인 상태에서 배치 관계로
//! 기존 검사를 교체했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const SAMPLE: &str = "samples/issue5820/156560092_ecard_meeting_press.hwpx";

#[test]
fn issue_5820_partial_page_start_keeps_body_and_logo_frame_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("열기");
    assert_eq!(core.page_count(), 2, "한컴 기준 PDF는 두 쪽이다");

    let tree = core.build_page_render_tree(1).expect("둘째 쪽 배치");
    let page: Value = serde_json::from_str(&tree.root.to_json()).expect("배치 트리 JSON");
    let body = find_node(&page, "Body").expect("둘째 쪽 본문");
    let footer = find_node(&page, "Footer").expect("둘째 쪽 바닥글");
    let last_line = find_node_by_text(body, "할 수 있도록 최선의 지원을 다하겠다.")
        .expect("이어지는 본문의 마지막 문장");
    let frame = find_node(body, "Rect").expect("본문 뒤 로고 글상자 프레임");
    let text_box = find_node(body, "TextBox").expect("로고 글상자 내용");

    let (_, line_y, _, line_h) = bbox(last_line);
    let (frame_x, frame_y, frame_w, frame_h) = bbox(frame);
    let (box_x, box_y, box_w, box_h) = bbox(text_box);
    let (_, footer_y, _, _) = bbox(footer);
    assert!(
        line_y + line_h < frame_y && frame_y + frame_h < footer_y,
        "본문의 마지막 문장 → 로고 프레임 → 바닥글 순서가 유지되어야 한다"
    );
    assert!(
        frame_x <= box_x
            && frame_y <= box_y
            && box_x + box_w <= frame_x + frame_w
            && box_y + box_h <= frame_y + frame_h,
        "로고 내용은 글상자 프레임 안에 있어야 한다"
    );
}

fn find_node<'a>(node: &'a Value, ty: &str) -> Option<&'a Value> {
    if node["type"] == ty {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|child| find_node(child, ty))
}

fn find_node_by_text<'a>(node: &'a Value, text: &str) -> Option<&'a Value> {
    if node["type"] == "TextRun" && node["text"].as_str()?.contains(text) {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|child| find_node_by_text(child, text))
}

fn bbox(node: &Value) -> (f64, f64, f64, f64) {
    let b = &node["bbox"];
    (
        b["x"].as_f64().expect("x"),
        b["y"].as_f64().expect("y"),
        b["w"].as_f64().expect("너비"),
        b["h"].as_f64().expect("높이"),
    )
}
