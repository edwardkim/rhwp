//! Issue #2093 — 실문서(1192000 해양수산 수소경제) 페이지네이션 한글 정합 핀.
//!
//! `samples/task2093/1192000_hydrogen_policy_research.hwp` — 쪽 하단 단일 줄
//! (spacing_after=1000HU) 문단이 누적 fit 안전마진(4px) 구간에서 탈락하고
//! `saved_single_line_bottom_fits` 의 `spacing_after <= 0.5` 게이트에 막혀
//! saved-bounds 신뢰에서도 배제 → 한 줄 단독 과분할로 abs pi66부터 문서 끝까지
//! 83건 연쇄 +1 밀림 (17쪽 vs 한글 16쪽) 이던 실결함 문서.
//!
//! PR #2096(sa 게이트 제거)로 **16쪽 = 한글 2022 편집기 16쪽 정합**
//! (`pdf/task2093/1192000_hydrogen_policy_research-2022.pdf`, 편집기
//! PageCount=16 정합). 합성 fixture(saved_single_line_spacing_after.hwpx)는
//! 저장 LINE_SEG 신뢰 시맨틱 핀이고, 한글 정합의 권위 검증은 이 실문서가 담당한다.

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
fn hydrogen_1192000_page_count_matches_hangul() {
    let pages = page_count_of("samples/task2093/1192000_hydrogen_policy_research.hwp");
    assert_eq!(
        pages, 16,
        "issue2093 1192000 기대 16쪽 (한글 2022 정답지 16쪽 정합). 실측 {}p — \
         17p면 쪽 하단 단일 줄(sa>0) 과분할(#2093) 회귀.",
        pages
    );
}
