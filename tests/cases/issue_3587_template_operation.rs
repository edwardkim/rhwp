//! Full dry-run contracts. Synthetic paragraphs are not visual oracles.
use rhwp::{
    document_core::{DocumentCore, TemplateOperation},
    model::paragraph::Paragraph,
};
use serde_json::{json, Value};

fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().sections[0].paragraphs = ["before", "A😀B", "", "after"]
        .map(|s| {
            let mut p = Paragraph::default();
            p.insert_text_at(0, s);
            p
        })
        .to_vec();
    c
}
fn binding() -> Value {
    json!({"key":"value","target":{"kind":"textRange","path":[{"kind":"paragraph","index":0}],"start":0,"end":3}})
}
fn form() -> Value {
    json!({"action":"fill_template","request":{"scope":{"sectionIndex":0,"start":1,"end":3},"bindings":[binding()],"record":{"value":"새😀\n기록"}}})
}
fn block() -> Value {
    json!({"action":"repeat_and_fill_paragraph_block","request":{"block":{"sectionIndex":0,"sourceStart":1,"sourceEnd":3,"insertBefore":3,"count":2},"bindings":[binding()],"records":[{"value":"첫째"},{"value":"둘째"}]}})
}
fn state(c: &DocumentCore) -> String {
    format!(
        "{:?}|{}|{:?}|{:?}",
        c.document(),
        c.serialize_event_log(),
        c.get_clipboard_text_native(),
        c.page_count()
    )
}

#[test]
fn full_preview_preserves_state_and_matches_commit_for_form_and_block() {
    for request in [form(), block()] {
        let mut c = core();
        let before = state(&c);
        let op = TemplateOperation::from_json(&request.to_string()).unwrap();
        let preview = c.preview_template_operation_native(&op).unwrap();
        assert_eq!(before, state(&c));
        assert_eq!(preview, c.preview_template_operation_native(&op).unwrap());
        assert_eq!(preview, c.execute_template_operation_native(&op).unwrap());
        assert_ne!(before, state(&c));
        let paragraphs = &c.document().sections[0].paragraphs;
        assert_eq!(paragraphs[0].text, "before");
        assert_eq!(paragraphs.last().unwrap().text, "after");
        if request["action"] == "fill_template" {
            assert_eq!(paragraphs[1].text, "새😀\n기록");
            assert_eq!(paragraphs.len(), 4);
        } else {
            assert_eq!(paragraphs[1].text, "A😀B");
            assert_eq!(paragraphs[3].text, "첫째");
            assert_eq!(paragraphs[5].text, "둘째");
            assert_eq!(paragraphs.len(), 8);
        }
    }
}

#[test]
fn rows_preview_matches_real_template_commit_and_keeps_original_state() {
    let bytes = std::fs::read("samples/rnote/labnote-001.hwp").expect("required sample");
    let mut c = DocumentCore::from_bytes(&bytes).unwrap();
    let op = TemplateOperation::from_json(&json!({"action":"repeat_and_fill_table_rows","request":{
        "sectionIndex":0,"paragraphIndex":12,"controlIndex":1,"startRow":5,"endRow":6,"insertBefore":6,
        "bindings":[],"records":[{},{}]
    }}).to_string()).unwrap();
    let before = state(&c);
    let preview = c.preview_template_operation_native(&op).unwrap();
    assert_eq!(before, state(&c));
    assert_eq!(preview, c.execute_template_operation_native(&op).unwrap());
    let value = serde_json::to_value(preview).unwrap();
    assert_eq!(value["result"]["rowCount"], 36);
    assert_eq!(value["result"]["copies"].as_array().unwrap().len(), 2);
}

#[test]
fn invalid_last_record_and_target_are_equally_rejected_without_mutation() {
    for mut request in [block(), form()] {
        if request["action"] == "fill_template" {
            request["request"]["bindings"][0]["target"]["end"] = json!(100);
        } else {
            request["request"]["records"][1] = json!({"missing":"x"});
        }
        let mut c = core();
        let before = state(&c);
        let op = TemplateOperation::from_json(&request.to_string()).unwrap();
        let error = c
            .preview_template_operation_native(&op)
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            c.execute_template_operation_native(&op)
                .unwrap_err()
                .to_string()
        );
        assert_eq!(before, state(&c));
    }
}

#[test]
fn typed_json_rejects_unknown_fields_bad_types_and_oversized_input() {
    let mut bad = form();
    bad["request"]["scope"]["star"] = json!(0);
    assert!(TemplateOperation::from_json(&bad.to_string()).is_err());
    let too_large = json!({"record":{"value":"x".repeat(8 * 1024 * 1024)}});
    assert!(TemplateOperation::from_parts("fill_template", &too_large)
        .unwrap_err()
        .to_string()
        .contains("8 MiB"));
    bad = form();
    bad["request"]["record"]["value"] = json!(12);
    assert!(TemplateOperation::from_json(&bad.to_string()).is_err());
    bad = form();
    bad["request"]["bindings"][0]["target"]["start"] = json!(-1);
    assert!(TemplateOperation::from_json(&bad.to_string()).is_err());
    assert!(
        TemplateOperation::from_json(&" ".repeat(8 * 1024 * 1024 + 1))
            .unwrap_err()
            .to_string()
            .contains("8 MiB")
    );
}

#[test]
fn wasm_options_boundary_uses_same_engine_and_strict_boolean() {
    let mut c = core();
    let before = state(&c);
    let options = json!({"operation":block(),"dryRun":true});
    let preview: Value = serde_json::from_str(
        &c.apply_template_operation_json_native(&options.to_string())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(state(&c), before);
    assert_eq!(preview["dryRun"], true);
    assert_eq!(preview["changedPages"], Value::Null);
    let mut bad = options.clone();
    bad["dryRun"] = json!("false");
    assert!(c
        .apply_template_operation_json_native(&bad.to_string())
        .is_err());
    assert_eq!(state(&c), before);
    let output: Value = serde_json::from_str(
        &c.apply_template_operation_json_native(&json!({"operation":block()}).to_string())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(output["dryRun"], false);
    assert_eq!(output["operationResult"], preview["operationResult"]);
    assert_eq!(c.document().sections[0].paragraphs[5].text, "둘째");
}

#[test]
fn independent_target_errors_are_reported_together_without_mutation() {
    let mut c = core();
    let mut request = form();
    request["request"]["bindings"][0]["target"]["end"] = json!(999);
    request["request"]["bindings"].as_array_mut().unwrap().push(json!({"key":"other","target":{"kind":"textRange","path":[{"kind":"paragraph","index":99}],"start":0,"end":0}}));
    request["request"]["record"]["other"] = json!("x");
    let op = TemplateOperation::from_json(&request.to_string()).unwrap();
    let before = state(&c);
    let error = c
        .preview_template_operation_native(&op)
        .unwrap_err()
        .to_string();
    assert!(error.contains("fillRange"), "{error}");
    assert!(error.contains("fillPath"), "{error}");
    assert_eq!(
        error,
        c.execute_template_operation_native(&op)
            .unwrap_err()
            .to_string()
    );
    assert_eq!(before, state(&c));
}
