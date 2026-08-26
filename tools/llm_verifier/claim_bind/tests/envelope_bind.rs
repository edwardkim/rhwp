//! 실측 형태 search/extract-data 봉투에 주장을 결속한다.

use llm_verifier_claim_bind::{bind_claim_to_envelope, NaturalClaim};
use serde_json::json;

fn load_unit(name: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("envelopes")
        .join(name);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&raw).expect("json")
}

#[test]
fn search_with_page_binds_matching_claim() {
    let env = load_unit("search_with_page.json");
    let claim = NaturalClaim::from_value(&json!({
        "rowId": "CB-ENV-001",
        "claimText": "본 사업의 핵심은 도시 데이터 플랫폼 구축이다.",
        "section": 0,
        "paragraph": 4,
        "page": 1,
        "charOffset": 8,
        "envelopeKind": "search"
    }))
    .unwrap();
    let d = bind_claim_to_envelope(&claim, &env);
    assert!(d.is_pass(), "{d:?}");
    assert!(d.coords_present);
}

#[test]
fn search_missing_page_does_not_bind() {
    let env = load_unit("search_missing_page.json");
    let claim = NaturalClaim::from_value(&json!({
        "rowId": "CB-ENV-002",
        "claimText": "머리말 안내문은 아직 조판에 배치되지 않았다.",
        "section": 0,
        "paragraph": 0,
        "charOffset": 3,
        "envelopeKind": "search"
    }))
    .unwrap();
    let d = bind_claim_to_envelope(&claim, &env);
    assert!(!d.is_pass());
    assert_eq!(
        d.fail_kind,
        Some(llm_verifier_claim_bind::FailKind::IncompleteCoords)
    );
}

#[test]
fn extract_amount_binds() {
    let env = load_unit("extract_amount.json");
    let claim = NaturalClaim::from_value(&json!({
        "rowId": "CB-ENV-003",
        "claimText": "사업 예산은 3,180백만원으로 명시되어 있다.",
        "section": 0,
        "paragraph": 7,
        "page": 0,
        "charOffset": 55,
        "envelopeKind": "extract-data"
    }))
    .unwrap();
    let d = bind_claim_to_envelope(&claim, &env);
    assert!(d.is_pass(), "{d:?}");
}

#[test]
fn extract_date_binds() {
    let env = load_unit("extract_date.json");
    let claim = NaturalClaim::from_value(&json!({
        "rowId": "CB-ENV-004",
        "claimText": "제안서 마감일은 2026-09-12 이다.",
        "section": 0,
        "paragraph": 1,
        "page": 0,
        "charOffset": 6,
        "envelopeKind": "extract-data"
    }))
    .unwrap();
    let d = bind_claim_to_envelope(&claim, &env);
    assert!(d.is_pass(), "{d:?}");
}

#[test]
fn search_cell_binds_with_cell_in_field_set() {
    let env = load_unit("search_cell.json");
    let claim = NaturalClaim::from_value(&json!({
        "rowId": "CB-ENV-005",
        "claimText": "기술평가 배점은 80점이다.",
        "section": 0,
        "paragraph": 0,
        "page": 5,
        "charOffset": 0,
        "cell": {"row": 2, "col": 1},
        "envelopeKind": "search"
    }))
    .unwrap();
    let d = bind_claim_to_envelope(&claim, &env);
    assert!(d.is_pass(), "{d:?}");
    assert!(d.field_set.iter().any(|k| k == "cell"));
}

#[test]
fn wrong_paragraph_is_envelope_mismatch() {
    let env = load_unit("search_with_page.json");
    let claim = NaturalClaim::from_value(&json!({
        "rowId": "CB-ENV-006",
        "claimText": "본 사업의 핵심은 도시 데이터 플랫폼 구축이다.",
        "section": 0,
        "paragraph": 99,
        "page": 1,
        "charOffset": 8,
        "envelopeKind": "search"
    }))
    .unwrap();
    let d = bind_claim_to_envelope(&claim, &env);
    assert_eq!(
        d.fail_kind,
        Some(llm_verifier_claim_bind::FailKind::EnvelopeMismatch)
    );
}
