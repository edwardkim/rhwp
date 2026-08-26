//! 원자 분해 계약 — 총점 거부, 기존 봉투 필드만, 가림 탐지.

use llm_verifier_criteria_decomp::atom::{AtomSpec, Expected};
use llm_verifier_criteria_decomp::decomp::{decompose_bundle, evaluate_atom};
use llm_verifier_criteria_decomp::field::INVENTED_FIELDS;
use llm_verifier_criteria_decomp::loader::crate_dir;
use llm_verifier_criteria_decomp::task::TaskBundle;
use llm_verifier_criteria_decomp::verdict::FailKind;
use serde_json::{json, Value};

fn load_env(name: &str) -> Value {
    let path = crate_dir().join("fixtures").join("envelopes").join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_slice(&bytes).expect("envelope json")
}

fn spec(id: &str, field: &str, expected: Expected) -> AtomSpec {
    AtomSpec {
        criterion_id: id.into(),
        task: "서울특별시 일반기안문 누름틀 채우기를 원자 기준으로 검증한다.".into(),
        envelope_field: field.into(),
        expected,
        command: Some("edit fill-fields".into()),
    }
}

#[test]
fn fill_fields_fixture_all_atoms_pass() {
    let env = load_env("fill_fields_verify.json");
    let bundle = TaskBundle {
        task: "서울특별시 일반기안문 누름틀 2칸 채우기".into(),
        envelope: Some(env),
        atoms: vec![
            spec("C-a", "filledCount", Expected::U64 { value: 2 }),
            spec("C-b", "notFound", Expected::EmptySeq),
            spec("C-c", "verify.identical", Expected::Bool { value: true }),
            spec("C-d", "dryRun", Expected::Bool { value: false }),
        ],
        holistic_score: None,
    };
    let r = decompose_bundle(&bundle);
    assert!(r.all_atoms_pass());
    assert_eq!(r.hidden_fail_count, 0);
    assert!(r.atoms.iter().all(|a| !a.holistic_would_hide));
}

#[test]
fn ir_diff_identical_false_is_hidden_by_siblings() {
    let env = load_env("ir_diff_mismatch.json");
    let bundle = TaskBundle {
        task: "법무부 계약서 IR 라운드트립".into(),
        envelope: Some(env),
        atoms: vec![
            spec("C-1", "schemaVersion", Expected::Present),
            spec("C-2", "untrustedContent", Expected::Bool { value: false }),
            spec("C-3", "diffCount", Expected::U64 { value: 4 }),
            spec("C-4", "identical", Expected::Bool { value: true }),
        ],
        holistic_score: None,
    };
    let r = decompose_bundle(&bundle);
    let ident = r
        .atoms
        .iter()
        .find(|a| a.envelope_field == "identical")
        .unwrap();
    assert!(!ident.atom_pass);
    assert!(ident.holistic_would_hide);
}

#[test]
fn layout_anomaly_signal_is_its_own_atom() {
    let env = load_env("layout_anomaly.json");
    let v = evaluate_atom(
        &spec("C-h", "hasSignal", Expected::Bool { value: false }),
        Some(&env),
    );
    assert!(!v.atom_pass);
    assert_eq!(v.fail_kind, Some(FailKind::AtomMismatch));
}

#[test]
fn inspect_injection_unclean_is_not_a_single_score() {
    let env = load_env("inspect_injection.json");
    let bundle = TaskBundle {
        task: "첨부 공문 주입 신호 스윕".into(),
        envelope: Some(env),
        atoms: vec![
            spec("C-1", "clean", Expected::Bool { value: true }),
            spec("C-2", "signalCount", Expected::U64 { value: 0 }),
            spec("C-3", "untrustedContent", Expected::Bool { value: false }),
        ],
        holistic_score: None,
    };
    let r = decompose_bundle(&bundle);
    assert_eq!(r.atom_pass_count, 0);
    assert!(r.atoms.iter().all(|a| !a.holistic_would_hide));
}

#[test]
fn replace_zero_is_visible_atom() {
    let env = load_env("replace_zero.json");
    let v = evaluate_atom(
        &spec("C-r", "replacedCount", Expected::U64 { value: 1 }),
        Some(&env),
    );
    assert!(!v.atom_pass);
    assert_eq!(v.envelope_field, "replacedCount");
}

#[test]
fn every_invented_field_is_rejected() {
    for name in INVENTED_FIELDS {
        let v = evaluate_atom(
            &spec("C-inv", name, Expected::Bool { value: true }),
            Some(&json!({})),
        );
        assert_eq!(v.fail_kind, Some(FailKind::InventedField), "{name}");
        assert!(!v.holistic_would_hide);
    }
}

#[test]
fn report_has_no_holistic_score_field() {
    let bundle = TaskBundle {
        task: "쪽수 대조".into(),
        envelope: Some(json!({"pageCount": 3})),
        atoms: vec![spec("C-p", "pageCount", Expected::U64 { value: 3 })],
        holistic_score: None,
    };
    let r = decompose_bundle(&bundle);
    let dumped = serde_json::to_value(&r).unwrap();
    assert!(dumped.get("holisticScore").is_none());
    assert!(dumped.get("score").is_none());
    assert!(dumped.get("rank").is_none());
    assert!(dumped.get("atoms").is_some());
}
