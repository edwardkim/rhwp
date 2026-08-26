//! Issue #4968 W9-Q3: kerning intent must reach TextStyle without changing K0 layout.

use std::collections::BTreeMap;

use rhwp::document_core::DocumentCore;

fn body_runs(value: &serde_json::Value) -> Vec<&serde_json::Value> {
    fn walk<'a>(value: &'a serde_json::Value, output: &mut Vec<&'a serde_json::Value>) {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("type").and_then(serde_json::Value::as_str) == Some("textRun")
                    && map
                        .get("text")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|text| text.starts_with("BODY R"))
                {
                    output.push(value);
                }
                for child in map.values() {
                    walk(child, output);
                }
            }
            serde_json::Value::Array(values) => {
                for child in values {
                    walk(child, output);
                }
            }
            _ => {}
        }
    }

    let mut output = Vec::new();
    walk(value, &mut output);
    output
}

#[test]
fn issue_4968_kerning_intent_reaches_text_style_without_changing_positions() {
    let fixture = include_bytes!(
        "../../mydocs/tech/investigations/issue-4968/fixtures/kerning_pair_fixture.hwpx"
    );
    let core = DocumentCore::from_bytes(fixture).expect("public kerning fixture parses");
    let json = core
        .get_page_layer_tree_native(0)
        .expect("public kerning fixture layer tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("layer tree JSON");
    let runs = body_runs(&tree);
    assert_eq!(runs.len(), 18, "fixture BODY matrix must remain bounded");

    let mut groups: BTreeMap<String, Vec<&serde_json::Value>> = BTreeMap::new();
    for run in runs {
        let text = run["text"].as_str().expect("BODY run text");
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let ratio = tokens.get(1).expect("ratio token");
        let spacing = tokens.get(2).expect("spacing token");
        let kerning = tokens.get(3).expect("kerning token");
        let lane = tokens.get(4).expect("lane token");
        match *kerning {
            "K0" => assert!(
                run["style"].get("kerning").is_none(),
                "K0 must preserve the pre-Q3 serialized schema"
            ),
            "K1" => assert_eq!(run["style"]["kerning"], true, "K1 intent was lost"),
            value => panic!("unexpected kerning token: {value}"),
        }
        groups
            .entry(format!("{ratio}/{spacing}/{lane}"))
            .or_default()
            .push(run);
    }

    assert_eq!(groups.len(), 9, "expected nine controlled K0/K1 groups");
    for (key, runs) in groups {
        assert_eq!(runs.len(), 2, "controlled group must have K0/K1: {key}");
        let (off, on) = if runs[0]["text"].as_str().unwrap().contains(" K0 ") {
            (runs[0], runs[1])
        } else {
            (runs[1], runs[0])
        };
        assert_eq!(off["positions"], on["positions"], "plumbing moved K1: {key}");
        let off_style = off["style"].clone();
        let mut on_style = on["style"].clone();
        on_style
            .as_object_mut()
            .expect("K1 style object")
            .remove("kerning");
        assert_eq!(off_style, on_style, "K1 changed another style field: {key}");
    }
}
