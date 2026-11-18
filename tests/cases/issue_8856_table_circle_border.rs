//! 고정형: 독립 한컴 2020/2024 PDF 벡터의 표 원형 점 지름·중심 간격을 보호한다.
//! 주석 갱신: 2026-11-10T00:51:41+09:00 (Asia/Seoul).
//! idol_goods_trimmed.hwpx 전2쪽 stage1 최소 글꼴 실측 최저: Native 99.50595%,
//! fresh WASM 99.50727%. 근거: mydocs/pr/assets/issue8856/stage1-distributed-forms/validation.json.
//! 표 borderFill 7의 CIRCLE·0.2mm는 원본·독립 Print에서 확인했다. 문단 원형 점선과
//! 다른 표 pen 계약(600dpi 외곽 지름6칸·간격10칸), 채움+외곽선과 두 번 도색을
//! 최종 Path에서 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::PathCommand;

fn circle_rows(node: &RenderNode, in_table: bool, rows: &mut Vec<Vec<(f64, f64, f64)>>) {
    let in_table = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    if in_table {
        if let RenderNodeType::Path(path) = &node.node_type {
            if path.style.fill_color.is_some()
                && path.commands.len() >= 12
                && path.commands.len() % 6 == 0
                && path.commands.chunks_exact(6).all(|dot| {
                    matches!(dot[0], PathCommand::MoveTo(..))
                        && dot[1..5]
                            .iter()
                            .all(|p| matches!(p, PathCommand::CurveTo(..)))
                        && matches!(dot[5], PathCommand::ClosePath)
                })
            {
                let points = path
                    .commands
                    .chunks_exact(6)
                    .map(|dot| {
                        let PathCommand::MoveTo(right, cy) = dot[0] else {
                            unreachable!()
                        };
                        let PathCommand::CurveTo(_, _, _, _, cx, _) = dot[1] else {
                            unreachable!()
                        };
                        (cx, cy, right - cx)
                    })
                    .collect::<Vec<_>>();
                if points.windows(2).all(|p| (p[1].1 - p[0].1).abs() < 1e-6) {
                    rows.push(
                        points
                            .into_iter()
                            .map(|(x, _, r)| (x, r, path.style.stroke_width))
                            .collect(),
                    );
                }
            }
        }
    }
    for child in &node.children {
        circle_rows(child, in_table, rows);
    }
}

#[test]
fn table_circle_uses_its_own_print_pen_in_the_final_path() {
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue8596/idol_goods_trimmed.hwpx"
    ))
    .expect("독립 Print와 같은 공개 원본");
    assert_eq!(core.page_count(), 2);
    let tree = core.build_page_render_tree(0).expect("첫 쪽의 실제 배치");
    let mut rows = Vec::new();
    circle_rows(&tree.root, false, &mut rows);
    assert!(rows.len() >= 2, "표의 위·아래 원형 경계 보존");
    let unit = 96.0 / 600.0;
    assert_eq!(
        rows.len(),
        4,
        "독립 Print처럼 위·아래 경계를 각각 두 번 도색"
    );
    for row in rows {
        for &(_, radius, stroke) in &row {
            assert!(
                (stroke - unit).abs() < 1e-6,
                "원형 점의 600dpi 한 칸 외곽선"
            );
            assert!(
                (2.0 * radius + stroke - 6.0 * unit).abs() < 1e-6,
                "표 원형 점의 채움+외곽 지름 {radius}, stroke={stroke}"
            );
        }
        for pair in row.windows(2) {
            assert!(
                (pair[1].0 - pair[0].0 - 10.0 * unit).abs() < 1e-6,
                "표 점 중심 간격"
            );
        }
    }
}
