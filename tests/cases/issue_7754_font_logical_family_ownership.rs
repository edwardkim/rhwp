//! 관계형·고정형: 논리 원본 이름과 선택 프로그램 family의 소유 및 명시 조회 우선을 검사한다.
//! 주석 갱신: 2026-11-22T12:44:52+09:00. 새 public API 계약 검사다.
//! 합성 DocInfo/FontEnvironment의 이름 해석만 검사하며 실제 글꼴·glyph·픽셀의 시각 측정은 비해당이다.
//! 존재하지 않는 family와 program ID를 선언하므로 시스템 글꼴의 설치를 가정하지 않는다.
//! 실제 별칭 회귀의 독립 PDF·전후 Native 증거는 mydocs/pr/assets/issues7754_7765/source37/native-boundaries/.
//! 위 원본의 raw 값을 이 합성 입력의 측정값으로 재사용하지 않는다. fresh WASM 전쪽 증거는 별도로 확보한다.

use rhwp::{model::style::Font, renderer::font_environment::FontEnvironment, DocumentCore};

#[test]
fn selected_program_identity_does_not_promote_a_logical_legacy_family() {
    let source = "Logical HFT Name";
    let family = "Selected Physical Family";
    let fallback = "Declared Glyph Fallback";
    for (source_type, query, expected_first) in [
        (2, source, family),
        (2, "Selected-PostScript-Name", "Selected-PostScript-Name"),
        (1, source, source),
    ] {
        let mut document = DocumentCore::new_empty();
        document.create_blank_document_native().unwrap();
        document.insert_text_native(0, 0, 0, "A").unwrap();
        document.document_mut().doc_info.font_faces = vec![
            vec![Font {
                name: source.into(),
                alt_type: source_type,
                ..Default::default()
            }];
            7
        ];
        for shape in &mut document.document_mut().doc_info.char_shapes {
            shape.font_ids = [0; 7];
        }
        let environment = FontEnvironment::from_json(
            &serde_json::json!({
                "id": "selected-physical-program-name-contract",
                "preserveSourceMetrics": true,
                "substitutions": {(source): family},
                "selectedPrograms": {(source): {
                    "query": query, "family": family, "programId": "01".repeat(32),
                    "trueType": source_type == 1, "openTypeCff": source_type == 2
                }},
                "glyphFallbacks": {(family): [fallback]}
            })
            .to_string(),
        )
        .unwrap();
        document.set_font_environment(Some(environment)).unwrap();
        let trace: serde_json::Value = serde_json::from_str(
            &document
                .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        let record = &trace["records"][0];
        assert_eq!(record["layoutName"]["requestedFace"].as_str(), Some(source));
        let chain = record["layoutName"]["cssFamilyChain"].as_array().unwrap();
        assert_eq!(chain[0].as_str(), Some(expected_first));
        assert!(chain.iter().any(|name| name.as_str() == Some(fallback)));
        if source_type == 2 && query == source {
            assert_eq!(
                record["layoutName"]["cssFamilyChain"],
                serde_json::json!([family, fallback])
            );
        }
    }
}
