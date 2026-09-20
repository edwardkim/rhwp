//! Issue #6766: 빈 누름틀의 캐럿은 문단 정렬이 정한 앵커에 남아야 한다.
//!
//! `get_cursor_rect_native`는 빈 필드의 zero-width TextRun을 반환한다. 그 앵커가
//! 본문 좌단에 고정되면 가운데·오른쪽 정렬 필드가 왼쪽으로 튀어 보인다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

fn cursor_x(core: &DocumentCore, para: usize) -> f64 {
    let json = core
        .get_cursor_rect_native(0, para, 0)
        .unwrap_or_else(|e| panic!("paragraph {para} cursor rect: {e:?}"));
    serde_json::from_str::<Value>(&json)
        .unwrap_or_else(|e| panic!("cursor rect JSON {json:?}: {e}"))["x"]
        .as_f64()
        .expect("cursor x")
}

#[test]
fn empty_clickhere_caret_follows_left_center_and_right_alignment() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    core.split_paragraph_native(0, 0, 0, None)
        .expect("second paragraph");
    core.split_paragraph_native(0, 1, 0, None)
        .expect("third paragraph");

    // 실제 Studio와 같은 문단 서식 API를 써서, 변경된 문단 모양을 recompose·pagination
    // 상태까지 전파한다. 테스트에서 IR만 직접 바꾸면 오래된 resolved style cache를
    // 읽는 별도 문제를 이 캐럿 회귀와 혼동하게 된다.
    core.apply_para_format_native(0, 1, r#"{"alignment":"center"}"#)
        .expect("center alignment");
    core.apply_para_format_native(0, 2, r#"{"alignment":"right"}"#)
        .expect("right alignment");

    for (para, name) in [(0, "left"), (1, "center"), (2, "right")] {
        core.insert_click_here_field_at(0, para, 0, "입력", "", name, true)
            .unwrap_or_else(|e| panic!("{name} field: {e:?}"));
    }

    let left_x = cursor_x(&core, 0);
    let center_x = cursor_x(&core, 1);
    let right_x = cursor_x(&core, 2);
    assert!(
        left_x + 20.0 < center_x && center_x + 20.0 < right_x,
        "empty ClickHere caret must follow alignment: left={left_x}, center={center_x}, right={right_x}",
    );
}
