#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

fn load() -> DocumentCore {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/3-09월_교육_통합_2023.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn state(core: &DocumentCore) -> (u32, u32, Vec<u8>, String) {
    let Control::Picture(p) = &core.document().sections[0].paragraphs[236].controls[0] else {
        panic!("실물 그림 경로 전제");
    };
    (
        p.common.height,
        p.shape_attr.current_height,
        p.shape_attr.raw_rendering.clone(),
        serde_json::to_string(&p.shape_attr).unwrap(),
    )
}

#[test]
fn actual_resize_undo_redo_restores_original_transform_without_document_snapshot() {
    let mut core = load();
    let original = state(&core);
    assert_eq!(original.1, 7295);
    assert_eq!(original.2.len(), 146);
    let id = core
        .capture_picture_transform_native(r#"{"sec":0,"ppi":236,"ci":0}"#)
        .unwrap();
    core.set_picture_properties_native(0, 236, 0, &format!(r#"{{"height":{}}}"#, original.0 + 400))
        .unwrap();
    let resized = state(&core);
    assert!(resized.2.is_empty());
    assert_ne!(resized.0, original.0);
    for _ in 0..3 {
        core.swap_picture_transform_native(id).unwrap();
        assert_eq!(
            state(&core),
            original,
            "실제 Undo는 원본 행렬과 파생 치수까지 복원"
        );
        core.swap_picture_transform_native(id).unwrap();
        assert_eq!(state(&core), resized, "Redo는 실제 변경 상태 복원");
    }
    core.discard_picture_transform_native(id);
    assert!(core.swap_picture_transform_native(id).is_err());
    assert_eq!(state(&core), resized, "해제된 ID는 문서를 변경하지 않는다");
}

#[test]
fn capture_and_discard_preserve_picture_state_and_section_passthrough() {
    let mut core = load();
    let before = state(&core);
    let raw_before = core.document().sections[0].raw_stream.clone();
    assert!(raw_before.is_some(), "원본 구역 패스스루가 있는 실물 문서");
    let id = core
        .capture_picture_transform_native(r#"{"sec":0,"ppi":236,"ci":0}"#)
        .unwrap();
    assert_eq!(state(&core), before);
    assert_eq!(core.document().sections[0].raw_stream, raw_before);
    core.discard_picture_transform_native(id);
    assert_eq!(state(&core), before);
    assert_eq!(core.document().sections[0].raw_stream, raw_before);
    assert!(core.swap_picture_transform_native(id).is_err());
    assert_eq!(state(&core), before);
    assert_eq!(core.document().sections[0].raw_stream, raw_before);
}

#[test]
fn invalid_picture_journal_target_fails_without_mutation() {
    let mut core = load();
    let before = state(&core);
    for target in [
        r#"{"sec":0,"ppi":999999,"ci":0}"#,
        r#"{"sec":0,"ppi":236,"ci":0,"cellPath":[{}]}"#,
        r#"{"sec":0,"ppi":236,"ci":0,"headerFooter":{"kind":"footer","outerParaIdx":236,"outerControlIdx":0}}"#,
    ] {
        assert!(core.capture_picture_transform_native(target).is_err());
        assert_eq!(state(&core), before);
    }
    assert!(core.swap_picture_transform_native(u32::MAX).is_err());
    assert_eq!(state(&core), before);
}
