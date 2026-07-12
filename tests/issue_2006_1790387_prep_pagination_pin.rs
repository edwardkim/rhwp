//! Issue #2006 — 1790387 HIV PrEP 최종결과보고서 페이지네이션 드리프트 핀.
//!
//! `samples/issue2006/1790387_prep_final_report.hwpx` — 빈 문단에 전면급
//! tac(treat_as_char) 이미지 여러 장이 스택된 프레임 페이지가 많은 정책연구
//! 최종결과보고서. PR #2082(전면 tac 이미지 스택 라인 경계 강제분할)로
//! 130쪽 → 141쪽 (스택 문단 h>1500px 잔여 0).
//!
//! 권위 정답지는 한글 2022 편집기 146쪽
//! (`pdf-large/issue2006/1790387_prep_final_report-2022.pdf`, Git LFS,
//! 편집기 PageCount=146 정합). 잔여 −5는 텍스트 줄-채움 누적(#1921 계열) 별건 —
//! 본 테스트는 현재 도달값 141을 핀해 개선(146 방향)과 회귀(140−)를 모두 표면화한다.

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
fn prep_1790387_page_count_pin() {
    let pages = page_count_of("samples/issue2006/1790387_prep_final_report.hwpx");
    assert_eq!(
        pages, 141,
        "issue2006 1790387 핀 141쪽 (한글 2022 정답지 146쪽, 잔여 -5=줄-채움 누적 축). \
         실측 {}p — 130p 부근이면 tac 이미지 스택 미분할(#2006) 회귀, 141p 초과 개선 시 \
         핀과 정답지(146)를 갱신할 것.",
        pages
    );
}
