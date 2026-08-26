//! Issue #2277 (C2a stage5, #1431 Track C): 특이케이스 1카테고리 미니차트 축 회귀 가드.
//!
//! 정답지(`pdf/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목-2022.pdf`) 실측:
//! 값 4.3의 가로막대 1카테고리 미니차트는 가로 값축이 **0~5 step 0.5**(라벨 11개) —
//! 기존 가로축 앵커(12.3→step2 / 5.0→step1 / 2.6→step0.5)와 단일 규칙이 성립하지
//! 않아 C1c v2에서 기록만 했던 특수 동작. `가로 && 1카테고리 && 비누적·비3D`로
//! 좁게 게이트해 step 절반 적용 (코퍼스 나머지 27종은 전부 4카테고리 — 회귀 반경 0).

use std::fs;
use std::path::Path;

const STEM: &str = "특이케이스/가로막대형_하나만있을떄_단일시리즈제목";

fn render_page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {}: {:?}", rel, e))
}

#[test]
fn mini_chart_horizontal_axis_uses_half_step() {
    for ext in ["hwpx", "hwp"] {
        let rel = format!("samples/chart/{STEM}.{ext}");
        let svg = render_page0_svg(&rel);
        for want in [">0.5<", ">4.5<", ">5<"] {
            assert!(
                svg.contains(want),
                "{rel}: 미니차트 0.5 step 축 라벨 {want} 없음 (정답지 0~5 step 0.5)",
            );
        }
        // 기존 가드 유지 확인: 자동 제목 = 시리즈 이름 (issue_1882 v2와 중복 핀)
        assert!(svg.contains(">계열 1<"), "{rel}: 단일 시리즈 제목 유지");
    }
}
