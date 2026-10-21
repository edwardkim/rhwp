//! 한컴 2020/2024 독립 Print의 문서 대체 필드 선택 계약.
//! 합성 API 입력이며 설치한 글꼴이나 실물 좌표의 증거로 사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::{json, Value};

fn core(
    primary: &str,
    substitute: &str,
    alt_type: u8,
    embedded: bool,
    sub_embedded: bool,
) -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.insert_text_native(0, 0, 0, "04 비교 & 고전소설(2) ABC 0123")
        .unwrap();
    let mut d = c.document().clone();
    for f in d.doc_info.font_faces.iter_mut().flatten() {
        f.name = primary.into();
        f.alt_type = alt_type;
        f.is_embedded = embedded;
        f.subst_font = Some(rhwp::model::style::SubstFont {
            face: substitute.into(),
            font_type: 1,
            is_embedded: sub_embedded,
            ..Default::default()
        });
    }
    c.set_document(d);
    c
}

#[test]
fn document_substitute_lookup_distinguishes_field_and_primary_names() {
    let mut errors = Vec::new();
    for (primary, substitute, alt, expected) in [
        ("a로케트", "HCR Batang", 1, "Haansoft Batang"),
        ("a로케트", "함초롬바탕", 1, "함초롬바탕"),
        ("a로케트", "Arial", 1, "Arial"),
        ("a로케트", "한컴바탕", 1, "한컴바탕"),
        ("HCR Batang", "Arial", 1, "Arial"),
        ("함초롬바탕", "Arial", 1, "Arial"),
        // 독립 Print가 증명한 TTF 필드 외의 HFT/미상 선언은 확장하지 않는다.
        ("a로케트", "HCR Batang", 2, "HCR Batang"),
        ("a로케트", "HCR Batang", 0, "HCR Batang"),
    ] {
        let c = core(primary, substitute, alt, false, false);
        let raw = c.export_hwp_native().unwrap();
        for request in FontEnvironment::document_requests(&c.document().doc_info) {
            if request.substitute.first().map(String::as_str) != Some(expected) {
                errors.push(format!(
                    "{primary}/{substitute}/type{alt}: {:?}, expected {expected}",
                    request.substitute
                ));
            }
            if primary == "HCR Batang" {
                assert_eq!(
                    request.primary.first().map(String::as_str),
                    Some("HCR Batang")
                );
            }
        }
        assert!(c.document().doc_info.font_faces.iter().flatten().all(|f| f
            .subst_font
            .as_ref()
            .unwrap()
            .face
            == substitute));
        assert_eq!(c.export_hwp_native().unwrap(), raw);
    }
    assert!(
        errors.is_empty(),
        "independent Print selection mismatch: {errors:?}"
    );
    for font_type in [0, 2] {
        let mut c = core("a로케트", "HCR Batang", 1, false, false);
        let mut d = c.document().clone();
        for f in d.doc_info.font_faces.iter_mut().flatten() {
            f.subst_font.as_mut().unwrap().font_type = font_type;
        }
        c.set_document(d);
        assert!(FontEnvironment::document_requests(&c.document().doc_info)
            .iter()
            .all(|r| r.substitute.first().map(String::as_str) == Some("HCR Batang")));
    }
}

