//! A cell-format edit exported from rhwp retains the table's 283 HU outer top.
//! Hancom 2020's PDF of the same HWPX starts the border at y≈136 px at 96 dpi;
//! the page body starts at y≈132.3 px. The edited cell text grows to two lines
//! per paragraph, but that does not consume the table's outer margin.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue7265/pr7400_cell_char_format_after.hwpx";

fn top_of(node: &RenderNode, table: bool) -> Option<f64> {
    let selected = match &node.node_type {
        RenderNodeType::Table(_) => table,
        RenderNodeType::Body { .. } => !table,
        _ => false,
    };
    selected
        .then_some(node.bbox.y)
        .or_else(|| node.children.iter().find_map(|child| top_of(child, table)))
}

fn sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read HWPX")).expect("open HWPX")
}

#[test]
fn edited_cell_table_starts_after_outer_top_margin() {
    let core = sample();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).expect("page render tree");
    let body_y = top_of(&tree.root, false).expect("body");
    let table_y = top_of(&tree.root, true).expect("table");
    let outer_top_px = 283.0 / 75.0;
    assert!((body_y - 132.3).abs() < 0.5, "body y={body_y:.2}");
    assert!(
        (table_y - (body_y + outer_top_px)).abs() < 0.5,
        "table y={table_y:.2}, expected body + 283 HU = {:.2}",
        body_y + outer_top_px
    );
    assert!(
        (table_y - 136.0).abs() < 1.0,
        "Hancom PDF border y≈136: {table_y:.2}"
    );
}

#[test]
fn zero_outer_top_stays_at_body_start() {
    let mut core = sample();
    let table = core.document_mut().sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|p| &mut p.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("table");
    table.outer_margin_top = 0;
    let tree = core.build_page_render_tree(0).expect("page render tree");
    let body_y = top_of(&tree.root, false).expect("body");
    let table_y = top_of(&tree.root, true).expect("table");
    assert!(
        (table_y - body_y).abs() < 0.5,
        "zero top margin: {table_y:.2} vs {body_y:.2}"
    );
}
