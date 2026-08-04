//! Task #3206: 머리말/꼬리말 편집 대상은 그 쪽에 **실제로 렌더되는** 컨트롤이어야 한다.
//!
//! 어느 컨트롤이 렌더되는지는 쪽 홀짝이 정한다 — `renderer/pagination/engine.rs` 의
//! `active_header = if is_odd { odd.or(both) } else { even.or(both) }`, 즉 홀수/짝수가
//! 양 쪽을 이긴다. 진입 경로가 `양 쪽`으로 대상을 고정하면:
//!
//!   - 홀수 전용 머리말이 있는 홀수 쪽에서 `양 쪽` 이 없다고 판단해 사용자가 요청한 적
//!     없는 빈 `양 쪽` 머리말을 새로 만들고,
//!   - 진입은 `양 쪽` 으로 하는데 화면에 그려지는 건 여전히 홀수 컨트롤이라, 캐럿은
//!     홀수 머리말 글자에 찍히고 입력은 보이지 않는 `양 쪽` 컨트롤로 들어간다.
//!
//! `get_header_footer_edit_target_native` 는 히트테스트(`hit_test_header_footer_native`)가
//! 쓰는 것과 같은 해석을 좌표 없이 쪽만으로 제공한다.

use rhwp::wasm_api::HwpDocument;

/// 3쪽 문서 — 쪽 번호 1(홀), 2(짝), 3(홀).
fn three_page_doc() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.insert_text_native(0, 0, 0, "본문")
        .expect("insert text");
    doc.insert_page_break_native(0, 0, 2).expect("page break 1");
    doc.insert_page_break_native(0, 1, 0).expect("page break 2");
    doc
}

fn edit_target(doc: &HwpDocument, page: u32) -> String {
    doc.get_header_footer_edit_target_native(page, true)
        .expect("edit target")
}

#[test]
fn odd_only_header_is_the_edit_target_on_odd_pages() {
    let mut doc = three_page_doc();
    // 홀수 쪽 전용 머리말만 만든다 (apply_to 2 = 홀수).
    doc.create_header_footer_native(0, true, 2)
        .expect("create odd header");

    // 쪽 인덱스 0 = 쪽 번호 1 = 홀수 → 홀수 컨트롤이 렌더된다.
    let target = edit_target(&doc, 0);
    assert!(
        target.contains("\"applyTo\":2"),
        "홀수 쪽의 편집 대상은 홀수 머리말이어야 한다 (실제: {})",
        target
    );
    assert!(
        target.contains("\"sectionIndex\":0"),
        "구역 0 (실제: {})",
        target
    );
}

#[test]
fn even_page_without_matching_header_falls_back_to_both_for_creation() {
    let mut doc = three_page_doc();
    doc.create_header_footer_native(0, true, 2)
        .expect("create odd header");

    // 쪽 인덱스 1 = 쪽 번호 2 = 짝수. 짝수·양 쪽 머리말이 없으니 렌더되는 것이 없다
    // → 새로 만들 대상인 `양 쪽`(0) 이 답이다.
    let target = edit_target(&doc, 1);
    assert!(
        target.contains("\"applyTo\":0"),
        "렌더되는 머리말이 없으면 양 쪽으로 새로 만드는 것이 대상 (실제: {})",
        target
    );
}

#[test]
fn odd_specific_header_wins_over_both() {
    let mut doc = three_page_doc();
    doc.create_header_footer_native(0, true, 0)
        .expect("create both header");
    doc.create_header_footer_native(0, true, 2)
        .expect("create odd header");

    // 둘 다 있으면 홀수 쪽에서는 홀수가 이긴다 — 양 쪽을 답으로 주면 보이지 않는
    // 컨트롤을 편집하게 된다.
    let odd_page = edit_target(&doc, 0);
    assert!(
        odd_page.contains("\"applyTo\":2"),
        "홀수 쪽은 홀수 머리말이 렌더된다 (실제: {})",
        odd_page
    );

    // 짝수 쪽은 홀수가 적용되지 않으므로 양 쪽이 렌더된다.
    let even_page = edit_target(&doc, 1);
    assert!(
        even_page.contains("\"applyTo\":0"),
        "짝수 쪽은 양 쪽 머리말이 렌더된다 (실제: {})",
        even_page
    );
}
