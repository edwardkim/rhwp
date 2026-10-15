//! RowBreak 표의 행 내부 조각 때문에 전체 조각이 쪽 경계를 넘으면
//! 그 조각을 현재 쪽에 남기지 않는다.
//!
//! `samples/kps-ai.hwp` 37쪽의 329문단은 32행 2열 RowBreak 표다.
//! 독립 Print의37쪽은 성능 요구사항으로 끝나고38쪽은 데이터 요구사항부터 잇는다.
//! 고정 행 번호 대신 원본 소유·쪽별 내용·본문 포함 관계를 검사한다.
//! v448 Native/fresh 전77쪽 최저90.85010%, 누락·미달0, 전쪽 RGBA 동일.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn page_dump(rel_path: &str, page_idx: u32) -> String {
    let doc = load_doc(rel_path);
    doc.dump_page_items(Some(page_idx))
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn owned_table(root: &RenderNode, pi: usize, ci: usize) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
        .into_iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
        if t.para_index == Some(pi) && t.control_index == Some(ci))
        })
        .expect("원본 표의 실제 쪽 조각")
}

fn subtree_text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    node.children.iter().fold(own, |mut text, child| {
        text.push_str(&subtree_text(child));
        text
    })
}

#[test]
fn kps_ai_page37_defers_overflowing_split_row_slice() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/kps-ai.hwp");
    let core = DocumentCore::from_bytes(&fs::read(path).expect("KPS 원본")).unwrap();
    let (pi, ci, original) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(ci, control)| {
                    let Control::Table(table) = control else {
                        return None;
                    };
                    table
                        .cells
                        .iter()
                        .any(|c| {
                            c.paragraphs
                                .iter()
                                .any(|p| compact(&p.text) == "4.성능요구사항")
                        })
                        .then_some((pi, ci, table))
                        .filter(|(_, _, table)| {
                            table.cells.iter().any(|c| {
                                c.paragraphs.iter().any(|p| compact(&p.text) == "작성방법")
                            })
                        })
                })
        })
        .expect("제안서 작성방법 표의 원본 소유자");
    let boundary = original
        .cells
        .iter()
        .find(|c| {
            c.paragraphs
                .iter()
                .any(|p| compact(&p.text) == "5.데이터요구사항")
        })
        .expect("독립 Print의 다음 쪽 시작 항목")
        .row;
    let first = core.build_page_render_tree(36).unwrap();
    let next = core.build_page_render_tree(37).unwrap();
    let first_table = owned_table(&first.root, pi, ci);
    let next_table = owned_table(&next.root, pi, ci);
    assert!(
        first_table
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TableCell(c) => Some(c),
                _ => None,
            })
            .all(|c| c.row < boundary),
        "데이터 항목의 일부를 앞 조각에 남기지 않는다"
    );
    assert!(next_table.children.iter().any(|n| matches!(&n.node_type,
        RenderNodeType::TableCell(c) if c.row == boundary)));
    // 두 조각은 원본의 모든 행을 보존하고 반복 머리행만 공유한다.
    let left_rows = |table: &RenderNode| {
        table
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TableCell(cell) if cell.col == 0 => Some(cell.row),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    let rows = left_rows(first_table);
    let following = left_rows(next_table);
    let headers = first_table
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell)
                if cell.col == 0 && compact(&subtree_text(node)) == "항목" =>
            {
                Some(cell.row)
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        rows.intersection(&following)
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        headers,
        "반복 머리행만 두 조각에 속한다"
    );
    assert_eq!(
        rows.union(&following).count(),
        usize::from(original.row_count),
        "두 조각은 원본의 모든 행을 보존한다"
    );
    for (tree, table) in [(&first, first_table), (&next, next_table)] {
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        let column = nodes
            .into_iter()
            .find(|n| {
                matches!(n.node_type, RenderNodeType::Column(_))
                    && n.children.iter().any(|c| c.id == table.id)
            })
            .expect("조각의 소유 단");
        assert!(
            table.bbox.y >= column.bbox.y - 1e-8
                && table.bbox.y + table.bbox.height <= column.bbox.y + column.bbox.height + 1e-8,
            "표 조각 전체는 소유 본문 단 안에 있어야 한다"
        );
    }
    let first_text = compact(&core.extract_page_text_native(36).unwrap());
    let next_text = compact(&core.extract_page_text_native(37).unwrap());
    assert!(first_text.contains("4.성능요구사항") && !first_text.contains("5.데이터요구사항"));
    assert!(!next_text.contains("4.성능요구사항") && next_text.contains("5.데이터요구사항"));
}

#[test]
fn synam_001_page14_uses_row_budget_after_repeated_header() {
    let sample = "samples/synam-001.hwp";

    let page13 = page_dump(sample, 12);
    assert!(
        page13.contains("PartialTable   pi=140 ci=0  rows=2..7"),
        "page 13 should include the first visible slice of row 6 after repeated header rows:\n{page13}"
    );
    assert!(
        page13.contains("start_cut=[2, 19] end_cut=[2, 13]"),
        "page 13 should continue row 2 and cut into row 6 instead of deferring row 6:\n{page13}"
    );

    let page14 = page_dump(sample, 13);
    assert!(
        page14.contains("start_cut=[2, 13]"),
        "page 14 should continue from the row 6 cut already started on page 13:\n{page14}"
    );
}
