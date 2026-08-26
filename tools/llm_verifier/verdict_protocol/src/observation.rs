//! 검증기 입력 관측. 종료코드 + 기존 봉투 + 출처 표지.

use crate::command::CommandFamily;
use crate::exit_class::ExitClass;
use crate::extract::extract_judgment;
use crate::judgment::JudgmentFields;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 한 번의 기존 rhwp 호출을 검증기가 읽는 관측 단위.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub record_id: String,
    pub source_tag: String,
    pub command: CommandFamily,
    #[serde(default)]
    pub argv: Vec<String>,
    pub exit_class: ExitClass,
    #[serde(default)]
    pub stdout_present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Value>,
    #[serde(default)]
    pub judgment: JudgmentFields,
}

impl Observation {
    pub fn uniqueness_key(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.command.as_str(),
            self.exit_class.code(),
            self.judgment.fingerprint(),
            self.source_tag
        )
    }

    /// 봉투가 있으면 판정 필드를 다시 추출해 채운다.
    pub fn refresh_judgment(&mut self) {
        if let Some(env) = &self.envelope {
            self.judgment = extract_judgment(env);
        }
    }

    pub fn has_envelope(&self) -> bool {
        match &self.envelope {
            Some(Value::Null) | None => false,
            Some(Value::Object(o)) => !o.is_empty(),
            Some(_) => true,
        }
    }
}

/// (command, exitClass, judgment fingerprint, sourceTag) 유일 제약.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniquenessKey {
    pub command: String,
    pub exit_class: u8,
    pub judgment: String,
    pub source_tag: String,
}

impl UniquenessKey {
    pub fn from_observation(obs: &Observation) -> Self {
        Self {
            command: obs.command.as_str().to_string(),
            exit_class: obs.exit_class.code(),
            judgment: obs.judgment.fingerprint(),
            source_tag: obs.source_tag.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn uniqueness_includes_source_tag() {
        let mut a = Observation {
            record_id: "a".into(),
            source_tag: "gov/a#info".to_string(),
            command: CommandFamily::Info,
            argv: vec!["info".into(), "a.hwp".into(), "--json".into()],
            exit_class: ExitClass::Ok,
            stdout_present: true,
            stderr_kind: None,
            envelope: Some(json!({"schemaVersion":"1.0","source":"a.hwp"})),
            judgment: JudgmentFields::default(),
        };
        let mut b = a.clone();
        b.source_tag = "gov/b#info".into();
        a.refresh_judgment();
        b.refresh_judgment();
        assert_ne!(a.uniqueness_key(), b.uniqueness_key());
    }
}
