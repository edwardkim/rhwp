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
        // #7445에 명시적으로 이관한 렌더링 절대핀만 생략한다.
        // 같은 문서의 폰트 프로필과 다른 문서의 핀은 계속 검사한다.
        let deferred_pins = document["renderPinsDeferredToIssue7445"].as_array();
        let is_render_pin_deferred = |name: &str| {
            deferred_pins.is_some_and(|pins| pins.iter().any(|pin| pin.as_str() == Some(name)))
        };
        if !is_render_pin_deferred("expectedCounts") {
            assert_eq!(trace["counts"], document["expectedCounts"], "{id}");
        }
        if !is_render_pin_deferred("expectedLayoutHash") {
            assert_eq!(
                trace["layoutHash"]["value"], document["expectedLayoutHash"],
                "{id}"
            );
        }
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

    let original =
        DocumentCore::from_bytes(include_bytes!("../../samples/3-10월_교육_통합_2022.hwp"))
            .unwrap();
    // 해시와 별개로 독립 원문의 모든 글자 주소를 확인한다. 미주 표시는 원문 글자가 아니다.
    for paragraph in [42, 48, 57, 81] {
        let expected: Vec<_> = original.document().sections[0].paragraphs[paragraph]
            .text
            .chars()
            .enumerate()
            .map(|(offset, ch)| (offset as u64, ch.to_string()))
            .collect();
        let actual: Vec<_> = hwp["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["source"]["sectionIndex"] == 0
                    && r["source"]["paragraphIndex"] == paragraph
                    && r["source"]["nestedPath"] == serde_json::json!([])
                    && r["source"]["charOffset"].is_number()
            })
            .map(|r| {
                assert_eq!(r["source"]["status"], "complete");
                (
                    r["source"]["charOffset"].as_u64().unwrap(),
                    r["source"]["character"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        assert_eq!(actual, expected, "공개 원문 문단 {paragraph}");
    }

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

    rhwp::diagnostics::perf_counters::reset_thread_page_tree_builds();
    std::thread::spawn(|| {
        let parallel_core = DocumentCore::from_bytes(include_bytes!("../../samples/field-01.hwp"))
            .expect("parallel public fixture parses");
        rhwp::diagnostics::perf_counters::reset_thread_page_tree_builds();
        parallel_core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
            .expect("parallel standalone trace");
        assert_eq!(
            rhwp::diagnostics::perf_counters::thread_page_tree_builds(),
            1
        );
    })
    .join()
    .expect("parallel counter probe joins");
    assert_eq!(
        rhwp::diagnostics::perf_counters::thread_page_tree_builds(),
        0
    );
    let evidence: serde_json::Value = serde_json::from_str(
        &core
            .get_font_layout_evidence_native(0, options)
            .expect("same-snapshot evidence"),
    )
    .expect("evidence JSON");
    assert_eq!(
        rhwp::diagnostics::perf_counters::thread_page_tree_builds(),
        1
    );
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

fn readonly_trace(core: &DocumentCore) -> serde_json::Value {
    let model = format!("{:?}", core.document());
    let svg = core.render_page_svg_native(0).unwrap();
    let hwp = core.export_hwp_native().unwrap();
    let hwpx = core.export_hwpx_native().unwrap();
    let trace = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(format!("{:?}", core.document()), model);
    assert_eq!(core.render_page_svg_native(0).unwrap(), svg);
    assert_eq!(core.export_hwp_native().unwrap(), hwp);
    assert_eq!(core.export_hwpx_native().unwrap(), hwpx);
    trace
}

#[test]
fn logical_tac_runs_join_exact_scalar_source_in_body_and_nested_cells() {
    use rhwp::model::control::Control;
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가🦦나나끝").unwrap();
    core.apply_char_format_native(0, 0, 0, 1, r#"{"bold":true}"#)
        .unwrap();
    core.apply_char_format_native(0, 0, 2, 4, r#"{"fontSize":1800}"#)
        .unwrap();
    for _ in 0..2 {
        let image: serde_json::Value = serde_json::from_str(
            &core
                .insert_picture_native(
                    0,
                    0,
                    1,
                    &[],
                    include_bytes!("../../assets/logo/logo-16.png"),
                    1200,
                    400,
                    16,
                    16,
                    "png",
                    "동일 자리 그림",
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
        core.set_picture_properties_native(
            0,
            0,
            image["controlIdx"].as_u64().unwrap() as usize,
            r#"{"treatAsChar":true}"#,
        )
        .unwrap();
    }
    let leaf = core.document().sections[0].paragraphs[0].clone();
    assert_eq!(
        leaf.control_text_positions()
            .iter()
            .copied()
            .filter(|p| *p == 1)
            .count(),
        2
    );
    core.split_paragraph_native(0, 0, 7, None).unwrap();
    core.create_table_native(0, 1, 0, 1, 1).unwrap();
    let mut document = core.document().clone();
    let parent = document.sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .unwrap();
    let outer_ci = document.sections[0].paragraphs[parent]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    let mut host = document.sections[0].paragraphs[parent].clone();
    let Control::Table(outer) = &mut document.sections[0].paragraphs[parent].controls[outer_ci]
    else {
        unreachable!()
    };
    let mut inner = outer.clone();
    inner.cells[0].paragraphs = vec![leaf.clone()];
    host.controls = vec![Control::Table(inner)];
    host.text.clear();
    host.char_offsets.clear();
    outer.cells[0].paragraphs = vec![
        rhwp::model::paragraph::Paragraph::new_empty_like(&leaf),
        host,
    ];
    document.sections[0].raw_stream = None;
    core.set_document(document);
    // 원문과 그림 두 칸은 다른 축이다. 보조평면 문자도 원문에서 한 scalar다.
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(doc.logical_to_text_offset(0, 0, 3).unwrap(), 1);
    assert_eq!(doc.logical_to_text_offset(0, 0, 4).unwrap(), 2);
    for (format, bytes) in [
        ("hwp", core.export_hwp_native().unwrap()),
        ("hwpx", core.export_hwpx_native().unwrap()),
    ] {
        let loaded = DocumentCore::from_bytes(&bytes).unwrap();

        let trace = readonly_trace(&loaded);
        for (para, path) in [(0, vec![]), (parent, vec![outer_ci, 0, 1, 0, 0, 0])] {
            let records: Vec<_> = trace["records"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| {
                    r["source"]["sectionIndex"] == 0
                        && r["source"]["paragraphIndex"] == para
                        && r["source"]["nestedPath"] == serde_json::json!(path)
                })
                .collect();
            let expected: Vec<_> = "가🦦나나끝"
                .chars()
                .enumerate()
                .map(|(offset, ch)| (offset as u64, ch.to_string()))
                .collect();
            let actual: Vec<_> = records
                .iter()
                .map(|r| {
                    assert_eq!(r["source"]["status"], "complete", "{format}: {r}");
                    (
                        r["source"]["charOffset"].as_u64().unwrap(),
                        r["source"]["character"].as_str().unwrap().to_string(),
                    )
                })
                .collect();
            assert_eq!(actual, expected, "{format} source path {path:?}");
        }
    }
}

#[test]
fn synthetic_overlap_cannot_claim_repeated_source_and_real_pua_keeps_source() {
    use rhwp::model::control::{CharOverlap, Control};
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가가").unwrap();
    let mut document = core.document().clone();
    let para = &mut document.sections[0].paragraphs[0];
    para.controls = vec![Control::CharOverlap(CharOverlap {
        chars: vec!['가'],
        border_type: 1,
        inner_char_size: 100,
        expansion: 0,
        char_shape_ids: vec![0],
    })];
    para.char_offsets = vec![0, 9];
    para.char_count = 11;
    document.sections[0].raw_stream = None;
    core.set_document(document);
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(trace["counts"]["charactersSeen"], 3);
    let records = trace["records"].as_array().unwrap();
    assert_eq!(records[0]["source"]["status"], "complete");
    assert_eq!(records[0]["source"]["charOffset"], 0);
    for record in &records[1..] {
        assert_eq!(
            record["source"]["status"], "unavailable",
            "겹침 원문 소유권을 추정하지 않는다: {record}"
        );
        assert!(record["source"]["charOffset"].is_null());
    }
    let mut plain = DocumentCore::new_empty();
    plain.create_blank_document_native().unwrap();
    plain.insert_text_native(0, 0, 0, "가\u{f02ce}끝").unwrap();
    let trace: serde_json::Value = serde_json::from_str(
        &plain
            .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let records = trace["records"].as_array().unwrap();
    assert_eq!(records.len(), 3);
    for (offset, record) in records.iter().enumerate() {
        assert_eq!(
            record["source"]["status"], "complete",
            "PUA는 실제 원문이다: {record}"
        );
        assert_eq!(record["source"]["charOffset"], offset);
    }
}

#[test]
fn note_markers_are_transparent_to_body_and_nested_source_glyph_addresses() {
    use rhwp::model::control::Control;
    for (footnote, at, has_tac) in [
        (true, 0, false),
        (true, 1, false),
        (false, 0, false),
        (false, 1, false),
        (true, 0, true),
        (true, 1, true),
        (false, 0, true),
        (false, 1, true),
    ] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "가가🦦나끝").unwrap();
        if footnote {
            core.insert_footnote_native(0, 0, at).unwrap();
        } else {
            core.insert_endnote_native(0, 0, at).unwrap();
        }
        if has_tac {
            let image: serde_json::Value = serde_json::from_str(
                &core
                    .insert_picture_native(
                        0,
                        0,
                        at,
                        &[],
                        include_bytes!("../../assets/logo/logo-16.png"),
                        1200,
                        400,
                        16,
                        16,
                        "png",
                        "주석과 같은 자리",
                        None,
                        None,
                    )
                    .unwrap(),
            )
            .unwrap();
            core.set_picture_properties_native(
                0,
                0,
                image["controlIdx"].as_u64().unwrap() as usize,
                r#"{"treatAsChar":true}"#,
            )
            .unwrap();
        }
        core.apply_char_format_native(0, 0, 1, 3, r#"{"bold":true}"#)
            .unwrap();
        let leaf = core.document().sections[0].paragraphs[0].clone();
        core.split_paragraph_native(0, 0, if has_tac { 7 } else { 6 }, None)
            .unwrap();
        core.create_table_native(0, 1, 0, 1, 1).unwrap();
        let mut document = core.document().clone();
        let mut host = document.sections[0].paragraphs[1].clone();
        let Control::Table(outer) = &mut document.sections[0].paragraphs[1].controls[0] else {
            panic!("표")
        };
        let mut inner = outer.clone();
        inner.cells[0].paragraphs = vec![leaf.clone()];
        host.controls = vec![Control::Table(inner)];
        outer.cells[0].paragraphs = vec![
            rhwp::model::paragraph::Paragraph::new_empty_like(&leaf),
            host,
        ];
        document.sections[0].raw_stream = None;
        core.set_document(document);
        for (format, bytes) in [
            ("hwp", core.export_hwp_native().unwrap()),
            ("hwpx", core.export_hwpx_native().unwrap()),
        ] {
            let loaded = DocumentCore::from_bytes(&bytes).unwrap();

            let tree = loaded.build_page_render_tree(0).unwrap();
            let trace = readonly_trace(&loaded);
            // 런 순서는 따르되 원문 주소는 독립 원문의 scalar 순서로 정한다.
            let mut runs = Vec::new();
            fn collect<'a>(
                node: &'a rhwp::renderer::render_tree::RenderNode,
                framed: bool,
                runs: &mut Vec<(&'a rhwp::renderer::render_tree::TextRunNode, bool)>,
            ) {
                use rhwp::renderer::render_tree::RenderNodeType;
                let framed = framed || matches!(node.node_type, RenderNodeType::TextLine(_));
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    runs.push((run, framed));
                }
                for child in &node.children {
                    collect(child, framed, runs);
                }
            }
            collect(&tree.root, false, &mut runs);
            let records = trace["records"].as_array().unwrap();
            for (parent, path) in [(0, vec![]), (1, vec![0, 0, 1, 0, 0, 0])] {
                let expected: Vec<_> = "가가🦦나끝"
                    .chars()
                    .enumerate()
                    .map(|(i, ch)| (i as u64, ch.to_string()))
                    .collect();
                let actual: Vec<_> = records
                    .iter()
                    .filter(|r| {
                        let index = r["source"]["runIndex"].as_u64().unwrap() as usize;
                        let (run, framed) = runs[index];
                        framed
                            && run.char_start.is_some()
                            && r["source"]["sectionIndex"] == 0
                            && r["source"]["paragraphIndex"] == parent
                            && r["source"]["nestedPath"] == serde_json::json!(path)
                    })
                    .map(|r| {
                        assert_eq!(
                            r["source"]["status"], "complete",
                            "{format} footnote={footnote} at={at}: {r}"
                        );
                        (
                            r["source"]["charOffset"].as_u64().unwrap(),
                            r["source"]["character"].as_str().unwrap().to_string(),
                        )
                    })
                    .collect();
                assert_eq!(
                    actual, expected,
                    "{format} footnote={footnote} at={at} path={path:?}"
                );
            }
            for record in records {
                let (run, framed) = runs[record["source"]["runIndex"].as_u64().unwrap() as usize];
                if !framed
                    || run.char_start.is_none()
                    || record["source"]["paragraphIndex"].is_null()
                {
                    assert_eq!(record["source"]["status"], "unavailable");
                    assert!(record["source"]["charOffset"].is_null());
                }
            }
        }
    }
}

#[test]
fn coanchored_logical_copy_preserves_endpoint_inclusion_and_source_in_both_formats() {
    use rhwp::model::control::Control;
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나").unwrap();
    for _ in 0..2 {
        let image: serde_json::Value = serde_json::from_str(
            &core
                .insert_picture_native(
                    0,
                    0,
                    1,
                    &[],
                    include_bytes!("../../assets/logo/logo-16.png"),
                    1200,
                    400,
                    16,
                    16,
                    "png",
                    "같은 자리",
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
        core.set_picture_properties_native(
            0,
            0,
            image["controlIdx"].as_u64().unwrap() as usize,
            r#"{"treatAsChar":true}"#,
        )
        .unwrap();
    }
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        for (start, end, text, controls) in [(1, 3, "", 2), (0, 1, "가", 0), (3, 4, "나", 0)] {
            let mut loaded = DocumentCore::from_bytes(&bytes).unwrap();
            let before_model = format!("{:?}", loaded.document());
            let before_svg = loaded.render_page_svg_native(0).unwrap();
            let before_hwp = loaded.export_hwp_native().unwrap();
            let before_hwpx = loaded.export_hwpx_native().unwrap();
            loaded
                .copy_selection_logical_native(0, 0, start, 0, end)
                .unwrap();
            assert_eq!(loaded.get_clipboard_text_native(), text);
            assert_eq!(format!("{:?}", loaded.document()), before_model);
            assert_eq!(loaded.render_page_svg_native(0).unwrap(), before_svg);
            assert_eq!(loaded.export_hwp_native().unwrap(), before_hwp);
            assert_eq!(loaded.export_hwpx_native().unwrap(), before_hwpx);
            let source = format!("{:?}", loaded.document().sections[0].paragraphs[0]);
            loaded.paste_internal_native(0, 1, 0).unwrap();
            assert_eq!(
                format!("{:?}", loaded.document().sections[0].paragraphs[0]),
                source
            );
            let pasted: Vec<_> = loaded.document().sections[0].paragraphs[1..]
                .iter()
                .collect();
            assert_eq!(
                pasted.iter().map(|p| p.text.as_str()).collect::<String>(),
                text
            );
            assert_eq!(
                pasted
                    .iter()
                    .flat_map(|p| &p.controls)
                    .filter(|c| matches!(c, Control::Picture(_)))
                    .count(),
                controls
            );
        }
    }
}

#[test]
fn inline_table_endnote_prefix_cannot_alias_its_body_paragraph() {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "원문").unwrap();
    core.insert_endnote_native(0, 0, 0).unwrap();
    let leaf = core.document().sections[0].paragraphs[0].clone();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.create_table_native(0, 1, 0, 1, 1).unwrap();
    let mut document = core.document().clone();
    let mut parent = document.sections[0].paragraphs[1].clone();
    let mut host = parent.clone();
    let Control::Table(outer) = &mut parent.controls[0] else {
        panic!("표")
    };
    let mut inner = outer.clone();
    inner.raw_ctrl_data.clear();
    inner.raw_ctrl_seal = None;
    inner.common.treat_as_char = true;
    inner.attr |= 1;
    inner.common.width = 15000;
    inner.common.height = 6000;
    inner.cells[0].width = 15000;
    inner.cells[0].height = 6000;
    inner.cells[0].paragraphs = vec![leaf.clone()];
    host.controls = vec![Control::Table(inner)];
    host.text = "앞뒤".into();
    host.char_offsets = vec![0, 9];
    host.char_count = 11;
    outer.cells[0].paragraphs = vec![
        rhwp::model::paragraph::Paragraph::new_empty_like(&leaf),
        host,
    ];
    outer.raw_ctrl_data.clear();
    outer.raw_ctrl_seal = None;
    outer.cells[0].height = 18000;
    outer.common.height = 18000;
    // 셀의 미주 표시와 본문이 같아도 두 원문의 소유자는 다르다.
    parent.text = "1) ".into();
    parent.char_offsets = vec![0, 1, 2];
    parent.char_count = 12;
    document.sections[0].paragraphs = vec![parent];
    document.sections[0].raw_stream = None;
    core.set_document(document);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let loaded = DocumentCore::from_bytes(&bytes).unwrap();

        let table_ci = loaded.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        let Control::Table(table) = &loaded.document().sections[0].paragraphs[0].controls[table_ci]
        else {
            unreachable!()
        };
        assert!(!table.common.treat_as_char);
        let Control::Table(inner) = &table.cells[0].paragraphs[1].controls[0] else {
            panic!("중첩 표")
        };
        assert!(inner.common.treat_as_char);
        let tree = loaded.build_page_render_tree(0).unwrap();
        let mut runs: Vec<&TextRunNode> = Vec::new();
        let mut prefixes: Vec<&TextRunNode> = Vec::new();
        fn collect<'a>(
            node: &'a RenderNode,
            runs: &mut Vec<&'a TextRunNode>,
            prefixes: &mut Vec<&'a TextRunNode>,
        ) {
            if let RenderNodeType::TableCell(_) = node.node_type {
                for child in &node.children {
                    if let RenderNodeType::TextRun(run) = &child.node_type {
                        if run.cell_context.is_none() && !run.text.is_empty() {
                            prefixes.push(run);
                        }
                    }
                }
            }
            if let RenderNodeType::TextRun(run) = &node.node_type {
                runs.push(run);
            }
            for child in &node.children {
                collect(child, runs, prefixes);
            }
        }
        collect(&tree.root, &mut runs, &mut prefixes);
        assert_eq!(prefixes.len(), 1);
        assert_eq!(prefixes[0].text, "1) ");
        let prefix_index = runs
            .iter()
            .position(|run| std::ptr::eq(*run, prefixes[0]))
            .unwrap();
        let trace = readonly_trace(&loaded);
        let records = trace["records"].as_array().unwrap();
        let prefix_records: Vec<_> = records
            .iter()
            .filter(|r| r["source"]["runIndex"] == prefix_index)
            .collect();
        assert_eq!(prefix_records.len(), 3);
        for record in prefix_records {
            assert_eq!(
                record["source"]["status"], "unavailable",
                "미주 표시의 본문 alias: {record}"
            );
            assert!(record["source"]["charOffset"].is_null());
        }
        for (path, text) in [
            (vec![], "1) "),
            (vec![table_ci, 0, 1], "앞뒤"),
            (vec![table_ci, 0, 1, 0, 0, 0], "원문"),
        ] {
            let actual: Vec<_> = records
                .iter()
                .filter(|r| {
                    r["source"]["status"] == "complete"
                        && r["source"]["nestedPath"] == serde_json::json!(path)
                })
                .map(|r| {
                    (
                        r["source"]["charOffset"].as_u64().unwrap(),
                        r["source"]["character"].as_str().unwrap().to_string(),
                    )
                })
                .collect();
            let expected: Vec<_> = text
                .chars()
                .enumerate()
                .map(|(i, ch)| (i as u64, ch.to_string()))
                .collect();
            assert_eq!(actual, expected, "path={path:?}");
        }
    }
}
