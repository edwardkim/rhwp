//! #7711: 본문 전역 문단 번호(`para_in_list`)와 중첩 리스트 호스트 문단 번호 계약 검증.
//!
//! 다중 구역 문서에서 `FieldInfo.para_in_list`는 본문 전역 0-based 인덱스를 따르고,
//! 표·글상자 등 중첩 리스트의 `para_in_list`는 해당 리스트 내 로컬 인덱스를 유지하며
//! `insert_click_here_field_at_cursor`도 본문 전역 번호로 올바른 구역·문단에 삽입되는지 확인한다.

use rhwp::document_core::queries::field_query::ROOT_LIST_ID;
use rhwp::document_core::DocumentCore;

#[test]
fn issue7711_sample_fields_and_create_field_follow_root_cursor() {
    let bytes = include_bytes!("../../samples/field-01.hwp");
    let core = DocumentCore::from_bytes(bytes).unwrap();
    let model: serde_json::Value = serde_json::from_str(&core.get_cursor_model_json()).unwrap();
    assert_eq!(model["root"]["paraCount"], 16);

    let fields = core.collect_all_fields();
    let title = fields
        .iter()
        .find(|f| f.field.field_name() == Some("제목"))
        .unwrap();
    assert_eq!(
        (title.location.section_index, title.location.para_index),
        (2, 0)
    );
    assert_eq!((title.list_id, title.para_in_list), (0, 13));
    assert_eq!(core.cur_field_state(0, 13, title.start_pos), 0x12);

    for section_index in 0..3 {
        let mut core = DocumentCore::from_bytes(bytes).unwrap();
        let para_in_list: usize = core.document().sections[..section_index]
            .iter()
            .map(|s| s.paragraphs.len())
            .sum();
        let others: Vec<_> = core
            .document()
            .sections
            .iter()
            .map(|s| s.paragraphs[0].text.clone())
            .collect();
        core.insert_click_here_field_at_cursor(0, para_in_list, 0, "안내", "", "새필드", true)
            .unwrap();
        let inserted = core
            .collect_all_fields()
            .into_iter()
            .find(|f| f.field.field_name() == Some("새필드"))
            .unwrap();
        assert_eq!(inserted.location.section_index, section_index);
        assert_eq!(inserted.location.para_index, 0);
        assert_eq!(inserted.para_in_list, para_in_list);
        assert_eq!(
            core.cur_field_state(0, para_in_list, inserted.start_pos),
            0x12
        );
        for (si, text) in others.iter().enumerate() {
            if si != section_index {
                assert_eq!(&core.document().sections[si].paragraphs[0].text, text);
            }
        }
    }

    let mut core = DocumentCore::from_bytes(bytes).unwrap();
    assert!(core
        .insert_click_here_field_at_cursor(0, 16, 0, "안내", "", "범위초과", true)
        .is_err());
}

#[test]
fn issue7711_nested_table_and_root_fields_keep_distinct_addressing() {
    let bytes = include_bytes!("../../samples/tac-case-001.hwp");
    let mut core = DocumentCore::from_bytes(bytes).unwrap();
    let (_, lists) = core.collect_fields_and_lists();
    let cell_list = lists.iter().find(|l| l.is_cell).unwrap();
    core.insert_click_here_field_at_cursor(cell_list.list_id, 0, 0, "셀안내", "", "셀필드", true)
        .unwrap();

    let (fields, lists) = core.collect_fields_and_lists();
    let cell_field = fields
        .iter()
        .find(|f| f.field.field_name() == Some("셀필드"))
        .unwrap();
    assert_ne!(cell_field.list_id, ROOT_LIST_ID);
    assert_eq!(cell_field.para_in_list, 0);
    assert_eq!(
        core.cur_field_state(
            cell_field.list_id,
            cell_field.para_in_list,
            cell_field.start_pos
        ),
        0x12
    );
    let entry = lists
        .iter()
        .find(|l| l.list_id == cell_field.list_id)
        .unwrap();
    assert_eq!(entry.host_list_id, ROOT_LIST_ID);
    assert!(
        entry.host_para_index
            < core.document().sections[entry.section_index]
                .paragraphs
                .len()
    );
}
