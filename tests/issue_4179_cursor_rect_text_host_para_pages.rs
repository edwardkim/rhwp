//! Issue #4179: 텍스트가 있는 표 호스트 문단에서 `getCursorRect` 가 후보 페이지
//! 전부의 render tree 를 지어보던 회귀 가드 — #4126(빈 문단) 미커버 자매 케이스.
//!
//! 재현: `samples/issue1949_giant_cell_nested_tables_perf.hwp` 의 호스트 문단(0,0)에
//! 텍스트를 삽입하면 그 텍스트는 분할 표 뒤 = 마지막 페이지에만 렌더된다. 캐럿을
//! 그 텍스트 끝에 두고 rect 를 질의하면 #4127 가드(텍스트 0자 한정)가 미발동,
//! 페이지 루프가 후보 115쪽을 순서대로 빌드하다 마지막 후보에서야 히트한다
//! (studio 실측: 열기 시 단일 long task 2.56s, native 재현 2.54s).
//!
//! 정정: `find_text_scan_pages_for_paragraph` 가 pagination 메타데이터만으로
//! 순수-중간 연속 컷(cont=true && end_cut 비어있지 않음) 페이지를 후보에서 뺀다 —
//! 텍스트는 표 시작 페이지(cont=false) 또는 표 소진 페이지(end_cut=[])에만 렌더
//! 가능하다. 115 후보 → 2. 스캔 순서·결과 좌표 불변.
//!
//! 판별은 #4126 과 같은 작업량 카운터 상한 — 카운터는 프로세스 누적이므로
//! 이 파일에는 테스트를 1개만 둔다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::perf_counters;

#[test]
fn text_host_para_cursor_rect_builds_few_page_trees() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    let bytes = fs::read(&path).expect("read sample");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    assert_eq!(doc.page_count(), 115, "issue1949 기준 쪽수 핀");

    // "(1)" 변형 재현: 호스트 문단에 타이핑 — 별도 체크인 픽스처 불필요.
    // 실물 "(1)" 파일의 캐럿 메타데이터가 (0,0,7) = 이 텍스트의 끝이었다 (#4180).
    doc.insert_text(0, 0, 0, "rhwp pe")
        .expect("호스트 문단 텍스트 삽입");
    assert_eq!(doc.page_count(), 115, "삽입 후 쪽수 불변 핀");

    perf_counters::reset();
    let rect = doc
        .get_cursor_rect(0, 0, 7)
        .expect("텍스트 있는 호스트 문단 캐럿 rect");

    // 스킵이 결과를 바꾸지 않는다는 계약: 표 뒤 텍스트의 캐럿은 마지막 페이지(114).
    assert!(
        rect.contains("\"pageIndex\":114"),
        "표 뒤 텍스트 캐럿은 마지막 페이지에 렌더되어야 함: {rect}"
    );

    let builds = perf_counters::page_tree_builds();
    // 필터 후 후보 = 표 시작 페이지(cont=false) + 표 소진 페이지(end_cut=[]) = 2.
    // 수정 원복 시 후보 115쪽 전부를 빌드해 이 상한을 크게 넘는다 (실측 ~115회).
    assert!(
        builds <= 3,
        "#4179 회귀: 텍스트 있는 호스트 문단 캐럿 배치가 page tree 를 {builds}회 빌드 \
         (기대 ≤3). find_text_scan_pages_for_paragraph 필터가 무력화됐는지 확인."
    );
}
