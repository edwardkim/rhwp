//! 값이 최댓값 근처에 몰린 양수 차트의 값축은 0이 아닌 값에서 시작한다.
//!
//! 한컴(웹한글기안기 인쇄)은 `(최대-최소)/최대 < 0.2` 인 양수 데이터의 값축을
//! `최소 - 최대×0.1` 을 눈금 단위로 내린 값에서 시작한다. rhwp 는 양수 최솟값을
//! 항상 0으로 내려 막대 차이가 사라져 보였다. 0이 아닌 값에서 시작하는
//! 세로 값축의 눈금 간격도 차트 높이에 따른 한컴 칸 수와 맞춘다. 입력은 공개 샘플 차트의 값을 바꾸고
//! 차트 제목을 지운 합성 문서이고, 기대 눈금은 같은 문서의 웹한글기안기 인쇄 PDF
//! (`pdf/issue8678/*-webhwp-print.pdf`)에서 읽은 값이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 첫 쪽 SVG 의 차트 묶음 안에서 숫자로만 된 글자(값축 눈금)를 순서대로 모은다.
fn value_axis_labels(sample: &str) -> Vec<f64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("첫 쪽 SVG");
    let doc = roxmltree::Document::parse(&svg).expect("SVG 파싱");
    let chart = doc
        .descendants()
        .find(|n| n.attribute("class") == Some("hwp-ooxml-chart"))
        .unwrap_or_else(|| panic!("{sample}: 차트 묶음 없음"));
    chart
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .filter_map(|t| t.trim().parse::<f64>().ok())
        .collect()
}

fn assert_axis(sample: &str, expected: &[f64]) {
    let mut labels = value_axis_labels(sample);
    labels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    labels.dedup();
    assert_eq!(
        labels.len(),
        expected.len(),
        "{sample}: 눈금 개수 (실제 {labels:?}, 기대 {expected:?})"
    );
    for (actual, want) in labels.iter().zip(expected) {
        assert!(
            (actual - want).abs() < 1e-9,
            "{sample}: 눈금 {labels:?} != 기대 {expected:?}"
        );
    }
}

/// 묶은 세로 막대 4.65~4.95 → 4~5, 0.2 단위 (최솟값 0 아님).
#[test]
fn column_axis_starts_below_clustered_minimum() {
    assert_axis(
        "samples/issue8678/column_axis_floor.hwpx",
        &[4.0, 4.2, 4.4, 4.6, 4.8, 5.0],
    );
}

/// 최댓값 100 이상이고 차이가 최대의 1/6 미만이면 최솟값 바로 아래에서 시작한다.
/// 290,300 → 288~302, 2 단위.
#[test]
fn large_column_axis_hugs_minimum() {
    assert_axis(
        "samples/issue8678/column_axis_floor_large.hwpx",
        &[288.0, 290.0, 292.0, 294.0, 296.0, 298.0, 300.0, 302.0],
    );
}

/// 묶은 가로 막대도 같은 규칙: 4.65~4.95 → 값축이 0이 아닌 4.1 에서 시작하고 5에서 끝난다.
#[test]
fn bar_axis_starts_below_clustered_minimum() {
    let mut labels = value_axis_labels("samples/issue8678/bar_axis_floor.hwpx");
    labels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let first = *labels.first().expect("눈금 없음");
    let last = *labels.last().expect("눈금 없음");
    assert!(
        (first - 4.1).abs() < 1e-9 && (last - 5.0).abs() < 1e-9,
        "가로 막대 값축 {labels:?} 은 4.1~5 이어야 한다"
    );
}

/// 세로 값축의 칸 수는 차트 높이를 따른다. 그림 영역이 높으면(26000 HU 차트)
/// 2.9,3 의 축이 2.6~3.05, 0.05 단위로 촘촘해진다.
#[test]
fn tall_column_axis_uses_finer_step() {
    assert_axis(
        "samples/issue8678/column_axis_step_tall.hwpx",
        &[2.6, 2.65, 2.7, 2.75, 2.8, 2.85, 2.9, 2.95, 3.0, 3.05],
    );
}
