//! 명시적 폰트 환경 선언의 유효성과 편집 batch 경계를 검증한다.
//! KoPub 미설치 환경의 대체 출력·폭·재조판 회귀는 작업지시자 요청으로 제거했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::json;

fn environment() -> FontEnvironment {
    FontEnvironment::from_json(
        r#"{"id":"explicit-substitution","substitutions":{"원본글꼴":"대체글꼴"}}"#,
    )
    .unwrap()
}

#[test]
fn issue_6389_environment_rejects_invalid_declarations_and_active_batches() {
    for invalid in [
        json!({"id":"", "substitutions":{}}),
        json!({"id":"x", "substitutions":{"원본글꼴":""}}),
        json!({"id":"x", "substitutions":{"원본글꼴":"대체글꼴,serif"}}),
        json!({"id":"x", "substitutions":{" 원본글꼴":"대체글꼴"}}),
        json!({"id":"x", "substitutions":{}, "typo":true}),
        json!({"id":"x", "substitutions":{}, "glyphFallbacks":{"휴먼명조":[]}}),
        json!({"id":"x", "substitutions":{}, "glyphFallbacks":{"휴먼명조":["HCR Batang,serif"]}}),
        json!({"id":"x", "substitutions":{}, "glyphFallbacks":{"휴먼명조":["휴먼명조"]}}),
        json!({"id":"x", "substitutions":{}, "glyphFallbacks":{"휴먼명조":["HCR Batang", "HCR Batang"]}}),
    ] {
        assert!(FontEnvironment::from_json(&invalid.to_string()).is_err());
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.begin_batch_native().unwrap();
    assert!(core.set_font_environment(Some(environment())).is_err());
    assert!(core.font_environment().is_none());
}
