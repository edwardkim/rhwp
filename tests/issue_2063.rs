//! Issue #2063: 초대형 표(52,694셀 CellBreak)에서 셀 측정 O(n²) 폭증 → 렌더 타임아웃.
//!
//! 재현 문서 (tracked 공개 샘플): `samples/issue2063_huge_cellbreak_table.hwp`
//! (화성시 사무전결 처리규칙 [별표 2], 행정규칙 공개 문서 admrul, HWP5, 694KB).
//! 단일 표 5,277행 × 10열 = **52,694셀**, 쪽나눔 CellBreak.
//!
//! 결함 본질: `cell_units_uncached` 가 **표 단위 불변량** `has_visible_text_with_nested_table`
//! (전체 셀 스캔)를 셀별 함수 안에서 계산. `cell_units` 는 셀별 메모이즈되지만 캐시를
//! 채우는 과정에서 셀마다 전체 셀(52,694)을 스캔 → 52,694² ≈ **28억 회**(각 회 문단·컨트롤
//! 중첩 순회) → dump-pages 47s→timeout, render-diff >420s TIMEOUT.
//!
//! 정정: 표 포인터 키 캐시 `table_nested_text_flag_cache` 로 표 단위 1회 계산하도록 hoist.
//! O(셀²) → O(셀). 수정 후 dump-pages 2s, render-diff 283s(배치 임계 이내). 페이지 수·좌표
//! 불변(순수 최적화, render-diff 0.00px PASS).
//!
//! 본 테스트는 (1) 페이지네이션이 **완주**함(= O(n²) 재발 시 CI 타임아웃으로 검출)과
//! (2) 산출 페이지 수 안정을 가드한다. (과분할 자체 — rhwp 213 vs 한글 162 — 는 행높이
//! 드리프트 #1842/#1937 트랙으로 분리.)

use std::fs;
use std::path::Path;

fn load_page_count(rel: &str) -> u32 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    doc.page_count()
}

#[test]
fn huge_cellbreak_table_paginates_without_quadratic_blowup() {
    // O(n²)(28억 회) 재발 시 이 호출이 완주하지 못해 CI 타임아웃으로 검출된다.
    let pages = load_page_count("samples/issue2063_huge_cellbreak_table.hwp");
    // 수정은 순수 최적화라 페이지 수 불변(213). 하한(150)은 페이지네이션 붕괴/조기중단,
    // 상한(260)은 O(n²) 우회로 인한 재분할 폭주를 잡는다. (한글 2022 = 162, #1842 타깃.)
    assert!(
        (150..=260).contains(&pages),
        "issue2063: 페이지 수 {pages} 가 기대 범위(150..=260) 밖 — \
         52,694셀 CellBreak 표 페이지네이션 회귀 의심",
    );
}
