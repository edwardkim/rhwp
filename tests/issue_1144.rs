//! Issue #1144: filename field context must be resolved before PageLayerTree.
//!
//! Filename is document context, not renderer-local metadata. PageLayerTree,
//! native Skia export, and CanvasKit direct replay all consume the same
//! filename-resolved layer state through `build_page_layer_tree`.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn document_with_filename_footer() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document()
        .expect("create blank document fixture");
    doc.apply_hf_template(0, false, 0, 4)
        .expect("apply footer template with page number and filename field");
    doc
}

fn collect_text_runs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("textRun") {
                if let Some(text) = map.get("text").and_then(Value::as_str) {
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

fn layer_tree_texts(doc: &HwpDocument) -> Vec<String> {
    let json = doc
        .get_page_layer_tree_native(0)
        .expect("page 1 PageLayerTree JSON");
    let parsed: Value = serde_json::from_str(&json).expect("parse PageLayerTree JSON");
    let mut texts = Vec::new();
    collect_text_runs(&parsed, &mut texts);
    texts
}

#[test]
fn issue_1144_page_layer_tree_uses_filename_context() {
    let mut doc = document_with_filename_footer();
    doc.set_file_name("issue-1144-fixture.hwp");

    let texts = layer_tree_texts(&doc);

    assert!(
        texts
            .iter()
            .any(|text| text.contains("issue-1144-fixture.hwp")),
        "PageLayerTree text runs should contain resolved filename. texts={texts:?}"
    );
    assert!(
        texts.iter().all(|text| !text.contains('\u{0017}')),
        "PageLayerTree should not leak raw filename field marker. texts={texts:?}"
    );
}

#[test]
fn issue_1144_set_file_name_invalidates_cached_page_tree() {
    let mut doc = document_with_filename_footer();
    doc.set_file_name("old-name.hwp");

    let old_texts = layer_tree_texts(&doc);
    assert!(
        old_texts.iter().any(|text| text.contains("old-name.hwp")),
        "initial render should resolve old filename. texts={old_texts:?}"
    );

    doc.set_file_name("new-name.hwp");
    let new_texts = layer_tree_texts(&doc);

    assert!(
        new_texts.iter().any(|text| text.contains("new-name.hwp")),
        "cached PageRenderTree should be invalidated after filename change. texts={new_texts:?}"
    );
    assert!(
        new_texts.iter().all(|text| !text.contains("old-name.hwp")),
        "stale filename should not survive cache invalidation. texts={new_texts:?}"
    );
}

#[test]
fn issue_1144_canvas_kit_plan_entrypoint_does_not_freeze_filename_context() {
    let mut doc = document_with_filename_footer();
    doc.set_file_name("canvas-kit-old.hwp");

    doc.get_canvaskit_replay_plan_native(0, "default")
        .expect("CanvasKit replay plan should build through PageLayerTree");

    doc.set_file_name("canvas-kit-new.hwp");
    let texts = layer_tree_texts(&doc);

    assert!(
        texts.iter().any(|text| text.contains("canvas-kit-new.hwp")),
        "CanvasKit replay plan entrypoint should not leave stale cached filename. texts={texts:?}"
    );
    assert!(
        texts
            .iter()
            .all(|text| !text.contains("canvas-kit-old.hwp")),
        "old filename from CanvasKit plan build should not remain cached. texts={texts:?}"
    );
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn issue_1144_skia_png_export_entrypoint_does_not_freeze_filename_context() {
    let mut doc = document_with_filename_footer();
    doc.set_file_name("skia-old.hwp");

    let png = doc
        .render_page_png_native(0)
        .expect("Skia PNG export should build through PageLayerTree");
    assert!(!png.is_empty(), "Skia PNG export should produce bytes");

    doc.set_file_name("skia-new.hwp");
    let texts = layer_tree_texts(&doc);

    assert!(
        texts.iter().any(|text| text.contains("skia-new.hwp")),
        "Skia export entrypoint should not leave stale cached filename. texts={texts:?}"
    );
    assert!(
        texts.iter().all(|text| !text.contains("skia-old.hwp")),
        "old filename from Skia export should not remain cached. texts={texts:?}"
    );
}

#[test]
fn issue_1144_empty_filename_context_resolves_to_empty_string() {
    let mut doc = document_with_filename_footer();
    doc.set_file_name("");

    let texts = layer_tree_texts(&doc);

    assert!(
        texts.iter().any(|text| text == "1\t"),
        "empty filename should resolve to empty string while preserving surrounding text. texts={texts:?}"
    );
    assert!(
        texts.iter().all(|text| !text.contains('\u{0017}')),
        "empty filename fallback should not leak raw marker. texts={texts:?}"
    );
}
