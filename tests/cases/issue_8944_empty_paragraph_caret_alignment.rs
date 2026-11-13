//! 글자 없는 빈 문단의 캐럿은 문단 정렬이 정한 자리에 있다.
//! 첫 글자를 넣었을 때 캐럿이 옆으로 튀지 않아야 한다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

fn cursor_x(core: &DocumentCore, para: usize, offset: usize) -> f64 {
    let json = core
        .get_cursor_rect_native(0, para, offset)
        .unwrap_or_else(|e| panic!("paragraph {para} cursor rect: {e:?}"));
    serde_json::from_str::<Value>(&json).expect("cursor rect JSON")["x"]
        .as_f64()
        .expect("cursor x")
}

/// 정렬만 다른 빈 문단 세 개 (왼쪽·가운데·오른쪽)
fn three_aligned_empty_paragraphs() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    core.split_paragraph_native(0, 0, 0, None)
        .expect("second paragraph");
    core.split_paragraph_native(0, 1, 0, None)
        .expect("third paragraph");
    core.apply_para_format_native(0, 1, r#"{"alignment":"center"}"#)
        .expect("center alignment");
    core.apply_para_format_native(0, 2, r#"{"alignment":"right"}"#)
        .expect("right alignment");
    core
}

#[test]
fn empty_paragraph_caret_follows_left_center_and_right_alignment() {
    let core = three_aligned_empty_paragraphs();
    let (left, center, right) = (
        cursor_x(&core, 0, 0),
        cursor_x(&core, 1, 0),
        cursor_x(&core, 2, 0),
    );
    assert!(
        left + 20.0 < center && center + 20.0 < right,
        "empty paragraph caret must follow alignment: left={left}, center={center}, right={right}",
    );
}

#[test]
fn first_character_does_not_move_the_caret_sideways() {
    let mut core = three_aligned_empty_paragraphs();
    for para in [1, 2] {
        let empty = cursor_x(&core, para, 0);
        core.insert_text_native(0, para, 0, "가")
            .expect("insert text");
        let before_char = cursor_x(&core, para, 0);
        let after_char = cursor_x(&core, para, 1);
        // 가운데는 글자 폭의 절반, 오른쪽은 글자 폭만큼 빈 캐럿이 글자 앞뒤 사이에 온다.
        assert!(
            before_char - 1.0 <= empty && empty <= after_char + 1.0,
            "paragraph {para}: empty caret {empty} must sit within the first character [{before_char}, {after_char}]",
        );
    }
}
