//! Issue #1143: external image bytes injection and PageLayerTree cache invalidation.
//!
//! Injection must share the renderer's binDataId index-first lookup semantics and
//! must invalidate cached page trees when image bytes become available after the
//! tree has already been built.

use rhwp::model::bin_data::BinDataContent;
use rhwp::wasm_api::HwpDocument;
use serde::Deserialize;
use serde_json::Value;
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

const SAMPLE10_EXTERNAL_FORMATS: &[(&str, &str)] = &[
    ("hwp3", "samples/hwp3-sample10.hwp"),
    ("hwp5", "samples/hwp3-sample10-hwp5.hwp"),
    ("hwpx", "samples/hwp3-sample10-hwpx.hwpx"),
];

const SAMPLE10_EXTERNAL_IMAGES: &[(u16, &str, &str, &str)] = &[
    (1, "oracle.gif", "gif", "samples/oracle.gif"),
    (2, "rdb02.gif", "gif", "samples/rdb02.gif"),
    (3, "s1.jpg", "jpg", "samples/s1.jpg"),
];

fn load_doc(rel_path: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn read_sample(rel_path: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn external_image_refs(doc: &HwpDocument) -> Vec<ExternalImageReference> {
    let json = doc.get_external_image_references();
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse refs JSON {json}: {e}"))
}

fn external_image_basenames(doc: &HwpDocument) -> Vec<String> {
    let json = doc.get_external_image_basenames();
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse basename JSON {json}: {e}"))
}

fn collect_image_ops<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if value.get("type").and_then(Value::as_str) == Some("image") {
                out.push(value);
            }
            for child in map.values() {
                collect_image_ops(child, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_image_ops(child, out);
            }
        }
        _ => {}
    }
}

fn page_layer_image_payload_count(doc: &HwpDocument, page: u32) -> usize {
    let json = doc
        .get_page_layer_tree_native(page)
        .unwrap_or_else(|e| panic!("page layer tree {page}: {e}"));
    let parsed: Value =
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("PageLayerTree JSON: {e}"));
    let mut images = Vec::new();
    collect_image_ops(&parsed, &mut images);
    images
        .iter()
        .filter(|image| {
            image.get("mime").is_some()
                && image
                    .get("base64")
                    .and_then(Value::as_str)
                    .is_some_and(|data| !data.is_empty())
        })
        .count()
}

fn canvaskit_image_items(doc: &HwpDocument, page: u32) -> Vec<Value> {
    let json = doc
        .get_canvaskit_replay_plan_native(page, "default")
        .unwrap_or_else(|e| panic!("CanvasKit replay plan {page}: {e}"));
    let parsed: Value =
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("CanvasKit plan JSON: {e}"));
    parsed["items"]
        .as_array()
        .expect("CanvasKit plan items")
        .iter()
        .filter(|item| item["opType"].as_str() == Some("image"))
        .cloned()
        .collect()
}

fn item_detail_contains(item: &Value, needle: &str) -> bool {
    item["detail"]
        .as_str()
        .is_some_and(|detail| detail.split(';').any(|part| part == needle))
}

fn expected_sample10_basenames_from(start: usize) -> Vec<String> {
    SAMPLE10_EXTERNAL_IMAGES[start..]
        .iter()
        .map(|(_, basename, _, _)| (*basename).to_string())
        .collect()
}

fn assert_sample10_external_refs(
    label: &str,
    refs: &[ExternalImageReference],
    loaded_through: usize,
) {
    assert_eq!(
        refs.len(),
        SAMPLE10_EXTERNAL_IMAGES.len(),
        "{label}: sample10 should expose three external image refs"
    );

    for (index, (reference, (bin_id, basename, extension, _sample_path))) in
        refs.iter().zip(SAMPLE10_EXTERNAL_IMAGES.iter()).enumerate()
    {
        assert_eq!(reference.bin_data_id, *bin_id, "{label}: binDataId");
        assert_eq!(reference.key, format!("binData:{bin_id}"), "{label}: key");
        assert_eq!(reference.basename, *basename, "{label}: basename");
        assert_eq!(reference.extension, *extension, "{label}: extension");
        assert!(
            reference.original_path.ends_with(basename),
            "{label}: original path should end with {basename}. got={}",
            reference.original_path
        );
        assert_eq!(
            reference.loaded,
            index < loaded_through,
            "{label}: loaded state for {basename}"
        );
    }
}

