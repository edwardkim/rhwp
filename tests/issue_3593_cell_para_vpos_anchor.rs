//! [#3593] 저장 vpos=0 문단의 소유·순서·물리 셀 포함을 검사한다.
//!
//! 독립 Print에서 「로터리 조종기」와 「(Rotary controls)」는 같은 논리 셀의
//! 103·104쪽 조각에 나뉜다. 두 문단이 한 물리 셀에 함께 있어야 한다는 전제는
//! 올바른 분할을 실패시킨다. 각 줄은 자신의 조각 안에 한 번씩 남아야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::layout::CellContext;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType, TableNode};

const SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const PARA_A: &str = "로터리 조종기";
const PARA_B: &str = "(Rotary controls";

#[derive(Clone, Debug, PartialEq)]
struct CellOwner {
    table_para: Option<usize>,
    table_control: Option<usize>,
    context: Option<CellContext>,
    cell_index: Option<u32>,
    row: u16,
    col: u16,
}

#[derive(Clone, Debug)]
struct TargetLine {
    page: usize,
    owner: CellOwner,
    cell: BoundingBox,
    line: BoundingBox,
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

fn collect_targets(
    node: &RenderNode,
    page: usize,
    table: Option<&TableNode>,
    cell: Option<(CellOwner, BoundingBox)>,
    targets: &mut [Vec<TargetLine>; 2],
) {
    let (table, cell) = match &node.node_type {
        RenderNodeType::Table(table) => (Some(table), None),
        RenderNodeType::TableCell(info) => {
            let table = table.expect("cell must belong to a table");
            let owner = CellOwner {
                table_para: table.para_index,
                table_control: table.control_index,
                context: table.cell_context.clone(),
                cell_index: info.model_cell_index,
                row: info.row,
                col: info.col,
            };
            (Some(table), Some((owner, node.bbox)))
        }
        _ => (table, cell),
    };
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text = subtree_text(node);
        for (index, needle) in [PARA_A, PARA_B].iter().enumerate() {
            if text.contains(needle) {
                let (owner, cell) = cell.as_ref().expect("target paragraph must own a cell");
                targets[index].push(TargetLine {
                    page,
                    owner: owner.clone(),
                    cell: *cell,
                    line: node.bbox,
                });
            }
        }
    }
    for child in &node.children {
        collect_targets(child, page, table, cell.clone(), targets);
    }
}

fn load_targets() -> [TargetLine; 2] {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse sample");
    let mut targets = [Vec::new(), Vec::new()];
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render page");
        collect_targets(&tree.root, page as usize, None, None, &mut targets);
    }
    let [a, b] = targets;
    assert_eq!(a.len(), 1, "paragraph A must occur exactly once: {a:?}");
    assert_eq!(b.len(), 1, "paragraph B must occur exactly once: {b:?}");
    [a[0].clone(), b[0].clone()]
}

#[test]
fn cell_paragraphs_with_zero_vpos_preserve_order_across_fragments() {
    let [a, b] = load_targets();
    assert_eq!(
        a.owner, b.owner,
        "both paragraphs must retain their logical cell"
    );
    assert!(
        a.page < b.page || (a.page == b.page && a.line.y + a.line.height <= b.line.y + 0.5),
        "paragraph order/occupancy must survive the split: {a:?}, {b:?}"
    );
}

#[test]
fn each_cell_fragment_contains_its_accepted_paragraph_lines() {
    for target in load_targets() {
        assert!(
            target.line.y >= target.cell.y - 0.5
                && target.line.y + target.line.height <= target.cell.y + target.cell.height + 0.5,
            "accepted line escapes its own physical cell fragment: {target:?}"
        );
    }
}
