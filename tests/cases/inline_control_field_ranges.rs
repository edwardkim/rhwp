//! 누름틀 앞에 인라인 개체를 넣어도 누름틀 범위가 그 개체를 가리키지 않는다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const KINDS: [&str; 6] = ["각주", "미주", "수식", "새 번호", "그림", "도형"];

/// 이름·범위·값만 비교한다. 저장 위치(startPos·endPos)는 앞에 넣은 개체만큼 밀린다.
fn fields(core: &DocumentCore) -> Value {
    let mut values: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    for value in values.as_array_mut().unwrap() {
        value.as_object_mut().unwrap().remove("startPos");
        value.as_object_mut().unwrap().remove("endPos");
    }
    values
}

/// "앞 뒤끝"의 끝 앞(3번 글자)에 값이 든 누름틀이 있는 문단.
fn fixture() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞 뒤끝").unwrap();
    core.insert_click_here_field_at(0, 0, 3, "안내문", "메모", "뒤 필드", true)
        .unwrap();
    core.set_field_value_by_name("뒤 필드", "기존").unwrap();
    assert_eq!(fields(&core)[0]["value"], "기존");
    core
}

/// 누름틀보다 앞인 1번 글자 자리에 개체를 넣는다.
fn insert_before_field(core: &mut DocumentCore, kind: &str) {
    match kind {
        "각주" => {
            core.insert_footnote_native(0, 0, 1).unwrap();
        }
        "미주" => {
            core.insert_endnote_native(0, 0, 1).unwrap();
        }
        "수식" => {
            core.insert_equation_native(0, 0, 1, "x+1", 1000, 0)
                .unwrap();
        }
        "새 번호" => {
            core.insert_new_number_native(0, 0, 1, 3).unwrap();
        }
        "그림" => {
            let png = std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/logo/logo-16.png"
            ))
            .unwrap();
            core.insert_picture_native(
                0,
                0,
                1,
                &[],
                &png,
                1200,
                1200,
                16,
                16,
                "png",
                "그림",
                None,
                None,
            )
            .unwrap();
        }
        "도형" => {
            core.create_shape_control_native(
                0,
                0,
                1,
                4000,
                3000,
                7500,
                9000,
                false,
                "InFrontOfText",
                "rectangle",
                false,
                false,
                &[],
            )
            .unwrap();
        }
        _ => unreachable!(),
    }
}

#[test]
fn inline_controls_before_field_keep_field_live_and_after_reopen() {
    let mut broken = Vec::new();
    for kind in KINDS {
        let mut core = fixture();
        let expected = fields(&core);
        let controls = core.document().sections[0].paragraphs[0].controls.len();
        insert_before_field(&mut core, kind);
        let p = &core.document().sections[0].paragraphs[0];
        assert_eq!(p.controls.len(), controls + 1, "{kind}");
        let offsets = p.char_offsets.clone();
        if fields(&core) != expected {
            broken.push(format!("{kind} 편집 중"));
        }
        for (format, bytes) in [
            ("HWP", core.export_hwp_native().unwrap()),
            ("HWPX", core.export_hwpx_native().unwrap()),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            if fields(&reopened) != expected
                || reopened.document().sections[0].paragraphs[0].char_offsets != offsets
            {
                broken.push(format!("{kind} {format} 재열기"));
            }
        }
    }
    assert!(broken.is_empty(), "누름틀 범위를 잃은 경로: {broken:?}");
}

#[test]
fn inline_control_before_active_field_keeps_typing_inside_field() {
    let mut broken = Vec::new();
    for kind in KINDS {
        let mut core = fixture();
        let end = fields(&core)[0]["endCharIdx"].as_u64().unwrap() as usize;
        assert!(core.set_active_field(0, 0, end), "{kind}");
        insert_before_field(&mut core, kind);
        // 앞에 넣은 개체는 글자가 아니므로 누름틀 끝 글자 위치는 그대로다.
        core.insert_text_native(0, 0, end, "이어 입력").unwrap();
        if fields(&core)[0]["value"] != "기존이어 입력" {
            broken.push(kind);
        }
    }
    assert!(
        broken.is_empty(),
        "활성 누름틀 밖에 입력된 경로: {broken:?}"
    );
}
