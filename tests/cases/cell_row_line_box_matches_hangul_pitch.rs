//! 칸 줄 상자가 한/글 정본과 같은 높이를 갖는다(#7685).
//!
//! # 무엇이 깨져 있었나
//!
//! 1. 저장 줄이 없는 문단의 그림 칸 — `samples/issue7182/rowbreak_cell_picture_only_paragraph.hwp`
//!    두 번째 표의 칸 (1,1) 은 글자 없는 문단 하나에 `TopAndBottom` 그림 하나를 단다. 측정은
//!    400HU 자리 줄(5.33px)을 `text_height` 에 싣고 그림 높이를 또 더했다. #6660 이 같은 자리를
//!    빼지만 **저장 줄** 높이만 빼서, 저장 줄이 없는 문단에는 0 을 뺐다. 행이 5.3px 씩 커져
//!    3쪽 표 바닥이 1070.0 → 1079.6px 로 내려갔다.
//! 2. 강제 줄 나눔만 남은 줄 — `samples/issue1891/80168_regulatory_analysis.hwpx` 150쪽 1×1 표의
//!    칸 문단은 저장 줄이 모두 0 이라 다시 조판된다. `⏎②공익사업의…` 처럼 줄 나눔으로 시작하면
//!    첫 줄에 글자가 없어 물리 프레임 재조판이 12px 기본 글꼴로 줄 상자를 만들었다(14.4px 간격).
//!
//! # 독립 기대값
//!
//! - 한/글 정본 `pdf/rowbreak_cell_picture_only_paragraph-2020.pdf` 3쪽(쪽 척도 보정) 가로 괘선:
//!   72.6 → 220.2 (행 1 = 147.6px). 저장 기하로는 칸 위아래 여백 141HU×2 + 그림 세로 오프셋
//!   186HU + 그림 높이 10595HU = 11063HU = 147.51px.
//! - 한/글 정본 `pdf/issue1891/80168_regulatory_analysis-hwpx-2020.pdf` 150쪽: `축물이 무허가…`
//!   502.7 → `②공익사업의…` 541.1 = 38.4px = 글자 줄 두 칸(12pt 160% = 19.2px × 2).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("재현체 {}: {e}", full.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn find_cell(node: &RenderNode, rows: u16, row: u16, col: u16, in_table: bool) -> Option<f64> {
    let in_table = match &node.node_type {
        RenderNodeType::Table(table) => table.row_count == rows,
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
        .find_map(|child| find_cell(child, rows, row, col, in_table))
}

fn text_of(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    own + &node.children.iter().map(text_of).collect::<String>()
}

fn line_tops(node: &RenderNode, out: &mut Vec<(f64, String)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push((node.bbox.y, text_of(node)));
    }
    for child in &node.children {
        line_tops(child, out);
    }
}

#[test]
fn picture_only_cell_without_stored_lines_is_padding_offset_and_picture() {
    let core = load("samples/issue7182/rowbreak_cell_picture_only_paragraph.hwp");
    let tree = core.build_page_render_tree(2).expect("3쪽 렌더 트리");
    let height = find_cell(&tree.root, 14, 1, 1, false).expect("14행 표의 칸 (1,1)");
    let expected = (141.0 * 2.0 + 186.0 + 10595.0) * 96.0 / 7200.0;
    assert!(
        (height - expected).abs() < 0.5,
        "칸 (1,1) 높이 {height:.2} ≠ 여백+오프셋+그림 {expected:.2}"
    );
}

#[test]
fn line_break_only_line_keeps_the_break_char_line_box() {
    let core = load("samples/issue1891/80168_regulatory_analysis.hwpx");
    let tree = core.build_page_render_tree(149).expect("150쪽 렌더 트리");
    let mut lines = Vec::new();
    line_tops(&tree.root, &mut lines);
    let top = |prefix: &str| {
        lines
            .iter()
            .find(|(_, text)| text.starts_with(prefix))
            .map(|(y, _)| *y)
            .unwrap_or_else(|| panic!("`{prefix}` 줄을 찾지 못했습니다"))
    };
    let gap = top("②공익사업의 시행으로") - top("축물이 무허가건축물등인");
    assert!(
        (gap - 38.4).abs() < 0.5,
        "줄 나눔만 남은 줄을 사이에 둔 간격 {gap:.2} ≠ 한/글 38.4"
    );
}
