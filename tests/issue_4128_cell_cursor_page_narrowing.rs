//! Issue #4128: 셀 캐럿 질의(`get_cursor_rect_in_cell` 등)가
//! `find_pages_for_paragraph` 의 para_index-만 매칭 후보(분할 표가 걸친 전 페이지)를
//! 오름차순으로 render tree 를 지어보며 탐색해, 115쪽 거대 셀 문서에서 뒤쪽 행
//! 콜드 질의 1회가 O(pages) 빌드였던 회귀 가드 (기존 평균 ~57, 최악 115).
//!
//! 정정: `find_pages_for_cell_position` 이 PartialTable 의 start_row/end_row/
//! start_cut/end_cut 를 cell_units 서수와 대조해 대상 위치가 실제 렌더되는
//! 페이지(보통 1, 컷 경계 2)만 반환한다 — render tree 없이 pagination 메타데이터만
//! 사용 (src/document_core/commands/text_editing.rs).
//!
//! 재현 문서: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
//! (바깥 3×1 RowBreak 표, cell[2] = 2507문단 + 중첩표, 115쪽에 걸침).
//!
//! 판별은 작업량 카운터(`diagnostics::perf_counters::PAGE_TREE_BUILDS`) 상한 —
//! 수정 원복 시 깊은 행 질의가 후보를 순차 빌드해 상한을 크게 넘는다(red→green).
//! 카운터는 프로세스 누적이므로 이 파일에는 테스트를 1개만 둔다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::perf_counters;

#[test]
fn deep_cell_cursor_queries_build_few_page_trees() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    let bytes = fs::read(&path).expect("read sample");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    assert_eq!(doc.page_count(), 115, "issue1949 기준 쪽수 핀");

    // 거대 셀(cell_idx=2, 2507문단) 앞/중간/끝 위치의 콜드 캐럿 질의.
    // 페이지 tree 캐시가 비어 있는 상태에서 시작해 3회 질의 누적 빌드를 잰다.
    perf_counters::reset();
    let mut page_indices = Vec::new();
    for cell_para_idx in [0u32, 1250, 2400] {
        let rect = doc
            .get_cursor_rect_in_cell(0, 0, 2, 2, cell_para_idx, 0)
            .unwrap_or_else(|_| panic!("cell_para {cell_para_idx} 캐럿 rect 실패"));
        let page: u64 = rect
            .split("\"pageIndex\":")
            .nth(1)
            .and_then(|s| s.split([',', '}']).next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(|| panic!("rect JSON 파싱 실패: {rect}"));
        page_indices.push(page);
    }

    // 행 서수가 클수록 뒤 페이지 — 끝 위치는 문서 후반부여야 한다.
    assert!(
        page_indices[0] < page_indices[1] && page_indices[1] < page_indices[2],
        "행 순서와 페이지 순서 불일치: {page_indices:?}"
    );
    assert!(
        page_indices[2] >= 100,
        "끝 위치가 문서 후반부 페이지가 아님: {page_indices:?}"
    );

    let builds = perf_counters::page_tree_builds();
    // 수정 후 실측: 질의당 1~2회 × 3회. 수정 원복 시 오름차순 후보 순회로
    // 앞(≈1)+중간(≈50)+끝(≈110) ≈ 160회 이상 — 상한 12 를 크게 넘는다.
    assert!(
        builds <= 12,
        "#4128 회귀: 셀 캐럿 콜드 질의 3회가 page tree 를 {builds}회 빌드 (기대 ≤12). \
         find_pages_for_cell_position 좁히기가 무력화됐는지 확인."
    );
}
