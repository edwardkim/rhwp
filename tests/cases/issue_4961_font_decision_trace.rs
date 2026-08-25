use rhwp::document_core::DocumentCore;

fn standalone_native_reason() -> &'static str {
    if cfg!(all(not(target_arch = "wasm32"), feature = "native-skia")) {
        "nativeRendererSnapshotRequired"
    } else {
        "nativeSkiaFeatureUnavailable"
    }
}

fn stage4_e2e_manifest() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../mydocs/tech/investigations/issue-4961/font_decision_trace_e2e.json"
    ))
    .expect("Stage 4 E2E manifest JSON")
}

fn public_trace(path: &str, page: u32, max_characters: u64) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("public fixture read failed ({}): {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| {
        panic!("public fixture parse failed ({}): {error}", path.display())
    });
    let json = core
        .get_font_decision_trace_native(page, &format!(r#"{{"maxCharacters":{max_characters}}}"#))
        .unwrap_or_else(|error| {
            panic!("public fixture trace failed ({}): {error}", path.display())
        });
    serde_json::from_str(&json).expect("public fixture trace JSON")
}

fn assert_stage4_profile(trace: &serde_json::Value, profile: &serde_json::Value) {
    let record_id = profile["recordId"].as_str().expect("profile recordId");
    let record = trace["records"]
        .as_array()
        .expect("trace records")
        .iter()
        .find(|record| record["recordId"] == record_id)
        .unwrap_or_else(|| panic!("profile record is absent: {record_id}"));
    let expected = &profile["expected"];
    assert_eq!(record["source"]["status"], expected["sourceStatus"]);
    assert_eq!(record["source"]["character"], expected["character"]);
    assert_eq!(record["document"]["face"], expected["documentFace"]);
    assert_eq!(record["document"]["substFont"], expected["substFont"]);
    assert_eq!(
        record["layoutName"]["normalizedFace"],
        expected["normalizedFace"]
    );
    let expected_step = &expected["layoutStepKind"];
    if expected_step.is_null() {
        assert!(record["layoutName"]["steps"]
            .as_array()
            .expect("layout steps")
            .is_empty());
    } else {
        assert!(record["layoutName"]["steps"]
            .as_array()
            .expect("layout steps")
            .iter()
            .any(|step| step.get("kind") == Some(expected_step)));
    }
    assert_eq!(
        record["layoutMetric"]["matchKind"],
        expected["metricMatchKind"]
    );
    assert_eq!(
        record["layoutMetric"]["characterMatch"],
        expected["metricCharacterMatch"]
    );
    assert_eq!(
        record["layoutMetric"]["widthSource"],
        expected["widthSource"]
    );
}

#[test]
fn stage4_public_hwp_hwpx_profiles_are_end_to_end_and_feature_detected() {
    let manifest = stage4_e2e_manifest();
    let max_characters = manifest["options"]["maxCharacters"]
        .as_u64()
        .expect("maxCharacters");
    let mut traces = std::collections::HashMap::new();
    for document in manifest["documents"].as_array().expect("documents") {
        let id = document["id"].as_str().expect("document id");
        let trace = public_trace(
            document["path"].as_str().expect("document path"),
            document["page"].as_u64().expect("page") as u32,
            max_characters,
        );
        assert_eq!(trace["status"], document["expectedStatus"], "{id}");
        assert_eq!(trace["counts"], document["expectedCounts"], "{id}");
        assert_eq!(
            trace["layoutHash"]["value"], document["expectedLayoutHash"],
            "{id}"
        );
        assert_eq!(trace["backendSummary"]["native"]["status"], "unsupported");
        assert_eq!(
            trace["backendSummary"]["native"]["reasons"][0],
            standalone_native_reason()
        );
        traces.insert(id.to_string(), trace);
    }

    for profile in manifest["profiles"].as_array().expect("profiles") {
        let document_id = profile["documentId"].as_str().expect("profile documentId");
        assert_stage4_profile(
            traces.get(document_id).expect("profile document trace"),
            profile,
        );
    }

    let parity = manifest["comparisons"]["portableFormatParity"]
        .as_array()
        .expect("portable parity pair");
    let hwp = traces
        .get(parity[0].as_str().expect("HWP parity id"))
        .expect("HWP parity trace");
    let hwpx = traces
        .get(parity[1].as_str().expect("HWPX parity id"))
        .expect("HWPX parity trace");
    assert_eq!(hwp["layoutHash"], hwpx["layoutHash"]);
    assert_eq!(hwp["records"], hwpx["records"]);

    let feature = &manifest["comparisons"]["substFeatureDetection"];
    let without = traces
        .get(
            feature["withoutSubstFont"]
                .as_str()
                .expect("without subst id"),
        )
        .expect("without subst trace");
    let with = traces
        .get(feature["withSubstFont"].as_str().expect("with subst id"))
        .expect("with subst trace");
    assert!(without["records"]
        .as_array()
        .expect("without records")
        .iter()
        .all(|record| record["document"]["substFont"].is_null()));
    assert!(with["records"]
        .as_array()
        .expect("with records")
        .iter()
        .any(|record| !record["document"]["substFont"].is_null()));
    assert_ne!(without["layoutHash"], with["layoutHash"]);
}

#[test]
fn stage4_public_trace_limit_and_unsupported_backends_fail_closed() {
    let manifest = stage4_e2e_manifest();
    let document = &manifest["documents"][2];
    let trace = public_trace(
        document["path"].as_str().expect("document path"),
        document["page"].as_u64().expect("page") as u32,
        1,
    );
    assert_eq!(trace["status"], "truncated");
    assert_eq!(trace["counts"]["recordsEmitted"], 1);
    assert!(trace["counts"]["recordsOmitted"].as_u64().unwrap() > 0);
    assert!(trace["reasons"]
        .as_array()
        .expect("reasons")
        .iter()
        .any(|reason| reason["code"] == "characterLimitExceeded"));
    for backend in ["canvas2d", "canvaskit"] {
        assert_eq!(trace["backendSummary"][backend]["status"], "unsupported");
        assert!(trace["records"]
            .as_array()
            .expect("records")
            .iter()
            .all(|record| record["paint"][backend]["failures"][0] == "studioSnapshotRequired"));
    }
}

#[test]
fn public_fixture_trace_is_bounded_and_deterministic() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let first = core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":8}"#)
        .expect("trace");
    let second = core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":8}"#)
        .expect("repeat trace");
    assert_eq!(
        first, second,
        "same document/page must serialize identically"
    );

    let value: serde_json::Value = serde_json::from_str(&first).expect("valid JSON");
    assert_eq!(value["schemaVersion"], 1);
    assert!(value["records"].as_array().unwrap().len() <= 8);
    assert_eq!(
        value["counts"]["recordsEmitted"],
        value["records"].as_array().unwrap().len()
    );
    for field in ["layoutHash", "normalizedHash"] {
        let digest = value[field]["value"].as_str().expect("digest");
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    for record in value["records"].as_array().unwrap() {
        assert_eq!(record["paint"]["native"]["status"], "unsupported");
        assert_eq!(
            record["paint"]["native"]["failures"][0],
            standalone_native_reason()
        );
        assert_eq!(record["paint"]["canvas2d"]["status"], "unsupported");
        assert_eq!(record["paint"]["canvaskit"]["status"], "unsupported");
    }
    assert!(value["records"]
        .as_array()
        .unwrap()
        .iter()
        .any(|record| record["source"]["status"] == "complete"));

    let ledger: serde_json::Value = serde_json::from_str(include_str!(
        "../../mydocs/tech/investigations/issue-4939/font_rule_ledger.json"
    ))
    .expect("W1 ledger JSON");
    let rules = ledger["rules"].as_array().expect("ledger rules");
    for provenance in value["records"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|record| record["provenance"].as_array().unwrap())
    {
        let Some(rule_id) = provenance["ruleId"].as_str() else {
            assert_eq!(provenance["reason"], "ledgerRuleMissing");
            continue;
        };
        let rule = rules
            .iter()
            .find(|rule| rule["ruleId"] == rule_id)
            .unwrap_or_else(|| panic!("trace rule missing from W1 ledger: {rule_id}"));
        assert_eq!(rule["sourceOwner"], provenance["sourceOwner"]);
        assert_eq!(rule["relationType"], provenance["relationType"]);
        assert_eq!(rule["evidenceStatus"], provenance["evidenceStatus"]);
        assert!(rule["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|evidence| evidence["reference"] == provenance["evidenceAnchor"]));
        assert_eq!(provenance["reason"], "ledgerSourceDrift");
    }
}

#[test]
fn issue4967_combined_evidence_uses_one_tree_and_preserves_trace() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/field-01.hwp"))
        .expect("public fixture parses");
    let options = r#"{"maxCharacters":4096}"#;
    let standalone: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, options)
            .expect("standalone trace"),
    )
    .expect("standalone trace JSON");

    rhwp::diagnostics::perf_counters::reset();
    let evidence: serde_json::Value = serde_json::from_str(
        &core
            .get_font_layout_evidence_native(0, options)
            .expect("same-snapshot evidence"),
    )
    .expect("evidence JSON");
    assert_eq!(rhwp::diagnostics::perf_counters::page_tree_builds(), 1);
    assert_eq!(evidence["scope"]["sameSnapshot"], true);
    assert_eq!(evidence["scope"]["pageTreeBuilds"], 1);
    assert_eq!(evidence["trace"], standalone);
    assert_eq!(evidence["status"], "complete");
    assert_eq!(evidence["counts"]["unframedRuns"], 0);
    assert_eq!(evidence["counts"]["runs"], evidence["counts"]["framedRuns"]);

    let lines = evidence["lines"].as_array().expect("line evidence");
    let mut memberships = std::collections::BTreeMap::<u64, usize>::new();
    for line in lines {
        for index in line["runIndices"].as_array().expect("run indices") {
            *memberships
                .entry(index.as_u64().expect("run index"))
                .or_default() += 1;
        }
    }
    assert_eq!(memberships.len() as u64, evidence["counts"]["runs"]);
    assert!(memberships.values().all(|count| *count == 1));
    for record in evidence["trace"]["records"]
        .as_array()
        .expect("trace records")
    {
        let run_index = record["source"]["runIndex"]
            .as_u64()
            .expect("trace run index");
        assert_eq!(memberships.get(&run_index), Some(&1));
    }
    assert!(lines
        .iter()
        .any(|line| line["storedRow"]["disposition"] == "admitted"));
}

