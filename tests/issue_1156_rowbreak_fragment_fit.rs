//! RowBreak 표의 행 내부 조각 때문에 전체 조각이 쪽 경계를 넘으면
//! 그 조각을 현재 쪽에 남기지 않는다.
//!
//! `samples/kps-ai.hwp` 37쪽의 329문단은 32행 2열 RowBreak 표다.
//! 첫 조각은 0~15행까지 수용하지만 16행 일부까지 남기면 본문을 넘으므로
//! 16행은 38쪽으로 이월한다.

use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn page_dump(rel_path: &str, page_idx: u32) -> String {
    let doc = load_doc(rel_path);
    doc.dump_page_items(Some(page_idx))
}

#[test]
fn kps_ai_page37_defers_overflowing_split_row_slice() {
    let sample = "samples/kps-ai.hwp";

    let page37 = page_dump(sample, 36);
    assert!(
        page37.contains("PartialTable   pi=329 ci=0  rows=0..16"),
        "page 37 should end at the last fully fitting row:\n{page37}"
    );
    assert!(
        !page37.contains("end_cut="),
        "page 37 must not keep an overflowing row slice:\n{page37}"
    );

    let page38 = page_dump(sample, 37);
    assert!(
        page38.contains("PartialTable   pi=329 ci=0  rows=16..32  cont=true"),
        "page 38 should continue from row 16:\n{page38}"
    );
}

#[test]
fn synam_001_page14_uses_row_budget_after_repeated_header() {
    let sample = "samples/synam-001.hwp";

    let page13 = page_dump(sample, 12);
    assert!(
        page13.contains("PartialTable   pi=140 ci=0  rows=2..7"),
        "page 13 should include the first visible slice of row 6 after repeated header rows:\n{page13}"
    );
    assert!(
        page13.contains("start_cut=[2, 19] end_cut=[2, 13]"),
        "page 13 should continue row 2 and cut into row 6 instead of deferring row 6:\n{page13}"
    );

    let page14 = page_dump(sample, 13);
    assert!(
        page14.contains("start_cut=[2, 13]"),
        "page 14 should continue from the row 6 cut already started on page 13:\n{page14}"
    );
}
