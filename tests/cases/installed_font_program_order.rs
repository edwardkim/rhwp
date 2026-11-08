//! 관계형: 실제 공급한 원본 프로그램의 TTF → TTC face → OTF → 대체 선택을 검사한다.
//! 주석 갱신: 2026-11-07 19:00:32 KST. 관계형이며 설치된 사설 글꼴에 의존하지 않는다.
//! 설치 글꼴이나 사설 바이너리 없이 원본 이름·형식·프로그램 신원의 소유를 확인한다.
use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::{json, Value};

fn environment(source: &str, programs: Value) -> Value {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = source.into();
            font.alt_type = 1;
        }
    }
    let requests = FontEnvironment::document_requests(&doc.doc_info);
    assert!(requests
        .iter()
        .all(|request| request.primary.first().map(String::as_str) == Some(source)));
    serde_json::to_value(
        FontEnvironment::from_available_programs(&doc.doc_info, &programs.to_string()).unwrap(),
    )
    .unwrap()
}
fn program(query: &str, family: &str, id: &str, true_type: bool, collection: bool) -> Value {
    json!({"query":query,"family":family,"programId":id,"trueType":true_type,
        "openTypeCff":!true_type,"collectionFace":collection})
}
#[test]
fn original_otf_is_selected_before_compatibility_ttf() {
    let value = environment(
        "옥수수",
        json!([
            program("옥수수", "Corn", "original-otf", false, false),
            program("돋움", "Dotum", "substitute-ttf", true, false)
        ]),
    );
    assert_eq!(value["substitutions"]["옥수수"], "Corn");
    assert_eq!(
        value["selectedPrograms"]["옥수수"]["programId"],
        "original-otf"
    );
}
#[test]
fn original_program_aliases_use_ttf_then_ttc_then_otf() {
    let source = "KoPub바탕체 Light";
    let alias = "KoPubBatangLight";
    for (primary, secondary, expected) in [
        ((false, false), (true, false), "alias"),
        ((true, true), (true, false), "alias"),
        ((false, false), (true, true), "alias"),
        ((true, false), (true, true), "source"),
    ] {
        let value = environment(
            source,
            json!([
                program(source, "KoPubBatang Light", "source", primary.0, primary.1),
                program(
                    alias,
                    "KoPubBatang Light",
                    "alias",
                    secondary.0,
                    secondary.1
                )
            ]),
        );
        assert_eq!(value["selectedPrograms"][source]["programId"], expected);
    }
}

#[test]
fn native_discovery_checks_installed_otf_even_for_a_ttf_declared_source() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    let source = "RHWP HFT Outline Fixture";
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = source.into();
            font.alt_type = 1;
        }
    }
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts");
    let selected = FontEnvironment::from_native_document_substitutions(&doc.doc_info, &[directory])
        .unwrap()
        .expect("원본 OTF를 TTF 선언 때문에 조회 전에 건너뛰지 않는다");
    let value = serde_json::to_value(selected).unwrap();
    assert_eq!(value["selectedPrograms"][source]["openTypeCff"], true);
    assert_eq!(value["selectedPrograms"][source]["trueType"], false);
}
