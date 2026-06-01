//! Issue #1142: external image reference discovery contract.
//!
//! The discovery API must expose renderer-consumer preparation data without
//! depending on bytes injection (#1143). In particular, loaded detection must
//! follow the renderer's bin_data_id index-first lookup semantics rather than
//! assuming `BinDataContent.id == bin_data_id`.

use rhwp::model::bin_data::BinDataContent;
use rhwp::wasm_api::HwpDocument;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalImageReference {
    key: String,
    bin_data_id: u16,
    original_path: String,
    basename: String,
    extension: String,
    loaded: bool,
}

fn load_doc(rel_path: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn external_image_refs(doc: &HwpDocument) -> Vec<ExternalImageReference> {
    let json = doc.get_external_image_references();
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse refs JSON {json}: {e}"))
}

fn external_image_basenames(doc: &HwpDocument) -> Vec<String> {
    let json = doc.get_external_image_basenames();
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse basename JSON {json}: {e}"))
}

fn assert_sample10_external_refs(rel_path: &str) {
    let doc = load_doc(rel_path);
    let refs = external_image_refs(&doc);

    assert_eq!(
        refs.len(),
        3,
        "{rel_path} should expose three external image refs"
    );

    let expected = [
        (1, "binData:1", "oracle.gif", "gif"),
        (2, "binData:2", "rdb02.gif", "gif"),
        (3, "binData:3", "s1.jpg", "jpg"),
    ];

    for (reference, (bin_id, key, basename, extension)) in refs.iter().zip(expected) {
        assert_eq!(reference.bin_data_id, bin_id, "{rel_path}: binDataId");
        assert_eq!(reference.key, key, "{rel_path}: key");
        assert_eq!(reference.basename, basename, "{rel_path}: basename");
        assert_eq!(reference.extension, extension, "{rel_path}: extension");
        assert!(
            reference.original_path.ends_with(basename),
            "{rel_path}: original path should preserve source path and end with {basename}. got={}",
            reference.original_path,
        );
        assert!(
            !reference.loaded,
            "{rel_path}: external reference should be missing before injection"
        );
    }
}

#[test]
fn issue_1142_hwp3_external_image_references_are_structured() {
    assert_sample10_external_refs("samples/hwp3-sample10.hwp");
}

#[test]
fn issue_1142_hwp5_link_external_image_references_are_structured() {
    assert_sample10_external_refs("samples/hwp3-sample10-hwp5.hwp");
}

#[test]
fn issue_1142_hwpx_external_image_references_are_structured() {
    assert_sample10_external_refs("samples/hwp3-sample10-hwpx.hwpx");
}

#[test]
fn issue_1142_basename_api_keeps_existing_json_array_contract() {
    let doc = load_doc("samples/hwp3-sample10.hwp");
    let basenames = external_image_basenames(&doc);

    assert_eq!(
        basenames,
        vec![
            "oracle.gif".to_string(),
            "rdb02.gif".to_string(),
            "s1.jpg".to_string()
        ],
        "legacy basename API should keep deterministic string-array output"
    );
}

#[test]
fn issue_1142_loaded_detection_uses_bin_data_index_before_storage_id() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    doc.document_mut().bin_data_content[0] = BinDataContent {
        id: 99,
        data: vec![b'G', b'I', b'F'],
        extension: "gif".to_string(),
    };

    let refs = external_image_refs(&doc);
    let oracle = refs
        .iter()
        .find(|reference| reference.bin_data_id == 1)
        .expect("binDataId 1 ref");
    assert!(
        oracle.loaded,
        "loaded must follow bin_data_content[binDataId - 1], even when storage id differs"
    );

    let basenames = external_image_basenames(&doc);
    assert!(
        !basenames.iter().any(|name| name == "oracle.gif"),
        "legacy basename API must hide references loaded through index-first lookup"
    );
    assert!(
        basenames.iter().any(|name| name == "rdb02.gif")
            && basenames.iter().any(|name| name == "s1.jpg"),
        "only still-missing references should remain in basename API"
    );
}
