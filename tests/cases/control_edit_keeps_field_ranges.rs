//! 문단 컨트롤을 넣거나 지우는 편집은 뒤 누름틀·코드의 컨트롤 번호(`field_ranges[].control_idx`)를
//! 함께 옮긴다. 옛 번호에 남으면 다른 컨트롤을 가리켜 그 필드가 목록과 저장에서 사라진다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn reopen_both(doc: &HwpDocument) -> Vec<HwpDocument> {
    [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ]
    .iter()
    .map(|bytes| HwpDocument::from_bytes(bytes).unwrap())
    .collect()
}

/// 본문 누름틀의 (이름, 안내문).
fn click_here(doc: &HwpDocument) -> Vec<(String, String)> {
    let fields: Vec<Value> = serde_json::from_str(&doc.get_field_list_json()).unwrap();
    fields
        .iter()
        .filter(|f| f["fieldType"] == "clickhere")
        .map(|f| {
            (
                f["name"].as_str().unwrap_or_default().to_string(),
                f["guide"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

fn assert_kept(doc: &HwpDocument) {
    let expected = vec![("남을 필드".to_string(), "안내문".to_string())];
    assert_eq!(click_here(doc), expected, "편집 중");
    for reopened in reopen_both(doc) {
        assert_eq!(click_here(&reopened), expected, "저장 뒤");
    }
}

fn doc_with_field(text: &str, field_at: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, text).unwrap();
    doc.insert_click_here_field_at(0, 0, field_at, "안내문", "메모", "남을 필드", true)
        .unwrap();
    doc
}

#[test]
fn deleting_a_footnote_keeps_the_following_field() {
    let mut doc = doc_with_field("앞 각주 뒤", 5);
    doc.insert_footnote_native(0, 0, 1).unwrap();
    assert_eq!(click_here(&doc).len(), 1);
    let footnote = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, rhwp::model::control::Control::Footnote(_)))
        .unwrap();
    doc.delete_footnote_native(0, 0, footnote).unwrap();
    assert_kept(&doc);
}

#[test]
fn page_hide_on_and_off_keeps_the_field() {
    let mut doc = doc_with_field("감추기 문단", 2);
    doc.set_page_hide_native(0, 0, true, false, false, false, false, false)
        .unwrap();
    assert_kept(&doc);
    doc.set_page_hide_native(0, 0, false, false, false, false, false, false)
        .unwrap();
    assert_kept(&doc);
}
