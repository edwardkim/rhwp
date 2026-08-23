#![cfg(not(target_arch = "wasm32"))]

//! [#5769] Stage 1 게이트 — 선택 삭제 조각(fragment) 복원의 저장 바이트 왕복 동일성.
//!
//! capture → delete → restore 뒤 `export_hwp` 결과가 삭제 전과 **완전히 같아야** 하고,
//! 같은 삭제의 스냅샷 복원 결과와도 같아야 한다. 삭제 형태별 커버:
//! 단일 문단 부분 삭제 / 문단 중간 범위 삭제 / 표 포함 문단 삭제 /
//! 구역 선두·끝 경계 삭제.
//!
//! 조각이 되돌리는 것(각각이 깨지면 바이트가 어긋난다):
//! - 범위 문단 전체(`paragraphs[start..=end]` 원본 클론)
//! - 꼬리 line_segs — `recalculate_section_vpos` 가 start_para 이후 vertical_pos 를
//!   덮어쓰고 HWP5 직렬화기는 그 값을 그대로 기록한다
//! - 구역 `raw_stream`/`raw_provenance` — delete 가 None 으로 박는 raw 재사용 경로
//! - 캐럿 — DocInfo 봉인 다이제스트(`doc_info_model_digest`)가 DocProperties 를
//!   포함하므로 캐럿만 남겨도 raw 재사용이 깨져 IR 재직렬화 폴백으로 간다

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::parse_document;

const HONGBO: &str = "samples/20250130-hongbo-no.hwp";
const TABLE_TEST: &str = "samples/hwp_table_test.hwp";

