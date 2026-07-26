//! Task #3315: 그림 바이트를 JSON 으로 내보내는 두 생산자(레이어 트리 / 오버레이)가
//! 이스케이프 스캔 없이 base64 를 버퍼로 직접 흘린다.
//!
//! 스캔을 없앨 수 있는 근거는 "base64 알파벳에는 이스케이프 대상이 없다"는 전제이므로,
//! 실제 문서로 ①JSON 이 유효하고 ②바이트가 원본으로 되돌아오며 ③두 생산자가 같은
//! 바이트를 내놓는지 고정한다.

use base64::Engine;
use serde_json::Value;

fn collect_image_base64(node: &Value, out: &mut Vec<String>) {
    if let Some(ops) = node.get("ops").and_then(Value::as_array) {
        for op in ops {
            if op.get("type").and_then(Value::as_str) == Some("image") {
                if let Some(encoded) = op.get("base64").and_then(Value::as_str) {
                    out.push(encoded.to_string());
                }
            }
        }
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_image_base64(child, out);
        }
    }
    // clipRect 노드는 단일 `child` 를 갖는다.
    if let Some(child) = node.get("child") {
        collect_image_base64(child, out);
    }
}

#[test]
fn issue_3315_layer_and_overlay_json_emit_identical_image_bytes() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join("samples/hwpx/issue_241.hwpx");
    let bytes = std::fs::read(&path).expect("read samples/hwpx/issue_241.hwpx");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse issue_241.hwpx");

    let overlay_json = doc
        .get_page_overlay_images_native(0)
        .expect("overlay images");
    let overlay: Value =
        serde_json::from_str(&overlay_json).expect("overlay JSON 이 유효해야 한다");
    let stamp = &overlay["front"][0];
    let overlay_encoded = stamp["base64"].as_str().expect("overlay base64");
    let overlay_bytes = base64::engine::general_purpose::STANDARD
        .decode(overlay_encoded)
        .expect("overlay base64 왕복");
    assert!(!overlay_bytes.is_empty(), "도장 그림 바이트가 비어 있다");

    let layer_json = doc.get_page_layer_tree_native(0).expect("layer tree");
    let layer: Value = serde_json::from_str(&layer_json).expect("레이어 JSON 이 유효해야 한다");
    let mut layer_encoded = Vec::new();
    collect_image_base64(&layer["root"], &mut layer_encoded);
    assert!(
        !layer_encoded.is_empty(),
        "레이어 트리에 그림 op 이 있어야 한다"
    );

    let layer_bytes: Vec<Vec<u8>> = layer_encoded
        .iter()
        .map(|encoded| {
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .expect("레이어 base64 왕복")
        })
        .collect();
    assert!(
        layer_bytes.contains(&overlay_bytes),
        "같은 그림인데 두 생산자의 바이트가 다르다"
    );
}
