//! Issue #1749 v2: 누적좌표 문서라도 다음 문단이 명시적 쪽나누기면 saved bounds 를 신뢰한다.
//!
//! Regression shape (samples/task1749/saved_bounds_cumulative_page_break.hwpx):
//! - 2쪽 말미 pi=26 은 누적높이 검사 탈락(919.2+36.3 > 930.5px)이지만 저장 bounds
//!   (vpos 137484 − 2쪽 기준 69310 → bottom ≈ 930.3px ≤ avail)로 2쪽 배치가 정답.
//! - 이 문서는 누적좌표(쪽 경계에서도 vpos 리셋 없음)인데 다음 문단 pi=27 이 명시적
//!   [쪽나누기](column_type=Page)라 "vpos 리셋" 검사로는 페이지-마지막 증거를 못 본다.
//!   #1749 1차 게이트가 이 증거를 누락해 pi=26 이 3쪽 단독 문단으로 밀렸다(5쪽→6쪽).
//! - 저장 lineseg 근거: pi=25(vpos=134764)와 pi=26(vpos=137484)은 한 줄(2720HU) 간격
//!   연속 배치 = 한글은 pi=26 을 2쪽 마지막 줄로 인코딩.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1749_v2_pi26_stays_on_page_2() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        5,
        "쪽나누기 직전 단일 줄 문단 pi=26 이 밀리면 5쪽 문서가 6쪽이 된다"
    );

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("pi=26"),
        "pi=26 은 2쪽 마지막 문단이어야 한다 (저장 lineseg: pi=25 와 한 줄 간격 연속)\n--- page 2 ---\n{}",
        page2
    );
}
