//! [#7444] 각주 뒤 캐럿 위치(논리 오프셋)에서 입력·복사하면 각주 앞뒤가 뒤바뀌지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

/// 본문 첫 문단을 `text` 로 채우고 `footnote_at` 글자 자리에 각주를 넣는다.
fn doc_with_footnote(text: &str, footnote_at: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.insert_text_native(0, 0, 0, text).expect("본문 입력");
    doc.insert_footnote_native(0, 0, footnote_at)
        .expect("각주 삽입");
    doc
}

/// 첫 문단의 글자와 각주들의 글자 자리.
fn text_and_footnotes(doc: &HwpDocument) -> (String, Vec<usize>) {
    let para = &doc.document().sections[0].paragraphs[0];
    let positions = para.control_text_positions();
    let footnotes = para
        .controls
        .iter()
        .zip(positions)
        .filter(|(ctrl, _)| matches!(ctrl, Control::Footnote(_)))
        .map(|(_, pos)| pos)
        .collect();
    (para.text.clone(), footnotes)
}

#[test]
fn insert_text_logical_after_footnote_marker_goes_after_marker() {
    // 앞[각주]뒤 — 논리 오프셋 2 는 각주 바로 뒤, `뒤` 앞이다.
    let mut doc = doc_with_footnote("앞뒤", 1);
    let result = doc.insert_text_logical(0, 0, 2, "가").expect("입력");
    assert_eq!(text_and_footnotes(&doc), ("앞가뒤".to_string(), vec![1]));
    assert!(result.contains("\"logicalOffset\":3"), "{result}");

    // 문단 끝 각주 뒤: 앞[각주] → 앞[각주]가
    let mut doc = doc_with_footnote("앞", 1);
    doc.insert_text_logical(0, 0, 2, "가").expect("입력");
    assert_eq!(text_and_footnotes(&doc), ("앞가".to_string(), vec![1]));

    // 각주 앞 입력은 그대로 각주 앞에 들어간다.
    let mut doc = doc_with_footnote("앞뒤", 1);
    doc.insert_text_logical(0, 0, 1, "가").expect("입력");
    assert_eq!(text_and_footnotes(&doc), ("앞가뒤".to_string(), vec![2]));
}

#[test]
fn copy_selection_logical_after_footnote_leaves_footnote_out() {
    // AB[각주]CD 에서 각주 뒤 CD(논리 3..5)를 복사해 문단 끝(글자 위치 4)에 붙인다.
    let mut doc = doc_with_footnote("ABCD", 2);
    let copied = doc.copy_selection_logical(0, 0, 3, 0, 5).expect("복사");
    assert!(copied.contains("\"text\":\"CD\""), "{copied}");
    doc.paste_internal_native(0, 0, 4).expect("붙여넣기");
    assert_eq!(text_and_footnotes(&doc), ("ABCDCD".to_string(), vec![2]));

    // 각주를 덮는 선택(각주 앞에서 시작, 각주 뒤에서 끝)은 각주를 담는다.
    // 개체가 든 클립보드는 붙일 위치를 개체까지 한 칸으로 세므로, 두 셈이 같은 맨 앞에 붙인다.
    for (start, end) in [(2, 5), (0, 3)] {
        let mut doc = doc_with_footnote("ABCD", 2);
        doc.copy_selection_logical(0, 0, start, 0, end)
            .expect("복사");
        doc.paste_internal_native(0, 0, 0).expect("붙여넣기");
        assert_eq!(text_and_footnotes(&doc).1.len(), 2, "{start}..{end}");
    }
}

#[test]
fn copy_selection_logical_to_paragraph_end_keeps_trailing_floating_shape() {
    // AB 끝(글자 위치 2)에 글자처럼 취급하지 않는 사각형을 둔다. 이 도형은 논리 칸이 없어
    // 문단 논리 길이가 2 다. 0..2 는 문단 전체 선택이므로 copySelection 처럼 도형을 담는다.
    let mut doc = HwpDocument::create_empty();
    doc.insert_text_native(0, 0, 0, "AB").expect("본문 입력");
    doc.create_shape_control_native(
        0,
        0,
        2,
        9000,
        6750,
        0,
        0,
        false,
        "Square",
        "rectangle",
        false,
        false,
        &[],
    )
    .expect("도형 삽입");
    let shapes = |doc: &HwpDocument| {
        doc.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(|ctrl| matches!(ctrl, Control::Shape(_)))
            .count()
    };
    assert_eq!(shapes(&doc), 1);

    doc.copy_selection_logical(0, 0, 0, 0, 2).expect("복사");
    doc.paste_internal_native(0, 0, 2).expect("붙여넣기");
    assert_eq!(shapes(&doc), 2);
}
