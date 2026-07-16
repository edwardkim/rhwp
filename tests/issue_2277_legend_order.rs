//! Issue #2277 (C2a stage3, #1431 Track C): 범례 순서 규칙 회귀 가드.
//!
//! 정답지 PDF 28종 전수 실측(stage3 보고서 표, 예외 0) 결과, 한컴은 계열이 플롯에서
//! **세로 방향으로 배열되는 차트**의 우측 세로 범례를 시각적 상→하 순서와 일치시키기
//! 위해 역순으로 나열한다:
//!
//!   역순 = (세로 값축 && 누적/백프로: 누적·백프로 세로막대/라인) || (가로막대 && 묶음)
//!   3D는 2D와 동일 규칙. pie(카테고리)/scatter/stock/콤보 = 정순.
//!
//! C1c에서 "관찰 상충"으로 이관됐던 항목 — 상충이 아니라 위 기하학적 규칙이었다.
//! 함께 발견된 묶은가로의 플롯 슬롯 내 계열 배치(계열1이 맨 아래)도 동일 커밋에서
//! 반전 (렌더러 유닛 `test_hbar_clustered_slot_series1_at_bottom`).
//!
//! 검증: 대표 역순 4종 + 정순 4종 × (hwp, hwpx) — 우측 범례에서 `계열 3` 라벨의
//! y좌표가 `계열 1`보다 작으면(위면) 역순.

use std::fs;
use std::path::Path;

/// 역순 대표 (실측: 계열 3 → 계열 1)
const REVERSED_STEMS: &[&str] = &[
    "세로막대형/누적세로막대형",
    "세로막대형/백프로기준누적세로막대형",
    "가로막대형/묶은가로막대형",
    "라인/누적꺽은선형",
];
/// 정순 대표 (실측: 계열 1 → 계열 3)
const FORWARD_STEMS: &[&str] = &[
    "세로막대형/묶은세로막대형",
    "가로막대형/누적가로막대형",
    "가로막대형/백프로기준누적가로막대형",
    "라인/꺽은선형",
];

fn render_page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {}: {:?}", rel, e))
}

/// 계열 이름 라벨(범례에만 등장)의 `<text>` y좌표.
fn legend_label_y(svg: &str, label: &str, rel: &str) -> f64 {
    let needle = format!(">{label}</text>");
    let pos = svg
        .find(&needle)
        .unwrap_or_else(|| panic!("{rel}: 범례 라벨 {label} 없음"));
    let tstart = svg[..pos]
        .rfind("<text ")
        .unwrap_or_else(|| panic!("{rel}: {label} text 태그 없음"));
    let tag = &svg[tstart..pos];
    let p = tag.find("y=\"").expect("y attr") + 3;
    let e = tag[p..].find('"').expect("y close");
    tag[p..p + e].parse().expect("y parse")
}

#[test]
fn reversed_legends_show_last_series_on_top() {
    for stem in REVERSED_STEMS {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);
            let y3 = legend_label_y(&svg, "계열 3", &rel);
            let y1 = legend_label_y(&svg, "계열 1", &rel);
            assert!(
                y3 < y1,
                "{rel}: 역순이어야 — 계열 3(y={y3})이 계열 1(y={y1}) 위",
            );
        }
    }
}

#[test]
fn marker_line_and_scatter_legend_swatches_have_glyphs() {
    // stage4 SwatchKind 실 코퍼스 가드 — 정답지 실측:
    // 표식이있는꺽은선형 = 선분+글리프(계열 3) / 표식만있는분산형 = 글리프만(계열 2)
    for ext in ["hwpx", "hwp"] {
        let line = render_page0_svg(&format!("samples/chart/라인/표식이있는꺽은선형.{ext}"));
        assert_eq!(
            line.matches("hwp-legend-glyph").count(),
            3,
            "표식 라인({ext}): 계열별 범례 글리프",
        );
        let scatter = render_page0_svg(&format!("samples/chart/분산형/표식만있는분산형.{ext}"));
        assert_eq!(
            scatter.matches("hwp-legend-glyph").count(),
            2,
            "표식만 분산형({ext}): 계열별 범례 글리프",
        );
    }
}

#[test]
fn forward_legends_show_first_series_on_top() {
    for stem in FORWARD_STEMS {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);
            let y1 = legend_label_y(&svg, "계열 1", &rel);
            let y3 = legend_label_y(&svg, "계열 3", &rel);
            assert!(
                y1 < y3,
                "{rel}: 정순이어야 — 계열 1(y={y1})이 계열 3(y={y3}) 위",
            );
        }
    }
}
