//! [#7140] HWP 원본의 중첩 행 점유 검사.
//!
//! HWPX 원본의 쪽수·행 점유 검사는 #7445 하위 이슈 #7812로 이관했다.
//! 원문은 mydocs/pr/assets/issue7445/giant-regression-scope-20261011/에 보존한다.
//! 별도 HWP 저장본의 기존 검사는 계속 실행한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;

/// 이관된 HWPX와 구분한 별도 HWP 원본.
const SAMPLES: [&str; 1] = ["samples/task1718/table_giant_cell_overfill.hwp"];

/// `pi324` 가 조각으로 놓이는 쪽(0 기준).
const NESTED_TABLE_PAGE: u32 = 18;

fn open(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {rel}: {e:?}"))
}

#[test]
fn nested_row_units_reserve_painted_height() {
    for rel in SAMPLES {
        let core = open(rel);
        let page_count = core.page_count();
        assert!(
            page_count > NESTED_TABLE_PAGE,
            "{rel}: {page_count}쪽 — 시험 대상 쪽이 없다"
        );
        let tree = core
            .build_page_render_tree(NESTED_TABLE_PAGE)
            .unwrap_or_else(|e| panic!("{rel}: render tree: {e:?}"));
        let opts = AnomalyOptions::default();
        let page = scan_page(NESTED_TABLE_PAGE, &tree.root, page_count, &opts);

        assert!(
            page.empty_page.is_none(),
            "{rel}: {}쪽이 비었다 — 시험 설정 오류",
            NESTED_TABLE_PAGE + 1
        );
        let over: Vec<String> = page
            .overflow
            .iter()
            .filter(|o| o.over_bottom > opts.overflow_tolerance_px)
            .map(|o| format!("{} +{:.2}px", o.path, o.over_bottom))
            .collect();
        assert!(
            over.is_empty(),
            "{rel}: {}쪽 표 조각이 본문 바닥을 넘는다 {over:?} — #7140 회귀. 중첩 표 행 \
             유닛이 내용 높이만 세면 선언까지 늘어난 마지막 행(29.8→54.1px)이 예산에서 \
             빠져 수정 전처럼 +3.95px 넘친다.",
            NESTED_TABLE_PAGE + 1
        );
    }
}
