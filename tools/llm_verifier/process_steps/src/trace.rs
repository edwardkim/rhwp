//! 편집 한 스텝의 과정 추적 레코드.

use crate::check::CheckObservation;
use crate::exit_class::ExitClass;
use crate::reward::ProcessReward;
use crate::step_kind::StepKind;
use serde::{Deserialize, Serialize};

/// 편집 한 스텝 + 그 직후 기계 검사 + 과정 보상.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStep {
    pub record_id: String,
    pub episode_id: String,
    pub source_tag: String,
    pub step_index: u32,
    pub step_kind: StepKind,
    pub source: String,
    #[serde(default)]
    pub argv: Vec<String>,
    pub edit_exit_class: ExitClass,
    pub checks: Vec<CheckObservation>,
    pub process_reward: ProcessReward,
}

impl ProcessStep {
    pub fn refresh(&mut self) {
        for c in &mut self.checks {
            c.refresh_fields();
        }
    }

    pub fn uniqueness_key(&self) -> String {
        let checks: Vec<String> = self
            .checks
            .iter()
            .map(CheckObservation::fingerprint)
            .collect();
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.step_kind.as_str(),
            self.step_index,
            checks.join(";"),
            self.process_reward.fingerprint(),
            self.edit_exit_class.code(),
            self.source_tag
        )
    }
}

/// (stepKind, stepIndex, check fingerprints, processReward, sourceTag)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniquenessKey {
    pub step_kind: String,
    pub step_index: u32,
    pub checks: String,
    pub reward: String,
    pub source_tag: String,
}

impl UniquenessKey {
    pub fn from_step(step: &ProcessStep) -> Self {
        Self {
            step_kind: step.step_kind.as_str().to_string(),
            step_index: step.step_index,
            checks: step
                .checks
                .iter()
                .map(CheckObservation::fingerprint)
                .collect::<Vec<_>>()
                .join(";"),
            reward: step.process_reward.fingerprint(),
            source_tag: step.source_tag.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::CheckKind;
    use crate::envelope::CheckFields;
    use serde_json::json;

    #[test]
    fn uniqueness_includes_source_tag() {
        let check = CheckObservation {
            check: CheckKind::Verify,
            argv: vec![],
            exit_class: ExitClass::Ok,
            pass: true,
            fail_signals: vec![],
            envelope: Some(json!({"verdict":"pass","failCount":0})),
            fields: CheckFields::default(),
        };
        let mut a = ProcessStep {
            record_id: "a".into(),
            episode_id: "e".into(),
            source_tag: "gov/a#fill-fields/s0".into(),
            step_index: 0,
            step_kind: StepKind::FillFields,
            source: "a.hwp".into(),
            argv: vec![],
            edit_exit_class: ExitClass::Ok,
            checks: vec![check],
            process_reward: ProcessReward {
                pass: true,
                check_count: 1,
                pass_count: 1,
                fail_count: 0,
                failed_checks: vec![],
                worst_exit_class: 0,
                consistent: true,
            },
        };
        let mut b = a.clone();
        b.source_tag = "gov/b#fill-fields/s0".into();
        a.refresh();
        b.refresh();
        assert_ne!(a.uniqueness_key(), b.uniqueness_key());
    }
}
