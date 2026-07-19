//! Issue #2322 — 전면 서식 표 2장 문서 과소분할 회귀 (r15 재검증 −1 서식 계열).
//!
//! 두 결함 축을 각각 대표 문서로 고정한다:
//!
//! **A. float 배타영역 미소비** — `19439117_gokseong_voucher_form.hwp`
//! pi=0 비-TAC 자리차지 표(870px)가 만든 exclusion 존 [31..902] 을 표 문단
//! 경로(typeset_block_table fit)가 참조하지 않아, pi=1 의 866px 표가 존 위에
//! y≈36 통배치됐다 (rhwp 1쪽 vs 한글 2쪽).
//!
//! **B. 전면 TAC 표 인라인 오판** — `20862337_cheongyang_voucher_form.hwp`
//! 제목 텍스트와 TAC 표 2장(851/866px). 저장 lineseg 3줄이 각 표의 자기 줄과
//! line2 vpos=0 리셋을 명시하는데, is_tac_table_inline 폭 기준이 전면 표를
//! 인라인 판정 → paragraph_has_table=false → 텍스트 경로에서 문단 전체가
//! 한 줄(1789px)로 합성돼 분할 불가 (rhwp 1쪽 vs 한글 2쪽).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

fn core(rel: &str) -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e))
}

#[test]
fn issue_2322_a_float_exclusion_consumed_by_table_path() {
    let core = core("samples/task2322/19439117_gokseong_voucher_form.hwp");
    assert_eq!(
        core.page_count(),
        2,
        "한글 오라클 2쪽 정합 (수정 전 1쪽: 두 번째 표가 exclusion 존 위에 통배치)"
    );
    let dump = core.dump_page_items(None);
    let p2 = dump.split("=== 페이지").nth(2).expect("페이지 2 블록");
    assert!(
        p2.contains("pi=1 ci=0"),
        "p2 가 두 번째 서식 표(pi=1)로 시작해야 함:\n{}",
        p2
    );
}

#[test]
fn issue_2322_b_fullpage_tac_pair_not_inline() {
    let core = core("samples/task2322/20862337_cheongyang_voucher_form.hwp");
    assert_eq!(
        core.page_count(),
        2,
        "한글 오라클 2쪽 정합 (수정 전 1쪽: 문단이 한 줄 1789px 로 합성)"
    );
}
