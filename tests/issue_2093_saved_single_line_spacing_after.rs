//! Issue #2093: 아래 간격(sa>0) 단일 줄 문단이 쪽 하단 saved-bounds 신뢰에서 배제되어 과분할.
//!
//! Regression shape (samples/task2093/saved_single_line_spacing_after.hwpx, 합성):
//! - pi=0 `FILL`은 10pt/160% 문단이지만 저장 LINE_SEG가 68800HU 높이를 가진다.
//! - 한컴 2020/2022는 이 비정상적인 순수 텍스트 줄을 재조판해 pi=0~2를 모두
//!   1쪽에 둔다. rhwp도 저장 `line_height`와 `text_height`가 스타일상 가능한 줄
//!   advance보다 모두 과도할 때만 font metrics로 되돌린다.
//! - 실제 쪽 하단 saved-bounds 회귀(17쪽 -> 16쪽)는
//!   `issue_2093_1192000_real_doc_pin`이 검증한다.

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
        1,
        "한컴 2020 PDF와 같이 전체 1쪽이어야 한다"
    );

    let page1 = doc.dump_page_items(Some(0));

    for para_idx in [0, 1, 2] {
        assert!(
            page1.contains(&format!("pi={para_idx}")),
            "pi={para_idx}가 1쪽에 있어야 한다\n--- page 1 ---\n{page1}"
        );
    }
}
