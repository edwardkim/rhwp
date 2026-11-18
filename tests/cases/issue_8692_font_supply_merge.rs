//! 관계형: 공급 결과와 문서 환경을 합친 뒤의 글꼴 선택 계약을 검사한다.
//! 주석 갱신: 2026-11-06T21:20:00+09:00.
//! 공급 합치기 계약은 합성 입력으로, 실패 원인별 문서는 실물 공개 문서로 검사한다.
//! 어느 검사도 페이지 일치율이나 고정 px를 단언하지 않는다.
//!
//! 측정일 2026-11-06, `silhouette_raw_match_percent` 96 DPI, 기준 PDF는 한컴 2020 Print.
//! Native / fresh WASM 전쪽 최저값과 PDF 쪽수:
//! - `scholarship_recommendation_form.hwp`: 1쪽, 99.89 / 99.98
//! - `tax_source_ledger_form.hwp`: 1쪽, 96.64 / 100.00
//! - `employment_service_interim_report.hwp`: 90쪽, 91.29 / 91.09
//!
//! 수정 전 Native는 세 문서 모두 글꼴 환경 오류로 측정 불가다. WASM은 이 경로를 쓰지 않아
//! 수정 전후가 같다. 증거: `mydocs/pr/assets/issue_8692/`.
//!
//! 공급 합치기가 만든 환경이 `FontEnvironment::validate` 에 걸리면 내보내기 전체가 실패한다.
//! 보조 공급이 원본과 같은 글꼴이면 자기 참조 문자 대체가 생겼고, 공급 매핑이 덮어쓴 원본에는
//! 선택 규칙이 남아 치환 대상과 어긋났다. 문서가 선언한 글꼴 이름은 256개를 넘을 수 있는데
//! 치환 상한이 그보다 작아 공급을 합친 환경이 거부됐다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::native_webfonts::NativeWebfontSupply;
use rhwp::DocumentCore;

const RULE_SOURCE: &str = "하이텔울릉도제목체";
const RULE_TARGET: &str = "RHWP Public Batang";

fn fallback(source: &str, faces: &[&str]) -> serde_json::Value {
    serde_json::json!({"languageSlot":0,"fontId":0,"source":source,"faces":faces})
}

fn merged_json(supply: &NativeWebfontSupply, native: Option<FontEnvironment>) -> serde_json::Value {
    let environment = supply
        .merge_into_environment(native)
        .expect("합친 환경은 검증을 통과한다");
    serde_json::to_value(environment).unwrap()
}

/// 문서 글꼴이 등록 규칙으로 선택된 환경.
fn rule_selected_environment() -> FontEnvironment {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for face in document.doc_info.font_faces.iter_mut().flatten() {
        face.name = RULE_SOURCE.into();
        face.is_embedded = false;
        face.subst_font = None;
    }
    core.set_document(document);
    let programs = serde_json::json!([{
        "query":RULE_TARGET,"family":RULE_TARGET,"programId":"synthetic-8692","trueType":true
    }]);
    let environment =
        FontEnvironment::from_available_programs(&core.document().doc_info, &programs.to_string())
            .unwrap()
            .expect("규칙 선택 환경");
    let value = serde_json::to_value(&environment).unwrap();
    assert!(
        value["selectionRules"][RULE_SOURCE].is_string(),
        "전제: 규칙이 선택됨"
    );
    environment
}

#[test]
fn glyph_fallback_to_its_own_source_is_not_a_fallback() {
    let supply = NativeWebfontSupply {
        glyph_fallbacks: vec![fallback("HCRBatang", &["HCRBatang"])],
        ..Default::default()
    };
    assert!(
        merged_json(&supply, None).is_null(),
        "남은 공급이 없으면 환경을 만들지 않는다"
    );
}

#[test]
fn distinct_glyph_fallback_survives_beside_a_self_reference() {
    let mut other = fallback("KoPubBatangLight", &["Noto Serif", "KoPubBatangLight"]);
    other["fontId"] = 1.into();
    let supply = NativeWebfontSupply {
        glyph_fallbacks: vec![fallback("HCRBatang", &["HCRBatang"]), other],
        ..Default::default()
    };
    let merged = merged_json(&supply, None);
    let entries = merged["documentGlyphFallbacks"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "자기 참조 항목만 빠진다");
    assert_eq!(entries[0]["source"], "KoPubBatangLight");
    assert_eq!(entries[0]["faces"], serde_json::json!(["Noto Serif"]));
}

