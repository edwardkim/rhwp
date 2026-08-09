use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

pub fn document_with_filename_footer() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document()
        .expect("create blank document fixture");
    doc.apply_hf_template(0, false, 0, 4)
        .expect("apply footer template with page number and filename field");
    doc
}

/// 화면에 실제로 그려지는 글자를 모은다.
///
/// 필드처럼 모델 1자가 표시 N자인 런은 `text` 에 모델 마커를 남기고 `displayText` 에
/// 치환값을 담는다 — 스튜디오도 `op.displayText ?? op.text` 로 그린다 (Task #3216).
fn collect_text_runs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("textRun") {
                let rendered = map
                    .get("displayText")
                    .and_then(Value::as_str)
                    .or_else(|| map.get("text").and_then(Value::as_str));
                if let Some(text) = rendered {
                    out.push(text.to_string());
                }
            }
            for child in map.values() {
                collect_text_runs(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_text_runs(item, out);
            }
        }
        _ => {}
    }
}

pub fn layer_tree_texts(doc: &HwpDocument) -> Vec<String> {
    let json = doc
        .get_page_layer_tree_native(0)
        .expect("page 1 PageLayerTree JSON");
    let parsed: Value = serde_json::from_str(&json).expect("parse PageLayerTree JSON");
    let mut texts = Vec::new();
    collect_text_runs(&parsed, &mut texts);
    texts
}