#[test]
fn issue_1143_basename_injection_marks_reference_loaded() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");
    let oracle = read_sample("samples/oracle.gif");

    let before = external_image_refs(&doc);
    let oracle_ref = before
        .iter()
        .find(|reference| reference.bin_data_id == 1)
        .expect("binDataId 1 ref");
    assert_eq!(oracle_ref.key, "binData:1");
    assert_eq!(oracle_ref.basename, "oracle.gif");
    assert_eq!(oracle_ref.extension, "gif");
    assert!(oracle_ref.original_path.ends_with("oracle.gif"));
    assert!(!oracle_ref.loaded);

    let injected = doc.inject_external_image("oracle.gif", &oracle, "/tmp/oracle.gif");
    assert_eq!(injected, 1);

    let after = external_image_refs(&doc);
    let oracle_ref = after
        .iter()
        .find(|reference| reference.bin_data_id == 1)
        .expect("binDataId 1 ref");
    assert!(oracle_ref.loaded);

    let basenames = external_image_basenames(&doc);
    assert!(
        !basenames.iter().any(|name| name == "oracle.gif"),
        "injected basename should be hidden from the legacy missing-image API"
    );
    assert!(
        basenames.iter().any(|name| name == "rdb02.gif")
            && basenames.iter().any(|name| name == "s1.jpg"),
        "still-missing image basenames should remain visible"
    );
}

#[test]
fn issue_1143_canvaskit_plan_distinguishes_missing_and_injected_external_images() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    let before_items = canvaskit_image_items(&doc, 0);
    assert!(
        before_items.iter().any(|item| {
            item["status"].as_str() == Some("directRequired")
                && item_detail_contains(item, "externalImage")
                && item_detail_contains(item, "missingImageData")
        }),
        "external image without injected bytes should be reported as missing: {before_items:?}"
    );

    let oracle_ref = external_image_refs(&doc)
        .into_iter()
        .find(|reference| reference.bin_data_id == 1)
        .expect("binDataId 1 ref");
    let oracle = read_sample("samples/oracle.gif");
    assert_eq!(
        doc.inject_external_image_by_key(&oracle_ref.key, &oracle, "/tmp/oracle.gif"),
        1
    );

    let after_items = canvaskit_image_items(&doc, 0);
    assert!(
        after_items.iter().any(|item| {
            item["status"].as_str() == Some("direct")
                && item_detail_contains(item, "externalImage")
                && item_detail_contains(item, "injectedImageData")
                && !item_detail_contains(item, "missingImageData")
        }),
        "injected external image should be directly replayable and not reported as missing: {after_items:?}"
    );
    assert!(
        after_items.iter().any(|item| {
            item["status"].as_str() == Some("directRequired")
                && item_detail_contains(item, "externalImage")
                && item_detail_contains(item, "missingImageData")
        }),
        "other external images on the page should remain missing until injected: {after_items:?}"
    );
}

