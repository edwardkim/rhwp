//! 독립 Print65/66쪽의 병합 라벨은 원본 셀과 같은 내용 컷을 이어받는다.
//! 고정 SVG 좌표 대신 바깥 표 → 소유 셀 → 안쪽 표 → 병합 셀의 경로를 따른다.
//! v448 Native/fresh 전77쪽 최저90.85010%, 누락·미달0, 전쪽 RGBA 동일.

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph, table::Cell};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[derive(Clone, Copy)]
struct Owner {
    pi: usize,
    ci: usize,
    row: u16,
    col: u16,
}

fn source(paragraphs: &[Paragraph]) -> Option<(Vec<Owner>, &Cell)> {
    for (pi, paragraph) in paragraphs.iter().enumerate() {
        for (ci, control) in paragraph.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            for cell in &table.cells {
                let owner = Owner {
                    pi,
                    ci,
                    row: cell.row,
                    col: cell.col,
                };
                let text: String = cell.paragraphs.iter().map(|p| compact(&p.text)).collect();
                if text == "3.민간소프트웨어시장침해가능성" {
                    return Some((vec![owner], cell));
                }
                if let Some((mut path, label)) = source(&cell.paragraphs) {
                    path.insert(0, owner);
                    return Some((path, label));
                }
            }
        }
    }
    None
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn owned_cell<'a>(root: &'a RenderNode, path: &[Owner]) -> &'a RenderNode {
    let mut scope = root;
    for owner in path {
        let mut nodes = Vec::new();
        walk(scope, &mut nodes);
        let table = nodes
            .into_iter()
            .find(|n| {
                matches!(&n.node_type,
            RenderNodeType::Table(t) if t.para_index == Some(owner.pi)
                && t.control_index == Some(owner.ci))
            })
            .expect("원본 경로의 표 조각");
        scope = table
            .children
            .iter()
            .find(|n| {
                matches!(&n.node_type,
            RenderNodeType::TableCell(c) if c.row == owner.row && c.col == owner.col)
            })
            .expect("원본 경로의 소유 셀");
    }
    scope
}

fn text_inside(cell: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(cell, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|node| {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                return None;
            };
            if !run.text.trim().is_empty() {
                assert!(
                    node.bbox.y >= cell.bbox.y - 1e-8
                        && node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height + 1e-8,
                    "글줄은 자신의 병합 셀 조각 안에 표시되어야 한다"
                );
            }
            Some(compact(&run.text))
        })
        .collect()
}

fn check_fragments() -> (String, String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/kps-ai.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let (owners, original) = source(&core.document().sections[0].paragraphs).unwrap();
    let reset = original
        .paragraphs
        .windows(2)
        .position(|pair| {
            pair[0]
                .line_segs
                .last()
                .zip(pair[1].line_segs.first())
                .is_some_and(|(before, after)| after.vertical_pos < before.vertical_pos)
        })
        .expect("독립 Print와 대응하는 원본 문단의 저장 줄 재시작")
        + 1;
    let expected_first: String = original.paragraphs[..reset]
        .iter()
        .map(|p| compact(&p.text))
        .collect();
    let expected_next: String = original.paragraphs[reset..]
        .iter()
        .map(|p| compact(&p.text))
        .collect();
    assert_eq!(expected_first, "3.민간소프트웨어");
    assert_eq!(expected_next, "시장침해가능성");
    let first = core.build_page_render_tree(64).unwrap();
    let next = core.build_page_render_tree(65).unwrap();
    let first_text = text_inside(owned_cell(&first.root, &owners));
    let next_text = text_inside(owned_cell(&next.root, &owners));
    assert_eq!(
        first_text.clone() + &next_text,
        original
            .paragraphs
            .iter()
            .map(|p| compact(&p.text))
            .collect::<String>(),
        "병합 라벨의 순서·누락·중복을 함께 검사한다"
    );
    (first_text, next_text)
}

#[test]
fn first_fragment_does_not_spill_continuation_paragraphs() {
    let (first, _) = check_fragments();
    assert_eq!(first, "3.민간소프트웨어");
}

#[test]
fn continuation_fragment_receives_the_remaining_paragraphs() {
    let (_, next) = check_fragments();
    assert_eq!(next, "시장침해가능성");
}
