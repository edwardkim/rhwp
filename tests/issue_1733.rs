//! Issue #1733: 국제고속선기준 tail/vpos-reset 잔여 over-pagination 회귀 방지.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const EXPECTED_PAGE_COUNT: u32 = 242;

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {}: {err:?}", path.display()))
}

fn assert_matches_pdf_page_count(sample: &str) {
    let doc = load_doc(sample);
    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGE_COUNT,
        "{sample} should match the HWP 2024/PDF oracle page count"
    );
}

#[test]
fn issue_1733_hwpx_matches_pdf_page_count() {
    assert_matches_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwpx");
}

#[test]
fn issue_1733_hwp_matches_pdf_page_count() {
    assert_matches_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwp");
}
