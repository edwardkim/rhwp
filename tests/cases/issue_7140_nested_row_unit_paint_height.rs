//! [Issue #7140] 칸 안 다중 행 중첩 표의 행 유닛 높이가 그려지는 높이보다 작아 쪽 끝
//! 표 조각이 본문 바닥을 넘던 결함의 가드.
//!
//! `cell_units` 는 텍스트 없는 호스트 문단의 다중 행 중첩 표를 행마다 한 유닛
//! (`nested_row = Some(ri)`)으로 올린다. 그 행 높이를 **내용 기준**(맞춤 없는
//! `resolve_row_heights_with_common_fit`)으로 셌는데, 렌더러는 통째 배치든 조각이든
//! `resolve_row_heights` 로 행 합이 선언 표 높이보다 작으면 마지막 행을 선언까지 늘려
//! 그린다. 늘어난 몫만큼 페이지네이터 예산이 모자랐다.
//!
//! 같은 PR 의 둘째 축은 조각 예산이다 — 조각이 쪽에서 차지하는 높이는 유닛 합 위에
//! mixed nested 첫 가시 유닛을 한 번 더 예약하는데, 컷 예산이 그 몫을 빼지 않아
//! 예산 안의 컷이 본문을 넘었다(`row_cut_mixed_nested_reserve`).
//!
//! `samples/table_giant_cell_overfill.hwpx` 19쪽 `pi324`(5×3, `treatAsChar=1`) 실측(px):
//!
//! ```text
//!                  행 높이                              합     유닛 합
//!   내용 기준      28.3 32.1 29.8 29.8 29.8            149.8   163.0
//!   페인트(선언)   28.3 32.1 29.8 29.8 54.1            174.0   187.3
//! ```
//!
//! 수정 전 19쪽 표 조각은 본문 바닥을 **+3.95px** 넘었다(`layout-anomaly` overflow).
//! 기대값은 구현과 무관한 문서 자신의 본문 영역이다 — 조각은 본문 안에 있어야 한다.
//!
//! 두 축을 함께 고치면 HWPX 판 쪽수가 정본과 같은 **48쪽**이 된다(수정 전 47).
//!
//! ⚠ HWP 판(`task1718`)은 컷이 그대로라 47쪽이다 — 그 문서의 40쪽 거대 조각 넘침은
//! #5908 축이고 이 수정의 술어 밖이다. 그래서 쪽수 계약은 HWPX 판에만 건다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;

/// 같은 원본의 HWPX 와 HWP 저장본. 두 판 모두 19쪽에서 같은 결함을 냈다.
const SAMPLES: [&str; 2] = [
    "samples/table_giant_cell_overfill.hwpx",
    "samples/task1718/table_giant_cell_overfill.hwp",
];

/// `pi324` 가 조각으로 놓이는 쪽(0 기준).
const NESTED_TABLE_PAGE: u32 = 18;

/// HWPX 판의 한/글 정본 쪽수 — `pdf/table_giant_cell_overfill-hwpx-2024.pdf` 48쪽.
const ORACLE_PAGE_COUNT_HWPX: u32 = 48;

fn open(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {rel}: {e:?}"))
}

/// 두 축을 함께 고치면 HWPX 판이 정본 쪽수와 같아진다.
#[test]
fn hwpx_page_count_matches_the_oracle() {
    let core = open(SAMPLES[0]);
    assert_eq!(
        core.page_count(),
        ORACLE_PAGE_COUNT_HWPX,
        "{}: 한/글 정본은 {ORACLE_PAGE_COUNT_HWPX}쪽이다 — 행 유닛이 내용 높이만 세거나 \
         컷 예산이 mixed nested 예약을 빼지 않으면 47쪽이 된다",
        SAMPLES[0]
    );
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
