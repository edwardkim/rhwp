//! Issue #1660 (C1b, #1431 Track C): 분산형(scatter) 차트 렌더 커버리지 회귀 가드.
//!
//! 분산형 5종은 파서가 `c:xVal`/`c:yVal`를 읽지 못하고 `scatterChart` 미인식으로
//! `chart_type=Unknown`이 되어 "차트 (미지원)" placeholder로 렌더되던 문제를,
//! 파서(`c:scatterChart`/`c:xVal`/`c:yVal`/`c:scatterStyle`)와 렌더러(`render_scatter`)를
//! 추가해 실제 산점도로 그리도록 한 회귀 가드.
//!
//! 검증: 5종 × (hwp, hwpx) = 10파일 각각 page 0 SVG가
//!   - "차트 (미지원)" placeholder **미포함**
//!   - 정상 차트 클래스 `hwp-ooxml-chart"` **포함** (fallback `hwp-ooxml-chart-fallback` 아님)

use std::fs;
use std::path::Path;

/// 분산형 5종 (samples/chart 하위 상대경로, 확장자 제외)
const SCATTER_STEMS: &[&str] = &[
    "분산형/표식만있는분산형",       // scatterStyle=marker → 표식만
    "분산형/직선이있는분산형",       // scatterStyle=line → 직선
    "분산형/직선및표식이있는분산형", // scatterStyle=lineMarker → 직선+표식
    "분산형/곡선이있는분산형",       // scatterStyle=smoothMarker → 곡선
    "분산형/곡선및표식이있는분산형", // scatterStyle=smoothMarker → 곡선+표식
];

fn render_page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {}: {:?}", rel, e))
}

#[test]
fn scatter_charts_no_unsupported_placeholder() {
    for stem in SCATTER_STEMS {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);

            assert!(
                !svg.contains("차트 (미지원)"),
                "{rel}: '차트 (미지원)' placeholder가 남아있음 (scatter 렌더 누락)",
            );
            assert!(
                svg.contains("hwp-ooxml-chart\""),
                "{rel}: 정상 차트(hwp-ooxml-chart) 미렌더",
            );
            assert!(
                !svg.contains("hwp-ooxml-chart-fallback"),
                "{rel}: fallback 차트가 렌더됨",
            );
        }
    }
}
