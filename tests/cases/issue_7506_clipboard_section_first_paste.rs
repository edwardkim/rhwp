//! [#7506] 구역 첫 문단을 맨 앞부터 여러 문단 복사해 붙이면 그 문단 글이 그려지지 않았다.
//!
//! 복사본에서 구역·단 정의를 떼면서 그 자리(16칸)를 `char_offsets` 에 남겼다. 붙일 때
//! `merge_from` 이 `char_count` 를 글자 수로 다시 세므로, 갭 뒤 글자가 줄 밖으로 밀렸다.
//! 한글 클립보드 조각도 첫 문단에 두 정의를 달고 와 같은 정리를 거친다.

use rhwp::document_core::DocumentCore;

fn rendered_text(core: &DocumentCore, para_idx: u64) -> String {
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["paraIdx"].as_u64() == Some(para_idx))
        .map(|run| run["text"].as_str().unwrap())
        .collect()
}

#[test]
fn pasting_from_section_first_paragraph_draws_every_pasted_character() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다라마바사아")
        .unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.apply_char_format_native(0, 0, 0, 2, r#"{"bold":true}"#)
        .unwrap();
    let source = core.document().sections[0].paragraphs[0].clone();
    // 구역·단 정의 자리 뒤에서 글이 시작해야 이 경로를 탄다.
    assert_eq!(source.char_offsets[0], 16);

    core.copy_selection_native(0, 0, 0, 1, 4).unwrap();
    core.paste_internal_native(0, 1, 4).unwrap();

    let merged = &core.document().sections[0].paragraphs[1];
    assert_eq!(merged.text, "마바사아가나다라");
    assert_eq!(rendered_text(&core, 1), "마바사아가나다라");
    // 자리를 거둘 때 글자 모양도 함께 옮겨야 굵게가 "가나"에 남는다.
    for i in 0..4 {
        assert_eq!(merged.char_shape_id_at(4 + i), source.char_shape_id_at(i));
    }
    assert_ne!(source.char_shape_id_at(0), source.char_shape_id_at(2));
    // 거둔 자리 안의 글자 모양이 한 위치에 겹쳐 남지 않는다.
    assert!(merged
        .char_shapes
        .windows(2)
        .all(|pair| pair[0].start_pos < pair[1].start_pos));
}

#[test]
fn pasting_hancom_fragment_draws_its_section_first_paragraph() {
    // 한글 클립보드 조각의 첫 문단은 구역 정의(secd)·단 정의(cold)를 달고 온다.
    let json = r#"{
        "ro": {
            "hp": "p0",
            "p0": {"id": 0, "np": "p1", "ru": [{"cp": "", "ch": [
                {"cc": 2, "ci": 1936024420, "co": "s0"},
                {"cc": 2, "ci": 1668246628, "co": "c0"},
                {"t": "가나다라"}]}]},
            "p1": {"id": 1, "ru": [{"cp": "", "ch": [{"t": "마바사아"}]}]}
        },
        "cs": {"s0": {}, "c0": {}}
    }"#;
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "XY").unwrap();
    core.paste_hwp_json_native(0, 0, 1, json).unwrap();

    assert_eq!(core.document().sections[0].paragraphs[0].text, "X가나다라");
    assert_eq!(rendered_text(&core, 0), "X가나다라");
}