#[test]
fn issue4967_combined_evidence_keeps_unowned_legacy_geometry_unmodelled() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample16.hwp"))
        .expect("public HWP3 fixture parses");
    let evidence: serde_json::Value = serde_json::from_str(
        &core
            .get_font_layout_evidence_native(0, r#"{"maxCharacters":4096}"#)
            .expect("same-snapshot HWP3 evidence"),
    )
    .expect("HWP3 evidence JSON");
    let lines = evidence["lines"].as_array().expect("line evidence");
    assert!(!lines.is_empty());
    assert!(lines.iter().all(|line| {
        line["storedRow"]["disposition"] == "unmodelled"
            && line["storedRow"]["reason"] == "frameProvenanceIncomplete"
    }));
}

#[test]
fn issue4967_combined_evidence_reports_actual_cache_key_rejection() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/rowbreak-problem-pages.hwp"))
        .expect("public rowbreak fixture parses");
    let evidence: serde_json::Value = serde_json::from_str(
        &core
            .get_font_layout_evidence_native(0, r#"{"maxCharacters":4096}"#)
            .expect("same-snapshot rowbreak evidence"),
    )
    .expect("rowbreak evidence JSON");
    assert!(evidence["lines"]
        .as_array()
        .expect("line evidence")
        .iter()
        .any(|line| {
            line["storedRow"]["disposition"] == "rejected"
                && line["storedRow"]["reason"] == "cacheKeyRejectedOrStale"
        }));
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn standalone_native_trace_requires_a_prepared_renderer_snapshot() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":8}"#)
            .expect("standalone trace"),
    )
    .expect("standalone trace JSON");
    assert_eq!(trace["backendSummary"]["native"]["status"], "unsupported");
    assert_eq!(
        trace["backendSummary"]["native"]["reasons"],
        serde_json::json!(["nativeRendererSnapshotRequired"])
    );
    assert!(trace["records"].as_array().unwrap().iter().all(|record| {
        record["paint"]["native"]["failures"][0] == "nativeRendererSnapshotRequired"
    }));
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn prepared_native_renderer_snapshot_preserves_custom_font_inventory() {
    use rhwp::renderer::skia::SkiaLayerRenderer;

    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    let custom_fonts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ttfs/opensource");
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[custom_fonts]);
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":64}"#)
            .expect("renderer-bound trace"),
    )
    .expect("renderer-bound trace JSON");
    assert_eq!(trace["backendSummary"]["native"]["status"], "complete");
    let custom_records: Vec<&serde_json::Value> = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["paint"]["native"]["source"] == "custom")
        .collect();
    assert!(
        !custom_records.is_empty(),
        "the prepared custom font inventory must be observable"
    );
    assert!(custom_records.iter().all(|record| {
        record["paint"]["native"]["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|capability| capability == "nativeGlyphCoverageObserved")
    }));
}

#[test]
fn trace_options_fail_closed() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    assert!(core
        .get_font_decision_trace_native(0, r#"{"unknown":1}"#)
        .is_err());
    assert!(core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":4097}"#)
        .is_err());
    assert!(core
        .get_font_decision_trace_native(core.page_count(), "{}")
        .is_err());
}
