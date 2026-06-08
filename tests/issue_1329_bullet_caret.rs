//! Issue #1329: 글머리표/번호 문단에서 Enter 직후 빈 list 줄의 caret 이
//! marker 앞쪽으로 되감기지 않아야 한다.
//!
//! 번호/글머리표 marker 는 문서 문자 좌표에 포함되지 않는 `char_start: None`
//! TextRun 이다. 문단 끝 Enter 로 생성된 빈 list 문단에서도 offset 0 caret 은
//! marker 시작점이 아니라 marker 뒤 본문 시작점에 있어야 한다.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load_doc(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

fn cursor_x(doc: &HwpDocument, para: usize, offset: usize) -> f64 {
    let json = doc
        .get_cursor_rect_native(0, para, offset)
        .unwrap_or_else(|e| panic!("cursor rect para={para} offset={offset}: {e:?}"));
    let rect: Value =
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse cursor rect `{json}`: {e}"));
    rect["x"].as_f64().expect("cursor x")
}

fn paragraph_len(doc: &HwpDocument, para: usize) -> usize {
    doc.get_paragraph_length_native(0, para)
        .unwrap_or_else(|e| panic!("paragraph length para={para}: {e:?}"))
}

fn assert_caret_x_matches_body_start(actual: f64, expected: f64, context: &str) {
    assert!(
        (actual - expected).abs() <= 1.0,
        "{context}: empty list caret x must stay at body start, expected {expected:.1}, got {actual:.1}"
    );
}

#[test]
fn issue_1329_bullet_enter_empty_line_caret_stays_after_marker() {
    let mut doc = load_doc("rhwp-studio/public/samples/number-bullet.hwp");
    let split_para = 1;
    let split_offset = paragraph_len(&doc, split_para);

    doc.split_paragraph_native(0, split_para, split_offset)
        .expect("split bullet paragraph");

    let new_para = split_para + 1;
    let empty_caret_x = cursor_x(&doc, new_para, 0);
    doc.insert_text_native(0, new_para, 0, "가")
        .expect("insert text into empty bullet paragraph");
    let typed_body_start_x = cursor_x(&doc, new_para, 0);
    assert_caret_x_matches_body_start(empty_caret_x, typed_body_start_x, "bullet paragraph split");
}

#[test]
fn issue_1329_number_enter_empty_line_caret_stays_after_marker() {
    let mut doc = load_doc("rhwp-studio/public/samples/para-head-num-2.hwp");
    let split_para = 1;
    let split_offset = paragraph_len(&doc, split_para);

    doc.split_paragraph_native(0, split_para, split_offset)
        .expect("split numbered paragraph");

    let new_para = split_para + 1;
    let empty_caret_x = cursor_x(&doc, new_para, 0);
    doc.insert_text_native(0, new_para, 0, "가")
        .expect("insert text into empty numbered paragraph");
    let typed_body_start_x = cursor_x(&doc, new_para, 0);
    assert_caret_x_matches_body_start(empty_caret_x, typed_body_start_x, "number paragraph split");
}

#[test]
fn issue_1329_plain_empty_paragraph_caret_keeps_original_start() {
    let mut doc = load_doc("saved/blank2010.hwp");
    doc.convert_to_editable_native()
        .expect("convert blank document to editable");
    doc.insert_text_native(0, 0, 0, "테스트")
        .expect("insert text");

    let expected_x = cursor_x(&doc, 0, 0);
    doc.split_paragraph_native(0, 0, 3)
        .expect("split plain paragraph");
    let actual_x = cursor_x(&doc, 1, 0);

    assert!(
        (actual_x - expected_x).abs() <= 1.0,
        "plain empty paragraph caret should keep the normal paragraph start, expected {expected_x:.1}, got {actual_x:.1}"
    );
}
