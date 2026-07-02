//! Issue #1749: 누적좌표 문서에서 saved bounds 신뢰가 쪽 경계 overfill 을 만들면 안 된다.
//!
//! Regression shape (samples/task1749/saved_bounds_cumulative_vpos.hwpx):
//! - 1쪽 말미 pi=18(" ")이 누적높이 검사 탈락(998.7 > 986.2px)인데도
//!   `saved_single_line_bottom_fits`(저장 bounds 985.7px ≤ avail)로 1쪽 배치
//!   → used 1011.8px > 본문 990.2px overfill.
//! - 이 문서는 누적좌표(pi19 vpos=74902 가 리셋 없이 본문높이 74265HU 초과)라
//!   저장 vpos 가 페이지 배정을 인코딩하지 않는다. 한글(OLE 캐럿)은 pi18 을 2쪽 배치.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_vpos.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1749_pi18_starts_on_page_2() {
    let doc = load_doc();
    let page1 = doc.dump_page_items(Some(0));
    let page2 = doc.dump_page_items(Some(1));

    assert!(
        !page1.contains("pi=18"),
        "누적좌표 문서의 꼬리 공백 문단 pi=18 은 1쪽 누적높이를 초과하므로 1쪽에 배치되면 안 된다\n--- page 1 ---\n{}",
        page1
    );
    assert!(
        page2.contains("pi=18"),
        "pi=18 은 2쪽 시작이어야 한다 (한글 OLE 정합)\n--- page 2 ---\n{}",
        page2
    );
}
