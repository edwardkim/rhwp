//! 구역 끝 빈 문단의 쪽나누기는 그 문단이 앞 쪽에 어차피 들어가지 않으면 새 빈 쪽을
//! 만들지 않는다.
//!
//! 한/글 2020 PDF(별지 서식 `05-gambling-business-succession.hwp`)는 쪽을 꽉 채운
//! 글자처럼 취급 표 뒤의 빈 쪽나누기 문단을 1쪽으로 인쇄한다. 앞 쪽에 자리가 남은
//! 같은 문단은 쪽나누기대로 빈 쪽을 인쇄한다(`samples/footnote-01.hwp` 6쪽).
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
fn page_break_after_full_page_adds_no_blank_page() {
    let lines = full_page_lines();
    let mut doc = blank_lines(lines);
    doc.insert_page_break_native(0, lines - 1, 0).unwrap();
    assert_eq!(
        doc.page_count(),
        1,
        "꽉 찬 쪽 뒤 빈 쪽나누기 문단은 새 쪽을 열지 않는다"
    );
    doc.get_cursor_rect_native(0, lines, 0)
        .expect("쪽나누기 문단도 캐럿 자리를 가져야 한다");
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
