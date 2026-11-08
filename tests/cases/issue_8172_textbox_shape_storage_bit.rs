//! #8172: 편집으로 만든 글상자는 SHAPE_COMPONENT 저장 워드에 글상자 비트(0x0100_0000)를 담는다.
//! 공개 편집 API로 만든 메모리 입력이다. 한컴 2024가 이 비트 없는 글상자 HWP를 거부하고
//! 이 비트만 세우면 정상으로 여는 것은 이슈의 MCP 실측이 근거다. 이 검사는 그 저장 계약만 본다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::DrawingObjAttr;

const BIT: u32 = DrawingObjAttr::TEXT_BOX_STORAGE_BIT;

fn core_with(kinds: &[&str]) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for (i, kind) in kinds.iter().enumerate() {
        core.create_shape_control_native(
            0,
            0,
            0,
            17008,
            5669,
            (i as u32) * 20000,
            2000,
            false,
            "TopAndBottom",
            kind,
            false,
            false,
            &[],
        )
        .unwrap();
    }
    core
}

/// 첫 문단의 그리기 개체를 (컨트롤 번호, 글상자 유무, 저장 워드) 로 앞에서부터 돌려준다.
fn shapes(core: &DocumentCore) -> Vec<(usize, bool, u32)> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| match c {
            Control::Shape(s) => s
                .drawing()
                .map(|d| (i, d.text_box.is_some(), d.shape_attr.flip)),
            _ => None,
        })
        .collect()
}

fn assert_bit_matches_text_box(core: &DocumentCore, expected_shapes: usize, stage: &str) {
    let found = shapes(core);
    assert_eq!(found.len(), expected_shapes, "{stage}: 그리기 개체 수");
    for (i, has_text_box, flip) in found {
        assert_eq!(
            flip & BIT != 0,
            has_text_box,
            "{stage}: 컨트롤 {i} 글상자={has_text_box} flip={flip:#010x}"
        );
        // 한컴 기본 비트는 글상자 유무와 관계없이 유지한다.
        assert_ne!(flip & 0x0008_0000, 0, "{stage}: 컨트롤 {i} 기본 비트");
    }
}

#[test]
fn created_textbox_carries_storage_bit_and_rectangle_does_not() {
    let core = core_with(&["rectangle", "textbox"]);
    let found = shapes(&core);
    assert_eq!(
        found.iter().map(|s| s.1).collect::<Vec<_>>(),
        vec![false, true],
        "사각형 하나와 글상자 하나"
    );
    assert_bit_matches_text_box(&core, 2, "생성 직후");
}

#[test]
fn storage_bit_survives_hwp_save_and_reopen() {
    let core = core_with(&["rectangle", "textbox"]);
    for (name, bytes) in [
        ("export_hwp_native", core.export_hwp_native().unwrap()),
        (
            "export_hwp_with_adapter_snapshot",
            core.export_hwp_with_adapter_snapshot().unwrap(),
        ),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_bit_matches_text_box(&reopened, 2, name);
        assert_eq!(
            shapes(&reopened).iter().filter(|s| s.1).count(),
            1,
            "{name}: 글상자 하나 보존"
        );
    }
}

#[test]
fn attaching_and_detaching_text_box_keeps_storage_bit_in_sync() {
    let mut core = core_with(&["rectangle"]);
    let (rect, has_text_box, _) = shapes(&core)[0];
    assert!(!has_text_box);

    assert_eq!(
        core.set_text_box_at(0, rect, true).unwrap(),
        r#"{"ok":true}"#
    );
    assert_bit_matches_text_box(&core, 1, "글상자 붙인 뒤");
    assert!(shapes(&core)[0].1);
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_bit_matches_text_box(&reopened, 1, "붙인 뒤 저장·다시 열기");

    assert_eq!(
        core.set_text_box_at(0, rect, false).unwrap(),
        r#"{"ok":true}"#
    );
    assert_bit_matches_text_box(&core, 1, "글상자 뗀 뒤");
    assert!(!shapes(&core)[0].1);
}
