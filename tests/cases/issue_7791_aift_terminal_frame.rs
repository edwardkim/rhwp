//! 관계형 — #7791 표 뒤 저장 빈 줄과 다음 제목의 위치 및 순서.
//! 주석 갱신 2026-11-28T04:50:06+09:00. 직전 파일 커밋 1c5b9928b96c1aba5654d62029a9f9aabd75efba 2026-11-28T03:38:16+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 1e773b0314979e9198435eb4a6249e00430c6f58.
//! aift · 기준 mydocs/pr/assets/issue7445/aift74/aift-hwp-2020.pdf · 전체 74쪽.
//! 전쪽 최저 Native 98.48% / fresh WASM 98.49%.
//! 45쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/aift/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn visit<'a>(node: &'a RenderNode, results: &mut Vec<&'a RenderNode>) {
    results.push(node);
    for child in &node.children {
        visit(child, results);
    }
}

#[test]
fn aift_completed_table_ends_before_its_saved_blank_line_and_heading() {
    let bytes = std::fs::read("mydocs/pr/assets/issue7445/aift74/aift.hwp").unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let [table_source, blank_source, heading_source] =
        &core.document().sections[2].paragraphs[584..=586]
    else {
        panic!("original table and its successors");
    };
    let [Control::Table(table)] = table_source.controls.as_slice() else {
        panic!("table at the recorded source address");
    };
    assert!(blank_source.text.is_empty());
    assert!(heading_source.text.contains("사회적 가치 창출 계획"));
    let tree = core.build_page_render_tree(44).unwrap();
    let mut nodes = Vec::new();
    visit(&tree.root, &mut nodes);
    let find_line = |index| {
        let matching = nodes
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.section_index == Some(2) && line.para_index == Some(index))
            })
            .collect::<Vec<_>>();
        let [line] = matching.as_slice() else {
            panic!(
                "expected a single body line for paragraph {index}, found {}",
                matching.len()
            );
        };
        line.bbox
    };
    let matching_tables = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.section_index == Some(2)
                && table.para_index == Some(584)
                && table.control_index == Some(0)
                && table.cell_context.is_none())
        })
        .collect::<Vec<_>>();
    let [table_node] = matching_tables.as_slice() else {
        panic!("expected the whole table on the heading's page");
    };

    let blank = find_line(585);
    let heading = find_line(586);
    let dpi = rhwp::renderer::DEFAULT_DPI;
    let outer_margin = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let blank_after_table = blank.y - (table_node.bbox.y + table_node.bbox.height);
    assert!(
        (0.0..=outer_margin).contains(&blank_after_table),
        "the saved blank line starts in the table's trailing margin once, gap={blank_after_table}, margin={outer_margin}"
    );
    let source_advance = hwpunit_to_px(
        heading_source.line_segs[0].vertical_pos - blank_source.line_segs[0].vertical_pos,
        dpi,
    );
    let rounding = hwpunit_to_px(2, dpi);
    assert!(blank.y + blank.height < heading.y);
    assert!(((heading.y - blank.y) - source_advance).abs() <= rounding);
}
