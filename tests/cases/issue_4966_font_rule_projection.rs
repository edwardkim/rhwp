use std::collections::{BTreeMap, BTreeSet};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::font_metrics_data::find_metric;

fn sealed_v1_registry() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../assets/font-rules/font_rule_registry.json"
    ))
    .expect("sealed v1 font rule registry")
}

fn registry() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../assets/font-rules/font_rule_registry_v2.json"
    ))
    .expect("canonical v2 font rule lifecycle registry")
}

fn projection_rules<'a>(
    registry: &'a serde_json::Value,
    projection: &str,
) -> Vec<&'a serde_json::Value> {
    let mut rules: Vec<_> = registry["rules"]
        .as_array()
        .expect("registry rules")
        .iter()
        .filter(|rule| {
            rule["status"] == "active"
                && rule["projections"]
                    .as_array()
                    .expect("rule projections")
                    .iter()
                    .any(|entry| entry["id"] == projection)
        })
        .collect();
    rules.sort_by_key(|rule| {
        rule["projectionSequence"]
            .as_u64()
            .expect("active v2 projection sequence")
    });
    rules
}

fn source_boundary(rule: &serde_json::Value) -> &str {
    rule["sourceBoundaryId"]
        .as_str()
        .expect("v2 source boundary")
}

fn semantic_rule(rule: &serde_json::Value, source_boundary_id: &str) -> serde_json::Value {
    serde_json::json!({
        "conditions": rule["conditions"],
        "decisionPlane": rule["decisionPlane"],
        "metricEntryIds": rule["metricEntryIds"],
        "order": rule["order"],
        "projections": rule["projections"],
        "relationType": rule["relationType"],
        "ruleId": rule["ruleId"],
        "sourceBoundaryId": source_boundary_id,
        "sourceFace": rule["sourceFace"],
        "status": rule["status"],
        "supply": rule["supply"],
        "targetFaceOrPolicy": rule["targetFaceOrPolicy"],
    })
}

fn source_face(rule: &serde_json::Value) -> &str {
    rule["sourceFace"].as_str().expect("source face")
}

fn language_slot(rule: &serde_json::Value) -> &str {
    rule["conditions"]["languageSlot"]
        .as_str()
        .expect("language slot")
}

fn public_blank_with_font_text(font_name: &str, alt_type: u8, text: &str) -> DocumentCore {
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
            alt_type,
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
    let char_shape_id =
        u32::try_from(document.doc_info.char_shapes.len()).expect("char shape fixture id");
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

fn public_trace_record(font_name: &str, alt_type: u8, text: &str) -> serde_json::Value {
    let core = public_blank_with_font_text(font_name, alt_type, text);
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
            .expect("public font decision trace"),
    )
    .expect("font decision trace JSON");
    assert_eq!(trace["status"], "complete");
    trace["records"]
        .as_array()
        .expect("trace records")
        .iter()
        .find(|record| record["source"]["character"] == text)
        .unwrap_or_else(|| panic!("trace record for {font_name:?}/{text:?}"))
        .clone()
}

