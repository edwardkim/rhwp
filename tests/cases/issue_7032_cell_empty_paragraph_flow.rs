//! #7032: a saved empty cell paragraph owns a line even without LINE_SEG.
//! The HWPX fixture and the maintainer's Hancom judgment are independent controls.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, table::Cell};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn core(hwpx: bool) -> DocumentCore {
    let file = if hwpx {
        "samples/hwpx/21761835_jeonjik_exemption_table.hwpx"
    } else {
        "samples/task2146/21761835_jeonjik_exemption_table.hwp"
    };
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file))
        .expect("required fixture");
    DocumentCore::from_bytes(&bytes).expect("fixture parse")
}

fn source_cell(core: &DocumentCore) -> &Cell {
    let Control::Table(table) = &core.document().sections[0].paragraphs[4].controls[0] else {
        panic!("fixture table pi=4 ci=0")
    };
    &table.cells[0]
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![root];
    for child in &root.children {
        result.extend(nodes(child));
    }
    result
}

fn header(root: &RenderNode) -> &RenderNode {
    let cells: Vec<_> = nodes(root)
        .into_iter()
        .filter(
            |n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 0 && c.col == 0),
        )
        .collect();
    assert_eq!(cells.len(), 1, "one repeated title cell per page");
    cells[0]
}

fn paragraph_lines(cell: &RenderNode) -> Vec<&RenderNode> {
    nodes(cell)
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .collect()
}

fn assert_near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.11, "{actual} != {expected}");
}

fn assert_header_flow(core: &DocumentCore) {
    let cell = source_cell(core);
    let para = &cell.paragraphs[0];
    let shape = &core.document().doc_info.para_shapes[para.para_shape_id as usize];
    assert_eq!(
        shape.line_spacing_type,
        rhwp::model::style::LineSpacingType::Fixed
    );
    // HWP fixed spacing is stored at twice the effective HWPUNIT advance.
    let advance = f64::from(shape.line_spacing) / 2.0 * 96.0 / 7200.0;
    assert_eq!(core.page_count(), 6);
    for page in 0..6 {
        let tree = core.build_page_render_tree(page).expect("page render tree");
        let cell = header(&tree.root);
        let lines = paragraph_lines(cell);
        assert_eq!(lines.len(), 2, "p{} empty paragraph must remain", page + 1);
        for (pi, line) in lines.iter().enumerate() {
            let RenderNodeType::TextLine(info) = &line.node_type else {
                unreachable!()
            };
            assert_eq!(info.para_index, Some(pi));
        }
        let empty_runs: Vec<_> = nodes(lines[0])
            .into_iter()
            .filter_map(|n| {
                if let RenderNodeType::TextRun(run) = &n.node_type {
                    Some(run)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(empty_runs.len(), 1);
        assert!(empty_runs[0].text.is_empty());
        assert!(empty_runs[0].is_para_end);
        assert!(empty_runs[0].cell_context.is_some());
        assert!(nodes(lines[1])
            .iter()
            .any(|n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text == "직렬")));
        assert_near(lines[1].bbox.y - lines[0].bbox.y, advance);
        assert_near(cell.bbox.height, 52.4);
    }
}

#[test]
fn hwp_retains_empty_line_in_all_repeated_headers() {
    assert_header_flow(&core(false));
}

#[test]
fn hwpx_stored_empty_line_remains_unchanged() {
    assert_header_flow(&core(true));
}

#[test]
fn both_parsers_preserve_the_same_two_paragraphs() {
    for hwpx in [false, true] {
        let core = core(hwpx);
        let cell = source_cell(&core);
        assert_eq!(cell.paragraphs.len(), 2);
        assert!(cell.paragraphs[0].text.is_empty());
        assert_eq!(cell.paragraphs[1].text.trim(), "직렬");
        for para in &cell.paragraphs {
            assert!(para.controls.is_empty());
            assert!(para.char_count > 0);
            assert_eq!(para.line_segs.len(), usize::from(hwpx));
        }
    }
}

#[test]
fn hwp_header_text_origin_matches_stored_hwpx_control() {
    let hwp = core(false);
    let hwpx = core(true);
    for page in 0..6 {
        let a = hwp.build_page_render_tree(page).unwrap();
        let b = hwpx.build_page_render_tree(page).unwrap();
        let a = paragraph_lines(header(&a.root));
        let b = paragraph_lines(header(&b.root));
        assert_eq!((a.len(), b.len()), (2, 2));
        for (a, b) in a.iter().zip(&b) {
            assert_near(a.bbox.y, b.bbox.y);
            // Fixed-spacing fallback and stored LINE_SEG may encode the line
            // box differently. Ownership and the next origin must agree, not
            // the internal height/spacing split of the two formats.
        }
    }
}
