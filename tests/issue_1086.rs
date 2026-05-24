//! Issue #1086: k-water-rfp.hwp RowBreak/rowspan over-split regression.
//!
//! The Hancom 2022 PDF oracle for `samples/k-water-rfp.hwp` has 27 pages.

use std::fs;
use std::path::Path;

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

fn page_dump(rel_path: &str, page_idx: u32) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.dump_page_items(Some(page_idx))
}

#[test]
fn task1086_k_water_rfp_page_count_matches_hancom_pdf() {
    assert_eq!(page_count("samples/k-water-rfp.hwp"), 27);
}

#[test]
fn task1086_hwpspec_page_count_matches_hancom_office() {
    assert_eq!(page_count("samples/hwpspec.hwp"), 178);
}

#[test]
fn task1086_hwpspec_extended_control_figure_starts_on_next_page() {
    let page20 = page_dump("samples/hwpspec.hwp", 19);
    assert!(page20.contains("pi=88"));
    assert!(
        !page20.contains("pi=89"),
        "extended-control figure paragraph must not remain on page 20"
    );

    let page21 = page_dump("samples/hwpspec.hwp", 20);
    assert!(page21.contains("pi=89"));
}
