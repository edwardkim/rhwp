//! 기존 rhwp 명령 가족. 새 명령을 발명하지 않는다.

use serde::{Deserialize, Serialize};
use std::fmt;

/// 검증기가 읽는 기존 CLI 명령. `rhwp capabilities` 이름과 같다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandFamily {
    Info,
    Verify,
    #[serde(rename = "ir-diff")]
    IrDiff,
    #[serde(rename = "layout-anomaly")]
    LayoutAnomaly,
    Replay,
    #[serde(rename = "fill-fields")]
    FillFields,
    #[serde(rename = "render-diff")]
    RenderDiff,
}

impl CommandFamily {
    pub const ALL: [CommandFamily; 7] = [
        CommandFamily::Info,
        CommandFamily::Verify,
        CommandFamily::IrDiff,
        CommandFamily::LayoutAnomaly,
        CommandFamily::Replay,
        CommandFamily::FillFields,
        CommandFamily::RenderDiff,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Verify => "verify",
            Self::IrDiff => "ir-diff",
            Self::LayoutAnomaly => "layout-anomaly",
            Self::Replay => "replay",
            Self::FillFields => "fill-fields",
            Self::RenderDiff => "render-diff",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "info" => Some(Self::Info),
            "verify" => Some(Self::Verify),
            "ir-diff" | "ir_diff" => Some(Self::IrDiff),
            "layout-anomaly" | "layout_anomaly" => Some(Self::LayoutAnomaly),
            "replay" => Some(Self::Replay),
            "fill-fields" | "fill_fields" | "edit fill-fields" => Some(Self::FillFields),
            "render-diff" | "render_diff" => Some(Self::RenderDiff),
            _ => None,
        }
    }

    /// argv 의 기존 rhwp 호출 형태. 새 플래그를 만들지 않는다.
    pub fn rhwp_argv_head(self) -> &'static [&'static str] {
        match self {
            Self::Info => &["info"],
            Self::Verify => &["verify"],
            Self::IrDiff => &["ir-diff"],
            Self::LayoutAnomaly => &["layout-anomaly"],
            Self::Replay => &["replay"],
            Self::FillFields => &["edit", "fill-fields"],
            Self::RenderDiff => &["render-diff"],
        }
    }

    /// 지식지도 §2-2 에서 이 명령이 실을 수 있는 판정 필드.
    pub fn judgment_fields(self) -> &'static [&'static str] {
        match self {
            Self::Info => &[],
            Self::Verify => &["verdict", "passCount", "failCount", "expectations"],
            Self::IrDiff => &["identical", "diffCount", "categories"],
            Self::LayoutAnomaly => &[
                "hasSignal",
                "strict",
                "overflowCount",
                "overlapCount",
                "emptyPageCount",
            ],
            Self::Replay => &["reproduced", "expectedOutputSha256", "mode"],
            Self::FillFields => &[
                "filledCount",
                "notFound",
                "verify.identical",
                "verify.diffCount",
            ],
            Self::RenderDiff => &[
                "regression",
                "status",
                "maxDisp",
                "overPages",
                "pageCountMismatch",
            ],
        }
    }
}

impl fmt::Display for CommandFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uses_existing_cli_names() {
        for cmd in CommandFamily::ALL {
            let v = serde_json::to_value(cmd).unwrap();
            assert_eq!(v, serde_json::Value::String(cmd.as_str().to_string()));
            let back: CommandFamily = serde_json::from_value(v).unwrap();
            assert_eq!(back, cmd);
        }
    }
}
