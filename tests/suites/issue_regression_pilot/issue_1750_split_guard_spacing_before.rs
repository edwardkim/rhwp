//! Issue #1750: 문단 분할 진입 가드가 spacing_before 를 반영해야 한다.
//!
//! Regression shape (samples/task1750/split_guard_spacing_before.hwp):
//! - 1쪽 말미 cur_h=976.0, avail=1005.1 에서 pi=22 (sb=9.3px, 첫 줄 25.6px) 도달.
//! - 가드가 `remaining(29.1) < first_line_h(25.6)` 만 비교 → 페이지 넘김 생략,
//!   분할 루프가 첫 줄을 무조건 배치 → 1쪽 used 1010.9px (5.8px overfill).
//! - 한글(OLE 캐럿)과 저장 LINE_SEG(다음 줄 vpos=700=새 쪽 상단) 모두 pi=22 전체를
//!   2쪽 시작으로 배치한다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1750/split_guard_spacing_before.hwp";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1750_pi22_starts_on_page_2() {
    let doc = load_doc();
    let page1 = doc.dump_page_items(Some(0));
    let page2 = doc.dump_page_items(Some(1));

    assert!(
        !page1.contains("pi=22"),
        "sb 포함 시 첫 줄이 1쪽에 들어가지 않으므로 pi=22 는 1쪽에 분할 배치되면 안 된다\n--- page 1 ---\n{}",
        page1
    );
    assert!(
        page2.contains("pi=22"),
        "pi=22 는 2쪽 시작이어야 한다 (한글 OLE·저장 LINE_SEG 정합)\n--- page 2 ---\n{}",
        page2
    );
}
