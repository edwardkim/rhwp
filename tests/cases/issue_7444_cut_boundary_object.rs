//! #7444: 복사와 삭제가 같은 선택의 경계 개체를 다르게 다루면 오려 붙일 때 개체가 늘어난다.
//! 글자 축 API와 캐럿 축 API 모두 복사한 개체만 지우고, 선택 밖 원문과 저장 속성을 보존한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

fn document_with_table() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "abc").unwrap();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.create_table_native(0, 1, 0, 1, 2).unwrap();
    core.insert_text_native(0, 2, 0, "xyz").unwrap();
    // 목적지는 개체 없는 빈 문단으로 두어 붙이는 위치의 논리 축 문제와 분리한다.
    core.split_paragraph_native(0, 2, 3, None).unwrap();
    assert_eq!(core.document().sections[0].paragraphs.len(), 4);
    assert_eq!(table_count(&core), 1);
    core
}

fn table_count(core: &DocumentCore) -> usize {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter(|control| matches!(control, Control::Table(_)))
        .count()
}

fn cut_and_paste_at_end(
    core: &mut DocumentCore,
    start_para: usize,
    start_offset: usize,
    end_para: usize,
    end_offset: usize,
) -> usize {
    core.copy_selection_native(0, start_para, start_offset, end_para, end_offset)
        .unwrap();
    core.delete_range_native(0, start_para, start_offset, end_para, end_offset, None)
        .unwrap();
    let after_delete = table_count(core);
    let paragraphs = &core.document().sections[0].paragraphs;
    let last = paragraphs.len() - 1;
    assert!(paragraphs[last].text.is_empty());
    assert!(paragraphs[last].controls.is_empty());
    core.paste_internal_native(0, last, 0).unwrap();
    after_delete
}

#[test]
fn cut_with_table_in_selection_interior_moves_one_table() {
    let mut core = document_with_table();
    let after_delete = cut_and_paste_at_end(&mut core, 0, 1, 2, 2);
    assert_eq!(
        after_delete, 0,
        "선택 중간의 표는 오려 두면 원문에서 사라진다"
    );
    assert_eq!(table_count(&core), 1, "한 번 오려 붙여도 표는 하나다");
}

#[test]
fn cut_ending_at_table_paragraph_start_does_not_duplicate_table() {
    let mut core = document_with_table();
    let after_delete = cut_and_paste_at_end(&mut core, 0, 1, 1, 0);
    assert_eq!(
        table_count(&core),
        1,
        "같은 선택을 오려 붙이면 표 수가 보존돼야 한다 (삭제 직후: {after_delete})"
    );
}

#[test]
fn cut_starting_at_table_paragraph_start_does_not_duplicate_table() {
    let mut core = document_with_table();
    let after_delete = cut_and_paste_at_end(&mut core, 1, 0, 2, 2);
    assert_eq!(
        table_count(&core),
        1,
        "같은 선택을 오려 붙이면 표 수가 보존돼야 한다 (삭제 직후: {after_delete})"
    );
}

fn objects(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|control| match control {
            Control::Picture(pic) => Some(format!(
                "picture:{}:{}:{}:{:?}:{:?}:{:?}",
                pic.common.description,
                pic.common.width,
                pic.common.height,
                pic.crop,
                pic.image_attr,
                pic.img_dim
            )),
            Control::Footnote(note) => Some(format!(
                "note:{}:{}:{}:{}",
                note.paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                note.before_decoration_letter,
                note.after_decoration_letter,
                note.number_shape
            )),
            Control::Endnote(note) => Some(format!(
                "note:{}:{}:{}:{}",
                note.paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                note.before_decoration_letter,
                note.after_decoration_letter,
                note.number_shape
            )),
            _ => None,
        })
        .collect()
}

