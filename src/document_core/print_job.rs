//! 파일명과 현지 인쇄 시각은 호출자가 공급하는 작업 정보다.

use crate::error::HwpError;
use chrono::{DateTime, NaiveDateTime, Timelike};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub struct PrintJobContext {
    file_name: String,
    printed_at: NaiveDateTime,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PrintJobInput {
    file_name: String,
    printed_at: String,
}

impl PrintJobContext {
    pub fn from_json(json: &str) -> Result<Self, HwpError> {
        let input: PrintJobInput = serde_json::from_str(json)
            .map_err(|e| HwpError::InvalidField(format!("인쇄 작업 정보: {e}")))?;
        if input.file_name.trim().is_empty()
            || input.file_name.chars().count() > 255
            || input.file_name.contains(['/', '\\'])
            || input.file_name.chars().any(char::is_control)
        {
            return Err(HwpError::InvalidField(
                "인쇄 작업에는 경로를 제외한 파일명(1~255자)이 필요합니다.".into(),
            ));
        }
        let printed_at = DateTime::parse_from_rfc3339(&input.printed_at).map_err(|_| {
            HwpError::InvalidField(
                "인쇄 시각은 현지 오프셋을 포함한 RFC 3339 형식이어야 합니다.".into(),
            )
        })?;
        Ok(Self {
            file_name: input.file_name,
            printed_at: printed_at.naive_local(),
        })
    }

    pub(crate) fn file_name(&self) -> &str {
        &self.file_name
    }

    pub(crate) fn local_date_time(&self) -> String {
        let (pm, hour) = self.printed_at.hour12();
        let period = if pm { "오후" } else { "오전" };
        format!(
            "{} {period} {hour}:{:02}",
            self.printed_at.format("%Y-%m-%d"),
            self.printed_at.minute()
        )
    }
}
