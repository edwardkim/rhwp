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
//! (2) 산출 페이지 수 안정을 가드한다. #1842/#2070 도 같은 문서의 `page_count()` 를
//! 별도 testcase에서 다시 수행하고 있었으나, #6360에서 중복 조판을 제거하고 가장
//! 엄격한 현재 pin(161쪽)을 이 sentinel 하나로 통합한다.

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
    // #5922 이후 rhwp pin은 161쪽이다. 한글 정답지는 162쪽이고 잔여 −1은 행 경계
    // sub-pt 적산 축이다. 159p면 #5922 여백 재개방 회귀, 213p면 #1842 synthetic
    // 라인높이 팽창 회귀, 완주 실패면 #2063 O(n²) 회귀다.
    assert_eq!(
        pages, 161,
        "issue2063/#1842/#2070: 화성시 별표2 CellBreak 표 pin은 161쪽이다 \
         (한글 정답 162, 잔여 -1). 실측 {pages}p"
    );
}
