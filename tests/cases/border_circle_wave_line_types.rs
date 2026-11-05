//! 글자·문단 테두리의 원형 점선(7)·물결선(12)·이중 물결선(13)을 한컴 인쇄 기하로 긋는다.
//!
//! 단위는 1/600 inch 한 칸, w는 테두리 굵기다. 수정 전에는 원형 점선이 [2 2] 사각
//! 점선으로, 물결선 두 종류가 실선으로 그려졌다.
//!
//! 원형 점선은 저장소의 한/글 2020·2022·2024 인쇄 PDF로 쟀다
//! (`pdf/pr-1674-2020.pdf`·`pdf/pr-1674-2024.pdf` 20쪽, `pdf/tac_object_host_line_height-2020.pdf`
//! 2쪽은 w 5칸, `pdf/2022년 국립국어원 업무계획-2022.pdf` 20쪽은 w 12칸). 채운 원의 지름은
//! 6칸·12칸, 간격은 2w이고 선 시작점부터 선 위에 놓인다.
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
    let mut found = Vec::new();
    paths(&page.root, &mut found);

    // 문단 테두리의 가로 변만 본문 폭(500px 넘게)을 차지한다.
    let waves: Vec<&Vec<(f64, f64)>> = found
        .iter()
        .filter(|(points, stroke, filled)| {
            !filled && near(*stroke, W / 4.0) && points.last().unwrap().0 - points[0].0 > 500.0
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

    // 원형 점선: 지름 w의 채운 원을 2w 간격으로 선 위에 찍는다.
    let circles: Vec<&Vec<(f64, f64)>> = found
        .iter()
        .filter(|(points, _, filled)| *filled && points.len() % 5 == 0)
        .map(|(points, ..)| points)
        .collect();
    let row = circles
        .iter()
        .find(|points| points.len() / 5 > 30)
        .expect("문단 원형 점선 가로 변");
    let centers: Vec<(f64, f64)> = row.chunks(5).map(|c| (c[1].0, c[0].1)).collect();
    let radius = row[0].0 - centers[0].0;
    assert!(near(radius, W / 2.0), "반지름 {radius:.3}");
    for pair in centers.windows(2) {
        assert!(near(pair[1].0 - pair[0].0, 2.0 * W), "점 간격");
        assert!(near(pair[1].1, pair[0].1), "선 위에 찍는다");
    }
}
