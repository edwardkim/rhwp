//! Issue #1755: 지연 이월 표의 host 텍스트 줄은 이월 전 쪽에 pre-emit 되어야 한다.
//!
//! Regression shape (samples/task1753/deferred_takeplace_fill_ahead.hwpx):
//! - pi=51 host 제목("1) 투입인원수 산정기준")을 한글은 9쪽 하단(anchor 흐름 위치)에
//!   렌더하는데, 수정 전 rhwp 는 layout 의 defer_visible_rowbreak_host_text 경로로
//!   마지막 fragment 뒤(11쪽)에 렌더 — 제목이 자기 표/후속 문단 뒤에 나타나는 순서 결함.
//! - 수정 후: typeset 이 이월 직전 PartialParagraph{51, 0..1} 로 9쪽에 pre-emit 하고
//!   layout 은 pre_emitted_host_paras 신호로 fragment 쪽 host 렌더를 억제한다.

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
fn issue_1755_host_heading_pre_emitted_on_page_9() {
    let doc = load_doc();
    let page9 = doc.dump_page_items(Some(8));

    assert!(
        page9.contains("PartialParagraph  pi=51"),
        "host 제목 줄은 이월 전 쪽(9쪽)에 PartialParagraph 로 pre-emit 되어야 한다\n--- page 9 ---\n{}",
        page9
    );
    // 한글 순서: pi50 → pi51(제목) → pi52 → pi53
    let i51 = page9.find("PartialParagraph  pi=51").unwrap();
    let i52 = page9.find("FullParagraph  pi=52").unwrap_or(usize::MAX);
    assert!(
        i51 < i52,
        "제목 줄(pi=51)은 후속 문단(pi=52)보다 앞에 배치되어야 한다\n--- page 9 ---\n{}",
        page9
    );
}
