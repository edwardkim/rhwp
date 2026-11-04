//! 엇갈린 병합 칸들의 행 높이를 칸 내용으로 푼다.
//!
//! 한컴 한글 12.30(Mac)에서 표 일부 블록에 "셀 높이를 같게"를 적용해 저장한 문서다.
//! 한컴은 14행을 27행으로 다시 나눈다. 선택한 오른쪽 칸은 모두 1500HU(20px)이고,
//! 고르지 않은 왼쪽 칸은 두 행씩 걸치며 원래 선언 282HU를 그대로 둔다. 왼쪽 칸의
//! 실제 높이는 글줄과 안 여백(18.4px)이 정하고, 두 열은 한 행씩 엇갈려 있다.
//!
//! 한컴 인쇄 PDF(`pdf/rowspan_staggered_heights/`)에서 왼쪽 칸은 18.4px,
//! 오른쪽 칸은 20px 간격이고 두 열의 표 끝이 같다. 선언 높이만으로 행을 풀면
//! 사슬이 282HU에서 끊겨 남은 행을 똑같이 나누므로 왼쪽 칸이 20px로 늘고
//! 표가 약 20px 길어졌다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp";
const PAGE: u32 = 1;
const TABLE_PARA: usize = 25;

#[derive(Debug)]
struct Cell {
    row: u16,
    col: u16,
    row_span: u16,
    bbox: BoundingBox,
}

fn table_cells(node: &RenderNode, out: &mut Vec<Cell>) -> bool {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(TABLE_PARA) && table.cell_context.is_none() {
            for child in &node.children {
                if let RenderNodeType::TableCell(cell) = &child.node_type {
                    out.push(Cell {
                        row: cell.row,
                        col: cell.col,
                        row_span: cell.row_span,
                        bbox: child.bbox,
                    });
                }
            }
            return true;
        }
    }
    node.children.iter().any(|child| table_cells(child, out))
}

fn cell_at(cells: &[Cell], row: u16, col: u16) -> &Cell {
    cells
        .iter()
        .find(|cell| cell.row == row && cell.col == col)
        .unwrap_or_else(|| panic!("r{row} c{col} 칸"))
}

#[test]
fn staggered_span_cells_keep_their_own_heights() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 2 render tree");
    let mut cells = Vec::new();
    assert!(table_cells(&page.root, &mut cells), "27행 표");

    // 한 행짜리 칸이 각 열의 기준 높이다: 왼쪽 끝 행은 내용 높이, 오른쪽 r12는 1500HU.
    let left = cell_at(&cells, 26, 0).bbox.height;
    let right = cell_at(&cells, 12, 3).bbox.height;
    assert!(
        (right - 1500.0 / 75.0).abs() < 0.05,
        "선택한 칸 높이 1500HU: {right:.2}"
    );

    for cell in cells
        .iter()
        .filter(|cell| cell.row_span == 2 && cell.row >= 2)
    {
        let expected = if cell.col <= 2 { left } else { right };
        assert!(
            (cell.bbox.height - expected).abs() < 0.05,
            "r{} c{} 칸 높이 {:.2}, 같은 열의 한 행 칸 {:.2}",
            cell.row,
            cell.col,
            cell.bbox.height,
            expected
        );
    }

    // 두 열의 사슬이 같은 표 끝에서 닫히고, 오른쪽 열은 머리 칸 두 개와
    // 1500HU 칸 열둘이 그대로 쌓인다.
    let top = cell_at(&cells, 0, 3).bbox.y;
    let left_bottom = cell_at(&cells, 26, 0).bbox.y + left;
    let right_end = cell_at(&cells, 25, 3);
    let right_bottom = right_end.bbox.y + right_end.bbox.height;
    assert!(
        (left_bottom - right_bottom).abs() < 0.05,
        "열 끝: 왼쪽 {left_bottom:.2}, 오른쪽 {right_bottom:.2}"
    );
    assert!(
        (right_bottom - top - 14.0 * right).abs() < 0.05,
        "표 높이 {:.2}, 1500HU 칸 14개 {:.2}",
        right_bottom - top,
        14.0 * right
    );
}
