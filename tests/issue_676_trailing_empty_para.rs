//! Issue #676: 통합재정통계 2010.11/2011.10 — 본문 끝 trailing 빈 줄이
//! `LAYOUT_DRIFT_SAFETY_PX = 10.0` (typeset.rs) 영역 내 미세 overflow 로 fit 실패하여
//! 단독 빈 페이지 (페이지 2) 발생.
//!
//! 정정: `typeset_paragraph` 에 trailing empty paragraph 가드 추가 — 섹션 마지막
//! 빈 paragraph + 단단(col_count==1) + 페이지 첫 항목 아님 + overflow ≤ safety_margin
//! 시 height=0 흡수.
//!
//! 1단계 trace 진단:
//!   pi=14 cur_h=751.0 + h_for_fit=16.0 = 767.0 > avail 766.2 (= 776.2 - 10 safety),
//!   overflow=0.8px ≤ safety_margin 10px → 흡수 대상.
//!
//! 한컴2022 정합: 두 문서 모두 1페이지 출력.

use std::fs;
use std::path::Path;

#[test]
fn issue_676_t재정통계_2010_11_single_page() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/통합재정통계(2010.11월).hwp");
    let bytes = fs::read(&hwp_path).expect("read 통합재정통계(2010.11월).hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 통합재정통계(2010.11월).hwp");

    // 한컴2022 정합 정답지: 1 페이지
    // 회귀 시: 2 페이지 (pi=14 trailing 빈 줄이 단독 페이지 발생)
    assert_eq!(
        doc.page_count(),
        1,
        "통합재정통계(2010.11월).hwp 는 1 페이지여야 함 (Task #676 trailing empty para 가드 회귀 시 2)"
    );
}

#[test]
fn issue_676_t재정통계_2011_10_single_page() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/통합재정통계(2011.10월).hwp");
    let bytes = fs::read(&hwp_path).expect("read 통합재정통계(2011.10월).hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 통합재정통계(2011.10월).hwp");

    // 한컴2022 정합 정답지: 1 페이지 (2010.11 과 동일 패턴)
    assert_eq!(
        doc.page_count(),
        1,
        "통합재정통계(2011.10월).hwp 는 1 페이지여야 함 (Task #676 trailing empty para 가드 회귀 시 2)"
    );
}

#[test]
fn issue_676_t재정통계_2014_08_no_regression() {
    // 2014.8 은 본 결함 미발현 (rhwp 1p, PDF 1p 일치) — 가드 도입 후 무회귀 검증.
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/통합재정통계(2014.8월).hwp");
    let bytes = fs::read(&hwp_path).expect("read 통합재정통계(2014.8월).hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 통합재정통계(2014.8월).hwp");
    assert_eq!(doc.page_count(), 1);
}