fn load(path: &str) -> Vec<u8> {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read(&full).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn inspect(path: &str) -> Document {
    parse_document(&load(path)).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

fn char_len(doc: &Document, sec: usize, p: usize) -> usize {
    doc.sections[sec].paragraphs[p].text.chars().count()
}

fn para_count(doc: &Document, sec: usize) -> usize {
    doc.sections[sec].paragraphs.len()
}

fn find_para_with_min_text(doc: &Document, sec: usize, min: usize) -> Option<usize> {
    doc.sections[sec]
        .paragraphs
        .iter()
        .position(|p| p.text.chars().count() >= min)
}

fn find_table_para(doc: &Document, sec: usize) -> Option<usize> {
    doc.sections[sec]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
}

/// 게이트 본체 — 조각 경로와 스냅샷 경로가 모두 원본 바이트와 일치하는지 확인한다.
fn assert_byte_identity(path: &str, sec: usize, s: usize, so: usize, e: usize, eo: usize) {
    let bytes = load(path);
    let label = format!("{path} sec{sec} [{s}+{so} .. {e}+{eo}]");

    // 조각 경로 — capture 는 반드시 delete 직전에 한다(delete_text_at 의
    // char_shapes 병합이 사후 재구성을 막으므로).
    let mut core = DocumentCore::from_bytes(&bytes).expect("조각 경로 파싱");
    let before = core.export_hwp_native().expect("삭제 전 export");
    let frag = core
        .capture_delete_range_native(sec, s, e)
        .expect("조각 캡처");
    core.delete_range_native(sec, s, so, e, eo, None)
        .expect("범위 삭제");
    core.restore_delete_fragment_native(frag)
        .expect("조각 복원");
    let after = core.export_hwp_native().expect("복원 후 export");
    assert_eq!(before.len(), after.len(), "바이트 길이 불일치 ({label})");
    let first_diff = before.iter().zip(after.iter()).position(|(a, b)| a != b);
    assert!(
        first_diff.is_none(),
        "첫 바이트 불일치 @{} ({label})",
        first_diff.unwrap_or(0)
    );

    // 스냅샷 경로 대조 — 같은 삭제를 스냅샷으로 되돌려도 원본과 같아야 한다.
    let mut snap_core = DocumentCore::from_bytes(&bytes).expect("스냅샷 경로 파싱");
    let snap = snap_core.save_snapshot_native();
    snap_core
        .delete_range_native(sec, s, so, e, eo, None)
        .expect("스냅샷 경로 삭제");
    snap_core
        .restore_snapshot_native(snap)
        .expect("스냅샷 복원");
    let after_snap = snap_core.export_hwp_native().expect("스냅샷 경로 export");
    assert_eq!(before, after_snap, "스냅샷 경로 대조 실패 ({label})");
}

#[test]
fn issue_5769_single_paragraph_partial_delete_restores_bytes() {
    let doc = inspect(HONGBO);
    let sec = 0;
    let p = find_para_with_min_text(&doc, sec, 8).expect("8자 이상 문단 필요");
    let len = char_len(&doc, sec, p);
    assert_byte_identity(HONGBO, sec, p, 2, p, (len - 2).min(6));
}

#[test]
fn issue_5769_multi_paragraph_range_delete_restores_bytes() {
    let doc = inspect(HONGBO);
    let sec = 0;
    assert!(para_count(&doc, sec) >= 5, "5문단 이상 필요");
    assert_byte_identity(HONGBO, sec, 1, 0, 3, char_len(&doc, sec, 3));
}

#[test]
fn issue_5769_table_containing_selection_delete_restores_bytes() {
    let doc = inspect(TABLE_TEST);
    let sec = 0;
    let tp = find_table_para(&doc, sec).expect("표 문단 필요");
    assert_byte_identity(TABLE_TEST, sec, tp, 0, tp, char_len(&doc, sec, tp));
}

#[test]
fn issue_5769_section_head_boundary_delete_restores_bytes() {
    let doc = inspect(HONGBO);
    let sec = 0;
    assert!(para_count(&doc, sec) >= 3, "3문단 이상 필요");
    assert_byte_identity(HONGBO, sec, 0, 0, 1, char_len(&doc, sec, 1));
}

#[test]
fn issue_5769_section_tail_boundary_delete_restores_bytes() {
    let doc = inspect(HONGBO);
    let sec = 0;
    let last = para_count(&doc, sec) - 1;
    assert!(last >= 2, "3문단 이상 필요");
    assert_byte_identity(HONGBO, sec, last - 1, 0, last, char_len(&doc, sec, last));
}

#[test]
fn issue_5769_restore_without_delete_is_rejected_for_multi_para_fragment() {
    let mut core = DocumentCore::from_bytes(&load(HONGBO)).expect("파싱");
    let before = core.export_hwp_native().expect("삭제 전 export");
    let frag = core.capture_delete_range_native(0, 1, 3).expect("캡처");
    // 삭제 없이 복원 시도 → 전제 검증(문단 수 불일치)으로 거부되어 무음 중복 삽입을 막는다
    let err = core.restore_delete_fragment_native(frag).unwrap_err();
    assert!(
        err.to_string().contains("전제가 어긋났"),
        "의외의 오류: {err}"
    );

    // 거부는 저장 조각을 소비하지 않는다. 이후 실제 삭제를 적용하면 같은 ID로
    // 되돌릴 수 있어야 CommandHistory 의 재시도 가능성이 보장된다.
    let doc = inspect(HONGBO);
    core.delete_range_native(0, 1, 0, 3, char_len(&doc, 0, 3), None)
        .expect("범위 삭제");
    core.restore_delete_fragment_native(frag)
        .expect("거부 뒤 같은 조각으로 복원");
    assert_eq!(before, core.export_hwp_native().expect("복원 후 export"));
}

#[test]
fn issue_5769_discard_removes_fragment() {
    let mut core = DocumentCore::from_bytes(&load(HONGBO)).expect("파싱");
    let frag = core.capture_delete_range_native(0, 1, 3).expect("캡처");
    core.discard_delete_fragment_native(frag);
    let err = core.restore_delete_fragment_native(frag).unwrap_err();
    assert!(err.to_string().contains("없음"), "의외의 오류: {err}");
}
