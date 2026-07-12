//! Issue #2105: RowBreak 표 선언-fit 무시 — 실측 팽창으로 불필요 행 sliver 분할.
//!
//! Regression shape (samples/task2105/rowbreak_table_declared_fits.hwpx, 합성 —
//! task2097 fixture 의 pageBreak=CELL(→IR RowBreak) 판):
//! - 3행 표(RowBreak, 자리차지): 선언 69700HU(929.3px) ≤ 본문 933.6px 이지만
//!   r1/r2 내용 실측 팽창으로 측정 합(946.2px)이 본문 초과.
//! - 수정 전: 측정 fit 실패 → 행 분할 발동 → 마지막 행 sliver 2쪽 조각.
//!   (실문서: 19378753 밀양시 별지서식 24×27 RowBreak 표, 선언 907.7px vs 실측
//!   955.9px → rows 22..24 sliver, rhwp 2쪽 vs 한글 1쪽. 백로그 44/96건 주력.)
//! - 수정 후(#2097 게이트의 RowBreak 확대): 선언 높이가 현재 쪽에 fit 하면 통째 배치.
//!   RowBreak 는 "나눔 허용"이지 강제가 아니며, 선언이 fit 하지 않는 다쪽 표의 분할
//!   의미론은 불변.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2105/rowbreak_table_declared_fits.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2105_rowbreak_table_placed_whole_by_declared_height() {
    let doc = load_doc();
    assert_eq!(doc.page_count(), 2, "표 통째 1쪽 + AFTER TABLE 2쪽");

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("Table") && !page1.contains("PartialTable"),
        "RowBreak 표도 선언 높이(929.3px ≤ 본문 933.6px)가 fit 하면 1쪽에 통째 배치 —\
         PartialTable 분할은 #2105 회귀\n--- page 1 ---\n{}",
        page1
    );
    assert!(
        !doc.dump_page_items(Some(1)).contains("PartialTable"),
        "표 조각이 2쪽으로 밀리면 #2105 회귀"
    );
}
