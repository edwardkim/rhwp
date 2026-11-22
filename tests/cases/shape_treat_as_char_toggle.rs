//! 떠 있는 도형을 글자처럼 취급으로 바꾸면 한/글 12.30처럼 줄 높이가 도형을 따라 늘고,
//! 도형은 본문 첫 줄 위에 선다. 다시 끄면 줄 높이가 글자 높이로 돌아간다.
//! 종전에는 도형 속성 setter가 저장 줄을 다시 새기지 않아 도형이 이전 줄 높이에 걸려
//! 본문 위쪽 밖으로 솟고, 그 줄 높이가 그대로 저장됐다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn json_usize(json: &str, key: &str) -> usize {
    let at = json.find(&format!("\"{key}\":")).expect(key) + key.len() + 3;
    json[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .expect(key)
}

const SHAPE_H: i32 = 8000;

fn first_line_height(core: &DocumentCore) -> i32 {
    core.document().sections[0].paragraphs[0].line_segs[0].line_height
}

#[test]
fn toggling_treat_as_char_reflows_the_host_line() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, "앞 글").expect("text");
    core.split_paragraph_native(0, 0, 3, None).expect("split");
    core.insert_text_native(0, 1, 0, "다음 문단")
        .expect("text 2");
    let shape = core
        .create_shape_control_native(
            0,
            0,
            0,
            12000,
            SHAPE_H as u32,
            5000,
            5000,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .expect("shape");
    let (para, ctrl) = (
        json_usize(&shape, "paraIdx"),
        json_usize(&shape, "controlIdx"),
    );
    let text_line = first_line_height(&core);
    assert!(
        text_line < SHAPE_H,
        "전제: 떠 있는 도형은 줄 높이에 들지 않는다"
    );

    core.set_shape_properties_native(
        0,
        para,
        ctrl,
        r#"{"treatAsChar":true,"textWrap":"TopAndBottom"}"#,
    )
    .expect("tac on");
    assert!(
        first_line_height(&core) >= SHAPE_H,
        "글자처럼 취급한 도형의 줄 높이({}) ≥ 도형 높이({SHAPE_H})",
        first_line_height(&core)
    );
    // 다시 연 문서도 같은 줄 높이로 시작한다(저장 줄이 새로 새겨졌다).
    let reopened =
        DocumentCore::from_bytes(&core.export_hwp_native().expect("hwp")).expect("reopen");
    assert!(first_line_height(&reopened) >= SHAPE_H);

    core.set_shape_properties_native(0, para, ctrl, r#"{"treatAsChar":false}"#)
        .expect("tac off");
    assert_eq!(first_line_height(&core), text_line);
}
