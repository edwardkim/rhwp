//! 관계형 회귀 검사. #8579 원본과 메인테이너 경계 사례.
//! 주석 갱신 2026-11-26 11:26 KST. 직전 파일 변경 b825ba521f 2026-11-26T00:17:55+09:00.
//! 독립 한컴 Print와 96 DPI의 2px raw 실루엣. 전쪽 TSV · 확대 판독: `mydocs/pr/assets/hyunmyungjanelee-20261126/final/README.md`.
//! `picture_both_sides` 전 2쪽. Native 100.00, 100.00%, fresh WASM 100.00, 100.00% (쪽 순서).
//!
//! 검사 유형: 관계형.
//!
//! #8579 잔여: 저장 줄 없는 칸에서 문단 기준 어울림(Square) 그림 뒤 글 문단.
//!
//! 재현물 `samples/issue8579/partial_table_following_text.hwpx` 의 첫 행 둘째 칸은
//! 그림만 든 문단(p0, 가로 오프셋 1650HU) 뒤에 `FOLLOW PICTURE` 문단(p1)을 둔다.
//! 칸 안쪽 폭 8584HU 에서 그림 왼쪽 빈 폭은 1650HU, 오른쪽은 1428HU 다.
//! 한/글 2020 데스크톱 인쇄본(`pdf/issue8579/partial_table_following_text-2020.pdf`)은
//! p1 을 p0 줄 바로 아래에서 시작해 그림 왼쪽 빈 폭(최소 줄 폭 1440HU 이상인 쪽)에
//! 글자 단위로 흘려 쓰고(`F O`·`L L`·`OW`·`PIC`), 그림 상자 아래부터 칸 전폭으로 쓴다.
//! 오른쪽 빈 폭은 최소 줄 폭보다 좁아 비운다. 종전에는 p1 전체가 그림 상자 아래로
//! 내려가 칸 아래쪽 줄 위치가 달랐다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8579/partial_table_following_text.hwpx");
    DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture")).expect("HWPX parse")
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

struct Probe {
    cell: BoundingBox,
    image: BoundingBox,
    /// (문단, 줄 상자, 글자 run 상자들)
    lines: Vec<(usize, BoundingBox, Vec<(String, BoundingBox)>)>,
}

fn first_row_picture_text_cell(core: &DocumentCore) -> Probe {
    let tree = core.build_page_render_tree(0).expect("page tree");
    let cell = nodes(&tree.root)
        .into_iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 0 && c.col == 1))
        .expect("r0c1 cell");
    let image = nodes(cell)
        .into_iter()
        .find_map(|n| matches!(n.node_type, RenderNodeType::Image(_)).then_some(n.bbox))
        .expect("Square picture");
    let lines = nodes(cell)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextLine(line) => {
                let runs = nodes(n)
                    .into_iter()
                    .filter_map(|r| match &r.node_type {
                        RenderNodeType::TextRun(run) if !run.text.trim().is_empty() => {
                            Some((run.text.clone(), r.bbox))
                        }
                        _ => None,
                    })
                    .collect();
                Some((line.para_index?, n.bbox, runs))
            }
            _ => None,
        })
        .collect();
    Probe {
        cell: cell.bbox,
        image,
        lines,
    }
}

#[test]
fn following_text_flows_beside_square_picture_from_host_line() {
    let probe = first_row_picture_text_cell(&core());
    let host_bottom = probe
        .lines
        .iter()
        .filter(|(para, _, _)| *para == 0)
        .map(|(_, bbox, _)| bbox.y + bbox.height)
        .reduce(f64::max)
        .expect("host line");
    let text: Vec<_> = probe
        .lines
        .iter()
        .filter(|(para, _, runs)| *para == 1 && !runs.is_empty())
        .collect();
    let first = text.first().expect("following text line");
    // 글은 그림 아래가 아니라 host 줄 바로 아래(그림 상자 안 높이)에서 시작한다.
    assert!(
        first.1.y < probe.image.y + probe.image.height - 1.0,
        "첫 글줄 위 {:.1} 가 그림 바닥 {:.1} 보다 위여야 한다",
        first.1.y,
        probe.image.y + probe.image.height
    );
    assert!(
        first.1.y >= host_bottom - 0.5,
        "첫 글줄 위 {:.1} 는 host 줄 바닥 {host_bottom:.1} 아래다",
        first.1.y
    );
    // 그림 상자와 세로로 겹치는 글줄은 그림 왼쪽 빈 폭(줄 상자)에 놓이고, 낱말은
    // 그 폭에 맞춰 글자 단위로 끊긴다.
    let mut beside = 0;
    for (_, line, runs) in &text {
        if line.y >= probe.image.y + probe.image.height {
            continue;
        }
        assert!(
            line.x >= probe.cell.x - 0.5 && line.x + line.width <= probe.image.x + 0.5,
            "그림 옆 줄 상자 [{:.1}, {:.1}] 가 그림 왼쪽 [{:.1}, {:.1}] 안이어야 한다",
            line.x,
            line.x + line.width,
            probe.cell.x,
            probe.image.x
        );
        for (run, bbox) in runs {
            assert!(
                bbox.x >= line.x - 0.5 && bbox.x < probe.image.x,
                "그림 옆 글 {run:?} 시작 {:.1} 가 줄 상자 안이어야 한다",
                bbox.x
            );
            assert!(
                run.trim().chars().count() <= 3,
                "그림 옆 글 {run:?} 은 좁은 폭에서 글자 단위로 끊겨야 한다"
            );
            beside += 1;
        }
    }
    assert!(beside >= 3, "그림 옆에 놓인 글 run 이 너무 적다: {beside}");
    // 마지막 글줄은 그림 아래에서 칸 안에 들어간다.
    let last = text.last().expect("last text line");
    assert!(
        last.1.y + last.1.height <= probe.cell.y + probe.cell.height + 0.5,
        "마지막 글줄이 칸 아래를 넘는다"
    );
}

#[test]
fn following_text_uses_both_available_sides_of_the_same_row() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8579/picture_both_sides.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let probe = first_row_picture_text_cell(&core);
    let inside =
        |line: &BoundingBox| line.y >= probe.image.y && line.y < probe.image.y + probe.image.height;
    let mut text = String::new();
    let mut left = Vec::new();
    let mut right = Vec::new();
    let picture_right = probe.image.x + probe.image.width;
    for (paragraph, line, runs) in &probe.lines {
        if *paragraph != 1 || runs.is_empty() {
            continue;
        }
        for (run, _) in runs {
            text.extend(run.chars().filter(|ch| !ch.is_whitespace()));
        }
        if !inside(line) {
            continue;
        }
        assert!(
            line.x >= probe.cell.x - 0.5
                && line.x + line.width <= probe.cell.x + probe.cell.width + 0.5
        );
        if line.x + line.width <= probe.image.x + 0.5 {
            left.push(line.y);
        } else {
            assert!(line.x >= picture_right - 0.5, "줄이 그림과 겹친다");
            right.push(line.y);
        }
    }
    assert_eq!(text, "FOLLOWPICTURE");
    assert!(
        !left.is_empty() && left.len() == right.len(),
        "왼쪽 {left:?}, 오른쪽 {right:?}"
    );
    for (l, r) in left.iter().zip(right) {
        assert!((l - r).abs() < 0.05, "같은 줄의 글자 {l} / {r}");
    }
}