#[test]
fn sealed_v1_semantics_change_only_by_recorded_replacement() {
    let v1 = sealed_v1_registry();
    let v2 = registry();
    let v1_rules = v1["rules"].as_array().expect("sealed v1 rules");
    let v2_rules = v2["rules"].as_array().expect("current v2 rules");

    assert_eq!(v1_rules.len(), 830);
    assert_eq!(v2["summary"]["activeRuleCount"], 846);
    assert_eq!(v2["summary"]["retiredRuleCount"], 4);

    let mut expected_semantics: BTreeMap<_, _> = v1_rules
        .iter()
        .map(|rule| {
            let rule_id = rule["ruleId"].as_str().expect("sealed v1 ruleId");
            let source_boundary_id = rule["evidence"]["sourceBoundaryIds"][0]
                .as_str()
                .expect("sealed v1 source boundary");
            (rule_id.to_owned(), semantic_rule(rule, source_boundary_id))
        })
        .collect();
    // v1 봉인은 변경하지 않는다. 독립적으로 기록된 교체와 추가만 활성 의미에 반영한다.
    let changes: [serde_json::Value; 4] = [
        serde_json::from_str(include_str!(
            "../../assets/font-rules/changes/issue-7023-batangche-serif-supply.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../assets/font-rules/changes/issue-6936-new-gulim-face-identity.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../assets/font-rules/changes/issue-4969-oldhangul-license-webfont.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../assets/font-rules/changes/issue-4969-oldhangul-license-canvaskit.json"
        ))
        .unwrap(),
    ];
    let mut replacements = BTreeMap::new();
    for change in &changes {
        assert_eq!(change["operations"].as_array().unwrap().len(), 1);
        let operation = &change["operations"][0];
        assert_eq!(operation["type"], "retire-and-replace");
        let retired_id = operation["retiredRuleId"].as_str().unwrap();
        let mut replacement = operation["replacementRule"].clone();
        replacement["status"] = "active".into();
        replacement["projections"] = serde_json::json!([replacement["projection"]]);
        let replacement_id = replacement["ruleId"].as_str().unwrap();
        let retired = v2_rules
            .iter()
            .find(|rule| rule["ruleId"] == retired_id)
            .unwrap();
        assert_eq!(retired["status"], "retired");
        let mut historical_semantics = semantic_rule(retired, source_boundary(retired));
        historical_semantics["status"] = "active".into();
        assert_eq!(
            expected_semantics.remove(retired_id),
            Some(historical_semantics)
        );
        assert!(expected_semantics
            .insert(
                replacement_id.to_owned(),
                semantic_rule(&replacement, source_boundary(&replacement)),
            )
            .is_none());
        assert!(replacements
            .insert(retired_id.to_owned(), replacement_id.to_owned())
            .is_none());
    }
    let mut added_by_projection: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (payload, projection) in [
        (
            include_str!("../../assets/font-rules/changes/issue-7196-ygodic230-metric-name.json"),
            "rust-layout-metric",
        ),
        (
            include_str!("../../assets/font-rules/changes/issue-5884-noto-symbols2-webfont.json"),
            "canvas2d-webfont",
        ),
        (
            include_str!("../../assets/font-rules/changes/issue-5884-noto-symbols2-canvaskit.json"),
            "canvaskit-sfnt",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7958-h2hdrm-postscript-identity.json"
            ),
            "rust-layout-name",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7959-malgun-postscript-identity.json"
            ),
            "rust-layout-name",
        ),
        (
            include_str!("../../assets/font-rules/changes/issue-7966-postscript-layout-name.json"),
            "rust-layout-name",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7966-postscript-layout-metric.json"
            ),
            "rust-layout-metric",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7445-public-batang-metric-name.json"
            ),
            "rust-layout-metric",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7445-public-batang-webfont-supply.json"
            ),
            "canvas2d-webfont",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7445-public-batang-canvaskit-supply.json"
            ),
            "canvaskit-sfnt",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7445-missing-title-public-fallback.json"
            ),
            "rust-layout-name",
        ),
        (
            include_str!(
                "../../assets/font-rules/changes/issue-7445-document-substitute-lookup.json"
            ),
            "rust-layout-name",
        ),
    ] {
        // 공개 프로그램 공급 두 건도 기록된 add-rule만 의미 비교에 반영한다.
        let addition: serde_json::Value =
            serde_json::from_str(payload).expect("기록된 폰트 규칙 추가");
        assert_eq!(addition["expectedDelta"]["projectionId"], projection);
        let operations = addition["operations"].as_array().expect("추가 작업");
        assert_eq!(
            addition["expectedDelta"]["activeRuleDelta"],
            operations.len()
        );
        for operation in operations {
            assert_eq!(operation["type"], "add-rule");
            let mut added = operation["rule"].clone();
            added["status"] = "active".into();
            added["projections"] = serde_json::json!([added["projection"]]);
            let added_id = added["ruleId"].as_str().expect("추가 규칙 ID").to_owned();
            assert!(expected_semantics
                .insert(
                    added_id.clone(),
                    semantic_rule(&added, source_boundary(&added))
                )
                .is_none());
            added_by_projection
                .entry(projection.to_owned())
                .or_default()
                .push(added_id);
        }
    }
    let v2_semantics: BTreeMap<_, _> = v2_rules
        .iter()
        .filter(|rule| rule["status"] == "active")
        .map(|rule| {
            let rule_id = rule["ruleId"].as_str().expect("current v2 ruleId");
            (
                rule_id.to_owned(),
                semantic_rule(rule, source_boundary(rule)),
            )
        })
        .collect();
    assert_eq!(v2_semantics, expected_semantics);

    for projection in [
        "rust-layout-name",
        "rust-layout-metric",
        "canvas2d-paint",
        "canvas2d-webfont",
        "canvaskit-sfnt",
    ] {
        let mut v1_rule_ids: Vec<_> = v1_rules
            .iter()
            .filter(|rule| {
                rule["projections"]
                    .as_array()
                    .expect("sealed v1 projections")
                    .iter()
                    .any(|entry| entry["id"] == projection)
            })
            .map(|rule| {
                let id = rule["ruleId"].as_str().expect("sealed v1 ruleId");
                replacements.get(id).map(String::as_str).unwrap_or(id)
            })
            .collect();
        if let Some(additions) = added_by_projection.get(projection) {
            v1_rule_ids.extend(additions.iter().map(String::as_str));
        }
        let v2_rule_ids: Vec<_> = projection_rules(&v2, projection)
            .into_iter()
            .map(|rule| rule["ruleId"].as_str().expect("current v2 ruleId"))
            .collect();
        assert_eq!(v2_rule_ids, v1_rule_ids, "{projection}");
    }
}

