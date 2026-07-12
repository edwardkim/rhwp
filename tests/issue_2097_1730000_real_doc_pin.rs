//! Issue #2097 — 실문서(1730000 선정 결과보고서) 페이지네이션 한글 정합 핀.
//!
//! `samples/task2097/1730000_selection_report.hwp` — 쪽나눔=None 18행 표 서식.
//! 한글 실측 행높이 합 910.6px = 저장 선언 68291HU(910.5px)인데 rhwp 실측은
//! 954.1px(+43.5px, 내용 실측 팽창 + rowspan 사다리 퇴화 행)로 본문(933.6px)을
//! 초과 → 수정 전 None 표임에도 분할 루프 강제 진입, 마지막 행 sliver 로 2쪽.
//!
//! PR #2101(`declared_none_table_whole_fits` — None 표 선언높이 신뢰)로
//! **1쪽 = 한글 2022 편집기 1쪽 정합**
//! (`pdf/task2097/1730000_selection_report-2022.pdf`, 편집기 PageCount=1 정합).
//! 합성 fixture(none_table_declared_fits.hwpx)는 rhwp 측정 드리프트를 모사한
//! 시맨틱 핀이고, 한글 정합의 권위 검증은 이 실문서가 담당한다.

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
fn selection_report_1730000_page_count_matches_hangul() {
    let pages = page_count_of("samples/task2097/1730000_selection_report.hwp");
    assert_eq!(
        pages, 1,
        "issue2097 1730000 기대 1쪽 (한글 2022 정답지 1쪽 정합). 실측 {}p — \
         2p면 None 표 선언높이 신뢰 실패로 마지막 행 sliver 분할(#2097) 회귀.",
        pages
    );
}
