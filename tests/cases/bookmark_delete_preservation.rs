//! 책갈피 삭제는 보이지 않는 8유닛 슬롯만 지우고 본문·서식·이웃 필드를 보존한다.

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

fn fields(core: &DocumentCore) -> Value {
    let mut values: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    for value in values.as_array_mut().unwrap() {
        value.as_object_mut().unwrap().remove("startPos");
        value.as_object_mut().unwrap().remove("endPos");
    }
    values
}

fn fixture(field_before: Option<bool>, value: &str, shapes: bool, title: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let text = "왼쪽 ANCHOR🦦 가운데 오른쪽";
    core.insert_text_native(0, 0, 0, text).unwrap();
    if shapes {
        for y in [9000, 10000, 11000] {
            core.create_shape_control_native(
                0,
                0,
                9,
                4000,
                3000,
                7500,
                y,
                false,
                "InFrontOfText",
                "rectangle",
                false,
                false,
                &[],
            )
            .unwrap();
        }
    }
    if let Some(before) = field_before {
        core.insert_click_here_field_at(
            0,
            0,
            if before { 0 } else { 9 },
            "안내문",
            "메모",
            "남을 필드",
            true,
        )
        .unwrap();
        if !value.is_empty() {
            core.set_field_value_by_name("남을 필드", value).unwrap();
        }
    }
    let len = core.document().sections[0].paragraphs[0]
        .text
        .chars()
        .count();
    core.apply_char_format_native(0, 0, len - 3, len, r#"{"bold":true}"#)
        .unwrap();

    // add_bookmark의 별도 좌표 결함에 기대지 않는다. 공개 API의 저장본에 책갈피
    // XML 하나를 넣고 다시 파싱하여 유효한 입력의 삭제 계약만 검사한다.
    let bytes = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches("ANCHOR").count(), 1);
            let replacement = if title {
                "<hp:titleMark ignore=\"1\"/></hp:t><hp:ctrl><hp:bookmark name=\"지울 책갈피\"/></hp:ctrl><hp:t>"
            } else {
                "</hp:t><hp:ctrl><hp:bookmark name=\"지울 책갈피\"/></hp:ctrl><hp:t>"
            };
            bytes = xml.replace("ANCHOR", replacement).into_bytes();
        }
        output
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let core = DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap();
    let p = &core.document().sections[0].paragraphs[0];
    assert!(!p.text.contains("ANCHOR"));
    assert_eq!(
        fields(&core).as_array().unwrap().len(),
        usize::from(field_before.is_some())
    );
    if let Some(before) = field_before {
        assert_eq!(fields(&core)[0]["value"], value);
        assert_eq!(p.field_ranges[0].control_idx < bookmark(&core), before);
    }
    assert_eq!(!p.title_marks.is_empty(), title);
    core
}

fn bookmark(core: &DocumentCore) -> usize {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Bookmark(_)))
        .unwrap()
}

fn formats(core: &DocumentCore) -> Vec<Value> {
    let p = &core.document().sections[0].paragraphs[0];
    (0..p.text.chars().count())
        .map(|i| {
            serde_json::to_value(
                &core.document().doc_info.char_shapes[p.char_shape_id_at(i).unwrap() as usize],
            )
            .unwrap()
        })
        .collect()
}

fn check_delete(field: bool, value: &str, shapes: bool) {
    let mut core = fixture(field.then_some(false), value, shapes, false);
    let text = core.document().sections[0].paragraphs[0].text.clone();
    let original = core.document().sections[0].paragraphs[0].clone();
    let original_fields = fields(&core);
    let original_formats = formats(&core);
    // HWP 파서는 raw_data·attr 비트를 채우므로 같은 형식으로 저장·재열기한 원본과 비교한다.
    let saved_formats = [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .map(|bytes| formats(&DocumentCore::from_bytes(&bytes).unwrap()));
    let snapshot = core.save_snapshot_native();
    let ci = bookmark(&core);
    core.delete_bookmark_native(0, 0, ci).unwrap();
    let after = &core.document().sections[0].paragraphs[0];
    assert_eq!(after.text, text);
    assert_eq!(after.controls.len(), original.controls.len() - 1);
    assert_eq!(fields(&core), original_fields);
    assert_eq!(formats(&core), original_formats);
    // secd/cold 16유닛과 '왼쪽 ' 3유닛 뒤의 책갈피 8유닛만 빠진다.
    assert_eq!(after.char_count, original.char_count - 8);
    assert_eq!(
        after.char_offsets,
        original
            .char_offsets
            .iter()
            .map(|&p| if p >= 27 { p - 8 } else { p })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        after.control_text_positions(),
        original
            .control_text_positions()
            .into_iter()
            .enumerate()
            .filter_map(|(i, p)| (i != ci).then_some(p))
            .collect::<Vec<_>>()
    );
    for (bytes, saved_formats) in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .into_iter()
    .zip(saved_formats)
    {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, text);
        assert_eq!(fields(&reopened), original_fields);
        assert_eq!(formats(&reopened), saved_formats);
        assert_eq!(
            reopened.document().sections[0].paragraphs[0].controls.len(),
            original.controls.len() - 1
        );
    }
    core.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(fields(&core), original_fields);
    assert_eq!(formats(&core), original_formats);
    assert_eq!(
        serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
}

#[test]
fn bookmark_delete_preserves_raw_text_formats_and_following_shapes() {
    check_delete(false, "", true);
}

#[test]
fn bookmark_delete_preserves_empty_and_filled_following_fields() {
    for value in ["", "입력한 값"] {
        check_delete(true, value, false);
    }
}

#[test]
fn bookmark_delete_keeps_following_body_field_active() {
    let mut core = fixture(Some(false), "기존", false, false);
    let end = fields(&core)[0]["endCharIdx"].as_u64().unwrap() as usize;
    assert!(core.set_active_field(0, 0, end));
    core.delete_bookmark_native(0, 0, bookmark(&core)).unwrap();
    core.insert_text_native(0, 0, end, "이어 입력").unwrap();
    assert_eq!(fields(&core)[0]["value"], "기존이어 입력");
}

#[test]
fn ambiguous_marker_slots_reject_bookmark_delete_without_mutating_document() {
    for mut core in [
        fixture(Some(true), "기존", false, false),
        fixture(None, "", false, true),
    ] {
        let before = format!("{:?}", core.document());
        assert!(core.delete_bookmark_native(0, 0, bookmark(&core)).is_err());
        assert_eq!(format!("{:?}", core.document()), before);
    }
}
