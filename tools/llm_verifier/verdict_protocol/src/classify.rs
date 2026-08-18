//! 합격/불합격은 종료코드와 봉투 필드만으로 결정한다.

use crate::command::CommandFamily;
use crate::exit_class::ExitClass;
use crate::judgment::JudgmentFields;
use crate::observation::Observation;
use serde::{Deserialize, Serialize};

/// 기계 판정. 산문 등급이 아니라 닫힌 열거.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineVerdict {
    Pass,
    IoFail,
    UsageFail,
    JudgmentFail,
    PageVerifyFail,
    Inconsistent,
}

impl MachineVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::IoFail => "io_fail",
            Self::UsageFail => "usage_fail",
            Self::JudgmentFail => "judgment_fail",
            Self::PageVerifyFail => "page_verify_fail",
            Self::Inconsistent => "inconsistent",
        }
    }

    pub fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// 분류기 출력. 근거는 필드 이름과 값뿐이다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolDecision {
    pub exit_class: ExitClass,
    pub exit_name: String,
    pub machine_verdict: MachineVerdict,
    pub consistent: bool,
    pub has_envelope: bool,
    pub command: CommandFamily,
    pub judgment: JudgmentFields,
    pub fail_signals: Vec<String>,
    pub rule_id: String,
}

impl ProtocolDecision {
    pub fn is_pass(&self) -> bool {
        self.machine_verdict.is_pass()
    }
}

/// 단일 프로토콜. 입력은 Observation (exit + 기존 봉투).
pub fn classify(obs: &Observation) -> ProtocolDecision {
    let has_envelope = obs.has_envelope();
    let judgment = if has_envelope {
        if let Some(env) = &obs.envelope {
            crate::extract::extract_judgment(env)
        } else {
            obs.judgment.clone()
        }
    } else {
        obs.judgment.clone()
    };
    let fail_signals: Vec<String> = judgment
        .fail_signals()
        .into_iter()
        .map(str::to_string)
        .collect();
    let (verdict, rule_id) = decide(obs.command, obs.exit_class, has_envelope, &judgment);
    ProtocolDecision {
        exit_class: obs.exit_class,
        exit_name: obs.exit_class.name().to_string(),
        consistent: !matches!(verdict, MachineVerdict::Inconsistent),
        machine_verdict: verdict,
        has_envelope,
        command: obs.command,
        judgment,
        fail_signals,
        rule_id: rule_id.to_string(),
    }
}

fn decide(
    command: CommandFamily,
    exit: ExitClass,
    has_envelope: bool,
    judgment: &JudgmentFields,
) -> (MachineVerdict, &'static str) {
    match exit {
        ExitClass::Io => (MachineVerdict::IoFail, "exit.io"),
        ExitClass::Usage => (MachineVerdict::UsageFail, "exit.usage"),
        ExitClass::PageVerify => page_verify(has_envelope, judgment),
        ExitClass::Ok => ok_path(command, has_envelope, judgment),
        ExitClass::Judgment => judgment_path(command, has_envelope, judgment),
    }
}

fn page_verify(has_envelope: bool, judgment: &JudgmentFields) -> (MachineVerdict, &'static str) {
    if !has_envelope {
        return (MachineVerdict::Inconsistent, "page_verify.missing-envelope");
    }
    if judgment.page_count_mismatch == Some(false)
        && judgment.verify.as_ref().and_then(|v| v.identical) == Some(true)
        && !judgment.has_any_fail_signal()
    {
        return (
            MachineVerdict::Inconsistent,
            "page_verify.exit4-but-ok-fields",
        );
    }
    (MachineVerdict::PageVerifyFail, "exit.page_verify")
}

fn ok_path(
    command: CommandFamily,
    has_envelope: bool,
    judgment: &JudgmentFields,
) -> (MachineVerdict, &'static str) {
    if !has_envelope {
        return (MachineVerdict::Inconsistent, "ok.missing-envelope");
    }
    if judgment.has_any_fail_signal() {
        return (MachineVerdict::Inconsistent, "ok.fail-signal-present");
    }
    match command {
        CommandFamily::LayoutAnomaly if judgment.layout_strict_fail() => {
            (MachineVerdict::Inconsistent, "ok.layout-strict-signal")
        }
        CommandFamily::Replay
            if judgment.reproduced.as_ref().and_then(|r| r.as_bool()) == Some(false) =>
        {
            (MachineVerdict::Inconsistent, "ok.replay-not-reproduced")
        }
        _ => (MachineVerdict::Pass, "exit.ok"),
    }
}

