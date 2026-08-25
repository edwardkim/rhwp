//! [Issue #5921] 저장 near-top 리셋이 잔여 예산을 안 보고 쪽을 가른다.
//!
//! `samples/task2136/neartop_reset_sb2500.hwpx` — pi0 채움 뒤 pi1 이
//! stored vpos=2500(=sb) 이라 `native_near_top_reset` 이 켜진다. 그런데
//! 본문 잔여 80.3px 에 pi1 필요 높이 63.2px 가 들어간다. 한글 2020 PDF 는
//! 1쪽. rhwp 는 잔여를 안 보면 2쪽.
//!
//! 과적 보호(#2136, 148753276 pi46 used 942>933.6)는 잔여 < 필요일 때만
//! 리셋을 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/task2136/neartop_reset_sb2500.hwpx";

#[test]
fn issue_5921_fitting_neartop_reset_stays_on_page_1() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    assert_eq!(
        doc.page_count(),
        1,
        "잔여에 들어가는 near-top 문단은 한글처럼 1쪽에 남아야 한다 (#5921)"
    );

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("pi=1"),
        "pi=1 이 1쪽에 있어야 한다 — 저장 vpos=2500 리셋이 잔여 80px 를 무시하면 2쪽으로 밀린다\n{page1}"
    );
}
