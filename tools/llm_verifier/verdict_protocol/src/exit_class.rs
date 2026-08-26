//! rhwp 종료 코드 0/1/2/3/4. 새 코드는 만들지 않는다.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// 기존 rhwp 종료 코드. `capabilities.exitCodes` 와 동일하다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ExitClass {
    /// 성공. `--json` 이면 stdout 에 봉투가 있다.
    Ok = 0,
    /// IO. 파일 없음·잘림·암호 틀림. 실패 경로 stdout 은 0바이트일 수 있다.
    Io = 1,
    /// 사용법. 플래그·위치인자·스키마 위반. 문서를 읽기 전에 끊는다.
    Usage = 2,
    /// 판정 실패. 도구는 정상 동작했고 단언이 실패한 것. 봉투가 판정 본문이다.
    Judgment = 3,
    /// 쪽수 검증 실패(`--verify-pages`). convert/export-hwpx 계열.
    PageVerify = 4,
}

impl ExitClass {
    pub const ALL: [ExitClass; 5] = [
        ExitClass::Ok,
        ExitClass::Io,
        ExitClass::Usage,
        ExitClass::Judgment,
        ExitClass::PageVerify,
    ];

    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Ok),
            1 => Some(Self::Io),
            2 => Some(Self::Usage),
            3 => Some(Self::Judgment),
            4 => Some(Self::PageVerify),
            _ => None,
        }
    }

    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Io => "io",
            Self::Usage => "usage",
            Self::Judgment => "judgment",
            Self::PageVerify => "page_verify",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ok" | "success" => Some(Self::Ok),
            "io" => Some(Self::Io),
            "usage" => Some(Self::Usage),
            "judgment" | "judgement" => Some(Self::Judgment),
            "page_verify" | "pageVerify" | "verify_pages" => Some(Self::PageVerify),
            _ => None,
        }
    }

    /// exit 1/2 는 봉투 없이 끝나는 기존 규약이 있다.
    pub fn envelope_optional(self) -> bool {
        matches!(self, Self::Io | Self::Usage)
    }

    /// exit 3/4 는 판정이 데이터이므로 `--json` 이면 봉투가 있어야 한다.
    pub fn requires_envelope_when_json(self) -> bool {
        matches!(self, Self::Ok | Self::Judgment | Self::PageVerify)
    }
}

impl fmt::Display for ExitClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name(), self.code())
    }
}

impl Serialize for ExitClass {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.code())
    }
}

impl<'de> Deserialize<'de> for ExitClass {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = i64::deserialize(deserializer)?;
        if raw < 0 || raw > i32::MAX as i64 {
            return Err(serde::de::Error::custom(format!(
                "exitClass out of i32 range: {raw}"
            )));
        }
        ExitClass::from_code(raw as i32).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unknown rhwp exitClass {raw}; protocol accepts only 0/1/2/3/4"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_contiguous_0_to_4() {
        for (i, cls) in ExitClass::ALL.iter().enumerate() {
            assert_eq!(cls.code() as usize, i);
            assert_eq!(ExitClass::from_code(i as i32), Some(*cls));
        }
        assert_eq!(ExitClass::from_code(5), None);
        assert_eq!(ExitClass::from_code(-1), None);
    }

    #[test]
    fn json_is_a_number_not_a_string() {
        let v = serde_json::to_value(ExitClass::Judgment).unwrap();
        assert_eq!(v, serde_json::json!(3));
        let back: ExitClass = serde_json::from_value(v).unwrap();
        assert_eq!(back, ExitClass::Judgment);
    }
}
