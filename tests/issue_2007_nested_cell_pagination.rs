//! Issue #2007: 1×1 단일 셀 중첩 표의 셀 콘텐츠 페이지 분할(intra-cell pagination).
//!
//! `samples/basic/issue2007_nested_cell_pagination_42065.hwp` (규제영향분석서)는
//! 1×1 RowBreak 표(자리차지) 안에 중첩 1×1 표가 있고, 그 중첩 셀에 135+문단(약 8164px,
//! 8쪽 분량)이 담긴다.
//!
//! 회귀 (수정 전 버그, rhwp 6p vs 한글 17p):
//! - per-중첩행 유닛 분해(`cell_units`)는 중첩 표 `row_count >= 2` 에만 적용 →
//!   1×1(단일 행) 중첩 표는 atomic 유닛 1개로 취급 → 8164px 콘텐츠가 한 페이지에 통째
//!   배치(오버플로/크램) → under-pagination.
//!
//! 정정: 1×1 중첩 표의 셀 콘텐츠가 한 페이지를 명백히 초과(>1000px)하면 기존
//! `nested_table_mixed_fragment_heights`(텍스트+중첩표 문단에 쓰던 페이지 분할 fragment)
//! 를 빈-텍스트 문단에도 적용해 splittable 유닛으로 분해 → 페이지 경계로 분할.
//! 한글 2022 = 17페이지. 본 수정으로 6→15페이지(pi-page n_mismatch 5)로 대폭 개선.

use std::fs;
use std::path::Path;

#[test]
fn issue_2007_nested_cell_content_paginates() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // 한글 2022 = 17페이지. 수정 전 rhwp = 6페이지(1×1 중첩셀 콘텐츠 미분할 → 크램).
    // 수정 후 콘텐츠가 페이지 경계로 분할되어 15페이지(±). 미분할 회귀 시 다시 ~6페이지로 붕괴.
    let pages = doc.page_count();
    assert!(
        pages >= 12,
        "1×1 중첩셀 콘텐츠 미분할 회귀 — 페이지 수 {pages} (기대 ≥12, 한글 17, 수정 전 6)"
    );
}