#[test]
fn document_substitute_selection_preserves_program_and_user_precedence() {
    let programs = json!([
        {"query":"Haansoft Batang","family":"Haansoft Batang","programId":"mock-hb","trueType":true},
        {"query":"HCR Batang","family":"HCR Batang","programId":"mock-hcr","trueType":true}
    ]).to_string();
    let mut c = core("a로케트", "HCR Batang", 1, false, false);
    let raw = c.export_hwp_native().unwrap();
    let env = FontEnvironment::from_available_programs(&c.document().doc_info, &programs)
        .unwrap()
        .unwrap();
    c.set_font_environment(Some(env)).unwrap();
    let trace: Value = serde_json::from_str(
        &c.get_font_decision_trace_native(0, r#"{"maxCharacters":80}"#)
            .unwrap(),
    )
    .unwrap();
    let r = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["character"] == "비")
        .unwrap();
    assert_eq!(r["layoutName"]["normalizedFace"], "Haansoft Batang");
    assert_eq!(r["paint"]["native"]["requested"], "Haansoft Batang");
    assert_eq!(
        r["document"]["substFont"], "HCR Batang",
        "trace preserves the source declaration"
    );
    assert!(r["layoutName"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step["kind"] == "documentSubstituteLookup"
            && step["input"] == "HCR Batang"
            && step["output"] == "Haansoft Batang"));
    assert!(r["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |p| p["ruleId"] == "rule.rust-style-resolution.8f1a484b6e8dff748728"
                && p["reason"].is_null()
        ));
    c.set_font_environment(Some(
        FontEnvironment::from_json(
            r#"{"id":"explicit-hcr","substitutions":{"a로케트":"HCR Batang"}}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    let explicit: Value = serde_json::from_str(
        &c.get_font_decision_trace_native(0, r#"{"maxCharacters":80}"#)
            .unwrap(),
    )
    .unwrap();
    let r = explicit["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["character"] == "비")
        .unwrap();
    assert_eq!(r["layoutName"]["normalizedFace"], "HCR Batang");
    assert_eq!(c.export_hwp_native().unwrap(), raw);
    let primary = core("HCR Batang", "Arial", 1, false, false);
    assert!(
        FontEnvironment::from_available_programs(&primary.document().doc_info, &programs)
            .unwrap()
            .is_none()
    );
    let present = json!([{ "query":"a로케트", "family":"a로케트", "programId":"mock-original", "trueType":true }]).to_string();
    assert!(
        FontEnvironment::from_available_programs(&c.document().doc_info, &present)
            .unwrap()
            .is_none()
    );
    for (embedded, sub_embedded) in [(true, false), (false, true)] {
        let c = core("a로케트", "HCR Batang", 1, embedded, sub_embedded);
        assert!(FontEnvironment::document_requests(&c.document().doc_info).is_empty());
        assert!(
            FontEnvironment::from_available_programs(&c.document().doc_info, &programs)
                .unwrap()
                .is_none()
        );
    }
}

// Stage35의 같은 원본 전46쪽 Native/fresh>=90 및46쪽 양쪽>=98·직접 판독 뒤 추가.
// 독립 Print의 04 왼쪽44.16pt·제목 끝213.78pt를96dpi CSSpx로 대조한다.
#[test]
fn document_substitute_preserves_the_print_heading_owner_and_width() {
    use rhwp::paint::{LayerJsonOptions, RenderProfile};
    fn runs<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
        match v {
            Value::Object(m) => {
                if m.get("type").and_then(Value::as_str) == Some("textRun") {
                    out.push(v);
                }
                for c in m.values() {
                    runs(c, out);
                }
            }
            Value::Array(a) => {
                for c in a {
                    runs(c, out);
                }
            }
            _ => {}
        }
    }
    for ext in ["hwp", "hwpx"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("samples/[2027] 온새미로 1 본교재.{ext}"));
        let mut c = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        c.set_hangul2024_compat(ext == "hwpx");
        let mut programs = Vec::new();
        for prefix in ["KoPub", "KoPubWorld"] {
            for family in ["Batang", "Dotum"] {
                for weight in ["Light", "Medium", "Bold"] {
                    programs.push(json!({"query":format!("{prefix}{family}{weight}"),"family":format!("{prefix}{family} {weight}"),"programId":format!("mock-{prefix}-{family}-{weight}"),"trueType":true}));
                }
            }
        }
        for family in ["Haansoft Batang", "HCR Batang"] {
            programs.push(json!({"query":family,"family":family,"programId":format!("mock-{family}"),"trueType":true}));
        }
        let raw = c.export_hwp_native().unwrap();
        let env = FontEnvironment::from_available_programs(
            &c.document().doc_info,
            &serde_json::to_string(&programs).unwrap(),
        )
        .unwrap();
        c.set_font_environment(env).unwrap();
        assert_eq!(c.page_count(), 46, "independent Print count");
        let page: Value = serde_json::from_str(
            &c.get_page_layer_tree_with_options_native(
                45,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let mut all = Vec::new();
        runs(&page["root"], &mut all);
        let headings: Vec<_> = all
            .iter()
            .filter(|r| {
                r["text"]
                    .as_str()
                    .is_some_and(|t| t.contains("비교 & 고전소설("))
            })
            .collect();
        assert_eq!(headings.len(), 1, "46쪽 제목의 누락·중복 방지");
        let owner = headings[0]["source"]["stableSourceKey"].as_str().unwrap();
        let address = owner.split("/char:").next().unwrap();
        let cell = owner.split("/cell:").nth(1).unwrap();
        let group: Vec<_> = all
            .iter()
            .filter(|r| {
                r["source"]["stableSourceKey"].as_str().is_some_and(|k| {
                    k.split("/char:").next() == Some(address)
                        && k.split("/cell:").nth(1) == Some(cell)
                })
            })
            .collect();
        let text: String = group.iter().map(|r| r["text"].as_str().unwrap()).collect();
        assert_eq!(
            text.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
            "04비교&고전소설(2)",
            "같은 원본 셀에서 제목 순서·내용 소유 보존"
        );
        let mut previous_end = None;
        for r in &group {
            let x = r["placement"]["runToPage"]["e"].as_f64().unwrap();
            let end = x + r["bbox"]["width"].as_f64().unwrap();
            if let Some(p) = previous_end {
                assert!(x + 0.001 >= p, "제목 조각 순서·겹침 방지");
            }
            previous_end = Some(end);
        }
        let left = group[0]["placement"]["runToPage"]["e"].as_f64().unwrap();
        assert!(
            (left - 44.16 * 96.0 / 72.0).abs() <= 2.0,
            "독립 Print 제목 시작 ±2CSSpx: {left}"
        );
        let right = previous_end.unwrap();
        assert!(
            (right - 213.78 * 96.0 / 72.0).abs() <= 2.0,
            "독립 Print 닫는 괄호 끝 ±2CSSpx: {right}"
        );
        assert!(
            group
                .iter()
                .all(|r| r["style"]["fontFamily"] == "Haansoft Batang"),
            "독립 Print의 선택 프로그램"
        );
        assert_eq!(
            c.export_hwp_native().unwrap(),
            raw,
            "선택이 저장 글꼴·본문을 변경하지 않는다"
        );
    }
}
