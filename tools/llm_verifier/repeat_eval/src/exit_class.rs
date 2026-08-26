//! rhwp 종료 코드 0/1/2/3/4. 새 코드는 만들지 않는다.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// 기존 rhwp 종료 코드. `capabilities.exitCodes` 와 동일하다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ExitClass {
    Ok = 0,
    Io = 1,
    Usage = 2,
    Judgment = 3,
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

    /// 동률이면 더 나쁜 종료코드를 고른다 (fail-closed).
    pub fn worse(self, other: Self) -> Self {
        if other > self {
            other
        } else {
            self
        }
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
    }

    #[test]
    fn worse_picks_higher_code() {
        assert_eq!(
            ExitClass::Ok.worse(ExitClass::Judgment),
            ExitClass::Judgment
        );
        assert_eq!(ExitClass::Io.worse(ExitClass::Usage), ExitClass::Usage);
    }
}
