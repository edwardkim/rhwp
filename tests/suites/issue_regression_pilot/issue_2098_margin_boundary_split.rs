//! Issue #2098/#2138: 불확실 앵커(vpos≤0) footer fit 의 62px 마진 — 경계 분할 회귀.
//!
//! Regression shape (samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx, 합성):
//! - 본문 흐름 끝(저장) 56600HU(754.7px), 쪽-하단 고정 표 133.3px →
//!   배타 잔여 800.2px, 슬랙 45.6px — **마진(62px) 이내의 경계 케이스**.
//! - 10k r12 warm PDF 권위: 이 대역(슬랙 3.4~61.3px)의 결재문서 60건은 한글이
//!   **분할(2쪽)** — 마진 없는 fit 은 흡수(1쪽)해 회귀했다 (#2104 재보정 경위).
//! - 기대: 앵커+틀이 2쪽 단독 배치 (마진이 경계 케이스를 분할로 보냄).
//!   흡수 방향은 기존 fixture(page_bottom_fixed_anchor_vpos0, 슬랙 ~700px)가 보증.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2098_margin_boundary_footer_splits_to_page_2() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        2,
        "슬랙 45.6px(< 마진 62px) 경계 footer 는 분할(2쪽)이어야 한다 — 흡수는 \
         10k r12 회귀 재발 (#2098/#2138)"
    );
    assert!(
        doc.dump_page_items(Some(1)).contains("Table"),
        "고정 틀 표는 2쪽 단독이어야 한다"
    );
}
