//! 본문 선택은 렌더·캐럿 질의가 이미 만든 쪽 트리만 재사용한다.
//! 선택 때문에 보이지 않는 모든 쪽을 공유 캐시에 남기지는 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::perf_counters;
use rhwp::wasm_api::HwpDocument;

#[test]
fn body_selection_reuses_a_rendered_page_without_changing_rects() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/SO-SUEOP.hwp"),
    )
    .expect("공개 46쪽 문서");
    let doc = HwpDocument::from_bytes(&bytes).expect("문서 열기");
    let expected = doc.get_selection_rects(0, 57, 4, 57, 5).unwrap();
    assert_ne!(expected, "[]");
    doc.get_cursor_rect_native(0, 57, 5).unwrap();

    perf_counters::reset_thread_page_tree_builds();
    for _ in 0..4 {
        assert_eq!(doc.get_selection_rects(0, 57, 4, 57, 5).unwrap(), expected);
    }
    assert_eq!(
        perf_counters::thread_page_tree_builds(),
        0,
        "캐럿 질의가 만든 유효한 쪽 트리를 선택 표시가 다시 만들면 안 된다"
    );
}

#[test]
fn body_selection_keeps_uncached_pages_local() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let text = "가나다라마바사 ".repeat(800);
    doc.insert_text_native(0, 0, 0, &text).unwrap();
    let end = text.chars().count() as u32;
    let pages = doc.page_count() as u64;
    assert!(pages >= 3);
    let expected = doc.get_selection_rects(0, 0, 0, 0, end).unwrap();

    perf_counters::reset_thread_page_tree_builds();
    assert_eq!(doc.get_selection_rects(0, 0, 0, 0, end).unwrap(), expected);
    assert_eq!(perf_counters::thread_page_tree_builds(), pages);

    doc.get_cursor_rect_native(0, 0, 0).unwrap();
    for _ in 0..2 {
        perf_counters::reset_thread_page_tree_builds();
        assert_eq!(doc.get_selection_rects(0, 0, 0, 0, end).unwrap(), expected);
        assert_eq!(
            perf_counters::thread_page_tree_builds(),
            pages - 1,
            "이미 캐시된 첫 쪽만 재사용하고 나머지 쪽은 선택 함수 안에서만 유지한다"
        );
    }
}

#[test]
fn body_selection_observes_edits_that_invalidate_a_warm_tree() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "가나다").unwrap();
    doc.get_cursor_rect_native(0, 0, 2).unwrap();
    let before = doc.get_selection_rects(0, 0, 1, 0, 2).unwrap();

    doc.replace_body_text_local_native(0, 0, 1, 1, "WW")
        .unwrap();
    perf_counters::reset_thread_page_tree_builds();
    let after = doc.get_selection_rects(0, 0, 1, 0, 3).unwrap();
    assert_eq!(perf_counters::thread_page_tree_builds(), 1);
    let width = |raw: &str| {
        let rects: serde_json::Value = serde_json::from_str(raw).unwrap();
        rects[0]["width"].as_f64().unwrap()
    };
    assert!(width(&after) > width(&before));

    doc.get_cursor_rect_native(0, 0, 3).unwrap();
    perf_counters::reset_thread_page_tree_builds();
    assert_eq!(doc.get_selection_rects(0, 0, 1, 0, 3).unwrap(), after);
    assert_eq!(perf_counters::thread_page_tree_builds(), 0);
}
