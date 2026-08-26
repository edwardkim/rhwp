use llm_verifier_process_steps::{
    score_check, score_step, CheckKind, CheckObservation, ExitClass, ProcessReward,
};
use serde_json::json;

fn obs(check: CheckKind, exit: ExitClass, envelope: Option<serde_json::Value>) -> CheckObservation {
    let mut o = CheckObservation {
        check,
        argv: vec![],
        exit_class: exit,
        pass: false,
        fail_signals: vec![],
        envelope,
        fields: Default::default(),
    };
    o.refresh_fields();
    o
}

#[test]
fn io_is_fail_without_envelope() {
    let o = obs(CheckKind::Verify, ExitClass::Io, None);
    let v = score_check(&o);
    assert!(!v.pass);
    assert_eq!(v.rule_id, "exit.io");
}

#[test]
fn usage_is_fail() {
    let o = obs(CheckKind::LayoutAnomaly, ExitClass::Usage, None);
    let v = score_check(&o);
    assert!(!v.pass);
    assert_eq!(v.rule_id, "exit.usage");
}

#[test]
fn fill_verify_ok_requires_identical_true() {
    let o = obs(
        CheckKind::FillVerify,
        ExitClass::Ok,
        Some(json!({"verify":{"identical":true,"diffCount":0},"filledCount":2})),
    );
    assert!(score_check(&o).pass);
}

#[test]
fn empty_step_is_not_a_pass() {
    let r = score_step(&[]);
    assert!(!r.pass);
    assert_eq!(r.check_count, 0);
}

#[test]
fn process_reward_is_not_a_ranking() {
    let checks = vec![obs(
        CheckKind::PageCount,
        ExitClass::Ok,
        Some(json!({
            "pageCount": 2,
            "expectedPageCount": 2,
            "pageCountMismatch": false
        })),
    )];
    let r: ProcessReward = score_step(&checks);
    let text = serde_json::to_string(&r).unwrap();
    assert!(!text.contains("rank"));
    assert!(!text.contains("bestOfN"));
    assert!(!text.contains("\"score\""));
}
