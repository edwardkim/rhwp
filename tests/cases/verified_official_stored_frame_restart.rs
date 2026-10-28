//! 독립 한컴 Print에서 확인한 연속 표의 쪽 소유와 후속 순서를 보호한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn find_table(node: &RenderNode, para: usize, matches: &mut Vec<(u32, f64, f64)>, page: u32) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(0)
            && table.para_index == Some(para)
            && table.control_index == Some(0)
            && table.cell_context.is_none()
        {
            matches.push((page, node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        find_table(child, para, matches, page);
    }
}

#[test]
fn complete_inline_frame_and_successor_tables_keep_their_print_pages() {
    let input = include_bytes!(
        "../../mydocs/pr/assets/pr8319_evidence_alignment_20261027/official-inputs/pr8036-seoul-official.hwpx"
    );
    let core = DocumentCore::from_bytes(input).expect("서울시 공식 배포 HWPX");
    let mut prior = Vec::new();
    let mut successor = Vec::new();
    let mut heading = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        find_table(&tree.root, 145, &mut prior, page);
        find_table(&tree.root, 146, &mut successor, page);
        find_table(&tree.root, 148, &mut heading, page);
    }
    assert_eq!(prior.len(), 1, "앞 표는 누락·중복 없이 한 번 표시");
    assert_eq!(
        successor.len(),
        1,
        "실행계획 표는 누락·중복 없이 한 번 표시"
    );
    assert_eq!(heading.len(), 1, "후속 제목 표는 누락·중복 없이 한 번 표시");
    assert_eq!(
        prior[0].0 + 1,
        successor[0].0,
        "실행계획 표는 앞 표의 다음 쪽에 배치"
    );
    assert_eq!(
        successor[0].0, heading[0].0,
        "실행계획과 후속 제목은 같은 쪽에 배치"
    );
    assert!(
        successor[0].1 + successor[0].2 <= heading[0].1,
        "표 다음에 제목이 겹치지 않고 이어짐"
    );
}
