//! 한 편집 스텝 뒤에 돌리는 기존 기계 검사.

use crate::envelope::CheckFields;
use crate::exit_class::ExitClass;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// 이슈 #5490 가 고정한 네 검사. 새 검사 명령을 만들지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckKind {
    Verify,
    #[serde(rename = "layout-anomaly")]
    LayoutAnomaly,
    #[serde(rename = "pageCount")]
    PageCount,
    #[serde(rename = "fill-verify")]
    FillVerify,
}

impl CheckKind {
    pub const ALL: [CheckKind; 4] = [
        CheckKind::Verify,
        CheckKind::LayoutAnomaly,
        CheckKind::PageCount,
        CheckKind::FillVerify,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verify => "verify",
            Self::LayoutAnomaly => "layout-anomaly",
            Self::PageCount => "pageCount",
            Self::FillVerify => "fill-verify",
        }
    }

    /// 기존 rhwp 호출 머리. pageCount 는 `info` 봉투의 필드를 읽는다.
    pub fn rhwp_argv_head(self) -> &'static [&'static str] {
        match self {
            Self::Verify => &["verify"],
            Self::LayoutAnomaly => &["layout-anomaly"],
            Self::PageCount => &["info"],
            Self::FillVerify => &["edit", "fill-fields"],
        }
    }
}

impl fmt::Display for CheckKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 한 검사의 관측. 종료코드 + 기존 봉투 + 추출 필드.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckObservation {
    pub check: CheckKind,
    #[serde(default)]
    pub argv: Vec<String>,
    pub exit_class: ExitClass,
    #[serde(default)]
    pub pass: bool,
    #[serde(default)]
    pub fail_signals: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Value>,
    #[serde(default)]
    pub fields: CheckFields,
}

impl CheckObservation {
    pub fn has_envelope(&self) -> bool {
        match &self.envelope {
            Some(Value::Null) | None => false,
            Some(Value::Object(o)) => !o.is_empty(),
            Some(_) => true,
        }
    }

    pub fn refresh_fields(&mut self) {
        if let Some(env) = &self.envelope {
            self.fields = crate::envelope::extract_check_fields(self.check, env);
        }
    }

    pub fn fingerprint(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.check.as_str(),
            self.exit_class.code(),
            if self.pass { "p" } else { "f" },
            self.fields.fingerprint()
        )
    }
}

/// 검사 한 건의 기계 판정. 순위 점수가 아니다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckVerdict {
    pub check: CheckKind,
    pub exit_class: ExitClass,
    pub pass: bool,
    pub consistent: bool,
    pub fail_signals: Vec<String>,
    pub rule_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_names_match_issue_axes() {
        assert_eq!(
            serde_json::to_value(CheckKind::PageCount).unwrap(),
            serde_json::json!("pageCount")
        );
        assert_eq!(
            serde_json::to_value(CheckKind::FillVerify).unwrap(),
            serde_json::json!("fill-verify")
        );
        assert_eq!(
            serde_json::to_value(CheckKind::LayoutAnomaly).unwrap(),
            serde_json::json!("layout-anomaly")
        );
    }
}
