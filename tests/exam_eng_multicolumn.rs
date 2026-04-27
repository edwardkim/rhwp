//! Task #391: 다단 섹션 단 전환 정상화 회귀 테스트.
//!
//! `samples/exam_eng.hwp` (Section-level 2단 다단) 가 #359 merge 직후부터
//! 8 → 11 페이지로 회귀하고 단 채움이 비대칭으로 어그러진 문제를 검증.
//!
//! 원인: `src/renderer/typeset.rs` 의 `next_will_vpos_reset` 선제 가드가
//! 다단 비-마지막 단의 단 전환 (vpos=0 reset) 도 "단독 항목 페이지 위험" 으로
//! 오인하여 발동.
//!
//! 본 테스트는 페이지 수 8 을 기대값으로 둔다 (#359 이전 동작 복원).

use std::fs;
use std::path::Path;

#[test]
fn exam_eng_page_count_after_359_fix() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_eng.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse exam_eng.hwp");

    let pages = doc.page_count();
    assert_eq!(
        pages, 8,
        "exam_eng.hwp 8 페이지 기대 (Task #391 / #359 회귀 복원). 실측 {}p.",
        pages
    );
}
