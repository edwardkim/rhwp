//! 공개 API의 명시적 쪽 나누기는 끝 빈 문단의 쪽 소속을 보존한다.
//!
//! 같은 API로 만든 38줄·41줄 원본의 HWP/HWPX를 한컴 2020에서 Print하면 모두 2쪽이다.
//! 양형식의 전쪽 Native/fresh WASM raw 100% 대조 후, 빈 줄이 꽉 찬 경우에도
//! 마지막 문단을 앞 쪽으로 흡수한다는 잘못된 기대를 제거한다. 픽셀 위치는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

/// 빈 문서에 Enter 를 `lines - 1` 번 눌러 `lines` 개의 빈 줄을 만든다.
fn blank_lines(lines: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    for para in 0..lines - 1 {
        doc.split_paragraph_native(0, para, 0, None).unwrap();
    }
    doc
}

/// 1쪽에 들어가는 최대 빈 줄 수.
fn full_page_lines() -> usize {
    let mut lines = 1;
    while blank_lines(lines + 1).page_count() == 1 {
        lines += 1;
    }
    lines
}

#[test]
fn fresh_full_page_break_preserves_terminal_paragraph_on_next_page() {
    let lines = full_page_lines();
    let mut doc = blank_lines(lines);
    doc.insert_page_break_native(0, lines - 1, 0).unwrap();
    assert_eq!(
        doc.page_count(),
        2,
        "같은 API 입력의 독립 한컴 Print는 끝 빈 쪽을 포함한 2쪽이다"
    );
    assert_eq!(
        doc.get_paragraph_count_native(0).unwrap(),
        lines + 1,
        "쪽 나누기 전 문단들과 새 끝 문단을 모두 보존한다"
    );
    let first: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect_native(0, 0, 0).unwrap()).unwrap();
    let terminal: serde_json::Value = serde_json::from_str(
        &doc.get_cursor_rect_native(0, lines, 0)
            .expect("쪽 나누기 문단의 실제 캐럿"),
    )
    .unwrap();
    assert_eq!(first["pageIndex"], 0, "기존 첫 문단은 첫 쪽에 남는다");
    assert_eq!(
        terminal["pageIndex"], 1,
        "마지막 빈 문단은 독립 Print와 같은 둘째 쪽에 속한다"
    );
}

#[test]
fn page_break_with_room_left_keeps_blank_page() {
    let mut doc = blank_lines(full_page_lines() - 3);
    let last = doc.get_paragraph_count_native(0).unwrap() - 1;
    doc.insert_page_break_native(0, last, 0).unwrap();
    assert_eq!(
        doc.page_count(),
        2,
        "앞 쪽에 자리가 있으면 쪽나누기대로 빈 쪽을 연다"
    );
}