fn judgment_path(
    command: CommandFamily,
    has_envelope: bool,
    judgment: &JudgmentFields,
) -> (MachineVerdict, &'static str) {
    if !has_envelope {
        return (MachineVerdict::Inconsistent, "judgment.missing-envelope");
    }
    if command_has_explicit_success(command, judgment) {
        return (MachineVerdict::Inconsistent, "judgment.success-fields");
    }
    (MachineVerdict::JudgmentFail, "exit.judgment")
}

fn command_has_explicit_success(command: CommandFamily, judgment: &JudgmentFields) -> bool {
    match command {
        CommandFamily::IrDiff => judgment.identical == Some(true) && judgment.diff_count == Some(0),
        CommandFamily::Verify => {
            judgment.verdict.as_deref() == Some("pass") && judgment.fail_count == Some(0)
        }
        CommandFamily::RenderDiff => {
            judgment.regression == Some(false)
                && judgment
                    .status
                    .as_deref()
                    .is_some_and(|s| s.eq_ignore_ascii_case("OK"))
        }
        CommandFamily::Replay => {
            judgment.reproduced.as_ref().and_then(|r| r.as_bool()) == Some(true)
        }
        CommandFamily::FillFields => {
            judgment.verify.as_ref().and_then(|v| v.identical) == Some(true)
                && judgment.verify.as_ref().and_then(|v| v.diff_count) == Some(0)
        }
        CommandFamily::LayoutAnomaly => {
            judgment.has_signal == Some(false)
                && judgment.overflow_count.unwrap_or(0) == 0
                && judgment.overlap_count.unwrap_or(0) == 0
        }
        CommandFamily::Info => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judgment::VerifyBlock;
    use serde_json::{json, Value};

    fn obs(command: CommandFamily, exit: ExitClass, envelope: Option<Value>) -> Observation {
        let mut o = Observation {
            record_id: "t".into(),
            source_tag: "t#x".into(),
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
    fn ir_diff_identical_true_is_pass() {
        let o = obs(
            CommandFamily::IrDiff,
            ExitClass::Ok,
            Some(json!({"identical": true, "diffCount": 0})),
        );
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::Pass);
        assert!(d.consistent);
    }

    #[test]
    fn ir_diff_identical_false_exit3() {
        let o = obs(
            CommandFamily::IrDiff,
            ExitClass::Judgment,
            Some(json!({"identical": false, "diffCount": 2})),
        );
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::JudgmentFail);
        assert!(d.fail_signals.iter().any(|s| s == "identical=false"));
    }

    #[test]
    fn exit0_with_identical_false_is_inconsistent() {
        let o = obs(
            CommandFamily::IrDiff,
            ExitClass::Ok,
            Some(json!({"identical": false, "diffCount": 1})),
        );
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::Inconsistent);
    }

    #[test]
    fn exit3_without_envelope_is_inconsistent() {
        let o = obs(CommandFamily::Verify, ExitClass::Judgment, None);
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::Inconsistent);
        assert_eq!(d.rule_id, "judgment.missing-envelope");
    }

    #[test]
    fn io_without_envelope_is_io_fail() {
        let o = obs(CommandFamily::Info, ExitClass::Io, None);
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::IoFail);
    }

    #[test]
    fn fill_fields_verify_identical_false_exit3() {
        let o = obs(
            CommandFamily::FillFields,
            ExitClass::Judgment,
            Some(json!({
                "filledCount": 1,
                "verify": { "identical": false, "diffCount": 3 }
            })),
        );
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::JudgmentFail);
        assert!(d.fail_signals.iter().any(|s| s == "verify.identical=false"));
    }

    #[test]
    fn replay_attest_null_is_pass() {
        let o = obs(
            CommandFamily::Replay,
            ExitClass::Ok,
            Some(json!({"mode":"attest","reproduced":null})),
        );
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::Pass);
    }

    #[test]
    fn layout_strict_overflow_exit3() {
        let o = obs(
            CommandFamily::LayoutAnomaly,
            ExitClass::Judgment,
            Some(json!({
                "hasSignal": true,
                "strict": true,
                "overflowCount": 2,
                "overlapCount": 0,
                "emptyPageCount": 0
            })),
        );
        let d = classify(&o);
        assert_eq!(d.machine_verdict, MachineVerdict::JudgmentFail);
    }

    #[test]
    fn verify_block_roundtrip_fields() {
        let j = JudgmentFields {
            verify: Some(VerifyBlock {
                identical: Some(false),
                diff_count: Some(1),
            }),
            ..Default::default()
        };
        assert!(j.has_any_fail_signal());
    }
}
