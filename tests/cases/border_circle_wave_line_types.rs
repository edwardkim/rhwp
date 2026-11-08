//! 검사 유형: 관계형·고정형(독립 물리 pen과 선 종류 기하); 페이지 절대 좌표 핀 없음.
//! 주석 갱신: 2026-11-05T13:18:44+09:00; 직전 파일 변경: 7915a7ed9f75f559f3e82aa9a18feb80c0326ce1 2026-10-29T16:17:52+09:00.
//! 측정일 2026-11-05, source 6337fe4d10; 96 DPI·Print·2px raw 실루엣(Native Skia/fresh WASM Canvas).
//! borders 전1쪽 Native 최저 98.04% / fresh WASM 최저 97.73%; 누락 없음. 관계 검사로 본문 프레임과 선 패턴을 구분한다.
//! 증거: mydocs/pr/assets/semanticist21-20261105-next10/validation/pre8622/candidate/{native,wasm}/borders.tsv 및 circle-print-geometry.md.
//! 독립 PDF와 변경 영역을 직접 확대 판독한다.
//!
//! 글자·문단 테두리의 원형 점선(7)·물결선(12)·이중 물결선(13)을 한컴 인쇄 기하로 긋는다.
//!
//! 단위는 1/600 inch 한 칸, w는 테두리 굵기다. 수정 전에는 원형 점선이 [2 2] 사각
//! 점선으로, 물결선 두 종류가 실선으로 그려졌다.
//!
//! 원형 점선은 Windows 한컴 2020/2024의 16개 선언 굵기 Print를 대조했다.
//! index 10(1.0 mm)은 실제 지름 36칸·중심 간격 88칸이다. 반올림한 실선 폭 w로
//! 지름과 주기를 다시 추정하지 않는다. 측정표는 정식 검토 증적에 연결한다.
//!
//! 물결선은 rhwp로 만든 시험 문서를 한컴 한글 12.30(Mac)에서 인쇄한 PDF
//! (`pdf/border_line_types/`, w 24칸)로 쟀다. 위아래 폭 w인 45° 지그재그를 굵기 w/4로
//! 긋고 꼭짓점마다 한 칸을 평평하게 이어 반주기가 w + 한 칸이다. 가닥은 선에서 3w/8
//! 떨어진 위·왼쪽에 놓이고, 상자의 아래·오른쪽 변에서는 3w/4 늦게 시작한다. 이중
//! 물결선은 반대쪽 3w/8에 가닥을 하나 더 두며, 바깥쪽 가닥이 먼저 시작한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::PathCommand;

const SAMPLE: &str = "tests/fixtures/border_line_types/borders.hwp";
/// 1/600 inch 한 칸과 테두리 굵기(24칸)의 px 값
const UNIT: f64 = 96.0 / 600.0;
const W: f64 = 24.0 * UNIT;

fn paths(node: &RenderNode, out: &mut Vec<(Vec<(f64, f64)>, f64, bool)>) {
    if let RenderNodeType::Path(path) = &node.node_type {
        let points = path
            .commands
            .iter()
            .filter_map(|command| match *command {
                PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => Some((x, y)),
                PathCommand::CurveTo(.., x, y) => Some((x, y)),
                _ => None,
            })
            .collect();
        out.push((
            points,
            path.style.stroke_width,
            path.style.fill_color.is_some(),
        ));
    }
    for child in &node.children {
        paths(child, out);
    }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn circle_and_wave_borders_follow_hancom_print_geometry() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let body_width = page
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap()
        .bbox
        .width;
    let mut found = Vec::new();
    paths(&page.root, &mut found);

    // 문단 테두리의 가로 변은 짧은 글자 테두리와 달리 본문 프레임 너비를 차지한다.
    let waves: Vec<&Vec<(f64, f64)>> = found
        .iter()
        .filter(|(points, stroke, filled)| {
            !filled
                && near(*stroke, W / 4.0)
                && points.last().unwrap().0 - points[0].0 > body_width / 2.0
        })
        .map(|(points, ..)| points)
        .collect();
    assert_eq!(waves.len(), 6, "물결선 위·아래 2가닥 + 이중 물결선 4가닥");
    for points in &waves {
        let top = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        for (i, pair) in points.windows(2).enumerate() {
            let (dx, dy) = (pair[1].0 - pair[0].0, (pair[1].1 - pair[0].1).abs());
            // 꼭짓점 사이 45° 사선(w)과 꼭짓점의 평평한 한 칸이 번갈아 온다.
            let (want_dx, want_dy) = if i % 2 == 0 { (W, W) } else { (UNIT, 0.0) };
            assert!(
                near(dx, want_dx) && near(dy, want_dy),
                "{i}번째 마디 {dx:.3},{dy:.3}"
            );
        }
        assert!(near(points[0].1, top), "위 꼭짓점에서 시작");
    }

    // 물결선 문단: 위 변과 아래 변은 같은 높이 차이로 위쪽에 놓이고, 아래 변이 3w/4 늦다.
    let (top, bottom) = (waves[0], waves[1]);
    assert!(near(bottom[0].0 - top[0].0, 0.75 * W), "아래 변 시작 지연");
    // 이중 물결선 문단: 위 변은 위 가닥이, 아래 변은 아래 가닥이 바깥이라 먼저 시작한다.
    let mut double: Vec<_> = waves[2..].to_vec();
    double.sort_by(|a, b| a[0].1.total_cmp(&b[0].1));
    let [t_out, t_in, b_in, b_out] = [double[0], double[1], double[2], double[3]];
    assert!(near(t_in[0].1 - t_out[0].1, 0.75 * W), "위 변 두 가닥 간격");
    assert!(
        near(b_out[0].1 - b_in[0].1, 0.75 * W),
        "아래 변 두 가닥 간격"
    );
    assert!(
        near(t_in[0].0 - t_out[0].0, 0.75 * W),
        "위 변 안쪽 가닥 지연"
    );
    assert!(
        near(b_in[0].0 - b_out[0].0, 0.75 * W),
        "아래 변 안쪽 가닥 지연"
    );
    assert!(
        near(t_out[0].0, b_out[0].0),
        "바깥 가닥은 같은 자리에서 시작"
    );

    // 원형 점선: 같은 선언 index의 독립 Print 지름·중심 간격으로 선 위에 찍는다.
    let circles: Vec<&Vec<(f64, f64)>> = found
        .iter()
        .filter(|(points, _, filled)| *filled && points.len() % 5 == 0)
        .map(|(points, ..)| points)
        .collect();
    let row = circles
        .iter()
        .find(|points| points.last().unwrap().0 - points[0].0 > body_width / 2.0)
        .expect("문단 원형 점선 가로 변");
    let centers: Vec<(f64, f64)> = row.chunks(5).map(|c| (c[1].0, c[0].1)).collect();
    let radius = row[0].0 - centers[0].0;
    assert!(near(radius, 36.0 * UNIT / 2.0), "반지름 {radius:.3}");
    for pair in centers.windows(2) {
        assert!(near(pair[1].0 - pair[0].0, 88.0 * UNIT), "점 간격");
        assert!(near(pair[1].1, pair[0].1), "선 위에 찍는다");
    }
}
