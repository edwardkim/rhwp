//! 같은 산출. 후보 집합(V-bon)이 아니다.

use crate::command::CommandFamily;
use crate::exit_class::ExitClass;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 고정된 산출. K번 다시 읽히는 대상.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub artifact_id: String,
    pub command: CommandFamily,
    pub sample: String,
    #[serde(default)]
    pub argv: Vec<String>,
    pub intended_exit: ExitClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended: Option<Value>,
}

impl Artifact {
    pub fn fingerprint(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.command.as_str(),
            self.sample,
            self.intended_exit.code(),
            intended_fp(self.intended.as_ref())
        )
    }
}

fn intended_fp(value: Option<&Value>) -> String {
    match value {
        None => "-".into(),
        Some(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fingerprint_changes_with_intended() {
        let a = Artifact {
            artifact_id: "a".into(),
            command: CommandFamily::FillFields,
            sample: "samples/x.hwp".into(),
            argv: vec![],
            intended_exit: ExitClass::Ok,
            intended: Some(json!({"filledCount": 1})),
        };
        let mut b = a.clone();
        b.intended = Some(json!({"filledCount": 2}));
        assert_ne!(a.fingerprint(), b.fingerprint());
    }
}