#[test]
fn canonical_layout_name_projection_reaches_public_trace() {
    let registry = registry();
    let rules = projection_rules(&registry, "rust-layout-name");
    assert_eq!(rules.len(), 179);

    let legacy_keys: BTreeSet<(&str, &str)> = rules
        .iter()
        .filter(|rule| source_boundary(rule) == "rust-style-resolution.legacy-latin")
        .map(|rule| (source_face(rule), language_slot(rule)))
        .collect();
    let mut observed = BTreeSet::new();
    let mut priority_shadowed = BTreeSet::new();

    for rule in rules {
        let boundary = source_boundary(rule);
        let slot = language_slot(rule);
        if boundary == "rust-style-resolution.hft"
            && slot == "1"
            && legacy_keys.contains(&(source_face(rule), slot))
        {
            priority_shadowed.insert(rule["ruleId"].as_str().expect("ruleId"));
            continue;
        }

        if boundary == "rust-style-resolution.document-substitute" {
            use rhwp::model::style::SubstFont;
            use rhwp::renderer::font_environment::FontEnvironment;
            assert_eq!(slot, "all");
            assert_eq!(rule["conditions"]["altType"], "1");
            assert_eq!(
                rule["conditions"]["availability"],
                "nonembedded-ttf-document-substitute"
            );
            // 독립 Print에서 확인한 대체 필드의 영문 이름 경로다.
            // 원본 이름에 적용하는 일반 alias와 구분해 공개 API를 통과시킨다.
            let source = source_face(rule);
            let target = rule["targetFaceOrPolicy"].as_str().unwrap();
            let mut core = public_blank_with_font_text("__font_rule_missing_primary__", 1, "가");
            let mut document = core.document().clone();
            for font in document.doc_info.font_faces.iter_mut().flatten() {
                font.subst_font = Some(SubstFont {
                    face: source.to_owned(),
                    font_type: 1,
                    ..Default::default()
                });
            }
            core.set_document(document);
            let programs = serde_json::json!([
                {"query":target,"family":target,"programId":"substitute-target","trueType":true},
                {"query":source,"family":source,"programId":"primary-name-control","trueType":true}
            ]);
            let selected = core
                .build_font_environment_from_available_programs(&programs.to_string())
                .unwrap();
            core.set_font_environment(Some(FontEnvironment::from_json(&selected).unwrap()))
                .unwrap();
            let trace: serde_json::Value = serde_json::from_str(
                &core
                    .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                    .unwrap(),
            )
            .unwrap();
            let record = trace["records"]
                .as_array()
                .unwrap()
                .iter()
                .find(|record| record["source"]["character"] == "가")
                .unwrap();
            assert_eq!(record["document"]["substFont"], source);
            assert_eq!(record["layoutName"]["normalizedFace"], target);
            assert!(record["layoutName"]["steps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|step| step["kind"] == "documentSubstituteLookup"
                    && step["input"] == source
                    && step["output"] == target));
            assert!(record["provenance"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["ruleId"] == rule["ruleId"]));
            assert_eq!(
                public_trace_record(source, 1, "가")["layoutName"]["normalizedFace"],
                source,
                "원본 이름은 문서 대체 필드 규칙의 대상이 아니다"
            );
            observed.insert(rule["ruleId"].as_str().unwrap());
            continue;
        }
        if boundary == "rust-style-resolution.missing-program" {
            use rhwp::renderer::font_environment::FontEnvironment;
            assert_eq!(slot, "all");
            assert_eq!(
                rule["conditions"]["availability"],
                "primary-and-document-substitute-missing"
            );
            let source = source_face(rule);
            let target = rule["targetFaceOrPolicy"].as_str().unwrap();
            let mut core = public_blank_with_font_text(source, 2, "가");
            let programs = serde_json::json!([{
                "query":target,"family":target,"programId":"canonical-conditional-protocol-control","trueType":true
            }]);
            let selected = core
                .build_font_environment_from_available_programs(&programs.to_string())
                .unwrap();
            let environment: serde_json::Value = serde_json::from_str(&selected).unwrap();
            assert_eq!(environment["selectionRules"][source], rule["ruleId"]);
            core.set_font_environment(Some(FontEnvironment::from_json(&selected).unwrap()))
                .unwrap();
            let trace: serde_json::Value = serde_json::from_str(
                &core
                    .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                    .unwrap(),
            )
            .unwrap();
            let record = trace["records"]
                .as_array()
                .unwrap()
                .iter()
                .find(|record| record["source"]["character"] == "가")
                .unwrap();
            assert_eq!(record["layoutName"]["normalizedFace"], target);
            assert!(record["provenance"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["ruleId"] == rule["ruleId"]));
            let primary = serde_json::json!([
                {"query":target,"family":target,"programId":"public-control","trueType":true},
                {"query":source,"family":source,"programId":"original-control","trueType":true}
            ]);
            let kept: serde_json::Value = serde_json::from_str(
                &core
                    .build_font_environment_from_available_programs(&primary.to_string())
                    .unwrap(),
            )
            .unwrap();
            assert!(
                kept.is_null(),
                "원본 프로그램이 있으면 조건부 공개 규칙은 적용하지 않음"
            );
            observed.insert(rule["ruleId"].as_str().unwrap());
            continue;
        }
        let (alt_type, text) = match (boundary, slot) {
            ("rust-style-resolution.legacy-latin", "1") => (1, "A"),
            ("rust-style-resolution.hft", "all") => (2, "가"),
            ("rust-style-resolution.hft", "1") => (2, "A"),
            ("rust-style-resolution.ttf", "all") => (1, "가"),
            other => panic!("unexpected layout-name route: {other:?}"),
        };
        let record = public_trace_record(source_face(rule), alt_type, text);
        assert_eq!(record["document"]["face"], rule["sourceFace"]);
        assert_eq!(
            record["layoutName"]["normalizedFace"], rule["targetFaceOrPolicy"],
            "{}",
            rule["ruleId"]
        );
        assert!(
            record["provenance"]
                .as_array()
                .expect("trace provenance")
                .iter()
                .any(|entry| entry["ruleId"] == rule["ruleId"]),
            "public trace did not report {}",
            rule["ruleId"]
        );
        observed.insert(rule["ruleId"].as_str().expect("ruleId"));
    }

    assert_eq!(observed.len() + priority_shadowed.len(), 179);
    assert_eq!(priority_shadowed.len(), 34);

    let sentinel = public_trace_record("__rhwp_w7_unregistered_font__", 1, "가");
    assert_eq!(
        sentinel["layoutName"]["normalizedFace"],
        "__rhwp_w7_unregistered_font__"
    );
    assert!(sentinel["layoutName"]["steps"]
        .as_array()
        .expect("layout steps")
        .is_empty());
}

