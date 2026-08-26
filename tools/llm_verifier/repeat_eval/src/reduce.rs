//! K회 관측을 다수결·평균으로 줄인다.

use crate::check::{CheckSpec, ValueKind};
use crate::report::{FinalValue, ReduceReport};
use crate::row::RepeatRow;
use crate::trial::Trial;
use crate::variance::VarianceStats;
use crate::vote::VoteTally;

/// 같은 검사 K회를 축소한다. 후보를 순위 매기지 않는다.
pub fn reduce_trials(artifact_id: &str, check: &CheckSpec, trials: &[Trial]) -> ReduceReport {
    let k = trials.len() as u32;
    let observed: Vec<String> = trials.iter().map(|t| t.observe(check)).collect();
    let votes = VoteTally::from_values(&observed, check.value_kind);
    if check.value_kind.is_numeric() {
        let xs: Vec<f64> = trials.iter().filter_map(|t| t.numeric(check)).collect();
        let variance = VarianceStats::numeric(&xs);
        let mean = variance.mean.unwrap_or(0.0);
        ReduceReport {
            artifact_id: artifact_id.to_string(),
            k,
            check: check.name.clone(),
            votes,
            variance,
            final_value: FinalValue::from_mean(mean, None),
        }
    } else {
        let variance = VarianceStats::categorical(&observed, votes.majority_frac);
        let final_value = FinalValue::from_votes(&votes, check.value_kind);
        ReduceReport {
            artifact_id: artifact_id.to_string(),
            k,
            check: check.name.clone(),
            votes,
            variance,
            final_value,
        }
    }
}

/// 코퍼스 행을 다시 계산한다. 저장된 final 을 믿지 않는다.
pub fn reduce_row(row: &RepeatRow) -> ReduceReport {
    reduce_trials(&row.artifact.artifact_id, &row.check, &row.trials)
}

pub fn intended_numeric(row: &RepeatRow) -> Option<f64> {
    if row.check.value_kind != ValueKind::U64 {
        return None;
    }
    let env = row.artifact.intended.as_ref()?;
    let path = row.check.path.as_deref().unwrap_or(&row.check.name);
    crate::envelope::read_path(env, path).as_number()
}

pub fn reduce_row_with_intended(row: &RepeatRow) -> ReduceReport {
    let mut report = reduce_row(row);
    if row.check.value_kind.is_numeric() {
        if let Some(mean) = report.variance.mean {
            report.final_value = FinalValue::from_mean(mean, intended_numeric(row));
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::Artifact;
    use crate::command::CommandFamily;
    use crate::exit_class::ExitClass;
    use crate::row::RepeatRow;
    use serde_json::json;

    fn trial(seed: u64, exit: ExitClass, env: serde_json::Value) -> Trial {
        Trial {
            seed,
            exit_class: exit,
            observed: String::new(),
            envelope: Some(env),
        }
    }

    #[test]
    fn majority_reduces_bool_noise() {
        let check = CheckSpec::envelope_bool("verify.identical");
        let trials = vec![
            trial(0, ExitClass::Ok, json!({"verify":{"identical":true}})),
            trial(1, ExitClass::Ok, json!({"verify":{"identical":true}})),
            trial(2, ExitClass::Ok, json!({"verify":{"identical":true}})),
            trial(
                3,
                ExitClass::Judgment,
                json!({"verify":{"identical":false}}),
            ),
            trial(4, ExitClass::Ok, json!({"verify":{"identical":true}})),
        ];
        let r = reduce_trials("art", &check, &trials);
        assert_eq!(r.k, 5);
        assert_eq!(r.final_value.value, "true");
        assert!(r.final_value.pass);
        assert!((r.variance.disagreement - 0.2).abs() < 1e-9);
    }

    #[test]
    fn mean_reduces_count_jitter() {
        let check = CheckSpec::envelope_u64("filledCount");
        let trials = vec![
            trial(0, ExitClass::Ok, json!({"filledCount": 4})),
            trial(1, ExitClass::Ok, json!({"filledCount": 5})),
            trial(2, ExitClass::Ok, json!({"filledCount": 3})),
            trial(3, ExitClass::Ok, json!({"filledCount": 4})),
            trial(4, ExitClass::Ok, json!({"filledCount": 4})),
        ];
        let r = reduce_trials("art", &check, &trials);
        assert_eq!(r.final_value.reduce, crate::report::ReduceKind::Mean);
        assert!((r.final_value.numeric.unwrap() - 4.0).abs() < 1e-9);
        assert!(r.variance.sample_variance.unwrap() > 0.0);
    }

    #[test]
    fn row_recompute_ignores_stored_final() {
        let row = RepeatRow {
            schema_version: crate::schema::PROTOCOL_SCHEMA_VERSION.into(),
            claim: crate::schema::CLAIM_ID.into(),
            kind: crate::schema::KIND.into(),
            record_id: "t".into(),
            uniqueness_key: "t".into(),
            artifact: Artifact {
                artifact_id: "a".into(),
                command: CommandFamily::IrDiff,
                sample: "s.hwp".into(),
                argv: vec![],
                intended_exit: ExitClass::Ok,
                intended: Some(json!({"identical": true, "diffCount": 0})),
            },
            k: 3,
            check: CheckSpec::envelope_bool("identical"),
            trials: vec![
                trial(0, ExitClass::Ok, json!({"identical": true, "diffCount": 0})),
                trial(1, ExitClass::Ok, json!({"identical": true, "diffCount": 0})),
                trial(
                    2,
                    ExitClass::Judgment,
                    json!({"identical": false, "diffCount": 1}),
                ),
            ],
            votes: VoteTally::from_values(&["wrong".into()], ValueKind::Bool),
            variance: VarianceStats::categorical(&["wrong".into()], 1.0),
            final_value: FinalValue {
                reduce: crate::report::ReduceKind::Majority,
                value: "wrong".into(),
                tie: false,
                pass: false,
                numeric: None,
            },
            profile: Some("flip_one".into()),
        };
        let r = reduce_row(&row);
        assert_eq!(r.final_value.value, "true");
        assert_eq!(r.votes.majority.as_deref(), Some("true"));
    }
}
