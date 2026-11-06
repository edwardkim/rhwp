//! 관계형: 환경 이름 계약을 어기는 원본 글꼴 이름은 관측 대체 요청에서 빠지고, 같은 문서의 정상 이름은 그대로 관측된다.
//! 주석 갱신: 2026-11-06T03:22:00+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 합성 세션 프로토콜 검사다. 실물 글꼴 설치·픽셀 일치의 증거가 아니며 시각 측정 수치를 고정하지 않는다.
//! 실물 계기: 10k 코퍼스 HWP 두 건의 글꼴 이름 `산돌고딕 L\,한컴돋움`(역슬래시·쉼표 포함)이
//! #8597 의 Native 글꼴 공급 확대 뒤 관측 치환 원본 키가 되어 `FontEnvironment::validate` 가
//! 문서 전체 export-png 를 거부했다(`font environment requires a valid id, …`). 같은 이름은
//! `FontEnvironment::document_requests` 가 이미 `valid_name` 으로 제외하는 계약이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{document_core::ObservedFontSource, model::style::SubstFont, DocumentCore};

const INVALID: &str = "산돌고딕 L\\,한컴돋움";
const VALID: &str = "RHWP Valid Fixture";

/// 언어 슬롯 0 은 계약 위반 이름, 나머지 슬롯은 정상 이름. 둘 다 같은 TTF 대체를 선언한다.
fn core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for (language, fonts) in document.doc_info.font_faces.iter_mut().enumerate() {
        for font in fonts.iter_mut() {
            font.name = if language == 0 { INVALID } else { VALID }.into();
            font.alt_type = 1;
            font.subst_font = Some(SubstFont {
                face: "Observed Fixture".into(),
                font_type: 1,
                ..Default::default()
            });
        }
    }
    core.set_document(document);
    core
}

fn observed() -> ObservedFontSource {
    ObservedFontSource {
        family: "Observed Fixture".into(),
        sha256: "ab".repeat(32),
        face_index: 0,
    }
}

#[test]
fn invalid_source_name_is_not_requested_but_valid_name_still_is() {
    let core = core();
    let families: Vec<String> = core
        .declared_font_fallback_requests()
        .into_iter()
        .map(|request| request.family)
        .collect();
    assert!(
        !families.iter().any(|family| family == INVALID),
        "계약 위반 이름은 관측 대체 요청이 아니다: {families:?}"
    );
    assert!(
        families.iter().any(|family| family == VALID),
        "같은 문서의 정상 이름은 그대로 요청한다: {families:?}"
    );
}

#[test]
fn observed_fallback_succeeds_when_document_has_invalid_source_name() {
    let mut core = core();
    let mut observed_families = Vec::new();
    let changed = core
        .prepare_declared_font_fallbacks_with(|request| {
            observed_families.push(request.family.clone());
            Some(observed())
        })
        .expect("계약 위반 이름 하나로 문서 전체 글꼴 환경이 거부되지 않는다");
    assert!(changed, "정상 이름의 관측 대체는 반영된다");
    assert!(
        observed_families.iter().all(|family| family == VALID),
        "관측은 정상 이름만 받는다: {observed_families:?}"
    );
    let environment = core
        .resolved_font_environment()
        .expect("관측 대체가 만든 환경");
    let value = serde_json::to_value(environment).unwrap();
    let substitutions = value["substitutions"].as_object().unwrap();
    assert!(!substitutions.contains_key(INVALID));
    assert_eq!(
        substitutions.get(VALID).and_then(|target| target.as_str()),
        Some("Observed Fixture")
    );
}
