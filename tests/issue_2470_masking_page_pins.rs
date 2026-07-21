//! Issue #2279 / PR #2470: 마스킹 생성기 잔존 2종 원본 오라클 — 장기 재현 샘플 핀.
//!
//! PR #2470 검토(pr_2470_review.md)의 후속 권고("36382471/36341511 원본을 장기
//! 재현 샘플로 추가") 대응. 두 문서는 마스킹('*' 치환) 결재문서이며 한글 2022
//! COM 재저장 오라클 PDF(`pdf/issue2470/*-2022.pdf`, Producer=Hancom)를 함께
//! 보존한다.
//!
//! - 36382471: stale-lh 표(#2470 수정 1) — 한글 2쪽 정합 핀.
//! - 36341511: 저장 줄수 과소 재래핑(#2470 수정 2) 후 잔여 — 한글 8쪽 vs rhwp
//!   9쪽 (실텍스트 재래핑 글리프 정밀도 별건, #2279 코멘트 추적). 현재값 9를
//!   핀해 양방향 회귀(개선 포함)를 표면화한다.

use std::fs;
use std::path::Path;

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

/// 36382471 (stale-lh 표): 한글 2022 오라클 2쪽 정합 유지.
#[test]
fn issue_2470_stale_lh_table_36382471_matches_hangul_two_pages() {
    assert_eq!(
        page_count("samples/issue2470/36382471_masked.hwpx"),
        2,
        "한글 2022 오라클(pdf/issue2470/36382471_masked-2022.pdf) 2쪽 정합 회귀"
    );
}

/// 36341511 (저장 줄수 과소 재래핑 잔여): 현재 9쪽 핀 (한글 8쪽).
///
/// 8쪽이 되면 잔여 축(#2279 실텍스트 재래핑) 해소 — 이 핀을 8로 갱신하고
/// #2279 코멘트에 기록할 것. 10쪽 이상이면 순수 회귀.
#[test]
fn issue_2470_masked_rewrap_36341511_pins_current_nine_pages() {
    assert_eq!(
        page_count("samples/issue2470/36341511_masked.hwpx"),
        9,
        "한글 8쪽(오라클 pdf/issue2470/36341511_masked-2022.pdf) 대비 잔여 +1 상태 핀"
    );
}
