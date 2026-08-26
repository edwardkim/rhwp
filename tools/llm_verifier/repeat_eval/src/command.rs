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
    Convert,
    #[serde(rename = "replace-text")]
    ReplaceText,
    Redact,
    #[serde(rename = "set-cell")]
    SetCell,
    #[serde(rename = "csv-to-table")]
    CsvToTable,
    Sanitize,
}

impl CommandFamily {
    pub const ALL: [CommandFamily; 13] = [
        CommandFamily::Info,
        CommandFamily::Verify,
        CommandFamily::IrDiff,
        CommandFamily::LayoutAnomaly,
        CommandFamily::Replay,
        CommandFamily::FillFields,
        CommandFamily::RenderDiff,
        CommandFamily::Convert,
        CommandFamily::ReplaceText,
        CommandFamily::Redact,
        CommandFamily::SetCell,
        CommandFamily::CsvToTable,
        CommandFamily::Sanitize,
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
            Self::Convert => "convert",
            Self::ReplaceText => "replace-text",
            Self::Redact => "redact",
            Self::SetCell => "set-cell",
            Self::CsvToTable => "csv-to-table",
            Self::Sanitize => "sanitize",
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
            "convert" => Some(Self::Convert),
            "replace-text" | "replace_text" | "edit replace-text" => Some(Self::ReplaceText),
            "redact" | "edit redact" => Some(Self::Redact),
            "set-cell" | "set_cell" | "edit set-cell" => Some(Self::SetCell),
            "csv-to-table" | "csv_to_table" => Some(Self::CsvToTable),
            "sanitize" | "edit sanitize" => Some(Self::Sanitize),
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
            Self::Convert => &["convert"],
            Self::ReplaceText => &["edit", "replace-text"],
            Self::Redact => &["edit", "redact"],
            Self::SetCell => &["edit", "set-cell"],
            Self::CsvToTable => &["csv-to-table"],
            Self::Sanitize => &["edit", "sanitize"],
        }
    }

    /// 이 명령이 반복 평가할 수 있는 기존 봉투 필드.
    pub fn repeatable_fields(self) -> &'static [&'static str] {
        match self {
            Self::Info => &["untrustedContent", "pageCount", "paraCount"],
            Self::Verify => &["verdict", "passCount", "failCount"],
            Self::IrDiff => &["identical", "diffCount"],
            Self::LayoutAnomaly => &[
                "hasSignal",
                "overflowCount",
                "overlapCount",
                "emptyPageCount",
                "strict",
            ],
            Self::Replay => &["reproduced"],
            Self::FillFields => &[
                "filledCount",
                "verify.identical",
                "verify.diffCount",
                "untrustedContent",
            ],
            Self::RenderDiff => &["regression", "status", "pageCountMismatch", "overPages"],
            Self::Convert => &["pageCountMismatch", "verify.identical"],
            Self::ReplaceText => &["replacedCount", "verify.identical", "verify.diffCount"],
            Self::Redact => &["redactedCount", "verify.identical"],
            Self::SetCell => &["changedCount", "verify.identical"],
            Self::CsvToTable => &["changedCount", "verify.identical", "invalid"],
            Self::Sanitize => &["removedCount", "wasDistribution", "verify.identical"],
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

    #[test]
    fn no_invented_command() {
        assert!(CommandFamily::parse("best-of-n").is_none());
        assert!(CommandFamily::parse("repeat-eval").is_none());
        assert!(CommandFamily::parse("decompose").is_none());
    }
}