#[test]
fn supply_mapping_removes_the_selection_rule_it_replaced() {
    let mut supply = NativeWebfontSupply::default();
    supply
        .mappings
        .insert(RULE_SOURCE.into(), "NotoSansKR-Regular".into());
    let merged = merged_json(&supply, Some(rule_selected_environment()));
    assert_eq!(merged["substitutions"][RULE_SOURCE], "NotoSansKR-Regular");
    assert!(merged["selectionRules"][RULE_SOURCE].is_null());
    assert!(merged["selectedPrograms"][RULE_SOURCE].is_null());
}

#[test]
fn selection_rule_of_an_unmapped_source_is_kept() {
    let mut supply = NativeWebfontSupply::default();
    supply
        .mappings
        .insert("다른 글꼴".into(), "NotoSansKR-Regular".into());
    let merged = merged_json(&supply, Some(rule_selected_environment()));
    assert_eq!(merged["substitutions"][RULE_SOURCE], RULE_TARGET);
    assert!(merged["selectionRules"][RULE_SOURCE].is_string());
}

#[test]
fn empty_supply_returns_the_document_environment_unchanged() {
    let native = rule_selected_environment();
    let before = serde_json::to_value(&native).unwrap();
    let merged = merged_json(&NativeWebfontSupply::default(), Some(native));
    assert_eq!(merged, before);
}

#[test]
fn document_with_more_than_256_supplied_names_is_accepted_up_to_the_bound() {
    let supply_of = |count: usize| {
        let mut supply = NativeWebfontSupply::default();
        for index in 0..count {
            supply
                .mappings
                .insert(format!("글꼴{index}"), "NotoSansKR-Regular".into());
        }
        supply
    };
    let merged = merged_json(&supply_of(267), None);
    assert_eq!(merged["substitutions"].as_object().unwrap().len(), 267);
    assert!(
        supply_of(1025).merge_into_environment(None).is_err(),
        "상한은 유지한다"
    );
}

fn fixture_doc_info(name: &str) -> rhwp::model::document::DocInfo {
    let path = format!("{}/samples/issue8692/{name}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    rhwp::parser::parse_document(&bytes).unwrap().doc_info
}

/// 웹폰트 내려받기 없이도 원인이 재현되는 문서는 CLI 끝까지 실행한다.
#[test]
fn form_with_selection_rule_exports_through_the_cli() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let work = std::env::temp_dir().join(format!("rhwp-8692-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    let output = std::process::Command::new(binary)
        .current_dir(root)
        .env("RHWP_WEBFONT_OFFLINE", "1")
        .env("RHWP_WEBFONT_CACHE", work.join("cache"))
        .args([
            "export-svg",
            "samples/issue8692/scholarship_recommendation_form.hwp",
            "--compat",
            "2022",
            "--font-path",
            "ttfs/opensource",
            "-o",
        ])
        .arg(work.join("out"))
        .output()
        .unwrap();
    std::fs::remove_dir_all(&work).ok();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 문서 저장 글꼴 이름이 267개인 서식의 공급을 합친다. 공급은 네트워크 없이 고정한다.
#[test]
fn form_declaring_more_than_256_font_names_merges() {
    let doc_info = fixture_doc_info("tax_source_ledger_form.hwp");
    let mut supply = NativeWebfontSupply::default();
    for request in FontEnvironment::document_requests(&doc_info) {
        supply
            .mappings
            .insert(request.source, "NotoSansKR-Regular".into());
    }
    assert!(supply.mappings.len() > 256, "전제: 256개를 넘는 문서");
    let merged = merged_json(&supply, None);
    assert_eq!(
        merged["substitutions"].as_object().unwrap().len(),
        supply.mappings.len()
    );
}

/// 공급 대상과 같은 글꼴을 쓰는 문서의 자기 참조 문자 대체를 합친다.
#[test]
fn report_whose_supply_names_its_own_face_merges() {
    let doc_info = fixture_doc_info("employment_service_interim_report.hwp");
    let source = FontEnvironment::document_requests(&doc_info)
        .into_iter()
        .map(|request| request.source)
        .find(|source| source == "HCRBatang")
        .expect("전제: 문서가 HCRBatang 을 쓴다");
    let supply = NativeWebfontSupply {
        glyph_fallbacks: vec![fallback(&source, &[&source])],
        ..Default::default()
    };
    assert!(supply.merge_into_environment(None).is_ok());
}
