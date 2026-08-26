//! 코퍼스 한 행. `(claim_text, coords_present, field_set, pass/fail)`.

use crate::coords::DocumentCoords;
use crate::envelope::EnvelopeKind;
use crate::verdict::{FailKind, Verdict};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimBindRow {
    pub row_id: String,
    pub claim_text: String,
    pub coords_present: bool,
    pub field_set: Vec<String>,
    pub envelope_kind: String,
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_kind: Option<FailKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub char_offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textbox: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invented_keys: Vec<String>,
}

impl ClaimBindRow {
    pub fn locator(&self) -> Option<DocumentCoords> {
        let c = DocumentCoords {
            section: self.section,
            paragraph: self.paragraph,
            page: self.page,
            char_offset: self.char_offset,
            length: self.length,
            cell: self.cell.clone(),
            textbox: self.textbox.clone(),
        };
        if c.is_empty() {
            None
        } else {
            Some(c)
        }
    }

    pub fn parsed_envelope_kind(&self) -> Option<EnvelopeKind> {
        EnvelopeKind::parse(&self.envelope_kind)
    }

    /// 행이 스스로 적은 fieldSet/coordsPresent 가 실제 키와 같은지.
    pub fn field_set_consistent(&self) -> bool {
        let actual = self.locator().map(|c| c.field_set()).unwrap_or_default();
        actual == self.field_set
            && self.coords_present == self.locator().is_some_and(|c| c.coords_present())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_pass_row() {
        let raw = r#"{
            "rowId":"CB-000001",
            "claimText":"과업지시서 제3조는 표준 API 연계를 필수기능으로 정한다.",
            "coordsPresent":true,
            "fieldSet":["charOffset","length","page","paragraph","section"],
            "envelopeKind":"search",
            "verdict":"pass",
            "section":0,
            "paragraph":12,
            "page":2,
            "charOffset":48,
            "length":18
        }"#;
        let row: ClaimBindRow = serde_json::from_str(raw).expect("row");
        assert!(row.coords_present);
        assert!(row.field_set_consistent());
        assert_eq!(row.verdict, Verdict::Pass);
    }
}
