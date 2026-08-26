//! 기존 rhwp 편집 명령. 새 명령을 발명하지 않는다.

use serde::{Deserialize, Serialize};
use std::fmt;

/// 과정 추적의 한 스텝이 가리키는 기존 편집 명령.
/// `rhwp capabilities` / 지식지도에 있는 이름만 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepKind {
    #[serde(rename = "fill-fields")]
    FillFields,
    #[serde(rename = "replace-text")]
    ReplaceText,
    #[serde(rename = "delete-text")]
    DeleteText,
    #[serde(rename = "insert-text")]
    InsertText,
    Redact,
    Sanitize,
    #[serde(rename = "csv-to-table")]
    CsvToTable,
    #[serde(rename = "insert-table")]
    InsertTable,
    #[serde(rename = "delete-row")]
    DeleteRow,
    #[serde(rename = "apply-char-format")]
    ApplyCharFormat,
}

impl StepKind {
    pub const ALL: [StepKind; 10] = [
        StepKind::FillFields,
        StepKind::ReplaceText,
        StepKind::DeleteText,
        StepKind::InsertText,
        StepKind::Redact,
        StepKind::Sanitize,
        StepKind::CsvToTable,
        StepKind::InsertTable,
        StepKind::DeleteRow,
        StepKind::ApplyCharFormat,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FillFields => "fill-fields",
            Self::ReplaceText => "replace-text",
            Self::DeleteText => "delete-text",
            Self::InsertText => "insert-text",
            Self::Redact => "redact",
            Self::Sanitize => "sanitize",
            Self::CsvToTable => "csv-to-table",
            Self::InsertTable => "insert-table",
            Self::DeleteRow => "delete-row",
            Self::ApplyCharFormat => "apply-char-format",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "fill-fields" | "fill_fields" | "edit fill-fields" => Some(Self::FillFields),
            "replace-text" | "replace_text" => Some(Self::ReplaceText),
            "delete-text" | "delete_text" => Some(Self::DeleteText),
            "insert-text" | "insert_text" => Some(Self::InsertText),
            "redact" => Some(Self::Redact),
            "sanitize" => Some(Self::Sanitize),
            "csv-to-table" | "csv_to_table" => Some(Self::CsvToTable),
            "insert-table" | "insert_table" => Some(Self::InsertTable),
            "delete-row" | "delete_row" => Some(Self::DeleteRow),
            "apply-char-format" | "apply_char_format" => Some(Self::ApplyCharFormat),
            _ => None,
        }
    }

    /// 기존 `rhwp edit <cmd>` argv 머리. 새 플래그를 만들지 않는다.
    pub fn rhwp_argv_head(self) -> &'static [&'static str] {
        match self {
            Self::FillFields => &["edit", "fill-fields"],
            Self::ReplaceText => &["edit", "replace-text"],
            Self::DeleteText => &["edit", "delete-text"],
            Self::InsertText => &["edit", "insert-text"],
            Self::Redact => &["edit", "redact"],
            Self::Sanitize => &["edit", "sanitize"],
            Self::CsvToTable => &["edit", "csv-to-table"],
            Self::InsertTable => &["edit", "insert-table"],
            Self::DeleteRow => &["edit", "delete-row"],
            Self::ApplyCharFormat => &["edit", "apply-char-format"],
        }
    }
}

impl fmt::Display for StepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uses_existing_cli_names() {
        for kind in StepKind::ALL {
            let v = serde_json::to_value(kind).unwrap();
            assert_eq!(v, serde_json::Value::String(kind.as_str().to_string()));
            let back: StepKind = serde_json::from_value(v).unwrap();
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn unknown_name_is_none() {
        assert!(StepKind::parse("best-of-n").is_none());
        assert!(StepKind::parse("rank-candidates").is_none());
    }
}
