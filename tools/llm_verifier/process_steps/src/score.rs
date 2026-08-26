//! 검사 관측 → 과정 보상. 순위 함수가 아니다.

use crate::check::{CheckKind, CheckObservation, CheckVerdict};
use crate::envelope::CheckFields;
use crate::exit_class::ExitClass;
use crate::reward::ProcessReward;

/// 한 검사의 합격/불합격을 종료코드와 봉투 필드로만 결정한다.
pub fn score_check(obs: &CheckObservation) -> CheckVerdict {
    let fields = if obs.has_envelope() {
        if let Some(env) = &obs.envelope {
            crate::envelope::extract_check_fields(obs.check, env)
        } else {
            obs.fields.clone()
        }
    } else {
        obs.fields.clone()
    };
    let signals: Vec<String> = fields
        .fail_signals(obs.check)
        .into_iter()
        .map(str::to_string)
        .collect();
    let (pass, consistent, rule_id) =
        decide(obs.check, obs.exit_class, obs.has_envelope(), &fields);
    CheckVerdict {
        check: obs.check,
        exit_class: obs.exit_class,
        pass,
        consistent,
        fail_signals: signals,
        rule_id: rule_id.to_string(),
    }
}

fn decide(
    check: CheckKind,
    exit: ExitClass,
    has_envelope: bool,
    fields: &CheckFields,
) -> (bool, bool, &'static str) {
    match exit {
        ExitClass::Io => (false, true, "exit.io"),
        ExitClass::Usage => (false, true, "exit.usage"),
        ExitClass::PageVerify => page_verify(has_envelope, fields),
        ExitClass::Ok => ok_path(check, has_envelope, fields),
        ExitClass::Judgment => judgment_path(check, has_envelope, fields),
    }
}

fn page_verify(has_envelope: bool, fields: &CheckFields) -> (bool, bool, &'static str) {
    if !has_envelope {
        return (false, false, "page_verify.missing-envelope");
    }
    if fields.page_count_mismatch == Some(false)
        && fields.verify_identical != Some(false)
        && fields.fail_count.unwrap_or(0) == 0
        && !page_counts_differ(fields)
    {
        return (false, false, "page_verify.exit4-but-ok-fields");
    }
    (false, true, "exit.page_verify")
}

fn page_counts_differ(fields: &CheckFields) -> bool {
    match (fields.page_count, fields.expected_page_count) {
        (Some(a), Some(e)) => a != e,
        _ => false,
    }
}

fn ok_path(
    check: CheckKind,
    has_envelope: bool,
    fields: &CheckFields,
) -> (bool, bool, &'static str) {
    if !has_envelope {
        return (false, false, "ok.missing-envelope");
    }
    let signals = fields.fail_signals(check);
    if !signals.is_empty() {
        return (false, false, "ok.fail-signal-present");
    }
    match check {
        CheckKind::Verify
            if fields
                .verdict
                .as_deref()
                .is_some_and(|v| v.eq_ignore_ascii_case("fail")) =>
        {
            (false, false, "ok.verify-fail-verdict")
        }
        CheckKind::LayoutAnomaly if layout_strict_fail(fields) => {
            (false, false, "ok.layout-strict-signal")
        }
        CheckKind::PageCount if page_counts_differ(fields) => {
            (false, false, "ok.page-count-differ")
        }
        CheckKind::FillVerify if fields.verify_identical == Some(false) => {
            (false, false, "ok.fill-verify-false")
        }
        _ => (true, true, "exit.ok"),
    }
}

fn judgment_path(
    check: CheckKind,
    has_envelope: bool,
    fields: &CheckFields,
) -> (bool, bool, &'static str) {
    if !has_envelope {
        return (false, false, "judgment.missing-envelope");
    }
    if explicit_success(check, fields) {
        return (false, false, "judgment.success-fields");
    }
    (false, true, "exit.judgment")
}

fn layout_strict_fail(fields: &CheckFields) -> bool {
    fields.strict == Some(true)
        && (fields.overflow_count.unwrap_or(0) > 0 || fields.overlap_count.unwrap_or(0) > 0)
}

fn explicit_success(check: CheckKind, fields: &CheckFields) -> bool {
    match check {
        CheckKind::Verify => {
            fields.verdict.as_deref() == Some("pass") && fields.fail_count == Some(0)
        }
        CheckKind::LayoutAnomaly => {
            fields.has_signal == Some(false)
                && fields.overflow_count.unwrap_or(0) == 0
                && fields.overlap_count.unwrap_or(0) == 0
        }
        CheckKind::PageCount => {
            fields.page_count_mismatch == Some(false)
                && !page_counts_differ(fields)
                && fields.page_count.is_some()
        }
        CheckKind::FillVerify => {
            fields.verify_identical == Some(true) && fields.verify_diff_count == Some(0)
        }
    }
}

/// 네 검사의 pass/fail 을 모아 과정 보상을 만든다. 순위를 매기지 않는다.
pub fn score_step(checks: &[CheckObservation]) -> ProcessReward {
    let mut pass_count = 0u64;
    let mut fail_count = 0u64;
    let mut failed_checks = Vec::new();
    let mut worst = 0u8;
    let mut consistent = true;
    for obs in checks {
        let v = score_check(obs);
        if !v.consistent {
            consistent = false;
        }
        if v.pass {
            pass_count += 1;
        } else {
            fail_count += 1;
            failed_checks.push(obs.check.as_str().to_string());
        }
        worst = worst.max(obs.exit_class.code());
    }
    ProcessReward {
        pass: fail_count == 0 && !checks.is_empty() && consistent,
        check_count: checks.len() as u64,
        pass_count,
        fail_count,
        failed_checks,
        worst_exit_class: worst,
        consistent,
    }
}

