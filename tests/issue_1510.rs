//! Issue #1510: visible text 가 있는 한 문단에 co-anchored para-relative
//! TopAndBottom floating 표가 여러 개 있을 때, vertical_offset 정렬/누적으로
//! 페이지가 늘어나거나 표 순서가 뒤집히는 회귀를 막는다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const HWP_SAMPLE: &str = "samples/issue1510_coanchored_float_tables.hwp";
const HWPX_SAMPLE: &str = "samples/issue1510_coanchored_float_tables.hwpx";
const TARGET_PI: usize = 0;
const TARGET_TABLES: [usize; 3] = [2, 3, 4];

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn collect_table_order(root: &RenderNode, out: &mut Vec<usize>) {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) {
            if let Some(ci) = table.control_index {
                if TARGET_TABLES.contains(&ci) {
                    out.push(ci);
                }
            }
        }
    }
    for child in &root.children {
        collect_table_order(child, out);
    }
}

fn find_table_bbox(root: &RenderNode, target_ci: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) && table.control_index == Some(target_ci) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_table_bbox(child, target_ci) {
            return Some(found);
        }
    }
    None
}

fn find_text_bbox(root: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        if run.para_index.is_some() && run.text == needle {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_text_bbox(child, needle) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_1510_coanchored_visible_para_float_tables_stay_on_one_page() {
    let doc = load_doc(HWP_SAMPLE);

    assert_eq!(
        doc.page_count(),
        1,
        "{} should match the Hancom 2024 HWP PDF baseline as a one-page document",
        HWP_SAMPLE,
    );
}

#[test]
fn issue_1510_visible_para_float_tables_apply_offsets_without_text_overlap() {
    let doc = load_doc(HWP_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (a_top, a_bottom) = find_table_bbox(&tree.root, 2).expect("A table bbox");
    let (b_top, _) = find_table_bbox(&tree.root, 3).expect("B table bbox");
    let (c_top, _) = find_table_bbox(&tree.root, 4).expect("C table bbox");
    let (_, filler_07_bottom) =
        find_text_bbox(&tree.root, "filler paragraph 07").expect("filler 07 bbox");
    let (filler_08_top, _) =
        find_text_bbox(&tree.root, "filler paragraph 08").expect("filler 08 bbox");

    assert!(
        b_top + 0.5 < c_top,
        "negative vertical_offset table should render above the zero-offset sibling: b_top={b_top:.1}, c_top={c_top:.1}",
    );
    assert!(
        filler_07_bottom <= a_top + 12.0,
        "text before the positive-offset table should remain above the table zone: filler07_bottom={filler_07_bottom:.1}, a_top={a_top:.1}",
    );
    assert!(
        filler_08_top >= a_bottom - 0.5,
        "text after reaching the positive-offset table should resume below it: filler08_top={filler_08_top:.1}, a_bottom={a_bottom:.1}",
    );
}

#[test]
fn issue_1510_visible_para_float_tables_keep_document_order() {
    let doc = load_doc(HWP_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let mut order = Vec::new();
    collect_table_order(&tree.root, &mut order);

    assert_eq!(
        order, TARGET_TABLES,
        "co-anchored visible-host float tables should retain document/control order",
    );
}

#[test]
fn issue_1510_hwpx_unsigned_negative_offset_and_visible_flow_match_two_page_baseline() {
    let doc = load_doc(HWPX_SAMPLE);

    assert_eq!(
        doc.page_count(),
        2,
        "{} should match the Hancom 2024 HWPX PDF baseline as a two-page document",
        HWPX_SAMPLE,
    );

    let page0 = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let page1 = doc
        .build_page_render_tree(1)
        .expect("build_page_render_tree(1)");

    let (_, b_bottom) = find_table_bbox(&page0.root, 3).expect("B table bbox");
    let (c_top, _) = find_table_bbox(&page0.root, 4).expect("C table bbox");
    assert!(
        c_top + 0.5 >= b_bottom,
        "HWPX visible-host non-positive float siblings should stack vertically: b_bottom={b_bottom:.1}, c_top={c_top:.1}",
    );

    assert!(
        find_text_bbox(&page0.root, "filler paragraph 29").is_some(),
        "filler 29 should remain on page 1",
    );
    assert!(
        find_text_bbox(&page0.root, "filler paragraph 30").is_none(),
        "filler 30 should move to page 2",
    );
    assert!(
        find_text_bbox(&page1.root, "filler paragraph 30").is_some()
            && find_text_bbox(&page1.root, "filler paragraph 32").is_some(),
        "page 2 should contain filler 30..32",
    );
}
