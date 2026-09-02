//! 기존 봉투에서 꺼내는 판정 필드. 필드 이름은 지식지도 §2-2 가 단일 출처다.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `replay.reproduced` 는 bool|null (영수증) 또는 number (`audit` 재사용).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Reproduced {
    Flag(bool),
    Count(u64),
    Null,
}

impl Reproduced {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Flag(v) => Some(*v),
            Self::Count(n) => Some(*n > 0),
            Self::Null => None,
        }
    }

    pub fn fingerprint(&self) -> String {
        match self {
            Self::Flag(true) => "true".into(),
            Self::Flag(false) => "false".into(),
            Self::Count(n) => format!("n{n}"),
            Self::Null => "null".into(),
        }
    }
}

/// `--verify` 자기검증 조각 `{identical,diffCount}`. 옵션을 안 주면 봉투에서 null.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VerifyBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identical: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_count: Option<u64>,
}

impl VerifyBlock {
    pub fn is_empty(&self) -> bool {
        self.identical.is_none() && self.diff_count.is_none()
    }
}

/// 검증기가 읽는 판정 필드만. 산문 요약은 없다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JudgmentFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identical: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_signal: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reproduced: Option<Reproduced>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify: Option<VerifyBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regression: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlap_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_page_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_count_mismatch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub over_pages: Option<u64>,
}

impl JudgmentFields {
    /// 기계 실패 신호. 산문 해석 없이 필드 값만 본다.
    pub fn fail_signals(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.identical == Some(false) {
            out.push("identical=false");
        }
        if let Some(v) = &self.verify {
            if v.identical == Some(false) {
                out.push("verify.identical=false");
            }
            if v.diff_count.unwrap_or(0) > 0 && v.identical != Some(true) {
                out.push("verify.diffCount>0");
            }
        }
        if self.reproduced.as_ref().and_then(Reproduced::as_bool) == Some(false) {
            out.push("reproduced=false");
        }
        if self.fail_count.unwrap_or(0) > 0 {
            out.push("failCount>0");
        }
        if self
            .verdict
            .as_deref()
            .is_some_and(|v| matches!(v, "fail" | "FAIL" | "invalid" | "mismatch"))
        {
            out.push("verdict=fail");
        }
        if self.regression == Some(true) {
            out.push("regression=true");
        }
        if self.clean == Some(false) {
            out.push("clean=false");
        }
        if self.valid == Some(false) {
            out.push("valid=false");
        }
        if self.page_count_mismatch == Some(true) {
            out.push("pageCountMismatch=true");
        }
        out
    }

    /// layout-anomaly `--strict` 확정 신호. 빈 쪽만 있으면 실패가 아니다.
    pub fn layout_strict_fail(&self) -> bool {
        self.strict == Some(true)
            && (self.overflow_count.unwrap_or(0) > 0 || self.overlap_count.unwrap_or(0) > 0)
    }

    pub fn has_any_fail_signal(&self) -> bool {
        !self.fail_signals().is_empty() || self.layout_strict_fail()
    }

    /// 명령+종료코드+판정+출처 유일키에 넣는 정규화 지문.
    pub fn fingerprint(&self) -> String {
        fn b(v: Option<bool>) -> &'static str {
            match v {
                Some(true) => "t",
                Some(false) => "f",
                None => "-",
            }
        }
        fn n(v: Option<u64>) -> String {
            match v {
                Some(x) => x.to_string(),
                None => "-".into(),
            }
        }
        let verify = match &self.verify {
            Some(v) => format!("{}:{}", b(v.identical), n(v.diff_count)),
            None => "-:-".into(),
        };
        let reproduced = self
            .reproduced
            .as_ref()
            .map(Reproduced::fingerprint)
            .unwrap_or_else(|| "-".into());
        format!(
            "id={} hs={} rp={} fc={} vi={} fail={} pass={} vd={} rg={} st={} cl={} sc={} va={} dc={} stc={} ovf={} ovl={} emp={} pcm={} op={}",
            b(self.identical),
            b(self.has_signal),
            reproduced,
            n(self.finding_count),
            verify,
            n(self.fail_count),
            n(self.pass_count),
            self.verdict.as_deref().unwrap_or("-"),
            b(self.regression),
            self.status.as_deref().unwrap_or("-"),
            b(self.clean),
            n(self.signal_count),
            b(self.valid),
            n(self.diff_count),
            b(self.strict),
            n(self.overflow_count),
            n(self.overlap_count),
            n(self.empty_page_count),
            b(self.page_count_mismatch),
            n(self.over_pages),
        )
    }

    pub fn from_envelope(envelope: &Value) -> Self {
        crate::extract::extract_judgment(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_distinguishes_verify_block() {
        let a = JudgmentFields {
            verify: Some(VerifyBlock {
                identical: Some(true),
                diff_count: Some(0),
            }),
            ..Default::default()
        };
        let b = JudgmentFields {
            verify: Some(VerifyBlock {
                identical: Some(false),
                diff_count: Some(3),
            }),
            ..Default::default()
        };
        assert_ne!(a.fingerprint(), b.fingerprint());
    }
}
