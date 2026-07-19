//! Issue #2278 (C2b, #1431 Track C): 3D 입체·ofPie 보조플롯 렌더 회귀 가드.
//!
//! Stage 1 — 3D 막대 압출: bar3DChart 4종(묶은/누적 × 세로/가로)이 top/side
//! 압출 면(`hwp-bar3d-top`/`hwp-bar3d-side` 폴리곤)을 방출하고, 축 라벨
//! (#1882 3D 축 앵커)은 불변임을 가드.
//!
//! Stage 2 — 3D 원형: pie3DChart(코퍼스 rotX=30/persp=30)가 타원 top 슬라이스
//! (`hwp-pie3d-top`)와 하반부 측벽(`hwp-pie3d-wall`)을 방출함을 가드.
//!
//! Stage 3 — ofPie 보조플롯: 원형대원형/원형대가로막대형이 주 원 + 보조 플롯 +
//! serLines(`hwp-ofpie-*`)를 방출하고, 결합 슬라이스가 실측 팔레트 [4]
//! (#27a172)를 사용함을 가드.
//!
//! 주의: 페이지 SVG 전역에는 도형/WMF `<polygon>`이 존재할 수 있으므로
//! 면 계수는 반드시 `hwp-bar3d-*`/`hwp-pie3d-*`/`hwp-ofpie-*` 클래스 기준으로 한다.

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

/// (stem, 축 라벨 존재 문구(#1882 앵커), 축 라벨 부재 문구)
const BAR3D_STEMS: &[(&str, &[&str], &[&str])] = &[
    ("세로막대형/3차원묶은세로막대형", &[">5<"], &[">6<"]),
    ("세로막대형/3차원누적세로막대형", &[">20<"], &[]),
    ("가로막대형/3차원묶은가로막대형", &[">5<"], &[">6<"]),
    ("가로막대형/3차원누적가로막대형", &[">14<"], &[">20<"]),
];

#[test]
fn bar3d_charts_emit_extrusion_faces_with_stable_axis() {
    for (stem, present, absent) in BAR3D_STEMS {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);

            // 코퍼스 3계열 × 4카테고리 = 12 막대(세그먼트) — 면 12쌍
            let tops = svg.matches("hwp-bar3d-top").count();
            let sides = svg.matches("hwp-bar3d-side").count();
            assert_eq!(tops, 12, "{rel}: top 면 12개 (3계열×4카테고리)");
            assert_eq!(sides, 12, "{rel}: side 면 12개");

            // 3D 방(뒷벽+바닥+커넥터) 1회 — 시각판정 보정(2026-07-16) 범위 추가
            assert_eq!(svg.matches("hwp-bar3d-room").count(), 1, "{rel}: 3D 방 1회");

            // #1882 3D 축 앵커 불변 (압출은 rect 방출만 대체 — 축 계산 무접촉)
            for want in *present {
                assert!(
                    svg.contains(want),
                    "{rel}: 축 라벨 {want} 소실 (#1882 앵커)"
                );
            }
            for no in *absent {
                assert!(!svg.contains(no), "{rel}: 축 라벨 {no} 출현 (#1882 앵커)");
            }
        }
    }
}

#[test]
fn ofpie_charts_emit_secondary_plot_and_serlines() {
    // 코퍼스 [10, 3.5, 1.5, 1.2] → 주 원 3(2+결합) + 보조 2 (기본 k=2)
    for (stem, second_is_rect) in [("원형대원형", false), ("원형대가로막대형", true)] {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/원형/{stem}.{ext}");
            let svg = render_page0_svg(&rel);

            assert_eq!(
                svg.matches("hwp-ofpie-main").count(),
                3,
                "{rel}: 주 원 슬라이스 3"
            );
            assert_eq!(
                svg.matches("hwp-ofpie-second").count(),
                2,
                "{rel}: 보조 플롯 2"
            );
            assert_eq!(
                svg.matches("hwp-ofpie-serline").count(),
                2,
                "{rel}: serLines 2"
            );
            // 결합 슬라이스 = 실측 팔레트 [4] 초록계
            assert!(svg.contains("#27a172"), "{rel}: 결합 슬라이스 실측색");
            if second_is_rect {
                assert!(
                    svg.contains("<rect class=\"hwp-ofpie-second\""),
                    "{rel}: 가로막대형 보조는 rect"
                );
            }
            assert!(
                !svg.contains("hwp-ooxml-chart-fallback"),
                "{rel}: placeholder 잔존"
            );
        }
    }
}

#[test]
fn pie3d_chart_emits_ellipse_tops_and_lower_walls() {
    for ext in ["hwpx", "hwp"] {
        let rel = format!("samples/chart/원형/3차원원형.{ext}");
        let svg = render_page0_svg(&rel);

        // 코퍼스 4슬라이스 — top 4개, 하반부 노출 슬라이스만 벽(≥1)
        assert_eq!(
            svg.matches("hwp-pie3d-top").count(),
            4,
            "{rel}: top 타원 슬라이스 4개"
        );
        assert!(
            svg.matches("hwp-pie3d-wall").count() >= 1,
            "{rel}: 하반부 측벽 ≥ 1"
        );
        // placeholder 미출현 (렌더 전환 완료)
        assert!(
            !svg.contains("hwp-ooxml-chart-fallback"),
            "{rel}: placeholder 잔존"
        );
    }
}
