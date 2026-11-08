//! 관계형·고정형: 실물 표 첫 행의 원본 셀 소유와 높이를 검사한다.
//! 주석 갱신: 2026-10-30T05:51:30+09:00; 직전 파일 커밋: 2026-07-26T19:44:24+09:00 (2e0cd528c5).
//! 2026-10-30 측정 raw(96dpi, PDFium, 2px): 2쪽 Native 99.64% / fresh WASM 99.65%.
//! 전 20쪽 최저 Native 96.43% / fresh WASM 96.42%; 리베이스 직후 렌더 소스의 측정값이다.
//! 근거: mydocs/pr/assets/issue_501/rebase-20261030/README.md.
//!
//! #501: 글자처럼 취급하는 표의 행 높이가 원본보다 비례 축소되지 않아야 한다.
//! SVG 부동소수점 직렬화가 아니라 원본 셀과 실제 출력의 소유 관계를 검사한다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::path::Path;

fn find_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.section_index == Some(0) && table.para_index == Some(22) && table.control_index == Some(0))
    {
        return Some(node);
    }
    node.children.iter().find_map(find_table)
}

#[test]
fn mel001_pi22_row0_height_matches_ir() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/mel-001.hwp");
    let document =
        HwpDocument::from_bytes(&std::fs::read(path).expect("원본 HWP")).expect("원본 문서 해석");
    assert_eq!(document.page_count(), 20, "독립 Print의 전체 쪽수");
    let source = match &document.document().sections[0].paragraphs[22].controls[0] {
        Control::Table(table) => table,
        _ => panic!("원본 22번 문단의 표"),
    };
    let source_cells: Vec<_> = source.cells.iter().filter(|cell| cell.row == 0).collect();
    let page = document.build_page_render_tree(1).expect("실물 2쪽");
    let table = find_table(&page.root).expect("2쪽의 원본 표 소유");
    let actual_cells: Vec<_> = table
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) if cell.row == 0 => Some((cell, node)),
            _ => None,
        })
        .collect();
    assert_eq!(
        actual_cells.len(),
        source_cells.len(),
        "첫 행의 셀 누락·중복 없음"
    );
    let first_y = actual_cells.first().expect("첫 행 셀").1.bbox.y;
    for source_cell in source_cells {
        let matches: Vec<_> = actual_cells
            .iter()
            .filter(|(cell, _)| {
                cell.col == source_cell.col && cell.col_span == source_cell.col_span
            })
            .collect();
        assert_eq!(matches.len(), 1, "원본 열과 걸침의 유일한 소유");
        let node = matches[0].1;
        let source_height = f64::from(source_cell.height) * 96.0 / 7200.0;
        assert!(
            (node.bbox.y - first_y).abs() <= 1.0 / 75.0,
            "같은 행의 위쪽 경계"
        );
        assert!(
            (node.bbox.height - source_height).abs() <= 2.0,
            "원본 셀 높이 {source_height}px, 실제 {}px: 행의 비례 축소 방지",
            node.bbox.height
        );
    }
}
