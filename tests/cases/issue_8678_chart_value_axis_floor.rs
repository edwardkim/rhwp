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

/// 모든 쪽의 차트마다 값축 눈금의 (시작, 끝, 간격) 을 쪽·차트 순서대로 모은다.
fn chart_axes(sample: &str) -> Vec<(f64, f64, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open");
    let mut axes = Vec::new();
    for page in 0..core.page_count() {
        let svg = core.render_page_svg_native(page).expect("SVG");
        let doc = roxmltree::Document::parse(&svg).expect("SVG 파싱");
        for chart in doc
            .descendants()
            .filter(|n| n.attribute("class") == Some("hwp-ooxml-chart"))
        {
            let mut labels: Vec<f64> = chart
                .descendants()
                .filter(|n| n.has_tag_name("text"))
                .filter_map(|n| n.text())
                .filter_map(|t| t.trim().parse::<f64>().ok())
                .collect();
            labels.sort_by(|a, b| a.partial_cmp(b).unwrap());
            labels.dedup();
            assert!(labels.len() >= 2, "{sample}: 눈금 부족 {labels:?}");
            let step = labels[1] - labels[0];
            assert!(
                labels
                    .windows(2)
                    .all(|w| ((w[1] - w[0]) - step).abs() < 1e-9),
                "{sample}: 간격이 고르지 않음 {labels:?}"
            );
            axes.push((labels[0], *labels.last().unwrap(), step));
        }
    }
    axes
}

fn assert_axes(sample: &str, expected: &[(f64, f64, f64)]) {
    let actual = chart_axes(sample);
    assert_eq!(actual.len(), expected.len(), "{sample}: 차트 수 {actual:?}");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a.0 - e.0).abs() < 1e-9 && (a.1 - e.1).abs() < 1e-9 && (a.2 - e.2).abs() < 1e-9,
            "{sample}: {}번째 차트 {a:?} != 데스크톱 {e:?}",
            i + 1
        );
    }
}

/// 값만 바꾼 묶은 세로·가로 막대 18개. 기대값은 같은 문서의 데스크톱 한컴 2020 Print
/// (`pdf/issue8678/axis_residual_set1-hwpx-2020.pdf`) 에서 읽은 눈금이다.
/// 9.5,9.8 · 95,98 · 9.5,9.9 는 최댓값 아래로 3칸 남짓만 남아 한 칸 더 내려가고,
/// 3,3(18750 HU) 은 0~3.5 0.5 단위, 가로 2.9,3(32250 HU) 은 0.1 단위다.
#[test]
fn desktop_axis_set1_matches_print() {
    assert_axes(
        "samples/issue8678/axis_residual_set1.hwpx",
        &[
            (8.0, 10.0, 0.5),
            (80.0, 100.0, 5.0),
            (8.6, 10.0, 0.2),
            (8.0, 10.0, 0.5),
            (8.6, 10.0, 0.2),
            (8.0, 10.5, 0.5),
            (86.0, 100.0, 2.0),
            (2.6, 3.1, 0.1),
            (17.0, 20.5, 0.5),
            (75.0, 100.0, 5.0),
            (0.0, 4.0, 1.0),
            (0.0, 3.5, 0.5),
            (0.0, 3.5, 0.5),
            (0.0, 5.0, 1.0),
            (2.6, 3.1, 0.1),
            (2.6, 3.05, 0.05),
            (2.6, 3.1, 0.1),
            (4.1, 5.0, 0.1),
        ],
    );
}

/// 바닥값을 내리는 경계(최댓값 아래 3.2칸은 내리고 3.4칸은 유지), 칸 수 한도로 고르는
/// 간격(9.6,9.95 → 0.2, 1,5 → 2, 2,4.5 높은 차트 → 0.5), 가로 막대 너비별 간격 12개.
/// 기대값은 `pdf/issue8678/axis_residual_set2-hwpx-2020.pdf` 에서 읽었다.
#[test]
fn desktop_axis_set2_matches_print() {
    assert_axes(
        "samples/issue8678/axis_residual_set2.hwpx",
        &[
            (8.0, 10.5, 0.5),
            (8.0, 10.5, 0.5),
            (8.5, 10.5, 0.5),
            (8.6, 10.0, 0.2),
            (0.0, 7.0, 1.0),
            (0.0, 6.0, 2.0),
            (0.0, 5.0, 0.5),
            (0.0, 7.0, 1.0),
            (8.4, 10.0, 0.2),
            (2.6, 3.1, 0.1),
            (2.6, 3.05, 0.05),
            (4.1, 5.0, 0.1),
        ],
    );
}
