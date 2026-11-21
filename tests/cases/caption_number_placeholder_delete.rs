//! 관계형: 캡션 번호 자리표시자를 지우면 자동 번호 컨트롤도 함께 지워진다.
//! 2026-11-19 KST 작성. 렌더링 위치를 고정하지 않고 캡션 문단의 글·컨트롤·저장 후 재열기만 본다.
//!
//! 새 캡션은 "그림 " 뒤에 자동 번호를 공백 자리표시자 한 글자(8유닛)로 둔다. 수정 전에는
//! 그 글자를 지워도 번호 컨트롤이 남아, 캡션 글을 모두 새 글로 바꾸면 번호가 글 뒤에 다시 붙었다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 본문 첫 문단에 그림을 넣고 아래 캡션을 단 문서와 그림의 컨트롤 번호.
fn captioned_picture() -> (DocumentCore, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_picture_native(
        0,
        0,
        0,
        &[],
        include_bytes!("../../assets/logo/logo-16.png"),
        4000,
        3000,
        16,
        16,
        "png",
        "캡션 그림",
        None,
        None,
    )
    .unwrap();
    let ctrl = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Picture(_)))
        .expect("그림");
    core.set_picture_properties_native(
        0,
        0,
        ctrl,
        r#"{"hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850,"captionIncludeMargin":false}"#,
    )
    .unwrap();
    (core, ctrl)
}

/// 캡션 첫 문단의 자동 번호 수.
fn caption_numbers(core: &DocumentCore, ctrl: usize) -> usize {
    match &core.document().sections[0].paragraphs[0].controls[ctrl] {
        Control::Picture(picture) => picture.caption.as_ref().expect("캡션").paragraphs[0]
            .controls
            .iter()
            .filter(|control| matches!(control, Control::AutoNumber(_)))
            .count(),
        _ => panic!("그림이 아닙니다"),
    }
}

#[test]
fn deleting_caption_number_placeholder_removes_auto_number() {
    let (mut core, ctrl) = captioned_picture();
    assert_eq!(caption_numbers(&core, ctrl), 1, "새 캡션에는 번호가 있다");
    let length = core
        .get_cell_paragraph_length_native(0, 0, ctrl, 0, 0)
        .unwrap();
    core.insert_text_in_cell_native(0, 0, ctrl, 0, 0, length, "검토 시작")
        .unwrap();

    // 번호까지 앞 글을 모두 지우고 새 글만 남긴다.
    core.delete_text_in_cell_native(0, 0, ctrl, 0, 0, 0, length)
        .unwrap();
    assert_eq!(
        core.get_text_in_cell_native(0, 0, ctrl, 0, 0, 0, 100)
            .unwrap(),
        "검토 시작"
    );
    assert_eq!(
        caption_numbers(&core, ctrl),
        0,
        "지운 자리표시자의 번호도 지운다"
    );

    // 저장 후 다시 열어도 번호가 되살아나지 않는다.
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            reopened
                .get_text_in_cell_native(0, 0, ctrl, 0, 0, 0, 100)
                .unwrap(),
            "검토 시작"
        );
        assert_eq!(caption_numbers(&reopened, ctrl), 0);
    }
}

#[test]
fn deleting_text_before_caption_number_keeps_it() {
    let (mut core, ctrl) = captioned_picture();
    // "그림 " 만 지우면 번호는 그대로다.
    core.delete_text_in_cell_native(0, 0, ctrl, 0, 0, 0, 3)
        .unwrap();
    assert_eq!(caption_numbers(&core, ctrl), 1);
}

#[test]
fn plain_text_delete_keeps_auto_number() {
    // 미주 앞 공백 정리·파서처럼 글자만 다듬는 지우기는 자리표시자를 지워도 번호를 남긴다.
    let (core, ctrl) = captioned_picture();
    let Control::Picture(picture) = &core.document().sections[0].paragraphs[0].controls[ctrl]
    else {
        panic!("그림이 아닙니다");
    };
    let mut para = picture.caption.as_ref().expect("캡션").paragraphs[0].clone();
    let length = para.text.chars().count();
    assert_eq!(para.delete_text_at(0, length), length);
    assert_eq!(
        para.controls
            .iter()
            .filter(|control| matches!(control, Control::AutoNumber(_)))
            .count(),
        1
    );
}
