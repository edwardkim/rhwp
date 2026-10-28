#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
fn descendants<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        descendants(child, nodes);
    }
}
// 독립 한컴 Print에서 확인한 개체 앵커와 바깥 위 여백의 관계를 검사한다.
// 특정 글꼴 파일이나 절대 px 대신 저장 줄 높이에 대한 비율과 내용 소유를 사용한다.
#[test]
fn saved_empty_table_anchor_preserves_its_outer_box_in_the_same_frame() {
    let core = DocumentCore::from_bytes(
        &std::fs::read(
            "mydocs/pr/assets/semanticist21-20261005/pr7504/pr7504-registered-arial.hwp",
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(core.page_count(), 1);
    let paras = &core.document().sections[0].paragraphs;
    let first = &paras[0].line_segs[0];
    let anchor = &paras[1].line_segs[0];
    assert_eq!(first.vertical_pos, 0);
    assert_eq!(anchor.segment_width, 0);
    let Control::Table(source) = &paras[1].controls[0] else {
        panic!("원본 표");
    };
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let lines: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(
                &node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(0)
            )
        })
        .collect();
    let first_line = lines
        .iter()
        .copied()
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .unwrap();
    let tables: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(
                &node.node_type, RenderNodeType::Table(table) if table.para_index == Some(1)
            )
        })
        .collect();
    assert_eq!(tables.len(), 1);
    let table = tables[0];
    let expected_ratio =
        f64::from(anchor.vertical_pos - first.vertical_pos + i32::from(source.outer_margin_top))
            / f64::from(first.line_height);
    let actual_ratio = (table.bbox.y - first_line.bbox.y) / first_line.bbox.height;
    assert!(
        (actual_ratio - expected_ratio).abs() <= f64::EPSILON * 128.0,
        "저장 원점과 바깥 위 여백의 비율: actual={actual_ratio}, expected={expected_ratio}"
    );
    let cells: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(
                &node.node_type, RenderNodeType::TextRun(run) if run.cell_context.is_some()
            )
        })
        .collect();
    assert!(!cells.is_empty());
    for cell in cells {
        assert!(cell.bbox.y >= table.bbox.y);
        assert!(cell.bbox.y + cell.bbox.height <= table.bbox.y + table.bbox.height);
    }
    let following = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(2)
            )
        })
        .unwrap();
    assert!(following.bbox.y >= table.bbox.y + table.bbox.height);
}
