//! 관계형 검사. 메모리 문단에 만든 각주·수식과 붙여넣은 글·표의 좌우 순서, 캐럿 및 상속 속성을 검사한다.
//! 주석 갱신 2026-11-21T15:21:50+09:00. 직전 파일 변경 82f43831f492beb7f3ec8c3d4cde946225b0f150 2026-11-21T10:04:32+09:00.
//! 실물 인쇄 좌표·픽셀을 대상으로 하는 검사는 없다. 실행 근거는 `mydocs/pr/pr_9265_review.md`에 있다.
//!
//! 입력은 메모리에서 생성한 API 계약이며 독립 Print의 실물 px 좌표나 golden을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{Paragraph, TitleMark};
use serde_json::Value;

fn ok(text: &str) -> Value {
    let value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(value["ok"], true, "{value}");
    value
}

fn para(core: &DocumentCore, index: usize) -> &Paragraph {
    &core.document().sections[0].paragraphs[index]
}

/// 문단의 글 사이에 각주는 `[fn]`, 그 밖의 글자 자리 개체는 `[ob]`로 끼워 읽는다. 구역·단 정의는 뺀다.
fn marked(core: &DocumentCore, index: usize) -> String {
    let paragraph = para(core, index);
    let positions = paragraph.control_text_positions();
    let chars: Vec<char> = paragraph.text.chars().collect();
    let mut out = String::new();
    for at in 0..=chars.len() {
        for (control, &position) in paragraph.controls.iter().zip(&positions) {
            if position != at {
                continue;
            }
            match control {
                Control::SectionDef(_) | Control::ColumnDef(_) => {}
                Control::Footnote(_) => out.push_str("[fn]"),
                _ => out.push_str("[ob]"),
            }
        }
        if let Some(character) = chars.get(at) {
            out.push(*character);
        }
    }
    out
}

fn paragraphs(core: &DocumentCore) -> Vec<String> {
    (0..core.document().sections[0].paragraphs.len())
        .map(|index| marked(core, index))
        .collect()
}

/// `앞뒤끝` 첫 문단의 `뒤` 앞(글자 자리 1)에 각주를 두고, `끝`만 굵게 한다. 둘째 문단은 복사 원본이다.
fn fixture(source: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, &format!("앞뒤끝{source}"))
        .unwrap();
    ok(&core.split_paragraph_native(0, 0, 3, None).unwrap());
    ok(&core
        .apply_char_format_native(0, 0, 2, 3, r#"{"bold":true}"#)
        .unwrap());
    ok(&core.insert_footnote_native(0, 0, 1).unwrap());
    assert_eq!(marked(&core, 0), "앞[fn]뒤끝");
    core
}

fn bold_at(core: &DocumentCore, para_index: usize, offset: usize) -> bool {
    let id = para(core, para_index).char_shape_id_at(offset).unwrap() as usize;
    core.document().doc_info.char_shapes[id].bold
}

#[test]
fn logical_internal_paste_of_annotated_single_paragraph_follows_both_sides_of_note() {
    for (logical, expected) in [(1, "앞가나[fn]뒤끝"), (2, "앞[fn]가나뒤끝")] {
        let mut core = fixture("가나");
        // 제목 차례 표시가 든 한 문단 조각은 순수 서식 글이 아니라 글 조각 API가 받지 않는다.
        let mut doc = core.document().clone();
        doc.sections[0].paragraphs[1].title_marks.push(TitleMark {
            char_idx: 0,
            ignore: false,
        });
        core.set_document(doc);
        core.copy_selection_logical_native(0, 1, 0, 1, 2).unwrap();
        assert!(!core.clipboard_is_single_text_native());

        let result = ok(&core.paste_internal_logical_native(0, 0, logical).unwrap());
        assert_eq!(paragraphs(&core), [expected, "가나"]);
        assert_eq!(result["paraIdx"], 0);
        assert_eq!(result["charOffset"], 3, "끝 글자 자리를 돌려준다");
        for bytes in [
            core.export_hwp_with_adapter().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(paragraphs(&reopened), [expected, "가나"]);
        }
    }
}

#[test]
fn logical_internal_paste_of_paragraphs_splits_after_note() {
    let mut core = fixture("가나");
    core.insert_paragraph_native(0, 2).unwrap();
    core.insert_text_native(0, 2, 0, "다").unwrap();
    core.copy_selection_logical_native(0, 1, 1, 2, 1).unwrap();

    let result = ok(&core.paste_internal_logical_native(0, 0, 2).unwrap());
    assert_eq!(paragraphs(&core), ["앞[fn]나", "다뒤끝", "가나", "다"]);
    assert_eq!(result["paraIdx"], 1);
}

#[test]
fn logical_internal_paste_rejects_caret_past_paragraph() {
    let mut core = fixture("가나");
    core.copy_selection_logical_native(0, 1, 0, 1, 2).unwrap();
    assert!(core.paste_internal_logical_native(0, 0, 5).is_err());
    assert_eq!(paragraphs(&core), ["앞[fn]뒤끝", "가나"]);
}

#[test]
fn table_created_right_after_note_inherits_caret_character_shape() {
    let mut core = fixture("");
    // 논리 캐럿 2는 각주 바로 뒤, 굵지 않은 `뒤` 앞이다.
    let result = ok(&core.create_table_native(0, 0, 2, 1, 1).unwrap());
    assert_eq!(paragraphs(&core)[..4], ["앞[fn]", "[ob]", "", "뒤끝"]);
    let table_para = result["paraIdx"].as_u64().unwrap() as usize;
    let Control::Table(table) = &para(&core, table_para).controls[0] else {
        panic!("표 문단이 아니다");
    };
    let id = table.cells[0].paragraphs[0].char_shape_id_at(0).unwrap() as usize;
    assert!(!core.document().doc_info.char_shapes[id].bold);
}

#[test]
fn control_pasted_right_after_note_gives_caret_character_shape_to_its_empty_paragraph() {
    let mut core = fixture("");
    core.insert_paragraph_native(0, 2).unwrap();
    ok(&core
        .create_shape_control_native(
            0,
            2,
            0,
            1000,
            1000,
            0,
            0,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .unwrap());
    ok(&core.copy_control_native(0, 2, &[], 0).unwrap());

    let result = ok(&core.paste_control_native(0, 0, 2).unwrap());
    assert_eq!(paragraphs(&core)[..4], ["앞[fn]", "[ob]", "", "뒤끝"]);
    assert_eq!(result["paraIdx"], 1);
    assert!(
        !bold_at(&core, 2, 0),
        "개체 아래 빈 문단은 캐럿 글자 모양을 잇는다"
    );
}

#[test]
fn paste_between_adjacent_inline_objects_uses_the_selected_gap() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나X").unwrap();
    core.split_paragraph_native(0, 0, 2, None).unwrap();
    for equation in ["x", "y"] {
        core.insert_equation_native(0, 0, 1, equation, 1000, 0)
            .unwrap();
    }
    assert_eq!(marked(&core, 0), "가[ob][ob]나");
    core.copy_selection_logical_native(0, 1, 0, 1, 1).unwrap();
    core.paste_internal_logical_native(0, 0, 2).unwrap();
    assert_eq!(marked(&core, 0), "가[ob]X[ob]나");
}
