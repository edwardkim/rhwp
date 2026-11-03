//! 머리말 틀을 넘친 문단도 한컴처럼 본문 위에 겹쳐 그린다.
//! 한컴 2024 확인: 70자 두 줄·빈 문단 뒤의 "마지막"이 본문 첫 줄과 겹쳐 보이고 본문은 밀리지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn run_y(core: &DocumentCore, text: &str) -> Option<f64> {
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run["text"] == text)
        .and_then(|run| run["y"].as_f64())
}

#[test]
fn header_paragraph_past_the_header_area_is_drawn_over_the_body() {
    for bytes in [
        &include_bytes!("../fixtures/header_overflow/header-overflow.hwp")[..],
        &include_bytes!("../fixtures/header_overflow/header-overflow.hwpx")[..],
    ] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        let body = run_y(&core, "본문").expect("본문 첫 줄");
        let overflow = run_y(&core, "마지막").expect("넘친 머리말 문단도 그린다");
        // 본문을 밀지 않고 그 위에 겹친다.
        assert!(overflow >= body - 1.0, "{overflow} < {body}");
        assert!(core.render_page_svg_native(0).unwrap().contains(">마<"));
    }
}
