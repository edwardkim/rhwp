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
fn sealed_v1_and_current_v2_projection_semantics_match() {
    let v1 = sealed_v1_registry();
    let v2 = registry();
    let v1_rules = v1["rules"].as_array().expect("sealed v1 rules");
    let v2_rules = v2["rules"].as_array().expect("current v2 rules");

    assert_eq!(v1_rules.len(), 830);
    assert_eq!(v2["summary"]["activeRuleCount"], 830);
    assert_eq!(v2["summary"]["retiredRuleCount"], 0);

    let v1_semantics: BTreeMap<_, _> = v1_rules
        .iter()
        .map(|rule| {
            let rule_id = rule["ruleId"].as_str().expect("sealed v1 ruleId");
            let source_boundary_id = rule["evidence"]["sourceBoundaryIds"][0]
                .as_str()
                .expect("sealed v1 source boundary");
            (rule_id.to_owned(), semantic_rule(rule, source_boundary_id))
        })
        .collect();
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
    assert_eq!(v2_semantics, v1_semantics);

    for projection in [
        "rust-layout-name",
        "rust-layout-metric",
        "canvas2d-paint",
        "canvas2d-webfont",
        "canvaskit-sfnt",
    ] {
        let v1_rule_ids: Vec<_> = v1_rules
            .iter()
            .filter(|rule| {
                rule["projections"]
                    .as_array()
                    .expect("sealed v1 projections")
                    .iter()
                    .any(|entry| entry["id"] == projection)
            })
            .map(|rule| rule["ruleId"].as_str().expect("sealed v1 ruleId"))
            .collect();
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
    assert_eq!(rules.len(), 171);

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

    assert_eq!(observed.len() + priority_shadowed.len(), 171);
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
    assert_eq!(rules.len(), 67);

    for rule in rules {
        let source = source_face(rule);
        let target = rule["targetFaceOrPolicy"].as_str().expect("metric target");
        assert_eq!(source_boundary(rule), "rust-metric.metric-alias");
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
