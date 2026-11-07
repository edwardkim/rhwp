//! [#8017] Memo read/edit API: `getMemoList`, `insertMemo`, `insertMemoByPath`,
//! `setMemoText`, `removeMemo` through the native `DocumentCore` entries.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Field, Parameter};
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use serde_json::Value;

fn sample(rel: &str) -> Vec<u8> {
    let path = format!("{}/samples/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// A blank document whose first paragraph holds `text`.
fn core_with(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    core
}

fn memo_list(core: &DocumentCore) -> Vec<Value> {
    serde_json::from_str::<Value>(&core.get_memo_list_json())
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn inserted_id(json: &str) -> u32 {
    let value: Value = serde_json::from_str(json).unwrap();
    assert_eq!(value["ok"], true, "{json}");
    value["id"].as_u64().unwrap() as u32
}

fn is_memo(field: &Field) -> bool {
    field.command.starts_with("MEMO/")
}

fn first_para(core: &DocumentCore) -> &Paragraph {
    &core.document().sections[0].paragraphs[0]
}

fn memo_fields(paragraphs: &[Paragraph]) -> Vec<&Field> {
    paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .filter_map(|ctrl| match ctrl {
            Control::Field(field) if is_memo(field) => Some(field),
            _ => None,
        })
        .collect()
}

fn first_memo_mut(core: &mut DocumentCore) -> &mut Field {
    core.document_mut().sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|ctrl| match ctrl {
            Control::Field(field) if is_memo(field) => Some(field),
            _ => None,
        })
        .expect("a memo field")
}

fn integer_param(field: &Field, key: &str) -> Option<i64> {
    field.parameters.items.iter().find_map(|item| match item {
        Parameter::Integer {
            name: Some(name),
            value,
        } if name == key => Some(*value),
        _ => None,
    })
}

fn string_param(field: &Field, key: &str) -> Option<String> {
    field.parameters.items.iter().find_map(|item| match item {
        Parameter::String {
            name: Some(name),
            value,
            ..
        } if name == key => Some(value.clone()),
        _ => None,
    })
}

#[test]
fn author_drops_command_separators_and_is_limited_to_64_units() {
    let mut core = core_with("abcdef");
    core.insert_memo_at(0, 0, 0, 1, "  A/B\\C;D\nE  ", "x")
        .unwrap();
    core.insert_memo_at(0, 0, 1, 2, "", "y").unwrap();
    let list = memo_list(&core);
    assert_eq!(list[0]["author"], "ABCDE");
    assert_eq!(list[1]["author"], "");

    let error = core
        .insert_memo_at(0, 0, 2, 3, &"가".repeat(65), "z")
        .unwrap_err();
    assert_eq!(error.code.as_str(), "memo_invalid_author");
    core.insert_memo_at(0, 0, 2, 3, &"가".repeat(64), "z")
        .unwrap();
    let author = memo_list(&core)[2]["author"].as_str().unwrap().to_string();
    assert_eq!(author.encode_utf16().count(), 64);
}

#[test]
fn the_memo_command_carries_shape_number_ids_and_author() {
    let mut core = core_with("anchor");
    core.insert_memo_at(0, 0, 0, 6, "hancom", "body").unwrap();
    let list = memo_list(&core);
    let field = memo_fields(std::slice::from_ref(first_para(&core)))[0];
    let tokens: Vec<&str> = field.command.split('/').collect();
    assert_eq!(tokens[0], "MEMO");
    let shape: u32 = tokens[1].parse().unwrap();
    let section_shape = core.document().sections[0].section_def.memo_shape_id as u32;
    assert_eq!(
        shape,
        if section_shape != 0 {
            section_shape
        } else {
            65535
        }
    );
    assert_eq!(tokens[2], list[0]["index"].to_string());
    assert_eq!(tokens[3], field.field_id.to_string());
    assert_ne!(tokens[4].parse::<u32>().unwrap(), 0);
    assert_eq!(tokens[5], "hancom");
    assert_eq!(tokens[6], "\\;;");
    assert_eq!(list[0]["author"], "hancom");
}

#[test]
fn memo_text_joins_body_paragraphs_with_newlines() {
    let mut core = core_with("anchor");
    core.insert_memo_at(0, 0, 0, 6, "t", "첫 줄\n\n끝").unwrap();
    assert_eq!(memo_list(&core)[0]["text"], "첫 줄\n\n끝");
}

#[test]
fn memo_body_splits_lines_and_keeps_a_trailing_empty_paragraph() {
    let mut core = core_with("anchor");
    core.insert_memo_at(0, 0, 0, 6, "t", "one\n\ntwo\n")
        .unwrap();
    let field = memo_fields(std::slice::from_ref(first_para(&core)))[0];
    let texts: Vec<&str> = field
        .memo_paragraphs
        .iter()
        .map(|para| para.text.as_str())
        .collect();
    assert_eq!(texts, ["one", "", "two", ""]);
}

#[test]
fn read_only_reason_tracks_formatting_and_objects() {
    let mut core = core_with("anchor");
    let id = inserted_id(&core.insert_memo_at(0, 0, 0, 6, "t", "한 줄").unwrap());
    assert_eq!(memo_list(&core)[0]["editable"], true);
    assert_eq!(memo_list(&core)[0]["readOnlyReason"], Value::Null);

    first_memo_mut(&mut core).memo_paragraphs[0].char_shapes = vec![
        CharShapeRef {
            start_pos: 0,
            char_shape_id: 1,
        },
        CharShapeRef {
            start_pos: 2,
            char_shape_id: 2,
        },
    ];
    let list = memo_list(&core);
    assert_eq!(list[0]["editable"], false);
    assert_eq!(list[0]["readOnlyReason"], "formatted");
    assert_eq!(
        core.set_memo_text(id, "new").unwrap_err().code.as_str(),
        "memo_read_only"
    );

    first_memo_mut(&mut core).memo_paragraphs[0]
        .char_shapes
        .truncate(1);
    first_memo_mut(&mut core).memo_paragraphs[0]
        .controls
        .push(Control::Table(Default::default()));
    assert_eq!(memo_list(&core)[0]["readOnlyReason"], "object");

    // A read-only memo can still be removed.
    core.remove_memo(id).unwrap();
    assert_eq!(core.get_memo_list_json(), "[]");
}

#[test]
fn insert_and_remove_keep_anchor_text_and_slots_consistent() {
    let mut core = core_with("앞뒤");
    let (ranges, controls, chars) = {
        let para = first_para(&core);
        (
            para.field_ranges.len(),
            para.controls.len(),
            para.char_count,
        )
    };
    let id = inserted_id(&core.insert_memo_at(0, 0, 1, 1, "t", "메모 본문").unwrap());
    {
        let para = first_para(&core);
        assert_eq!(para.text, "앞뒤");
        assert_eq!(para.field_ranges.len(), ranges + 1);
        let range = para.field_ranges.last().unwrap();
        assert_eq!((range.start_char_idx, range.end_char_idx), (1, 1));
        assert_eq!(para.controls.len(), controls + 1);
        // The begin slot is 8 code units, the end marker 8 more.
        assert_eq!(para.char_count, chars + 16);
    }
    core.remove_memo(id).unwrap();
    let para = first_para(&core);
    assert_eq!(para.text, "앞뒤");
    assert_eq!(para.field_ranges.len(), ranges);
    assert_eq!(para.controls.len(), controls);
    assert_eq!(para.char_count, chars);
}

#[test]
fn insert_rejects_overlap_but_allows_zero_length_anchors_on_the_boundary() {
    let mut core = core_with("abcdef");
    core.insert_memo_at(0, 0, 1, 3, "t", "a").unwrap();
    let error = core.insert_memo_at(0, 0, 2, 4, "t", "b").unwrap_err();
    assert_eq!(error.code.as_str(), "memo_overlap");
    core.insert_memo_at(0, 0, 1, 1, "t", "c").unwrap();
    core.insert_memo_at(0, 0, 3, 3, "t", "d").unwrap();
    assert_eq!(memo_list(&core).len(), 3);
}

#[test]
fn text_limit_counts_utf16_units() {
    const MAX: usize = 100_000;
    let mut core = core_with("abcd");
    core.insert_memo_at(0, 0, 0, 1, "t", &"가".repeat(MAX))
        .unwrap();
    let error = core
        .insert_memo_at(0, 0, 1, 2, "t", &"가".repeat(MAX + 1))
        .unwrap_err();
    assert_eq!(error.code.as_str(), "memo_text_too_long");
    // An emoji is two UTF-16 units.
    core.insert_memo_at(0, 0, 1, 2, "t", &"😀".repeat(MAX / 2))
        .unwrap();
    let error = core
        .insert_memo_at(0, 0, 2, 3, "t", &"😀".repeat(MAX / 2 + 1))
        .unwrap_err();
    assert_eq!(error.code.as_str(), "memo_text_too_long");
}

/// A document opened from a file keeps each section's original stream until an
/// edit marks it dirty; a memo text change that did not mark it was lost on save.
#[test]
fn set_memo_text_reaches_the_saved_file() {
    let mut core = core_with("anchor");
    core.insert_memo_at(0, 0, 0, 6, "tester", "body").unwrap();
    let saved = core.export_hwp_native().unwrap();

    let mut reopened = DocumentCore::from_bytes(&saved).unwrap();
    let id = memo_list(&reopened)[0]["id"].as_u64().unwrap() as u32;
    reopened.begin_batch_native().unwrap();
    reopened.set_memo_text(id, "new body").unwrap();
    reopened.end_batch_native().unwrap();
    let resaved = reopened.export_hwp_native().unwrap();

    let again = DocumentCore::from_bytes(&resaved).unwrap();
    assert_eq!(memo_list(&again)[0]["text"], "new body");
}

#[test]
fn deleting_the_whole_anchor_removes_the_memo_and_a_part_keeps_it() {
    let mut core = core_with("hello world");
    core.insert_memo_at(0, 0, 0, 5, "t", "body").unwrap();
    core.delete_text_native(0, 0, 0, 5).unwrap();
    assert_eq!(first_para(&core).text, " world");
    assert_eq!(
        core.get_memo_list_json(),
        "[]",
        "the memo went with its whole anchor"
    );

    let mut core = core_with("hello world");
    core.insert_memo_at(0, 0, 0, 5, "t", "body").unwrap();
    core.delete_text_native(0, 0, 1, 2).unwrap();
    assert_eq!(first_para(&core).text, "hlo world");
    let list = memo_list(&core);
    assert_eq!(list.len(), 1, "a shortened anchor keeps its memo");
    assert_eq!(
        (
            list[0]["startCharIdx"].clone(),
            list[0]["endCharIdx"].clone()
        ),
        (0.into(), 3.into())
    );
}

#[test]
fn native_replace_keeps_a_memo_until_its_whole_anchor_is_deleted() {
    let mut core = core_with("hello world");
    core.insert_memo_at(0, 0, 0, 5, "tester", "body").unwrap();
    core.replace_text_native(0, 0, 0, 5, "HELLO").unwrap();
    let list = memo_list(&core);
    assert_eq!(
        list[0]["text"], "body",
        "a replacement with text keeps the memo"
    );
    assert_eq!(
        (
            list[0]["startCharIdx"].clone(),
            list[0]["endCharIdx"].clone()
        ),
        (0.into(), 5.into())
    );

    core.replace_text_native(0, 0, 0, 5, "").unwrap();
    assert_eq!(core.get_memo_list_json(), "[]");
}

#[test]
fn replace_all_keeps_a_memo_over_replaced_text_and_drops_it_on_empty_text() {
    let mut core = core_with("hello world");
    core.insert_memo_at(0, 0, 0, 5, "tester", "body").unwrap();
    core.replace_all_native("hello", "HELLO!", true).unwrap();
    assert_eq!(first_para(&core).text, "HELLO! world");
    let list = memo_list(&core);
    assert_eq!(list.len(), 1, "text replacement keeps the memo");
    assert_eq!(
        (
            list[0]["startCharIdx"].clone(),
            list[0]["endCharIdx"].clone()
        ),
        (0.into(), 6.into())
    );

    core.replace_all_native("HELLO!", "", true).unwrap();
    assert_eq!(first_para(&core).text, " world");
    assert_eq!(core.get_memo_list_json(), "[]");
}

#[test]
fn the_hancom_hwp5_sample_reads_its_memo_body() {
    let doc = rhwp::parser::parse_document(&sample("issue5866/memo_field_hwp5.hwp"))
        .expect("parse the Hancom memo sample");
    let memos: Vec<&Field> = doc
        .sections
        .iter()
        .flat_map(|section| memo_fields(&section.paragraphs))
        .collect();
    assert_eq!(memos.len(), 1, "one memo in the Hancom sample");
    assert!(
        memos[0]
            .memo_paragraphs
            .iter()
            .any(|para| !para.text.trim().is_empty()),
        "the memo body paragraphs were assigned to the field"
    );
}

/// A memo copied and pasted in the same document gets a new number, and the HWPX
/// writer writes that number rather than the copied original parameters.
#[test]
fn a_pasted_hwpx_memo_is_written_with_its_new_number() {
    let mut core = DocumentCore::from_bytes(&sample("hwpx/aift.hwpx")).unwrap();
    let before = memo_list(&core);
    let next = before
        .iter()
        .map(|memo| memo["index"].as_u64().unwrap())
        .max()
        .expect("the sample has memos")
        + 1;
    let (section, para) = {
        let memo = &before[0];
        assert_eq!(memo["path"], Value::Null, "a body memo");
        (
            memo["sectionIndex"].as_u64().unwrap() as usize,
            memo["paraIndex"].as_u64().unwrap() as usize,
        )
    };
    let para_len = core.document().sections[section].paragraphs[para]
        .text
        .chars()
        .count();
    core.copy_selection_native(section, para, 0, para, para_len)
        .unwrap();
    let last = core.document().sections[section].paragraphs.len() - 1;
    let last_len = core.document().sections[section].paragraphs[last]
        .text
        .chars()
        .count();
    core.paste_internal_native(section, last, last_len).unwrap();

    let after = memo_list(&core);
    assert!(
        after
            .iter()
            .any(|memo| memo["index"].as_u64() == Some(next)),
        "the pasted memo takes number {next}"
    );

    let written = core.export_hwpx_native().unwrap();
    let reread = rhwp::parser::hwpx::parse_hwpx(&written).unwrap();
    let pasted: Vec<&Field> = reread
        .sections
        .iter()
        .flat_map(|section| memo_fields(&section.paragraphs))
        .filter(|field| integer_param(field, "Number") == Some(next as i64))
        .collect();
    assert_eq!(
        pasted.len(),
        1,
        "the written HWPX carries number {next} once"
    );
    assert_eq!(string_param(pasted[0], "ID"), Some(format!("memo{next}")));
    assert_eq!(
        string_param(pasted[0], "Command").as_deref(),
        Some(pasted[0].command.as_str())
    );
}

#[test]
fn the_insert_argument_is_read_as_json() {
    let mut core = core_with("abcdefgh");
    let id = inserted_id(
        &core
            .insert_memo_json_native(0, 0, 0, 1, r#"{"author": "AA", "text": "x\/y\b\fé"}"#)
            .unwrap(),
    );
    let list = memo_list(&core);
    assert_eq!(list[0]["id"].as_u64(), Some(id as u64));
    assert_eq!(list[0]["author"], "AA");
    // `\b` and `\f` decode to C0 controls, which a memo body does not keep.
    assert_eq!(list[0]["text"], "x/yé");

    core.insert_memo_json_native(0, 0, 1, 2, r#"{"text":"only"}"#)
        .unwrap();
    assert_eq!(memo_list(&core)[1]["author"], "");
    assert_eq!(memo_list(&core)[1]["text"], "only");

    for bad in [
        "[]",
        r#"{"text":1}"#,
        r#"{"author":null,"text":"x"}"#,
        "not json",
    ] {
        let error = core.insert_memo_json_native(0, 0, 2, 3, bad).unwrap_err();
        assert_eq!(error.code.as_str(), "memo_insert_failed", "{bad}");
    }
    assert_eq!(memo_list(&core).len(), 2);
}

#[test]
fn insert_by_path_reads_the_cell_path_and_json() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let table: Value =
        serde_json::from_str(&core.create_table_native(0, 0, 0, 1, 2).unwrap()).unwrap();
    let ppi = table["paraIdx"].as_u64().unwrap() as usize;
    let ci = table["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_cell_native(0, ppi, ci, 1, 0, 0, "칸 메모")
        .unwrap();
    let path = format!(r#"[{{"controlIndex":{ci},"cellIndex":1,"cellParaIndex":0}}]"#);
    core.insert_memo_by_path_json_native(0, ppi, &path, 2, 4, r#"{"author":"a","text":"b"}"#)
        .unwrap();
    let list = memo_list(&core);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["place"], "tableCell");
    assert_eq!(list[0]["text"], "b");
    let error = core
        .insert_memo_by_path_json_native(0, ppi, "not a path", 0, 1, r#"{"text":"c"}"#)
        .unwrap_err();
    assert_eq!(error.code.as_str(), "memo_invalid_range");
}

#[test]
fn typing_at_a_zero_length_memo_does_not_grow_it() {
    let mut core = core_with("hello");
    core.insert_memo_at(0, 0, 2, 2, "tester", "body").unwrap();

    core.insert_text_native(0, 0, 2, "ab").unwrap();
    assert_eq!(first_para(&core).text, "heabllo");
    let list = memo_list(&core);
    assert_eq!(
        (
            list[0]["startCharIdx"].clone(),
            list[0]["endCharIdx"].clone()
        ),
        (2.into(), 2.into()),
        "typed text stays outside a zero-length memo"
    );

    core.delete_text_native(0, 0, 2, 2).unwrap();
    assert_eq!(
        memo_list(&core)[0]["text"],
        "body",
        "deleting the typed text keeps the memo"
    );
}

#[test]
fn memo_text_control_characters_are_normalized() {
    let mut core = core_with("hello");
    let id = inserted_id(
        &core
            .insert_memo_at(0, 0, 0, 5, "tester", "a\r\nb\rc\u{1}d\te")
            .unwrap(),
    );
    let body = |core: &DocumentCore| -> Vec<String> {
        memo_fields(std::slice::from_ref(first_para(core)))[0]
            .memo_paragraphs
            .iter()
            .map(|para| para.text.clone())
            .collect()
    };
    assert_eq!(body(&core), ["a", "b", "cd\te"]);

    core.set_memo_text(id, "x\ry\u{7}").unwrap();
    assert_eq!(body(&core), ["x", "y"]);

    let saved = core.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert_eq!(
        body(&reopened),
        ["x", "y"],
        "the memo body survives an HWP save"
    );
}

/// A password guards opening only: memos in an opened password document are
/// editable, for HWP5 and HWPX alike, and opening does not touch the header.
#[test]
fn memos_in_an_opened_password_document_are_editable() {
    for rel in [
        "HWP5-password-123456.hwpx",
        "hwp3-sample16-hwp5-2024-password-123456.hwp",
    ] {
        let mut core = DocumentCore::from_bytes_with_password(&sample(rel), b"123456")
            .unwrap_or_else(|e| panic!("open {rel}: {e}"));
        let encrypted = core.document().header.encrypted;
        let first_text = core.document().sections[0]
            .paragraphs
            .iter()
            .position(|para| para.text.chars().count() >= 2)
            .expect("a paragraph with text");
        let id = inserted_id(
            &core
                .insert_memo_at(0, first_text, 0, 2, "t", "body")
                .unwrap_or_else(|e| panic!("{rel}: {}", e.to_json())),
        );
        assert_eq!(memo_list(&core)[0]["editable"], true, "{rel}");
        core.set_memo_text(id, "new").unwrap();
        core.remove_memo(id).unwrap();
        assert_eq!(core.document().header.encrypted, encrypted, "{rel}");
    }
}

#[test]
fn memos_in_a_distribution_document_are_read_only() {
    let mut core = core_with("anchor");
    let id = inserted_id(&core.insert_memo_at(0, 0, 0, 6, "t", "body").unwrap());
    core.document_mut().header.distribution = true;
    let list = memo_list(&core);
    assert_eq!(list[0]["editable"], false);
    assert_eq!(list[0]["readOnlyReason"], "protected");
    for error in [
        core.set_memo_text(id, "x").unwrap_err(),
        core.remove_memo(id).unwrap_err(),
        core.insert_memo_at(0, 0, 0, 0, "t", "y").unwrap_err(),
    ] {
        assert_eq!(error.code.as_str(), "memo_read_only");
    }
}
