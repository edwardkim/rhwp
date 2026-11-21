//! 같은 글자 자리의 빈 누름틀 둘은 한컴 커서 자리와 컨트롤 번호로 갈린다.
//!
//! 한/글은 같은 자리에 차례로 넣은 빈 누름틀 가·나를 가 시작·가 안내문·가 끝·나 시작·나 안내문·나 끝 차례로
//! 두고, ⌃F11로 하나씩 고른다. `getFieldList`의 `startPos`·`endPos`가 둘 다 나의 자리를 주면 가를 가리킬 수 없다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

#[test]
fn same_place_empty_click_here_fields_get_their_own_cursor_positions() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞 뒤끝").unwrap();
    core.insert_click_here_field_at(0, 0, 2, "가안내", "", "가", true)
        .unwrap();
    core.insert_click_here_field_at(0, 0, 2, "나안내", "", "나", true)
        .unwrap();
    let list: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    let rows: Vec<_> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["name"].as_str().unwrap().to_string(),
                f["startPos"].as_u64().unwrap(),
                f["endPos"].as_u64().unwrap(),
                f["controlIndex"].as_u64(),
            )
        })
        .collect();
    // 구역·단 정의(16)와 "앞 "(2) 뒤에 가 시작(8), 가 안내문(3), 가 끝(8), 나 시작(8), 나 안내문(3).
    assert_eq!(
        rows,
        vec![
            ("가".to_string(), 26, 29, Some(2)),
            ("나".to_string(), 45, 48, Some(3)),
        ]
    );
    // 글자 자리 2는 가의 시작이다. 응답이 그 컨트롤 번호를 함께 준다.
    let info: Value = serde_json::from_str(&core.get_field_info_at(0, 0, 2)).unwrap();
    assert_eq!(info["controlIndex"].as_u64(), Some(2));
}
