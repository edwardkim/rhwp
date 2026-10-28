//! 본문 개체만 지울 때 같은 문단의 누름틀 주소·값·저장 경계를 보존한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

fn fields(core: &DocumentCore) -> Value {
    let mut fields: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    for field in fields.as_array_mut().unwrap() {
        // 개체 슬롯이 빠지면 HWP 원시 좌표는 달라져도 본문 글자 범위는 그대로다.
        field.as_object_mut().unwrap().remove("startPos");
        field.as_object_mut().unwrap().remove("endPos");
    }
    fields
}

fn fixture(picture: bool, object_before_field: bool, value: &str) -> (DocumentCore, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞 🦦 가운데 뒤").unwrap();
    let offset = if object_before_field { 0 } else { 7 };
    if picture {
        core.insert_picture_native(
            0,
            0,
            offset,
            &[],
            include_bytes!("../../assets/logo/logo-16.png"),
            4000,
            3000,
            16,
            16,
            "png",
            "지울 그림",
            None,
            None,
        )
        .unwrap();
        let ci = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|control| matches!(control, Control::Picture(_)))
            .unwrap();
        core.set_picture_properties_native(
            0,
            0,
            ci,
            r#"{"treatAsChar":false,"textWrap":"InFrontOfText"}"#,
        )
        .unwrap();
    } else {
        core.create_shape_control_native(
            0,
            0,
            offset,
            4000,
            3000,
            7500,
            12000,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .unwrap();
    }
    core.insert_click_here_field_at(0, 0, 4, "안내문", "메모", "남을 필드", true)
        .unwrap();
    if !value.is_empty() {
        core.set_field_value_by_name("남을 필드", value).unwrap();
    }
    // 편집 중 임시 상태가 아니라 공개 HWP 저장·재열기 경로로 만든 유효한 입력이다.
    let core = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(fields(&core).as_array().unwrap().len(), 1);
    assert_eq!(fields(&core)[0]["value"], value);
    let para = &core.document().sections[0].paragraphs[0];
    let ci = para
        .controls
        .iter()
        .position(|control| {
            if picture {
                matches!(control, Control::Picture(_))
            } else {
                matches!(control, Control::Shape(_))
            }
        })
        .unwrap();
    assert_eq!(para.field_ranges.len(), 1);
    assert_eq!(para.field_ranges[0].control_idx > ci, object_before_field);
    (core, ci)
}

fn check_delete(picture: bool, object_before_field: bool, value: &str) {
    let (mut core, ci) = fixture(picture, object_before_field, value);
    let before = fields(&core);
    let text = core.document().sections[0].paragraphs[0].text.clone();
    let controls = core.document().sections[0].paragraphs[0].controls.len();
    let snapshot = core.save_snapshot_native();
    if picture {
        core.delete_picture_control_native(0, 0, ci).unwrap();
    } else {
        core.delete_shape_control_native(0, 0, ci).unwrap();
    }
    assert_eq!(
        fields(&core),
        before,
        "개체 삭제가 이웃 누름틀을 지우면 안 된다"
    );
    assert_eq!(core.document().sections[0].paragraphs[0].text, text);
    assert_eq!(
        core.document().sections[0].paragraphs[0].controls.len(),
        controls - 1
    );
    for (format, bytes) in [
        ("HWP", core.export_hwp_native().unwrap()),
        ("HWPX", core.export_hwpx_native().unwrap()),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(fields(&reopened), before, "{format} 누름틀 보존");
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, text);
        assert_eq!(
            reopened.document().sections[0].paragraphs[0].controls.len(),
            controls - 1
        );
    }
    core.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(fields(&core), before);
    assert_eq!(
        core.document().sections[0].paragraphs[0].controls.len(),
        controls
    );
}

#[test]
fn deleting_shape_before_field_preserves_field() {
    check_delete(false, true, "");
}

#[test]
fn deleting_picture_before_field_preserves_field() {
    check_delete(true, true, "");
}

#[test]
fn deleting_shape_before_filled_field_preserves_field() {
    check_delete(false, true, "입력한 🦦 값");
}

#[test]
fn deleting_picture_before_filled_field_preserves_field() {
    check_delete(true, true, "입력한 🦦 값");
}

#[test]
fn deleting_shape_after_field_preserves_field() {
    check_delete(false, false, "입력한 🦦 값");
}

#[test]
fn deleting_picture_after_field_preserves_field() {
    check_delete(true, false, "입력한 🦦 값");
}

fn check_active_field(picture: bool) {
    for object_before_field in [true, false] {
        let (mut core, ci) = fixture(picture, object_before_field, "기존");
        let end = fields(&core)[0]["endCharIdx"].as_u64().unwrap() as usize;
        assert!(core.set_active_field(0, 0, end));
        if picture {
            core.delete_picture_control_native(0, 0, ci).unwrap();
        } else {
            core.delete_shape_control_native(0, 0, ci).unwrap();
        }
        // 필드를 다시 선택하지 않아도 활성 필드 끝의 입력은 같은 필드에 붙어야 한다.
        core.insert_text_native(0, 0, end, "이어 입력").unwrap();
        assert_eq!(fields(&core)[0]["value"], "기존이어 입력");
    }
}

#[test]
fn active_body_field_survives_shape_removal() {
    check_active_field(false);
}

#[test]
fn active_body_field_survives_picture_removal() {
    check_active_field(true);
}
