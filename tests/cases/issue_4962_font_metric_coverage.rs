use rhwp::document_core::DocumentCore;

fn coverage(core: &DocumentCore) -> (String, serde_json::Value) {
    let raw = core
        .get_font_metric_coverage_analysis_native()
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
