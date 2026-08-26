use llm_verifier_verdict_protocol::{
    classify, CommandFamily, ExitClass, JudgmentFields, MachineVerdict, Observation,
};
use serde_json::json;

fn obs(
    command: CommandFamily,
    exit: ExitClass,
    envelope: Option<serde_json::Value>,
) -> Observation {
    let mut o = Observation {
        record_id: "unit".into(),
        source_tag: format!("{}#unit", command.as_str()),
        command,
        argv: vec![],
        exit_class: exit,
        stdout_present: envelope.is_some(),
        stderr_kind: None,
        envelope,
        judgment: JudgmentFields::default(),
    };
    o.refresh_judgment();
    o
}

#[test]
fn exit_codes_map_to_verdicts() {
    let cases = [
        (ExitClass::Io, MachineVerdict::IoFail),
        (ExitClass::Usage, MachineVerdict::UsageFail),
    ];
    for (exit, want) in cases {
        let d = classify(&obs(CommandFamily::Info, exit, None));
        assert_eq!(d.machine_verdict, want, "{exit}");
    }
}

#[test]
fn render_diff_regression_is_judgment_fail() {
    let d = classify(&obs(
        CommandFamily::RenderDiff,
        ExitClass::Judgment,
        Some(json!({
            "regression": true,
            "status": "OVER",
            "maxDisp": 6.2,
            "overPages": 2
        })),
    ));
    assert_eq!(d.machine_verdict, MachineVerdict::JudgmentFail);
    assert!(d.fail_signals.iter().any(|s| s == "regression=true"));
}

#[test]
fn layout_nonstrict_signal_is_pass() {
    let d = classify(&obs(
        CommandFamily::LayoutAnomaly,
        ExitClass::Ok,
        Some(json!({
            "hasSignal": true,
            "strict": false,
            "overflowCount": 2,
            "overlapCount": 0,
            "emptyPageCount": 0
        })),
    ));
    assert_eq!(d.machine_verdict, MachineVerdict::Pass);
}

#[test]
fn verify_pass_envelope_exit3_is_inconsistent() {
    let d = classify(&obs(
        CommandFamily::Verify,
        ExitClass::Judgment,
        Some(json!({
            "verdict": "pass",
            "passCount": 3,
            "failCount": 0
        })),
    ));
    assert_eq!(d.machine_verdict, MachineVerdict::Inconsistent);
}

#[test]
fn page_verify_exit4() {
    let d = classify(&obs(
        CommandFamily::FillFields,
        ExitClass::PageVerify,
        Some(json!({
            "pageCountMismatch": true,
            "verify": { "identical": true, "diffCount": 0 }
        })),
    ));
    assert_eq!(d.machine_verdict, MachineVerdict::PageVerifyFail);
}
