//! 관계형: 단일 face로 표현할 수 없는 선언은 관측에서 제외하고 정상 선언은 보존한다.
//! 주석 갱신: 2026-11-06T17:40:12+09:00.
//! 근거: #8687 및 mydocs/plans/issue_8687_font_fallback_names.md.
//! 합성 FontFace의 요청·환경·원본 보존 API 계약이며 실물 페이지의 시각 측정 대상이 아니다.
//! 설치 글꼴이나 네트워크 없이 공급 관측을 주입하며 픽셀·쪽수 기대값을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::ObservedFontSource,
    model::style::{Font, SubstFont},
    renderer::font_environment::FontEnvironment,
    DocumentCore,
};

const VALID_SOURCE: &str = "RHWP Valid Source";
const VALID_FALLBACK: &str = "RHWP Valid Fallback";

fn core_with_fonts(source: &str, fallback: &str, structured: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for (language, fonts) in document.doc_info.font_faces.iter_mut().enumerate() {
        let (source, fallback) = if language == 0 {
            (source, fallback)
        } else {
            (VALID_SOURCE, VALID_FALLBACK)
        };
        fonts[0] = Font {
            name: source.into(),
            alt_type: 1,
            alt_name: Some(fallback.into()),
            subst_font: structured.then(|| SubstFont {
                face: fallback.into(),
                font_type: 1,
                ..Default::default()
            }),
            ..Default::default()
        };
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    core.set_document(document);
    core
}

fn observed() -> ObservedFontSource {
    ObservedFontSource {
        family: VALID_FALLBACK.into(),
        sha256: "ab".repeat(32),
        face_index: 0,
    }
}

#[test]
fn invalid_source_names_do_not_poison_other_observed_substitutions() {
    for source in [
        r"산돌고딕 L\,한컴돋움",
        r"산돌명조 L\,한컴돋움",
        "Source, Other",
        "Source\\Name",
        "Source'Name",
        "Source\"Name",
        "Source;Name",
        "Source{Name}",
        "Source\nName",
        " Source",
        "Source ",
        "",
        &"A".repeat(257),
    ] {
        let mut core = core_with_fonts(source, VALID_FALLBACK, true);
        let original = core.export_hwp_native().unwrap();
        core.prepare_declared_font_fallbacks_with(|_| Some(observed()))
            .expect("부적합한 한 항목 때문에 정상 관측 환경 전체를 거부하지 않는다");
        let requests = core.declared_font_fallback_requests();
        assert!(
            !requests.is_empty(),
            "다른 언어 슬롯의 정상 요청을 보존한다"
        );
        assert!(requests.iter().all(|r| r.family == VALID_SOURCE));
        assert!(core
            .observed_font_fallbacks()
            .iter()
            .all(|r| r.request.family == VALID_SOURCE));
        let environment = serde_json::to_value(core.resolved_font_environment().unwrap()).unwrap();
        assert_eq!(
            environment["substitutions"][VALID_SOURCE].as_str(),
            Some(VALID_FALLBACK)
        );
        assert!(environment["substitutions"].get(source).is_none());
        assert_eq!(core.export_hwp_native().unwrap(), original);
    }
}

#[test]
fn invalid_fallback_names_never_reach_the_provider() {
    for structured in [false, true] {
        for fallback in [
            r"산돌고딕 L\,한컴돋움",
            "Fallback, Other",
            "Fallback\\Name",
            "Fallback'Name",
            "Fallback\"Name",
            "Fallback;Name",
            "Fallback{Name}",
            "Fallback\nName",
            "",
            "   ",
            &"A".repeat(257),
        ] {
            let mut core = core_with_fonts("RHWP Invalid Fallback Source", fallback, structured);
            core.prepare_declared_font_fallbacks_with(|request| {
                assert_eq!(request.family, VALID_SOURCE, "{fallback:?}, {structured}");
                assert_eq!(request.fallback, VALID_FALLBACK);
                Some(observed())
            })
            .unwrap();
            assert!(core
                .declared_font_fallback_requests()
                .iter()
                .all(|r| r.family == VALID_SOURCE));
        }
    }
}

#[test]
fn valid_single_face_names_and_trimmed_fallbacks_remain_observable() {
    for structured in [false, true] {
        let mut core = core_with_fonts("산돌고딕 L", "  함초롬돋움  ", structured);
        let requests = core.declared_font_fallback_requests();
        assert!(requests
            .iter()
            .any(|r| r.family == "산돌고딕 L" && r.fallback == "함초롬돋움"));
        assert!(requests
            .iter()
            .any(|r| r.family == VALID_SOURCE && r.fallback == VALID_FALLBACK));
        assert!(core
            .prepare_declared_font_fallbacks_with(|_| Some(observed()))
            .unwrap());
        assert!(!core
            .prepare_declared_font_fallbacks_with(|_| Some(observed()))
            .unwrap());
    }
    assert!(FontEnvironment::from_json(
        r#"{"id":"invalid-list","substitutions":{"Source, Other":"Fallback"}}"#
    )
    .is_err());
}

#[test]
fn only_invalid_lists_leave_the_explicit_environment_unchanged() {
    let mut core = core_with_fonts(r"산돌고딕 L\,한컴돋움", VALID_FALLBACK, true);
    let mut document = core.document().clone();
    for font in document.doc_info.font_faces.iter_mut().flatten() {
        font.name = r"산돌고딕 L\,한컴돋움".into();
    }
    core.set_document(document);
    let original = core.export_hwp_native().unwrap();
    let environment =
        FontEnvironment::from_json(r#"{"id":"preserved-environment","substitutions":{}}"#).unwrap();
    core.set_font_environment(Some(environment.clone()))
        .unwrap();
    assert!(!core
        .prepare_declared_font_fallbacks_with(|_| panic!(
            "부적합한 이름은 공급자에 전달하지 않는다"
        ))
        .unwrap());
    assert!(core.declared_font_fallback_requests().is_empty());
    assert!(core.observed_font_fallbacks().is_empty());
    assert_eq!(core.resolved_font_environment(), Some(&environment));
    assert_eq!(core.export_hwp_native().unwrap(), original);
}

fn hangul_font_chain(core: &DocumentCore) -> Vec<serde_json::Value> {
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["source"]["character"] == "가")
        .unwrap()["layoutName"]["cssFamilyChain"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn source_font_record_is_not_split_ahead_of_its_declared_substitute() {
    let source = r"산돌고딕 L\,한컴돋움";
    let mut core = core_with_fonts(source, "함초롬돋움", true);
    core.insert_text_native(0, 0, 0, "가").unwrap();
    assert_eq!(
        hangul_font_chain(&core),
        [serde_json::json!("함초롬돋움")],
        "원본 이름 내부의 쉼표로 다른 face를 선언된 대체보다 먼저 선택하지 않는다"
    );
    assert_eq!(core.document().doc_info.font_faces[0][0].name, source);
    assert_eq!(
        core.document().doc_info.font_faces[0][0]
            .subst_font
            .as_ref()
            .unwrap()
            .face,
        "함초롬돋움"
    );
}

#[test]
fn single_source_face_keeps_precedence_over_its_declared_substitute() {
    let mut core = core_with_fonts(VALID_SOURCE, "함초롬돋움", true);
    core.insert_text_native(0, 0, 0, "가").unwrap();
    assert_eq!(
        hangul_font_chain(&core),
        [
            serde_json::json!(VALID_SOURCE),
            serde_json::json!("함초롬돋움")
        ]
    );
}
