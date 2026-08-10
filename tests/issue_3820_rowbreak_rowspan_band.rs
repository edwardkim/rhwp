//! Issue #3820 — RowBreak rowspan band continuation geometry.
//!
//! Hancom 2024 PDF for `76076_regulatory_analysis.hwp` renders the short
//! `주요내용` row at the bottom of p35, then retains only that row's blank
//! physical tail above p36's `11.영향평가 여부`.  Page count alone cannot detect
//! this: moving the whole row to p36 keeps the document at 82 pages while
//! changing both page images.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/76076_regulatory_analysis.hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("parse 76076 authority fixture")
}

fn text_y(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| text_y(child, needle))
}

fn contains_text(node: &RenderNode, needle: &str) -> bool {
    text_y(node, needle).is_some()
}

fn table_count(node: &RenderNode) -> usize {
    usize::from(matches!(&node.node_type, RenderNodeType::Table(_)))
        + node.children.iter().map(table_count).sum::<usize>()
}

fn owned_table_bottom(node: &RenderNode, para_index: usize, control_index: usize) -> Option<f64> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) && table.control_index == Some(control_index) {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children
        .iter()
        .find_map(|child| owned_table_bottom(child, para_index, control_index))
}

fn owned_table<'a>(
    node: &'a RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<&'a RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table)
            if table.para_index == Some(para_index) && table.control_index == Some(control_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| owned_table(child, para_index, control_index))
}

fn all_text(node: &RenderNode) -> String {
    let own = if let RenderNodeType::TextRun(run) = &node.node_type {
        run.text.as_str()
    } else {
        ""
    };
    let mut text = String::from(own);
    for child in &node.children {
        text.push_str(&all_text(child));
    }
    text
}

fn row_cell_texts(table: &RenderNode, row: u16) -> Vec<String> {
    table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TableCell(cell) if cell.row == row => Some(all_text(child)),
            _ => None,
        })
        .collect()
}

fn row_col_cell_text(table: &RenderNode, row: u16, col: u16) -> String {
    table
        .children
        .iter()
        .find_map(|child| match &child.node_type {
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col => {
                Some(all_text(child))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing table cell row={row}, col={col}"))
}

#[test]
fn issue_3820_rowbreak_rowspan_band_keeps_pdf_page_35_36_boundary() {
    let core = core();

    // Hancom PDF p35: `주요내용` is still painted at y≈748pt (≈997px CSS).
    let p35 = core.build_page_render_tree(34).expect("render HWP PDF p35");
    let summary_y = text_y(&p35.root, "주요내용")
        .expect("p35 must retain the `주요내용` content before the page boundary");
    assert!(
        (995.0..=1001.0).contains(&summary_y),
        "p35 `주요내용` y={summary_y:.1}px; PDF-aligned row band must remain at the footer"
    );
    let p35_table_bottom =
        owned_table_bottom(&p35.root, 347, 0).expect("p35 must retain the outer RowBreak table");
    assert!(
        (1040.0..=1046.0).contains(&p35_table_bottom),
        "p35 outer table bottom={p35_table_bottom:.1}px; the PDF retains the RowBreak blank tail through ≈1043px"
    );
    assert!(
        contains_text(
            &p35.root,
            "최근 빵 등 식품을 제조하는 사업장에서 밀가루 등이 반죽된"
        ),
        "p35 한양중고딕 본문은 PDF처럼 `…반죽된`에서 줄바꿈해야 함"
    );
    assert!(
        !contains_text(&p35.root, "밀가루 등이 반죽된 용"),
        "p35 한양중고딕의 과소 공백폭으로 `용`이 앞줄에 남으면 안 됨"
    );
    assert!(
        contains_text(
            &p35.root,
            "용기를 들어 올려 부어주는 기계(이하 “볼 리프트”라 한다) 인근에"
        ),
        "p35 다음 줄은 PDF처럼 `용기를 … 인근에`로 끝나야 함"
    );

    // Hancom PDF p36 has the blank tail of that row, but not its text; the
    // next visible row (`11.영향평가 여부`) begins only after that tail at y≈108px.
    let p36 = core.build_page_render_tree(35).expect("render HWP PDF p36");
    assert!(
        text_y(&p36.root, "주요내용").is_none(),
        "p36 must not repaint p35-owned `주요내용` text"
    );
    let impact_y = text_y(&p36.root, "영향평가").expect("p36 must resume at `11.영향평가 여부`");
    assert!(
        (103.0..=113.0).contains(&impact_y),
        "p36 `11.영향평가` y={impact_y:.1}px; blank rowspan tail was lost or overgrown"
    );
}

#[test]
fn issue_3820_p18_p19_keeps_short_rowspan_result_with_its_pdf_owner() {
    let core = core();

    // Hancom 2024 PDF keeps row 14 (`해당 없음` ×3) and the second line of the
    // row-spanning label (`여부`) at physical p19.  Stage 76's blank-tail repair
    // must not split this 32px row merely because 21.8px remain at p18.
    let p18 = core.build_page_render_tree(17).expect("render HWP PDF p18");
    let p18_table = owned_table(&p18.root, 173, 0).expect("p18 outer RowBreak table");
    assert!(
        row_cell_texts(p18_table, 14)
            .iter()
            .all(|text| !text.contains("해당 없음")),
        "p18 must defer row 14 results instead of keeping a short pseudo-tail: {:?}",
        row_cell_texts(p18_table, 14),
    );
    let p18_label = row_col_cell_text(p18_table, 13, 1);
    assert!(
        p18_label.contains("11.영향평가") && !p18_label.contains("여부"),
        "p18 must own only the first stored rowspan paragraph: {p18_label:?}",
    );

    let p19 = core.build_page_render_tree(18).expect("render HWP PDF p19");
    let p19_table = owned_table(&p19.root, 173, 0).expect("p19 outer RowBreak table");
    let result_cells = row_cell_texts(p19_table, 14);
    assert_eq!(
        result_cells
            .iter()
            .filter(|text| text.contains("해당 없음"))
            .count(),
        3,
        "p19 must own all three row 14 results: {result_cells:?}",
    );
    let p19_label = row_col_cell_text(p19_table, 13, 1);
    assert!(
        p19_label.contains("여부") && !p19_label.contains("11.영향평가"),
        "p19 must own only the rowspan label tail that Hancom places above the results: {p19_label:?}",
    );
}

#[test]
fn issue_3820_p35_renders_control_only_nested_table_without_line_seg() {
    let p35 = core()
        .build_page_render_tree(34)
        .expect("render HWP PDF p35");

    // p35 row 6 column 2 has a second, control-only paragraph whose HWP5
    // LINE_SEG is absent. Hancom nevertheless paints the 2×3 nested table.
    // The outer table plus this child table must both exist in RHWP's tree.
    assert!(
        contains_text(&p35.root, "인원수 또는 규모"),
        "p35 control-only nested-table header disappeared before SVG paint"
    );
    assert!(
        contains_text(&p35.root, "피규제자") && contains_text(&p35.root, "200"),
        "p35 control-only nested-table body disappeared before SVG paint"
    );
    assert!(
        table_count(&p35.root) >= 2,
        "p35 must retain the nested Table node below the outer RowBreak table"
    );
}
