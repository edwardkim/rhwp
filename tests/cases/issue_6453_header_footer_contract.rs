//! [#6453] 머리말/꼬리말 편집 resolver와 원자 text event 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

fn header_with_text(text: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank2010 생성");
    doc.create_header_footer_native(0, true, 0)
        .expect("양쪽 머리말 생성");
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, text)
        .expect("머리말 입력");
    doc
}

fn only_event(raw: &str) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(raw).expect("event log JSON");
    let events = value["events"].as_array().expect("events array");
    assert_eq!(
        events.len(),
        1,
        "text mutation은 원자 event 하나여야 함: {raw}"
    );
    events[0].clone()
}

fn assert_range_event(
    event: &serde_json::Value,
    start: (usize, usize),
    end: (usize, usize),
    inserted_end: (usize, usize),
) {
    assert_eq!(event["type"], "HeaderFooterTextReplaced");
    assert_eq!(event["section"], 0);
    assert_eq!(event["isHeader"], true);
    assert_eq!(event["applyTo"], 0);
    assert_eq!(event["start"]["para"], start.0);
    assert_eq!(event["start"]["offset"], start.1);
    assert_eq!(event["end"]["para"], end.0);
    assert_eq!(event["end"]["offset"], end.1);
    assert_eq!(event["insertedEnd"]["para"], inserted_end.0);
    assert_eq!(event["insertedEnd"]["offset"], inserted_end.1);
}

#[test]
fn edit_query_and_preview_renderer_resolve_the_same_hf_definition() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank2010 생성");
    for (apply_to, text) in [(1, "E"), (2, "ODD-LONG")] {
        doc.create_header_footer_native(0, false, apply_to)
            .expect("홀짝 꼬리말 생성");
        doc.insert_text_in_header_footer_native(0, false, apply_to, 0, 0, text)
            .expect("홀짝 꼬리말 입력");
    }
    let preview_page: serde_json::Value = serde_json::from_str(
        &doc.get_header_footer_preview_page_native(0)
            .expect("대표 페이지 조회"),
    )
    .expect("대표 페이지 JSON");
    let preview_page = preview_page["pageIndex"].as_u64().expect("pageIndex") as u32;

    let mut widths = Vec::new();
    for (apply_to, expected_text) in [(1, "E"), (2, "ODD-LONG")] {
        let edit_query: serde_json::Value = serde_json::from_str(
            &doc.get_header_footer_native(0, false, apply_to)
                .expect("편집 command resolver 조회"),
        )
        .expect("HF JSON");
        assert_eq!(edit_query["text"], expected_text);

        let rendered: serde_json::Value = serde_json::from_str(
            &doc.get_selection_rects_in_header_footer_native(
                0,
                false,
                apply_to,
                preview_page,
                0,
                0,
                0,
                expected_text.chars().count(),
            )
            .expect("대표 페이지 renderer resolver 조회"),
        )
        .expect("selection rect JSON");
        widths.push(rendered[0]["width"].as_f64().expect("selection width"));
    }
    assert!(
        widths[1] > widths[0],
        "편집 조회와 renderer가 각각 요청한 동일 HF 정의를 사용해야 함: {widths:?}"
    );
}

#[test]
fn single_paragraph_insert_delete_and_replace_events_are_exact() {
    let mut insertion = header_with_text("AB");
    insertion.begin_batch_native().expect("event log 초기화");
    insertion
        .replace_range_in_header_footer_native(0, true, 0, 0, 1, 0, 1, "X")
        .expect("단일 문단 삽입");
    let event = only_event(&insertion.end_batch_native().expect("삽입 event log"));
    assert_range_event(&event, (0, 1), (0, 1), (0, 2));

    let mut deletion = header_with_text("AB");
    deletion.begin_batch_native().expect("event log 초기화");
    deletion
        .replace_range_in_header_footer_native(0, true, 0, 0, 0, 0, 1, "")
        .expect("단일 문단 삭제");
    let event = only_event(&deletion.end_batch_native().expect("삭제 event log"));
    assert_range_event(&event, (0, 0), (0, 1), (0, 0));

    let mut replacement = header_with_text("AB");
    replacement.begin_batch_native().expect("event log 초기화");
    replacement
        .replace_range_in_header_footer_native(0, true, 0, 0, 0, 0, 1, "XY")
        .expect("단일 문단 치환");
    let event = only_event(&replacement.end_batch_native().expect("치환 event log"));
    assert_range_event(&event, (0, 0), (0, 1), (0, 2));
}

#[test]
fn direct_insert_and_delete_events_report_the_actual_clamped_range() {
    let mut insertion = header_with_text("AB");
    insertion.begin_batch_native().expect("event log 초기화");
    insertion
        .insert_text_in_header_footer_native(0, true, 0, 0, 99, "X")
        .expect("문단 끝에 삽입");
    let event = only_event(&insertion.end_batch_native().expect("삽입 event log"));
    assert_range_event(&event, (0, 2), (0, 2), (0, 3));

    let mut deletion = header_with_text("AB");
    deletion.begin_batch_native().expect("event log 초기화");
    deletion
        .delete_text_in_header_footer_native(0, true, 0, 0, 1, 99)
        .expect("문단 끝까지 삭제");
    let event = only_event(&deletion.end_batch_native().expect("삭제 event log"));
    assert_range_event(&event, (0, 1), (0, 2), (0, 1));
}

#[test]
fn multi_paragraph_insert_delete_and_replace_events_are_exact() {
    let mut insertion = header_with_text("AB");
    insertion.begin_batch_native().expect("event log 초기화");
    insertion
        .replace_range_in_header_footer_native(0, true, 0, 0, 1, 0, 1, "X\nY")
        .expect("다문단 삽입");
    let event = only_event(&insertion.end_batch_native().expect("삽입 event log"));
    assert_range_event(&event, (0, 1), (0, 1), (1, 1));

    let mut deletion = header_with_text("AB");
    deletion
        .split_paragraph_in_header_footer_native(0, true, 0, 0, 1, None)
        .expect("삭제용 문단 분할");
    deletion.begin_batch_native().expect("event log 초기화");
    deletion
        .replace_range_in_header_footer_native(0, true, 0, 0, 1, 1, 1, "")
        .expect("다문단 삭제");
    let event = only_event(&deletion.end_batch_native().expect("삭제 event log"));
    assert_range_event(&event, (0, 1), (1, 1), (0, 1));

    let mut replacement = header_with_text("ABCD");
    replacement
        .split_paragraph_in_header_footer_native(0, true, 0, 0, 2, None)
        .expect("치환용 문단 분할");
    replacement.begin_batch_native().expect("event log 초기화");
    replacement
        .replace_range_in_header_footer_native(0, true, 0, 0, 1, 1, 1, "X\nYZ")
        .expect("다문단 치환");
    let event = only_event(&replacement.end_batch_native().expect("치환 event log"));
    assert_range_event(&event, (0, 1), (1, 1), (1, 2));
}
