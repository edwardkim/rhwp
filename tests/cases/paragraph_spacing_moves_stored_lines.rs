//! 관계형: paragraph_spacing_moves_stored_lines의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T21:45:03+09:00 39b4f81f6aab4cc08c6145ed6911db2a8fe5125e.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. spacing-before/spacing/spacing-after 각 1쪽 Native/fresh WASM 최저 100.00%; 독립 Print 확대 판독 완료. 뒤 간격은 앞 문단·자기 줄 보존과 후속 문단 이동, snapshot undo/redo 관계로 검사한다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
//! 문단 간격을 바꾸면 그 문단과 뒤 문단의 저장 세로 위치가 바로 옮겨지고 저장 뒤에도 같다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn caret_y(core: &DocumentCore, para: usize) -> f64 {
    let rect: serde_json::Value =
        serde_json::from_str(&core.get_cursor_rect_native(0, para, 0).unwrap()).unwrap();
    rect["y"].as_f64().unwrap()
}

#[test]
fn spacing_before_moves_the_paragraph_and_its_successor() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞문단뒤문단").unwrap();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.split_paragraph_native(0, 1, 0, None).unwrap();
    let before = [caret_y(&core, 1), caret_y(&core, 2)];
    let original_shape = core.document().sections[0].paragraphs[1].para_shape_id;
    core.apply_para_format_native(0, 1, r#"{"spacingBefore":1500}"#)
        .unwrap();
    let after = [caret_y(&core, 1), caret_y(&core, 2)];
    assert!(after[0] > before[0] + 5.0, "{before:?} -> {after:?}");
    assert!((after[1] - after[0] - (before[1] - before[0])).abs() < 0.05);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!([caret_y(&reopened, 1), caret_y(&reopened, 2)], after);
    }
    // 서식 되돌리기 경로도 원래 세로 위치로 돌아간다.
    core.set_para_shape_id_native(0, 1, original_shape).unwrap();
    assert_eq!([caret_y(&core, 1), caret_y(&core, 2)], before);
}

#[test]
fn spacing_after_moves_only_successors_and_snapshot_undo_redo_restores_both() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞문단뒤문단").unwrap();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.split_paragraph_native(0, 1, 0, None).unwrap();
    let before = [caret_y(&core, 0), caret_y(&core, 1), caret_y(&core, 2)];
    let undo = core.save_snapshot_native();
    core.apply_para_format_native(0, 1, r#"{"spacingAfter":1500}"#)
        .unwrap();
    let after = [caret_y(&core, 0), caret_y(&core, 1), caret_y(&core, 2)];
    assert_eq!(&after[..2], &before[..2]);
    assert!(after[2] > before[2]);
    let redo = core.save_snapshot_native();
    core.restore_snapshot_native(undo).unwrap();
    assert_eq!(
        [caret_y(&core, 0), caret_y(&core, 1), caret_y(&core, 2)],
        before
    );
    core.restore_snapshot_native(redo).unwrap();
    assert_eq!(
        [caret_y(&core, 0), caret_y(&core, 1), caret_y(&core, 2)],
        after
    );
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            [
                caret_y(&reopened, 0),
                caret_y(&reopened, 1),
                caret_y(&reopened, 2)
            ],
            after
        );
    }
}
