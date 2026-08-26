//! 기존 `--json` 봉투에서 과정 검증이 읽는 필드만 추출한다.

use crate::check::CheckKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 지식지도 §2-2 판정 필드 부분집합. 산문 요약은 없다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CheckFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_signal: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlap_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_page_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_page_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_count_mismatch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_identical: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_diff_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filled_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_found_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identical: Option<bool>,
}

impl CheckFields {
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
        format!(
            "vd={} fail={} pass={} hs={} st={} ovf={} ovl={} emp={} pc={} epc={} pcm={} vi={} vdc={} fc={} nf={} id={}",
            self.verdict.as_deref().unwrap_or("-"),
            n(self.fail_count),
            n(self.pass_count),
            b(self.has_signal),
            b(self.strict),
            n(self.overflow_count),
            n(self.overlap_count),
            n(self.empty_page_count),
            n(self.page_count),
            n(self.expected_page_count),
            b(self.page_count_mismatch),
            b(self.verify_identical),
            n(self.verify_diff_count),
            n(self.filled_count),
            n(self.not_found_count),
            b(self.identical),
        )
    }

    pub fn fail_signals(&self, check: CheckKind) -> Vec<&'static str> {
        let mut out = Vec::new();
        match check {
            CheckKind::Verify => {
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
            }
            CheckKind::LayoutAnomaly => {
                if self.strict == Some(true)
                    && (self.overflow_count.unwrap_or(0) > 0 || self.overlap_count.unwrap_or(0) > 0)
                {
                    out.push("layout-strict-signal");
                }
                if self.has_signal == Some(true)
                    && self.strict == Some(true)
                    && (self.overflow_count.unwrap_or(0) > 0 || self.overlap_count.unwrap_or(0) > 0)
                {
                    out.push("hasSignal=true");
                }
            }
            CheckKind::PageCount => {
                if self.page_count_mismatch == Some(true) {
                    out.push("pageCountMismatch=true");
                }
                if let (Some(a), Some(e)) = (self.page_count, self.expected_page_count) {
                    if a != e {
                        out.push("pageCount!=expected");
                    }
                }
            }
            CheckKind::FillVerify => {
                if self.verify_identical == Some(false) {
                    out.push("verify.identical=false");
                }
                if self.verify_diff_count.unwrap_or(0) > 0 && self.verify_identical != Some(true) {
                    out.push("verify.diffCount>0");
                }
            }
        }
        out
    }
}

pub fn extract_check_fields(check: CheckKind, envelope: &Value) -> CheckFields {
    let mut f = CheckFields::default();
    f.verdict = str_field(envelope, "verdict");
    f.fail_count = u64_field(envelope, "failCount");
    f.pass_count = u64_field(envelope, "passCount");
    f.has_signal = bool_field(envelope, "hasSignal");
    f.strict = bool_field(envelope, "strict");
    f.overflow_count = u64_field(envelope, "overflowCount");
    f.overlap_count = u64_field(envelope, "overlapCount");
    f.empty_page_count = u64_field(envelope, "emptyPageCount");
    f.page_count = u64_field(envelope, "pageCount");
    f.expected_page_count = u64_field(envelope, "expectedPageCount")
        .or_else(|| nested_u64(envelope, "verifyPages", "expected"));
    f.page_count_mismatch = bool_field(envelope, "pageCountMismatch")
        .or_else(|| nested_bool(envelope, "verifyPages", "match").map(|m| !m));
    f.identical = bool_field(envelope, "identical");
    f.filled_count = u64_field(envelope, "filledCount");
    if let Some(arr) = envelope.get("notFound").and_then(Value::as_array) {
        f.not_found_count = Some(arr.len() as u64);
    }
    if let Some(v) = envelope.get("verify") {
        f.verify_identical = bool_field(v, "identical");
        f.verify_diff_count = u64_field(v, "diffCount");
    }
    if check == CheckKind::PageCount {
        if f.expected_page_count.is_none() {
            if let Some(exps) = envelope.get("expectations").and_then(Value::as_array) {
                for exp in exps {
                    if exp.get("kind").and_then(Value::as_str) == Some("pages") {
                        f.expected_page_count = u64_field(exp, "expected");
                        if f.page_count.is_none() {
                            f.page_count = u64_field(exp, "actual");
                        }
                        if f.page_count_mismatch.is_none() {
                            f.page_count_mismatch = bool_field(exp, "pass").map(|p| !p);
                        }
                    }
                }
            }
        }
    }
    f
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn bool_field(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_bool)
}

fn u64_field(v: &Value, key: &str) -> Option<u64> {
    v.get(key).and_then(Value::as_u64)
}

fn nested_u64(v: &Value, a: &str, b: &str) -> Option<u64> {
    v.get(a).and_then(|n| u64_field(n, b))
}

fn nested_bool(v: &Value, a: &str, b: &str) -> Option<bool> {
    v.get(a).and_then(|n| bool_field(n, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_fill_verify_block() {
        let env = json!({
            "filledCount": 3,
            "notFound": [],
            "verify": { "identical": false, "diffCount": 2 }
        });
        let f = extract_check_fields(CheckKind::FillVerify, &env);
        assert_eq!(f.verify_identical, Some(false));
        assert_eq!(f.verify_diff_count, Some(2));
        assert_eq!(f.filled_count, Some(3));
        assert!(f
            .fail_signals(CheckKind::FillVerify)
            .contains(&"verify.identical=false"));
    }

    #[test]
    fn extracts_page_count_from_info() {
        let env = json!({
            "pageCount": 5,
            "expectedPageCount": 4,
            "pageCountMismatch": true
        });
        let f = extract_check_fields(CheckKind::PageCount, &env);
        assert_eq!(f.page_count, Some(5));
        assert_eq!(f.expected_page_count, Some(4));
        assert_eq!(f.page_count_mismatch, Some(true));
    }

    #[test]
    fn fingerprints_differ_on_verdict() {
        let mut a = CheckFields::default();
        a.verdict = Some("pass".into());
        a.fail_count = Some(0);
        let mut b = CheckFields::default();
        b.verdict = Some("fail".into());
        b.fail_count = Some(1);
        assert_ne!(a.fingerprint(), b.fingerprint());
    }
}
