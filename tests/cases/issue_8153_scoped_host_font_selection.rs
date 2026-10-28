//! 공급 관측은 먼저 선택한 문서 슬롯/전역 프로그램을 덮어쓰지 않는다.
//! 합성 세션 프로토콜 검사이며 실물 글꼴 설치나 픽셀 일치의 증거가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::ObservedFontSource, model::style::SubstFont,
    renderer::font_environment::FontEnvironment, DocumentCore,
};

const SOURCE: &str = "RHWP Scoped Fixture";

fn core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for font in document.doc_info.font_faces.iter_mut().flatten() {
        font.name = SOURCE.into();
        font.alt_type = 1;
        font.subst_font = Some(SubstFont {
            face: "Observed Fixture".into(),
            font_type: 1,
            ..Default::default()
        });
    }
    core.set_document(document);
    assert!(!core.declared_font_fallback_requests().is_empty());
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
fn observed_host_fallback_preserves_scoped_and_explicit_global_selections() {
    for json in [
        r#"{"id":"scoped-programs","substitutions":{},"documentSubstitutions":[{"languageSlot":0,"fontId":0,"source":"RHWP Scoped Fixture","target":"Slot A","programId":"program-a"},{"languageSlot":1,"fontId":0,"source":"RHWP Scoped Fixture","target":"Slot B","programId":"program-b"}]}"#,
        r#"{"id":"explicit-program","substitutions":{"RHWP Scoped Fixture":"Explicit Fixture"}}"#,
    ] {
        let mut core = core();
        let original = core.export_hwp_native().unwrap();
        let environment = FontEnvironment::from_json(json).unwrap();
        core.set_font_environment(Some(environment.clone()))
            .unwrap();
        let mut observations = 0;
        let changed = core
            .prepare_declared_font_fallbacks_with(|_| {
                observations += 1;
                Some(observed())
            })
            .expect("주소별 프로그램에 충돌하는 전역 관측을 추가하지 않는다");
        assert_eq!(observations, 0, "이미 선택한 소스를 전역 관측하지 않는다");
        assert!(!changed);
        assert_eq!(core.resolved_font_environment(), Some(&environment));
        assert!(core.observed_font_fallbacks().is_empty());
        assert_eq!(core.export_hwp_native().unwrap(), original);
    }
}

#[test]
fn observed_host_fallback_still_selects_an_unclaimed_source() {
    let mut core = core();
    let original = core.export_hwp_native().unwrap();
    let mut observations = 0;
    assert!(core
        .prepare_declared_font_fallbacks_with(|_| {
            observations += 1;
            Some(observed())
        })
        .unwrap());
    assert!(
        observations > 0,
        "선택하지 않은 원본 대체 요청은 공급자가 확인한다"
    );
    let expected = FontEnvironment::from_json(
        r#"{"id":"observed-host-fonts","substitutions":{"RHWP Scoped Fixture":"Observed Fixture"},"trueTypeFaces":["Observed Fixture"]}"#,
    )
    .unwrap();
    assert_eq!(core.resolved_font_environment(), Some(&expected));
    assert_eq!(core.observed_font_fallbacks().len(), observations);
    assert!(
        !core
            .prepare_declared_font_fallbacks_with(|_| Some(observed()))
            .unwrap(),
        "같은 공급 관측은 반복 재조판하지 않는다"
    );
    assert_eq!(core.export_hwp_native().unwrap(), original);

    let mut public_core = self::core();
    let public_original = public_core.export_hwp_native().unwrap();
    let records: Vec<_> = public_core
        .declared_font_fallback_requests()
        .into_iter()
        .map(|request| {
            serde_json::json!({
                "request": {
                    "family": request.family,
                    "fallback": request.fallback,
                    "bold": request.bold,
                    "italic": request.italic,
                },
                "source": {
                    "family": "Observed Fixture",
                    "sha256": "ab".repeat(32),
                    "faceIndex": 0,
                },
            })
        })
        .collect();
    assert!(public_core
        .set_observed_font_fallbacks_native(&serde_json::to_string(&records).unwrap())
        .unwrap());
    assert_eq!(public_core.observed_font_fallbacks().len(), records.len());
    assert_eq!(public_core.export_hwp_native().unwrap(), public_original);
}