#[test]
fn canonical_layout_metric_projection_reaches_public_lookup() {
    let registry = registry();
    let rules = projection_rules(&registry, "rust-layout-metric");
    assert_eq!(rules.len(), 71);

    for rule in rules {
        let source = source_face(rule);
        let target = rule["targetFaceOrPolicy"].as_str().expect("metric target");
        assert_eq!(source_boundary(rule), "rust-metric.metric-alias");
        // v2 규칙 ID는 명시적 변경 원장에서 발급한 식별자다. 기존 hash ID뿐
        // 아니라 추가된 서술형 ID도 실제 선택된 projection과 trace가 연결돼야 한다.
        // 이름 정규화를 우회하되 IR은 그대로인 명시 환경으로 metric 경계를 직접 검사한다.
        let mut core = public_blank_with_font_text(source, 1, "A");
        let environment = serde_json::json!({
            "id":"metric-projection-trace-contract",
            "substitutions": {source: source},
            "trueTypeFaces": [source]
        });
        core.set_font_environment(Some(
            rhwp::renderer::font_environment::FontEnvironment::from_json(&environment.to_string())
                .unwrap(),
        ))
        .unwrap();
        let trace: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        let record = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["source"]["character"] == "A")
            .unwrap();
        assert_eq!(record["layoutMetric"]["requestedFace"], source);
        assert_eq!(record["layoutMetric"]["aliasResolvedFace"], target);
        assert!(
            record["layoutMetric"]["metricEntry"].is_number(),
            "실제 metric entry를 관측해야 함"
        );
        let reported = record["provenance"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["ruleId"] == rule["ruleId"]);
        if source == target {
            // D2Coding 등의 identity 보존 규칙은 별칭 변경이 아니다. 존재하지 않는
            // alias step을 요구하지 않고 위에서 같은 face의 실제 metric 선택을 확인한다.
            assert!(
                !reported,
                "identity mapping must not invent an alias transition"
            );
        } else {
            assert!(reported, "metric trace {}", rule["ruleId"]);
        }
        for bold in [false, true] {
            for italic in [false, true] {
                let selected = find_metric(source, bold, italic).unwrap_or_else(|| {
                    panic!(
                        "{} did not resolve for bold={bold} italic={italic}",
                        rule["ruleId"]
                    )
                });
                assert_eq!(selected.metric.name, target, "{}", rule["ruleId"]);
            }
        }
    }

    assert!(find_metric("__rhwp_w7_unregistered_metric__", false, false).is_none());
}
