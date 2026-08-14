//! Issue #2006 — 1790387 HIV PrEP 최종결과보고서 페이지네이션 드리프트 핀.
//!
//! `samples/issue2006/1790387_prep_final_report.hwpx` — 빈 문단에 전면급
//! tac(treat_as_char) 이미지 여러 장이 스택된 프레임 페이지가 많은 정책연구
//! 최종결과보고서. PR #2082(전면 tac 이미지 스택 라인 경계 강제분할)로
//! 130쪽 → 141쪽 (스택 문단 h>1500px 잔여 0).
//!
//! 권위 정답지는 최신 HWP 2020 MCP 변환 PDF 140쪽
//! (`pdf/issue2006/1790387_prep_final_report-hwp2020-20260814.pdf`,
//! PrintToPDFEx·PDF PageCount=140 정합)이다. 이전 한글 2022 PDF(146쪽)는
//! 다른 폰트 환경에서 산출되어 비교 기준으로 쓰지 않는다.

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
        pages, 140,
        "issue2006 1790387 HWP 2020 MCP 정본 140쪽과 달라짐: 실제 {}쪽",
        pages
    );
}
