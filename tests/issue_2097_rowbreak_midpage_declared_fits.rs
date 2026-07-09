//! Issue #2097 잔존 백로그: 중간-쪽 RowBreak 표 선언-fit 무시 — 마지막 행 sliver 분할.
//!
//! Regression shape (samples/task2097/rowbreak_midpage_declared_fits.hwpx, 합성 —
//! task2105 fixture 에 HEAD 문단을 추가해 표를 중간-쪽(cur_h 21.3px)에 배치한 판):
//! - HEAD 문단 1개 + 3행 RowBreak 표: HEAD(21.3px) + 선언 68000HU(906.7px) ≤ 본문
//!   933.6px 이지만 r1/r2 내용 실측 팽창으로 측정 합 944.9px 이 본문을 11.3px 초과.
//! - 수정 전: RowBreak 선언-fit 게이트가 쪽 상단(current_height≤0.5) 한정이라
//!   중간-쪽에서 측정 fit 실패 → 행 분할 → 마지막 행 sliver 2쪽 조각.
//!   (실문서: 3080901 지식재산처 별지 2, 17×4 RowBreak 표 — 문단 2개 뒤 cur_h
//!   49.6px, 선언 816.9px fit, 실측 +13px 팽창 → 2.6px 초과로 rows 16..17
//!   88.3px sliver, rhwp 2쪽 vs 한글 1쪽.)
//! - 수정 후: 중간-쪽에서도 실측 초과(overshoot)가 측정 노이즈 수준(≤16px)이면
//!   선언 높이 신뢰로 통째 배치. overshoot 가 큰 표(한글도 실측 분할)는 불변.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2097/rowbreak_midpage_declared_fits.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2097_midpage_rowbreak_table_placed_whole_by_declared_height() {
    let doc = load_doc();
    assert_eq!(doc.page_count(), 2, "HEAD+표 통째 1쪽 + AFTER TABLE 2쪽");

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("Table") && !page1.contains("PartialTable"),
        "중간-쪽(cur_h 21.3px) RowBreak 표도 선언 높이(906.7px)가 fit 하고 실측
         초과(11.3px)가 노이즈 수준이면 통째 배치 — PartialTable 분할은 #2097 회귀\
         \n--- page 1 ---\n{}",
        page1
    );
    assert!(
        !doc.dump_page_items(Some(1)).contains("PartialTable"),
        "표 조각이 2쪽으로 밀리면 #2097 회귀"
    );
}
