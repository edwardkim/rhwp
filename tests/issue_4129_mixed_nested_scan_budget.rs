//! Issue #4129: 분할 표 컷 높이 평가(`row_cut_content_height`)가 fragment 마다 부르는
//! `mixed_nested_flow_extra_from_cut` 이 셀의 문단 P개 × 유닛 U개 전체 재스캔이라,
//! 115-fragment 거대 셀 문서에서 paginate() 가 O(fragments×P×U) ≈ 수십억 반복
//! (native debug 실측 16.9s, 전체 open 의 99%)이던 회귀 가드.
//!
//! 정정: mixed 유닛의 `para_idx` 단조성(cell_units_uncached 의 단일 문단 루프)을
//! 이용한 1-pass run-walk 로 호출당 O(U) — paginate 16,923ms → 431ms
//! (src/renderer/layout/table_layout.rs).
//!
//! 재현 문서: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
//! (3×1 RowBreak 표, cell[2] = 2507문단, 115쪽 분할).
//!
//! 판별은 작업량 카운터(`diagnostics::perf_counters::MIXED_NESTED_UNITS_SCANNED`,
//! 함수 내부 루프의 실제 유닛 방문 횟수 누적) 상한 — per-para 재스캔이 되살아나면
//! 문서 열기 1회의 방문 총량이 수억~수십억으로 폭발해 상한을 넘는다(red→green:
//! 카운터 패치를 유지한 채 수정 커밋만 원복하면 FAIL).
//! 카운터는 프로세스 누적이므로 이 파일에는 테스트를 1개만 둔다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::perf_counters;

#[test]
fn giant_cell_paginate_scan_budget_holds() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    let bytes = fs::read(&path).expect("read sample");

    perf_counters::reset();
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    assert_eq!(doc.page_count(), 115, "issue1949 기준 쪽수 핀");

    let scanned = perf_counters::mixed_nested_units_scanned();
    // 카운터 배선 자체가 끊기면 테스트가 공회전하므로 하한도 함께 고정한다
    // (115 fragment 문서에서 이 경로는 반드시 수만 단위로 방문된다).
    assert!(
        scanned > 10_000,
        "#4129 카운터 미배선 의심: 스캔 방문이 {scanned}회뿐 — \
         MIXED_NESTED_UNITS_SCANNED 누적 지점이 살아있는지 확인."
    );
    // 수정 후 실측 대비 ~10× 여유 상한. per-para O(P×U) 재스캔이 되살아나면
    // P≈2507 배로 폭발해(수억 이상) 이 상한을 수십 배 넘는다.
    assert!(
        scanned <= 20_000_000,
        "#4129 회귀: 문서 열기 1회의 mixed_nested 유닛 방문이 {scanned}회 \
         (기대 ≤20,000,000). 컷 높이 평가에 per-para 재스캔이 되살아났는지 확인."
    );
}