#[test]
fn issue_1143_key_injection_uses_discovery_key_and_invalidates_cache() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    let oracle_ref = external_image_refs(&doc)
        .into_iter()
        .find(|reference| reference.bin_data_id == 1)
        .expect("binDataId 1 ref");
    assert_eq!(oracle_ref.key, "binData:1");

    let before_payloads = page_layer_image_payload_count(&doc, 0);
    assert_eq!(
        before_payloads, 0,
        "sample10 page 1 external images should have no payload before key injection"
    );

    let oracle = read_sample("samples/oracle.gif");
    assert_eq!(
        doc.inject_external_image_by_key(&oracle_ref.key, &oracle, "/tmp/oracle.gif"),
        1
    );
    assert_eq!(
        doc.inject_external_image_by_key(&oracle_ref.key, &oracle, "/tmp/oracle.gif"),
        0,
        "already-loaded key injection should be idempotent"
    );

    let refs = external_image_refs(&doc);
    assert!(
        refs.iter()
            .any(|reference| reference.bin_data_id == 1 && reference.loaded),
        "target reference should be marked loaded after key injection"
    );
    assert!(
        refs.iter()
            .any(|reference| reference.bin_data_id == 2 && !reference.loaded)
            && refs
                .iter()
                .any(|reference| reference.bin_data_id == 3 && !reference.loaded),
        "key injection should not load unrelated references"
    );

    let basenames = external_image_basenames(&doc);
    assert!(
        !basenames.iter().any(|name| name == "oracle.gif"),
        "key-injected basename should be hidden from the legacy missing-image API"
    );
    assert!(
        basenames.iter().any(|name| name == "rdb02.gif")
            && basenames.iter().any(|name| name == "s1.jpg"),
        "unrelated missing basenames should remain visible"
    );

    let after_payloads = page_layer_image_payload_count(&doc, 0);
    assert!(
        after_payloads > before_payloads,
        "key injection should invalidate cached PageLayerTrees"
    );
}

#[test]
fn issue_1143_key_injection_works_across_hwp3_hwp5_and_hwpx_sample10() {
    for (label, rel_path) in SAMPLE10_EXTERNAL_FORMATS {
        let mut doc = load_doc(rel_path);
        let refs = external_image_refs(&doc);
        assert_sample10_external_refs(label, &refs, 0);
        assert_eq!(
            external_image_basenames(&doc),
            expected_sample10_basenames_from(0),
            "{label}: basename API should list missing images before injection"
        );

        let before_payloads = page_layer_image_payload_count(&doc, 0);
        assert_eq!(
            before_payloads, 0,
            "{label}: sample10 page 1 external images should have no payload before injection"
        );

        let before_items = canvaskit_image_items(&doc, 0);
        assert!(
            before_items.iter().any(|item| {
                item["status"].as_str() == Some("directRequired")
                    && item_detail_contains(item, "externalImage")
                    && item_detail_contains(item, "missingImageData")
            }),
            "{label}: CanvasKit should report missing external image bytes before injection: {before_items:?}"
        );

        for (image_index, (bin_id, basename, _extension, sample_path)) in
            SAMPLE10_EXTERNAL_IMAGES.iter().enumerate()
        {
            let reference = refs
                .iter()
                .find(|reference| reference.bin_data_id == *bin_id)
                .unwrap_or_else(|| panic!("{label}: binDataId {bin_id} ref"));
            let data = read_sample(sample_path);
            let display_path = format!("/tmp/{basename}");
            assert_eq!(
                doc.inject_external_image_by_key(&reference.key, &data, &display_path),
                1,
                "{label}: key injection should load {basename}"
            );

            let refs_after_step = external_image_refs(&doc);
            assert_sample10_external_refs(label, &refs_after_step, image_index + 1);
            assert_eq!(
                external_image_basenames(&doc),
                expected_sample10_basenames_from(image_index + 1),
                "{label}: basename API should list only still-missing images after injecting {basename}"
            );
        }

        let after_payloads = page_layer_image_payload_count(&doc, 0);
        assert!(
            after_payloads >= SAMPLE10_EXTERNAL_IMAGES.len(),
            "{label}: PageLayerTree should include mime/base64 payloads after injection"
        );

        let after_items = canvaskit_image_items(&doc, 0);
        let injected_direct = after_items
            .iter()
            .filter(|item| {
                item["status"].as_str() == Some("direct")
                    && item_detail_contains(item, "externalImage")
                    && item_detail_contains(item, "injectedImageData")
            })
            .count();
        assert!(
            injected_direct >= SAMPLE10_EXTERNAL_IMAGES.len(),
            "{label}: injected external images should be directly replayable: {after_items:?}"
        );
        assert!(
            !after_items.iter().any(|item| {
                item_detail_contains(item, "externalImage")
                    && item_detail_contains(item, "missingImageData")
            }),
            "{label}: injected external images should not remain in missing-image state: {after_items:?}"
        );
    }
}

