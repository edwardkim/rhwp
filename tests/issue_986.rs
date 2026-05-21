//! Issue #986: landscape receipt document with multiple non-TAC para-relative
//! TopAndBottom tables in one empty host paragraph.
//!
//! Regression shape:
//! - right-side tables were pushed below the large left table because pagination
//!   and layout advanced one global vertical cursor without considering
//!   horizontally independent float lanes.
//! - `ci=4` and `ci=6` were split as `PartialTable`, inflating the document from
//!   the expected one-page visual shape to three pages.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue-986-receipt.hwp";
const TARGET_PI: usize = 0;

#[derive(Debug, Clone, Copy)]
struct TableBBox {
    page: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl TableBBox {
    fn right(self) -> f64 {
        self.x + self.width
    }

    fn bottom(self) -> f64 {
        self.y + self.height
    }
}

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

fn collect_table_bboxes(
    root: &RenderNode,
    page: u32,
    target_pi: usize,
    target_ci: usize,
    out: &mut Vec<TableBBox>,
) {
    if let RenderNodeType::Table(t) = &root.node_type {
        if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
            out.push(TableBBox {
                page,
                x: root.bbox.x,
                y: root.bbox.y,
                width: root.bbox.width,
                height: root.bbox.height,
            });
        }
    }
    for child in &root.children {
        collect_table_bboxes(child, page, target_pi, target_ci, out);
    }
}

fn table_bboxes(doc: &rhwp::wasm_api::HwpDocument, control_index: usize) -> Vec<TableBBox> {
    let mut out = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("build_page_render_tree page {}: {}", page, e));
        collect_table_bboxes(&tree.root, page, TARGET_PI, control_index, &mut out);
    }
    out
}

fn first_page_table(doc: &rhwp::wasm_api::HwpDocument, control_index: usize) -> TableBBox {
    table_bboxes(doc, control_index)
        .into_iter()
        .find(|bbox| bbox.page == 0)
        .unwrap_or_else(|| panic!("pi=0 ci={} Table node missing on page 1", control_index))
}

#[test]
fn issue_986_receipt_tables_do_not_split_to_later_pages() {
    let doc = load_doc();
    let page_count = doc.page_count();

    assert_eq!(
        page_count, 1,
        "{} should fit on one page; actual page_count={}",
        SAMPLE, page_count,
    );

    for ci in 2..=8 {
        let boxes = table_bboxes(&doc, ci);
        assert!(!boxes.is_empty(), "pi=0 ci={} Table node missing", ci);

        let pages: BTreeSet<u32> = boxes.iter().map(|bbox| bbox.page).collect();
        assert_eq!(
            pages,
            BTreeSet::from([0]),
            "pi=0 ci={} should be rendered only on page 1; pages={:?} indicates a split/regression",
            ci,
            pages,
        );
    }
}

#[test]
fn issue_986_receipt_right_tables_keep_independent_float_lanes() {
    let doc = load_doc();

    let left_top = first_page_table(&doc, 2);
    let middle_top = first_page_table(&doc, 4);
    let right_top = first_page_table(&doc, 6);

    eprintln!(
        "[issue_986] ci2=[x {:.1}..{:.1}, y {:.1}..{:.1}] ci4=[x {:.1}..{:.1}, y {:.1}..{:.1}] ci6=[x {:.1}..{:.1}, y {:.1}..{:.1}]",
        left_top.x,
        left_top.right(),
        left_top.y,
        left_top.bottom(),
        middle_top.x,
        middle_top.right(),
        middle_top.y,
        middle_top.bottom(),
        right_top.x,
        right_top.right(),
        right_top.y,
        right_top.bottom(),
    );

    assert!(
        middle_top.x >= left_top.right() - 5.0,
        "ci=4 should occupy a horizontal lane to the right of ci=2; ci2={:?}, ci4={:?}",
        left_top,
        middle_top,
    );
    assert!(
        right_top.x >= middle_top.right() - 5.0,
        "ci=6 should occupy a horizontal lane to the right of ci=4; ci4={:?}, ci6={:?}",
        middle_top,
        right_top,
    );

    assert!(
        middle_top.y <= left_top.y + 5.0,
        "ci=4 should start with the left table lane, not below ci=2; ci2={:?}, ci4={:?}",
        left_top,
        middle_top,
    );
    assert!(
        right_top.y <= left_top.y + 5.0,
        "ci=6 should start with the left table lane, not below ci=2; ci2={:?}, ci6={:?}",
        left_top,
        right_top,
    );
}
