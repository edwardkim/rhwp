//! 관계형: 칸 대각선 모양 코드마다 그리는 대각선 수와 방향, 꼭짓점을 검사한다.
//! MEASUREMENT
//!
//! BORDER_FILL attr의 대각선 모양은 `/`(bit 2~4)와 `\`(bit 5~7)마다 3비트다.
//! HWPX는 이 값을 NONE(0)·CENTER(0b010)·CENTER_BELOW(0b011)·CENTER_ABOVE(0b110)·
//! ALL(0b111)로 쓰며, 꼭짓점 사이 대각선은 0b010 비트다. 한컴 한글 12.30(Mac)
//! 인쇄 PDF(`pdf/cell_diagonal_shapes/`)에서 꺾은선 비트가 없을 때 2·3·6은 같은
//! 꼭짓점 대각선 하나이고, 0b010 비트가 없는 1은 아무것도 그리지 않는다.
//! 두 방향을 함께 준 칸(2/2, 3/3, 6/6, 3/6, 6/3)은 모두 X 하나다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/cell_diagonal_shapes/diagonal-shape-codes.hwp";

/// 행마다 오른쪽 칸에 준 (slash, backSlash) 모양 코드. 왼쪽 칸은 그 값을 적은 이름표다.
const CODES: [(u8, u8); 13] = [
    (1, 0),
    (2, 0),
    (3, 0),
    (6, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 6),
    (2, 2),
    (3, 3),
    (6, 6),
    (3, 6),
    (6, 3),
];

fn collect(node: &RenderNode, cells: &mut Vec<(u16, BoundingBox)>, lines: &mut Vec<[f64; 4]>) {
    match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.col == 1 => cells.push((cell.row, node.bbox)),
        RenderNodeType::Line(l) if (l.x1 - l.x2).abs() > 0.5 && (l.y1 - l.y2).abs() > 0.5 => {
            lines.push([l.x1, l.y1, l.x2, l.y2]);
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, cells, lines);
    }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.05
}

#[test]
fn only_center_bit_draws_corner_diagonal() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let (mut cells, mut lines) = (Vec::new(), Vec::new());
    collect(&page.root, &mut cells, &mut lines);
    assert_eq!(cells.len(), CODES.len(), "13행 표의 오른쪽 칸");

    for (row, bbox) in cells {
        let (slash, back) = CODES[row as usize];
        let (left, top) = (bbox.x, bbox.y);
        let (right, bottom) = (bbox.x + bbox.width, bbox.y + bbox.height);
        let inside: Vec<_> = lines
            .iter()
            .filter(|l| {
                [l[0], l[2]]
                    .iter()
                    .all(|&x| x > left - 0.05 && x < right + 0.05)
                    && [l[1], l[3]]
                        .iter()
                        .all(|&y| y > top - 0.05 && y < bottom + 0.05)
            })
            .collect();
        // `/`는 왼쪽 아래 → 오른쪽 위, `\`는 왼쪽 위 → 오른쪽 아래 꼭짓점을 잇는다.
        let slash_lines = inside
            .iter()
            .filter(|l| {
                near(l[0], left) && near(l[1], bottom) && near(l[2], right) && near(l[3], top)
            })
            .count();
        let back_lines = inside
            .iter()
            .filter(|l| {
                near(l[0], left) && near(l[1], top) && near(l[2], right) && near(l[3], bottom)
            })
            .count();
        let want = (
            usize::from(slash & 0b010 != 0),
            usize::from(back & 0b010 != 0),
        );
        assert_eq!(
            (slash_lines, back_lines, inside.len()),
            (want.0, want.1, want.0 + want.1),
            "행 {row} slash={slash} backSlash={back}: 칸 안 사선 {inside:?}"
        );
    }
}