#[test]
fn issue_1143_key_injection_targets_only_the_requested_reference() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    let rdb02_ref = external_image_refs(&doc)
        .into_iter()
        .find(|reference| reference.bin_data_id == 2)
        .expect("binDataId 2 ref");
    assert_eq!(rdb02_ref.key, "binData:2");
    assert_eq!(rdb02_ref.basename, "rdb02.gif");

    let rdb02 = read_sample("samples/rdb02.gif");
    assert_eq!(
        doc.inject_external_image_by_key(&rdb02_ref.key, &rdb02, "/tmp/rdb02.gif"),
        1
    );

    let refs = external_image_refs(&doc);
    assert!(
        refs.iter()
            .any(|reference| reference.bin_data_id == 2 && reference.loaded),
        "requested key should be loaded"
    );
    assert!(
        refs.iter()
            .any(|reference| reference.bin_data_id == 1 && !reference.loaded)
            && refs
                .iter()
                .any(|reference| reference.bin_data_id == 3 && !reference.loaded),
        "key injection must not fall back to basename matching"
    );
}

#[test]
fn issue_1143_key_injection_rejects_invalid_or_unknown_keys() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");
    let oracle = read_sample("samples/oracle.gif");

    for key in [
        "",
        "oracle.gif",
        "binData:",
        "binData:0",
        "binData:not-a-number",
        "binData:999",
    ] {
        assert_eq!(
            doc.inject_external_image_by_key(key, &oracle, ""),
            0,
            "invalid or unknown key should not inject: {key}"
        );
    }

    assert!(
        external_image_refs(&doc)
            .iter()
            .all(|reference| !reference.loaded),
        "invalid key attempts must not mutate external image loaded state"
    );
}

#[test]
fn issue_1143_basename_injection_respects_index_first_loaded_state() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    doc.document_mut().bin_data_content[0] = BinDataContent {
        id: 99,
        data: vec![b'G', b'I', b'F'],
        extension: "gif".to_string(),
    };
    let before = doc.document().bin_data_content[0].clone();

    let injected = doc.inject_external_image("oracle.gif", &read_sample("samples/oracle.gif"), "");
    assert_eq!(
        injected, 0,
        "already-loaded detection must follow bin_data_content[binDataId - 1]"
    );
    assert_eq!(doc.document().bin_data_content[0].id, before.id);
    assert_eq!(doc.document().bin_data_content[0].data, before.data);
    assert_eq!(
        doc.document().bin_data_content[0].extension,
        before.extension
    );
}

#[test]
fn issue_1143_wasm_injection_invalidates_cached_page_layer_tree() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    let before = page_layer_image_payload_count(&doc, 0);
    assert_eq!(
        before, 0,
        "sample10 page 1 external images should have no payload before injection"
    );

    for (basename, sample_path) in [
        ("oracle.gif", "samples/oracle.gif"),
        ("rdb02.gif", "samples/rdb02.gif"),
        ("s1.jpg", "samples/s1.jpg"),
    ] {
        let data = read_sample(sample_path);
        assert_eq!(doc.inject_external_image(basename, &data, ""), 1);
    }

    let after = page_layer_image_payload_count(&doc, 0);
    assert!(
        after > before,
        "PageLayerTree cache should be rebuilt with injected image payloads"
    );
}

#[test]
fn issue_1143_native_dir_population_invalidates_cached_page_layer_tree() {
    let mut doc = load_doc("samples/hwp3-sample10.hwp");

    let before = page_layer_image_payload_count(&doc, 0);
    assert_eq!(
        before, 0,
        "sample10 page 1 external images should have no payload before directory population"
    );

    let sample_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
    let loaded = doc.populate_external_images_from_dir(&sample_dir);
    assert_eq!(loaded, 3);

    assert!(
        external_image_refs(&doc)
            .iter()
            .all(|reference| reference.loaded),
        "all discovered sample10 external image refs should be loaded"
    );

    let after = page_layer_image_payload_count(&doc, 0);
    assert!(
        after > before,
        "native directory population should rebuild cached PageLayerTrees"
    );
}
