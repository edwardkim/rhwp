//! Issue #4126: 표/도형만 호스팅하는 빈 문단(텍스트 0자)에서 `getCursorRect` 가
//! `find_pages_for_paragraph` 후보 페이지 전부의 render tree 를 지어보며 텍스트
//! 매칭을 시도하던 회귀 가드. 115쪽 분할 표 문서에서 콜드 캐럿 배치 1회가
//! O(pages) render tree 빌드(≈5초, wasm 실측 5,093ms)였다.
//!
//! 재현 문서: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
//! (3×1 RowBreak 표가 PartialTable continuation 으로 115쪽에 걸침, 문단 0 은
//! 표만 호스팅하는 빈 문단).
//!
//! 정정: 대상 문단에 텍스트가 없으면 페이지 순회 자체를 건너뛰고 앵커 위치
//! 폴백으로 간다 (src/document_core/queries/cursor_rect.rs).
//!
//! 판별은 시계가 아니라 작업량 카운터(`diagnostics::perf_counters::PAGE_TREE_BUILDS`)
//! 상한이다 — 수정 원복 시 빌드가 115회로 폭증해 상한(8)을 넘는다(red→green).
//! 카운터는 프로세스 누적이므로 이 파일에는 테스트를 1개만 둔다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::perf_counters;

#[test]
fn empty_host_para_cursor_rect_builds_few_page_trees() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    let bytes = fs::read(&path).expect("read sample");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    assert_eq!(doc.page_count(), 115, "issue1949 기준 쪽수 핀");

    // 콜드 캐럿 배치: 문서 열기 직후 (0,0,0) — 표 호스트 빈 문단.
    perf_counters::reset();
    let rect = doc
        .get_cursor_rect(0, 0, 0)
        .expect("빈 호스트 문단 캐럿 rect");
    assert!(
        rect.contains("\"pageIndex\""),
        "rect JSON 형식이 아님: {rect}"
    );

    let builds = perf_counters::page_tree_builds();
    // 수정 후 실측: 폴백 경로의 1~2회. 수정 원복 시 후보 115쪽 전부를 빌드해
    // 이 상한을 크게 넘는다. 상한 8 은 폴백 경로 변형 여지를 둔 여유값.
    assert!(
        builds <= 8,
        "#4126 회귀: 빈 호스트 문단 캐럿 배치가 page tree 를 {builds}회 빌드 \
         (기대 ≤8). find_pages_for_paragraph 후보 전수 순회가 되살아났는지 확인."
    );
}
