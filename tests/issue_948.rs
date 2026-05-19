//! Issue #948: PageLayerTree textRun JSON display text contract.
//!
//! PageLayerTree consumers should not have to duplicate rhwp core's
//! HWP/Hancom PUA display-text rules. The JSON contract keeps source `text`
//! while exposing additive `displayText` / `displayPositions` for drawing.

use rhwp::wasm_api::HwpDocument;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

fn read_bokhakwonseo() -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/복학원서.hwp");
    fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e))
}

fn collect_text_runs<'a>(value: &'a Value, out: &mut Vec<&'a Map<String, Value>>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("textRun") {
                out.push(map);
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

#[test]
fn issue_948_layer_tree_text_run_exposes_signature_display_text() {
    let bytes = read_bokhakwonseo();
    let doc = HwpDocument::from_bytes(&bytes).expect("parse samples/복학원서.hwp");
    let json = doc
        .get_page_layer_tree_native(0)
        .expect("render samples/복학원서.hwp page 1 layer tree");
    let root: Value = serde_json::from_str(&json).expect("parse PageLayerTree JSON");

    assert!(
        json.contains("\"text.displayText\""),
        "PageLayerTree metadata should advertise the additive display text feature",
    );

    let mut runs = Vec::new();
    collect_text_runs(&root, &mut runs);
    assert!(
        !runs.is_empty(),
        "page layer tree should contain textRun ops"
    );

    let signature_run = runs
        .iter()
        .find(|run| {
            run.get("text")
                .and_then(Value::as_str)
                .is_some_and(|text| text.contains('\u{F012B}'))
        })
        .expect("signature source textRun should preserve U+F012B in text");

    let source_text = signature_run
        .get("text")
        .and_then(Value::as_str)
        .expect("signature textRun should have text");
    let display_text = signature_run
        .get("displayText")
        .and_then(Value::as_str)
        .expect("signature textRun should expose displayText");
    let source_positions = signature_run
        .get("positions")
        .and_then(Value::as_array)
        .expect("signature textRun should have source positions");
    let display_positions = signature_run
        .get("displayPositions")
        .and_then(Value::as_array)
        .expect("signature textRun should have display positions");

    assert!(source_text.contains('\u{F012B}'));
    assert!(
        display_text.contains("(인)"),
        "displayText should expose the core-rendered signature seal text. got: {:?}",
        display_text,
    );
    assert!(
        !display_text.contains('\u{F012B}') && !display_text.contains('\u{F081C}'),
        "displayText should not leak source PUA/filler characters. got: {:?}",
        display_text,
    );
    assert_eq!(
        source_positions.len(),
        source_text.chars().count() + 1,
        "source positions should remain source-text based",
    );
    assert_eq!(
        display_positions.len(),
        display_text.chars().count() + 1,
        "displayPositions should be displayText based",
    );

    assert!(
        runs.iter().any(|run| {
            run.get("displayText").is_none()
                && run
                    .get("text")
                    .and_then(Value::as_str)
                    .is_some_and(|text| !text.contains('\u{F012B}') && !text.contains('\u{F081C}'))
        }),
        "ordinary textRun ops should keep using text/positions fallback without displayText",
    );
}
