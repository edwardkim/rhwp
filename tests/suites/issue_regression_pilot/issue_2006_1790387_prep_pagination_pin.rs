//! Issue #2006 — 1790387 HIV PrEP 최종결과보고서의 140쪽 검사.
//!
//! 원본 `samples/issue2006/1790387_prep_final_report.hwpx`는 한컴2024 저장본이다.
//! 같은 원본을 HWP MCP `--engine 2024`로 출력한 독립 기준은
//! `pdf/issue2006/1790387_prep_final_report-2024.pdf`의 140쪽이다.
//! 원본/PDF SHA와 변환 출처는 `mydocs/pr/archives/pr_7406_review.md`에 있다.
//!
//! KoPub이 내장된 기존 2020 PDF도 140쪽이며 보조 대조군으로 보존한다.
//! KoPub이 없는 2022 PDF의 146쪽은 다른 글꼴 환경의 출력이므로 기준으로
//! 사용하지 않는다. PDF Producer의 연도나 writer 이름만으로 고르지 않는다.
//!
//! PR #2082의 130→141쪽은 당시 개선 결과이며 현재 기준값이 아니다.
//! 보정59는 본문 표의 물리 안 여백과 그림/빈 후속 줄의 저장 프레임을 복원했다.
//! 동일 원본 전체 140쪽의 Native/fresh WASM 최저 90% 이상을 확인한 근거는
//! `mydocs/pr/assets/pr7382_20260926/stage59_prep2006_validation.json`에 있다.
//! 기존 140쪽 기대값과 검사 함수 수는 유지한다.

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
        "issue2006 1790387 동일 원본 한컴2024 PDF 140쪽과 달라짐: 실제 {}쪽",
        pages
    );
}
