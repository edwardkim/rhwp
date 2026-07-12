//! Issue #2129 (C1d, #1431 Track C): 라인 누적(stacked/percentStacked) + 표식 회귀 가드.
//!
//! 한컴 2022 정답지(`pdf/chart/라인/`) 실측 기준:
//!   - 누적꺽은선형: 축 0~15 step 5 (카테고리 합 12.3), 누적합 위치, 무마커
//!   - 백프로기준누적꺽은선형: 축 0%~100% step 20%, 최상위 계열 100% 수평선
//!   - 표식이있는누적꺽은선형: 누적 + 계열별 마커 ◆■▲
//!   - 표식이있는꺽은선형: 독립 선(개별값 축 0~6) + 마커
//!   - 꺽은선형: 독립 선 + 무마커 (무회귀 핀)
//!
//! 라인 5종 × (hwp, hwpx) = 10파일의 page 0 SVG substring으로 검증.

use std::fs;
use std::path::Path;

fn render_page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {}: {:?}", rel, e))
}

fn for_both_exts(stem: &str, f: impl Fn(&str, &str)) {
    for ext in ["hwpx", "hwp"] {
        let rel = format!("samples/chart/라인/{stem}.{ext}");
        f(&rel, &render_page0_svg(&rel));
    }
}

#[test]
fn line_all_render_without_placeholder() {
    // 라인 5종 전부 placeholder 없이 정상 차트 클래스로 렌더.
    for stem in [
        "꺽은선형",
        "표식이있는꺽은선형",
        "누적꺽은선형",
        "표식이있는누적꺽은선형",
        "백프로기준누적꺽은선형",
    ] {
        for_both_exts(stem, |rel, svg| {
            assert!(!svg.contains("차트 (미지원)"), "{rel}: placeholder 발생");
            assert!(
                svg.contains("hwp-ooxml-chart\""),
                "{rel}: 차트 그룹 클래스 없음"
            );
        });
    }
}

#[test]
fn line_stacked_axis_and_no_markers() {
    // 누적: 축 = 카테고리 합(12.3) 기반 0~15 step 5 (개별값 0~6 아님) + 무마커.
    for_both_exts("누적꺽은선형", |rel, svg| {
        for want in [">5<", ">10<", ">15<"] {
            assert!(
                svg.contains(want),
                "{rel}: 누적 축 라벨 {want} 없음 (0~15 step 5)"
            );
        }
        assert!(!svg.contains(">14<"), "{rel}: step 2 회귀 (14 라벨)");
        assert!(!svg.contains(">6<"), "{rel}: 개별값 축(0~6) 회귀");
        assert!(
            !svg.contains("hwp-chart-marker"),
            "{rel}: 무마커 샘플에 마커"
        );
    });
}

#[test]
fn line_percent_stacked_axis() {
    // 백프로: 축 0%~100% step 20% + 무마커.
    for_both_exts("백프로기준누적꺽은선형", |rel, svg| {
        for want in ["0%", "20%", "100%"] {
            assert!(svg.contains(want), "{rel}: percent 축 라벨 {want} 없음");
        }
        assert!(
            !svg.contains("hwp-chart-marker"),
            "{rel}: 무마커 샘플에 마커"
        );
    });
}

#[test]
fn line_stacked_with_markers() {
    // 표식누적: 누적 축(0~15) + 마커 12개(3계열×4점).
    for_both_exts("표식이있는누적꺽은선형", |rel, svg| {
        assert!(svg.contains(">15<"), "{rel}: 누적 축 라벨 15 없음");
        assert_eq!(
            svg.matches("hwp-chart-marker").count(),
            12,
            "{rel}: 마커 수 12(3계열×4점) 아님"
        );
    });
}

#[test]
fn line_nonstacked_with_markers() {
    // 표식(비누적): 개별값 축(0~6) 유지 + 마커 12개 — 같은 플래그로 함께 정합.
    for_both_exts("표식이있는꺽은선형", |rel, svg| {
        assert!(
            svg.contains(">6<"),
            "{rel}: 개별값 축 라벨 6 없음 (0~6 step 2)"
        );
        assert!(!svg.contains(">15<"), "{rel}: 비누적에 누적 축 적용됨");
        assert_eq!(
            svg.matches("hwp-chart-marker").count(),
            12,
            "{rel}: 마커 수 12 아님"
        );
    });
}

#[test]
fn line_plain_unchanged() {
    // 꺽은선형 무회귀 핀: 개별값 축 0~6 + 무마커.
    for_both_exts("꺽은선형", |rel, svg| {
        for want in [">0<", ">2<", ">4<", ">6<"] {
            assert!(
                svg.contains(want),
                "{rel}: 축 라벨 {want} 없음 (0~6 step 2)"
            );
        }
        assert!(!svg.contains("hwp-chart-marker"), "{rel}: 기본 샘플에 마커");
        assert!(!svg.contains(">15<"), "{rel}: 기본 샘플에 누적 축");
    });
}
