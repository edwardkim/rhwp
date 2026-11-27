//! 둘째 구역 표 캡션 글을 누르면 그 구역 주소를 받는다.
//! 전에는 캡션 글줄을 구역 0의 글로 조판해, 히트가 첫 구역의 같은 문단·개체 번호를 가리켰다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const CAPTION: usize = 65534;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// 그 구역 첫 문단에 1×2 표를 만들고 아래 캡션 글 앞에 `text`를 넣는다. (문단, 개체) 번호를 준다.
fn captioned_table(doc: &mut HwpDocument, sec: usize, text: &str) -> (usize, usize) {
    let table = json(&doc.create_table_native(sec, 0, 0, 1, 2).unwrap());
    let (para, control) = (
        table["paraIdx"].as_u64().unwrap() as usize,
        table["controlIdx"].as_u64().unwrap() as usize,
    );
    doc.set_table_properties_native(
        sec,
        para,
        control,
        r#"{"hasCaption":true,"captionDirection":3}"#,
    )
    .unwrap();
    doc.insert_text_in_cell_native(sec, para, control, CAPTION, 0, 0, text)
        .unwrap();
    (para, control)
}

#[test]
fn caption_hit_in_second_section_reports_that_section() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "가").unwrap();
    doc.split_paragraph_native(0, 0, 1, None).unwrap();
    doc.split_section_native(0, 1, 0).unwrap();
    let second = captioned_table(&mut doc, 1, "CD");

    let rect = json(
        &doc.get_cursor_rect_in_cell_native(1, second.0, second.1, CAPTION, 0, 1)
            .unwrap(),
    );
    let page = rect["pageIndex"].as_u64().unwrap() as u32;
    let (x, y) = (
        rect["x"].as_f64().unwrap(),
        rect["y"].as_f64().unwrap() + rect["height"].as_f64().unwrap() / 2.0,
    );
    let hit = json(&doc.hit_test_native(page, x, y).unwrap());
    assert_eq!(hit["sectionIndex"], 1, "{hit}");
    assert_eq!(hit["cellIndex"], CAPTION as u64, "{hit}");
}
