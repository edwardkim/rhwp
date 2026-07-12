//! Issue #2136: near-top 저장 리셋 상한 2000→2500HU — sb=2500HU 리셋 배제 과적.
//!
//! Regression shape (samples/task2136/neartop_reset_sb2500.hwpx, 합성):
//! - pi0 채움(저장 하단 64000HU > 60000) 뒤 pi1 텍스트 문단이 **저장 vpos=2500 =
//!   문단 앞 간격 sb(5000유닛=2500HU)와 정확 일치** — 저장 흐름이 "새 쪽 상단"을
//!   인코딩한다.
//! - 수정 전: `native_near_top_reset` 상한 `cv <= 2000` 에 500HU 차로 배제 →
//!   측정 fit(853+63 ≤ 가용)으로 pi1 이 1쪽 말미에 과적. (실문서 148753276 pi46:
//!   p4 used 942px > 본문 933.6px, 한글 p5 — 10k r12 PI TAIL_PUSH 계열.)
//! - 수정 후: 상한 2500HU(#1750 split-precheck 상한과 정합, sb±150 일치 조건 유지)
//!   → pi1 은 새 쪽 시작.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2136/neartop_reset_sb2500.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2136_sb2500_reset_starts_new_page() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        2,
        "sb=2500HU 저장 리셋 문단은 새 쪽 시작 (#2136)"
    );

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("pi=1"),
        "pi=1 은 저장 리셋(vpos=2500=sb)대로 2쪽 시작이어야 한다 — 1쪽 말미 과적은 \
         #2136 회귀\n--- page 2 ---\n{}",
        page2
    );
}
