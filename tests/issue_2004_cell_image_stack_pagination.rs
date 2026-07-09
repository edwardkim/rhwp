//! Issue #2004 잔여: 부동 표 셀 이미지 스택 콘텐츠 페이지네이션 회귀 테스트.
//!
//! `samples/issue2004_cell_image_stack.hwp(x)` — 1×1 RowBreak 부동 표(자리차지,
//! tac=false) 셀에 전면급 Square 부동 이미지 5장이 varying offset으로 스택된 문서.
//! 수정 전에는 셀 측정이 저장 높이(871.9px)에 머물러 표가 원자 배치되고 이미지
//! 5장이 한 쪽에 겹쳐 4쪽으로 렌더됐다.
//!
//! 수정: 정규화(`compute_render_normalized`)에서 셀 스택을 이미지 1장짜리 inline
//! 문단 N개로 분할하고 각 문단에 이미지 높이 합성 line_seg를 부여 → 셀 측정이
//! 스택 총높이(4310.6px)를 반영, 기존 RowBreak 분할 스캔에 자연 진입해 쪽당
//! 이미지 1장씩 배치된다.
//!
//! 기대값 8쪽 = 한글 2022 편집기 출력(`pdf/issue2004_cell_image_stack-2022.pdf`,
//! 8쪽: p1~p3 본문, p4~p8 프레임 이미지 1장씩).

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
fn cell_image_stack_hwp_paginates_to_8_pages() {
    let pages = page_count_of("samples/issue2004_cell_image_stack.hwp");
    assert_eq!(
        pages, 8,
        "issue2004 HWP5 8쪽 기대(한글 2022 정답지). 실측 {}p — 4p면 부동 표 셀 \
         이미지 스택 미분할(#2004), 9p+면 과분할 회귀.",
        pages
    );
}

#[test]
fn cell_image_stack_hwpx_paginates_to_8_pages() {
    let pages = page_count_of("samples/issue2004_cell_image_stack.hwpx");
    assert_eq!(
        pages, 8,
        "issue2004 HWPX 8쪽 기대(한글 2022 정답지). 실측 {}p — 4p면 부동 표 셀 \
         이미지 스택 미분할(#2004), 9p+면 과분할 회귀.",
        pages
    );
}
