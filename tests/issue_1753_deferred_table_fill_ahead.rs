//! Issue #1753: 지연 이월되는 visible-host 자리차지 표의 후속 문단 선행 채움.
//!
//! Regression shape (samples/task1753/deferred_takeplace_fill_ahead.hwpx):
//! - pi=51: 제목 텍스트 + 57×9 자리차지(TopAndBottom·vert=Para) RowBreak 표.
//!   표 몸체가 9쪽 잔여 공간에 안 들어가 10쪽으로 이월(multirow_clean_defer).
//! - 한글(PDF 시각 + 저장 LINE_SEG vpos=72581/74121): 후속 pi=52("※ 시공단계...")·
//!   pi=53("2) 보정계수")을 9쪽 하단에 선행 채움(fill-before-deferred-float).
//! - 수정 전 rhwp: 표 fragment 뒤 11쪽으로 밀림.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1753/deferred_takeplace_fill_ahead.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1753_following_paras_prefill_page_9() {
    let doc = load_doc();
    let page9 = doc.dump_page_items(Some(8));
    let page10 = doc.dump_page_items(Some(9));
    let page11 = doc.dump_page_items(Some(10));

    assert!(
        page9.contains("FullParagraph  pi=52") && page9.contains("FullParagraph  pi=53"),
        "pi=52/53 은 지연 표 이월 전 9쪽 잔여 공간에 선행 채움되어야 한다 (한글 정합)\n--- page 9 ---\n{}",
        page9
    );
    assert!(
        page10.contains("PartialTable   pi=51"),
        "표 첫 fragment 는 10쪽 시작 유지\n--- page 10 ---\n{}",
        page10
    );
    assert!(
        !page11.contains("FullParagraph  pi=52") && !page11.contains("FullParagraph  pi=53"),
        "pi=52/53 이 표 뒤 11쪽에 중복/잔류하면 안 된다\n--- page 11 ---\n{}",
        page11
    );
    assert!(
        page11.contains("pi=54"),
        "후속 표 pi=54 는 마지막 fragment 뒤 11쪽 유지\n--- page 11 ---\n{}",
        page11
    );
}
