//! Issue #1417: whitespace-only TAC table host paragraphs must not advance
//! the pagination cursor twice.
//!
//! Regression shape:
//! - `pi=16` contains one space plus a TAC table.
//! - The table is emitted as `PageItem::Table`.
//! - The whitespace-only host line was also emitted as `PartialParagraph`,
//!   adding another line advance to the cursor.
//! - The accumulated drift pushed the `pi=27` TAC image group from page 2 to
//!   page 3 even though the rendered placement model left enough room.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwpx/pagenation-001.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1417_tac_image_group_stays_on_page_2() {
    let doc = load_doc();
    let page2 = doc.dump_page_items(Some(1));

    assert!(
        page2.contains("Table          pi=26"),
        "baseline table before the target image group should remain on page 2\n--- page 2 ---\n{}",
        page2
    );
    assert!(
        page2.contains("Shape          pi=27"),
        "pi=27 TAC image group should fit on page 2\n--- page 2 ---\n{}",
        page2
    );
    assert!(
        !page2.contains("PartialParagraph  pi=16"),
        "whitespace-only pi=16 TAC table host must not emit a duplicate post-text paragraph\n--- page 2 ---\n{}",
        page2
    );
}
