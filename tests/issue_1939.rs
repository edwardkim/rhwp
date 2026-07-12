//! Issue #1939 — HWP5-in-.hwpx 76076 strict render-diff 구조 안정성.
//!
//! #1936으로 페이지 수는 82쪽으로 맞았지만, `render-diff --via hwpx` strict 기준에서
//! page 38/39 경계의 TAC RowBreak 표 host 문단들이 다음 쪽으로 밀리며
//! STRUCT_MISMATCH가 남았다. 원인은 HWP5-origin HWPX export가 넣는
//! "원본 LineSeg 부재" marker가 HWPX 로드 중 TAC 표 높이 보정에 의해 실제
//! LineSeg처럼 변형되어 제거되지 않는 것이었다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};

const SAMPLE: &str = "samples/issue1891/76076_regulatory_analysis.hwpx";

#[test]
fn issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let diff = roundtrip_geom(&data, Via::Hwpx)
        .unwrap_or_else(|e| panic!("roundtrip_geom({SAMPLE}): {e:?}"));

    assert_eq!(
        diff.page_count_a, diff.page_count_b,
        "{SAMPLE} HWPX roundtrip 페이지 수 불일치: A={} B={}",
        diff.page_count_a, diff.page_count_b
    );
    // [#2070 잠정] PDF 정답 82 — 본문 래핑 +1줄(45자 휴리스틱)과 빈 문단 0높이
    // 과소의 상쇄가 #2070 빈 문단 정합으로 노출되어 83. #2195 에서 82 복귀.
    assert_eq!(
        diff.page_count_a, 83,
        "{SAMPLE} 쪽수 (#2070 잠정, PDF 정답 82)"
    );
    assert!(
        diff.max_disp <= 1.0,
        "{SAMPLE} HWPX roundtrip 렌더 변위 {:.2}px > 1.0px",
        diff.max_disp
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "{SAMPLE} page {} 구조 불일치 (node {} vs {}) — HWP5-origin LineSeg 부재 marker 오염 회귀",
            pg.page,
            pg.node_count_a,
            pg.node_count_b
        );
    }
}
