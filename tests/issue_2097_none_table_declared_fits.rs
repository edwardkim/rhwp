//! Issue #2097: 쪽나눔=None 표의 선언 높이 신뢰 — 마지막 행 sliver 과분할 방지.
//!
//! Regression shape (samples/task2097/none_table_declared_fits.hwpx, 합성):
//! - 3행 표(pageBreak=NONE, 자리차지): 선언 높이 69700HU(929.3px) ≤ 본문 933.6px 이지만
//!   r1/r2 셀의 내용 실측이 저장 셀높이(1200/500HU)를 초과 팽창해 측정 합(946.2px)이
//!   본문을 넘는다.
//! - 수정 전: 측정 fit 실패 → 쪽나눔=None 임에도 행 분할(PartialTable) → 마지막 행이
//!   2쪽으로 조각. (실문서: 1730000 새만금 등 정책연구 서식류 — 한글 COM 3자 비교로
//!   한글 실측 행높이 합 = 저장 선언 크기 확인, 백로그 50/106건 계열.)
//! - 수정 후: 선언 높이가 현재 쪽에 들어가면 통째 배치(PartialTable 미발생).

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2097/none_table_declared_fits.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2097_none_table_placed_whole_by_declared_height() {
    let doc = load_doc();
    assert_eq!(doc.page_count(), 2, "표 통째 1쪽 + AFTER TABLE 2쪽");

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("Table") && !page1.contains("PartialTable"),
        "쪽나눔=None 표는 선언 높이(929.3px ≤ 본문 933.6px)로 1쪽에 통째 배치되어야 \
         한다 — PartialTable 분할은 #2097 회귀\n--- page 1 ---\n{}",
        page1
    );

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        !page2.contains("PartialTable"),
        "표 조각이 2쪽으로 밀리면 #2097 회귀\n--- page 2 ---\n{}",
        page2
    );
}
