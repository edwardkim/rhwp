//! Task #3315: 레이어 JSON 의 그림 신원 키(`imageKey`).
//!
//! 소비자가 디코드 결과를 캐시하려면 "이 바이트가 아까 그 바이트"임을 편집을 건너뛰어
//! 알아볼 수 있어야 한다. 그래서 키가 만족해야 하는 성질은 두 가지다 —
//! ①본문 편집으로는 바뀌지 않는다 ②그림이 달라질 수 있는 경계에서는 반드시 바뀐다.

use serde_json::Value;

fn collect_image_keys(node: &Value, out: &mut Vec<String>) {
    if let Some(ops) = node.get("ops").and_then(Value::as_array) {
        for op in ops {
            if op.get("type").and_then(Value::as_str) != Some("image") {
                continue;
            }
            if op.get("base64").is_none() {
                continue;
            }
            out.push(
                op.get("sourceImageKey")
                    .and_then(Value::as_str)
                    .unwrap_or("<없음>")
                    .to_string(),
            );
        }
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_image_keys(child, out);
        }
    }
    if let Some(child) = node.get("child") {
        collect_image_keys(child, out);
    }
}

fn image_keys(doc: &rhwp::wasm_api::HwpDocument) -> Vec<String> {
    let json = doc.get_page_layer_tree_native(0).expect("layer tree");
    let value: Value = serde_json::from_str(&json).expect("valid layer JSON");
    let mut keys = Vec::new();
    collect_image_keys(&value["root"], &mut keys);
    keys
}

fn open_sample() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(std::path::Path::new(repo_root).join("samples/hwpx/issue_241.hwpx"))
        .expect("read samples/hwpx/issue_241.hwpx");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse issue_241.hwpx")
}

#[test]
fn issue_3315_every_emitted_image_carries_a_key() {
    let doc = open_sample();
    let keys = image_keys(&doc);

    assert!(!keys.is_empty(), "도장 그림이 있어야 한다");
    for key in &keys {
        assert!(
            key.starts_with("bin:"),
            "바이트를 내보낸 그림에는 키가 붙어야 한다: {key}"
        );
    }
}

#[test]
fn issue_3315_image_key_survives_text_edits() {
    let mut doc = open_sample();
    let before = image_keys(&doc);

    for _ in 0..3 {
        doc.insert_text_native(0, 0, 0, "a").expect("insert text");
    }
    let after = image_keys(&doc);

    assert_eq!(
        before, after,
        "본문 편집은 그림 바이트를 바꾸지 않으므로 키가 유지돼야 한다"
    );
}

#[test]
fn issue_3315_image_key_changes_after_snapshot_restore() {
    let mut doc = open_sample();
    let before = image_keys(&doc);

    let id = doc.save_snapshot_native();
    doc.restore_snapshot_native(id).expect("restore snapshot");
    let after = image_keys(&doc);

    assert_eq!(before.len(), after.len(), "그림 개수는 그대로여야 한다");
    assert_ne!(
        before, after,
        "스냅샷 복원은 같은 bin_data_id 가 다른 그림을 가리키게 만들 수 있으므로 \
         키가 갱신돼야 한다"
    );
}

#[test]
fn issue_3315_distinct_images_get_distinct_keys() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let jpeg = std::fs::read(std::path::Path::new(repo_root).join("samples/images/tiger01.jpg"))
        .expect("read tiger01.jpg");
    let mut doc = open_sample();
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &jpeg,
        400,
        300,
        2400,
        1800,
        "jpg",
        "tiger",
        None,
        None,
    )
    .expect("insert picture");

    let keys = image_keys(&doc);
    let unique: std::collections::BTreeSet<&String> = keys.iter().collect();

    assert!(
        keys.len() >= 2,
        "도장 + 삽입한 그림이 있어야 한다: {keys:?}"
    );
    assert_eq!(
        unique.len(),
        keys.len(),
        "서로 다른 그림은 서로 다른 키를 받아야 한다: {keys:?}"
    );
}

#[test]
fn issue_3315_page_image_keys_match_layer_tree_keys() {
    let doc = open_sample();

    let keys_json = doc
        .get_page_source_image_keys_native(0)
        .expect("image keys");
    let value: Value = serde_json::from_str(&keys_json).expect("valid image-keys JSON");
    let compact: Vec<String> = value["keys"]
        .as_array()
        .expect("keys array")
        .iter()
        .map(|entry| entry.as_str().unwrap_or("<없음>").to_string())
        .collect();

    assert_eq!(
        compact,
        image_keys(&doc),
        "작은 키 조회 API 와 레이어 트리 JSON 이 같은 키를 같은 순서로 내야 한다"
    );
    assert!(
        keys_json.len() < 512,
        "서명 조회는 작아야 의미가 있다: {} bytes",
        keys_json.len()
    );
}

/// 그림 추가는 기존 그림의 바이트를 바꾸지 않는다 — 키가 유지돼야 캐시가 산다.
///
/// 세대 번호를 그림 등록에서 올리면 이 성질이 깨진다(무관한 그림의 키까지 바뀐다).
#[test]
fn issue_3315_inserting_an_image_keeps_existing_image_keys() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let jpeg = std::fs::read(std::path::Path::new(repo_root).join("samples/images/tiger01.jpg"))
        .expect("read tiger01.jpg");
    let mut doc = open_sample();
    let before = image_keys(&doc);

    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &jpeg,
        400,
        300,
        2400,
        1800,
        "jpg",
        "tiger",
        None,
        None,
    )
    .expect("insert picture");
    let after = image_keys(&doc);

    assert!(
        after.len() > before.len(),
        "그림이 하나 늘어야 한다: {before:?} -> {after:?}"
    );
    for key in &before {
        assert!(
            after.contains(key),
            "기존 그림의 키가 사라졌다: {key} (before={before:?}, after={after:?})"
        );
    }
}
