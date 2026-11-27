//! 머리말 글의 덧말·글자 겹치기는 선택·누르기에서도 한 칸이다(머리말 캐럿·본문과 같은 자리 축).
//! 전에는 선택 조회가 범위를 글자 수로 검사해 끝 덧말·겹침 뒤까지 고르면 "문자 범위 초과"로 던졌고,
//! 누르기는 끝 덧말 run을 본말 글자 수로 세어 문단 끝을 넘는 자리를 줬다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::ComposeHost;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// 머리말 "머리글" 뒤에 겹침 "12"와 덧말 "漢字"를 차례로 둔다. 자리는 글자 3 + 개체 2 = 5칸이다.
fn header_ending_with_compose_controls() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.create_header_footer_native(0, true, 0).unwrap();
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리글")
        .unwrap();
    let info = json(&doc.get_header_footer_native(0, true, 0).unwrap());
    let host = info["paraIndex"].as_u64().unwrap() as usize;
    let control = info["controlIndex"].as_u64().unwrap() as usize;
    let path = [(control, 0, 0)];
    let header = ComposeHost::Cell {
        sec: 0,
        parent_para: host,
        path: &path,
    };
    doc.insert_char_overlap_at(header, 3, r#"{"text":"12","shape":5}"#)
        .unwrap();
    doc.insert_ruby_at(header, 4, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    doc
}

#[test]
fn header_selection_reaches_past_trailing_ruby_and_overlap() {
    let doc = header_ending_with_compose_controls();
    let rects = json(
        &doc.get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 0, 5)
            .unwrap(),
    );
    let rects = rects.as_array().unwrap();
    assert!(!rects.is_empty());
    let right = rects
        .iter()
        .map(|r| r["x"].as_f64().unwrap() + r["width"].as_f64().unwrap())
        .fold(f64::MIN, f64::max);
    // 끝 덧말 뒤 캐럿까지 칠한다.
    let end = json(
        &doc.get_cursor_rect_in_header_footer_native(0, true, 0, 0, 5, 0)
            .unwrap(),
    );
    assert!(
        (right - end["x"].as_f64().unwrap()).abs() < 1.0,
        "선택 {rects:?} / 끝 캐럿 {end}"
    );
}

#[test]
fn header_click_after_trailing_ruby_lands_on_paragraph_end() {
    let doc = header_ending_with_compose_controls();
    let start = json(
        &doc.get_cursor_rect_in_header_footer_native(0, true, 0, 0, 0, 0)
            .unwrap(),
    );
    let end = json(
        &doc.get_cursor_rect_in_header_footer_native(0, true, 0, 0, 5, 0)
            .unwrap(),
    );
    // 줄 끝 덧말 뒤 빈 곳을 누른다.
    let x = end["x"].as_f64().unwrap() + 40.0;
    let y = start["y"].as_f64().unwrap() + start["height"].as_f64().unwrap() / 2.0;
    let hit = json(&doc.hit_test_in_header_footer_native(0, true, x, y).unwrap());
    assert_eq!(hit["paraIndex"], 0, "{hit}");
    assert_eq!(hit["charOffset"], 5, "{hit}");
}