fn boundary_document() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가🦦나\t다끝").unwrap();
    core.split_paragraph_native(0, 0, 6, None).unwrap();
    core.insert_text_native(0, 1, 0, "앞뒤끝").unwrap();
    core.split_paragraph_native(0, 1, 3, None).unwrap();
    core.apply_char_format_native(0, 0, 0, 1, r#"{"bold":true}"#)
        .unwrap();
    core.apply_char_format_native(0, 1, 1, 3, r#"{"italic":true}"#)
        .unwrap();
    let png = include_bytes!("../../assets/logo/logo-16.png");
    for (paragraph, offset, name) in [(0, 2, "옮길 그림"), (1, 2, "남길 그림")] {
        let result: serde_json::Value = serde_json::from_str(
            &core
                .insert_picture_native(
                    0,
                    paragraph,
                    offset,
                    &[],
                    png,
                    1200,
                    1400,
                    16,
                    16,
                    "png",
                    name,
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
        let index = result["controlIdx"].as_u64().unwrap() as usize;
        core.set_picture_properties_native(
            0,
            paragraph,
            index,
            r#"{"treatAsChar":true,"transparency":25}"#,
        )
        .unwrap();
    }
    core.insert_footnote_native(0, 0, 4).unwrap();
    core.insert_endnote_native(0, 1, 1).unwrap();
    for paragraph in 0..2 {
        let index = core.document().sections[0].paragraphs[paragraph]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Footnote(_) | Control::Endnote(_)))
            .unwrap();
        core.insert_text_in_footnote_native(0, paragraph, index, 0, 1, "각주 본문🦦")
            .unwrap();
        let para = &mut core.document_mut().sections[0].paragraphs[paragraph];
        para.align_ctrl_data_records();
        para.ctrl_data_records[index] = Some(vec![0, 1, 2, 3]);
        match &mut para.controls[index] {
            Control::Footnote(note) => {
                note.before_decoration_letter = '[' as u16;
                note.after_decoration_letter = ']' as u16;
                note.instance_id = 100 + paragraph as u32;
            }
            Control::Endnote(note) => {
                note.before_decoration_letter = '[' as u16;
                note.after_decoration_letter = ']' as u16;
                note.instance_id = 100 + paragraph as u32;
            }
            _ => unreachable!(),
        }
    }
    core
}

#[test]
fn logical_boundary_cut_preserves_outside_text_picture_note_properties_and_history() {
    let mut core = boundary_document();
    let original = objects(&core);
    let note_ids = |core: &DocumentCore| {
        core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter_map(|control| match control {
                Control::Footnote(note) => Some(note.instance_id),
                Control::Endnote(note) => Some(note.instance_id),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let original_ids = note_ids(&core);
    let leading_shape = core.document().sections[0].paragraphs[0].char_shape_id_at(0);
    let trailing_shape = core.document().sections[0].paragraphs[1].char_shape_id_at(2);
    let before = core.save_snapshot_native();
    // 시작 그림 앞부터 끝 문단의 미주 뒤까지: 두 경계의 개체를 담되 끝 그림은 남긴다.
    core.copy_selection_logical_native(0, 0, 2, 1, 2).unwrap();
    let fragment = core.capture_delete_range_native(0, 0, 1).unwrap();
    core.delete_range_logical_native(0, 0, 2, 1, 2).unwrap();
    assert_eq!(core.document().sections[0].paragraphs[0].text, "가🦦뒤끝");
    assert_eq!(objects(&core), vec![original[3].clone()]);
    assert_eq!(
        core.document().sections[0].paragraphs[0].char_shape_id_at(0),
        leading_shape
    );
    assert_eq!(
        core.document().sections[0].paragraphs[0].char_shape_id_at(3),
        trailing_shape
    );
    let after = core.save_snapshot_native();
    core.restore_delete_fragment_native(fragment).unwrap();
    assert_eq!(objects(&core), original);
    assert_eq!(note_ids(&core), original_ids);
    assert_eq!(
        core.document().sections[0].paragraphs[0].text,
        "가🦦나\t다끝"
    );
    core.restore_snapshot_native(after).unwrap();
    assert_eq!(objects(&core), vec![original[3].clone()]);
    core.restore_snapshot_native(before).unwrap();
    assert_eq!(objects(&core), original);
    assert_eq!(note_ids(&core), original_ids);
    core.delete_range_logical_native(0, 0, 2, 1, 2).unwrap();
    core.paste_internal_native(0, 1, 0).unwrap();
    // 붙이기는 기존 식별자 재발급 계약을 따르고, 각주 내용과 장식은 그대로 옮긴다.
    let pasted_ids = note_ids(&core);
    assert_eq!(pasted_ids.len(), 2);
    assert_ne!(pasted_ids, original_ids);
    assert_ne!(pasted_ids[0], pasted_ids[1]);
    let mut actual = objects(&core);
    let mut expected = original;
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
    // 각주 CTRL_DATA도 조각에서 제 컨트롤과 함께 이동한다.
    assert_eq!(
        core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.ctrl_data_records)
            .filter(|record| **record == Some(vec![0, 1, 2, 3]))
            .count(),
        2
    );
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let mut saved = objects(&reopened);
        saved.sort();
        assert_eq!(saved, expected);
        assert_eq!(reopened.document().bin_data_content.len(), 2);
        for entry in &reopened.document().bin_data_content {
            assert_eq!(
                entry.data.load_limited(1024 * 1024).unwrap(),
                include_bytes!("../../assets/logo/logo-16.png")
            );
        }
        assert_eq!(
            reopened.document().sections[0].paragraphs[0].text,
            "가🦦뒤끝"
        );
    }
}

#[test]
fn logical_single_slot_and_before_after_boundaries_agree_with_copy() {
    for (from, to, remaining, removed) in [
        (2, 3, "가🦦나\t다끝", 1),
        (5, 6, "가🦦나\t다끝", 1),
        (3, 4, "가🦦\t다끝", 0),
        (1, 2, "가나\t다끝", 0),
    ] {
        let mut core = boundary_document();
        let count = objects(&core).len();
        core.copy_selection_logical_native(0, 0, from, 0, to)
            .unwrap();
        core.delete_range_logical_native(0, 0, from, 0, to).unwrap();
        assert_eq!(core.document().sections[0].paragraphs[0].text, remaining);
        assert_eq!(objects(&core).len(), count - removed);
        core.paste_internal_native(0, 2, 0).unwrap();
        assert_eq!(objects(&core).len(), count);
    }
}

#[test]
fn unsupported_same_position_controls_and_inverted_ranges_keep_original() {
    let mut core = boundary_document();
    core.insert_endnote_native(0, 0, 4).unwrap();
    let before = core.export_hwp_native().unwrap();
    assert!(core.delete_range_logical_native(0, 0, 5, 0, 6).is_err());
    assert_eq!(core.export_hwp_native().unwrap(), before);
    assert!(core.delete_range_logical_native(0, 0, 6, 0, 7).is_err());
    assert_eq!(core.export_hwp_native().unwrap(), before);
    assert!(core.delete_range_logical_native(0, 0, 3, 0, 2).is_err());
    assert_eq!(core.export_hwp_native().unwrap(), before);
}

#[test]
fn boundary_with_field_rejects_before_mutation_and_keeps_clipboard() {
    let mut core = boundary_document();
    core.copy_selection_logical_native(0, 1, 0, 1, 1).unwrap();
    core.insert_click_here_field_at(0, 0, 0, "안내문", "", "이름", true)
        .unwrap();
    let before = core.export_hwp_native().unwrap();
    let clipboard = core.get_clipboard_text_native();
    assert!(core.delete_range_native(0, 0, 0, 1, 2, None).is_err());
    assert_eq!(core.export_hwp_native().unwrap(), before);
    assert_eq!(core.get_clipboard_text_native(), clipboard);
}
