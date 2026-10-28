//! [#6452] 머리말/꼬리말 선택 갱신이 mousemove·Shift 탐색마다 page tree를
//! 다시 빌드하던 회귀 가드.
//!
//! 시계 대신 current-thread page tree build 카운터로 작업량을 고정한다. 실제 적용
//! 페이지와 구역 첫 페이지의 Odd/Even 대표 투영 모두 첫 질의만 빌드하고, 같은 문서
//! 상태의 후속 hit-test·선택 질의는 캐시를 재사용해야 한다. HF 편집 뒤에는 캐시가
//! 무효화되어 새 geometry를 정확히 한 번 다시 빌드해야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::perf_counters;
use rhwp::wasm_api::HwpDocument;

fn multi_page_document() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("blank2010 문서 생성");
    doc.insert_text_native(0, 0, 0, &"가나다라마바사 ".repeat(4_000))
        .expect("여러 쪽 본문 생성");
    assert!(doc.page_count() >= 3, "다중 페이지 캐시 검증 전제");
    doc
}

fn selection_width(raw: &str) -> f64 {
    let rects: serde_json::Value = serde_json::from_str(raw).expect("selection rect JSON");
    rects
        .as_array()
        .and_then(|values| values.first())
        .and_then(|rect| rect["width"].as_f64())
        .expect("첫 selection rect width")
}

#[test]
fn active_header_selection_reuses_each_pages_cached_tree() {
    let mut doc = multi_page_document();
    doc.create_header_footer_native(0, true, 0)
        .expect("양쪽 머리말 생성");
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "P")
        .expect("머리말 입력");
    doc.insert_field_in_hf_native(0, true, 0, 0, 1, 1)
        .expect("현재 쪽번호 필드 입력");

    let visible_pages: Vec<u32> = (0..doc.page_count().min(3)).collect();
    perf_counters::reset_thread_page_tree_builds();

    for frame in 0..4 {
        for &page_num in &visible_pages {
            let selection_end = if frame % 2 == 0 { 1 } else { 2 };
            let raw = doc
                .get_selection_rects_in_header_footer_native(
                    0,
                    true,
                    0,
                    page_num,
                    0,
                    0,
                    0,
                    selection_end,
                )
                .expect("양쪽 머리말 selection rect");
            let rects: serde_json::Value = serde_json::from_str(&raw).expect("rect JSON");
            assert_eq!(rects[0]["pageIndex"], page_num);
            doc.hit_test_in_header_footer_native(
                page_num,
                true,
                rects[0]["x"].as_f64().expect("rect x") + 1.0,
                rects[0]["y"].as_f64().expect("rect y") + 1.0,
            )
            .expect("같은 frame의 머리말 hit-test");
        }
    }

    assert_eq!(
        perf_counters::thread_page_tree_builds(),
        visible_pages.len() as u64,
        "동일 문서 상태에서는 보이는 페이지마다 최초 한 번만 page tree를 빌드해야 한다"
    );

    doc.insert_text_native(0, 0, 0, "본문 편집 ")
        .expect("본문 편집");
    perf_counters::reset_thread_page_tree_builds();
    for _ in 0..2 {
        doc.get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 0, 2)
            .expect("본문 편집 후 머리말 selection rect");
    }
    assert_eq!(
        perf_counters::thread_page_tree_builds(),
        1,
        "본문 편집은 실제 page cache를 무효화하고 후속 질의가 한 번 다시 빌드해야 한다"
    );
}

#[test]
fn preview_target_reuses_tree_and_header_footer_edit_invalidates_it() {
    let mut doc = multi_page_document();
    for apply_to in [1, 2] {
        doc.create_header_footer_native(0, false, apply_to)
            .expect("홀짝 꼬리말 생성");
        doc.insert_text_in_header_footer_native(0, false, apply_to, 0, 0, "AB")
            .expect("홀짝 꼬리말 입력");
    }

    let actual_raw = doc
        .get_header_footer_edit_target_native(0, false)
        .expect("첫 페이지 실제 꼬리말 target");
    let actual: serde_json::Value = serde_json::from_str(&actual_raw).expect("target JSON");
    let actual_apply = actual["applyTo"].as_u64().expect("applyTo") as u8;
    let preview_apply = if actual_apply == 1 { 2 } else { 1 };

    perf_counters::reset_thread_page_tree_builds();
    let mut before = String::new();
    for _ in 0..4 {
        before = doc
            .get_selection_rects_in_header_footer_native(0, false, preview_apply, 0, 0, 0, 0, 2)
            .expect("대표 투영 selection rect");
        let rects: serde_json::Value = serde_json::from_str(&before).expect("preview rect JSON");
        doc.hit_test_in_header_footer_target_native(
            0,
            0,
            false,
            preview_apply,
            rects[0]["x"].as_f64().expect("preview rect x") + 1.0,
            rects[0]["y"].as_f64().expect("preview rect y") + 1.0,
        )
        .expect("같은 frame의 대표 target hit-test");
    }
    assert_eq!(
        perf_counters::thread_page_tree_builds(),
        1,
        "같은 Odd/Even 대표 투영은 최초 한 번만 빌드해야 한다"
    );

    // #4969가 도입한 page-local 공통 무효화도 HF preview를 함께 비워야 한다.
    // 꼬리말 선택은 유지한 채 같은 페이지의 머리말 숨김 상태만 바꾼다.
    doc.toggle_hide_header_footer_native(0, true)
        .expect("대표 페이지 머리말 숨김");
    perf_counters::reset_thread_page_tree_builds();
    for _ in 0..2 {
        doc.get_selection_rects_in_header_footer_native(0, false, preview_apply, 0, 0, 0, 0, 2)
            .expect("페이지 단위 무효화 후 대표 꼬리말 selection rect");
    }
    assert_eq!(
        perf_counters::thread_page_tree_builds(),
        1,
        "페이지 단위 무효화도 preview tree를 한 번 다시 빌드해야 한다"
    );

    doc.insert_text_in_header_footer_native(0, false, preview_apply, 0, 2, "C")
        .expect("대표 투영 target 편집");
    perf_counters::reset_thread_page_tree_builds();
    let mut after = String::new();
    for _ in 0..4 {
        after = doc
            .get_selection_rects_in_header_footer_native(0, false, preview_apply, 0, 0, 0, 0, 3)
            .expect("편집 후 대표 투영 selection rect");
        let rects: serde_json::Value = serde_json::from_str(&after).expect("edited rect JSON");
        doc.hit_test_in_header_footer_target_native(
            0,
            0,
            false,
            preview_apply,
            rects[0]["x"].as_f64().expect("edited rect x") + 1.0,
            rects[0]["y"].as_f64().expect("edited rect y") + 1.0,
        )
        .expect("편집 뒤 같은 frame의 대표 target hit-test");
    }
    assert_eq!(
        perf_counters::thread_page_tree_builds(),
        1,
        "HF 편집은 preview cache를 무효화하고 후속 질의가 한 번 다시 빌드해야 한다"
    );
    assert!(
        selection_width(&after) > selection_width(&before),
        "편집 뒤 재빌드된 geometry가 추가 문자를 반영해야 한다: before={before}, after={after}"
    );
}
