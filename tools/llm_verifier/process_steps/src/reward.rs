//! 과정 보상. 중간 스텝의 pass/fail 집계. Best-of-N 순위가 아니다.

use serde::{Deserialize, Serialize};

/// 한 편집 스텝의 과정 보상.
///
/// `pass` 는 그 스텝에서 돌린 기계 검사가 모두 통과했는가이다.
/// `rank`, `score`, `bestOfN` 같은 순위 필드는 두지 않는다 (V-bon / #5489).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProcessReward {
    pub pass: bool,
    pub check_count: u64,
    pub pass_count: u64,
    pub fail_count: u64,
    #[serde(default)]
    pub failed_checks: Vec<String>,
    pub worst_exit_class: u8,
    #[serde(default)]
    pub consistent: bool,
}

impl ProcessReward {
    pub fn fingerprint(&self) -> String {
        format!(
            "p={}|c={}|ok={}|fail={}|worst={}|cons={}|fc={}",
            if self.pass { "t" } else { "f" },
            self.check_count,
            self.pass_count,
            self.fail_count,
            self.worst_exit_class,
            if self.consistent { "t" } else { "f" },
            self.failed_checks.join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_has_no_rank_field() {
        let r = ProcessReward {
            pass: true,
            check_count: 4,
            pass_count: 4,
            fail_count: 0,
            failed_checks: vec![],
            worst_exit_class: 0,
            consistent: true,
        };
        let v = serde_json::to_value(&r).unwrap();
        let obj = v.as_object().unwrap();
        assert!(!obj.contains_key("rank"));
        assert!(!obj.contains_key("score"));
        assert!(!obj.contains_key("bestOfN"));
        assert!(!obj.contains_key("bestOfn"));
        assert!(obj.contains_key("pass"));
        assert!(obj.contains_key("failedChecks"));
    }
}
