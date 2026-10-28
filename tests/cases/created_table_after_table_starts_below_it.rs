#![cfg(not(target_arch = "wasm32"))]
//! 새 문서에서 표 바로 아래 문단에 표를 하나 더 만들면 둘째 표는 첫 표 아래에서 시작한다.
//!
//! 표 생성의 vpos 재계산은 표 호스트 문단을 한 줄 높이(1000 + 600HU)로만 잇는다. 2×2 표의
//! 본체는 2564HU라서, 이 사다리를 저장 프레임 원점으로 읽으면 둘째 표가 첫 표의 둘째 행 위에
//! 놓인다. 그 사다리를 그대로 저장한 HWP·HWPX를 다시 열어도 같아야 한다.
use rhwp::wasm_api::HwpDocument;

fn table_span(doc: &HwpDocument, para: u32) -> (f64, f64) {
    let bbox: serde_json::Value =
        serde_json::from_str(&doc.get_table_bbox(0, para, 0).unwrap()).unwrap();
    let top = bbox["y"].as_f64().unwrap();
    (top, top + bbox["height"].as_f64().unwrap())
}

fn assert_below(doc: &HwpDocument, upper: u32, lower: u32) {
    let (_, upper_bottom) = table_span(doc, upper);
    let (lower_top, _) = table_span(doc, lower);
    assert!(
        lower_top >= upper_bottom,
        "문단 {lower}의 표 위끝 {lower_top}px가 문단 {upper}의 표 아래끝 {upper_bottom}px보다 위다"
    );
}

/// 머리 글 아래 문단 1에 2×2 표를 만든다. 표 아래에 빈 문단 2가 생긴다.
fn blank_with_table() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "머리").unwrap();
    doc.split_paragraph(0, 0, 2, None).unwrap();
    doc.create_table(0, 1, 0, 2, 2).unwrap();
    doc
}

#[test]
fn a_table_created_right_below_a_table_starts_below_it() {
    let mut doc = blank_with_table();
    doc.create_table(0, 2, 0, 1, 2).unwrap();
    assert_below(&doc, 1, 2);
    for bytes in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
        assert_below(&HwpDocument::new(&bytes).unwrap(), 1, 2);
    }
}

#[test]
fn a_table_after_a_text_line_below_a_table_starts_below_the_text() {
    let mut doc = blank_with_table();
    doc.insert_text(0, 2, 0, "사이 글").unwrap();
    doc.split_paragraph(0, 2, 4, None).unwrap();
    doc.create_table(0, 3, 0, 2, 2).unwrap();
    let line: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect(0, 2, 0).unwrap()).unwrap();
    let line_bottom = line["y"].as_f64().unwrap() + line["height"].as_f64().unwrap();
    let (table_top, _) = table_span(&doc, 3);
    assert!(
        table_top >= line_bottom,
        "둘째 표 위끝 {table_top}px가 사이 글 아래끝 {line_bottom}px보다 위다"
    );
}
