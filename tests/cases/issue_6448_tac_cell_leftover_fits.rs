//! [Issue #6448] leftover 에 들어가는 HWPX CELL(모델 RowBreak) 글자처럼 취급 표를
//! 다음 쪽으로 통째 이월하지 않는다.
//!
//! `samples/issue6448/tac_cell_leftover_fits.hwpx` — 2097 중간-쪽 3행 표의
//! treatAsChar=1 판. HEAD 문단 뒤에 선언 높이 68000HU 표가 leftover 에 들어간다.
//! HWPX `pageBreak="CELL"` 은 모델 RowBreak 이고, 선언-fit 이 treatAsChar 를
//! 빼면 측정 팽창으로 표를 다음 쪽에 밀어 쪽수가 늘어난다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6448/tac_cell_leftover_fits.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

#[test]
fn issue_6448_tac_cell_table_stays_on_leftover_page() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        2,
        "HEAD+표 leftover 통째 1쪽 + 후속 1쪽이어야 한다"
    );

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("Table") && !page1.contains("PartialTable"),
        "leftover 에 선언 높이가 들어가는 HWPX CELL/TAC 표는 1쪽에 통째 배치 — \
         PartialTable 이월은 #6448 회귀\n--- page 1 ---\n{page1}"
    );
    assert!(
        !doc.dump_page_items(Some(1)).contains("PartialTable"),
        "표 조각이 2쪽으로 밀리면 #6448 회귀"
    );
}
