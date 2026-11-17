//! 표 칸 안의 글 앞으로(InFrontOfText) 중첩 표는 칸의 글 흐름을 밀지 않는다. 그 칸이 쪽 끝에서
//! 나뉠 때도 행 나눔 유닛에 중첩 표 높이를 더하지 않는다.
//!
//! 재현물 `samples/pk40991_overlay_nested/overlay_nested_split.{hwpx,hwp}`: 공개 샘플
//! `samples/issue3738/tac_sibling_shape_line_advance.hwpx` 의 머리·구역 정의·표 XML 을 조합한
//! 합성본이다. 1열 3행 표의 가운데 칸 첫 문단에 문단 기준 글 앞으로 중첩 표(1열 24행,
//! 높이 57600 HU, 세로 오프셋 -10385 HU, flowWithText=1)를 두고, 그 뒤에 빈 문단 60개를 둔다.
//! 칸 문단에는 저장 줄이 없다. HWP 는 같은 HWPX 를 rhwp 로 변환한 것이다.
//!
//! 참고(웹한글기안기 인쇄): 2쪽. 중첩 표는 칸 위끝에 놓이고, 칸 높이는 빈 문단 줄만큼이며,
//! `TAIL ROW` 행과 `END` 문단은 2쪽에 놓인다.
//!
//! 수정 전 rhwp 는 칸이 쪽을 넘어 나뉠 때 중첩 표를 행 단위 흐름 유닛으로 쪼개 칸 높이에
//! 중첩 표 높이(768px)를 더했고, 3쪽이 되었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/pk40991_overlay_nested")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn has_text(node: &RenderNode, text: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(text) {
            return true;
        }
    }
    node.children.iter().any(|child| has_text(child, text))
}

/// 바깥 표(깊이 1) 가운데 행(1행) 칸 높이의 쪽별 합.
fn outer_row1_height(node: &RenderNode, depth: usize) -> f64 {
    let depth = match node.node_type {
        RenderNodeType::Table(_) => depth + 1,
        _ => depth,
    };
    if depth == 1 {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            if cell.row == 1 {
                return node.bbox.height;
            }
        }
    }
    node.children
        .iter()
        .map(|child| outer_row1_height(child, depth))
        .sum()
}

fn check(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}: 쪽 수");
    let last = core.build_page_render_tree(1).unwrap();
    assert!(has_text(&last.root, "TAIL"), "{name}: 끝 행은 2쪽");
    assert!(has_text(&last.root, "END"), "{name}: 표 뒤 문단은 2쪽");
    let row1: f64 = (0..core.page_count())
        .map(|page| outer_row1_height(&core.build_page_render_tree(page).unwrap().root, 0))
        .sum();
    // 빈 문단 60개 줄 높이(약 1280px)에 칸 여백 정도. 중첩 표 높이(768px)를 더하지 않는다.
    assert!(row1 < 1400.0, "{name}: 가운데 행 높이 {row1:.1}px");
}

#[test]
fn overlay_nested_table_does_not_grow_split_row_hwpx() {
    check("overlay_nested_split.hwpx");
}

#[test]
fn overlay_nested_table_does_not_grow_split_row_hwp5() {
    check("overlay_nested_split.hwp");
}
