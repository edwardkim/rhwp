//! [#6035] CELL 분할 평가표가 셀 안 줄을 나누지 않고 통째로 밀어 쪽 하단을 비운다.
//!
//! `samples/issue6035/cgmp_evaluation_table.hwpx` 는 식약처 고시 별표 2
//! (147행 3열, `pageBreak="CELL"`) 를 포함한 공개 HWPX. 수정 전 rhwp 는
//! 문단 로컬 vpos=0 을 쪽 프레임으로 오인해 "다. 물 공급 설비는…" 한 줄만
//! 남긴 채 쪽을 비우고, 1) 2) 3) 항목을 다음 쪽으로 민다. 한글은 같은 행을
//! 한 쪽에 줄 단위로 채운다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6035/cgmp_evaluation_table.hwpx";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open")
}

fn page_text(doc: &DocumentCore, page: u32) -> String {
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트: {e}", page + 1))
}

#[test]
fn issue_6035_water_supply_subitems_stay_with_heading() {
    let doc = load_doc();
    let mut heading_page = None;
    let mut item_page = None;
    for page in 0..doc.page_count() {
        let text = page_text(&doc, page);
        if text.contains("다. 물 공급 설비는 다음을 만족하는가") {
            heading_page = Some(page);
        }
        if text.contains("물의 정체와 오염") {
            item_page = Some(page);
        }
    }
    let heading_page = heading_page.expect("다. 물 공급 설비 행이 있어야 한다");
    let item_page = item_page.expect("1) 물의 정체… 항목이 있어야 한다");
    assert_eq!(
        heading_page,
        item_page,
        "#6035: CELL 표는 문단 로컬 vpos=0 으로 쪽을 가르지 않는다 \
         (수정 전 heading={}쪽 / item={}쪽 으로 빈 쪽을 남김)",
        heading_page + 1,
        item_page + 1
    );
}

#[test]
fn issue_6035_does_not_emit_header_only_ghost_page() {
    let doc = load_doc();
    let mut lonely = Vec::new();
    for page in 0..doc.page_count() {
        let text = page_text(&doc, page);
        let has_heading = text.contains("다. 물 공급 설비는 다음을 만족하는가");
        let has_item = text.contains("물의 정체와 오염");
        if has_heading && !has_item {
            lonely.push(page + 1);
        }
    }
    assert!(
        lonely.is_empty(),
        "#6035: 헤더+한 줄만 남긴 빈 쪽이 없어야 한다: {lonely:?}"
    );
}
