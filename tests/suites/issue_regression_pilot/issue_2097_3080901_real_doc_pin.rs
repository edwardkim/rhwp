//! Issue #2097 잔존 백로그 실문서: 3080901 개인정보 제공 대장 — 페이지네이션 한글 정합 핀.
//!
//! `samples/task2097/3080901_pii_ledger.hwp` 는 문단 2개 뒤 중간-쪽(cur_h 49.6px)에
//! 배치된 17×4 RowBreak 표 별지서식. 선언 61269HU(816.9px)는 잔여(827.3px)에
//! fit 인데 rhwp 실측 829.9px(+13px 팽창)가 2.6px 초과 → 수정 전 RowBreak
//! 선언-fit 게이트가 쪽 상단 한정이라 분할 강제, rows 16..17 이 88.3px sliver 로
//! 2쪽 조각.
//!
//! 중간-쪽 확대(overshoot ≤16px 시 선언 신뢰)로 **1쪽 = 한글 2022 편집기 1쪽 정합**
//! (`pdf/task2097/3080901_pii_ledger-2022.pdf`, 편집기 PageCount=1 정합).

use std::fs;
use std::path::Path;

fn page_count_of(rel: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.page_count()
}

#[test]
fn pii_ledger_3080901_page_count_matches_hangul() {
    let pages = page_count_of("samples/task2097/3080901_pii_ledger.hwp");
    assert_eq!(
        pages, 1,
        "issue2097 3080901 기대 1쪽 (한글 2022 정답지 1쪽 정합). 실측 {}p — \
         2p면 중간-쪽 RowBreak 선언-fit 게이트 실패로 마지막 행 sliver 분할(#2097) 퇴행.",
        pages
    );
}
