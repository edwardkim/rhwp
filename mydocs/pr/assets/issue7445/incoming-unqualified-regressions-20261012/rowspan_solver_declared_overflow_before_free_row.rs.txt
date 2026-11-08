//! 병합 칸 선언으로 행 높이를 풀 때, 걸친 행이 모두 정해진 제약의 선언 초과분을 먼저 반영한 뒤
//! 미지 행을 푼다(#7685).
//!
//! # 무엇이 깨져 있었나
//!
//! `samples/task2287/1342000_edu_curriculum_map.hwp` 구역 10 의 217×9 표에서 행 191 에는 rs=1 칸이
//! 없다(미지 행). 측정기와 조판의 행 높이 풀이는
//!
//! ```text
//!   1. 「초 3~4」 (행 182~191, 26063HU = 347.51px) 로 행 191 = 347.51 − 335.44 = 12.07
//!   2. 「미술」   (행 184~187, 11352HU = 151.36px) 선언 초과 12.06px 를 마지막 행 187 에 가산
//! ```
//!
//! 순서로 같은 부족분을 두 번 넣었다. 행 191~192 에 걸친 칸들(2782HU = 37.09px)이 49.15px 로
//! 그려지고, 그 아래 행이 모두 12px 내려갔다.
//!
//! # 독립 기대값
//!
//! - 저장 칸 높이: 행 191~192 에 걸친 칸 (191,4)~(191,8) 의 선언이 2782HU, 행 191~193 의 「국어」
//!   (191,3) 가 5564HU = 2782 + 2782 이다. 행 191+192 는 2782HU 여야 이 둘이 함께 성립한다.
//! - 한/글 정본(`pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf` 209쪽, 쪽 척도 보정)에서 행 188~200
//!   글줄은 rhwp 와 일정한 차(+5.2px)를 유지한다. 수정 전에는 행 191·193 에서 −6·−12px 계단이 생겼다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task2287/1342000_edu_curriculum_map.hwp";
/// 행 191~192 에 걸친 칸의 저장 선언 높이(2782HU).
const STORED_TWO_ROW_CELL_PX: f64 = 2782.0 * 96.0 / 7200.0;

fn load() -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("재현체 {}: {e}", full.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 행 수가 217 인 표에서 `(row, col)` 칸 상자 높이.
fn cell_height(node: &RenderNode, in_table: bool, row: u16, col: u16) -> Option<f64> {
    let in_table = match &node.node_type {
        RenderNodeType::Table(table) => table.row_count == 217,
        _ => in_table,
    };
    if in_table {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            if cell.row == row && cell.col == col {
                return Some(node.bbox.height);
            }
        }
    }
    node.children
        .iter()
        .find_map(|child| cell_height(child, in_table, row, col))
}

#[test]
fn two_row_cells_keep_their_stored_height_when_a_free_row_is_solved() {
    let core = load();
    let mut found = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        for col in 4..=8 {
            if let Some(height) = cell_height(&tree.root, false, 191, col) {
                found.push((page, col, height));
            }
        }
    }
    assert!(found.len() >= 5, "행 191 칸을 찾지 못했습니다: {found:?}");
    for (page, col, height) in found {
        assert!(
            (height - STORED_TWO_ROW_CELL_PX).abs() < 0.5,
            "{page}쪽 칸 (191,{col}) 높이 {height:.2} ≠ 저장 선언 {STORED_TWO_ROW_CELL_PX:.2}"
        );
    }
}
