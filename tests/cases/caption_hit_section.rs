//! 고정형 회귀 — 둘째 구역 표 캡션의 누르기 결과는 구역 1과 캡션 셀 주소 65534를 반환한다.
//! 주석 갱신 2026-11-28T07:34:34+09:00. 직전 파일 커밋 2026-11-28T07:20:16+09:00 4859a7099ff27b9e1b0118bc4e4151d1b7ca14df.
//! 입력: 둘째 구역에 표 캡션을 추가한 합성 문서.
//! 누르기 API가 돌려주는 소유 주소를 검증하므로 시각 일치율은 해당하지 않는다. 근거: mydocs/pr/archives/pr_9476_review.md.
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
