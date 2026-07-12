//! Issue #1949: 거대 셀(수천 문단 + 중첩표 수십 개)을 가진 RowBreak 표가 여러 페이지에
//! 걸칠 때, 렌더가 각 페이지에서 그 셀 콘텐츠를 통째로 재계산(O(pages × cell))해
//! `render-diff`/`export-svg` 가 420초 타임아웃되던 병리적 성능의 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플):
//! `samples/issue1949_giant_cell_nested_tables_perf.hwpx` (해양수산부 별표, 0.3MB).
//! `samples/issue1949_giant_cell_nested_tables_perf.hwp` (같은 기준의 HWP 저장본).
//! 바깥 3×1 RowBreak 표의 셀[2] = 2507문단 + 중첩표 수십 개, 한컴 2024 기준
//! PDF와 같이 115쪽에 걸침.
//!
//! 정정: `cell_units`(셀 콘텐츠 유닛)를 셀 포인터 키로 메모이즈(`LayoutEngine`,
//! 재조판 경계에서 clear). O(pages × cell) → O(cell + pages). 수정 후 전체 렌더
//! ~3초(수정 전 >400s), 렌더 출력은 bit-identical(순수 함수 캐시).
//!
//! 이 테스트는 전체 페이지를 렌더해 (1) 완주(무한 루프/폭증 부재) (2) 페이지 수
//! (3) 거대 셀이 걸친 중간 페이지의 비어있지 않은 SVG 를 확인한다. 캐시가 없으면
//! 이 테스트는 사실상 완료 불가(수백 초) → CI 에서 폭증 회귀를 드러낸다.

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

#[test]
fn giant_cell_rowbreak_table_renders_all_pages_without_blowup() {
    let doc = load_doc("samples/issue1949_giant_cell_nested_tables_perf.hwpx");
    let pages = doc.page_count();
    assert_eq!(pages, 115, "issue1949/1999: 기준 PDF와 쪽수 불일치");

    let hwp_doc = load_doc("samples/issue1949_giant_cell_nested_tables_perf.hwp");
    assert_eq!(
        hwp_doc.page_count(),
        115,
        "issue1949/1999: HWP 저장본이 기준 PDF와 쪽수 불일치"
    );

    // 전체 페이지 렌더 — 캐시가 없으면 O(pages×cell) 로 사실상 완료 불가.
    let mut nonempty_mid = false;
    for p in 0..pages {
        let svg = doc
            .render_page_svg_native(p)
            .unwrap_or_else(|e| panic!("render page {p}: {e}"));
        assert!(
            svg.starts_with("<svg") || svg.contains("<svg"),
            "page {p} SVG 아님"
        );
        // 거대 셀이 걸친 중간 페이지에 실제 콘텐츠(text/rect)가 있는지.
        if (10..pages.saturating_sub(5)).contains(&p)
            && (svg.contains("<text") || svg.contains("<rect"))
        {
            nonempty_mid = true;
        }
    }
    assert!(nonempty_mid, "issue1949: 중간 페이지에 렌더 콘텐츠가 없음");
}
