//! Issue #703: BehindText/InFrontOfText 표가 paragraph 높이에 포함되어 trailing 항목이 다음 페이지로 밀림.
//!
//! 본질: `typeset.rs` 의 표 컨트롤 처리 분기에 BehindText/InFrontOfText (글뒤로/글앞으로) 가드가 누락됨.
//! `pagination/engine.rs:976-981` 에는 동일 가드가 존재하나 typeset.rs 경로 (메인 pagination) 에 미반영.
//!
//! 결함 메커니즘:
//! - 글뒤로 1×1 wrapper Table 캐리어 paragraph 의 measured height 에 표 height (≈37 px) 가 잘못 가산
//! - 누적 cur_h drift 가 trailing 항목 (PushButton, 빈 paragraph 등) 을 다음 페이지로 밀어냄
//!
//! 영향 샘플:
//! - `samples/basic/calendar_year.hwp` (HWP/PDF 1 페이지) — pi=12 PushButton 이 page 2 로 밀림
//! - `samples/통합재정통계(2010.11월).hwp` (HWP/PDF 1 페이지) — pi=14 빈 paragraph 가 page 2 로 밀림
//! - `samples/통합재정통계(2011.10월).hwp` (HWP/PDF 1 페이지) — pi=14 빈 paragraph 가 page 2 로 밀림

use std::fs;
use std::path::Path;

fn load_page_count(rel_path: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", rel_path, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", rel_path, e));
    doc.page_count()
}

#[test]
fn issue_703_calendar_year_single_page() {
    let pages = load_page_count("samples/basic/calendar_year.hwp");
    assert_eq!(
        pages, 1,
        "calendar_year.hwp 는 1 페이지여야 함 (한글2022 PDF 정답지). \
         결함 시 2 페이지: pi=12 PushButton 이 BehindText 표 height 가산으로 밀림"
    );
}

#[test]
fn issue_703_tonghap_2010_11_single_page() {
    let pages = load_page_count("samples/통합재정통계(2010.11월).hwp");
    assert_eq!(
        pages, 1,
        "통합재정통계(2010.11월).hwp 는 1 페이지여야 함 (한글2022 PDF 정답지). \
         결함 시 2 페이지: pi=14 빈 paragraph 이 누적 drift 로 밀림"
    );
}

#[test]
fn issue_703_tonghap_2011_10_single_page() {
    let pages = load_page_count("samples/통합재정통계(2011.10월).hwp");
    assert_eq!(
        pages, 1,
        "통합재정통계(2011.10월).hwp 는 1 페이지여야 함 (한글2022 PDF 정답지). \
         결함 시 2 페이지: pi=14 빈 paragraph 이 누적 drift 로 밀림"
    );
}
