use rhwp::document_core::DocumentCore;

mod corpus_scope {
    include!("../common/corpus_scope.rs");
}

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

include!("../common/font_trace_profile_assertions.rs");

#[test]
fn font_trace_profiles_follow_source_ownership_and_preserve_font_decisions() {
    let manifest = stage4_e2e_manifest();
    let max_characters = manifest["options"]["maxCharacters"]
        .as_u64()
        .expect("maxCharacters");
    let mut traces = std::collections::HashMap::new();
    for document in manifest["documents"].as_array().expect("documents") {
        // 사용자가 별도 작업으로 이관한 시험지의 프로필·렌더 핀은 제거한다.
        if !corpus_scope::sample_is_in_scope(std::path::Path::new(
            document["path"].as_str().expect("document path"),
        )) {
            continue;
        }
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
            let expected = document["expectedCounts"]
                .as_object()
                .expect("expectedCounts");
            for (key, value) in expected {
                assert_eq!(&trace["counts"][key], value, "{id}: {key}");
            }
            if document["validateRunOwnership"] == true {
                assert_profile_run_ownership(document, &trace);
            }
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
    // #8109: 대응하는 HWP에도 표준 FACE_NAME 대체 항목이 있다.
    // 독립 DocInfo 바이트에는 대체 레코드 217개가 있지만 정확한 글꼴을 지정한
    // 검증 문서에는 없다. 이 메타데이터 차이는 렌더 위치 검사와 구분한다.
    let counterpart = traces
        .get(
            feature["withHwpSubstFont"]
                .as_str()
                .expect("HWP counterpart id"),
        )
        .expect("HWP counterpart trace");
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
    assert!(counterpart["records"]
        .as_array()
        .expect("HWP alternative records")
        .iter()
        .any(|record| record["document"]["substFont"] == feature["expectedHwpSubstFont"]));
    assert_ne!(counterpart["layoutHash"], with["layoutHash"]);
    // HWP5 표 20의 0x80 선언과 HWPX substFont를 각각 보존한다.
    // HWP에도 대체 선언이 있으므로 형식만으로 null을 기대하지 않는다.
    for declaration in manifest["comparisons"]["documentSubstituteDeclarations"]
        .as_array()
        .expect("문서 대체 선언")
    {
        let trace = traces
            .get(declaration["documentId"].as_str().expect("선언 문서 ID"))
            .expect("선언 문서 trace");
        let slots = declaration["languageSlots"].as_array().expect("언어 슬롯");
        let records: Vec<_> = trace["records"]
            .as_array()
            .expect("선언 records")
            .iter()
            .filter(|record| {
                record["document"]["face"] == declaration["face"]
                    && slots.contains(&record["document"]["languageSlot"])
            })
            .collect();
        assert!(!records.is_empty(), "선언 대상 글꼴 record가 있어야 한다");
        assert!(records
            .iter()
            .all(|record| { record["document"]["substFont"] == declaration["substFont"] }));
    }
}

#[test]
fn stage4_public_trace_limit_and_unsupported_backends_fail_closed() {
    let manifest = stage4_e2e_manifest();
    let document = manifest["documents"]
        .as_array()
        .expect("공개 문서")
        .iter()
        .find(|document| document["id"] == "exact-face")
        .expect("exact-face 입력 유지");
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

    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public fixture parses");
    // 이 검사는 배치 정답지가 아니라 준비된 renderer의 inventory 계약이다.
    // 원본 face가 시스템에 설치됐는지에 따라 fallback이 달라지지 않도록,
    // 저장소에서 실제 공급하는 custom face를 요청한 합성 대조군으로 조회한다.
    let mut source = core.document().clone();
    for face in source.doc_info.font_faces.iter_mut().flatten() {
        face.name = "Noto Sans KR".into();
    }
    core.set_document(source);
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
        record["paint"]["native"]["resolved"] == "Noto Sans KR"
            && record["paint"]["native"]["capabilities"]
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
        hwp_wire_chars: false,
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
