use rhwp::document_core::DocumentCore;

fn coverage(core: &DocumentCore) -> (String, serde_json::Value) {
    let raw = core
        .get_font_metric_coverage_analysis_native("{}")
        .expect("font metric coverage aggregate");
    let value = serde_json::from_str(&raw).expect("font metric coverage JSON");
    (raw, value)
}

fn count(value: &serde_json::Value, pointer: &str) -> u64 {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| panic!("missing count at {pointer}"))
}

fn assert_reconciled(value: &serde_json::Value) {
    let layout = count(value, "/counts/layoutCharacters");
    let coverage = count(value, "/counts/coverageCharacters");
    let not_applicable = count(value, "/counts/notApplicableCharacters");
    let excluded = count(value, "/counts/excludedCharacters");
    assert_eq!(layout, coverage + not_applicable + excluded);
    assert_eq!(count(value, "/counts/truncatedCharacters"), 0);

    let expected_categories = [
        "measured-overlay",
        "identity-alias-hit",
        "metric-surrogate",
        "exact-hit",
        "char-miss",
        "face-miss",
        "heuristic",
    ];
    let categories = value["categories"].as_object().expect("categories");
    assert_eq!(categories.len(), expected_categories.len());
    let category_sum: u64 = expected_categories
        .iter()
        .map(|category| {
            categories[*category]
                .as_u64()
                .unwrap_or_else(|| panic!("missing category {category}"))
        })
        .sum();
    assert_eq!(category_sum, coverage);

    assert_eq!(
        layout,
        count(value, "/joins/joined")
            + count(value, "/joins/layoutOnly")
            + count(value, "/joins/excluded")
    );
    assert_eq!(count(value, "/documents/attempted"), 1);
    assert_eq!(count(value, "/documents/success"), 1);
    for status in [
        "cancelled",
        "drm",
        "empty",
        "encrypted",
        "parser",
        "resource-limit",
        "unsupported",
    ] {
        assert_eq!(count(value, &format!("/documents/failures/{status}")), 0);
    }
    assert_eq!(count(value, "/backends/requested"), 0);
    for status in ["complete", "failed", "notObserved", "unsupported"] {
        assert_eq!(count(value, &format!("/backends/{status}")), 0);
    }

    let legacy_chars: u64 = value["legacyUsage"]
        .as_array()
        .expect("legacy usage")
        .iter()
        .map(|row| row["charCount"].as_u64().expect("legacy charCount"))
        .sum();
    let decision_chars: u64 = value["decisionUsage"]
        .as_array()
        .expect("decision usage")
        .iter()
        .map(|row| row["charCount"].as_u64().expect("decision charCount"))
        .sum();
    assert_eq!(legacy_chars, count(value, "/joins/joined"));
    assert_eq!(decision_chars, legacy_chars);
}

fn assert_private_data_absent(value: &serde_json::Value) {
    const FORBIDDEN_KEYS: [&str; 13] = [
        "absolutePath",
        "blake3",
        "character",
        "codePoint",
        "documentHash",
        "fileName",
        "filename",
        "inputRoot",
        "path",
        "rawTrace",
        "records",
        "riskDocuments",
        "source",
    ];
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                assert!(!FORBIDDEN_KEYS.contains(&key.as_str()), "forbidden key: {key}");
                assert_private_data_absent(child);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                assert_private_data_absent(child);
            }
        }
        serde_json::Value::String(text) => {
            assert!(!text.contains("/home/"), "home path leaked");
            assert!(!text.contains("/Users/"), "macOS home path leaked");
            assert!(!text.contains(":\\Users\\"), "Windows home path leaked");
        }
        _ => {}
    }
}

#[test]
fn public_fixture_aggregate_is_deterministic_reconciled_and_private() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let (first_raw, first) = coverage(&core);
    let (second_raw, second) = coverage(&core);
    assert_eq!(first_raw, second_raw);
    assert_eq!(first, second);
    assert_eq!(first["schemaVersion"], 1);
    assert_eq!(first["kind"], "font-metric-coverage-aggregate");
    assert_eq!(first["status"], "complete");
    assert_eq!(first["format"], "hwp");
    assert_reconciled(&first);
    assert_private_data_absent(&first);

    for field in ["legacyProjectionHash", "aggregateHash"] {
        assert_eq!(first[field]["algorithm"], "sha256");
        let digest = first[field]["value"].as_str().expect("SHA-256 digest");
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    assert_eq!(first["categories"]["identity-alias-hit"], 0);
}

#[test]
fn collector_has_no_w2_page_character_limit() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let inserted = "가".repeat(5_000);
    core.insert_text_native(0, 0, 0, &inserted)
        .expect("long paragraph insertion");
    let (_, value) = coverage(&core);
    assert_reconciled(&value);
    assert!(count(&value, "/counts/layoutCharacters") >= 5_000);
    assert_eq!(count(&value, "/counts/truncatedCharacters"), 0);
}

#[test]
fn coverage_query_does_not_mutate_existing_w2_trace() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let before = core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":64}"#)
        .expect("W2 trace before coverage");
    let _ = coverage(&core);
    let after = core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":64}"#)
        .expect("W2 trace after coverage");
    assert_eq!(before, after);
}

#[test]
fn resource_budgets_and_cancellation_fail_without_partial_success() {
    use std::sync::{atomic::AtomicBool, Arc};

    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    for options in [
        r#"{"maxWorkUnits":1}"#,
        r#"{"maxAggregateRows":1}"#,
        r#"{"maxOutputBytes":1024}"#,
    ] {
        let error = core
            .get_font_metric_coverage_analysis_native(options)
            .expect_err("resource policy must stop the whole document");
        let message = error.to_string();
        assert!(message.contains("[RESOURCE_LIMIT_EXCEEDED]"), "{message}");
        assert!(!message.contains("\"status\":\"complete\""), "{message}");
    }

    let cancelled = Arc::new(AtomicBool::new(true));
    let error = core
        .get_font_metric_coverage_analysis_with_cancel_native("{}", &cancelled)
        .expect_err("pre-cancelled analysis must not start");
    assert!(error.to_string().contains("[ANALYSIS_CANCELLED]"));

    assert!(core
        .get_font_metric_coverage_analysis_native(r#"{"unknown":1}"#)
        .is_err());
    assert!(core
        .get_font_metric_coverage_analysis_native(r#"{"maxWorkUnits":0}"#)
        .is_err());
}

#[test]
fn many_char_shape_boundaries_use_a_linear_merge_walk() {
    use rhwp::model::paragraph::CharShapeRef;

    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let inserted = "가".repeat(10_000);
    core.insert_text_native(0, 0, 0, &inserted)
        .expect("adversarial paragraph insertion");
    let mut document = core.document().clone();
    let paragraph = &mut document.sections[0].paragraphs[0];
    paragraph.char_shapes = (0..10_000)
        .map(|start_pos| CharShapeRef {
            start_pos,
            char_shape_id: 0,
        })
        .collect();
    core.set_document(document);

    let raw = core
        .get_font_metric_coverage_analysis_native(
            r#"{"maxWorkUnits":100000,"deadlineMillis":10000}"#,
        )
        .expect("linear walker remains inside deterministic work budget");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("aggregate JSON");
    assert!(count(&value, "/counts/layoutCharacters") >= 10_000);
    assert_reconciled(&value);
}
