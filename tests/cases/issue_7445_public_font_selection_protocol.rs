//! 선택 프로토콜의 우선순위·소스 보존 계약. 픽셀 배치나 실물 fixture의 golden이 아니다.
//! `programId`는 공급자가 제공한 snapshot을 모형화한다. 이 모형을 실제 글꼴 공급 증거로 쓰지 않는다.
//! 실제 공급은 C168 실제 Studio의 HTTP WOFF2·FontFace.load·fresh WASM 응답 hash와
//! Native/fresh WASM 원본 전쪽 비교로 별도 검증한다. 단계8 C151/C168 기록 참조.
use rhwp::{
    model::style::{Font, SubstFont},
    renderer::font_environment::FontEnvironment,
    DocumentCore,
};
use serde_json::{json, Value};
const SOURCE: &str = "하이텔울릉도제목체";
const PUBLIC: &str = "RHWP Public Batang";
fn core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        *fonts = vec![Font {
            name: SOURCE.into(),
            alt_type: 2,
            ..Default::default()
        }];
    }
    core.set_document(doc);
    core
}
fn program(query: &str) -> Value {
    json!({"query":query,"family":query,"programId":format!("protocol-control-{query}"),"trueType":true})
}
fn select(core: &DocumentCore, input: Value) -> Value {
    serde_json::from_str(
        &core
            .build_font_environment_from_available_programs(&input.to_string())
            .unwrap(),
    )
    .unwrap()
}
#[test]
fn missing_source_uses_the_supplied_public_program_and_preserves_serialized_source() {
    let mut core = core();
    let before = core.export_hwp_native().unwrap();
    let selected = select(&core, json!([program(PUBLIC)]));
    assert_eq!(
        selected["substitutions"][SOURCE], PUBLIC,
        "실제 공급 snapshot에서만 공개 fallback 선택"
    );
    core.set_font_environment(Some(
        FontEnvironment::from_json(&selected.to_string()).unwrap(),
    ))
    .unwrap();
    assert_eq!(
        core.export_hwp_native().unwrap(),
        before,
        "출력 환경은 원본 글꼴/저장 IR을 바꾸지 않음"
    );
}
#[test]
fn available_original_program_precedes_the_public_program() {
    assert!(select(&core(), json!([program(PUBLIC), program(SOURCE)])).is_null());
}
#[test]
fn document_declared_program_precedes_the_public_program() {
    let mut core = core();
    let mut doc = core.document().clone();
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.subst_font = Some(SubstFont {
            face: "Arial".into(),
            font_type: 1,
            ..Default::default()
        });
    }
    core.set_document(doc);
    let selected = select(&core, json!([program(PUBLIC), program("Arial")]));
    assert_eq!(selected["substitutions"][SOURCE], "Arial");
}
#[test]
fn presence_only_source_blocks_public_selection_without_claiming_program_supply() {
    assert!(select(
        &core(),
        json!({"programs":[program(PUBLIC)],"publicFallbackBlockedSources":[SOURCE]})
    )
    .is_null());
}
#[test]
fn embedded_original_is_never_a_public_substitution_request() {
    let mut core = core();
    let mut doc = core.document().clone();
    doc.doc_info.font_faces[0][0].is_embedded = true;
    core.set_document(doc);
    assert!(select(&core, json!([program(PUBLIC)])).is_null());
}
#[test]
fn embedded_family_is_not_replaced_by_a_global_public_program() {
    let mut core = core();
    let mut doc = core.document().clone();
    doc.doc_info.font_faces[0].push(Font {
        name: PUBLIC.into(),
        is_embedded: true,
        ..Default::default()
    });
    core.set_document(doc);
    assert!(select(&core, json!([program(PUBLIC)])).is_null());
}
#[test]
fn conflicting_language_slot_selection_is_not_resurrected_by_a_later_fallback() {
    let mut core = core();
    let mut doc = core.document().clone();
    doc.doc_info.font_faces[0][0].subst_font = Some(SubstFont {
        face: "Arial".into(),
        font_type: 1,
        ..Default::default()
    });
    core.set_document(doc);
    assert!(select(&core, json!([program(PUBLIC), program("Arial")])).is_null());
}

#[test]
fn embedded_document_substitute_keeps_its_source_out_of_public_selection() {
    let mut core = core();
    let mut doc = core.document().clone();
    doc.doc_info.font_faces[0][0].subst_font = Some(SubstFont {
        face: "Document Embedded Program".into(),
        is_embedded: true,
        ..Default::default()
    });
    core.set_document(doc);
    let plan: Value = serde_json::from_str(&core.get_document_font_selection_plan()).unwrap();
    assert!(plan["embeddedFamilies"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == SOURCE));
    assert!(plan["requests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|request| request["source"] != SOURCE));
    assert!(select(&core, json!([program(PUBLIC)])).is_null());
}