/// 기록된 보상과 재계산이 같은지 본다.
pub fn scored_reward(step_checks: &[CheckObservation], recorded: &ProcessReward) -> bool {
    let computed = score_step(step_checks);
    computed.pass == recorded.pass
        && computed.fail_count == recorded.fail_count
        && computed.pass_count == recorded.pass_count
        && computed.failed_checks == recorded.failed_checks
        && computed.worst_exit_class == recorded.worst_exit_class
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obs(
        check: CheckKind,
        exit: ExitClass,
        envelope: Option<serde_json::Value>,
    ) -> CheckObservation {
        let mut o = CheckObservation {
            check,
            argv: vec![],
            exit_class: exit,
            pass: false,
            fail_signals: vec![],
            envelope,
            fields: CheckFields::default(),
        };
        o.refresh_fields();
        o
    }

    #[test]
    fn verify_pass_exit0() {
        let o = obs(
            CheckKind::Verify,
            ExitClass::Ok,
            Some(json!({"verdict":"pass","passCount":3,"failCount":0})),
        );
        let v = score_check(&o);
        assert!(v.pass);
        assert!(v.consistent);
        assert_eq!(v.rule_id, "exit.ok");
    }

    #[test]
    fn verify_fail_exit3() {
        let o = obs(
            CheckKind::Verify,
            ExitClass::Judgment,
            Some(json!({"verdict":"fail","passCount":2,"failCount":1})),
        );
        let v = score_check(&o);
        assert!(!v.pass);
        assert!(v.fail_signals.iter().any(|s| s == "failCount>0"));
    }

    #[test]
    fn layout_strict_overflow_is_fail() {
        let o = obs(
            CheckKind::LayoutAnomaly,
            ExitClass::Judgment,
            Some(json!({
                "hasSignal": true,
                "strict": true,
                "overflowCount": 2,
                "overlapCount": 0,
                "emptyPageCount": 0
            })),
        );
        let v = score_check(&o);
        assert!(!v.pass);
        assert_eq!(v.rule_id, "exit.judgment");
    }

    #[test]
    fn pagecount_mismatch_exit4() {
        let o = obs(
            CheckKind::PageCount,
            ExitClass::PageVerify,
            Some(json!({
                "pageCount": 5,
                "expectedPageCount": 4,
                "pageCountMismatch": true
            })),
        );
        let v = score_check(&o);
        assert!(!v.pass);
        assert_eq!(v.rule_id, "exit.page_verify");
    }

    #[test]
    fn fill_verify_identical_false() {
        let o = obs(
            CheckKind::FillVerify,
            ExitClass::Judgment,
            Some(json!({
                "filledCount": 2,
                "verify": {"identical": false, "diffCount": 3}
            })),
        );
        let v = score_check(&o);
        assert!(!v.pass);
        assert!(v.fail_signals.iter().any(|s| s == "verify.identical=false"));
    }

    #[test]
    fn exit0_with_fail_signal_is_inconsistent() {
        let o = obs(
            CheckKind::Verify,
            ExitClass::Ok,
            Some(json!({"verdict":"fail","failCount":1,"passCount":0})),
        );
        let v = score_check(&o);
        assert!(!v.pass);
        assert!(!v.consistent);
    }

    #[test]
    fn step_reward_fails_if_any_check_fails() {
        let checks = vec![
            obs(
                CheckKind::Verify,
                ExitClass::Ok,
                Some(json!({"verdict":"pass","failCount":0,"passCount":2})),
            ),
            obs(
                CheckKind::LayoutAnomaly,
                ExitClass::Ok,
                Some(json!({
                    "hasSignal": false,
                    "strict": false,
                    "overflowCount": 0,
                    "overlapCount": 0,
                    "emptyPageCount": 0
                })),
            ),
            obs(
                CheckKind::PageCount,
                ExitClass::PageVerify,
                Some(json!({
                    "pageCount": 6,
                    "expectedPageCount": 5,
                    "pageCountMismatch": true
                })),
            ),
            obs(
                CheckKind::FillVerify,
                ExitClass::Ok,
                Some(json!({"verify":{"identical":true,"diffCount":0},"filledCount":1})),
            ),
        ];
        let r = score_step(&checks);
        assert!(!r.pass);
        assert_eq!(r.fail_count, 1);
        assert_eq!(r.failed_checks, vec!["pageCount".to_string()]);
        assert_eq!(r.worst_exit_class, 4);
        assert!(r.consistent);
    }

    #[test]
    fn all_pass_is_process_reward_pass() {
        let checks = vec![
            obs(
                CheckKind::Verify,
                ExitClass::Ok,
                Some(json!({"verdict":"pass","failCount":0,"passCount":4})),
            ),
            obs(
                CheckKind::LayoutAnomaly,
                ExitClass::Ok,
                Some(json!({
                    "hasSignal": false,
                    "strict": true,
                    "overflowCount": 0,
                    "overlapCount": 0,
                    "emptyPageCount": 1
                })),
            ),
            obs(
                CheckKind::PageCount,
                ExitClass::Ok,
                Some(json!({
                    "pageCount": 3,
                    "expectedPageCount": 3,
                    "pageCountMismatch": false
                })),
            ),
            obs(
                CheckKind::FillVerify,
                ExitClass::Ok,
                Some(json!({"verify":{"identical":true,"diffCount":0},"filledCount":4})),
            ),
        ];
        let r = score_step(&checks);
        assert!(r.pass);
        assert_eq!(r.fail_count, 0);
        assert!(r.failed_checks.is_empty());
    }
}
