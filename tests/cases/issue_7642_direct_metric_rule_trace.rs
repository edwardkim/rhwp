//! 직접 등록 규칙도 공개 글꼴 추적에서 실제 ID와 관계를 보존해야 한다.
//! 출력 좌표·조판을 고정하는 검사가 아니라 레지스트리와 조회 JSON의 계약 검사다.

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::model::style::Font;

fn document_with_face(name: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    let mut font_ids = [0; 7];
    for (language, fonts) in document.doc_info.font_faces.iter_mut().take(7).enumerate() {
        font_ids[language] = u16::try_from(fonts.len()).unwrap();
        fonts.push(Font {
            name: name.into(),
            ..Default::default()
        });
    }
    let mut shape = document.doc_info.char_shapes[0].clone();
    shape.raw_data = None;
    shape.font_ids = font_ids;
    let id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(shape);
    let mut paragraph = Paragraph::new_empty();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: id,
    }];
    document.sections[0].paragraphs = vec![paragraph];
    core.set_document(document);
    core.insert_text_native(0, 0, 0, "가A").unwrap();
    core
}

#[test]
fn direct_metric_alias_trace_uses_the_registered_rule_identity() {
    let registry: serde_json::Value = serde_json::from_str(include_str!(
        "../../assets/font-rules/font_rule_registry_v2.json"
    ))
    .unwrap();
    let rules: Vec<_> = registry["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|rule| {
            rule["status"] == "active"
                && rule["sourceBoundaryId"] == "rust-metric.metric-alias"
                && rule["projections"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["id"] == "rust-layout-metric" && p["mode"] == "direct")
        })
        .collect();
    assert!(!rules.is_empty(), "직접 등록 메트릭 별칭을 실제로 실행한다");
    // 모든 선택을 먼저 조회해 일부 규칙의 정상 응답이 뒤 규칙의 오류를 가리지 않게 한다.
    let traces: Vec<_> = rules
        .iter()
        .map(|rule| {
            let source = rule["sourceFace"].as_str().unwrap();
            let core = document_with_face(source);
            let raw = core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                .unwrap();
            let trace: serde_json::Value = serde_json::from_str(&raw).unwrap();
            (rule, trace)
        })
        .collect();
    for (rule, trace) in traces {
        let source = rule["sourceFace"].as_str().unwrap();
        let records: Vec<_> = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["layoutMetric"]["requestedFace"] == source)
            .collect();
        assert!(!records.is_empty(), "원본 face의 실제 선택 기록: {source}");
        for record in records {
            assert_eq!(
                record["layoutMetric"]["aliasResolvedFace"],
                rule["targetFaceOrPolicy"]
            );
            let evidence = record["provenance"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["ruleId"] == rule["ruleId"])
                .expect("실제로 선택한 레지스트리 규칙의 ID");
            assert_eq!(evidence["relationType"], rule["relationType"]);
            assert!(evidence["evidenceAnchor"]
                .as_str()
                .unwrap()
                .starts_with("assets/font-rules/font_rule_registry_v2.json#"));
            assert_eq!(evidence["evidenceStatus"], "unknown");
            assert!(
                evidence["reason"].is_null(),
                "W1 근거가 있다고 꾸미지 않는다"
            );
        }
    }
}
