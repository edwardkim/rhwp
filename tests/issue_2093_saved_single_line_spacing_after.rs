//! Issue #2093: 아래 간격(sa>0) 단일 줄 문단이 쪽 하단 saved-bounds 신뢰에서 배제되어 과분할.
//!
//! Regression shape (samples/task2093/saved_single_line_spacing_after.hwpx, 합성):
//! - pi=0 채움(917.3px) 뒤 pi=1 단일 줄(lh 16.0px, sa 6.7px)이 누적 fit 을
//!   layout-drift 안전마진 4px 구간에서 탈락(917.3+16.0 > 가용 929.6px)하지만,
//!   저장 LINE_SEG vpos=68800(bottom 70000HU ≤ 본문 70018HU)은 한글이 이 줄을
//!   1쪽 하단에 배치했음을 인코딩하고 다음 문단 pi=2 는 vpos=1000 리셋(새 쪽 증거).
//! - `saved_single_line_bottom_fits`(#1749)의 옛 `spacing_after <= 0.5` 게이트가
//!   이 줄을 배제 → 1줄 단독 과분할 → 이후 전 페이지 +1 밀림.
//!   (실문서: hwpdocs 1192000-201900021 해양수산 수소경제, rhwp 17쪽 vs 한글 16쪽.)
//! - 수정: sa 게이트 제거 — 한글은 쪽 마지막 줄의 아래 간격을 쪽 하단에서 소비하지
//!   않으며, 신뢰 판정은 저장 줄의 시각 경계(vpos~vpos+lh)로 충분하다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2093/saved_single_line_spacing_after.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2093_sa_tail_line_stays_on_page_1() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        2,
        "전체 2쪽 (pi2 는 vpos 리셋으로 2쪽 시작)"
    );

    let page1 = doc.dump_page_items(Some(0));
    let page2 = doc.dump_page_items(Some(1));

    assert!(
        page1.contains("pi=1"),
        "sa>0 단일 줄 pi=1 은 저장 flow(쪽-마지막 인코딩 + 시각 경계 fit)대로 1쪽 하단에 \
         배치되어야 한다 (#2093 과분할 회귀)\n--- page 1 ---\n{}",
        page1
    );
    assert!(
        page2.contains("pi=2"),
        "pi=2 는 vpos 리셋으로 2쪽 시작이어야 한다\n--- page 2 ---\n{}",
        page2
    );
    assert!(
        !page2.contains("pi=1"),
        "pi=1 이 2쪽으로 밀리면 #2093 회귀\n--- page 2 ---\n{}",
        page2
    );
}
