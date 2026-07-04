//! Issue #1453 (C1a, #1431 Track C): 3D막대·3D원형·ofPie 차트 라우팅 회귀 가드.
//!
//! 파서가 이미 series·값·cats를 추출하던 7종(3D막대 4 + 3D원형 1 + ofPie 2)이
//! 요소명 미인식으로 `chart_type=Unknown`이 되어 "차트 (미지원)" placeholder로
//! 렌더되던 문제(`renderer.rs` fallback)를, `handle_start`에 `bar3DChart`/`pie3DChart`/
//! `ofPieChart` 라우팅을 추가해 기존 막대/원형 렌더러로 그리도록 한 회귀 가드.
//!
//! 검증: 7종 × (hwp, hwpx) = 14파일 각각 page 0 SVG가
//!   - "차트 (미지원)" placeholder **미포함**
//!   - 정상 차트 클래스 `hwp-ooxml-chart"` **포함** (fallback `hwp-ooxml-chart-fallback` 아님)

use std::fs;
use std::path::Path;

/// 7종 차트 (samples/chart 하위 상대경로, 확장자 제외)
const CHART_STEMS: &[&str] = &[
    "세로막대형/3차원묶은세로막대형", // bar3DChart, barDir=col → Column
    "세로막대형/3차원누적세로막대형", // bar3DChart, barDir=col → Column
    "가로막대형/3차원묶은가로막대형", // bar3DChart, barDir=bar → Bar
    "가로막대형/3차원누적가로막대형", // bar3DChart, barDir=bar → Bar
    "원형/3차원원형",                 // pie3DChart → Pie
    "원형/원형대원형",                // ofPieChart (ofPieType=pie) → Pie
    "원형/원형대가로막대형",          // ofPieChart (ofPieType=bar) → Pie
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
fn chart_3d_ofpie_routed_no_unsupported_placeholder() {
    for stem in CHART_STEMS {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);

            assert!(
                !svg.contains("차트 (미지원)"),
                "{rel}: '차트 (미지원)' placeholder가 남아있음 (라우팅 누락)",
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

/// Part B (#1453): 막대 누적(stacked/percentStacked) 6종 — C1a 3D 누적(2) + 기존 2D 누적/백프로(4).
/// percent 여부 = `c:grouping=percentStacked` 샘플.
const STACKED_BAR_STEMS: &[(&str, bool)] = &[
    ("세로막대형/3차원누적세로막대형", false),
    ("가로막대형/3차원누적가로막대형", false),
    ("세로막대형/누적세로막대형", false),
    ("가로막대형/누적가로막대형", false),
    ("세로막대형/백프로기준누적세로막대형", true),
    ("가로막대형/백프로기준누적가로막대형", true),
];

#[test]
fn chart_stacked_bars_render_with_percent_axis() {
    for (stem, is_percent) in STACKED_BAR_STEMS {
        let rel = format!("samples/chart/{stem}.hwpx");
        let svg = render_page0_svg(&rel);

        assert!(
            !svg.contains("차트 (미지원)") && svg.contains("hwp-ooxml-chart\""),
            "{rel}: 누적 막대 정상 렌더 실패",
        );
        // 백분율 누적은 % 축 라벨(0%/100%)을 가진다. 일반 누적은 가지지 않는다.
        if *is_percent {
            assert!(
                svg.contains("100%"),
                "{rel}: percentStacked인데 % 축(100%) 라벨 없음",
            );
        } else {
            assert!(
                !svg.contains("100%"),
                "{rel}: 일반 stacked인데 % 축이 잘못 적용됨",
            );
        }
    }
}
