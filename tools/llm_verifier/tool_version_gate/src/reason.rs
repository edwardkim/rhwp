//! 게이트 이유. `reproduced:true` 를 옛 도구로 인정하지 않는 축.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reason {
    /// 버전 일치 + `reproduced=true`. 유일하게 accepted.
    FreshReproduced,
    /// 버전 일치 + `reproduced=false`.
    FreshNotReproduced,
    /// 버전 일치 + 재현 주장 없음(attest).
    FreshAbsent,
    /// 버전 불일치 + `reproduced=true`. 낡은 도구. 합격 금지.
    StaleTool,
    /// 버전 불일치 + `reproduced=false`.
    StaleAndNotReproduced,
    /// 버전 불일치 + 재현 주장 없음.
    StaleAndAbsent,
    /// 영수증 `toolVersion` 공란.
    AttestVersionMissing,
    /// 검증기 바이너리 버전 공란.
    VerifyVersionMissing,
}

impl Reason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreshReproduced => "FRESH_REPRODUCED",
            Self::FreshNotReproduced => "FRESH_NOT_REPRODUCED",
            Self::FreshAbsent => "FRESH_ABSENT",
            Self::StaleTool => "STALE_TOOL",
            Self::StaleAndNotReproduced => "STALE_AND_NOT_REPRODUCED",
            Self::StaleAndAbsent => "STALE_AND_ABSENT",
            Self::AttestVersionMissing => "ATTEST_VERSION_MISSING",
            Self::VerifyVersionMissing => "VERIFY_VERSION_MISSING",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "FRESH_REPRODUCED" => Some(Self::FreshReproduced),
            "FRESH_NOT_REPRODUCED" => Some(Self::FreshNotReproduced),
            "FRESH_ABSENT" => Some(Self::FreshAbsent),
            "STALE_TOOL" => Some(Self::StaleTool),
            "STALE_AND_NOT_REPRODUCED" => Some(Self::StaleAndNotReproduced),
            "STALE_AND_ABSENT" => Some(Self::StaleAndAbsent),
            "ATTEST_VERSION_MISSING" => Some(Self::AttestVersionMissing),
            "VERIFY_VERSION_MISSING" => Some(Self::VerifyVersionMissing),
            _ => None,
        }
    }

    /// `STALE_TOOL` 은 절대 true 가 아니다.
    pub fn accepts(self) -> bool {
        matches!(self, Self::FreshReproduced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_fresh_reproduced_accepts() {
        for reason in [
            Reason::FreshNotReproduced,
            Reason::FreshAbsent,
            Reason::StaleTool,
            Reason::StaleAndNotReproduced,
            Reason::StaleAndAbsent,
            Reason::AttestVersionMissing,
            Reason::VerifyVersionMissing,
        ] {
            assert!(!reason.accepts(), "{}", reason.as_str());
        }
        assert!(Reason::FreshReproduced.accepts());
    }

    #[test]
    fn parse_roundtrip() {
        for reason in [
            Reason::FreshReproduced,
            Reason::FreshNotReproduced,
            Reason::FreshAbsent,
            Reason::StaleTool,
            Reason::StaleAndNotReproduced,
            Reason::StaleAndAbsent,
            Reason::AttestVersionMissing,
            Reason::VerifyVersionMissing,
        ] {
            assert_eq!(Reason::parse(reason.as_str()), Some(reason));
        }
    }
}
