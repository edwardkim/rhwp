//! Issue #2278 (C2b, #1431 Track C): 3D 입체·ofPie 보조플롯 렌더 회귀 가드.
//!
//! Stage 1 — 3D 막대 압출: bar3DChart 4종(묶은/누적 × 세로/가로)이 top/side
//! 압출 면(`hwp-bar3d-top`/`hwp-bar3d-side` 폴리곤)을 방출하고, 축 라벨
//! (#1882 3D 축 앵커)은 불변임을 가드.
//!
//! 주의: 페이지 SVG 전역에는 도형/WMF `<polygon>`이 존재할 수 있으므로
//! 면 계수는 반드시 `hwp-bar3d-*` 클래스 기준으로 한다.

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
