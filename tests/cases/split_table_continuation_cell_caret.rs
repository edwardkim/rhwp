//! 관계형 회귀 — 뒤쪽 조각에만 있는 표 셀의 캐럿은 앞쪽 셀보다 뒤쪽에 놓인다.
//! 주석 갱신 2026-11-28T07:34:34+09:00. 직전 파일 커밋 2026-11-28T07:20:14+09:00 7664a3ea5ccb01e57b31352d2f484f4e78c6a0cf.
//! 입력: tests/fixtures/issue7419-boundaries/partial-nested-wrapper.hwpx. 캐럿 조회 API의 쪽 소속을 검증한다.
//! 검사 대상은 캐럿의 쪽 인덱스이며 시각 일치율 측정은 해당하지 않는다. 근거: mydocs/pr/archives/pr_9474_review.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

// 바깥 2칸 표가 쪽을 넘고, 둘째 칸(cell 1)은 다음 쪽에만 있으며 그 첫 문단은 중첩 표 하나뿐이다.
const SAMPLE: &str = "tests/fixtures/issue7419-boundaries/partial-nested-wrapper.hwpx";

fn caret(doc: &HwpDocument, cell: usize) -> Value {
    let rect = doc
        .get_cursor_rect_in_cell_native(0, 4, 0, cell, 0, 0)
        .unwrap_or_else(|e| panic!("cell {cell} 캐럿: {e}"));
    serde_json::from_str(&rect).unwrap()
}

#[test]
fn continuation_cell_with_only_a_nested_table_has_a_caret() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let first = caret(&doc, 0);
    let second = caret(&doc, 1);
    // 둘째 칸 캐럿은 그 칸이 놓인 뒤쪽에 선다.
    assert!(
        second["pageIndex"].as_u64() > first["pageIndex"].as_u64(),
        "첫 칸 {first} / 둘째 칸 {second}"
    );
}
