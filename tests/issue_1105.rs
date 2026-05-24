//! Issue #1105: HWP3-origin HWP5 conversion keeps Hancom page break around sample16 p21.

use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

#[test]
fn task1105_sample16_hwp5_page_break_before_section_4_matches_hancom() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    assert_eq!(doc.page_count(), 64);

    let page21 = doc.dump_page_items(Some(20));
    assert!(page21.contains("FullParagraph  pi=439"));
    assert!(
        !page21.contains("pi=440"),
        "section 4 heading must not remain at the end of page 21:\n{page21}"
    );

    let page22 = doc.dump_page_items(Some(21));
    assert!(page22.contains("FullParagraph  pi=440"));
    assert!(page22.contains("Table          pi=441"));
    assert!(page22.contains("FullParagraph  pi=449"));
}
