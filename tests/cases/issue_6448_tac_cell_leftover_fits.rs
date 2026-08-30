//! [Issue #6448] HWPX CELL(모델 RowBreak) 글자처럼 취급 표의 host LINE_SEG가
//! 표 물리 높이를 잃었을 때, 짧은 host 줄을 표 band로 오인하지 않는다.
//!
//! `samples/issue6448/tac_cell_leftover_fits.hwpx`에는 HEAD 뒤의 3행
//! `treatAsChar=1`, `pageBreak="CELL"` 표(선언 높이 68000HU)가 한 줄
//! `LINE_SEG`(1000HU)만으로 저장되어 있다. Hancom 2020 기준 PDF는 HEAD/표/TAIL을
//! 각각 1/2/3쪽에 둔다. 표와 TAIL을 같은 쪽에 두면 표 물리 하단을 누락한 회귀다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6448/tac_cell_leftover_fits.hwpx";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

#[test]
fn issue_6448_tac_cell_uses_measured_band_and_moves_tail_to_page3() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        3,
        "Hancom 2020 기준은 HEAD/표/TAIL의 3쪽이다. 짧은 host LINE_SEG를 표 높이로\n\
         오인하면 1쪽 또는 2쪽으로 축소된다"
    );

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        !page1.contains("Table") && page1.contains("pi=0"),
        "1쪽은 HEAD만 포함해야 한다\n--- page 1 ---\n{page1}"
    );
    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("Table") && !page2.contains("PartialTable"),
        "3행 TAC 표는 2쪽에 통째 배치되어야 한다\n--- page 2 ---\n{page2}"
    );
    let page3 = doc
        .extract_page_text_native(2)
        .unwrap_or_else(|e| panic!("3쪽 텍스트: {e}"));
    assert!(
        page3.contains("AFTER TABLE"),
        "표의 trailing spacing 뒤 TAIL은 3쪽으로 이월되어야 한다\n--- page 3 ---\n{page3}"
    );
}
