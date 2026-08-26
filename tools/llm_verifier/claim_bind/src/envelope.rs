//! 기존 search / extract-data 봉투만 읽는다. 필드를 발명하지 않는다.

use crate::coords::DocumentCoords;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnvelopeKind {
    Search,
    ExtractData,
}

impl EnvelopeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::ExtractData => "extract-data",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "search" => Some(Self::Search),
            "extract-data" | "extract_data" | "extractData" => Some(Self::ExtractData),
            _ => None,
        }
    }
}

/// 봉투 매치/항목 한 건. 좌표는 있는 키만 옮긴다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeHit {
    pub kind: EnvelopeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalized: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub coords: DocumentCoords,
}

impl EnvelopeHit {
    pub fn from_search_match(v: &Value) -> Self {
        Self {
            kind: EnvelopeKind::Search,
            quote: v
                .get("text")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| v.get("quote").and_then(Value::as_str).map(str::to_string)),
            context: v.get("context").and_then(Value::as_str).map(str::to_string),
            data_kind: None,
            normalized: None,
            currency: None,
            unit: None,
            coords: DocumentCoords::from_envelope_value(v),
        }
    }

    pub fn from_extract_item(v: &Value) -> Self {
        Self {
            kind: EnvelopeKind::ExtractData,
            quote: v
                .get("raw")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| v.get("text").and_then(Value::as_str).map(str::to_string)),
            context: None,
            data_kind: v.get("kind").and_then(Value::as_str).map(str::to_string),
            normalized: v.get("normalized").cloned(),
            currency: v
                .get("currency")
                .and_then(Value::as_str)
                .map(str::to_string),
            unit: v.get("unit").and_then(Value::as_str).map(str::to_string),
            coords: DocumentCoords::from_envelope_value(v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchExtractEnvelope {
    pub kind: EnvelopeKind,
    pub hits: Vec<EnvelopeHit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_match_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omitted_count: Option<u64>,
}

impl SearchExtractEnvelope {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        if let Some(matches) = v.get("matches").and_then(Value::as_array) {
            return Ok(Self {
                kind: EnvelopeKind::Search,
                hits: matches.iter().map(EnvelopeHit::from_search_match).collect(),
                truncated: v.get("truncated").and_then(Value::as_bool),
                total_match_count: as_u64(v.get("totalMatchCount")),
                omitted_count: as_u64(v.get("omittedCount")),
            });
        }
        if let Some(items) = v.get("items").and_then(Value::as_array) {
            return Ok(Self {
                kind: EnvelopeKind::ExtractData,
                hits: items.iter().map(EnvelopeHit::from_extract_item).collect(),
                truncated: None,
                total_match_count: Some(items.len() as u64),
                omitted_count: None,
            });
        }
        Err("envelope is neither search.matches nor extract-data.items".into())
    }

    pub fn fully_bound_hits(&self) -> impl Iterator<Item = &EnvelopeHit> {
        self.hits.iter().filter(|h| h.coords.coords_present())
    }

    pub fn hit_with_required(&self, coords: &DocumentCoords) -> Option<&EnvelopeHit> {
        let want = coords.required_tuple()?;
        self.hits
            .iter()
            .find(|h| h.coords.required_tuple() == Some(want))
    }
}

fn as_u64(v: Option<&Value>) -> Option<u64> {
    v.and_then(Value::as_u64).or_else(|| {
        v.and_then(Value::as_i64)
            .and_then(|i| u64::try_from(i).ok())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_search_envelope_with_page() {
        let env = SearchExtractEnvelope::from_value(&json!({
            "matches": [{
                "text": "본 사업의 핵심은 도시 데이터 플랫폼 구축이다.",
                "context": "…본 사업의 핵심은 도시 데이터 플랫폼 구축이다.…",
                "section": 0,
                "paragraph": 4,
                "page": 1,
                "charOffset": 8,
                "length": 7
            }],
            "truncated": false,
            "totalMatchCount": 1,
            "omittedCount": 0
        }))
        .expect("search");
        assert_eq!(env.kind, EnvelopeKind::Search);
        assert_eq!(env.hits.len(), 1);
        assert!(env.hits[0].coords.coords_present());
        assert_eq!(env.hits[0].coords.page, Some(1));
        assert_eq!(env.hits[0].coords.char_offset, Some(8));
    }

    #[test]
    fn parse_extract_amount_envelope() {
        let env = SearchExtractEnvelope::from_value(&json!({
            "items": [{
                "kind": "amount",
                "raw": "3,180백만원",
                "normalized": 3180000000i64,
                "currency": "KRW",
                "section": 0,
                "paragraph": 7,
                "page": 0,
                "charOffset": 55,
                "length": 11
            }]
        }))
        .expect("extract");
        assert_eq!(env.kind, EnvelopeKind::ExtractData);
        assert_eq!(env.hits[0].data_kind.as_deref(), Some("amount"));
        assert_eq!(env.hits[0].coords.char_offset, Some(55));
    }

    #[test]
    fn parse_extract_date_envelope() {
        let env = SearchExtractEnvelope::from_value(&json!({
            "items": [{
                "kind": "date",
                "raw": "2026-09-12",
                "normalized": "2026-09-12",
                "section": 0,
                "paragraph": 1,
                "page": 0,
                "charOffset": 6,
                "length": 10
            }]
        }))
        .expect("extract");
        assert_eq!(env.hits[0].data_kind.as_deref(), Some("date"));
        assert!(env.hits[0].coords.coords_present());
    }

    #[test]
    fn parse_search_cell_coords() {
        let env = SearchExtractEnvelope::from_value(&json!({
            "matches": [{
                "text": "기술평가 80",
                "section": 0,
                "paragraph": 0,
                "page": 5,
                "charOffset": 0,
                "length": 6,
                "cell": {"row": 2, "col": 1}
            }],
            "truncated": false,
            "totalMatchCount": 1
        }))
        .expect("cell");
        assert!(env.hits[0].coords.cell.is_some());
        assert!(env.hits[0].coords.field_set().contains(&"cell".to_string()));
    }

    #[test]
    fn empty_object_is_not_an_envelope() {
        assert!(SearchExtractEnvelope::from_value(&json!({})).is_err());
    }
}
