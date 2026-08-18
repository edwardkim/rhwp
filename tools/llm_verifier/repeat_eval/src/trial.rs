//! K번 독립 봉투 읽기 한 회. 시드가 다르다.

use crate::check::{CheckSpec, ValueKind};
use crate::envelope::{fail_signals, read_path, Observed};
use crate::exit_class::ExitClass;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 한 시드의 관측.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trial {
    pub seed: u64,
    pub exit_class: ExitClass,
    pub observed: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Value>,
}

impl Trial {
    pub fn observe(&self, check: &CheckSpec) -> String {
        match check.kind {
            crate::check::CheckKind::ExitClass => self.exit_class.code().to_string(),
            crate::check::CheckKind::PassFail => {
                if self.is_pass() {
                    "pass".into()
                } else {
                    "fail".into()
                }
            }
            crate::check::CheckKind::EnvelopeField => match &self.envelope {
                None => "missing".into(),
                Some(env) => {
                    let path = check.path.as_deref().unwrap_or(&check.name);
                    format_observed(&read_path(env, path), check.value_kind)
                }
            },
        }
    }

    pub fn numeric(&self, check: &CheckSpec) -> Option<f64> {
        if !check.value_kind.is_numeric() {
            return None;
        }
        let env = self.envelope.as_ref()?;
        let path = check.path.as_deref().unwrap_or(&check.name);
        read_path(env, path).as_number()
    }

    pub fn is_pass(&self) -> bool {
        if self.exit_class != ExitClass::Ok {
            return false;
        }
        match &self.envelope {
            None => false,
            Some(env) => fail_signals(env).is_empty(),
        }
    }
}

fn format_observed(obs: &Observed, kind: ValueKind) -> String {
    match (kind, obs) {
        (_, Observed::Missing) => "missing".into(),
        (_, Observed::Null) => "null".into(),
        (ValueKind::Bool, Observed::Bool(b)) => {
            if *b {
                "true".into()
            } else {
                "false".into()
            }
        }
        (ValueKind::U64, Observed::U64(v)) => v.to_string(),
        (ValueKind::U64, Observed::I64(v)) if *v >= 0 => (*v as u64).to_string(),
        (ValueKind::Text, Observed::Text(s)) => s.clone(),
        _ => obs.as_display(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn observe_exit_and_bool() {
        let t = Trial {
            seed: 0,
            exit_class: ExitClass::Judgment,
            observed: "3".into(),
            envelope: Some(json!({"verify": {"identical": false}})),
        };
        assert_eq!(t.observe(&CheckSpec::exit_class()), "3");
        assert_eq!(
            t.observe(&CheckSpec::envelope_bool("verify.identical")),
            "false"
        );
        assert!(!t.is_pass());
    }
}
