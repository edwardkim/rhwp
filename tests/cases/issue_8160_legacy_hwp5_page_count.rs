//! #8160: sample16 전용 회귀. 이전 등록: tests/issue_1086.rs
use std::{fs, path::Path};

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

/// 독립 한컴 PDF와 전64쪽 Native/fresh 시각 검증의 같은 원본이다.
#[test]
fn issue_8160_hwp3_sample16_hwp5_page_count_matches_hancom_office() {
    assert_eq!(page_count("samples/hwp3-sample16-hwp5.hwp"), 64);
}
