//! 자연어 주장. 문장 자체는 데이터가 아니라 결속 대상이다.

use crate::coords::{invented_keys_in, DocumentCoords};
use crate::envelope::EnvelopeKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalClaim {
    pub id: String,
    pub claim_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<DocumentCoords>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope_kind: Option<EnvelopeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// 주장 JSON 원본에 남은 발명 키 탐지용. 평가기가 채운다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invented_keys: Vec<String>,
}

impl NaturalClaim {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let id = v
            .get("id")
            .or_else(|| v.get("rowId"))
            .or_else(|| v.get("claimId"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let claim_text = v
            .get("claimText")
            .or_else(|| v.get("claim_text"))
            .or_else(|| v.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            return Err("claim id/rowId is required".into());
        }
        let locator = if v.get("section").is_some()
            || v.get("paragraph").is_some()
            || v.get("page").is_some()
            || v.get("charOffset").is_some()
            || v.get("locator").is_some()
        {
            Some(
                v.get("locator")
                    .map(DocumentCoords::from_envelope_value)
                    .unwrap_or_else(|| DocumentCoords::from_envelope_value(v)),
            )
        } else {
            None
        };
        let envelope_kind = v
            .get("envelopeKind")
            .or_else(|| v.get("sourceCommand"))
            .and_then(Value::as_str)
            .and_then(EnvelopeKind::parse);
        Ok(Self {
            id,
            claim_text,
            locator,
            envelope_kind,
            quote: v.get("quote").and_then(Value::as_str).map(str::to_string),
            file: v.get("file").and_then(Value::as_str).map(str::to_string),
            invented_keys: invented_keys_in(v),
        })
    }

    pub fn is_blank(&self) -> bool {
        self.claim_text.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn claim_from_row_keeps_four_coords() {
        let c = NaturalClaim::from_value(&json!({
            "rowId": "CB-1",
            "claimText": "과업지시서 제3조는 표준 API 연계를 필수기능으로 정한다.",
            "section": 0,
            "paragraph": 12,
            "page": 2,
            "charOffset": 48,
            "envelopeKind": "search"
        }))
        .expect("claim");
        assert_eq!(c.id, "CB-1");
        assert!(c.locator.as_ref().unwrap().coords_present());
        assert_eq!(c.envelope_kind, Some(EnvelopeKind::Search));
    }

    #[test]
    fn invented_pdf_page_is_captured() {
        let c = NaturalClaim::from_value(&json!({
            "rowId": "CB-2",
            "claimText": "쪽 번호를 사람이 읽기 쉽게 고쳤다.",
            "section": 0,
            "paragraph": 1,
            "page": 0,
            "charOffset": 0,
            "pdfPage": 1
        }))
        .expect("claim");
        assert!(c.invented_keys.contains(&"pdfPage".to_string()));
    }
}
