//! 검사 유형: 관계형·고정형(독립 한컴 Print의 전체 쪽수).
//! 주석 갱신: 2026-11-24T13:25:13+09:00.
//! 측정일: 2026-11-24 KST. 13pt 예제 3쪽 최저 Native 100.00%, fresh WASM 100.00%.
//! 10pt 예제 2쪽 최저 Native 100.00%, fresh WASM 100.00%.
//! 측정 방법: 96 DPI·PDFium·색상 경계 보정 전 2px 실루엣.
//! 전쪽 TSV·대표 대조 이미지·출력 영수증: mydocs/pr/assets/stored-leading-float-page-break/.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const RESTART: &[u8] =
    include_bytes!("../fixtures/stored_leading_float/stored-negative-title-anchor.hwp");
const FITS: &[u8] =
    include_bytes!("../fixtures/stored_leading_float/stored-negative-title-fits.hwp");

fn collect_page<'a>(
    node: &'a RenderNode,
    labels: &mut Vec<(&'a str, &'a RenderNode)>,
    tables: &mut Vec<&'a RenderNode>,
) {
    match &node.node_type {
        RenderNodeType::TextRun(run) => {
            let text = run.display_or_text().trim();
            if !text.is_empty() {
                labels.push((text, node));
            }
        }
        RenderNodeType::Table(_) => tables.push(node),
        _ => {}
    }
    for child in &node.children {
        collect_page(child, labels, tables);
    }
}

fn assert_print_order(input: &[u8], expected: &[&[&str]]) {
    let document = DocumentCore::from_bytes(input).unwrap();
    assert_eq!(document.page_count(), expected.len() as u32);
    for (page, labels) in expected.iter().enumerate() {
        let tree = document.build_page_render_tree(page as u32).unwrap();
        let mut actual = Vec::new();
        let mut tables = Vec::new();
        collect_page(&tree.root, &mut actual, &mut tables);
        actual.sort_by(|a, b| a.1.bbox.y.total_cmp(&b.1.bbox.y));
        assert_eq!(
            actual.iter().map(|(text, _)| *text).collect::<Vec<_>>(),
            *labels
        );

        let table_count = labels.iter().filter(|label| **label != "Title").count();
        assert_eq!(tables.len(), table_count);
        if let [table] = tables.as_slice() {
            let label = actual.last().unwrap().1;
            assert!(table.bbox.y <= label.bbox.y);
            assert!(label.bbox.y + label.bbox.height <= table.bbox.y + table.bbox.height);
            if let [("Title", title), ..] = actual.as_slice() {
                assert!(title.bbox.y + title.bbox.height <= table.bbox.y);
            }
        }
    }
}

#[test]
fn stored_leading_float_keeps_title_and_tables_on_the_print_pages() {
    assert_print_order(
        RESTART,
        &[&["Title"], &["Leading table"], &["Following table"]],
    );
    assert_print_order(FITS, &[&["Title", "Leading table"], &["Following table"]]);
}
