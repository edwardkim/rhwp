//! 관계형·고정형: 실패 전후 문서·등록 상태의 관계와 기존 등록 상한을 확인한다.
//! 주석 갱신: 2026-11-07T02:29:09+09:00 (새 파일).
//! 입력은 공개 API로 만든 빈 문서와 1×1 표다. 실제 문서의 배치·px·쪽수를 검사하지 않는다.
//! 시각 측정 비해당: 글꼴 프로그램 없이 주소 오류의 원자성과 이름→언어 슬롯 참조를 검사한다.
//! 근거: mydocs/pr/assets/planet6897_20261106/pr8552/fontname-error-atomicity.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::Font;
use rhwp::DocumentCore;
use serde_json::Value;

const FACE: &str = "검토용등록글꼴";
const PROPS: &str = r#"{"fontName":"검토용등록글꼴"}"#;

fn table() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let result: Value =
        serde_json::from_str(&core.create_table_native(0, 0, 0, 1, 1).unwrap()).unwrap();
    let para = result["paraIdx"].as_u64().unwrap() as usize;
    let control = result["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_cell_native(0, para, control, 0, 0, 0, "셀ABC한글")
        .unwrap();
    (core, para, control)
}

fn snapshot(core: &DocumentCore) -> (Value, bool) {
    let document = core.document();
    (
        serde_json::json!({
            "fonts": document.doc_info.font_faces,
            "shapes": document.doc_info.char_shapes,
            "paragraphs": document.sections.iter().map(|section| &section.paragraphs).collect::<Vec<_>>(),
            "sectionRaw": document.sections.iter().map(|section| &section.raw_stream).collect::<Vec<_>>(),
            "docInfoRaw": document.doc_info.raw_stream,
        }),
        document.doc_info.raw_stream_dirty,
    )
}

fn assert_face(core: &DocumentCore, para: &Paragraph) {
    let info = &core.document().doc_info;
    let shape = &info.char_shapes[para.char_shape_id_at(0).unwrap() as usize];
    for (slot, id) in shape.font_ids.iter().enumerate() {
        assert_eq!(info.font_faces[slot][*id as usize].name, FACE);
    }
}

#[test]
fn rejected_flat_cell_font_name_does_not_register_or_dirty_document() {
    for bad_part in 0..5 {
        let (mut core, para, control) = table();
        let mut address = [0, para, control, 0, 0];
        address[bad_part] = 999;
        let before = snapshot(&core);
        assert!(core
            .apply_char_format_in_cell_native(
                address[0], address[1], address[2], address[3], address[4], 0, 1, PROPS,
            )
            .is_err());
        assert_eq!(
            snapshot(&core),
            before,
            "주소 축 {bad_part}의 실패는 문서 불변"
        );
    }
}

#[test]
fn rejected_nested_or_empty_path_does_not_register_or_dirty_document() {
    for use_empty_path in [false, true] {
        let (mut core, para, control) = table();
        let path = if use_empty_path {
            Vec::new()
        } else {
            vec![(control, 0, 0), (999, 0, 0)]
        };
        let before = snapshot(&core);
        assert!(core
            .apply_char_format_in_cell_by_path(0, para, &path, 0, 1, PROPS)
            .is_err());
        assert_eq!(snapshot(&core), before, "실패한 경로 {path:?}는 문서 불변");
    }
}

fn with_full_latin_slot(core: &mut DocumentCore, contains_target: bool) {
    let mut document = core.document().clone();
    document.doc_info.font_faces[1] = (0..256)
        .map(|index| Font {
            name: if contains_target && index == 255 {
                FACE.to_string()
            } else {
                format!("계약용기존글꼴{index}")
            },
            alt_type: 1,
            ..Default::default()
        })
        .collect();
    core.set_document(document);
}

#[test]
fn later_slot_capacity_error_preserves_all_earlier_slots_and_dirty_state() {
    let (mut core, para, control) = table();
    with_full_latin_slot(&mut core, false);
    let before = snapshot(&core);
    let error = core
        .apply_char_format_in_cell_native(0, para, control, 0, 0, 0, 6, PROPS)
        .unwrap_err();
    assert!(error.to_string().contains("등록할 수 없음"));
    assert_eq!(snapshot(&core), before, "뒤 슬롯 상한의 실패도 문서 불변");
}

#[test]
fn full_slot_reuses_its_existing_face_while_other_slots_register_the_name() {
    let (mut core, para, control) = table();
    with_full_latin_slot(&mut core, true);
    core.apply_char_format_in_cell_native(0, para, control, 0, 0, 0, 6, PROPS)
        .unwrap();
    assert_eq!(core.document().doc_info.font_faces[1].len(), 256);
    let Control::Table(outer) = &core.document().sections[0].paragraphs[para].controls[control]
    else {
        panic!("공개 API가 만든 표");
    };
    assert_face(&core, &outer.cells[0].paragraphs[0]);
}

#[test]
fn validated_flat_and_nested_cells_still_receive_all_language_font_refs() {
    let (mut core, para, control) = table();
    assert!(core
        .document()
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .all(|font| font.name != FACE));
    core.apply_char_format_in_cell_native(0, para, control, 0, 0, 0, 6, PROPS)
        .unwrap();
    let Control::Table(outer) = &core.document().sections[0].paragraphs[para].controls[control]
    else {
        panic!("공개 API가 만든 표");
    };
    assert_face(&core, &outer.cells[0].paragraphs[0]);

    // 새 입력을 써서 바깥 셀에서 이미 적용한 글꼴이 중첩 셀로 상속되는 거짓 통과를 막는다.
    let (mut core, para, control) = table();
    assert!(core
        .document()
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .all(|font| font.name != FACE));
    let created: Value = serde_json::from_str(
        &core
            .create_table_in_cell_by_path_native(
                0,
                para,
                &[(control, 0, 0)],
                0,
                1,
                1,
                None,
                None,
                true,
            )
            .unwrap(),
    )
    .unwrap();
    let inner_para = created["cellParaIdx"].as_u64().unwrap() as usize;
    let inner_control = created["controlIdx"].as_u64().unwrap() as usize;
    let path = [(control, 0, inner_para), (inner_control, 0, 0)];
    core.insert_text_in_cell_by_path(0, para, &path, 0, "중첩ABC")
        .unwrap();
    core.apply_char_format_in_cell_by_path(0, para, &path, 0, 5, PROPS)
        .unwrap();
    let Control::Table(outer) = &core.document().sections[0].paragraphs[para].controls[control]
    else {
        unreachable!();
    };
    let Control::Table(inner) = &outer.cells[0].paragraphs[inner_para].controls[inner_control]
    else {
        panic!("공개 API가 만든 중첩 표");
    };
    assert_face(&core, &inner.cells[0].paragraphs[0]);
}
