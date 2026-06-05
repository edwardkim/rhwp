//! Issue #1285: 한 셀 안에 TAC 표 2개가 같은 줄에 배치된 오른쪽 정렬 문단 회귀 가드.
//!
//! 재현 문서: `samples/21_언어_기출_편집가능본.hwp` 1쪽 머리말 영역의
//! `성명`/`수험번호` 답안지 표.
//!
//! 핵심:
//! - 부모 셀 문단은 `성명` TAC 표 + 공백 4자 + `수험번호` TAC 표를 같은 줄에 배치한다.
//! - 부모 셀의 오른쪽 정렬 폭 계산은 두 번째 TAC 표 전체 폭까지 포함해야 한다.
//! - `수험번호` TAC 표 내부 첫 셀은 오른쪽 정렬이며, 음수 자간 압축 후 실제 렌더 폭 기준으로
//!   셀 오른쪽에 붙어야 한다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TableCellNode};
use rhwp::wasm_api::HwpDocument;

fn load_tree() -> rhwp::renderer::render_tree::PageRenderTree {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/21_언어_기출_편집가능본.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse sample");
    doc.build_page_render_tree(0).expect("render page 1")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn all_nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
}

fn first_text_run<'a>(node: &'a RenderNode, text: &str) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(tr) if tr.text == text) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| first_text_run(child, text))
}

fn first_cell(node: &RenderNode, row: u16, col: u16) -> Option<&RenderNode> {
    node.children.iter().find(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::TableCell(TableCellNode {
                row: r,
                col: c,
                ..
            }) if *r == row && *c == col
        )
    })
}

fn direct_text_line_with_text<'a>(node: &'a RenderNode, text: &str) -> Option<&'a RenderNode> {
    node.children.iter().find(|child| {
        matches!(child.node_type, RenderNodeType::TextLine(_))
            && first_text_run(child, text).is_some()
    })
}

#[test]
fn answer_sheet_two_tac_tables_keep_inline_sequence() {
    let tree = load_tree();
    let nodes = all_nodes(&tree.root);

    let name_table = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::Table(t)
                    if t.row_count == 1
                        && t.col_count == 2
                        && node.bbox.y < 320.0
                        && first_text_run(node, "성명").is_some()
            )
        })
        .copied()
        .expect("성명 TAC table");
    let number_table = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::Table(t)
                    if t.row_count == 1
                        && t.col_count == 8
                        && node.bbox.y < 320.0
                        && first_text_run(node, "수험번호").is_some()
            )
        })
        .copied()
        .expect("수험번호 TAC table");
    let spaces = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(tr)
                    if tr.text == "    " && node.bbox.y < 320.0 && node.bbox.width > 10.0
            )
        })
        .copied()
        .expect("TAC tables 사이 공백 TextRun");
    let outer_cell = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TableCell(TableCellNode { row: 2, col: 1, .. })
            ) && first_text_run(node, "성명").is_some()
                && first_text_run(node, "수험번호").is_some()
        })
        .copied()
        .expect("성명/수험번호 TAC 표를 포함하는 부모 셀");
    let parent_line = direct_text_line_with_text(outer_cell, "    ").expect("부모 셀 inline line");

    let name_right = name_table.bbox.x + name_table.bbox.width;
    let spaces_right = spaces.bbox.x + spaces.bbox.width;
    let number_right = number_table.bbox.x + number_table.bbox.width;
    let parent_line_right = parent_line.bbox.x + parent_line.bbox.width;

    assert!(
        (name_right - spaces.bbox.x).abs() <= 0.75,
        "성명 TAC 표 뒤에 공백 TextRun이 바로 이어져야 함: name_right={name_right:.2}, spaces_x={:.2}",
        spaces.bbox.x
    );
    assert!(
        (spaces_right - number_table.bbox.x).abs() <= 0.75,
        "공백 TextRun 뒤에 수험번호 TAC 표가 바로 이어져야 함: spaces_right={spaces_right:.2}, number_x={:.2}",
        number_table.bbox.x
    );
    assert!(
        (parent_line_right - number_right).abs() <= 1.0,
        "부모 셀 오른쪽 정렬 폭 계산은 두 번째 TAC 표 전체 폭까지 포함해야 함: \
         parent_line_right={parent_line_right:.2}, number_right={number_right:.2}, \
         outer_cell={:?}, parent_line={:?}, number_table={:?}",
        outer_cell.bbox,
        parent_line.bbox,
        number_table.bbox
    );
}

#[test]
fn answer_sheet_number_label_right_align_uses_rendered_width() {
    let tree = load_tree();
    let nodes = all_nodes(&tree.root);

    let number_table = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::Table(t)
                    if t.row_count == 1
                        && t.col_count == 8
                        && node.bbox.y < 320.0
                        && first_text_run(node, "수험번호").is_some()
            )
        })
        .copied()
        .expect("수험번호 TAC table");
    let label_cell = first_cell(number_table, 0, 0).expect("수험번호 label cell");
    let label_line = label_cell
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .expect("수험번호 label line");
    let label_run = first_text_run(label_cell, "수험번호").expect("수험번호 TextRun");

    let line_right = label_line.bbox.x + label_line.bbox.width;
    let run_right = label_run.bbox.x + label_run.bbox.width;
    assert!(
        (line_right - run_right).abs() <= 1.0,
        "수험번호 TextRun은 압축 후 실제 렌더 폭 기준으로 첫 셀 오른쪽에 정렬되어야 함: \
         line_right={line_right:.2}, run_right={run_right:.2}, run={:?}, line={:?}",
        label_run.bbox,
        label_line.bbox
    );
}
