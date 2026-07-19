//! Issue #2311 — 붙임 전면 포스터 문단(tac 그림 + 후행 tac 헤더 표) 통밀림 회귀.
//!
//! `samples/task2311/156744475_nano_plan_poster.hwpx` (과기정통부 보도자료)
//! pi=21: ls[0](vpos=5435, lh=63156) = 전면 포스터 tac 그림,
//! ls[1](vpos=0) = 붙임2 헤더 tac 표 — 한글 저장 lineseg 는 "그림은 현재
//! 쪽(붙임1 헤더 아래), 표는 새 쪽"의 intra-para 분할을 명시한다.
//!
//! 회귀 시그니처 (수정 전, 10k r15 픽셀 58.91% / PAGE_DELTA +2):
//! - typeset_table_paragraph pre-flush 가 문단 전체 높이로 fit 판정
//!   → 포스터까지 새 쪽으로 통밀림 (붙임 헤더만 남은 유령 쪽 생성)
//! - 표 advance 후 새 쪽에 문단 전체 높이가 유령 계상
//!   → 후속 그래프 문단들이 또 한 쪽 밀림
//! - 결과: rhwp 5쪽 vs 한글 3쪽
//!
//! 고정: 총 3쪽 + 포스터 그림이 p2(0-based 1), 붙임2 내용이 p3 에 존재.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/task2311/156744475_nano_plan_poster.hwpx");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 156744475_nano_plan_poster.hwpx")
}

#[test]
fn issue_2311_poster_doc_paginates_to_three_pages() {
    let core = core();
    assert_eq!(
        core.page_count(),
        3,
        "한글 오라클 3쪽 정합 (수정 전 5쪽: 붙임 포스터/그래프 통밀림)"
    );
}

#[test]
fn issue_2311_poster_shape_stays_with_attachment_header() {
    let core = core();
    let dump = core.dump_page_items(None);
    // 페이지 블록별로 분해
    let mut pages: Vec<&str> = Vec::new();
    let mut starts: Vec<usize> = dump.match_indices("=== 페이지").map(|(i, _)| i).collect();
    starts.push(dump.len());
    for w in starts.windows(2) {
        pages.push(&dump[w[0]..w[1]]);
    }
    assert_eq!(pages.len(), 3, "페이지 블록 3개");
    // p2: 붙임1 헤더 표(pi=19) + 포스터 그림(pi=21 ci=0)이 같은 쪽
    assert!(
        pages[1].contains("pi=19") && pages[1].contains("pi=21 ci=0"),
        "p2 에 붙임1 헤더(pi=19)와 포스터 그림(pi=21 ci=0)이 함께 있어야 함:\n{}",
        pages[1]
    );
    // p3: 붙임2 헤더 표(pi=21 ci=1)가 새 쪽에서 시작 (저장 lineseg vpos==0 존중)
    assert!(
        pages[2].contains("pi=21 ci=1"),
        "p3 이 붙임2 헤더 표(pi=21 ci=1)로 시작해야 함:\n{}",
        pages[2]
    );
}
