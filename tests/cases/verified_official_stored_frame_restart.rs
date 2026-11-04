//! 검사 유형: 관계형·고정형.
//! 2026-10-28 측정 raw(96dpi, PDFium):
//! pr8036-seoul-official.hwpx: 11쪽 Native 99.99015% / fresh WASM 99.96975%;
//! 12쪽 Native 98.07866% / fresh WASM 98.10975%; 전13쪽 최저 95.97864% / 96.05079%.
//! pr8057-official.hwpx: 15쪽 Native 99.16179% / fresh WASM 99.14062%; 전37쪽 최저 95.46028% / 95.38098%.
//! 근거: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/floatfix-native-matrix.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/floatfix-wasm-matrix.json.
//! pr8036 독립 Print 경계: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/pr8036-pdf-boundaries.json.
//!
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
    let prior_bottom = prior[0].1 + prior[0].2;
    let heading_top = heading[0].1;
    let heading_bottom = heading_top + heading[0].2;
    assert!(
        (prior_bottom - 1021.28).abs() <= 2.0,
        "독립 Print 11쪽 앞 표 아랫변: {prior_bottom:.2}px"
    );
    assert!(
        (heading_top - 203.94).abs() <= 2.0,
        "독립 Print 12쪽 제목 표 윗변: {heading_top:.2}px"
    );
    assert!(
        (heading_bottom - 244.53).abs() <= 2.0,
        "독립 Print 12쪽 제목 표 아랫변: {heading_bottom:.2}px"
    );
}

#[test]
fn inline_no_adjust_rowspan_table_keeps_printed_row_boundaries() {
    let input = include_bytes!(
        "../../mydocs/pr/assets/pr8319_evidence_alignment_20261027/official-inputs/pr8057-official.hwpx"
    );
    let core = DocumentCore::from_bytes(input).expect("공식 HWPX");
    let page = core.build_page_render_tree(14).expect("독립 Print 15쪽");
    fn table(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(0) && t.para_index == Some(178)
                && t.control_index == Some(0) && t.cell_context.is_none())
        {
            return Some(node);
        }
        node.children.iter().find_map(table)
    }
    let table = table(&page.root).expect("15쪽 글자처럼 취급 표");
    for (row, expected_y) in [(13, 755.49), (19, 912.60)] {
        let cell = table.children.iter().find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == row && c.col == 3)
        }).expect("행의 대표 셀");
        assert!(
            (cell.bbox.y - expected_y).abs() <= 2.0,
            "행 {row}의 독립 Print 경계: {} vs {expected_y}",
            cell.bbox.y
        );
        assert!(
            (cell.bbox.height - 26.2).abs() <= 0.2,
            "행 {row}의 저장 높이: {}",
            cell.bbox.height
        );
    }
}
