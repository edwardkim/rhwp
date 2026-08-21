use rhwp::document_core::DocumentCore;
use std::collections::BTreeMap;

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

fn public_coverage(path: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("public fixture read failed ({}): {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| {
        panic!("public fixture parse failed ({}): {error}", path.display())
    });
    coverage(&core).1
}

fn width_source_counts(value: &serde_json::Value) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for row in value["decisionUsage"].as_array().expect("decision usage") {
        let source = row["widthSource"].as_str().expect("widthSource");
        *counts.entry(source.to_string()).or_default() +=
            row["charCount"].as_u64().expect("charCount");
    }
    counts
}

fn public_blank_with_font_text(font_name: &str, text: &str) -> DocumentCore {
    use rhwp::model::paragraph::{CharShapeRef, Paragraph};
    use rhwp::model::style::Font;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();
    assert!(document.doc_info.font_faces.len() >= 7);

    let mut font_ids = [0_u16; 7];
    for (language, faces) in document.doc_info.font_faces.iter_mut().take(7).enumerate() {
        font_ids[language] = u16::try_from(faces.len()).expect("font fixture id");
        faces.push(Font {
            name: font_name.to_string(),
            alt_type: 1,
            ..Default::default()
        });
    }
    let mut char_shape = document
        .doc_info
        .char_shapes
        .first()
        .cloned()
        .expect("blank char shape");
    char_shape.raw_data = None;
    char_shape.font_ids = font_ids;
    let char_shape_id = u32::try_from(document.doc_info.char_shapes.len())
        .expect("char shape fixture id");
    document.doc_info.char_shapes.push(char_shape);

    let mut paragraph = Paragraph::new_empty();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    document.sections[0].paragraphs = vec![paragraph];
    core.set_document(document);
    core.insert_text_native(0, 0, 0, text)
        .expect("fixture text insertion");
    core
}

fn trace_width_source_counts(core: &DocumentCore) -> BTreeMap<String, u64> {
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
            .expect("W2 fixture trace"),
    )
    .expect("W2 fixture trace JSON");
    assert_eq!(trace["status"], "complete");
    let mut counts = BTreeMap::new();
    for record in trace["records"].as_array().expect("trace records") {
        let source = record["layoutMetric"]["widthSource"]
            .as_str()
            .expect("trace widthSource");
        *counts.entry(source.to_string()).or_default() += 1;
    }
    counts
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
    assert!(core
        .get_font_metric_coverage_analysis_native(r#"{"maxNestingDepth":0}"#)
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

#[test]
fn public_hwp_hwpx_classification_golden_is_deterministic_and_format_portable() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../mydocs/tech/investigations/issue-4962/font_metric_coverage_public_fixtures.json"
    ))
    .expect("public fixture manifest");
    let mut results = BTreeMap::new();
    for fixture in manifest["documents"].as_array().expect("public documents") {
        let id = fixture["id"].as_str().expect("fixture id");
        let value = public_coverage(fixture["path"].as_str().expect("fixture path"));
        assert_reconciled(&value);
        assert_private_data_absent(&value);
        assert_eq!(value["format"], fixture["format"], "{id}");
        assert_eq!(value["counts"], fixture["counts"], "{id}");
        assert_eq!(value["categories"], fixture["categories"], "{id}");
        assert_eq!(
            serde_json::to_value(width_source_counts(&value)).expect("width source JSON"),
            fixture["widthSources"],
            "{id}"
        );
        assert_eq!(value["aggregateHash"]["value"], fixture["aggregateHash"], "{id}");
        assert_eq!(
            value["legacyProjectionHash"]["value"], fixture["legacyProjectionHash"],
            "{id}"
        );
        results.insert(id.to_string(), value);
    }

    let hwp = &results["format-parity-hwp"];
    let hwpx = &results["format-parity-hwpx"];
    for field in ["counts", "categories", "joins", "legacyUsage", "decisionUsage"] {
        assert_eq!(hwp[field], hwpx[field], "portable HWP/HWPX field: {field}");
    }
}

#[test]
fn public_blank_derived_documents_reach_every_current_positive_category() {
    let cases = [
        ("함초롬바탕", "가", "exact-hit", "embeddedMetric"),
        ("KoPub돋움체 Light", "가", "measured-overlay", "kopubTable"),
        ("본한글", "가", "metric-surrogate", "embeddedMetric"),
        ("함초롬바탕", "😀", "char-miss", "heuristicFullwidth"),
        ("W3 Missing Face", "A", "face-miss", "heuristicHalfwidth"),
        ("W3 Missing Face", "ㆍ", "heuristic", "areaDotFallback"),
    ];
    for (font, text, category, width_source) in cases {
        let core = public_blank_with_font_text(font, text);
        let (_, value) = coverage(&core);
        assert_reconciled(&value);
        assert_private_data_absent(&value);
        assert_eq!(count(&value, &format!("/categories/{category}")), 1, "{category}");
        assert_eq!(
            width_source_counts(&value),
            BTreeMap::from([(width_source.to_string(), 1)]),
            "{category}"
        );
        assert_eq!(
            trace_width_source_counts(&core),
            width_source_counts(&value),
            "W2/W3 decision equivalence: {category}"
        );
        assert_eq!(count(&value, "/categories/identity-alias-hit"), 0);
    }
}

#[test]
fn public_blank_derived_document_keeps_all_non_applicable_width_sources_out_of_coverage() {
    let text = "\u{1100}\u{1161}\u{11AB}\u{FFFC}\u{F081C}\u{2007}\t";
    let core = public_blank_with_font_text("함초롬바탕", text);
    let (_, value) = coverage(&core);
    assert_reconciled(&value);
    assert_private_data_absent(&value);
    assert_eq!(count(&value, "/counts/layoutCharacters"), 7);
    assert_eq!(count(&value, "/counts/coverageCharacters"), 1);
    assert_eq!(count(&value, "/counts/notApplicableCharacters"), 6);

    let expected = BTreeMap::from([
        ("clusterContinuation".to_string(), 2),
        ("embeddedMetric".to_string(), 1),
        ("figureSpace".to_string(), 1),
        ("hwpPuaFiller".to_string(), 1),
        ("inlineObjectPlaceholder".to_string(), 1),
        ("tabAdvance".to_string(), 1),
    ]);
    assert_eq!(width_source_counts(&value), expected);
    assert_eq!(trace_width_source_counts(&core), expected);
}
