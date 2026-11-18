//! 관계형.
//! 주석 갱신: 2026-11-10T18:02:22+09:00; 직전 파일 커밋: 2026-10-11T22:35:38+09:00 (defe8d69faccfca9691e197d91b4dc4a61151904).
//! 합성 문서로 선택 사각형 API의 계약(쪽 트리 재사용, 캐럿·단 경계와 사각형의 관계)을 검사한다.
//! 실제 문서 쪽의 시각 출력을 검증하지 않으므로 raw 일치율을 적지 않는다. 문단 부호 폭 2.5pt는
//! 한글 12.30 Mac 화면에서 10pt·20pt 글자로 확인했다(#8908).
//! 본문 선택은 렌더·캐럿 질의가 이미 만든 쪽 트리만 재사용한다.
//! 선택 때문에 보이지 않는 모든 쪽을 공유 캐시에 남기지는 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::perf_counters;
use rhwp::wasm_api::HwpDocument;

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

fn rects(raw: &str) -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(raw)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn caret_x(doc: &HwpDocument, para: usize, offset: usize) -> f64 {
    let caret: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect_native(0, para, offset).unwrap()).unwrap();
    caret["x"].as_f64().unwrap()
}

fn right(rect: &serde_json::Value) -> f64 {
    rect["x"].as_f64().unwrap() + rect["width"].as_f64().unwrap()
}

/// 한컴 12.30 Mac: 문단 끝까지 고른 줄은 글자 끝에 문단 부호 폭만 더 칠하고, 빈 문단은 그 폭의
/// 상자를 보인다. 단 오른쪽 끝까지 칠하는 것은 자동 줄바꿈된 줄뿐이다.
#[test]
fn body_selection_ends_lines_at_the_paragraph_mark() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let lines = ["첫째 줄입니다", "둘째 줄", "", "", "셋째 줄 글자"];
    doc.insert_text_native(0, 0, 0, lines[0]).unwrap();
    for (index, text) in lines.iter().enumerate().skip(1) {
        let end = lines[index - 1].chars().count();
        doc.split_paragraph_native(0, index - 1, end, None).unwrap();
        doc.insert_text_native(0, index, 0, text).unwrap();
    }

    let selected = rects(&doc.get_selection_rects(0, 0, 0, 4, 3).unwrap());
    assert_eq!(selected.len(), 5, "문단마다 한 줄씩 칠한다: {selected:?}");
    for (para, rect) in selected.iter().enumerate().take(4) {
        let end = caret_x(&doc, para, lines[para].chars().count());
        let mark = right(rect) - end;
        assert!(
            mark > 2.0 && mark < 5.0,
            "{para}번 문단은 글자 끝에서 문단 부호 폭만 더 칠한다: {mark}"
        );
    }
    assert!((right(&selected[4]) - caret_x(&doc, 4, 3)).abs() < 0.15);

    // 왼쪽 정렬 영어 문단의 첫 줄은 낱말 단위로 넘어가 글자 끝이 단 끝보다 앞에 선다.
    let text = "Extraordinarily long words like internationalization and characterization \
                wrap unevenly across ragged lines so the right edge stays uneven everywhere.";
    doc.split_paragraph_native(0, 4, lines[4].chars().count(), None)
        .unwrap();
    doc.insert_text_native(0, 5, 0, text).unwrap();
    doc.apply_para_format_native(0, 5, r#"{"alignment":"left"}"#)
        .unwrap();
    let wrapped = rects(
        &doc.get_selection_rects(0, 5, 0, 5, text.chars().count() as u32)
            .unwrap(),
    );
    assert!(wrapped.len() >= 2, "{wrapped:?}");
    let page: serde_json::Value =
        serde_json::from_str(&doc.get_page_info_native(0).unwrap()).unwrap();
    let column = &page["columns"][0];
    let column_right = column["x"].as_f64().unwrap() + column["width"].as_f64().unwrap();
    assert!(
        (right(&wrapped[0]) - column_right).abs() < 0.15,
        "자동 줄바꿈된 줄은 단 오른쪽 끝까지 칠한다: {wrapped:?} / {column_right}"
    );
}

/// 표 셀도 문단 끝 줄은 글자 끝에 문단 부호 폭만 더 칠하고, 빈 문단은 그 폭의 상자를 보인다(#8908).
#[test]
fn cell_selection_ends_lines_at_the_paragraph_mark() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let table: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 0, 1, 3).unwrap()).unwrap();
    let parent = table["paraIdx"].as_u64().unwrap() as usize;
    let control = table["controlIdx"].as_u64().unwrap() as usize;
    let lines = ["첫째 줄", "", "셋째 줄 글자"];
    doc.insert_text_in_cell_native(0, parent, control, 0, 0, 0, lines[0])
        .unwrap();
    for (index, text) in lines.iter().enumerate().skip(1) {
        let end = lines[index - 1].chars().count();
        doc.split_paragraph_in_cell_native(0, parent, control, 0, index - 1, end, None)
            .unwrap();
        doc.insert_text_in_cell_native(0, parent, control, 0, index, 0, text)
            .unwrap();
    }
    let caret_x = |para: usize, offset: usize| {
        let caret: serde_json::Value = serde_json::from_str(
            &doc.get_cursor_rect_in_cell_native(0, parent, control, 0, para, offset)
                .unwrap(),
        )
        .unwrap();
        caret["x"].as_f64().unwrap()
    };

    let selected = rects(
        &doc.get_selection_rects_in_cell(0, parent as u32, control as u32, 0, 0, 0, 2, 3)
            .unwrap(),
    );
    assert_eq!(selected.len(), 3, "빈 문단도 한 줄을 칠한다: {selected:?}");
    for (para, rect) in selected.iter().enumerate().take(2) {
        let mark = right(rect) - caret_x(para, lines[para].chars().count());
        assert!(
            mark > 2.0 && mark < 5.0,
            "셀 {para}번 문단은 글자 끝에서 문단 부호 폭만 더 칠한다: {mark}"
        );
    }
    assert!((right(&selected[2]) - caret_x(2, 3)).abs() < 0.15);
}
