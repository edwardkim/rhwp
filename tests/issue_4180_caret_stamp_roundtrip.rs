//! Issue #4180: 문서 캐럿 메타데이터가 "마지막 본문 편집 위치"가 아니라
//! **저장 시점 캐럿**이어야 한다는 계약 (한컴 의미론).
//!
//! 신고 재현: 호스트 문단에 타이핑 후 저장된 파일이 캐럿 (0,0,7) 을 갖고,
//! 열기 시 그 텍스트가 렌더되는 마지막 페이지로 캐럿이 복원됐다. 편집별
//! 스탬핑이 남긴 위치를 저장 흐름의 `setCaretPosition` 이 덮어써야 한다.

use std::fs;
use std::path::Path;

#[test]
fn set_caret_position_roundtrips_through_save() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    let bytes = fs::read(&path).expect("read sample");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");

    // 편집별 스탬핑이 (0,0,7) 을 남긴다 — 신고 파일과 같은 상태
    doc.insert_text(0, 0, 0, "rhwp pe").expect("insert");
    assert_eq!(
        doc.get_caret_position().expect("caret after edit"),
        r#"{"sectionIndex":0,"paragraphIndex":0,"charOffset":7}"#,
        "편집 스탬핑 전제 확인"
    );

    // 저장 시점 캐럿이 편집 스탬프를 이긴다 — 신고 버그의 회귀 계약
    doc.set_caret_position(0, 0, 3);
    let saved = doc.export_hwp().expect("export");

    let doc2 = rhwp::wasm_api::HwpDocument::from_bytes(&saved).expect("reload");
    assert_eq!(
        doc2.get_caret_position().expect("caret after reload"),
        r#"{"sectionIndex":0,"paragraphIndex":0,"charOffset":3}"#,
        "저장→재열기 후 캐럿은 set 값이어야 함 (편집 위치 7 이 아니라 3)"
    );
}

#[test]
fn set_caret_position_out_of_range_is_ignored() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    let bytes = fs::read(&path).expect("read sample");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");

    let before = doc.get_caret_position().expect("caret before");
    doc.set_caret_position(99, 0, 0); // 존재하지 않는 구역 — 무시가 계약
    assert_eq!(doc.get_caret_position().expect("caret after"), before);
}
